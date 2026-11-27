import argparse
import hashlib
import lzma
import os
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path, PurePosixPath

ASSET = "replay-data-48dbc7ccc1f9a605.tar.xz.enc"
ASSET_SHA256 = "13ae1c5340eee4f8a68446ae08af78163880dfffff5331353739173e8ecfe828"
ROM_SHA256 = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72"
RELEASE = "https://github.com/JimLundin/sm-decomp/releases/download/ci-data/"
PARTS = ("cache", "native")
CIPHER = ["openssl", "enc", "-aes-256-cbc", "-pbkdf2", "-iter", "200000", "-md", "sha256",
          "-pass", "env:SM_SEED_PASSPHRASE"]


def sha256(path: Path) -> str:
    with path.open("rb") as file:
        return hashlib.file_digest(file, "sha256").hexdigest()


def cipher_env(rom: Path) -> dict[str, str]:
    image = rom.read_bytes()
    if hashlib.sha256(image).hexdigest() != ROM_SHA256:
        raise SystemExit(f"{rom}: not the expected ROM")
    key = hashlib.sha256(b"sm-decomp-seeds-v1" + image).hexdigest()
    return {"PATH": os.environ["PATH"], "SM_SEED_PASSPHRASE": key}


def normalized(info: tarfile.TarInfo) -> tarfile.TarInfo:
    info.mtime = 0
    info.uid = info.gid = 0
    info.uname = info.gname = ""
    info.mode = 0o755 if info.isdir() else 0o644
    return info


def pack(source: Path, rom: Path, out: Path) -> None:
    env = cipher_env(rom)
    out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=out) as scratch:
        plain = Path(scratch) / "replay-data.tar.xz"
        with (lzma.open(plain, "wb", preset=9 | lzma.PRESET_EXTREME) as xz,
              tarfile.open(fileobj=xz, mode="w") as archive):
            for part in PARTS:
                archive.add(source / part, part, filter=normalized)
        asset = out / f"replay-data-{sha256(plain)[:16]}.tar.xz.enc"
        subprocess.run([*CIPHER, "-salt", "-in", str(plain), "-out", str(asset)], env=env, check=True)
    print(f"name   {asset.name}")
    print(f"sha256 {sha256(asset)}")
    print(f"bytes  {asset.stat().st_size}")
    print(f"path   {asset}")


def checked(archive: tarfile.TarFile) -> tarfile.TarFile:
    for member in archive:
        name = PurePosixPath(member.name)
        if name.is_absolute() or ".." in name.parts or not (member.isfile() or member.isdir()):
            raise SystemExit(f"replay data member {member.name!r} is not a plain relative file or folder")
    return archive


def fetch(rom: Path, destination: Path, local: Path | None) -> None:
    if destination.exists():
        raise SystemExit(f"{destination} already exists; delete it to install the replay data again")
    env = cipher_env(rom)
    with tempfile.TemporaryDirectory() as scratch:
        asset = local or Path(scratch) / ASSET
        if local is None:
            subprocess.run(["curl", "--fail", "--silent", "--show-error", "--location",
                            "--retry", "8", "--retry-delay", "15", "--retry-all-errors",
                            "--retry-max-time", "600", "--output", str(asset), RELEASE + ASSET],
                           env={"PATH": env["PATH"]}, check=True)
        if sha256(asset) != ASSET_SHA256:
            raise SystemExit(f"{asset}: SHA-256 differs from the pin {ASSET_SHA256}")
        plain = Path(scratch) / "replay-data.tar.xz"
        subprocess.run([*CIPHER, "-d", "-in", str(asset), "-out", str(plain)], env=env, check=True)
        destination.parent.mkdir(parents=True, exist_ok=True)
        staging = Path(tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent))
        try:
            with tarfile.open(plain, "r:xz") as archive:
                checked(archive).extractall(staging, filter="data")
            staging.rename(destination)
        finally:
            shutil.rmtree(staging, ignore_errors=True)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Fetch or pack the one encrypted asset of the record cache and native fixtures")
    commands = parser.add_subparsers(dest="command", required=True)
    fetching = commands.add_parser("fetch", help="install the pinned asset as DEST/cache and DEST/native")
    fetching.add_argument("rom", type=Path)
    fetching.add_argument("destination", type=Path)
    fetching.add_argument("--asset", type=Path, help="a local copy instead of the ci-data release's")
    packing = commands.add_parser("pack", help="encrypt SOURCE/cache and SOURCE/native into OUT")
    packing.add_argument("source", type=Path)
    packing.add_argument("rom", type=Path)
    packing.add_argument("out", type=Path)
    args = parser.parse_args()
    if args.command == "fetch":
        fetch(args.rom, args.destination, args.asset)
    else:
        pack(args.source, args.rom, args.out)


if __name__ == "__main__":
    main()
