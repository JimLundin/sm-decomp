import argparse
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path

HERE = Path(__file__).resolve().parent


@dataclass(frozen=True)
class Fixture:
    folder: str
    asset: str
    sha256: str
    size: int
    members: int
    check_rom: bool
    parent: str | None


def decode_fixture(entry: object) -> Fixture:
    match entry:
        case {
            "folder": str(folder),
            "asset": str(asset),
            "sha256": str(sha256),
            "size": int(size),
            "members": int(members),
            **qualification,
        } if type(size) is int and type(members) is int and size > 0 and members > 0:
            match qualification:
                case {} if not qualification:
                    return Fixture(folder, asset, sha256, size, members, False, None)
                case {"check_rom": bool(check_rom)} if len(qualification) == 1:
                    return Fixture(
                        folder, asset, sha256, size, members, check_rom, None
                    )
                case {"parent": str(parent)} if len(qualification) == 1:
                    return Fixture(folder, asset, sha256, size, members, False, parent)
            raise ValueError(f"invalid qualification for fixture {folder!r}")
    raise ValueError("invalid replay fixture entry")


def load_fixtures(path: Path) -> dict[str, Fixture]:
    with path.open("rb") as manifest:
        entries: object = tomllib.load(manifest).get("fixture")

    if not isinstance(entries, list) or not entries:
        raise ValueError("manifest requires an ordered fixture list")
    fixtures: dict[str, Fixture] = {}
    for entry in entries:
        fixture = decode_fixture(entry)
        if fixture.folder in fixtures:
            raise ValueError(f"duplicate fixture {fixture.folder!r}")
        if fixture.parent is not None and fixture.parent not in fixtures:
            raise ValueError(
                f"parent {fixture.parent!r} must precede fixture {fixture.folder!r}"
            )
        fixtures[fixture.folder] = fixture
    return fixtures


def fetch(rom: str, destination: str, fixture: Fixture) -> None:
    qualification: list[str] = []
    if fixture.check_rom:
        qualification = ["--rom", rom]
    elif fixture.parent is not None:
        qualification = ["--parent", str(Path(destination) / fixture.parent)]
    print(f"Fetching {fixture.folder}", flush=True)
    subprocess.run(
        [
            "bash",
            str(HERE / "fetch_native_fixture.sh"),
            rom,
            destination,
            fixture.folder,
            fixture.asset,
            fixture.sha256,
            str(fixture.size),
            str(fixture.members),
            *qualification,
        ],
        check=True,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Install the pinned replay fixtures")
    parser.add_argument("rom")
    parser.add_argument("destination")
    parser.add_argument(
        "folder",
        nargs="?",
        help="install one ordinary fixture into an existing destination",
    )
    args = parser.parse_args()
    rom: str = args.rom
    destination: str = args.destination
    folder: str | None = args.folder

    try:
        fixtures = load_fixtures(HERE / "replay_fixtures.toml")
    except (OSError, ValueError) as error:
        parser.error(str(error))

    if folder is not None and folder not in fixtures:
        parser.error(f"unknown replay fixture {folder!r}")

    try:
        if folder is None:
            print("Fetching audio_native", flush=True)
            subprocess.run(
                ["bash", str(HERE / "fetch_audio_native.sh"), rom, destination],
                check=True,
            )
        selected = fixtures.values() if folder is None else [fixtures[folder]]
        for fixture in selected:
            fetch(rom, destination, fixture)
    except subprocess.CalledProcessError as error:
        return error.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
