"""Invented fixtures for the native seed recapture boundary."""

import pathlib
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from relock_seed_state import same_state


class NativeSeedState(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = pathlib.Path(self.directory.name)
        self.members = {
            "projectid": b"invented-project",
            "corestate": b"invented-machine-state",
            "input": b"invented-input",
            "rrdata": b"invented-load-history",
            "rerecords": b"1\n",
        }
        self.old = self.write("old.lss", self.members)

    def write(self, name, members):
        path = self.root / name
        with zipfile.ZipFile(path, "w") as archive:
            for member, content in members.items():
                archive.writestr(member, content)
        return path

    def test_identical_members_pass(self):
        self.assertTrue(same_state(self.old, self.write("same.lss", self.members)))

    def test_rerecord_history_can_differ(self):
        changed = self.members | {"rrdata": b"another-load", "rerecords": b"2\n"}
        self.assertTrue(same_state(self.old, self.write("history.lss", changed)))

    def test_project_machine_and_input_bytes_stay_exact(self):
        for name in ("projectid", "corestate", "input"):
            with self.subTest(member=name):
                changed = self.members | {name: b"different-invented-content"}
                self.assertFalse(same_state(self.old, self.write("changed.lss", changed)))

    def test_added_member_fails(self):
        changed = self.members | {"extra": b"extra-invented-member"}
        self.assertFalse(same_state(self.old, self.write("extra.lss", changed)))

    def test_missing_members_including_history_fail(self):
        for name in self.members:
            with self.subTest(member=name):
                changed = {key: value for key, value in self.members.items() if key != name}
                self.assertFalse(same_state(self.old, self.write("missing.lss", changed)))

    def test_duplicate_member_fails(self):
        path = self.write("duplicate.lss", self.members)
        with zipfile.ZipFile(path, "a") as archive:
            with self.assertWarns(UserWarning):
                archive.writestr("corestate", self.members["corestate"])
        self.assertFalse(same_state(self.old, path))


if __name__ == "__main__":
    unittest.main()
