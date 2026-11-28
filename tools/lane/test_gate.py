import contextlib
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gate

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
CARGO_INCLUDES = ('include = [{ path = "more.toml" }]', '"include" = ["more.toml"]', "'include' = [\"more.toml\"]",
                  '[[include]]\npath = "more.toml"')


class Gate(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = Path(self.enterContext(tempfile.TemporaryDirectory(
            prefix="gate-test-", dir=os.environ.get("TEMP") or os.environ.get("TMPDIR"))))
        self.repo = self.scratch / "repo"
        self.bin = self.scratch / "bin"
        self.ran = self.scratch / "ran"
        for path in (self.repo / "tools/lane", self.repo / "tools/cleanroom", self.repo / "scenarios", self.bin, self.ran):
            path.mkdir(parents=True)
        for name in ("gate.py", "changes.py"):
            shutil.copy(HERE / name, self.repo / "tools/lane" / name)
        (self.repo / ".gitignore").write_text("__pycache__/\n")
        (self.repo / "rust-toolchain.toml").write_text('channel = "0.0.0"\n')
        (self.repo / "scenarios/demo.toml").write_text("Invented vectors.\n")
        self.fake("cargo", self.bin / "cargo")
        self.fake("deny", self.bin / "cargo-deny")
        for name, script in (("faults", "tools/check_mutants.py"), ("prereview", "tools/lane/prereview.py")):
            (self.repo / script).write_text(FAKE_PYTHON.format(ran=str(self.ran), name=name))
        (self.repo / "tools/cleanroom/adjusted_vectors_run.py").write_text(FAKE_ADJUSTED.format(ran=str(self.ran)))
        self.git("init", "-q", "-b", "lane/test")
        self.commit("Invented tree", "-A")

    def fake(self, name: str, path: Path) -> None:
        path.write_text(FAKE_TOOL.format(name=name, ran=str(self.ran)))
        path.chmod(0o755)

    def git(self, *args: str) -> str:
        return subprocess.run(["git", *args], cwd=self.repo, check=True, capture_output=True, text=True).stdout.strip()

    def commit(self, message: str, *paths: str) -> None:
        if paths:
            self.git("add", *paths)
        self.git("-c", "user.name=fixture", "-c", "user.email=fixture@invalid", "commit", "-q", "--allow-empty",
                 "-m", message)

    def gate(self, **env: str) -> tuple[int, str]:
        for record in self.ran.iterdir():
            record.unlink()
        settings = {"PATH": f"{self.bin}:{os.environ['PATH']}", "SM_RUST_BIN": str(self.bin),
                    "SM_GATE_PASSED": str(self.scratch / "passed"),
                    "CARGO_HOME": str(self.scratch / "no-cargo-home"), "SM_GATE_BASE": "HEAD", "SM_GATE_FRESH": ""}
        result = subprocess.run([sys.executable, "tools/lane/gate.py"], cwd=self.repo, env=os.environ | settings | env,
                                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, check=False)
        return result.returncode, result.stdout

    def ran_checks(self) -> list[str]:
        return sorted(record.name for record in self.ran.iterdir())

    def passes(self, **env: str) -> str:
        status, out = self.gate(**env)
        self.assertEqual(status, 0, out)
        self.assertRegex(out, r"(?m)^GATE OK ")
        return out

    def reuses(self, **env: str) -> bool:
        return "GATE REUSED: " in self.passes(**env)

    def settled(self, **env: str) -> None:
        self.assertTrue(self.reuses(**env) or self.reuses(**env), "a second gate with the same settings reused nothing")

    def test_an_unresolved_base_stops_before_any_check(self) -> None:
        status, out = self.gate(SM_GATE_BASE="no-such-base")
        self.assertEqual(status, 2, out)
        self.assertEqual(self.ran_checks(), [])

    def test_a_clean_branch_passes_and_its_tree_reuses_the_pass(self) -> None:
        self.passes()
        self.assertNotIn("verify_lib", self.ran_checks())
        self.commit("Same tree")
        out = self.passes()
        self.assertIn("GATE REUSED: ", out)
        self.assertRegex(out, rf"(?m)^GATE OK {self.git('rev-parse', '--short', 'HEAD')} ")
        self.assertEqual(self.ran_checks(), ["prereview"])
        self.passes(SM_GATE_FRESH="1")
        self.assertIn("cargo", self.ran_checks())

    def test_a_reused_pass_still_runs_the_review_findings(self) -> None:
        self.settled()
        self.commit("Same tree, a message the review findings refuse")
        status, out = self.gate(GATE_TEST_STATUS_prereview="1", GATE_TEST_OUT_prereview="invented dash finding")
        self.assertEqual(status, 1, out)
        self.assertIn("PREREVIEW FAIL", out.splitlines())
        self.assertIn("invented dash finding", out.splitlines())
        self.assertNotIn("GATE REUSED: ", out)
        self.assertEqual(self.ran_checks(), ["prereview"])

    def test_no_check_inherits_a_vector_file(self) -> None:
        (self.repo / "crates").mkdir()
        (self.repo / "crates/lib.rs").write_text("Invented source.\n")
        self.passes(SM_VECTOR_FILE=str(self.scratch / "stale.toml"))
        self.assertIn("verify_lib", self.ran_checks())
        self.assertNotIn("inherited", self.ran_checks())

    def test_other_compiler_flags_run_again(self) -> None:
        self.settled()
        self.assertFalse(self.reuses(RUSTFLAGS="--cap-lints=allow"))

    def test_a_missing_cargo_deny_runs_again_and_says_deny_was_skipped(self) -> None:
        self.settled()
        (self.bin / "cargo-deny").rename(self.bin / "no-cargo-deny")
        path = ":".join([str(self.bin)] + [entry for entry in os.environ["PATH"].split(":")
                                           if entry and not Path(entry, "cargo-deny").exists()])
        self.assertFalse(self.reuses(PATH=path))
        out = self.passes(PATH=path)
        self.assertIn("GATE REUSED: ", out)
        self.assertRegex(out, r"(?m)^DENY SKIPPED: ")

    def test_a_skip_worktree_file_turns_reuse_off(self) -> None:
        self.settled()
        self.git("update-index", "--skip-worktree", "scenarios/demo.toml")
        self.assertFalse(self.reuses())
        self.assertFalse(self.reuses())

    def test_what_the_checks_run_with_is_part_of_the_key(self) -> None:
        self.settled()
        (self.scratch / "rustfmt.toml").write_text("hard_tabs = true\n")
        self.assertFalse(self.reuses(), "a gate under a rustfmt.toml reused a pass")
        (self.scratch / "rustfmt.toml").unlink()
        self.settled()
        self.assertFalse(self.reuses(CLEANROOM_PY="/elsewhere/python"))

    def test_a_cargo_configuration_outside_the_tree_runs_again(self) -> None:
        self.settled()
        cargo = self.scratch / ".cargo"
        cargo.mkdir()
        (cargo / "config.toml").write_text("[build]\n")
        self.assertFalse(self.reuses(), "a gate under a parent .cargo/config.toml reused a pass")
        for include in CARGO_INCLUDES:
            (cargo / "config.toml").write_text(include + "\n")
            self.assertFalse(self.reuses() or self.reuses(), f"a configuration with {include} reused a pass")
        shutil.rmtree(cargo)
        self.settled()
        (self.scratch / "no-cargo-home").mkdir()
        (self.scratch / "no-cargo-home/config.toml").write_text("[build]\n")
        self.assertFalse(self.reuses(), "a gate with a CARGO_HOME configuration reused a pass")

    def test_rustfmt_configurations_outside_the_tree_run_again(self) -> None:
        homes = {"XDG_CONFIG_HOME": str(self.scratch / "xdg"), "HOME": str(self.scratch / "home")}
        self.settled(**homes)
        (self.scratch / "xdg/rustfmt").mkdir(parents=True)
        (self.scratch / "xdg/rustfmt/.rustfmt.toml").write_text("hard_tabs = true\n")
        self.assertFalse(self.reuses(**homes), "rustfmt's user configuration reused a pass")
        shutil.rmtree(self.scratch / "xdg")
        self.settled(**homes)
        (self.scratch / "home").mkdir()
        (self.scratch / "home/rustfmt.toml").write_text("hard_tabs = true\n")
        self.assertFalse(self.reuses(**homes), "a rustfmt.toml in the home reused a pass")
        (self.scratch / "home/rustfmt.toml").unlink()
        self.settled()
        (self.scratch / "rustfmt.toml").symlink_to("rustfmt.toml")
        self.assertFalse(self.reuses(), "a rustfmt.toml that is a link loop reused a pass")

    def test_a_cargo_configuration_above_temp_runs_again(self) -> None:
        elsewhere = Path(self.enterContext(tempfile.TemporaryDirectory(
            prefix="gate-test-temp-", dir=os.environ.get("TEMP") or os.environ.get("TMPDIR"))))
        (elsewhere / "tmp").mkdir()
        self.settled(TEMP=str(elsewhere / "tmp"))
        (elsewhere / ".cargo").mkdir()
        (elsewhere / ".cargo/config.toml").write_text("[build]\n")
        self.assertFalse(self.reuses(TEMP=str(elsewhere / "tmp")))

    def test_a_file_only_an_outside_ignore_rule_hides_turns_reuse_off(self) -> None:
        self.settled()
        (self.repo / "rustfmt.toml").write_text("hard_tabs = true\n")
        (self.scratch / "excludes").write_text("rustfmt.toml\n")
        excluded = {"GIT_CONFIG_COUNT": "1", "GIT_CONFIG_KEY_0": "core.excludesFile",
                    "GIT_CONFIG_VALUE_0": str(self.scratch / "excludes")}
        self.assertFalse(self.reuses(**excluded) or self.reuses(**excluded))

    def test_a_failed_gate_is_not_reused(self) -> None:
        (self.repo / "scenarios/rules.toml").write_text("Invented crate rules.\n")
        self.commit("Changed tree", "-A")
        status, out = self.gate(GATE_TEST_STATUS_crate_rules="1")
        self.assertNotEqual(status, 0, out)
        self.assertFalse(self.reuses())
        self.assertIn("crate_rules", self.ran_checks())

    def test_an_untracked_subject_file_runs_its_check(self) -> None:
        (self.repo / "scenarios/new").mkdir()
        (self.repo / "scenarios/new/demo.toml").write_text("Invented untracked vectors.\n")
        self.passes(GIT_CONFIG_COUNT="1", GIT_CONFIG_KEY_0="status.showUntrackedFiles", GIT_CONFIG_VALUE_0="no")
        self.assertIn("verify_lib", self.ran_checks())

    def test_a_failing_check_shows_its_log(self) -> None:
        (self.repo / "scenarios/new").mkdir()
        (self.repo / "scenarios/new/demo.toml").write_text("Invented untracked vectors.\n")
        (self.repo / "tools/cleanroom/adjusted_rule_owners").mkdir()
        (self.repo / "tools/cleanroom/adjusted_rule_owners/demo.toml").write_text("Invented owners.\n")
        status, out = self.gate(GATE_TEST_STATUS_verify_lib="1", GATE_TEST_OUT_verify_lib="invented link failure",
                                GATE_TEST_STATUS_owners="1", GATE_TEST_OUT_owners="invented decode error")
        self.assertNotEqual(status, 0, out)
        self.assertIn("invented link failure", out.splitlines())
        self.assertIn("invented decode error", out.splitlines())

    def test_a_failing_check_shows_its_finding_and_the_tools_tests_run_with_verify(self) -> None:
        (self.repo / "crates").mkdir()
        (self.repo / "crates/lib.rs").write_text("Invented source.\n")
        status, out = self.gate(GATE_TEST_STATUS_crate_rules="3", GATE_TEST_OUT_crate_rules="rule 5: invented hit")
        self.assertEqual(status, 1, out)
        self.assertEqual(out.splitlines()[:2], ["CRATE RULES FAIL", "rule 5: invented hit"])
        self.assertEqual((self.ran / "verify_lib").read_text(), "test -p tools")

    def test_the_clean_room_and_crate_rules_both_report_their_findings(self) -> None:
        status, out = self.gate(GATE_TEST_STATUS_clean_room="1", GATE_TEST_OUT_clean_room="invented clean-room hit",
                                GATE_TEST_STATUS_crate_rules="1", GATE_TEST_OUT_crate_rules="rule 4: invented hit")
        self.assertEqual(status, 1, out)
        lines = out.splitlines()
        for failure, finding in (("CLEAN ROOM FAIL", "invented clean-room hit"),
                                 ("CRATE RULES FAIL", "rule 4: invented hit")):
            self.assertEqual(lines[lines.index(failure) + 1], finding)

    def test_a_changed_domain_gates_its_adjusted_vectors(self) -> None:
        (self.repo / "docs/behaviours").mkdir(parents=True)
        (self.repo / "scenarios/vectors").mkdir()
        for path in ("docs/behaviours/demo.md", "docs/behaviours/alone.md"):
            (self.repo / path).write_text("Invented spec.\n")
        for path in ("scenarios/vectors/demo.toml", "scenarios/vectors/orphan.toml"):
            (self.repo / path).write_text("Invented vectors.\n")
        status, out = self.gate()
        self.assertNotEqual(status, 0, out)
        self.assertEqual((self.ran / "adjusted").read_text(), "--missing demo\n")
        self.assertIn("adjusted: demo: Entry has no adjusted vector", out.splitlines())


class SelfTestSelection(unittest.TestCase):
    def setUp(self) -> None:
        self.repo = Path(self.enterContext(tempfile.TemporaryDirectory(
            dir=os.environ.get("TEMP") or os.environ.get("TMPDIR"))))
        for path in (*gate.SUITES, *(subject for subjects in gate.SUITES.values() for subject in subjects),
                     "tools/lane/land.py", "tools/cleanroom/box_relay.py", "tools/lane/__init__.py"):
            (self.repo / path).parent.mkdir(parents=True, exist_ok=True)
            (self.repo / path).write_text("invented\n")
        self.git("init", "-q")
        for setting in ("user.name=Fixture", "user.email=fixture@example.invalid", "commit.gpgsign=false",
                        "status.showUntrackedFiles=no"):
            self.git("config", *setting.split("="))
        self.git("add", "-A")
        self.git("commit", "-qm", "base")
        self.base = self.git("rev-parse", "HEAD")
        self.enterContext(contextlib.chdir(self.repo))

    def git(self, *args: str) -> str:
        return subprocess.run(["git", *args], cwd=self.repo, check=True, capture_output=True, text=True).stdout.strip()

    def append(self, path: str) -> None:
        with (self.repo / path).open("a") as text:
            text.write("edited\n")

    def selected(self, base: str = "HEAD") -> list[str]:
        return sorted(Path(suite).stem for suite in gate.self_tests(base))

    def test_an_unchanged_branch_runs_nothing(self) -> None:
        self.assertEqual(self.selected(), [])

    def test_a_subject_edit_runs_its_suite_staged_unstaged_or_committed(self) -> None:
        self.append("tools/lane/gate.py")
        self.assertEqual(self.selected(), ["test_gate"])
        self.git("add", "tools/lane/gate.py")
        self.assertEqual(self.selected(), ["test_gate"])
        self.git("commit", "-qm", "Edit the gate")
        self.assertEqual(self.selected(), [])
        self.assertEqual(self.selected(self.base), ["test_gate"])

    def test_a_tool_with_its_own_tests_or_a_package_marker_runs_no_suite(self) -> None:
        for path in ("tools/lane/land.py", "tools/cleanroom/box_relay.py", "tools/lane/__init__.py"):
            with self.subTest(path=path):
                self.append(path)
                self.assertEqual(self.selected(), [])
                self.git("restore", "--", path)

    def test_an_adjusted_vector_input_runs_its_suite(self) -> None:
        self.append("tools/cleanroom/adjusted_vectors_run.py")
        self.assertEqual(self.selected(), ["test_adjusted_vectors_run"])
        self.append("tools/lane/changes.py")
        self.assertEqual(self.selected(), ["test_adjusted_vectors_run", "test_gate"])

    def test_an_unknown_tool_file_runs_every_suite(self) -> None:
        self.git("mv", "tools/lane/gate.py", "tools/lane/renamed.py")
        self.assertEqual(self.selected(), sorted(Path(suite).stem for suite in gate.SUITES))

    def test_a_subject_moved_out_of_its_folder_still_runs_its_suite(self) -> None:
        self.git("mv", "tools/lane/worktree.py", "removed-worktree.py")
        self.assertEqual(self.selected(), ["test_worktree"])


if __name__ == "__main__":
    unittest.main()
