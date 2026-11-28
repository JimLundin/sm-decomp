import zipfile
lazy from collections.abc import Mapping
lazy from pathlib import Path

import pytest
from ordered import FrozenOrderedDict

from sm_tools.oracle.shell.relock import same_native_state

MEMBERS = FrozenOrderedDict({
    "projectid": b"invented-project",
    "corestate": b"invented-machine-state",
    "input": b"invented-input",
    "rrdata": b"invented-load-history",
    "rerecords": b"1\n",
})


def write(tmp_path: Path, name: str, members: Mapping[str, bytes]) -> Path:
    path = tmp_path / name
    with zipfile.ZipFile(path, "w") as archive:
        for member, content in members.items():
            archive.writestr(member, content)
    return path


def old(tmp_path: Path) -> Path:
    return write(tmp_path, "old.lss", MEMBERS)


def test_identical_members_pass(tmp_path: Path) -> None:
    same = write(tmp_path, "same.lss", MEMBERS)
    assert same_native_state(old(tmp_path), same)


def test_rerecord_history_can_differ(tmp_path: Path) -> None:
    changed = MEMBERS | {"rrdata": b"another-load", "rerecords": b"2\n"}
    history = write(tmp_path, "history.lss", changed)
    assert same_native_state(old(tmp_path), history)


def test_project_machine_and_input_bytes_stay_exact(tmp_path: Path) -> None:
    before = old(tmp_path)
    for name in ("projectid", "corestate", "input"):
        changed = MEMBERS | {name: b"different-invented-content"}
        after = write(tmp_path, "changed.lss", changed)
        assert not same_native_state(before, after), name


def test_added_member_fails(tmp_path: Path) -> None:
    changed = MEMBERS | {"extra": b"extra-invented-member"}
    extra = write(tmp_path, "extra.lss", changed)
    assert not same_native_state(old(tmp_path), extra)


def test_missing_members_including_history_fail(tmp_path: Path) -> None:
    before = old(tmp_path)
    for name in MEMBERS:
        changed = FrozenOrderedDict(
            (key, value) for key, value in MEMBERS.items() if key != name
        )
        after = write(tmp_path, "missing.lss", changed)
        assert not same_native_state(before, after), name


def test_duplicate_member_fails(tmp_path: Path) -> None:
    path = write(tmp_path, "duplicate.lss", MEMBERS)
    with (
        zipfile.ZipFile(path, "a") as archive,
        pytest.warns(UserWarning, match="Duplicate name"),
    ):
        archive.writestr("corestate", MEMBERS["corestate"])
    assert not same_native_state(old(tmp_path), path)
