#!/usr/bin/env python3

import json
import os
from pathlib import Path
import subprocess
import sys


EXPECTED_COUNTS = {
    "ancient_city": 17,
    "bastion_remnant": 6,
    "desert_pyramid": 4,
    "jungle_pyramid": 4,
    "igloo": 1,
    "end_city": 2,
    "ruined_portal": 1,
    "ruined_portal_nether": 1,
    "trial_chambers": 95,
    "shipwreck": 3,
    "ocean_ruin": 1,
    "nether_fortress": 6,
    "village": 5,
    "buried_treasure": 1,
    "pillager_outpost": 1,
    "woodland_mansion": 7,
}


def command(cli: Path, *arguments: str) -> list[str]:
    result = [str(cli), *arguments]
    if os.name == "nt":
        return ["cmd.exe", "/d", "/c", *result]
    return result


def run_probe(cli: Path, engine: str) -> dict:
    result = subprocess.run(
        command(cli, "runtime", "probe", "--engine", engine, "--json"),
        text=True,
        capture_output=True,
        env=os.environ.copy(),
    )
    if result.returncode != 0:
        raise AssertionError(
            f"{engine} CLI exited with status {result.returncode}.\n"
            f"command: {result.args}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise AssertionError(
            f"{engine} CLI returned invalid JSON.\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        ) from error


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit(
            "usage: verify_engine_parity.py PATH_TO_ORACLE_CLI PATH_TO_SUBSET_CLI"
        )
    oracle_cli = Path(sys.argv[1]).resolve()
    subset_cli = Path(sys.argv[2]).resolve()
    for cli in (oracle_cli, subset_cli):
        if not cli.is_file():
            raise SystemExit(f"CLI does not exist: {cli}")

    oracle = run_probe(oracle_cli, "oracle")
    subset = run_probe(subset_cli, "subset")
    oracle.pop("engine", None)
    subset.pop("engine", None)
    if subset != oracle:
        raise AssertionError(
            "subset runtime probe differs from the oracle.\n"
            f"oracle:\n{json.dumps(oracle, indent=2, sort_keys=True)}\n"
            f"subset:\n{json.dumps(subset, indent=2, sort_keys=True)}"
        )

    vectors = {vector["structure"]: vector for vector in oracle["vectors"]}
    if set(vectors) != set(EXPECTED_COUNTS):
        raise AssertionError(f"unexpected runtime probe structures: {sorted(vectors)}")
    for structure, expected_chests in EXPECTED_COUNTS.items():
        vector = vectors[structure]
        if not vector["valid_structure"]:
            raise AssertionError(f"runtime probe structure is absent: {structure}")
        if len(vector["containers"]) != expected_chests:
            raise AssertionError(
                f"oracle vector drifted for {structure}: "
                f"expected {expected_chests}, got {len(vector['containers'])}"
            )
        print(f"verified {structure}: {expected_chests} containers")

    print("verified exact oracle/subset parity for all 16 structures")


if __name__ == "__main__":
    main()
