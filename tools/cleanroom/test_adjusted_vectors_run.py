"""Only a vector whose answer changes at the reference rules proves its entry adjustable."""

import contextlib
import io
import subprocess
import sys
import tempfile
import tomllib
import unittest
from collections.abc import Callable
from copy import deepcopy
from pathlib import Path
from unittest.mock import patch
from typing import Any

from adjusted_vectors_run import (CannotProbe, Proven, Unadjusted, Unproven, Verdict,
                                  candidates, check_domain, check_domain_owners, completed_test, main, parameter_entries, probe, prove, probe_test, rule_owners, test_command)

SPEC = """\
### Fixed
- switch: none.

### Step
- switch: **Parameter**, the step (params).

### Margin
- switch: parameter.
"""

REFERENCE = """\
[[vector]]
id = "reference"
entries = ["Fixed", "Step", "Margin"]
before = 2
after = 3
"""


def vector(body: str) -> str:
    return REFERENCE + "\n[[vector]]\n" + body


def model(path: Path, step_rule: str = "step", id_ends: str = "") -> Callable[[], bool]:
    """An invented model: a vector passes when its after is its before plus the step, 1 unless
    its rules say otherwise; `step_rule` names the rule the model reads, and every id must end
    with `id_ends`."""

    def run() -> bool:
        rows = tomllib.loads(path.read_text(encoding="utf-8"))["vector"]
        ids = [row["id"] for row in rows]
        return len(set(ids)) == len(ids) and all(i.endswith(id_ends) for i in ids) and all(
            row["after"] == row["before"] + row.get("rules", {}).get(step_rule, 1)
            for row in rows
        )

    return run


class AdjustedVectors(unittest.TestCase):
    def proof(self, vectors: str, step_rule: str = "step",
              id_ends: str = "") -> dict[str, Verdict]:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "demo.toml"
            path.write_text(vectors, encoding="utf-8", newline="\n")
            try:
                return prove(path, parameter_entries(SPEC), model(path, step_rule, id_ends),
                             {"step": frozenset({"Step"}), "margin": frozenset({"Margin"})})
            finally:
                self.assertEqual(path.read_text(encoding="utf-8"), vectors)

    def test_the_spec_names_its_parameter_entries(self) -> None:
        self.assertEqual(parameter_entries(SPEC), ["Step", "Margin"])

    def test_a_vector_that_fails_at_the_reference_rules_proves_its_entry(self) -> None:
        inline = vector('id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\n'
                        "before = 2\nafter = 5\n")
        self.assertEqual(self.proof(inline), {"Step": Proven("adjusted", ()), "Margin": Unadjusted()})
        table = vector('id = "adjusted"\nentries = [\n  "Step",\n]\nbefore = 2\nafter = 5\n'
                       "\n[vector.rules]\nstep = 3\n")
        self.assertEqual(self.proof(table), {"Step": Proven("adjusted", ()), "Margin": Unadjusted()})

    def test_empty_rules_adjust_nothing(self) -> None:
        empty = vector('id = "empty"\nentries = ["Step"]\nrules = {}\nbefore = 2\nafter = 3\n')
        self.assertEqual(self.proof(empty), {"Step": Unadjusted(), "Margin": Unadjusted()})

    def test_the_reference_value_proves_nothing(self) -> None:
        same = vector('id = "same"\nentries = ["Step"]\nrules = { step = 1 }\n'
                      "before = 2\nafter = 3\n")
        self.assertEqual(self.proof(same), {"Step": Unproven(), "Margin": Unadjusted()})

    def test_a_rule_the_case_does_not_reach_proves_nothing(self) -> None:
        other = vector('id = "other"\nentries = ["Step"]\nrules = { margin = 9 }\n'
                       "before = 2\nafter = 3\n")
        self.assertEqual(self.proof(other), {"Step": Unproven(), "Margin": Unadjusted()})

    def test_a_later_vector_proves_what_an_earlier_one_does_not(self) -> None:
        both = vector('id = "same"\nentries = ["Step"]\nrules = { step = 1 }\n'
                      "before = 2\nafter = 3\n\n[[vector]]\n"
                      'id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\n'
                      "before = 2\nafter = 5\n")
        self.assertEqual(self.proof(both),
                         {"Step": Proven("adjusted", ()), "Margin": Unadjusted()})

    def test_a_vector_of_its_own_is_preferred_and_a_shared_one_says_so(self) -> None:
        shared = vector('id = "shared"\nentries = ["Step", "Margin"]\nrules = { step = 3 }\n'
                        "before = 2\nafter = 5\n\n[[vector]]\n"
                        'id = "own"\nentries = ["Fixed", "Step"]\nrules = { step = 4 }\n'
                        "before = 2\nafter = 6\n")
        self.assertEqual(self.proof(shared), {"Step": Proven("own", ()),
                                              "Margin": Unproven()})

    def test_a_model_that_ignores_the_rule_fails_before_any_probe(self) -> None:
        adjusted = vector('id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\n'
                          "before = 2\nafter = 5\n")
        with self.assertRaises(CannotProbe):
            self.proof(adjusted, step_rule="ignored")

    def test_a_test_that_refuses_the_copy_itself_proves_nothing(self) -> None:
        strict = vector('id = "x-d"\nentries = ["Step"]\nrules = { step = 3 }\n'
                        "before = 2\nafter = 5\n").replace('"reference"', '"x-d"', 1)
        strict = strict.replace('id = "x-d"\nentries = ["Step"]', 'id = "y-d"\nentries = ["Step"]')
        with self.assertRaises(CannotProbe):
            self.proof(strict, id_ends="-d")

    def test_a_fixed_count_adapter_accepts_control_and_refutes_reference(self) -> None:
        text = vector('id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\n'
                      "before = 2\nafter = 5\n")
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "demo.toml"
            original = text.encode("utf-8")
            path.write_bytes(original)
            run_model = model(path)
            observed: list[tuple[list[str], bool]] = []

            def fixed_count_adapter() -> bool:
                rows = tomllib.loads(path.read_text(encoding="utf-8"))["vector"]
                passed = len(rows) == 2 and run_model()
                observed.append(([row["id"] for row in rows], passed))
                return passed

            self.assertEqual(prove(path, ["Step"], fixed_count_adapter, {"step": frozenset({"Step"})}),
                             {"Step": Proven("adjusted", ())})
            self.assertEqual(observed, [(["reference", "adjusted"], True),
                                        (["reference", "adjusted-probe"], False),
                                        (["reference", "adjusted-probe"], True)])
            self.assertEqual(path.read_bytes(), original)

    def test_dependent_failure_cannot_credit_an_ignored_direct_parameter(self) -> None:
        for step, after, expected in ((3, 3, Unproven()), (3, 5, Proven("adjusted", ()))):
            text = vector(f'id = "adjusted"\nentries = ["Step"]\nrules = {{ step = {step} }}\n'
                          f'before = 2\nafter = {after}\n')
            text += f'[[dependent]]\nbase = "adjusted"\nrules = {{ step = {step} }}\n'
            with tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "demo.toml"
                original = text.replace("\n", "\r\n").encode()
                path.write_bytes(original)
                calls = []
                def complete() -> bool:
                    calls.append("complete")
                    data = tomllib.loads(path.read_text())
                    v = data["vector"][1]
                    dep = data["dependent"][0]
                    return model(path, "ignored" if after == 3 else "step")() and dep["base"] == v["id"] and dep["rules"] == v["rules"]
                def direct() -> bool:
                    calls.append("direct")
                    data = tomllib.loads(path.read_text())
                    self.assertEqual(data["dependent"], tomllib.loads(text)["dependent"])
                    return model(path, "ignored" if after == 3 else "step")()
                self.assertEqual(prove(path, ["Step"], complete,
                                       {"step": frozenset({"Step"})}, direct), {"Step": expected})
                self.assertEqual(calls[:2], ["complete", "direct"])
                self.assertEqual(calls.count("complete"), 1)
                self.assertEqual(path.read_bytes(), original)

    def test_failed_direct_baseline_and_runner_error_restore_original_bytes(self) -> None:
        text = vector('id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\n'
                      'before = 2\nafter = 5\n')
        for failure_call in (1, 2, 3):
            with tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "demo.toml"
                original = text.encode()
                path.write_bytes(original)
                calls = 0
                def direct() -> bool:
                    nonlocal calls
                    calls += 1
                    if calls == failure_call:
                        raise CannotProbe("runner failure")
                    return model(path)()
                with self.assertRaises(CannotProbe):
                    prove(path, ["Step"], model(path), {"step": frozenset({"Step"})}, direct)
                self.assertEqual(path.read_bytes(), original)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "demo.toml"
            path.write_bytes(text.encode())
            with self.assertRaises(CannotProbe):
                prove(path, ["Step"], model(path), {"step": frozenset({"Step"})}, lambda: False)
            self.assertEqual(path.read_bytes(), text.encode())

    def test_exact_selection_requires_one_named_nonignored_completed_test(self) -> None:
        good = (b"running 1 test\ntest direct ... ok\n"
                b"test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s\n")
        self.assertTrue(completed_test(subprocess.CompletedProcess([], 0, good), 101, "direct"))
        failed = good.replace(b"... ok", b"... FAILED").replace(b"result: ok", b"result: FAILED").replace(b"1 passed", b"0 passed").replace(b"0 failed", b"1 failed")
        self.assertFalse(completed_test(subprocess.CompletedProcess([], 101, failed), 101, "direct"))
        for output in (good.replace(b"direct", b"wrong"),
                       good.replace(b"direct ... ok", b"direct ... ignored").replace(b"1 passed", b"0 passed").replace(b"0 ignored", b"1 ignored"),
                       good.replace(b"running 1", b"running 0").replace(b"1 passed", b"0 passed"),
                       good.replace(b"test direct ... ok\n", b"")):
            with self.subTest(output=output), self.assertRaises(CannotProbe):
                completed_test(subprocess.CompletedProcess([], 0, output), 101, "direct")

    def test_domain_runs_full_baseline_then_exact_direct_baseline_and_probes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            spec = root / "docs/behaviours/demo.md"
            spec.parent.mkdir(parents=True)
            spec.write_text("### Step\n- switch: Parameter.\n", newline="\n")
            metadata = root / "tools/cleanroom/adjusted_rule_owners/demo.toml"
            metadata.parent.mkdir(parents=True)
            metadata.write_text('[owners]\nstep = ["Step"]\n', newline="\n")
            (metadata.parent.parent / "adjusted_probe_tests.toml").write_text(
                '[tests]\ndemo = "direct"\n', newline="\n")
            path = root / "scenarios/vectors/demo.toml"
            path.parent.mkdir(parents=True)
            text = vector('id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\n'
                          'before = 2\nafter = 5\n') + '[[dependent]]\nbase = "adjusted"\n'
            path.write_bytes(text.encode())
            calls = []
            def runner(command: list[str], **kwargs: Any) -> subprocess.CompletedProcess[bytes]:
                calls.append((command, path.read_bytes()))
                passed = model(path)()
                status = "ok" if passed else "FAILED"
                output = (f"running 1 test\ntest direct ... {status}\n"
                          f"test result: {status}. {int(passed)} passed; {int(not passed)} failed; "
                          "0 ignored; 0 measured; 2 filtered out; finished in 0.00s\n").encode()
                return subprocess.CompletedProcess(command, 0 if passed else 101, output)
            with patch("adjusted_vectors_run.test_command", return_value=["runner"]), \
                 patch("adjusted_vectors_run.subprocess.run", side_effect=runner):
                self.assertEqual(check_domain(root, "demo"), 0)
            self.assertEqual([c for c, _ in calls], [["runner"]] +
                             [["runner", "--", "direct", "--exact"]] * 3)
            self.assertEqual([b for _, b in calls[:2]], [text.encode()] * 2)
            self.assertEqual(path.read_bytes(), text.encode())

    def test_linked_files_require_valid_reviewed_test_selection(self) -> None:
        text = REFERENCE + '[[dependent]]\nbase = "reference"\n'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.assertIsNone(probe_test(root, "demo", REFERENCE))
            with self.assertRaises(CannotProbe):
                probe_test(root, "demo", text)
            path = root / "tools/cleanroom/adjusted_probe_tests.toml"
            path.parent.mkdir(parents=True)
            for metadata in ('[tests]\ndemo = ""\n', '[tests]\ndemo = 3\n',
                             '[tests]\ndemo = "direct --ignored"\n'):
                path.write_text(metadata, newline="\n")
                with self.assertRaises(CannotProbe):
                    probe_test(root, "demo", text)
            path.write_text('[tests]\ndemo = "vectors::direct"\n', newline="\n")
            self.assertEqual(probe_test(root, "demo", text), "vectors::direct")

    def test_replacement_preserves_all_other_parsed_content(self) -> None:
        for rules in ('rules = { step = 3 }\n', '[vector.rules]\nstep = 3\n'):
            text = ('title = "globals"\n[settings]\ncount = 7\n'
                    '[[annotation]]\nid = "before"\n' + REFERENCE
                    + '\n[[vector]]\nid = "adjusted"\nentries = ["Step"]\n'
                    'before = 2\nafter = 5\n' + rules
                    + '[vector.note]\nid = "nested-id"\ntext = "keep"\n'
                    '[[annotation]]\nid = "between"\n'
                    '[[vector]]\nid = "later"\nentries = ["Fixed"]\n'
                    'before = 5\nafter = 6\n'
                    '[[annotation]]\nid = "after"\n')
            original = tomllib.loads(text)
            candidate = candidates(text)[0]
            for keep_rules in (False, True):
                with self.subTest(rules=rules, keep_rules=keep_rules):
                    expected = deepcopy(original)
                    expected["vector"][1]["id"] = "adjusted-probe"
                    if not keep_rules:
                        del expected["vector"][1]["rules"]
                    self.assertEqual(tomllib.loads(probe(text, candidate, keep_rules)), expected)

    def test_a_probe_id_collision_is_refused_and_restored(self) -> None:
        text = vector('id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\n'
                      "before = 2\nafter = 5\n").replace('"reference"', '"adjusted-probe"')
        with self.assertRaises(CannotProbe):
            self.proof(text)

    def test_two_overrides_do_not_credit_an_ignored_parameter(self) -> None:
        text = vector('id = "shared"\nentries = ["Step", "Margin"]\n'
                      'rules = { step = 3, margin = 9 }\nbefore = 2\nafter = 5\n')
        self.assertEqual(self.proof(text), {"Step": Proven("shared", ("Margin",)),
                                           "Margin": Unproven()})

    def test_shared_vector_proves_each_executed_parameter_separately(self) -> None:
        text = vector('id = "shared"\nentries = ["Step", "Margin"]\n'
                      'rules = { step = 3, margin = 9 }\nbefore = 2\nafter = 14\n')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "demo.toml"
            path.write_text(text, encoding="utf-8", newline="\n")
            probes: list[dict[str, int]] = []

            def run() -> bool:
                rows = tomllib.loads(path.read_text(encoding="utf-8"))["vector"]
                rules = rows[1].get("rules", {})
                probes.append(rules)
                return len(rows) == 2 and all(
                    row["after"] == row["before"] + row.get("rules", {}).get("step", 1)
                    + row.get("rules", {}).get("margin", 0) for row in rows)

            self.assertEqual(prove(path, ["Step", "Margin"], run,
                                   {"step": frozenset({"Step"}), "margin": frozenset({"Margin"})}),
                             {"Step": Proven("shared", ("Margin",)),
                              "Margin": Proven("shared", ("Step",))})
            self.assertIn({"margin": 9}, probes)
            self.assertIn({"step": 3}, probes)
            self.assertEqual(path.read_text(encoding="utf-8"), text)

    def test_missing_unknown_or_ambiguous_ownership_refuses_and_restores(self) -> None:
        text = vector('id = "shared"\nentries = ["Step", "Margin"]\n'
                      'rules = { step = 3, margin = 9 }\nbefore = 2\nafter = 5\n')
        for owners in ({}, {"step": frozenset({"Step"}), "margin": frozenset({"Unknown"})},
                       {"step": frozenset({"Step"}), "margin": frozenset({"Step", "Margin"})}):
            with self.subTest(owners=owners), tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "demo.toml"
                original = text.encode("utf-8")
                path.write_bytes(original)
                with self.assertRaises(CannotProbe):
                    prove(path, ["Step", "Margin"], model(path), owners)
                self.assertEqual(path.read_bytes(), original)

    def test_the_owners_check_refuses_an_unowned_override_and_runs_no_test(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            spec = root / "docs/behaviours/demo.md"
            spec.parent.mkdir(parents=True)
            spec.write_text(SPEC, newline="\n")
            path = root / "scenarios/vectors/demo.toml"
            path.parent.mkdir(parents=True)
            path.write_text(vector('id = "shared"\nentries = ["Step", "Margin"]\n'
                                   'rules = { step = 3, margin = 9 }\nbefore = 2\nafter = 5\n'),
                            newline="\n")
            metadata = root / "tools/cleanroom/adjusted_rule_owners/demo.toml"
            metadata.parent.mkdir(parents=True)
            with patch("adjusted_vectors_run.subprocess.run") as run:
                for owners, refused in (('step = ["Step"]\n', True),
                                        ('step = ["Step"]\nmargin = ["Margin"]\n', False)):
                    with self.subTest(owners=owners):
                        metadata.write_text("[owners]\n" + owners, newline="\n")
                        if refused:
                            with self.assertRaises(CannotProbe):
                                check_domain_owners(root, "demo")
                        else:
                            check_domain_owners(root, "demo")
                run.assert_not_called()

    def test_a_shared_rule_owner_is_proven_only_by_separate_entry_witnesses(self) -> None:
        text = vector('id = "step"\nentries = ["Step"]\nrules = { step = 3 }\n'
                      'before = 2\nafter = 5\n\n[[vector]]\n'
                      'id = "margin"\nentries = ["Margin"]\nrules = { step = 4 }\n'
                      'before = 2\nafter = 6\n')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "demo.toml"
            original = text.encode("utf-8")
            path.write_bytes(original)
            self.assertEqual(prove(path, ["Step", "Margin"], model(path),
                                   {"step": frozenset({"Step", "Margin"})}),
                             {"Step": Proven("step", ()), "Margin": Proven("margin", ())})
            self.assertEqual(path.read_bytes(), original)

    def test_runner_error_after_a_probe_propagates_and_restores(self) -> None:
        text = vector('id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\n'
                      'before = 2\nafter = 5\n')
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "demo.toml"
            original = text.encode("utf-8")
            path.write_bytes(original)
            calls = 0

            def run() -> bool:
                nonlocal calls
                calls += 1
                if calls == 2:
                    raise OSError()
                return True

            with self.assertRaises(OSError):
                prove(path, ["Step"], run, {"step": frozenset({"Step"})})
            self.assertEqual(path.read_bytes(), original)

    def test_selective_replacement_preserves_nested_unrelated_overrides(self) -> None:
        for rules in ('rules = { step = 3, margin = { values = [9, 8] } }\n',
                      '[vector.rules]\nstep = 3\n[vector.rules.margin]\nvalues = [9, 8]\n'):
            text = vector('id = "adjusted"\nentries = ["Step", "Margin"]\n'
                          'before = 2\nafter = 5\n' + rules)
            expected = deepcopy(tomllib.loads(text))
            expected["vector"][1]["id"] = "adjusted-probe"
            del expected["vector"][1]["rules"]["step"]
            self.assertEqual(tomllib.loads(probe(text, candidates(text)[0], False,
                                                frozenset({"step"}))), expected)

    def test_ownership_metadata_is_required_only_when_adjusted_candidates_exist(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.assertEqual(rule_owners(root, "demo", False), {})
            with self.assertRaises(CannotProbe):
                rule_owners(root, "demo", True)
            metadata = root / "tools/cleanroom/adjusted_rule_owners/demo.toml"
            metadata.parent.mkdir(parents=True)
            for text in ('[owners]\nstep = "Step"\n', '[owners]\nstep = [3]\n'):
                metadata.write_text(text, encoding="utf-8", newline="\n")
                with self.assertRaises(CannotProbe):
                    rule_owners(root, "demo", True)
            metadata.write_text('[owners]\nstep = ["Step"]\n', encoding="utf-8", newline="\n")
            self.assertEqual(rule_owners(root, "demo", True), {"step": frozenset({"Step"})})

    def test_completed_nonempty_test_results_are_the_only_model_answers(self) -> None:
        passed = b"running 2 tests\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
        failed = b"running 2 tests\ntest result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
        for failure_code in (1, 101):
            self.assertTrue(completed_test(subprocess.CompletedProcess([], 0, passed), failure_code))
            self.assertTrue(completed_test(subprocess.CompletedProcess([], 0, passed.replace(b"\n", b"\r\n")), failure_code))
            self.assertFalse(completed_test(subprocess.CompletedProcess([], failure_code, failed), failure_code))
            for code, output in ((126, failed), (2, failed), (-9, failed), (1, passed), (0, failed),
                                 (0, b""), (0, passed + passed),
                                 (0, passed.replace(b"running 2", b"running 3")),
                                 (0, passed.replace(b"running 2", b"running 0").replace(b"2 passed", b"0 passed")),
                                 (0, passed.replace(b"2 passed", b"0 passed").replace(b"0 ignored", b"2 ignored"))):
                with self.subTest(code=code, output=output), self.assertRaises(CannotProbe):
                    completed_test(subprocess.CompletedProcess([], code, output), failure_code)

    def test_reference_only_subprocess_launch_failure_cannot_prove_a_parameter(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            spec = root / "docs/behaviours/demo.md"
            spec.parent.mkdir(parents=True)
            spec.write_text("### Step\n- switch: Parameter.\n", encoding="utf-8", newline="\n")
            metadata = root / "tools/cleanroom/adjusted_rule_owners/demo.toml"
            metadata.parent.mkdir(parents=True)
            metadata.write_text('[owners]\nstep = ["Step"]\n', encoding="utf-8", newline="\n")
            path = root / "scenarios/vectors/demo.toml"
            path.parent.mkdir(parents=True)
            text = vector('id = "adjusted"\nentries = ["Step"]\nrules = { step = 1 }\n'
                          'before = 2\nafter = 3\n')
            path.write_text(text, encoding="utf-8", newline="\n")
            runner = root / "runner.py"
            receipt = root / "receipt.txt"
            runner.write_text("import sys,tomllib\nfrom pathlib import Path\n"
                              "row=tomllib.loads(Path(sys.argv[1]).read_text())['vector'][1]\n"
                              "with Path(sys.argv[2]).open('a') as receipt: receipt.write(str('rules' in row)+chr(10))\n"
                              "if 'rules' not in row: sys.exit(126)\n"
                              "print('running 1 test')\n"
                              "print('test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s')\n",
                              encoding="utf-8", newline="\n")
            with patch("adjusted_vectors_run.test_command", return_value=[sys.executable, str(runner), str(path), str(receipt)]):
                with self.assertRaises(CannotProbe):
                    check_domain(root, "demo")
            self.assertEqual(receipt.read_text(encoding="utf-8").splitlines(), ["True", "False"])
            self.assertEqual(path.read_text(encoding="utf-8"), text)
            with patch("adjusted_vectors_run.subprocess.run", side_effect=OSError()):
                with self.assertRaises(CannotProbe):
                    check_domain(root, "demo")
            self.assertEqual(path.read_text(encoding="utf-8"), text)

    def test_linux_selects_dedicated_vectors_or_library_adapter(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tests = root / "crates/clean/demo/tests"
            tests.mkdir(parents=True)
            self.assertEqual(test_command("demo", root),
                             ["cargo", "test", "--locked", "-p", "demo", "--lib"])
            # An unrelated integration target doesn't replace a unit-based adapter.
            (tests / "other.rs").write_text("// Invented target.\n", encoding="utf-8")
            self.assertEqual(test_command("demo", root)[-1], "--lib")
            (tests / "vectors.rs").write_text("// Invented vector adapter.\n", encoding="utf-8")
            self.assertEqual(test_command("demo", root),
                             ["cargo", "test", "--locked", "-p", "demo", "--test", "vectors"])

    def test_nested_vectors_select_the_integration_target(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tests = root / "crates/clean/demo/tests/vectors"
            tests.mkdir(parents=True)
            (tests / "main.rs").write_text("// Invented vector adapter.\n", encoding="utf-8")
            self.assertEqual(test_command("demo", root),
                             ["cargo", "test", "--locked", "-p", "demo", "--test", "vectors"])

    def test_owners_selects_the_check_and_its_domains(self) -> None:
        for arguments, owners, probed in ((["--owners"], ["alpha", "beta"], []),
                                          (["--owners", "beta"], ["beta"], []),
                                          ([], [], ["alpha", "beta"]),
                                          (["beta"], [], ["beta"])):
            with self.subTest(arguments=arguments), \
                    patch("adjusted_vectors_run.domains", return_value=["alpha", "beta"]), \
                    patch("adjusted_vectors_run.check_domain_owners") as owners_check, \
                    patch("adjusted_vectors_run.check_domain", return_value=0) as probe_check:
                self.assertEqual(main(arguments), 0)
                self.assertEqual([c.args[1] for c in owners_check.call_args_list], owners)
                self.assertEqual([c.args[1] for c in probe_check.call_args_list], probed)

    def test_a_toml_error_in_the_owners_or_the_vectors_is_a_refusal(self) -> None:
        owners = "tools/cleanroom/adjusted_rule_owners/demo.toml"
        valid = {"docs/behaviours/demo.md": SPEC,
                 "scenarios/vectors/demo.toml": vector('id = "step"\nentries = ["Step"]\n'
                                                       'rules = { step = 3 }\nbefore = 2\nafter = 5\n'),
                 owners: '[owners]\nstep = ["Step"]\n'}
        for broken in (None, owners, "scenarios/vectors/demo.toml",
                       "tools/cleanroom/adjusted_probe_tests.toml"):
            with self.subTest(broken=broken), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                for name, text in valid.items():
                    (root / name).parent.mkdir(parents=True, exist_ok=True)
                    (root / name).write_text(text, encoding="utf-8", newline="\n")
                if broken is not None:
                    (root / broken).write_text("[owners\n", encoding="utf-8", newline="\n")
                errors = io.StringIO()
                with contextlib.chdir(root), contextlib.redirect_stdout(io.StringIO()), \
                        contextlib.redirect_stderr(errors):
                    status = main(["--owners", "demo"])
                # 2 is the tool's own refusal, which it prints with its name.
                self.assertEqual(status, 0 if broken is None else 2, errors.getvalue())
                self.assertEqual(bool(errors.getvalue()), broken is not None)


if __name__ == "__main__":
    unittest.main()
