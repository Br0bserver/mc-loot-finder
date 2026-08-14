#!/usr/bin/env python3

import json
import os
from pathlib import Path
import subprocess
import sys
import zipfile


EXPECTED_STRUCTURES = {
    "ancient_city",
    "bastion_remnant",
    "desert_pyramid",
    "jungle_pyramid",
    "igloo",
    "end_city",
    "ruined_portal",
    "ruined_portal_nether",
    "trial_chambers",
    "shipwreck",
    "ocean_ruin",
    "nether_fortress",
    "village",
    "buried_treasure",
    "pillager_outpost",
    "woodland_mansion",
}


def run_json(cli: Path, *arguments: str) -> dict:
    if os.name == "nt":
        command = ["cmd.exe", "/d", "/c", str(cli), *arguments]
    else:
        command = [str(cli), *arguments]
    result = subprocess.run(command, text=True, capture_output=True)
    if result.returncode != 0:
        raise AssertionError(
            f"CLI exited with status {result.returncode}.\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise AssertionError(
            f"CLI returned invalid JSON.\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        ) from error


def require_fields(actual: dict, expected: dict, label: str) -> None:
    selected = {key: actual.get(key) for key in expected}
    if selected != expected:
        raise AssertionError(f"unexpected {label}: {actual}")


def verify_no_bundled_mojang_runtime(distribution_root: Path) -> None:
    for archive in distribution_root.rglob("*.jar"):
        if archive.name.startswith("minecraft-"):
            raise AssertionError(f"distribution contains a Minecraft jar: {archive}")
        with zipfile.ZipFile(archive) as jar:
            bundled = next(
                (
                    name
                    for name in jar.namelist()
                    if name.endswith(".class")
                    and (name.startswith("net/minecraft/")
                         or name.startswith("com/mojang/"))
                ),
                None,
            )
        if bundled is not None:
            raise AssertionError(
                f"distribution contains a Mojang class in {archive}: {bundled}"
            )


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: verify_distribution.py PATH_TO_CLI")
    cli = Path(sys.argv[1]).resolve()
    if not cli.is_file():
        raise SystemExit(f"CLI does not exist: {cli}")
    distribution_root = cli.parent.parent
    verify_no_bundled_mojang_runtime(distribution_root)
    for required_file in ("LICENSE", "README.md"):
        locations = (
            distribution_root / required_file,
            distribution_root / "lib" / required_file,
        )
        if not any(location.is_file() for location in locations):
            raise AssertionError(f"distribution is missing {required_file}")

    catalog = run_json(cli, "explain", "--json")
    structures = {entry["name"] for entry in catalog["structures"]}
    if structures != EXPECTED_STRUCTURES or len(catalog["structures"]) != 16:
        raise AssertionError(f"unexpected structure catalog: {catalog}")

    ancient_city = run_json(
        cli,
        "find",
        "--seed",
        "114514",
        "--structure",
        "ancient_city",
        "--item",
        "minecraft:silence_armor_trim_smithing_template",
        "--radius",
        "5000",
        "--json",
    )
    require_fields(
        ancient_city,
        {
            "placement_candidates": 530,
            "valid_structures": 17,
            "checked_chests": 358,
            "hits": 1,
            "unpredictable_zero_seeds": 0,
        },
        "ancient city search",
    )
    expected_match = {
        "x": 3965,
        "y": -37,
        "z": 2755,
        "loot_table": "minecraft:chests/ancient_city",
        "loot_seed": -5503126436529563106,
        "start_chunk_x": 244,
        "start_chunk_z": 171,
    }
    if ancient_city["matches"] != [expected_match]:
        raise AssertionError(f"unexpected ancient city match: {ancient_city}")

    bastion = run_json(
        cli,
        "chests",
        "--seed",
        "0",
        "--structure",
        "bastion_remnant",
        "--center-x",
        "1000",
        "--center-z",
        "520",
        "--radius",
        "0",
        "--json",
    )
    require_fields(
        bastion,
        {
            "placement_candidates": 1,
            "valid_structures": 1,
            "chest_count": 6,
        },
        "bastion container scan",
    )
    if {chest["loot_table"] for chest in bastion["chests"]} != {
        "minecraft:chests/bastion_other",
        "minecraft:chests/bastion_treasure",
    }:
        raise AssertionError(f"unexpected bastion loot tables: {bastion}")

    print("verified 16 structures, ancient city loot search, and bastion containers")


if __name__ == "__main__":
    main()
