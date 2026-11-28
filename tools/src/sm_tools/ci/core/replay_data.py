import hashlib
from dataclasses import dataclass
from pathlib import PurePosixPath

from ordered import FrozenList

ASSET = "replay-data-48dbc7ccc1f9a605.tar.xz.enc"
ASSET_SHA256 = (
    "13ae1c5340eee4f8a68446ae08af78163880dfffff5331353739173e8ecfe828"
)
ROM_SHA256 = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72"
RELEASE = "https://github.com/JimLundin/sm-decomp/releases/download/ci-data/"
PARTS = FrozenList(("cache", "native"))
PLAIN = "replay-data.tar.xz"
SEED_SALT = b"sm-decomp-seeds-v1"
CIPHER = FrozenList((
    "openssl",
    "enc",
    "-aes-256-cbc",
    "-pbkdf2",
    "-iter",
    "200000",
    "-md",
    "sha256",
    "-pass",
    "env:SM_SEED_PASSPHRASE",
))
ROM_CIPHER = FrozenList((
    "openssl",
    "enc",
    "-aes-256-cbc",
    "-pbkdf2",
    "-iter",
    "600000",
    "-pass",
    "env:SM_ROM_KEY",
))
DESCRIPTION = "Fetch or pack the one encrypted asset of the record cache and native fixtures"
FETCH_HELP = "install the pinned asset as DEST/cache and DEST/native"
ASSET_HELP = "a local copy instead of the ci-data release's"
SEALED_HELP = "decrypt this with SM_ROM_KEY into ROM first"
PACK_HELP = "encrypt SOURCE/cache and SOURCE/native into OUT"


@dataclass(frozen=True, slots=True)
class Pins:
    rom: str = ROM_SHA256
    asset: str = ASSET_SHA256


PINS = Pins()


def passphrase(image: bytes, pin: str) -> str:
    if hashlib.sha256(image).hexdigest() != pin:
        raise ValueError
    return hashlib.sha256(SEED_SALT + image).hexdigest()


def not_the_rom(rom: str) -> str:
    return f"{rom}: not the expected ROM"


def encrypting(plain: str, asset: str) -> FrozenList[str]:
    return FrozenList((*CIPHER, "-salt", "-in", plain, "-out", asset))


def decrypting(asset: str, plain: str) -> FrozenList[str]:
    return FrozenList((*CIPHER, "-d", "-in", asset, "-out", plain))


def unsealing(sealed: str, rom: str) -> FrozenList[str]:
    return FrozenList((*ROM_CIPHER, "-d", "-in", sealed, "-out", rom))


def downloading(asset: str) -> FrozenList[str]:
    return FrozenList((
        "curl",
        "--fail",
        "--silent",
        "--show-error",
        "--location",
        "--retry",
        "8",
        "--retry-delay",
        "15",
        "--retry-all-errors",
        "--retry-max-time",
        "600",
        "--output",
        asset,
        RELEASE + ASSET,
    ))


def asset_name(plain_digest: str) -> str:
    return f"replay-data-{plain_digest[:16]}.tar.xz.enc"


def packed(name: str, digest: str, size: int, path: str) -> FrozenList[str]:
    return FrozenList((
        f"name   {name}",
        f"sha256 {digest}",
        f"bytes  {size}",
        f"path   {path}",
    ))


def unsafe_member(name: str, *, plain: bool) -> str | None:
    path = PurePosixPath(name)
    if path.is_absolute() or ".." in path.parts or not plain:
        return f"replay data member {name!r} is not a plain relative file or folder"
    return None


def already_installed(destination: str) -> str:
    return f"{destination} already exists; delete it to install the replay data again"


def asset_refusal(asset: str, digest: str, pin: str) -> str | None:
    if digest != pin:
        return f"{asset}: SHA-256 differs from the pin {pin}"
    return None
