import subprocess
import tempfile
import unittest
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path
from unittest.mock import patch

from check_mutants import (
    Fault,
    FrozenInputs,
    replay_selectable,
    report_rule_coverage,
    require_unique_anchor,
    scratch_files,
)
from selectable_faults import Replacement, instrument_file
from test_check_mutants import outcome


def change(source: bytes, ordinal: int, before: str, after: str) -> Replacement:
    fault = Fault(
        str(ordinal), "test", "test.rs", before, after, "case", 1, "field"
    )
    return Replacement(
        ordinal,
        require_unique_anchor(fault, source),
        before.encode(),
        after.encode(),
    )


class SelectableMutants(unittest.TestCase):
    def test_overlapping_anchors_keep_independent_literal_bodies(self) -> None:
        source = b"""
extern crate self as gamma_rt;
pub fn active() -> u32 { std::env::args().nth(1).unwrap().parse().unwrap() }
const fn inner(x: u8) -> u8 { let value = x + 1; value * 2 }
const fn outer(x: u8) -> u8 { inner(x) }
fn main() { println!("{}", outer(3)); }
"""
        variants = [
            change(
                source,
                1,
                "let value = x + 1;",
                "let next = x * 3; let value = next + 1;",
            ),
            change(source, 2, "x + 1", "x - 1"),
            change(source, 3, "x + 1", "x + 10"),
        ]
        generated = instrument_file(source, variants, ["outer"])
        with tempfile.TemporaryDirectory() as directory:
            executable = Path(directory) / "switch"
            subprocess.run(
                ["rustc", "--edition=2024", "-", "-o", str(executable)],
                input=generated,
                check=True,
            )
            scores = [
                subprocess.check_output(
                    [str(executable), str(ordinal)], text=True
                ).strip()
                for ordinal in (0, 1, 2, 3, 0)
            ]
        self.assertEqual(scores, ["8", "20", "4", "26", "8"])

    def test_refuses_edits_outside_a_body_and_overlapping_nested_functions(
        self,
    ) -> None:
        source = b"const N: u8 = 7; fn run() -> u8 { const fn inner() -> u8 { 2 } inner() + 3 }"
        for variants, callers in [
            ([change(source, 1, "N: u8 = 7", "N: u8 = 9")], []),
            ([change(source, 1, "inner() + 3", "inner() - 3")], ["inner"]),
            ([], ["missing"]),
        ]:
            with self.subTest(callers=callers), self.assertRaises(ValueError):
                instrument_file(source, variants, callers)

    def test_worker_validates_the_private_inputs_used_for_replay(self) -> None:
        case = "audio/example"
        fault = Fault(
            "sample",
            "audio",
            "source.rs",
            "old",
            "new",
            case,
            3,
            "audio.reply0",
        )
        with tempfile.TemporaryDirectory() as directory:
            tree = Path(directory)
            scenario = tree / "scenario.toml"
            scenario.write_text("original")
            frozen = FrozenInputs.capture(tree, scratch_files)

            def replay(
                root: Path, _: str, exe: Path, *, env: dict[str, str]
            ) -> tuple[int, str]:
                if env.get("GAMMA_ACTIVE"):
                    (root / scenario.name).write_text("changed")
                    return 1, outcome("difference")
                return 0, outcome("pass")

            with (
                patch("check_mutants.run_case", side_effect=replay) as calls,
                self.assertRaises(ValueError),
            ):
                replay_selectable(tree, (fault,), tree / "tools", frozen, {})
            self.assertEqual(calls.call_count, 2)

    def test_shared_executable_restores_in_a_fresh_process_and_credits_each_rule(
        self,
    ) -> None:
        case = "audio/example"
        faults = [
            Fault(
                name,
                "audio",
                "source.rs",
                "old",
                "new",
                case,
                3,
                "audio.reply0",
                name,
            )
            for name in ("first", "second")
        ]
        requests = []

        def replay(
            root: Path, case: str, exe: Path, *, env: dict[str, str]
        ) -> tuple[int, str]:
            selected = env.get("GAMMA_ACTIVE")
            requests.append((exe, selected))
            print(f"TIMING replay case={case} seconds=0.100000")
            return (
                (1, outcome("difference")) if selected else (0, outcome("pass"))
            )

        with tempfile.TemporaryDirectory() as directory:
            tree = Path(directory)
            exe = tree / "immutable"
            frozen = FrozenInputs.capture(tree, scratch_files)
            log = StringIO()
            with (
                patch("check_mutants.run_case", side_effect=replay),
                redirect_stdout(log),
            ):
                self.assertTrue(
                    replay_selectable(tree, faults, exe, frozen, {})
                )
            self.assertEqual(
                requests,
                [(exe, ordinal) for ordinal in (None, "1", None, "2", None)],
            )
            coverage = StringIO()
            with redirect_stdout(coverage):
                report_rule_coverage(
                    ["first", "second"], faults, [log.getvalue()]
                )
            self.assertEqual(
                coverage.getvalue().splitlines(),
                [
                    "first: killed through first",
                    "second: killed through second",
                    "Unassigned faults: 0",
                ],
            )
