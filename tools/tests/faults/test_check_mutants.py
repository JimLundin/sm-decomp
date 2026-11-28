import json
import os
import re
from dataclasses import replace
from pathlib import Path, PurePosixPath
lazy from collections.abc import Iterator, Mapping, Sequence

import pytest
from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

from sm_tools.faults.core.check_mutants import (
    MANIFEST,
    PLAYED_RUNNER,
    Fault,
    Manifest,
    Replay,
    catalogue_rules,
    detected,
    manifest,
    receipt,
    receipt_of,
    rule_coverage,
    source_path,
    validate_coverage,
)
from sm_tools.faults.shell import check_mutants
from sm_tools.faults.shell.check_mutants import (
    CRASHES,
    Build,
    FrozenInputs,
    build_executable,
    capture,
    catalogue_specs,
    coverage_inputs,
    replay_selectable,
    scratch_files,
    sync_tree,
)
from sm_tools.shell import process
from tests.faults.outcomes import outcome

ROOT = Path(__file__).resolve().parents[3]


def roster() -> FrozenList[Fault]:
    return manifest((ROOT / MANIFEST).read_text(encoding="utf-8")).faults


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    _ = path.write_text(text, encoding="utf-8")


def test_rule_inventory_includes_entries_without_a_fault_or_case(
    tmp_path: Path,
) -> None:
    specs = tmp_path / "docs/behaviours"
    write(
        tmp_path / "docs/architecture/domains.toml",
        '[[model]]\nname = "world-blocks"\n',
    )
    for domain in ("world-intro-rooms", "world-shaft-entry"):
        write(specs / f"{domain}.md", "")
    write(specs / "uncatalogued.md", "### FutureRule\n")
    write(
        specs / "world-blocks.md",
        "### Closure\n### 2.1 A heading\n```\n### Example\n```\n### Untested\n",
    )
    write(specs / "world-blocks.decode.md", "### Notes\n")
    found = catalogue_rules(catalogue_specs(tmp_path))
    assert found == ("world-blocks/Closure", "world-blocks/Untested")
    with (specs / "world-blocks.md").open("a", encoding="utf-8") as spec:
        _ = spec.write("### Untested\n")
    with pytest.raises(ValueError, match="duplicate catalogue entry"):
        _ = catalogue_rules(catalogue_specs(tmp_path))


def kill_block(entry: Fault) -> FrozenList[str]:
    replay = f"TIMING replay case={entry.case} seconds=0.100000"
    return FrozenList((
        replay,
        f"KILLED {entry.name}: {entry.case} difference",
        f"TIMING restoration fault={entry.name} seconds=0.100000",
        "TIMING input-check seconds=0.100000",
        "TIMING compilation seconds=0.100000",
        replay,
        "TIMING input-check seconds=0.100000",
        f"RESTORED PASS {entry.case}",
    ))


def test_rule_credit_requires_its_assigned_fault_and_complete_receipt() -> None:
    fault = Fault(
        "contact",
        "played",
        PLAYED_RUNNER,
        "old",
        "new",
        "played steam 0 200",
        "PuffTouches",
    )
    placement = replace(fault, name="placement", rule="VentPlaced")
    unassigned = replace(fault, name="unassigned", rule=None)

    def report(logs: Sequence[str]) -> FrozenList[str]:
        return rule_coverage(
            ("PuffTouches", "PuffReach", "VentPlaced"),
            (fault, placement, unassigned),
            logs,
        )

    contact = kill_block(fault)
    other = kill_block(replace(placement, case="played other 0 200"))
    expected = (
        "PuffTouches: unknown",
        "PuffReach: unknown",
        "VentPlaced: unknown",
        "Unassigned faults: 1",
    )
    for label, lines in {
        "swapped restoration and resumed case": (
            *contact[:2],
            other[2],
            other[5],
            contact[-1],
        ),
        "wrong restoration fault": (*contact[:2], other[2], *contact[3:]),
        "wrong resumed case": (*contact[:5], other[5], *contact[6:]),
        "wrong kill name": kill_block(replace(fault, name="contacts")),
        "unassigned": kill_block(unassigned),
        "orphaned pass": contact[-1:],
        "missing restoration": (*contact[:2], *contact[3:]),
        "missing resumed replay": (*contact[:5], *contact[6:]),
        "duplicate kill": (*contact[:2], *contact[1:]),
        "duplicate restoration": (*contact[:3], *contact[2:]),
        "duplicate resumed replay": (*contact[:6], *contact[5:]),
        "duplicate pass": (*contact, *contact[-1:]),
    }.items():
        assert report(("\n".join(lines),)) == expected, label
    assert report(("\n".join(contact[:-1]), contact[-1])) == expected
    assert report(("\n".join((*contact, *kill_block(unassigned))),)) == (
        "PuffTouches: killed through contact",
        *expected[1:],
    )


def test_full_manifest_uses_the_selectable_runner(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    raw = (ROOT / MANIFEST).read_bytes()
    loaded = manifest(raw.decode())
    frozen = FrozenInputs(FrozenOrderedDict({ROOT / MANIFEST: raw}))
    calls: list[tuple[Path, FrozenInputs, Manifest]] = []

    def captured(root: Path) -> FrozenInputs:
        assert root == ROOT
        return frozen

    def selectable(root: Path, inputs: FrozenInputs, chosen: Manifest) -> int:
        calls.append((root, inputs, chosen))
        return 1

    def never_built(root: Path, *, target: Path) -> Path:
        raise AssertionError((root, target))

    monkeypatch.setattr(check_mutants, "capture", captured)
    monkeypatch.setattr(check_mutants, "build_executable", never_built)
    monkeypatch.setattr(check_mutants, "run_selectable", selectable)
    for arguments in ((), ("selectable",)):
        calls.clear()
        assert check_mutants.run(arguments, ROOT) == 1
        assert calls == [(ROOT, frozen, loaded)], arguments


def artifact(executable: Path) -> Mapping[str, object]:
    return {
        "reason": "compiler-artifact",
        "target": {"name": "tools", "kind": ["bin"]},
        "executable": str(executable),
    }


def finishing(
    monkeypatch: pytest.MonkeyPatch, status: int, rows: Sequence[object]
) -> None:
    stdout = "\n".join(json.dumps(row) for row in rows).encode()

    def finished(_command: Sequence[str], *, cwd: Path) -> process.Finished:
        assert cwd.is_dir()
        return process.Finished(status, stdout, b"")

    monkeypatch.setattr(process, "run", finished)


def test_build_failure_cannot_execute_a_stale_artifact(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    stale = tmp_path / "tools"
    _ = stale.write_bytes(b"stale executable")
    finishing(monkeypatch, 101, (artifact(stale),))
    with pytest.raises(process.CommandError):
        _ = build_executable(tmp_path, target=tmp_path / "build")


def test_cargo_reported_artifact_is_required_and_respects_its_location(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    executable = tmp_path / "custom-target" / "tools"
    executable.parent.mkdir()
    _ = executable.write_bytes(b"executable")
    built = artifact(executable)
    for rows in (
        (),
        (built, built),
        ({**built, "executable": str(tmp_path / "missing")},),
    ):
        finishing(monkeypatch, 0, rows)
        with pytest.raises(ValueError, match="executable"):
            _ = build_executable(tmp_path, target=tmp_path / "build")
    finishing(monkeypatch, 0, (built,))
    found = build_executable(tmp_path, target=tmp_path / "build")
    assert found == executable.resolve()


def test_frozen_inputs_preserve_dirty_missing_and_untracked_file_states(
    tmp_path: Path,
) -> None:
    root = tmp_path / "repository"
    _ = process.checked(("git", "init", "-q", str(root)), cwd=tmp_path)
    tracked = root / "tracked.rs"
    absent = root / "absent.rs"
    _ = tracked.write_bytes(b"initial\n")
    _ = absent.write_bytes(b"initial\n")
    _ = process.checked(("git", "add", "tracked.rs", "absent.rs"), cwd=root)
    _ = tracked.write_bytes(b"dirty\r\n")
    absent.unlink()
    untracked = root / "config.toml"
    _ = untracked.write_bytes(b"input\n")
    inputs = capture(root)
    _ = inputs.check(root)
    for path, changed in (
        (tracked, b"initial\n"),
        (absent, b"appeared\n"),
        (untracked, b"changed\n"),
        (root / "new.rs", b"new\n"),
    ):
        original = path.read_bytes() if path.exists() else None
        _ = path.write_bytes(changed)
        with pytest.raises(ValueError, match="worktree inputs changed"):
            _ = inputs.check(root)
        if original is None:
            path.unlink()
        else:
            _ = path.write_bytes(original)
    untracked.unlink()
    with pytest.raises(ValueError, match="worktree inputs changed"):
        _ = inputs.check(root)


def test_synced_tree_rewrites_only_changed_files_and_drops_the_rest(
    tmp_path: Path,
) -> None:
    root, tree = tmp_path / "root", tmp_path / "tree"
    kept, patched = (
        PurePosixPath("crates/kept/lib.rs"),
        PurePosixPath("patched.rs"),
    )
    gone = PurePosixPath("crates/gone/lib.rs")
    write(root / kept, "kept\n")
    write(root / patched, "original\n")
    write(root / gone, "gone\n")
    sync_tree(
        root, tree, {kept: b"kept\n", patched: b"fault 1\n", gone: b"gone\n"}
    )
    built = 1_000_000_000
    for path in scratch_files(tree):
        os.utime(path, ns=(built, built))
    sync_tree(root, tree, {kept: b"kept\n", patched: b"fault 2\n"})
    assert scratch_files(tree) == FrozenOrderedSet((
        tree / kept,
        tree / patched,
    ))
    assert not (tree / "crates/gone").exists()
    assert (tree / kept).stat().st_mtime_ns == built
    assert (tree / patched).stat().st_mtime_ns > built
    assert (tree / patched).read_bytes() == b"fault 2\n"


def exercise(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
    result: Replay,
    restored: Replay | None = None,
) -> bool:
    fault = Fault(
        "zero", "audio", "source.rs", "before", "after", "audio/example"
    )
    healthy = Replay(0, outcome("pass"))
    replays: Iterator[Replay] = iter((healthy, result, restored or healthy))

    def replay(
        _root: Path, _case: str, _executable: Path, _env: Mapping[str, str]
    ) -> Replay:
        return next(replays)

    monkeypatch.setattr(check_mutants, "run_case", replay)
    frozen = capture(tmp_path, scratch_files)
    return replay_selectable(
        Build(tmp_path, tmp_path / "tools", {}), (fault,), frozen, 1
    )


def test_only_production_adapters_are_allowed() -> None:
    for name in (
        "crates/game/src/production.rs",
        "crates/game/src/production/movement.rs",
        PLAYED_RUNNER,
        "crates/verify/src/replay/played/platform.rs",
    ):
        assert source_path(name) == PurePosixPath(name)
    for name in (
        "crates/clean/types/src/lib.rs",
        "crates/game/src/tick.rs",
        "crates/verify/src/replay/hud.rs",
        "crates/game/src/production_other/audio.rs",
        "crates/game/src/production/../production.rs",
    ):
        with pytest.raises(ValueError, match="fault source"):
            _ = source_path(name)


def test_any_reported_difference_data_failure_or_crash_kills(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    for result in (
        Replay(1, "tools: " + outcome("difference")),
        Replay(1, outcome("data-failure")),
        Replay(1, outcome("checkpoint-failure")),
        Replay(101, outcome("pass")),
        Replay(-11, ""),
        Replay(-6, ""),
    ):
        assert exercise(monkeypatch, tmp_path, result), result


def test_a_pass_an_unsupported_case_or_another_case_cannot_kill(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    for result in (
        Replay(0, outcome("pass")),
        Replay(1, outcome("unsupported")),
        Replay(1, ""),
        Replay(1, outcome("difference", case="another/case")),
        Replay(-9, ""),
        Replay(-9, outcome("difference")),
    ):
        assert not exercise(monkeypatch, tmp_path, result), result


def test_restored_baseline_must_pass(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    difference = Replay(1, outcome("difference"))
    assert not exercise(monkeypatch, tmp_path, difference, difference)


def test_restored_pass_must_keep_receipt_identity(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    for before, after in (
        ("cuts = 3", "cuts = 4"),
        ("restorations = 2", "restorations = 1"),
        ('observation_sha256 = "original"', 'observation_sha256 = "changed"'),
    ):
        restored = Replay(0, outcome("pass").replace(before, after))
        difference = Replay(1, outcome("difference"))
        assert not exercise(monkeypatch, tmp_path, difference, restored), after


def test_supported_responsibilities_cannot_lose_faults() -> None:
    faults = roster()
    known = coverage_inputs(ROOT)
    validate_coverage(known, faults)
    for owner in FrozenOrderedSet(fault.responsibility for fault in faults):
        kept = FrozenList(
            fault for fault in faults if fault.responsibility != owner
        )
        with pytest.raises(ValueError, match="production fault"):
            validate_coverage(known, kept)
    for changed in (
        replace(faults[0], responsibility="contact"),
        replace(faults[0], case="blocks/missing-requests"),
        replace(faults[0], rule="samus-movement/DoesNotExist"),
        replace(faults[0], rule="game-random/AdvanceRandom"),
    ):
        with pytest.raises(ValueError, match="fault"):
            validate_coverage(known, (changed, *faults[1:]))


def test_every_case_rule_is_a_catalogue_entry() -> None:
    faults = roster()
    known = coverage_inputs(ROOT)
    declared = FrozenOrderedSet(
        rule for case in known.cases.values() for rule in case.rules
    )
    missing = min(declared - FrozenOrderedSet(fault.rule for fault in faults))
    published = FrozenList(rule for rule in known.rules if rule != missing)
    with pytest.raises(
        ValueError, match=rf"names unknown catalogue rule {re.escape(missing)}$"
    ):
        validate_coverage(replace(known, rules=published), faults)


def test_every_anchor_names_one_place_in_its_source() -> None:
    for fault in roster():
        source = (ROOT / source_path(fault.file)).read_bytes()
        assert source.count(fault.before.encode()) == 1, fault.name


def fabricated_repository(root: Path) -> None:
    write(
        root / MANIFEST,
        "\n".join((
            "[[fault]]",
            'name = "zero"',
            'responsibility = "audio"',
            'file = "crates/game/src/production.rs"',
            'before = "measured"',
            'after = "zero"',
            'case = "audio/example"',
            "[[fault]]",
            'name = "stale"',
            'responsibility = "played"',
            f'file = "{PLAYED_RUNNER}"',
            'before = "pad"',
            'after = "stale"',
            'case = "played row-1 0 9"',
            "",
        )),
    )
    write(root / PLAYED_RUNNER, "let held = pad;\n")
    write(
        root / "scenarios/replay/index.toml",
        '[[case]]\nid = "audio/example"\nresponsibilities = ["audio"]\nrules = []\n',
    )
    write(
        root / "scenarios/replay/baseline.toml",
        '[[case]]\nid = "audio/example"\nexpected = { status = "pass" }\n',
    )
    write(root / "docs/architecture/domains.toml", "model = []\n")
    for domain in ("world-intro-rooms", "world-shaft-entry"):
        write(root / "docs/behaviours" / f"{domain}.md", "")


def test_manifest_validation_checks_anchors_without_building(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    def never_built(root: Path, *, target: Path) -> Path:
        raise AssertionError((root, target))

    monkeypatch.setattr(check_mutants, "build_executable", never_built)
    fabricated_repository(tmp_path)
    source = tmp_path / "crates/game/src/production.rs"
    for text, unique in (
        ("let elapsed = measured;\n", True),
        ("let elapsed = sampled;\n", False),
        ("let a = measured;\nlet b = measured;\n", False),
    ):
        write(source, text)
        if unique:
            assert check_mutants.run(("validate",), tmp_path) == 0
        else:
            with pytest.raises(ValueError, match="one unique changed fragment"):
                _ = check_mutants.run(("validate",), tmp_path)


def test_a_source_that_resolves_outside_its_owners_is_refused(
    tmp_path: Path,
) -> None:
    fabricated_repository(tmp_path)
    outside = tmp_path / "crates/other/src/lib.rs"
    write(outside, "let elapsed = measured;\n")
    adapter = tmp_path / "crates/game/src/production.rs"
    adapter.parent.mkdir(parents=True)
    adapter.symlink_to(outside)
    with pytest.raises(ValueError, match="outside production adapters"):
        _ = check_mutants.run(("validate",), tmp_path)
    adapter.unlink()
    write(adapter, "let elapsed = measured;\n")
    owned = tmp_path / "crates/game/src/production/owned.rs"
    write(
        tmp_path / MANIFEST,
        (tmp_path / MANIFEST).read_text()
        + '[runtime_callers]\n"crates/game/src/production/owned.rs" = ["f"]\n',
    )
    owned.parent.mkdir(parents=True, exist_ok=True)
    owned.symlink_to(outside)
    with pytest.raises(ValueError, match="outside production adapters"):
        _ = check_mutants.run(("validate",), tmp_path)


def test_a_baseline_case_without_an_expected_outcome_stops_validation(
    tmp_path: Path,
) -> None:
    fabricated_repository(tmp_path)
    write(
        tmp_path / "crates/game/src/production.rs", "let elapsed = measured;\n"
    )
    write(
        tmp_path / "scenarios/replay/baseline.toml",
        (tmp_path / "scenarios/replay/baseline.toml").read_text()
        + '[[case]]\nid = "audio/other"\n',
    )
    with pytest.raises(
        ValueError, match="'audio/other' has no expected outcome"
    ):
        _ = check_mutants.run(("validate",), tmp_path)


def played_report(stop: str, row: str = "row-1") -> str:
    return (
        f"PLAYED_ALONE row={row} start=0 held=4 stop={stop}"
        ' seed_held=["energy"]\n'
    )


def test_a_played_run_kills_on_any_difference_or_data_failure() -> None:
    case = "played row-1 0 9"
    stopped = played_report('At(Difference { cut: 5, field: "camera.x" })')
    ended = receipt(played_report("End"), case)
    assert ended is not None
    assert ended.passed
    assert receipt(stopped, case) == receipt_of({
        "id": case,
        "report": stopped.strip(),
        "outcome": {"status": "difference"},
    })
    for output in (
        played_report("End", row="row-2"),
        played_report("End") * 2,
        "",
    ):
        assert receipt(output, case) is None
    fault = Fault(
        "stale", "played", PLAYED_RUNNER, "controller.held", "stale.held", case
    )
    for result, killed in (
        (Replay(0, stopped), True),
        (
            Replay(
                1,
                'tools: replay played: DataFailure { cut: None, source: "pad" }',
            ),
            True,
        ),
        (
            Replay(
                0,
                played_report(
                    'At(DataFailure { cut: Some(5), source: "pad" })'
                ),
            ),
            True,
        ),
        (Replay(0, played_report("At(CheckpointFailure { cut: 5 })")), True),
        (
            Replay(
                0,
                played_report(
                    'At(Unsupported { cut: Some(5), dependency: "camera.x" })'
                ),
            ),
            False,
        ),
        (Replay(0, played_report("End")), False),
    ):
        found = detected(fault, result, CRASHES).value is not None
        assert found == killed, result


def test_a_played_startup_result_other_than_a_data_failure_is_invalid() -> None:
    fault = Fault(
        "stale",
        "played",
        PLAYED_RUNNER,
        "controller.held",
        "stale.held",
        "played row-1 0 9",
    )
    startup = (
        'tools: replay played: Unsupported { cut: None, dependency: "pad" }'
    )
    found = detected(fault, Replay(1, startup), CRASHES)
    assert found.value is None
    assert found.lines[0].startswith("INVALID stale: "), found.lines
