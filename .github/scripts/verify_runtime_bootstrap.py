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


def read_properties(path: Path) -> dict[str, str]:
    result = {}
    for line in path.read_text(encoding="iso-8859-1").splitlines():
        if line and not line.startswith(("#", "!")) and "=" in line:
            key, value = line.split("=", 1)
            result[key] = value
    return result


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
    properties = read_properties(runtime / "runtime.properties")
    library_count = int(properties["library.count"])
    if library_count != 17:
        raise AssertionError(f"unexpected generated library count: {library_count}")
    library_paths = {
        properties[f"library.{index}.path"] for index in range(library_count)
    }
    forbidden_fragments = (
        "com/azure/",
        "com/microsoft/",
        "com/google/code/gson/",
        "com/github/oshi/",
        "net/java/dev/jna/",
        "net/sf/jopt-simple/",
        "native-epoll",
        "native-kqueue",
    )
    unexpected = sorted(
        path for path in library_paths if any(part in path for part in forbidden_fragments)
    )
    if unexpected:
        raise AssertionError(f"generated runtime contains excluded libraries: {unexpected}")
    runtime_size = sum(path.stat().st_size for path in runtime.rglob("*") if path.is_file())
    if runtime_size >= 58_000_000:
        raise AssertionError(f"generated runtime is unexpectedly large: {runtime_size}")

    print(
        "verified official 26.1.2 download, 17-library runtime, cache, and hashes"
    )


if __name__ == "__main__":
    main()
