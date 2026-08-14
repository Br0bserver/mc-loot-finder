#!/usr/bin/env python3

import json
import os
from pathlib import Path
import subprocess
import sys


def run(cli: Path, cache: Path, *arguments: str) -> dict:
    environment = os.environ.copy()
    environment["MC_LOOT_FINDER_CACHE"] = str(cache)
    command = [str(cli), *arguments]
    if os.name == "nt":
        command = ["cmd.exe", "/d", "/c", *command]
    result = subprocess.run(
        command,
        check=True,
        text=True,
        capture_output=True,
        env=environment,
    )
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise AssertionError(
            f"runtime command returned invalid JSON.\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        ) from error


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: verify_runtime_bootstrap.py PATH_TO_CLI CACHE_DIRECTORY")
    cli = Path(sys.argv[1]).resolve()
    cache = Path(sys.argv[2]).resolve()

    installed = run(cli, cache, "runtime", "install", "--json")
    if not installed["source_ready"] or not installed["generated_runtime_ready"]:
        raise AssertionError(f"unexpected install status: {installed}")

    status = run(cli, cache, "runtime", "status", "--json")
    if status != installed:
        raise AssertionError(f"runtime status changed after installation: {status}")

    verified = run(cli, cache, "runtime", "verify", "--json")
    if verified != {
        "version": "26.1.2",
        "source_valid": True,
        "runtime_valid": True,
    }:
        raise AssertionError(f"unexpected verification result: {verified}")

    source = cache / "26.1.2" / "source"
    expected_sizes = {
        "server.jar": 60_417_480,
        "server-inner.jar": 24_555_215,
    }
    actual_sizes = {name: (source / name).stat().st_size for name in expected_sizes}
    if actual_sizes != expected_sizes:
        raise AssertionError(f"unexpected cached source sizes: {actual_sizes}")

    runtime = cache / "26.1.2" / "runtime"
    if (runtime / "server.jar").stat().st_size != 24_555_215:
        raise AssertionError("generated runtime has an unexpected server jar")
    if not any((runtime / "libraries").rglob("*.jar")):
        raise AssertionError("generated runtime contains no official server libraries")

    print("verified official 26.1.2 download, generated runtime, cache, and hashes")


if __name__ == "__main__":
    main()
