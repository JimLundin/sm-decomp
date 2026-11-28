import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path
lazy from collections.abc import Mapping

import pytest
from ordered import FrozenList, FrozenOrderedDict

from sm_tools.lane.core.gate import LANE, SUITES, TOOLS_PROJECT
from sm_tools.lane.shell.gate import run, self_tests
from sm_tools.shell.process import git_out

FAKE_PYTHON = """import os, pathlib, sys
pathlib.Path({ran!r}, "{name}").write_text("")
print(os.environ.get("GATE_TEST_OUT_{name}", ""))
sys.exit(int(os.environ.get("GATE_TEST_STATUS_{name}", "0")))
"""
FAKE_ADJUSTED = """import os, pathlib, sys
name = "adjusted" if sys.argv[1:2] == ["--missing"] else "owners"
pathlib.Path({ran!r}, name).write_text(" ".join(sys.argv[1:]) + "\\n")
if name == "adjusted":
    print(f"adjusted: {{sys.argv[2]}}: Entry has no adjusted vector")
    sys.exit(1)
print(os.environ.get("GATE_TEST_OUT_owners", ""))
sys.exit(int(os.environ.get("GATE_TEST_STATUS_owners", "0")))
"""
FAKE_TOOL = f"""#!{sys.executable}
import os, pathlib, sys
arguments = " ".join(sys.argv[1:])
name = next((check for word, check in (("crate-rules", "crate_rules"), ("cleanroom", "clean_room"),
                                       ("doc-links", "doc_links"), ("test", "verify_lib")) if word in sys.argv[1:]),
            "{{name}}")
pathlib.Path({{ran!r}}, name).write_text(arguments)
if "SM_VECTOR_FILE" in os.environ:
    pathlib.Path({{ran!r}}, "inherited").write_text(name)
if os.environ.get("GATE_TEST_OUT_" + name):
    print(os.environ["GATE_TEST_OUT_" + name])
sys.exit(int(os.environ.get("GATE_TEST_STATUS_" + name, "0")))
"""
CARGO_INCLUDES = FrozenList((
    'include = [{ path = "more.toml" }]',
    '"include" = ["more.toml"]',
    "'include' = [\"more.toml\"]",
    '[[include]]\npath = "more.toml"',
))
FIXTURE = FrozenList((
    "-c",
    "user.name=fixture",
    "-c",
    "user.email=fixture@invalid",
))
GATE_OK = re.compile(r"(?m)^GATE OK ")
DENY_SKIPPED = re.compile(r"(?m)^DENY SKIPPED: ")
HARD_TABS = "hard_tabs = true\n"
BUILD = "[build]\n"
UNTRACKED_HIDDEN = FrozenOrderedDict({
    "GIT_CONFIG_COUNT": "1",
    "GIT_CONFIG_KEY_0": "status.showUntrackedFiles",
    "GIT_CONFIG_VALUE_0": "no",
})


@dataclass(frozen=True, slots=True)
class Gate:
    scratch: Path
    monkeypatch: pytest.MonkeyPatch
    capsys: pytest.CaptureFixture[str]

    @property
    def repo(self) -> Path:
        return self.scratch / "repo"

    @property
    def bin(self) -> Path:
        return self.scratch / "bin"

    @property
    def ran(self) -> Path:
        return self.scratch / "ran"

    def fake(self, name: str, path: Path) -> None:
        _ = path.write_text(
            FAKE_TOOL.format(name=name, ran=str(self.ran)), encoding="utf-8"
        )
        path.chmod(0o755)

    def git(self, *arguments: str) -> str:
        return git_out(*arguments, cwd=self.repo).strip()

    def commit(self, message: str, *paths: str) -> None:
        if paths:
            _ = self.git("add", *paths)
        _ = self.git(*FIXTURE, "commit", "-q", "--allow-empty", "-m", message)

    def gate(
        self, environment: Mapping[str, str] | None = None
    ) -> tuple[int, str]:
        for record in self.ran.iterdir():
            record.unlink()
        settings = {
            "SM_RUST_BIN": str(self.bin),
            "SM_GATE_PASSED": str(self.scratch / "passed"),
            "CARGO_HOME": str(self.scratch / "no-cargo-home"),
            "SM_GATE_BASE": "HEAD",
            "SM_GATE_FRESH": "",
        }
        with self.monkeypatch.context() as patched:
            patched.setenv("PATH", str(self.bin), prepend=os.pathsep)
            patched.setattr(sys, "executable", str(self.bin / "python"))
            for name, value in (settings | dict(environment or {})).items():
                patched.setenv(name, value)
            patched.chdir(self.repo)
            status = run(())
        captured = self.capsys.readouterr()
        return status, captured.out + captured.err

    def ran_checks(self) -> FrozenList[str]:
        return FrozenList(sorted(record.name for record in self.ran.iterdir()))

    def passes(self, environment: Mapping[str, str] | None = None) -> str:
        status, out = self.gate(environment)
        assert status == 0, out
        assert GATE_OK.search(out)
        return out

    def reuses(self, environment: Mapping[str, str] | None = None) -> bool:
        return "GATE REUSED: " in self.passes(environment)

    def settled(self, environment: Mapping[str, str] | None = None) -> None:
        assert self.reuses(environment) or self.reuses(environment), (
            "a second gate with the same settings reused nothing"
        )


@pytest.fixture
def fixture(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> Gate:
    made = Gate(tmp_path, monkeypatch, capsys)
    for path in (
        made.repo / "tools/cleanroom",
        made.repo / "scenarios",
        made.bin,
        made.ran,
    ):
        path.mkdir(parents=True)
    _ = (made.repo / ".gitignore").write_text("__pycache__/\n")
    _ = (made.repo / "rust-toolchain.toml").write_text('channel = "0.0.0"\n')
    _ = (made.repo / "scenarios/demo.toml").write_text("Invented vectors.\n")
    made.fake("cargo", made.bin / "cargo")
    made.fake("deny", made.bin / "cargo-deny")
    prereview = made.bin / "prereview"
    _ = prereview.write_text(
        f"#!{sys.executable}\n"
        + FAKE_PYTHON.format(ran=str(made.ran), name="prereview")
    )
    prereview.chmod(0o755)
    _ = (made.repo / "tools/check_mutants.py").write_text(
        FAKE_PYTHON.format(ran=str(made.ran), name="faults")
    )
    _ = (made.repo / "tools/cleanroom/adjusted_vectors_run.py").write_text(
        FAKE_ADJUSTED.format(ran=str(made.ran))
    )
    _ = made.git("init", "-q", "-b", "lane/test")
    made.commit("Invented tree", "-A")
    return made


def test_an_unresolved_base_stops_before_any_check(fixture: Gate) -> None:
    status, out = fixture.gate({"SM_GATE_BASE": "no-such-base"})
    assert status == 2, out
    assert fixture.ran_checks() == ()


def test_a_clean_branch_passes_and_its_tree_reuses_the_pass(
    fixture: Gate,
) -> None:
    _ = fixture.passes()
    assert "verify_lib" not in fixture.ran_checks()
    fixture.commit("Same tree")
    out = fixture.passes()
    assert "GATE REUSED: " in out
    head = fixture.git("rev-parse", "--short", "HEAD")
    assert re.search(rf"(?m)^GATE OK {head} ", out)
    assert fixture.ran_checks() == ("prereview",)
    _ = fixture.passes({"SM_GATE_FRESH": "1"})
    assert "cargo" in fixture.ran_checks()


def test_a_reused_pass_still_runs_the_review_findings(fixture: Gate) -> None:
    fixture.settled()
    fixture.commit("Same tree, a message the review findings refuse")
    status, out = fixture.gate({
        "GATE_TEST_STATUS_prereview": "1",
        "GATE_TEST_OUT_prereview": "invented dash finding",
    })
    assert status == 1, out
    assert "PREREVIEW FAIL" in out.splitlines()
    assert "invented dash finding" in out.splitlines()
    assert "GATE REUSED: " not in out
    assert fixture.ran_checks() == ("prereview",)


def test_no_check_inherits_a_vector_file(fixture: Gate) -> None:
    (fixture.repo / "crates").mkdir()
    _ = (fixture.repo / "crates/lib.rs").write_text("Invented source.\n")
    _ = fixture.passes({"SM_VECTOR_FILE": str(fixture.scratch / "stale.toml")})
    assert "verify_lib" in fixture.ran_checks()
    assert "inherited" not in fixture.ran_checks()


def test_other_compiler_flags_run_again(fixture: Gate) -> None:
    fixture.settled()
    assert not fixture.reuses({"RUSTFLAGS": "--cap-lints=allow"})


def test_a_missing_cargo_deny_runs_again_and_says_deny_was_skipped(
    fixture: Gate,
) -> None:
    fixture.settled()
    _ = (fixture.bin / "cargo-deny").rename(fixture.bin / "no-cargo-deny")
    entries = (
        entry
        for entry in os.get_exec_path()
        if entry and not Path(entry, "cargo-deny").exists()
    )
    path = ":".join((str(fixture.bin), *entries))
    assert not fixture.reuses({"PATH": path})
    out = fixture.passes({"PATH": path})
    assert "GATE REUSED: " in out
    assert DENY_SKIPPED.search(out)


def test_a_skip_worktree_file_turns_reuse_off(fixture: Gate) -> None:
    fixture.settled()
    _ = fixture.git("update-index", "--skip-worktree", "scenarios/demo.toml")
    assert not fixture.reuses()
    assert not fixture.reuses()


def test_what_the_checks_run_with_is_part_of_the_key(fixture: Gate) -> None:
    fixture.settled()
    _ = (fixture.scratch / "rustfmt.toml").write_text(HARD_TABS)
    assert not fixture.reuses(), "a gate under a rustfmt.toml reused a pass"
    (fixture.scratch / "rustfmt.toml").unlink()
    fixture.settled()
    assert not fixture.reuses({"CLEANROOM_PY": "/elsewhere/python"})


def test_a_cargo_configuration_outside_the_tree_runs_again(
    fixture: Gate,
) -> None:
    fixture.settled()
    cargo = fixture.scratch / ".cargo"
    cargo.mkdir()
    _ = (cargo / "config.toml").write_text(BUILD)
    assert not fixture.reuses(), (
        "a gate under a parent .cargo/config.toml reused a pass"
    )
    for include in CARGO_INCLUDES:
        _ = (cargo / "config.toml").write_text(include + "\n")
        assert not fixture.reuses(), f"{include} reused a pass"
        assert not fixture.reuses(), f"{include} reused a pass the second time"
    (cargo / "config.toml").unlink()
    cargo.rmdir()
    fixture.settled()
    (fixture.scratch / "no-cargo-home").mkdir()
    _ = (fixture.scratch / "no-cargo-home/config.toml").write_text(BUILD)
    assert not fixture.reuses(), (
        "a gate with a CARGO_HOME configuration reused a pass"
    )


def test_rustfmt_configurations_outside_the_tree_run_again(
    fixture: Gate,
) -> None:
    homes = {
        "XDG_CONFIG_HOME": str(fixture.scratch / "xdg"),
        "HOME": str(fixture.scratch / "home"),
    }
    fixture.settled(homes)
    (fixture.scratch / "xdg/rustfmt").mkdir(parents=True)
    _ = (fixture.scratch / "xdg/rustfmt/.rustfmt.toml").write_text(HARD_TABS)
    assert not fixture.reuses(homes), (
        "rustfmt's user configuration reused a pass"
    )
    (fixture.scratch / "xdg/rustfmt/.rustfmt.toml").unlink()
    (fixture.scratch / "xdg/rustfmt").rmdir()
    fixture.settled(homes)
    (fixture.scratch / "home").mkdir()
    _ = (fixture.scratch / "home/rustfmt.toml").write_text(HARD_TABS)
    assert not fixture.reuses(homes), "a rustfmt.toml in the home reused a pass"
    (fixture.scratch / "home/rustfmt.toml").unlink()
    fixture.settled()
    (fixture.scratch / "rustfmt.toml").symlink_to("rustfmt.toml")
    assert not fixture.reuses(), (
        "a rustfmt.toml that is a link loop reused a pass"
    )


def test_a_cargo_configuration_above_temp_runs_again(fixture: Gate) -> None:
    elsewhere = fixture.scratch / "elsewhere"
    (elsewhere / "tmp").mkdir(parents=True)
    temp = {"TEMP": str(elsewhere / "tmp")}
    fixture.settled(temp)
    (elsewhere / ".cargo").mkdir()
    _ = (elsewhere / ".cargo/config.toml").write_text(BUILD)
    assert not fixture.reuses(temp)


def test_a_file_only_an_outside_ignore_rule_hides_turns_reuse_off(
    fixture: Gate,
) -> None:
    fixture.settled()
    _ = (fixture.repo / "rustfmt.toml").write_text(HARD_TABS)
    _ = (fixture.scratch / "excludes").write_text("rustfmt.toml\n")
    excluded = {
        "GIT_CONFIG_COUNT": "1",
        "GIT_CONFIG_KEY_0": "core.excludesFile",
        "GIT_CONFIG_VALUE_0": str(fixture.scratch / "excludes"),
    }
    assert not (fixture.reuses(excluded))
    assert not (fixture.reuses(excluded))


def test_a_failed_gate_is_not_reused(fixture: Gate) -> None:
    _ = (fixture.repo / "scenarios/rules.toml").write_text(
        "Invented crate rules.\n"
    )
    fixture.commit("Changed tree", "-A")
    status, out = fixture.gate({"GATE_TEST_STATUS_crate_rules": "1"})
    assert status != 0, out
    assert not fixture.reuses()
    assert "crate_rules" in fixture.ran_checks()


def test_an_untracked_subject_file_runs_its_check(fixture: Gate) -> None:
    (fixture.repo / "scenarios/new").mkdir()
    _ = (fixture.repo / "scenarios/new/demo.toml").write_text(
        "Invented untracked vectors.\n"
    )
    _ = fixture.passes(UNTRACKED_HIDDEN)
    assert "verify_lib" in fixture.ran_checks()


def test_a_failing_check_shows_its_log(fixture: Gate) -> None:
    (fixture.repo / "scenarios/new").mkdir()
    _ = (fixture.repo / "scenarios/new/demo.toml").write_text(
        "Invented untracked vectors.\n"
    )
    owners = fixture.repo / "tools/cleanroom/adjusted_rule_owners"
    owners.mkdir()
    _ = (owners / "demo.toml").write_text("Invented owners.\n")
    status, out = fixture.gate({
        "GATE_TEST_STATUS_verify_lib": "1",
        "GATE_TEST_OUT_verify_lib": "invented link failure",
        "GATE_TEST_STATUS_owners": "1",
        "GATE_TEST_OUT_owners": "invented decode error",
    })
    assert status != 0, out
    assert "invented link failure" in out.splitlines()
    assert "invented decode error" in out.splitlines()


def test_a_failing_check_shows_its_finding_and_the_tools_tests_run_with_verify(
    fixture: Gate,
) -> None:
    (fixture.repo / "crates").mkdir()
    _ = (fixture.repo / "crates/lib.rs").write_text("Invented source.\n")
    status, out = fixture.gate({
        "GATE_TEST_STATUS_crate_rules": "3",
        "GATE_TEST_OUT_crate_rules": "rule 5: invented hit",
    })
    assert status == 1, out
    assert out.splitlines()[:2] == ["CRATE RULES FAIL", "rule 5: invented hit"]
    assert (fixture.ran / "verify_lib").read_text() == "test -p tools"


def test_the_clean_room_and_crate_rules_both_report_their_findings(
    fixture: Gate,
) -> None:
    status, out = fixture.gate({
        "GATE_TEST_STATUS_clean_room": "1",
        "GATE_TEST_OUT_clean_room": "invented clean-room hit",
        "GATE_TEST_STATUS_crate_rules": "1",
        "GATE_TEST_OUT_crate_rules": "rule 4: invented hit",
    })
    assert status == 1, out
    lines = out.splitlines()
    for failure, finding in (
        ("CLEAN ROOM FAIL", "invented clean-room hit"),
        ("CRATE RULES FAIL", "rule 4: invented hit"),
    ):
        assert lines[lines.index(failure) + 1] == finding


def test_a_changed_domain_gates_its_adjusted_vectors(fixture: Gate) -> None:
    (fixture.repo / "docs/behaviours").mkdir(parents=True)
    (fixture.repo / "scenarios/vectors").mkdir()
    for path in ("docs/behaviours/demo.md", "docs/behaviours/alone.md"):
        _ = (fixture.repo / path).write_text("Invented spec.\n")
    for path in (
        "scenarios/vectors/demo.toml",
        "scenarios/vectors/orphan.toml",
    ):
        _ = (fixture.repo / path).write_text("Invented vectors.\n")
    status, out = fixture.gate()
    assert status != 0, out
    assert (fixture.ran / "adjusted").read_text() == "--missing demo\n"
    assert "adjusted: demo: Entry has no adjusted vector" in out.splitlines()


SELECTION_FILES = FrozenList((
    *SUITES,
    *(subject for suite in SUITES.values() for subject in suite.subjects),
    f"{LANE}/shell/land.py",
    "tools/cleanroom/box_relay.py",
    f"{LANE}/__init__.py",
    *TOOLS_PROJECT,
))


@dataclass(frozen=True, slots=True)
class Selection:
    repo: Path
    base: str

    def git(self, *arguments: str) -> str:
        return git_out(*arguments, cwd=self.repo).strip()

    def append(self, path: str) -> None:
        edited = self.repo / path
        _ = edited.write_text(edited.read_text() + "edited\n")


def selected(base: str = "HEAD") -> FrozenList[str]:
    return FrozenList(sorted(Path(suite).stem for suite in self_tests(base)))


@pytest.fixture
def selection(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Selection:
    for path in SELECTION_FILES:
        (tmp_path / path).parent.mkdir(parents=True, exist_ok=True)
        _ = (tmp_path / path).write_text("invented\n")
    _ = git_out("init", "-q", cwd=tmp_path)
    for setting in (
        "user.name=Fixture",
        "user.email=fixture@example.invalid",
        "commit.gpgsign=false",
        "status.showUntrackedFiles=no",
    ):
        _ = git_out("config", *setting.split("="), cwd=tmp_path)
    _ = git_out("add", "-A", cwd=tmp_path)
    _ = git_out("commit", "-qm", "base", cwd=tmp_path)
    monkeypatch.chdir(tmp_path)
    return Selection(
        tmp_path, git_out("rev-parse", "HEAD", cwd=tmp_path).strip()
    )


@pytest.mark.usefixtures("selection")
def test_an_unchanged_branch_runs_nothing() -> None:
    assert selected() == ()


def test_a_subject_edit_runs_its_suite_staged_unstaged_or_committed(
    selection: Selection,
) -> None:
    gate = f"{LANE}/core/gate.py"
    selection.append(gate)
    assert selected() == ("test_gate",)
    _ = selection.git("add", gate)
    assert selected() == ("test_gate",)
    _ = selection.git("commit", "-qm", "Edit the gate")
    assert selected() == ()
    assert selected(selection.base) == ("test_gate",)


def test_a_tool_with_its_own_tests_or_a_package_marker_runs_no_suite(
    selection: Selection,
) -> None:
    for path in (
        f"{LANE}/shell/land.py",
        "tools/cleanroom/box_relay.py",
        f"{LANE}/__init__.py",
    ):
        selection.append(path)
        assert selected() == (), path
        _ = selection.git("restore", "--", path)


def test_an_adjusted_vector_input_runs_its_suite(selection: Selection) -> None:
    selection.append("tools/cleanroom/adjusted_vectors_run.py")
    assert selected() == ("test_adjusted_vectors_run",)
    selection.append(f"{LANE}/core/changes.py")
    assert selected() == ("test_adjusted_vectors_run", "test_gate")


def test_an_unknown_tool_file_runs_every_suite(selection: Selection) -> None:
    _ = selection.git("mv", f"{LANE}/core/gate.py", f"{LANE}/core/renamed.py")
    assert selected() == tuple(sorted(Path(suite).stem for suite in SUITES))


def test_the_tools_project_runs_every_suite(selection: Selection) -> None:
    for path in TOOLS_PROJECT:
        selection.append(path)
        assert selected() == tuple(
            sorted(Path(suite).stem for suite in SUITES)
        ), path
        _ = selection.git("restore", "--", path)


def test_a_subject_moved_out_of_its_folder_still_runs_its_suite(
    selection: Selection,
) -> None:
    _ = selection.git("mv", f"{LANE}/shell/worktree.py", "removed-worktree.py")
    assert selected() == ("test_worktree",)
