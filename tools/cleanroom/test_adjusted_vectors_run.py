"""Only a vector whose answer changes at the reference rules proves its entry adjustable."""

import tempfile
import tomllib
import unittest
from collections.abc import Callable
from copy import deepcopy
from pathlib import Path
from unittest.mock import patch

from adjusted_vectors_run import (CannotProbe, Proven, Unadjusted, Unproven, Verdict,
                                  candidates, parameter_entries, probe, prove, test_command)

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
                return prove(path, parameter_entries(SPEC), model(path, step_rule, id_ends))
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
                                              "Margin": Proven("shared", ("Step",))})

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

            self.assertEqual(prove(path, ["Step"], fixed_count_adapter),
                             {"Step": Proven("adjusted", ())})
            self.assertEqual(observed, [(["reference", "adjusted"], True),
                                        (["reference", "adjusted-probe"], False),
                                        (["reference", "adjusted-probe"], True)])
            self.assertEqual(path.read_bytes(), original)

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

    def test_linux_runs_library_and_integration_tests(self) -> None:
        with patch("adjusted_vectors_run.os.name", "posix"):
            self.assertEqual(test_command("demo"),
                             ["cargo", "test", "--locked", "-p", "demo", "--lib", "--tests"])
        with patch("adjusted_vectors_run.os.name", "nt"):
            self.assertEqual(test_command("demo"), ["bash", "tools/lane/run_tests.sh", "demo"])


if __name__ == "__main__":
    unittest.main()
