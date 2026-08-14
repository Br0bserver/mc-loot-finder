#!/usr/bin/env python3

import os
from pathlib import Path
import shutil
import subprocess
import sys


MAIN_CLASS = "dev.br0b.mclootfinder.cli.Main"


def tool(name: str) -> Path:
    executable = name + (".exe" if os.name == "nt" else "")
    java_home = os.environ.get("JAVA_HOME")
    if java_home:
        candidate = Path(java_home) / "bin" / executable
        if candidate.is_file():
            return candidate
    resolved = shutil.which(executable)
    if resolved is None:
        raise SystemExit(f"JDK tool is not available: {executable}")
    return Path(resolved)


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: build_app_image.py INSTALL_DIRECTORY OUTPUT_DIRECTORY")
    install = Path(sys.argv[1]).resolve()
    output = Path(sys.argv[2]).resolve()
    libraries = install / "lib"
    application_jars = sorted(libraries.glob("mc-loot-finder-*.jar"))
    if len(application_jars) != 1:
        raise SystemExit(f"expected one application jar, found: {application_jars}")
    application_jar = application_jars[0]

    classpath = os.pathsep.join(str(path) for path in sorted(libraries.glob("*.jar")))
    dependency_result = subprocess.run(
        [
            str(tool("jdeps")),
            "--ignore-missing-deps",
            "--multi-release",
            "25",
            "--print-module-deps",
            "--class-path",
            classpath,
            str(application_jar),
        ],
        check=True,
        text=True,
        capture_output=True,
    )
    reported_modules = dependency_result.stdout.strip()
    if not reported_modules:
        raise SystemExit("jdeps did not report any runtime modules")
    module_set = set(reported_modules.split(","))
    # Loom's full development classpath includes server paths that the generated
    # Worldgen runtime removes. Runtime probes confirm these modules are not loaded.
    module_set.difference_update(
        {"java.compiler", "java.rmi", "java.scripting", "jdk.httpserver"}
    )
    module_set.update({"jdk.crypto.ec", "jdk.unsupported", "jdk.zipfs"})
    modules = ",".join(sorted(module_set))

    work = output.parent / (output.name + "-work")
    runtime = work / "runtime"
    application_input = work / "input"
    shutil.rmtree(output, ignore_errors=True)
    shutil.rmtree(work, ignore_errors=True)
    application_input.mkdir(parents=True)

    selected = [application_jar]
    for pattern in ("runtime-api*.jar", "gson-*.jar", "objenesis-*.jar"):
        matches = sorted(libraries.glob(pattern))
        if len(matches) != 1:
            raise SystemExit(f"expected one {pattern}, found: {matches}")
        selected.append(matches[0])
    for source in selected:
        shutil.copy2(source, application_input / source.name)

    subprocess.run(
        [
            str(tool("jlink")),
            "--add-modules",
            modules,
            "--strip-debug",
            "--no-header-files",
            "--no-man-pages",
            "--compress",
            "zip-9",
            "--output",
            str(runtime),
        ],
        check=True,
    )

    command = [
        str(tool("jpackage")),
        "--type",
        "app-image",
        "--name",
        "mc-loot-finder",
        "--app-version",
        "0.2.0",
        "--vendor",
        "mc-loot-finder",
        "--description",
        "Minecraft Java structure container and loot finder",
        "--input",
        str(application_input),
        "--main-jar",
        application_jar.name,
        "--main-class",
        MAIN_CLASS,
        "--runtime-image",
        str(runtime),
        "--java-options",
        "--sun-misc-unsafe-memory-access=allow",
        "--java-options",
        "-Dmclootfinder.engine=subset",
        "--app-content",
        str(Path("LICENSE").resolve()),
        "--app-content",
        str(Path("README.md").resolve()),
        "--dest",
        str(work / "image"),
    ]
    if os.name == "nt":
        command.append("--win-console")
    subprocess.run(command, check=True)

    generated = work / "image" / "mc-loot-finder"
    generated.replace(output)
    shutil.rmtree(work, ignore_errors=True)
    print(
        "built app image with "
        + ", ".join(path.name for path in selected)
        + f" and modules: {modules}"
    )


if __name__ == "__main__":
    main()
