"""The replay data installs only from the pinned ROM and asset, and only as plain files."""

import hashlib
import io
import lzma
import subprocess
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path
from typing import Any
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import replay_data

ROM = b"a test ROM, not the game"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class ReplayDataTests(unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.rom = self.root / "sm.smc"
        self.rom.write_bytes(ROM)
        self.destination = self.root / "installed" / "replay-data"
        for name, value in (("ROM_SHA256", digest(ROM)), ("ASSET_SHA256", "")):
            pin = patch.object(replay_data, name, value)
            pin.start()
            self.addCleanup(pin.stop)

    def packed(self) -> Path:
        source = self.root / "source"
        for part, text in (("cache", "a record"), ("native", "a fixture")):
            (source / part / "nested").mkdir(parents=True)
            (source / part / "nested" / "file.txt").write_text(text)
        out = self.root / "out"
        with patch("builtins.print"):
            replay_data.pack(source, self.rom, out)
        [asset] = out.iterdir()
        return asset

    def encrypted(self, members: list[tarfile.TarInfo]) -> Path:
        plain = self.root / "plain.tar.xz"
        with lzma.open(plain, "wb") as xz, tarfile.open(fileobj=xz, mode="w") as archive:
            for member in members:
                data = b"x" * member.size if member.isfile() else b""
                archive.addfile(member, io.BytesIO(data) if data else None)
        asset = self.root / "asset.enc"
        env = replay_data.cipher_env(self.rom)
        subprocess.run([*replay_data.CIPHER, "-salt", "-in", str(plain), "-out", str(asset)],
                       env=env, check=True)
        return asset

    def fetch(self, asset: Path) -> None:
        with patch.object(replay_data, "ASSET_SHA256", replay_data.sha256(asset)):
            replay_data.fetch(self.rom, self.destination, asset)

    def test_good_asset_installs_its_trees(self) -> None:
        self.fetch(self.packed())
        self.assertEqual((self.destination / "cache/nested/file.txt").read_text(), "a record")
        self.assertEqual((self.destination / "native/nested/file.txt").read_text(), "a fixture")
        self.assertEqual([path.name for path in self.destination.parent.iterdir()], ["replay-data"])

    def test_wrong_rom_is_refused_before_any_key_use(self) -> None:
        asset = self.packed()
        self.rom.write_bytes(b"another ROM")
        with patch("subprocess.run") as run, self.assertRaisesRegex(SystemExit, "not the expected ROM"):
            self.fetch(asset)
        run.assert_not_called()
        self.assertFalse(self.destination.exists())

    def test_wrong_asset_is_refused_before_decrypting(self) -> None:
        asset = self.packed()
        with patch("subprocess.run") as run, self.assertRaisesRegex(SystemExit, "differs from the pin"):
            replay_data.fetch(self.rom, self.destination, asset)
        run.assert_not_called()
        self.assertFalse(self.destination.exists())

    def test_unsafe_members_are_refused(self) -> None:
        def member(name: str, kind: bytes = tarfile.REGTYPE, **fields: Any) -> tarfile.TarInfo:
            info = tarfile.TarInfo(name)
            info.type = kind
            for field, value in fields.items():
                setattr(info, field, value)
            return info

        cases = {
            "parent": member("cache/../../escaped", size=1),
            "absolute": member("/tmp/escaped", size=1),
            "symlink": member("cache/link", tarfile.SYMTYPE, linkname="file"),
            "device": member("native/device", tarfile.CHRTYPE, devmajor=1, devminor=3),
        }
        for case, unsafe in cases.items():
            with self.subTest(case):
                asset = self.encrypted([member("cache", tarfile.DIRTYPE), member("cache/file", size=1),
                                        unsafe])
                with self.assertRaisesRegex(SystemExit, "not a plain relative file or folder"):
                    self.fetch(asset)
                self.assertFalse(self.destination.exists())
                self.assertEqual(list(self.destination.parent.iterdir()), [])

    def test_failed_extract_leaves_no_destination(self) -> None:
        asset = self.packed()
        extract = tarfile.TarFile.extractall

        def fail_after_writing(archive: tarfile.TarFile, path: str, **options: Any) -> None:
            extract(archive, path, **options)
            raise OSError("disk full")

        with patch.object(tarfile.TarFile, "extractall", fail_after_writing), \
                self.assertRaisesRegex(OSError, "disk full"):
            self.fetch(asset)
        self.assertFalse(self.destination.exists())
        self.assertEqual(list(self.destination.parent.iterdir()), [])

    def test_existing_destination_is_refused(self) -> None:
        self.destination.mkdir(parents=True)
        with self.assertRaisesRegex(SystemExit, "already exists"):
            self.fetch(self.packed())


if __name__ == "__main__":
    unittest.main()
