#!/usr/bin/env python3
"""Cross-platform behavioral smoke checks for the release CLI."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any


FULL_SCAN_STRUCTURES = [
    "ancient_city",
    "bastion_remnant",
    "desert_pyramid",
    "igloo",
    "shipwreck",
    "village",
    "buried_treasure",
    "pillager_outpost",
]


def run(binary: Path, *args: str, expected_status: int = 0) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        [str(binary), *args],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != expected_status:
        raise SystemExit(
            f"{' '.join(args)} exited with {result.returncode}, expected {expected_status}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    return result


def run_json(binary: Path, *args: str, expected_status: int = 0) -> dict[str, Any]:
    result = run(binary, *args, expected_status=expected_status)
    try:
        parsed = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise SystemExit(
            f"{' '.join(args)} returned invalid JSON: {error}\n{result.stdout}"
        ) from error
    if not isinstance(parsed, dict):
        raise SystemExit(f"{' '.join(args)} returned non-object JSON: {parsed!r}")
    return parsed


def assert_fields(label: str, result: dict[str, Any], expected: dict[str, int]) -> None:
    actual = {key: result.get(key) for key in expected}
    if actual != expected:
        raise SystemExit(f"unexpected {label}: expected {expected}, got {result}")


def check_capabilities(binary: Path) -> None:
    result = run_json(binary, "explain", "--json")
    structures = result.get("structures")
    if not isinstance(structures, list):
        raise SystemExit(f"explain returned invalid structures list: {result}")
    capabilities: list[tuple[str, bool]] = []
    for structure in structures:
        if not isinstance(structure, dict):
            raise SystemExit(f"explain returned invalid structure entry: {structure!r}")
        name = structure.get("name")
        supports_full_scan = structure.get("full_scan")
        if not isinstance(name, str) or not isinstance(supports_full_scan, bool):
            raise SystemExit(f"explain returned invalid structure capability: {structure}")
        capabilities.append((name, supports_full_scan))

    full_scan = [name for name, supports_full_scan in capabilities if supports_full_scan]
    if full_scan != FULL_SCAN_STRUCTURES:
        raise SystemExit(
            f"unexpected full-scan capability list: expected {FULL_SCAN_STRUCTURES}, got {full_scan}"
        )

    for name, supports_full_scan in capabilities:
        if supports_full_scan:
            continue
        result = run(
            binary,
            "chests",
            "--seed",
            "0",
            "--structure",
            name,
            "--radius",
            "0",
            "--json",
            expected_status=2,
        )
        error = result.stdout + result.stderr
        if f"do not support {name}" not in error:
            raise SystemExit(f"unexpected {name} fail-closed error: {error}")


def check_ancient_city(binary: Path, ancient_output: Path | None) -> None:
    if ancient_output:
        chests = run_json(
            binary,
            "chests",
            "--seed",
            "114514",
            "--structure",
            "ancient_city",
            "--radius",
            "5000",
            "--limit",
            "1000",
            "--json",
        )
        assert_fields(
            "ancient city chests",
            chests,
            {"placement_candidates": 530, "valid_structures": 17, "chest_count": 368},
        )
        ancient_output.write_text(json.dumps(chests, separators=(",", ":")) + "\n")

    found = run_json(
        binary,
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
    assert_fields(
        "ancient city find",
        found,
        {
            "placement_candidates": 530,
            "valid_structures": 17,
            "checked_chests": 358,
            "hits": 1,
        },
    )
    run(
        binary,
        "find",
        "--seed",
        "114514",
        "--structure",
        "ancient_city",
        "--item",
        "minecraft:bedrock",
        "--radius",
        "0",
        "--json",
        expected_status=1,
    )


def check_bastion(binary: Path) -> None:
    result = run_json(
        binary,
        "find",
        "--seed",
        "0",
        "--structure",
        "bastion_remnant",
        "--item",
        "minecraft:netherite_upgrade_smithing_template",
        "--center-x",
        "1000",
        "--center-z",
        "520",
        "--radius",
        "0",
        "--json",
    )
    assert_fields(
        "bastion find",
        result,
        {"placement_candidates": 1, "valid_structures": 1, "checked_chests": 6, "hits": 1},
    )


def check_structure_pair(
    binary: Path,
    structure: str,
    item: str,
    chests_expected: dict[str, int],
    find_expected: dict[str, int],
) -> dict[str, Any]:
    chests = run_json(
        binary,
        "chests",
        "--seed",
        "0",
        "--structure",
        structure,
        "--radius",
        "5000",
        "--limit",
        "1000",
        "--json",
    )
    assert_fields(f"{structure} chests", chests, chests_expected)
    found = run_json(
        binary,
        "find",
        "--seed",
        "0",
        "--structure",
        structure,
        "--item",
        item,
        "--radius",
        "5000",
        "--limit",
        "20",
        "--json",
    )
    assert_fields(f"{structure} find", found, find_expected)
    return chests


def check_shipwreck(binary: Path) -> None:
    search = [
        "--seed",
        "0",
        "--structure",
        "shipwreck",
        "--center-x",
        "232",
        "--center-z",
        "136",
        "--radius",
        "0",
        "--json",
    ]
    chests = run_json(binary, "chests", *search)
    assert_fields(
        "shipwreck chests",
        chests,
        {"placement_candidates": 1, "valid_structures": 1, "chest_count": 3},
    )
    expected_chests = [
        {
            "x": 219,
            "y": 60,
            "z": 142,
            "loot_table": "minecraft:chests/shipwreck_treasure",
            "loot_seed": -756_378_412_031_281_064,
            "start_chunk_x": 14,
            "start_chunk_z": 8,
            "ordinal": 0,
        },
        {
            "x": 235,
            "y": 61,
            "z": 144,
            "loot_table": "minecraft:chests/shipwreck_supply",
            "loot_seed": -3_774_492_170_699_737_302,
            "start_chunk_x": 14,
            "start_chunk_z": 8,
            "ordinal": 0,
        },
        {
            "x": 224,
            "y": 61,
            "z": 145,
            "loot_table": "minecraft:chests/shipwreck_map",
            "loot_seed": -2_986_182_992_758_690_057,
            "start_chunk_x": 14,
            "start_chunk_z": 8,
            "ordinal": 1,
        },
    ]
    if chests.get("chests") != expected_chests:
        raise SystemExit(f"unexpected shipwreck chests: expected {expected_chests}, got {chests}")

    found = run_json(
        binary,
        "find",
        *search[:-1],
        "--item",
        "minecraft:map",
        "--json",
    )
    assert_fields(
        "shipwreck find",
        found,
        {"placement_candidates": 1, "valid_structures": 1, "checked_chests": 3, "hits": 1},
    )

    unavailable = run(
        binary,
        "container-seed",
        "--seed",
        "0",
        "--structure",
        "shipwreck",
        "--chunk-x",
        "14",
        "--chunk-z",
        "8",
        expected_status=2,
    )
    unavailable_error = unavailable.stdout + unavailable.stderr
    if "container-seed is not available for shipwreck" not in unavailable_error:
        raise SystemExit(f"unexpected shipwreck container-seed error: {unavailable_error}")


def check_buried_treasure(binary: Path) -> None:
    search = [
        "--seed",
        "0",
        "--structure",
        "buried_treasure",
        "--center-x",
        "8",
        "--center-z",
        "-344",
        "--radius",
        "0",
        "--json",
    ]
    chests = run_json(binary, "chests", *search)
    assert_fields(
        "buried treasure chests",
        chests,
        {"placement_candidates": 1, "valid_structures": 1, "chest_count": 1},
    )
    entries = chests.get("chests")
    if not isinstance(entries, list) or len(entries) != 1 or not isinstance(entries[0], dict):
        raise SystemExit(f"buried treasure returned invalid chest list: {chests}")
    chest = entries[0]
    expected_chest = {
        "x": 9,
        "y": 64,
        "z": -343,
        "loot_table": "minecraft:chests/buried_treasure",
        "loot_seed": -2_156_648_588_641_602_659,
        "start_chunk_x": 0,
        "start_chunk_z": -22,
        "ordinal": 0,
    }
    if chest != expected_chest:
        raise SystemExit(f"unexpected buried treasure chest: expected {expected_chest}, got {chest}")

    found = run_json(
        binary,
        "find",
        *search[:-1],
        "--item",
        "minecraft:heart_of_the_sea",
        "--json",
    )
    assert_fields(
        "buried treasure find",
        found,
        {"placement_candidates": 1, "valid_structures": 1, "checked_chests": 1, "hits": 1},
    )

    predicted = run_json(
        binary,
        "container-seed",
        "--seed",
        "0",
        "--structure",
        "buried_treasure",
        "--chunk-x",
        "0",
        "--chunk-z",
        "-22",
        "--ordinal",
        "0",
        "--json",
    )
    if (
        predicted.get("loot_table_seed") != expected_chest["loot_seed"]
        or predicted.get("structure_index") != 0
        or predicted.get("step") != 3
    ):
        raise SystemExit(f"buried treasure seed contract mismatch: {predicted}")


def check_pillager_seed_contract(binary: Path, chests: dict[str, Any]) -> None:
    entries = chests.get("chests")
    if not isinstance(entries, list):
        raise SystemExit(f"pillager chests output has no chest list: {chests}")
    chest = next(
        (
            entry
            for entry in entries
            if isinstance(entry, dict)
            and (entry.get("x"), entry.get("y"), entry.get("z")) == (-826, 77, 1110)
        ),
        None,
    )
    if not isinstance(chest, dict):
        raise SystemExit(f"known pillager chest is missing: {chests}")
    expected_seed = chest.get("loot_seed")
    ordinal = chest.get("ordinal")
    if not isinstance(expected_seed, int) or not isinstance(ordinal, int):
        raise SystemExit(f"known pillager chest has invalid seed metadata: {chest}")

    predicted = run_json(
        binary,
        "container-seed",
        "--seed",
        "0",
        "--structure",
        "pillager_outpost",
        "--chunk-x",
        str(-826 // 16),
        "--chunk-z",
        str(1110 // 16),
        "--ordinal",
        str(ordinal),
        "--json",
    )
    if predicted.get("loot_table_seed") != expected_seed:
        raise SystemExit(
            f"pillager seed contract mismatch: chest {expected_seed}, container-seed {predicted}"
        )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--ancient-output", type=Path)
    args = parser.parse_args()

    binary = args.binary.resolve()
    if not binary.is_file():
        raise SystemExit(f"binary does not exist: {binary}")

    check_capabilities(binary)
    check_ancient_city(binary, args.ancient_output)
    check_bastion(binary)
    check_structure_pair(
        binary,
        "desert_pyramid",
        "minecraft:dune_armor_trim_smithing_template",
        {"placement_candidates": 307, "valid_structures": 3, "chest_count": 12},
        {"placement_candidates": 307, "valid_structures": 3, "checked_chests": 12, "hits": 2},
    )
    check_structure_pair(
        binary,
        "igloo",
        "minecraft:golden_apple",
        {"placement_candidates": 299, "valid_structures": 22, "chest_count": 10},
        {"placement_candidates": 299, "valid_structures": 22, "checked_chests": 10, "hits": 10},
    )
    check_shipwreck(binary)
    check_structure_pair(
        binary,
        "village",
        "minecraft:diamond",
        {"placement_candidates": 267, "valid_structures": 60, "chest_count": 155},
        {"placement_candidates": 267, "valid_structures": 60, "checked_chests": 155, "hits": 3},
    )
    check_buried_treasure(binary)
    pillager_chests = check_structure_pair(
        binary,
        "pillager_outpost",
        "minecraft:sentry_armor_trim_smithing_template",
        {"placement_candidates": 298, "valid_structures": 10, "chest_count": 10},
        {"placement_candidates": 298, "valid_structures": 10, "checked_chests": 10, "hits": 1},
    )
    check_pillager_seed_contract(binary, pillager_chests)


if __name__ == "__main__":
    main()
