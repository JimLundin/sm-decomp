import io
from os.path import samestat
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest


PUBLISH = Path(__file__).with_name("publish_native_fixture.py")
CHECK_PACKET = """\
import argparse
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--root", type=Path, required=True)
parser.add_argument("--parent", type=Path, required=True)
args = parser.parse_args()
if (args.root / "expected").read_bytes() != (args.parent / "reviewed").read_bytes():
    raise SystemExit(1)
"""


class PublishNativeFixtureTest(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.destination = self.root / "fixtures"
        self.destination.mkdir()
        self.target = self.destination / "packet"
        self.parent = self.root / "reviewed-parent"
        self.archive = self.root / "packet.tar.gz"

    def write_archive(self, expected):
        with tarfile.open(self.archive, "w:gz") as archive:
            for name, content in (
                ("check_packet.py", CHECK_PACKET.encode()),
                ("expected", expected),
                ("nested/content", b"nested"),
            ):
                entry = tarfile.TarInfo(f"packet/{name}")
                entry.size = len(content)
                archive.addfile(entry, io.BytesIO(content))

    def publish(self):
        return subprocess.run(
            [
                sys.executable, str(PUBLISH),
                str(self.archive), str(self.destination), "packet", "3",
                "--parent", str(self.parent),
            ],
            capture_output=True,
        ).returncode

    def write_parent(self, reviewed):
        self.parent.mkdir(exist_ok=True)
        (self.parent / "reviewed").write_bytes(reviewed)

    def assert_published(self, expected):
        self.assertEqual(self.publish(), 0)
        self.assertEqual(list(self.destination.iterdir()), [self.target])
        self.assertEqual((self.target / "expected").read_bytes(), expected)

    def test_missing_parent_allows_retry_at_same_path(self):
        self.write_archive(b"reviewed")
        self.assertNotEqual(self.publish(), 0)
        self.assertEqual(list(self.destination.iterdir()), [])

        self.write_parent(b"reviewed")
        self.assert_published(b"reviewed")

    def test_rejected_packet_allows_retry_at_same_path(self):
        self.write_parent(b"reviewed")
        self.write_archive(b"wrong")
        self.assertNotEqual(self.publish(), 0)
        self.assertEqual(list(self.destination.iterdir()), [])

        self.write_archive(b"reviewed")
        self.assert_published(b"reviewed")

    def test_existing_valid_installation_is_reused_and_preserved(self):
        self.write_parent(b"reviewed")
        self.write_archive(b"reviewed")
        self.assert_published(b"reviewed")
        expected = self.target / "expected"
        installed = expected.stat()
        self.assert_published(b"reviewed")
        self.assertTrue(samestat(installed, expected.stat()))

        self.write_parent(b"different")
        self.write_archive(b"different")
        self.assertNotEqual(self.publish(), 0)
        self.assertEqual(expected.read_bytes(), b"reviewed")
        self.assertTrue(samestat(installed, expected.stat()))

    def test_existing_installation_with_extra_file_is_rejected_in_place(self):
        self.write_parent(b"reviewed")
        self.write_archive(b"reviewed")
        self.assert_published(b"reviewed")
        extra = self.target / "nested" / "extra"
        extra.write_bytes(b"extra")
        self.assertNotEqual(self.publish(), 0)
        self.assertEqual(extra.read_bytes(), b"extra")

    def test_existing_installation_with_wrong_path_kind_is_rejected_in_place(self):
        self.write_parent(b"reviewed")
        self.write_archive(b"reviewed")
        self.assert_published(b"reviewed")
        expected = self.target / "expected"
        expected.unlink()
        expected.mkdir()
        self.assertNotEqual(self.publish(), 0)
        self.assertTrue(expected.is_dir())

    def test_existing_installation_with_symlink_is_rejected_in_place(self):
        self.write_parent(b"reviewed")
        self.write_archive(b"reviewed")
        self.assert_published(b"reviewed")
        source = self.root / "outside"
        source.write_bytes(b"reviewed")
        expected = self.target / "expected"
        expected.unlink()
        expected.symlink_to(source)
        self.assertNotEqual(self.publish(), 0)
        self.assertTrue(expected.is_symlink())

    def test_existing_installation_with_hardlink_is_rejected_in_place(self):
        self.write_parent(b"reviewed")
        self.write_archive(b"reviewed")
        self.assert_published(b"reviewed")
        expected = self.target / "expected"
        alias = self.root / "outside"
        alias.hardlink_to(expected)
        self.assertNotEqual(self.publish(), 0)
        self.assertTrue(alias.samefile(expected))


if __name__ == "__main__":
    unittest.main()
