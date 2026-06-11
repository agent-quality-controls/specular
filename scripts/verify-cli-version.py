#!/usr/bin/env python3

import subprocess
import sys
import tomllib
from pathlib import Path


def main() -> int:
    package_version = tomllib.loads(Path("Cargo.toml").read_text())["package"]["version"]
    completed = subprocess.run(
        ["cargo", "run", "--quiet", "--", "--version"],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    cli_version = completed.stdout.strip()

    if cli_version != package_version:
        print("FAIL cli-version")
        print(f"- Cargo.toml version: {package_version}")
        print(f"- specular --version: {cli_version}")
        return 1

    print("PASS cli-version")
    return 0


if __name__ == "__main__":
    sys.exit(main())
