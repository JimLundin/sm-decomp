import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
FAKE_PYTHON = """import os, pathlib, sys
pathlib.Path({ran!r}, "{name}").write_text("")
print(os.environ.get("GATE_TEST_OUT_{name}", ""))
sys.exit(int(os.environ.get("GATE_TEST_STATUS_{name}", "0")))
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
        for name in ("gate.sh", "changes.sh"):
            shutil.copy(HERE / name, self.repo / "tools/lane" / name)
        (self.repo / "rust-toolchain.toml").write_text('channel = "0.0.0"\n')
        (self.repo / "scenarios/demo.toml").write_text("Invented vectors.\n")
        if dash := shutil.which("dash"):
            self.executable(self.bin / "sh", f'#!/bin/sh\nexec "{dash}" "$@"\n')
        self.fake("cargo", self.bin / "cargo")
        self.fake("deny", self.bin / "cargo-deny")
        self.fake("self_tests", self.repo / "tools/lane/self_tests.sh")
        self.fake("verify_lib", self.repo / "tools/lane/run_tests.sh")
        for name, script in (("faults", "tools/check_mutants.py"), ("owners", "tools/cleanroom/adjusted_vectors_run.py"),
                             ("prereview", "tools/lane/prereview.py")):
            (self.repo / script).write_text(FAKE_PYTHON.format(ran=str(self.ran), name=name))
        self.git("init", "-q", "-b", "lane/test")
        self.commit("Invented tree", "-A")

    def executable(self, path: Path, text: str) -> None:
        path.write_text(text)
        path.chmod(0o755)

    def fake(self, name: str, path: Path) -> None:
        if name == "cargo":
            name = '$(case "$*" in *crate-rules*) echo crate_rules ;; *doc-links*) echo doc_links ;; *) echo cargo ;; esac)'
            self.executable(path, f'#!/bin/sh\nname={name}\necho >"{self.ran}/$name"\n'
                                  'eval "out=\\${GATE_TEST_OUT_$name-} status=\\${GATE_TEST_STATUS_$name:-0}"\n'
                                  '[ -z "$out" ] || echo "$out"\nexit "$status"\n')
            return
        self.executable(path, f'#!/bin/sh\necho >"{self.ran}/{name}"\n'
                              f'[ -z "${{GATE_TEST_OUT_{name}-}}" ] || echo "$GATE_TEST_OUT_{name}"\n'
                              f'exit "${{GATE_TEST_STATUS_{name}:-0}}"\n')

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
                    "SM_GATE_DIR": str(self.scratch / "gate"), "SM_GATE_PASSED": str(self.scratch / "passed"),
                    "CARGO_HOME": str(self.scratch / "no-cargo-home"), "SM_GATE_BASE": "HEAD"}
        result = subprocess.run(["bash", "tools/lane/gate.sh"], cwd=self.repo, env=os.environ | settings | env,
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
        self.settled()
        self.fake("shellcheck", self.bin / "shellcheck")
        self.assertFalse(self.reuses(), "a gate with shellcheck installed reused a pass made without it")
        (self.bin / "shellcheck").unlink()

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

    def test_a_changed_domain_gates_its_adjusted_vectors(self) -> None:
        self.executable(self.repo / "tools/cleanroom/adjusted_vectors.sh",
                        f'#!/bin/sh\necho "$*" >"{self.ran}/adjusted"\n'
                        'echo "adjusted: $2: Entry has no adjusted vector"\nexit 1\n')
        (self.repo / "docs/behaviours").mkdir(parents=True)
        (self.repo / "scenarios/vectors").mkdir()
        for path in ("docs/behaviours/demo.md", "docs/behaviours/alone.md"):
            (self.repo / path).write_text("Invented spec.\n")
        for path in ("scenarios/vectors/demo.toml", "scenarios/vectors/orphan.toml"):
            (self.repo / path).write_text("Invented vectors.\n")
        status, out = self.gate()
        self.assertNotEqual(status, 0, out)
        self.assertEqual((self.ran / "adjusted").read_text(), "--gate demo\n")
        self.assertIn("adjusted: demo: Entry has no adjusted vector", out.splitlines())


if __name__ == "__main__":
    unittest.main()
