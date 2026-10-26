#!/usr/bin/env python3
"""Build the pinned native dependencies used by Windows builds and Linux verification."""
import argparse
from pathlib import Path
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--zig", required=True)
    parser.add_argument("--target", required=True,
                        choices=("x86_64-windows-gnu", "x86_64-linux-gnu"))
    parser.add_argument("--prefix", type=Path, required=True)
    parser.add_argument("--library-directory", required=True)
    parser.add_argument("--sdl-include", required=True)
    args = parser.parse_args()
    tools = Path(__file__).resolve().parent
    common = ["--zig", args.zig, "--target", args.target,
              "--library-directory", args.library_directory]
    for component in ("cpu", "backend", "spc"):
        subprocess.run([sys.executable, str(tools / "install_lakesnes.py"), component,
                        *common, "--prefix", str(args.prefix / f"lakesnes-{component}")], check=True)
    components = ("config", "desktop", "music") if args.target == "x86_64-windows-gnu" else ("config", "music")
    for component in components:
        command = [sys.executable, str(tools / f"install_native_{component}.py"),
                   *common, "--prefix", str(args.prefix / component)]
        if component != "music":
            command += ["--sdl-include", args.sdl_include]
        subprocess.run(command, check=True)


if __name__ == "__main__":
    main()
