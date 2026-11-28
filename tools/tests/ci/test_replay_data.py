import hashlib
import io
import lzma
import tarfile
from dataclasses import dataclass
lazy from collections.abc import Mapping, Sequence
lazy from pathlib import Path

import pytest

from sm_tools.ci.core.replay_data import ROM_CIPHER, Pins, encrypting
from sm_tools.ci.shell import replay_data
from sm_tools.ci.shell.replay_data import cipher_env, fetch, pack, run, sha256

ROM = b"a test ROM, not the game"
PINS = Pins(rom=hashlib.sha256(ROM).hexdigest(), asset="")


@dataclass(frozen=True, slots=True)
class Layout:
    root: Path
    rom: Path
    destination: Path

    def packed(self) -> Path:
        source = self.root / "source"
        for part, text in (("cache", "a record"), ("native", "a fixture")):
            (source / part / "nested").mkdir(parents=True)
            _ = (source / part / "nested" / "file.txt").write_text(text)
        out = self.root / "out"
        pack(source, self.rom, out, PINS)
        [asset] = out.iterdir()
        return asset

    def encrypted(self, members: Sequence[tarfile.TarInfo]) -> Path:
        plain = self.root / "plain.tar.xz"
        with (
            lzma.open(plain, "wb") as xz,
            tarfile.open(fileobj=xz, mode="w") as archive,
        ):
            for member in members:
                data = b"x" * member.size if member.isfile() else b""
                archive.addfile(member, io.BytesIO(data) if data else None)
        asset = self.root / "asset.enc"
        run(encrypting(str(plain), str(asset)), cipher_env(self.rom, PINS.rom))
        return asset

    def fetch(self, asset: Path, sealed: Path | None = None) -> None:
        pins = Pins(PINS.rom, sha256(asset))
        fetch(self.rom, self.destination, asset, sealed, pins)

    def leftovers(self) -> list[Path]:
        return list(self.destination.parent.iterdir())


def layout(tmp_path: Path) -> Layout:
    rom = tmp_path / "sm.smc"
    _ = rom.write_bytes(ROM)
    return Layout(tmp_path, rom, tmp_path / "installed" / "replay-data")


def forbid_tools(monkeypatch: pytest.MonkeyPatch) -> list[Sequence[str]]:
    ran: list[Sequence[str]] = []

    def record(command: Sequence[str], _env: Mapping[str, str]) -> None:
        ran.append(command)

    monkeypatch.setattr(replay_data, "run", record)
    return ran


def member(
    name: str, kind: bytes = tarfile.REGTYPE, size: int = 0, linkname: str = ""
) -> tarfile.TarInfo:
    info = tarfile.TarInfo(name)
    info.type = kind
    info.size = size
    info.linkname = linkname
    return info


def test_good_asset_installs_its_trees(tmp_path: Path) -> None:
    files = layout(tmp_path)
    files.fetch(files.packed())
    installed = files.destination
    assert (installed / "cache/nested/file.txt").read_text() == "a record"
    assert (installed / "native/nested/file.txt").read_text() == "a fixture"
    assert [path.name for path in files.leftovers()] == ["replay-data"]


def test_sealed_rom_is_decrypted_with_its_key_before_the_install(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    files = layout(tmp_path)
    asset = files.packed()
    sealed = tmp_path / "payload.enc"
    monkeypatch.setenv("SM_ROM_KEY", "a test key")
    seal = (*ROM_CIPHER, "-salt", "-in", str(files.rom))
    run((*seal, "-out", str(sealed)), {"SM_ROM_KEY": "a test key"})
    files.rom.unlink()
    files.fetch(asset, sealed)
    assert files.rom.read_bytes() == ROM
    installed = files.destination
    assert (installed / "cache/nested/file.txt").read_text() == "a record"


def test_wrong_rom_is_refused_before_any_key_use(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    files = layout(tmp_path)
    asset = files.packed()
    _ = files.rom.write_bytes(b"another ROM")
    ran = forbid_tools(monkeypatch)
    with pytest.raises(SystemExit, match="not the expected ROM"):
        files.fetch(asset)
    assert not ran
    assert not files.destination.exists()


def test_wrong_asset_is_refused_before_decrypting(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    files = layout(tmp_path)
    asset = files.packed()
    ran = forbid_tools(monkeypatch)
    with pytest.raises(SystemExit, match="differs from the pin"):
        fetch(files.rom, files.destination, asset, None, PINS)
    assert not ran
    assert not files.destination.exists()


def test_unsafe_members_are_refused(tmp_path: Path) -> None:
    files = layout(tmp_path)
    unsafe = (
        member("cache/../../escaped", size=1),
        member("/escaped", size=1),
        member("cache/link", tarfile.SYMTYPE, linkname="file"),
        member("native/device", tarfile.CHRTYPE),
    )
    for refused in unsafe:
        asset = files.encrypted((
            member("cache", tarfile.DIRTYPE),
            member("cache/file", size=1),
            refused,
        ))
        with pytest.raises(SystemExit, match="not a plain relative file"):
            files.fetch(asset)
        assert not files.destination.exists()
        assert files.leftovers() == []


def test_failed_extract_leaves_no_destination(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    files = layout(tmp_path)
    asset = files.packed()
    extract = tarfile.TarFile.extractall

    def fail_after_writing(
        archive: tarfile.TarFile, path: Path, **_options: object
    ) -> None:
        extract(archive, path, filter="data")
        message = "disk full"
        raise OSError(message)

    monkeypatch.setattr(tarfile.TarFile, "extractall", fail_after_writing)
    with pytest.raises(OSError, match="disk full"):
        files.fetch(asset)
    assert not files.destination.exists()
    assert files.leftovers() == []


def test_existing_destination_is_refused(tmp_path: Path) -> None:
    files = layout(tmp_path)
    files.destination.mkdir(parents=True)
    with pytest.raises(SystemExit, match="already exists"):
        files.fetch(files.packed())
