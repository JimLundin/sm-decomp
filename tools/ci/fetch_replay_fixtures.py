"""Install the ordinary replay fixtures in manifest order."""

import argparse
from pathlib import Path
import subprocess
import tomllib


HERE = Path(__file__).resolve().parent


def fetch(
    rom, destination, *, folder, asset, sha256, size, members, check_rom=False, parent=None
):
    qualification = []
    if check_rom:
        qualification = ["--rom", rom]
    elif parent is not None:
        qualification = ["--parent", str(Path(destination) / parent)]
    print(f"Fetching {folder}", flush=True)
    subprocess.run(
        ["bash", str(HERE / "fetch_native_fixture.sh"), rom, destination,
         folder, asset, sha256, str(size), str(members), *qualification],
        check=True,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rom")
    parser.add_argument("destination")
    parser.add_argument(
        "folder", nargs="?", help="install one ordinary fixture into an existing destination"
    )
    args = parser.parse_args()

    with (HERE / "replay_fixtures.toml").open("rb") as manifest:
        fixtures = {entry["folder"]: entry for entry in tomllib.load(manifest)["fixture"]}

    if args.folder is not None and args.folder not in fixtures:
        parser.error(f"unknown replay fixture {args.folder!r}")

    selected = fixtures.values() if args.folder is None else [fixtures[args.folder]]
    try:
        for entry in selected:
            fetch(args.rom, args.destination, **entry)
    except subprocess.CalledProcessError as error:
        return error.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
