import argparse
import hashlib
import lzma
import os
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path
lazy from collections.abc import Mapping, Sequence

from sm_tools.ci.core.replay_data import (
    ASSET,
    ASSET_HELP,
    DESCRIPTION,
    FETCH_HELP,
    PACK_HELP,
    PARTS,
    PINS,
    PLAIN,
    ROM_SHA256,
    SEALED_HELP,
    Pins,
    already_installed,
    asset_name,
    asset_refusal,
    decrypting,
    downloading,
    encrypting,
    not_the_rom,
    packed,
    passphrase,
    unsafe_member,
    unsealing,
)


class Arguments(argparse.Namespace):
    command: str = ""
    rom: Path = Path()
    destination: Path = Path()
    asset: Path | None = None
    sealed_rom: Path | None = None
    source: Path = Path()
    out: Path = Path()


def sha256(path: Path) -> str:
    with path.open("rb") as file:
        return hashlib.file_digest(file, "sha256").hexdigest()


def cipher_env(rom: Path, pin: str = ROM_SHA256) -> dict[str, str]:
    try:
        key = passphrase(rom.read_bytes(), pin)
    except ValueError:
        raise SystemExit(not_the_rom(str(rom))) from None
    return {"PATH": os.environ["PATH"], "SM_SEED_PASSPHRASE": key}


def normalized(info: tarfile.TarInfo) -> tarfile.TarInfo:
    return info.replace(
        mtime=0,
        uid=0,
        gid=0,
        uname="",
        gname="",
        mode=0o755 if info.isdir() else 0o644,
        deep=False,
    )


def run(command: Sequence[str], env: Mapping[str, str]) -> None:
    _ = subprocess.run(command, env=env, check=True)


def pack(source: Path, rom: Path, out: Path, pins: Pins = PINS) -> None:
    env = cipher_env(rom, pins.rom)
    out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=out) as scratch:
        plain = Path(scratch) / PLAIN
        with (
            lzma.open(plain, "wb", preset=9 | lzma.PRESET_EXTREME) as xz,
            tarfile.open(fileobj=xz, mode="w") as archive,
        ):
            for part in PARTS:
                archive.add(source / part, part, filter=normalized)
        asset = out / asset_name(sha256(plain))
        run(encrypting(str(plain), str(asset)), env)
    lines = packed(asset.name, sha256(asset), asset.stat().st_size, str(asset))
    print("\n".join(lines))


def checked(archive: tarfile.TarFile) -> tarfile.TarFile:
    for member in archive:
        plain = member.isfile() or member.isdir()
        if refusal := unsafe_member(member.name, plain=plain):
            raise SystemExit(refusal)
    return archive


def unseal(sealed: Path, rom: Path) -> None:
    rom.parent.mkdir(parents=True, exist_ok=True)
    key = {"PATH": os.environ["PATH"], "SM_ROM_KEY": os.environ["SM_ROM_KEY"]}
    run(unsealing(str(sealed), str(rom)), key)


def fetch(
    rom: Path,
    destination: Path,
    local: Path | None,
    sealed: Path | None,
    pins: Pins = PINS,
) -> None:
    if destination.exists():
        raise SystemExit(already_installed(str(destination)))
    if sealed is not None:
        unseal(sealed, rom)
    env = cipher_env(rom, pins.rom)
    with tempfile.TemporaryDirectory() as scratch:
        asset = local or Path(scratch) / ASSET
        if local is None:
            run(downloading(str(asset)), {"PATH": env["PATH"]})
        if refusal := asset_refusal(str(asset), sha256(asset), pins.asset):
            raise SystemExit(refusal)
        plain = Path(scratch) / PLAIN
        run(decrypting(str(asset), str(plain)), env)
        destination.parent.mkdir(parents=True, exist_ok=True)
        staging = Path(
            tempfile.mkdtemp(
                prefix=f".{destination.name}.", dir=destination.parent
            )
        )
        try:
            with tarfile.open(plain, "r:xz") as archive:
                checked(archive).extractall(staging, filter="data")
            _ = staging.rename(destination)
        finally:
            shutil.rmtree(staging, ignore_errors=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=DESCRIPTION)
    commands = parser.add_subparsers(dest="command", required=True)
    fetching = commands.add_parser("fetch", help=FETCH_HELP)
    _ = fetching.add_argument("rom", type=Path)
    _ = fetching.add_argument("destination", type=Path)
    _ = fetching.add_argument("--asset", type=Path, help=ASSET_HELP)
    _ = fetching.add_argument("--sealed-rom", type=Path, help=SEALED_HELP)
    packing = commands.add_parser("pack", help=PACK_HELP)
    _ = packing.add_argument("source", type=Path)
    _ = packing.add_argument("rom", type=Path)
    _ = packing.add_argument("out", type=Path)
    args = parser.parse_args(namespace=Arguments())
    if args.command == "fetch":
        fetch(args.rom, args.destination, args.asset, args.sealed_rom)
    else:
        pack(args.source, args.rom, args.out)
    return 0
