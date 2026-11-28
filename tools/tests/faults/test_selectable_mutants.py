from concurrent.futures import Future
from itertools import count
lazy from collections.abc import Mapping, Sequence
lazy from pathlib import Path

import pytest
from ordered import FrozenList

from sm_tools.faults.core.check_mutants import (
    Fault,
    Replay,
    receipt,
    receipt_of,
    require_unique_anchor,
    rule_coverage,
)
from sm_tools.faults.core.selectable_faults import Replacement, instrument_file
from sm_tools.faults.shell import check_mutants
from sm_tools.faults.shell.check_mutants import (
    Build,
    capture,
    replay_selectable,
    scratch_files,
)
from sm_tools.faults.shell.rust_syntax import parse
from sm_tools.shell import process
from tests.faults.outcomes import outcome

CASE = "audio/example"


def change(source: bytes, ordinal: int, before: str, after: str) -> Replacement:
    fault = Fault(str(ordinal), "test", "test.rs", before, after, "case")
    return Replacement(
        ordinal,
        require_unique_anchor(fault, source),
        before.encode(),
        after.encode(),
    )


def instrument(
    source: bytes, variants: Sequence[Replacement], callers: Sequence[str]
) -> bytes:
    return instrument_file(source, parse(source), variants, callers)


def test_overlapping_anchors_keep_independent_literal_bodies(
    tmp_path: Path,
) -> None:
    source = b"""
extern crate self as gamma_rt;
pub fn active() -> u32 { std::env::args().nth(1).unwrap().parse().unwrap() }
const fn inner(x: u8) -> u8 { let value = x + 1; value * 2 }
const fn outer(x: u8) -> u8 { inner(x) }
fn main() { println!("{}", outer(3)); }
"""
    variants = (
        change(
            source,
            1,
            "let value = x + 1;",
            "let next = x * 3; let value = next + 1;",
        ),
        change(source, 2, "x + 1", "x - 1"),
        change(source, 3, "x + 1", "x + 10"),
    )
    generated = instrument(source, variants, ("outer",))
    executable = tmp_path / "switch"
    _ = process.checked(
        ("rustc", "--edition=2024", "-", "-o", str(executable)),
        cwd=tmp_path,
        stdin=generated,
    )
    scores = FrozenList(
        process
        .checked((str(executable), str(ordinal)), cwd=tmp_path)
        .decode()
        .strip()
        for ordinal in (0, 1, 2, 3, 0)
    )
    assert scores == ("8", "20", "4", "26", "8")


def test_refuses_edits_outside_a_body_and_overlapping_nested_functions() -> (
    None
):
    source = (
        b"const N: u8 = 7; fn run() -> u8 {"
        b" const fn inner() -> u8 { 2 } inner() + 3 }"
    )
    for variants, callers, refusal in (
        (
            (change(source, 1, "N: u8 = 7", "N: u8 = 9"),),
            (),
            "must change a Rust function body",
        ),
        (
            (change(source, 1, "inner() + 3", "inner() - 3"),),
            ("inner",),
            "overlapping Rust function bodies",
        ),
        ((), ("missing",), r"missing const caller\(s\): \['missing'\]"),
    ):
        with pytest.raises(ValueError, match=refusal):
            _ = instrument(source, variants, callers)
    nested = instrument(source, (change(source, 1, "{ 2 }", "{ 4 }"),), ())
    guarded = b"fn inner() -> u8 { match ::gamma_rt::active() { 1 => { 4 }, _ => { 2 } } }"
    assert nested == source.replace(
        b"const fn inner() -> u8 { 2 }", b" " + guarded
    )


def launches_until_refused(
    tree: Path, monkeypatch: pytest.MonkeyPatch, edit_at: int, phase: str
) -> int:
    fault = Fault("sample", "audio", "source.rs", "old", "new", CASE)
    tree.mkdir()
    scenario = tree / "scenario.toml"
    _ = scenario.write_text("original")
    frozen = capture(tree, scratch_files)
    launches = count(1)

    def replay(
        _root: Path,
        _case: str,
        _executable: Path,
        environment: Mapping[str, str],
    ) -> Replay:
        if next(launches) == edit_at:
            _ = scenario.write_text("changed")
        if environment.get("GAMMA_ACTIVE"):
            return Replay(1, outcome("difference"))
        return Replay(0, outcome("pass"))

    monkeypatch.setattr(check_mutants, "run_case", replay)
    with pytest.raises(ValueError, match=f"^{phase}: worktree inputs changed"):
        _ = replay_selectable(
            Build(tree, tree / "tools", {}), (fault,), frozen, 1
        )
    return next(launches) - 1


def test_worker_validates_the_private_inputs_used_for_replay(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    for edit_at, phase, launched in (
        (1, "baseline phase", 1),
        (2, "fault phase", 3),
        (3, "fault phase", 3),
    ):
        tree = tmp_path / str(edit_at)
        assert (
            launches_until_refused(tree, monkeypatch, edit_at, phase)
            == launched
        )


def test_shared_executable_restores_in_a_fresh_process_and_credits_each_rule(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    faults = FrozenList(
        Fault(name, "audio", "source.rs", "old", "new", CASE, name)
        for name in ("first", "second")
    )
    requests: list[tuple[Path, str | None]] = []

    def replay(
        _root: Path,
        _case: str,
        executable: Path,
        environment: Mapping[str, str],
    ) -> Replay:
        selected = environment.get("GAMMA_ACTIVE")
        requests.append((executable, selected))
        if selected:
            return Replay(1, outcome("difference"), 0.1)
        return Replay(0, outcome("pass"), 0.1)

    monkeypatch.setattr(check_mutants, "run_case", replay)
    executable = tmp_path / "immutable"
    frozen = capture(tmp_path, scratch_files)
    assert replay_selectable(Build(tmp_path, executable, {}), faults, frozen, 1)
    assert requests == [
        (executable, ordinal) for ordinal in (None, "1", None, "2", None)
    ]
    log = capsys.readouterr().out
    assert rule_coverage(("first", "second"), faults, (log,)) == (
        "first: killed through first",
        "second: killed through second",
        "Unassigned faults: 0",
    )


def started(ordinal: str | None) -> str:
    seconds = float(ordinal or 0)
    return f"TIMING replay case={CASE} seconds={seconds:.6f}"


def test_parallel_faults_report_each_verdict_whole_in_manifest_order(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    results = {
        "1": Replay(1, outcome("difference")),
        "2": Replay(0, outcome("pass")),
        "3": Replay(1, outcome("unsupported")),
        "4": Replay(-11, ""),
    }
    faults = FrozenList(
        Fault(name, "audio", "source.rs", "old", "new", CASE)
        for name in ("kill", "survivor", "invalid", "crash")
    )
    last_fault_ran: Future[bool] = Future()
    waited: list[bool] = []

    def replay(
        _root: Path,
        _case: str,
        _executable: Path,
        environment: Mapping[str, str],
    ) -> Replay:
        selected = environment.get("GAMMA_ACTIVE")
        if selected == "1":
            waited.append(last_fault_ran.result(timeout=30))
        if selected == "4":
            _ = last_fault_ran.set_result(True)
        found = results[selected] if selected else Replay(0, outcome("pass"))
        return Replay(found.status, found.output, float(selected or 0))

    monkeypatch.setattr(check_mutants, "run_case", replay)
    frozen = capture(tmp_path, scratch_files)
    assert not replay_selectable(
        Build(tmp_path, tmp_path / "tools", {}), faults, frozen, 4
    )
    assert waited == [True]
    lines = FrozenList(
        line
        for line in capsys.readouterr().out.splitlines()
        if not line.startswith("TIMING input-check ")
    )
    restored = (started(None), f"RESTORED PASS {CASE}")
    passed = receipt(outcome("pass"), CASE)
    difference = receipt(outcome("difference"), CASE)
    assert passed is not None
    assert difference is not None
    digest = passed.digest
    crash = receipt_of({
        "id": CASE,
        "outcome": {"status": "crash", "exit": -11},
    }).digest
    assert lines == (
        started(None),
        f"BASELINE PASS {CASE}",
        started("1"),
        f"KILLED kill: {CASE} difference",
        "SELECTOR OFF fault=kill",
        *restored,
        f"RECEIPTS kill {digest} {difference.digest} {digest}",
        started("2"),
        f"SURVIVED survivor: {CASE} status=pass exit=0",
        "SELECTOR OFF fault=survivor",
        *restored,
        started("3"),
        f"INVALID invalid: {CASE} status=unsupported exit=1",
        "SELECTOR OFF fault=invalid",
        *restored,
        started("4"),
        f"KILLED crash: {CASE} crash exit=-11",
        "SELECTOR OFF fault=crash",
        *restored,
        f"RECEIPTS crash {digest} {crash} {digest}",
    )
