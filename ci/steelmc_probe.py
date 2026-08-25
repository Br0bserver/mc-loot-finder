#!/usr/bin/env python3
"""Compare one buried-treasure location and loot seed across both backends."""

from __future__ import annotations

import argparse
import json
from compression import zstd
import os
import queue
import re
import subprocess
import tempfile
import threading
import time
from pathlib import Path
from typing import Any

DEFAULT_SEED = 215
SEARCH_RADIUS = 512
START_TIMEOUT_SECONDS = 60
LOCATE_TIMEOUT_SECONDS = 60
REGION_SIZE = 32
SECTOR_SIZE = 4096
REGION_MAGIC = b"STLR"
LOOT_TABLE = b"minecraft:chests/buried_treasure"
LOOT_TABLE_TAG = (
    b"\x08"
    + len(b"LootTable").to_bytes(2, "big")
    + b"LootTable"
    + len(LOOT_TABLE).to_bytes(2, "big")
    + LOOT_TABLE
)
LOOT_SEED_TAG = (
    b"\x04"
    + len(b"LootTableSeed").to_bytes(2, "big")
    + b"LootTableSeed"
)
SPAWN_PATTERN = re.compile(
    r"spawn initialized at BlockPos\(IVec3\((-?\d+), -?\d+, (-?\d+)\)\)"
)
LOCATE_PATTERN = re.compile(
    r"nearest minecraft:buried_treasure is at \[(-?\d+), ~, (-?\d+)\]"
)


def world_config(seed: int) -> str:
    return f'''save_path = "saves"
seed = "{seed}"
default_gamemode = "survival"
difficulty = "normal"

[storage]
type = "steel:disk"

[player_storage]
type = "steel:file"

[domains.minecraft]
default = true

[[domains.minecraft.worlds]]
name = "overworld"
generator = "minecraft:overworld"
default = true

[[domains.minecraft.worlds]]
name = "the_nether"
generator = "minecraft:the_nether"

[[domains.minecraft.worlds]]
name = "the_end"
generator = "minecraft:the_end"
'''


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--steel", required=True, type=Path, help="SteelMC 26.2 binary")
    parser.add_argument(
        "--loot-finder", required=True, type=Path, help="mc-loot-finder 26.1.2 binary"
    )
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    return parser.parse_args()


def resolve_binary(path: Path) -> Path:
    resolved = path.expanduser().resolve()
    if not resolved.is_file():
        raise SystemExit(f"binary does not exist: {resolved}")
    return resolved


def run_json(binary: Path, *args: str) -> dict[str, Any]:
    result = subprocess.run(
        [str(binary), *args], check=False, capture_output=True, text=True
    )
    if result.returncode != 0:
        raise SystemExit(
            f"mc-loot-finder exited with {result.returncode}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    try:
        value = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise SystemExit(f"mc-loot-finder returned invalid JSON: {error}") from error
    if not isinstance(value, dict):
        raise SystemExit(f"mc-loot-finder returned non-object JSON: {value!r}")
    return value


def wait_for_line(
    lines: queue.Queue[str],
    transcript: list[str],
    pattern: re.Pattern[str],
    timeout: int,
) -> re.Match[str]:
    deadline = time.monotonic() + timeout
    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise SystemExit(
                f"timed out waiting for {pattern.pattern!r}\nSteelMC output:\n"
                + "".join(transcript)
            )
        try:
            line = lines.get(timeout=remaining)
        except queue.Empty as error:
            raise SystemExit(
                f"timed out waiting for {pattern.pattern!r}\nSteelMC output:\n"
                + "".join(transcript)
            ) from error
        transcript.append(line)
        match = pattern.search(line)
        if match:
            return match


def start_reader(stream: Any, lines: queue.Queue[str]) -> threading.Thread:
    def read_lines() -> None:
        try:
            for line in stream:
                lines.put(line)
        except OSError:
            pass

    thread = threading.Thread(target=read_lines, daemon=True)
    thread.start()
    return thread


def nearest_loot_finder_chest(
    binary: Path, seed: int, center_x: int, center_z: int
) -> tuple[int, int, int, int]:
    result = run_json(
        binary,
        "chests",
        "--seed",
        str(seed),
        "--structure",
        "buried_treasure",
        "--center-x",
        str(center_x),
        "--center-z",
        str(center_z),
        "--radius",
        str(SEARCH_RADIUS),
        "--json",
    )
    chests = result.get("chests")
    if not isinstance(chests, list) or not chests:
        raise SystemExit(f"mc-loot-finder found no buried treasure: {result}")

    parsed_chests: list[tuple[int, int, int, int]] = []
    try:
        for chest in chests:
            if not isinstance(chest, dict):
                raise TypeError(f"non-object chest: {chest!r}")
            parsed_chests.append(
                (
                    int(chest["x"]),
                    int(chest["y"]),
                    int(chest["z"]),
                    int(chest["loot_seed"]),
                )
            )
    except (KeyError, TypeError, ValueError) as error:
        raise SystemExit(f"mc-loot-finder returned an invalid chest: {error}") from error

    return min(
        parsed_chests,
        key=lambda chest: (chest[0] - center_x) ** 2
        + (chest[2] - center_z) ** 2,
    )


def stop_server(process: subprocess.Popen[bytes], master_fd: int) -> None:
    if process.poll() is not None:
        return
    try:
        os.write(master_fd, b"stop\r")
    except OSError:
        pass
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


def read_chunk_payload(world_root: Path, block_x: int, block_z: int) -> bytes:
    chunk_x = block_x // 16
    chunk_z = block_z // 16
    region_x = chunk_x // REGION_SIZE
    region_z = chunk_z // REGION_SIZE
    local_x = chunk_x % REGION_SIZE
    local_z = chunk_z % REGION_SIZE
    region_path = (
        world_root
        / "saves"
        / "minecraft"
        / "worlds"
        / "overworld"
        / "region"
        / f"r.{region_x}.{region_z}.srg"
    )
    try:
        region = region_path.read_bytes()
    except OSError as error:
        raise SystemExit(f"failed to read SteelMC region {region_path}: {error}") from error
    if region[:4] != REGION_MAGIC:
        raise SystemExit(f"invalid SteelMC region magic in {region_path}")

    entry_index = local_z * REGION_SIZE + local_x
    entry_start = 8 + entry_index * 8
    entry = region[entry_start : entry_start + 8]
    if len(entry) != 8:
        raise SystemExit(f"missing SteelMC chunk table entry {chunk_x},{chunk_z}")
    sector_offset = int.from_bytes(entry[:4], "little")
    compressed_size = int.from_bytes(entry[4:7] + b"\0", "little")
    if sector_offset == 0 or compressed_size == 0:
        raise SystemExit(f"SteelMC did not save chunk {chunk_x},{chunk_z}")

    payload_start = sector_offset * SECTOR_SIZE
    payload_end = payload_start + compressed_size
    compressed = region[payload_start:payload_end]
    if len(compressed) != compressed_size:
        raise SystemExit(f"truncated SteelMC chunk payload {chunk_x},{chunk_z}")
    try:
        return zstd.decompress(compressed)
    except zstd.ZstdError as error:
        raise SystemExit(f"failed to decompress SteelMC chunk: {error}") from error


def read_buried_treasure_seed(payload: bytes) -> int:
    table_offset = payload.find(LOOT_TABLE_TAG)
    if table_offset < 0:
        raise SystemExit("SteelMC chunk has no buried-treasure LootTable NBT")
    seed_tag_offset = table_offset + len(LOOT_TABLE_TAG)
    if not payload.startswith(LOOT_SEED_TAG, seed_tag_offset):
        raise SystemExit("SteelMC buried-treasure NBT has no adjacent LootTableSeed")
    seed_offset = seed_tag_offset + len(LOOT_SEED_TAG)
    seed_bytes = payload[seed_offset : seed_offset + 8]
    if len(seed_bytes) != 8:
        raise SystemExit("SteelMC buried-treasure LootTableSeed is truncated")
    return int.from_bytes(seed_bytes, "big", signed=True)


def probe(steel: Path, loot_finder: Path, seed: int) -> None:
    with tempfile.TemporaryDirectory(prefix="steelmc-probe-") as temporary_directory:
        root = Path(temporary_directory)
        config = root / "config"
        config.mkdir()
        (config / "worlds.toml").write_text(world_config(seed), encoding="utf-8")

        master_fd, slave_fd = os.openpty()
        try:
            process = subprocess.Popen(
                [str(steel)],
                cwd=root,
                stdin=slave_fd,
                stdout=slave_fd,
                stderr=slave_fd,
                close_fds=True,
            )
        finally:
            os.close(slave_fd)

        output = os.fdopen(
            os.dup(master_fd), "r", encoding="utf-8", errors="replace", newline=""
        )
        lines: queue.Queue[str] = queue.Queue()
        transcript: list[str] = []
        reader = start_reader(output, lines)
        try:
            spawn = wait_for_line(
                lines, transcript, SPAWN_PATTERN, START_TIMEOUT_SECONDS
            )
            center_x, center_z = int(spawn.group(1)), int(spawn.group(2))
            expected_x, expected_y, expected_z, expected_seed = (
                nearest_loot_finder_chest(loot_finder, seed, center_x, center_z)
            )

            os.write(
                master_fd, b"locate structure minecraft:buried_treasure\r"
            )
            located = wait_for_line(
                lines, transcript, LOCATE_PATTERN, LOCATE_TIMEOUT_SECONDS
            )
            actual = int(located.group(1)), int(located.group(2))
            expected = expected_x, expected_z
            if actual != expected:
                raise SystemExit(
                    "buried-treasure location mismatch: "
                    f"SteelMC={actual}, mc-loot-finder={expected}, "
                    f"spawn=({center_x}, {center_z})"
                )

            stop_server(process, master_fd)
            payload = read_chunk_payload(root, expected_x, expected_z)
            actual_seed = read_buried_treasure_seed(payload)
            if actual_seed != expected_seed:
                raise SystemExit(
                    "buried-treasure LootTableSeed mismatch: "
                    f"SteelMC={actual_seed}, mc-loot-finder={expected_seed}"
                )

            print(
                f"PASS: seed {seed} spawn=({center_x}, {center_z}) "
                f"location=({actual[0]}, ~, {actual[1]}) "
                f"mc-loot-finder-y={expected_y} LootTableSeed={actual_seed}"
            )
            print("Scope: X/Z location, LootTable NBT, and LootTableSeed checked.")
        finally:
            stop_server(process, master_fd)
            os.close(master_fd)
            output.close()
            reader.join(timeout=1)


def main() -> None:
    args = parse_args()
    probe(resolve_binary(args.steel), resolve_binary(args.loot_finder), args.seed)


if __name__ == "__main__":
    main()
