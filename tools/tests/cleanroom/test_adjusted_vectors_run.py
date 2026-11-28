import json
import sys
from pathlib import Path
from typing import cast
lazy from collections.abc import Callable, Mapping, Sequence

import pytest
from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

from sm_tools.cleanroom.core.adjusted_vectors_run import (
    CannotProbeError,
    Owners,
    Proven,
    Unadjusted,
    Unproven,
    Verdict,
    artifacts,
    candidates,
    completed_test,
    is_list,
    is_table,
    parameter_entries,
    parsed,
    probe,
    probe_test,
    rule_owners,
    vectors,
)
from sm_tools.cleanroom.shell import adjusted_vectors_run as shell
from sm_tools.cleanroom.shell.adjusted_vectors_run import (
    check_domain_owners,
    check_domains,
    file_runs,
    passes,
    prove,
    run,
)
from sm_tools.cleanroom.shell.adjusted_vectors_run import (
    test_target as target_of,
)

FIXTURES = (
    Path(__file__).resolve().parents[2]
    / "cleanroom"
    / "adjusted_vectors_fixtures"
)
NONE: FrozenList[str] = FrozenList()
STEP = FrozenOrderedSet(("Step",))
MARGIN = FrozenOrderedSet(("Margin",))
STEP_OWNERS: Owners = FrozenOrderedDict({"step": STEP})

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

GOOD = b"running 1 test\ntest direct ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s\n"
PASSED = b"running 2 tests\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
FAILED = b"running 2 tests\ntest result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"


def vector(body: str) -> str:
    return REFERENCE + "\n[[vector]]\n" + body


def integer(row: Mapping[str, object], key: str, default: int = 0) -> int:
    value = row.get(key, default)
    assert isinstance(value, int)
    return value


def rules(row: Mapping[str, object]) -> Mapping[str, object]:
    value = row.get("rules", {})
    return value if is_table(value) else {}


def ids(text: str) -> FrozenList[str]:
    return FrozenList(str(row["id"]) for row in vectors(text))


def model(step_rule: str = "step", id_ends: str = "") -> Callable[[str], bool]:
    def answer(text: str) -> bool:
        found = ids(text)
        return (
            len(FrozenOrderedSet(found)) == len(found)
            and all(i.endswith(id_ends) for i in found)
            and all(
                integer(row, "after")
                == integer(row, "before") + integer(rules(row), step_rule, 1)
                for row in vectors(text)
            )
        )

    return answer


FAKE_TEST = """import json, os, sys, tomllib
with open(os.environ.get("SM_VECTOR_FILE", {committed!r}), "rb") as vectors:
    TEXT = vectors.read()
rows = tomllib.loads(TEXT.decode())["vector"]
with open({receipt!r}, "a") as receipt:
    receipt.write(json.dumps([sys.argv[1:], TEXT.decode().rstrip()]) + "\\n")
if "rules" not in rows[-1] and {step_rule!r} == "crash":
    sys.exit(126)
ok = all(row["after"] == row["before"] + row.get("rules", {{}}).get("step", 1) for row in rows)
name = sys.argv[1] if len(sys.argv) > 1 else "direct"
status = "ok" if ok else "FAILED"
print("running 1 test")
print(f"test {{name}} ... {{status}}")
print(f"test result: {{status}}. {{int(ok)}} passed; {{int(not ok)}} failed; 0 ignored; "
      "0 measured; 0 filtered out; finished in 0.00s")
sys.exit(0 if ok else 101)
"""


def fake_test(
    directory: Path, committed: Path, receipt: Path, step_rule: str = "step"
) -> Path:
    executable = directory / "vectors-fake"
    source = FAKE_TEST.format(
        committed=str(committed), receipt=str(receipt), step_rule=step_rule
    )
    _ = executable.write_text(f"#!{sys.executable}\n{source}")
    executable.chmod(0o755)
    return executable


def receipts(receipt: Path) -> FrozenList[tuple[FrozenList[str], str]]:
    found: list[tuple[FrozenList[str], str]] = []
    for line in receipt.read_text(encoding="utf-8").splitlines():
        decoded = cast("object", json.loads(line))
        assert is_list(decoded)
        arguments, text = decoded
        assert is_list(arguments)
        assert isinstance(text, str)
        found.append((FrozenList(str(a) for a in arguments), text))
    return FrozenList(found)


def proof_of(
    text: str, step_rule: str = "step", id_ends: str = ""
) -> Mapping[str, Verdict]:
    return prove(
        text,
        parameter_entries(SPEC),
        {"step": STEP, "margin": MARGIN},
        model(step_rule, id_ends),
    )


def demo(root: Path, text: str, selection: str | None) -> Path:
    spec = root / "docs/behaviours/demo.md"
    spec.parent.mkdir(parents=True)
    _ = spec.write_text("### Step\n- switch: Parameter.\n", newline="\n")
    metadata = root / "tools/cleanroom/adjusted_rule_owners/demo.toml"
    metadata.parent.mkdir(parents=True)
    _ = metadata.write_text('[owners]\nstep = ["Step"]\n', newline="\n")
    if selection is not None:
        _ = (metadata.parent.parent / "adjusted_probe_tests.toml").write_text(
            f'[tests]\ndemo = "{selection}"\n', newline="\n"
        )
    (root / "crates/clean/demo").mkdir(parents=True)
    path = root / "scenarios/vectors/demo.toml"
    path.parent.mkdir(parents=True)
    _ = path.write_bytes(text.encode())
    return path


def test_the_spec_names_its_parameter_entries() -> None:
    assert parameter_entries(SPEC) == ("Step", "Margin")


def test_a_vector_that_fails_at_the_reference_rules_proves_its_entry() -> None:
    inline = vector(
        'id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\nbefore = 2\nafter = 5\n'
    )
    expected = {"Step": Proven("adjusted", NONE), "Margin": Unadjusted()}
    assert proof_of(inline) == expected
    table = vector(
        'id = "adjusted"\nentries = [\n  "Step",\n]\nbefore = 2\nafter = 5\n\n[vector.rules]\nstep = 3\n'
    )
    assert proof_of(table) == expected


def test_empty_rules_adjust_nothing() -> None:
    empty = vector(
        'id = "empty"\nentries = ["Step"]\nrules = {}\nbefore = 2\nafter = 3\n'
    )
    assert proof_of(empty) == {"Step": Unadjusted(), "Margin": Unadjusted()}


def test_the_reference_value_proves_nothing() -> None:
    same = vector(
        'id = "same"\nentries = ["Step"]\nrules = { step = 1 }\nbefore = 2\nafter = 3\n'
    )
    assert proof_of(same) == {"Step": Unproven(), "Margin": Unadjusted()}


def test_a_rule_the_case_does_not_reach_proves_nothing() -> None:
    other = vector(
        'id = "other"\nentries = ["Step"]\nrules = { margin = 9 }\nbefore = 2\nafter = 3\n'
    )
    assert proof_of(other) == {"Step": Unproven(), "Margin": Unadjusted()}


def test_a_later_vector_proves_what_an_earlier_one_does_not() -> None:
    both = vector(
        'id = "same"\nentries = ["Step"]\nrules = { step = 1 }\nbefore = 2\nafter = 3\n\n[[vector]]\nid = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\nbefore = 2\nafter = 5\n'
    )
    assert proof_of(both) == {
        "Step": Proven("adjusted", NONE),
        "Margin": Unadjusted(),
    }


def test_a_vector_of_its_own_is_preferred_and_a_shared_one_says_so() -> None:
    shared = vector(
        'id = "shared"\nentries = ["Step", "Margin"]\nrules = { step = 3 }\nbefore = 2\nafter = 5\n\n[[vector]]\nid = "own"\nentries = ["Fixed", "Step"]\nrules = { step = 4 }\nbefore = 2\nafter = 6\n'
    )
    assert proof_of(shared) == {
        "Step": Proven("own", NONE),
        "Margin": Unproven(),
    }


def test_a_model_that_ignores_the_rule_fails_before_any_probe() -> None:
    adjusted = vector(
        'id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\nbefore = 2\nafter = 5\n'
    )
    with pytest.raises(CannotProbeError):
        _ = proof_of(adjusted, step_rule="ignored")


def test_a_test_that_refuses_the_copy_itself_proves_nothing() -> None:
    strict = vector(
        'id = "x-d"\nentries = ["Step"]\nrules = { step = 3 }\nbefore = 2\nafter = 5\n'
    ).replace('"reference"', '"x-d"', 1)
    strict = strict.replace(
        'id = "x-d"\nentries = ["Step"]', 'id = "y-d"\nentries = ["Step"]'
    )
    with pytest.raises(CannotProbeError):
        _ = proof_of(strict, id_ends="-d")


def test_a_fixed_count_adapter_accepts_control_and_refutes_reference() -> None:
    text = vector(
        'id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\nbefore = 2\nafter = 5\n'
    )
    run_model = model()
    observed: list[tuple[FrozenList[str], bool]] = []

    def fixed_count_adapter(probed: str) -> bool:
        passed = len(vectors(probed)) == 2 and run_model(probed)
        observed.append((ids(probed), passed))
        return passed

    assert prove(text, ("Step",), STEP_OWNERS, fixed_count_adapter) == {
        "Step": Proven("adjusted", NONE)
    }
    assert observed == [
        (("reference", "adjusted"), True),
        (("reference", "adjusted-probe"), False),
        (("reference", "adjusted-probe"), True),
    ]


def test_dependent_failure_cannot_credit_an_ignored_direct_parameter() -> None:
    for step, after, expected in (
        (3, 3, Unproven()),
        (3, 5, Proven("adjusted", NONE)),
    ):
        text = vector(
            f'id = "adjusted"\nentries = ["Step"]\nrules = {{ step = {step} }}\nbefore = 2\nafter = {after}\n'
        )
        text += (
            f'[[dependent]]\nbase = "adjusted"\nrules = {{ step = {step} }}\n'
        )
        dependent = parsed(text, "the vector file").get("dependent")
        reads = "ignored" if after == 3 else "step"

        def direct(
            probed: str, dependent: object = dependent, reads: str = reads
        ) -> bool:
            assert parsed(probed, "the probe").get("dependent") == dependent
            return model(reads)(probed)

        assert prove(text, ("Step",), STEP_OWNERS, direct) == {"Step": expected}


def failing_at(call: int) -> Callable[[str], bool]:
    calls: list[str] = []

    def direct(probed: str) -> bool:
        calls.append(probed)
        if len(calls) == call:
            message = "runner failure"
            raise CannotProbeError(message)
        return model()(probed)

    return direct


def never_passes(_probed: str) -> bool:
    return False


def test_a_failed_baseline_or_a_runner_error_refuses() -> None:
    text = vector(
        'id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\nbefore = 2\nafter = 5\n'
    )
    for failure_call in (1, 2, 3):
        with pytest.raises(CannotProbeError):
            _ = prove(text, ("Step",), STEP_OWNERS, failing_at(failure_call))
    with pytest.raises(CannotProbeError):
        _ = prove(text, ("Step",), STEP_OWNERS, never_passes)


def test_exact_selection_requires_one_named_nonignored_completed_test() -> None:
    assert completed_test(GOOD, 0, 101, "direct")
    failed = (
        GOOD
        .replace(b"... ok", b"... FAILED")
        .replace(b"result: ok", b"result: FAILED")
        .replace(b"1 passed", b"0 passed")
        .replace(b"0 failed", b"1 failed")
    )
    assert not completed_test(failed, 101, 101, "direct")
    for output in (
        GOOD.replace(b"direct", b"wrong"),
        GOOD
        .replace(b"direct ... ok", b"direct ... ignored")
        .replace(b"1 passed", b"0 passed")
        .replace(b"0 ignored", b"1 ignored"),
        GOOD.replace(b"running 1", b"running 0").replace(
            b"1 passed", b"0 passed"
        ),
        GOOD.replace(b"test direct ... ok\n", b""),
    ):
        with pytest.raises(CannotProbeError):
            _ = completed_test(output, 0, 101, "direct")


def test_one_build_answers_every_probe_of_its_vectors(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    text = (
        vector(
            'id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\nbefore = 2\nafter = 5\n'
        )
        + '[[dependent]]\nbase = "adjusted"\n'
    )
    path = demo(tmp_path, text, "direct")
    receipt = tmp_path / "receipt"
    builds: list[bytes] = []

    def build(_root: Path, crates: Sequence[str]) -> Mapping[str, Path]:
        assert crates == ("demo",)
        builds.append(path.read_bytes())
        built = tmp_path / f"build-{len(builds)}"
        built.mkdir()
        return {"demo": fake_test(built, path, receipt)}

    monkeypatch.setattr(shell, "build", build)
    assert check_domains(tmp_path, ("demo",)) == 0
    assert (
        "adjusted: demo: Step depends on its rules in adjusted"
        in capsys.readouterr().out
    )
    assert path.read_bytes() == text.encode()
    assert builds == [text.encode()]
    runs = receipts(receipt)
    assert [arguments for arguments, _ in runs] == [
        (),
        ("direct", "--exact"),
        ("direct", "--exact"),
        ("direct", "--exact"),
    ]
    assert [ids(t)[1] for _, t in runs] == [
        "adjusted",
        "adjusted",
        "adjusted-probe",
        "adjusted-probe",
    ]
    assert ["rules" in vectors(t)[1] for _, t in runs] == [
        True,
        True,
        False,
        True,
    ]


def test_a_probe_reaches_the_test_through_its_own_file(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    committed = vector('id = "a"\nbefore = 2\nafter = 3\n')
    probed = vector('id = "b"\nbefore = 2\nafter = 4\n')
    path = tmp_path / "demo.toml"
    _ = path.write_text(committed, encoding="utf-8")
    receipt = tmp_path / "receipt"
    executable = fake_test(tmp_path, path, receipt)
    scratch = tmp_path / "scratch"
    scratch.mkdir()
    monkeypatch.setenv("SM_VECTOR_FILE", str(tmp_path / "stale.toml"))
    assert passes(executable, tmp_path, None)
    assert not file_runs(executable, tmp_path, None, scratch)(probed)
    assert [t for _, t in receipts(receipt)] == [
        committed.rstrip(),
        probed.rstrip(),
    ]
    assert path.read_text(encoding="utf-8") == committed


def test_cargo_messages_name_each_executable_and_its_package() -> None:
    output = b"\n".join([
        b'{"reason":"compiler-artifact","manifest_path":"/r/crates/clean/demo/Cargo.toml","executable":"/r/target/debug/deps/vectors-1"}',
        b'{"reason":"compiler-artifact","manifest_path":"/r/crates/clean/types/Cargo.toml","executable":null}',
        b'{"reason":"build-finished","success":true}',
    ])
    assert [
        (a.manifest.parent.name, a.executable.name) for a in artifacts(output)
    ] == [("demo", "vectors-1")]


def test_linked_files_require_valid_reviewed_test_selection() -> None:
    text = REFERENCE + '[[dependent]]\nbase = "reference"\n'
    assert probe_test(None, "demo", REFERENCE) is None
    with pytest.raises(CannotProbeError):
        _ = probe_test(None, "demo", text)
    for metadata in (
        b'[tests]\ndemo = ""\n',
        b"[tests]\ndemo = 3\n",
        b'[tests]\ndemo = "direct --ignored"\n',
    ):
        with pytest.raises(CannotProbeError):
            _ = probe_test(metadata, "demo", text)
    selected = probe_test(b'[tests]\ndemo = "vectors::direct"\n', "demo", text)
    assert selected == "vectors::direct"


def expected_file(
    text: str, *, without: Sequence[str] | None
) -> dict[str, object]:
    rows = [dict(row) for row in vectors(text)]
    rows[1]["id"] = "adjusted-probe"
    if without is not None:
        left = dict(rules(rows[1]))
        for rule in without or tuple(left):
            del left[rule]
        if left:
            rows[1]["rules"] = left
        else:
            del rows[1]["rules"]
    return {**parsed(text, "the vector file"), "vector": rows}


def test_replacement_preserves_all_other_parsed_content() -> None:
    for rule_lines in ("rules = { step = 3 }\n", "[vector.rules]\nstep = 3\n"):
        text = (
            'title = "globals"\n[settings]\ncount = 7\n[[annotation]]\nid = "before"\n'
            + REFERENCE
            + '\n[[vector]]\nid = "adjusted"\nentries = ["Step"]\nbefore = 2\nafter = 5\n'
            + rule_lines
            + '[vector.note]\nid = "nested-id"\ntext = "keep"\n[[annotation]]\nid = "between"\n[[vector]]\nid = "later"\nentries = ["Fixed"]\nbefore = 5\nafter = 6\n[[annotation]]\nid = "after"\n'
        )
        candidate = candidates(text)[0]
        for keep_rules in (False, True):
            probed = probe(text, candidate, keep_rules=keep_rules)
            expected = expected_file(text, without=None if keep_rules else ())
            assert parsed(probed, "the probe") == expected


def test_a_probe_id_collision_is_refused() -> None:
    text = vector(
        'id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\nbefore = 2\nafter = 5\n'
    ).replace('"reference"', '"adjusted-probe"')
    with pytest.raises(CannotProbeError):
        _ = proof_of(text)


def test_two_overrides_do_not_credit_an_ignored_parameter() -> None:
    text = vector(
        'id = "shared"\nentries = ["Step", "Margin"]\nrules = { step = 3, margin = 9 }\nbefore = 2\nafter = 5\n'
    )
    assert proof_of(text) == {
        "Step": Proven("shared", FrozenList(("Margin",))),
        "Margin": Unproven(),
    }


def test_shared_vector_proves_each_executed_parameter_separately() -> None:
    text = vector(
        'id = "shared"\nentries = ["Step", "Margin"]\nrules = { step = 3, margin = 9 }\nbefore = 2\nafter = 14\n'
    )
    probes: list[Mapping[str, object]] = []

    def answer(probed: str) -> bool:
        rows = vectors(probed)
        probes.append(rules(rows[1]))
        return len(rows) == 2 and all(
            integer(row, "after")
            == integer(row, "before")
            + integer(rules(row), "step", 1)
            + integer(rules(row), "margin", 0)
            for row in rows
        )

    owners: Owners = {"step": STEP, "margin": MARGIN}
    assert prove(text, ("Step", "Margin"), owners, answer) == {
        "Step": Proven("shared", FrozenList(("Margin",))),
        "Margin": Proven("shared", FrozenList(("Step",))),
    }
    assert {"margin": 9} in probes
    assert {"step": 3} in probes


def test_missing_unknown_or_ambiguous_ownership_refuses() -> None:
    text = vector(
        'id = "shared"\nentries = ["Step", "Margin"]\nrules = { step = 3, margin = 9 }\nbefore = 2\nafter = 5\n'
    )
    unknown = FrozenOrderedSet(("Unknown",))
    both = FrozenOrderedSet(("Step", "Margin"))
    owners: FrozenList[Owners] = FrozenList((
        {},
        {"step": STEP, "margin": unknown},
        {"step": STEP, "margin": both},
    ))
    for owned in owners:
        with pytest.raises(CannotProbeError):
            _ = prove(text, ("Step", "Margin"), owned, model())


def test_the_owners_check_refuses_an_unowned_override_and_runs_no_test(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    spec = tmp_path / "docs/behaviours/demo.md"
    spec.parent.mkdir(parents=True)
    _ = spec.write_text(SPEC, newline="\n")
    path = tmp_path / "scenarios/vectors/demo.toml"
    path.parent.mkdir(parents=True)
    _ = path.write_text(
        vector(
            'id = "shared"\nentries = ["Step", "Margin"]\nrules = { step = 3, margin = 9 }\nbefore = 2\nafter = 5\n'
        ),
        newline="\n",
    )
    metadata = tmp_path / "tools/cleanroom/adjusted_rule_owners/demo.toml"
    metadata.parent.mkdir(parents=True)
    ran: list[Sequence[object]] = []

    def no_process(*args: object) -> None:
        ran.append(args)

    monkeypatch.setattr("subprocess.run", no_process)
    for owners, refused in (
        ('step = ["Step"]\n', True),
        ('step = ["Step"]\nmargin = ["Margin"]\n', False),
    ):
        _ = metadata.write_text("[owners]\n" + owners, newline="\n")
        if refused:
            with pytest.raises(CannotProbeError):
                check_domain_owners(tmp_path, "demo")
        else:
            check_domain_owners(tmp_path, "demo")
    assert ran == []


def test_a_shared_rule_owner_is_proven_only_by_separate_entry_witnesses() -> (
    None
):
    text = vector(
        'id = "step"\nentries = ["Step"]\nrules = { step = 3 }\nbefore = 2\nafter = 5\n\n[[vector]]\nid = "margin"\nentries = ["Margin"]\nrules = { step = 4 }\nbefore = 2\nafter = 6\n'
    )
    owners: Owners = {"step": FrozenOrderedSet(("Step", "Margin"))}
    assert prove(text, ("Step", "Margin"), owners, model()) == {
        "Step": Proven("step", NONE),
        "Margin": Proven("margin", NONE),
    }


def test_runner_error_after_a_probe_propagates() -> None:
    text = vector(
        'id = "adjusted"\nentries = ["Step"]\nrules = { step = 3 }\nbefore = 2\nafter = 5\n'
    )
    calls: list[str] = []

    def answer(probed: str) -> bool:
        calls.append(probed)
        if len(calls) == 2:
            raise ConnectionResetError
        return True

    with pytest.raises(ConnectionResetError):
        _ = prove(text, ("Step",), STEP_OWNERS, answer)


def test_selective_replacement_preserves_nested_unrelated_overrides() -> None:
    for rule_lines in (
        "rules = { step = 3, margin = { values = [9, 8] } }\n",
        "[vector.rules]\nstep = 3\n[vector.rules.margin]\nvalues = [9, 8]\n",
    ):
        text = vector(
            'id = "adjusted"\nentries = ["Step", "Margin"]\nbefore = 2\nafter = 5\n'
            + rule_lines
        )
        probed = probe(
            text,
            candidates(text)[0],
            keep_rules=False,
            remove_rules=FrozenOrderedSet(("step",)),
        )
        assert parsed(probed, "the probe") == expected_file(
            text, without=("step",)
        )


def test_ownership_metadata_is_required_only_when_adjusted_candidates_exist() -> (
    None
):
    assert rule_owners(None, "demo", required=False) == {}
    with pytest.raises(CannotProbeError):
        _ = rule_owners(None, "demo", required=True)
    for text in ('[owners]\nstep = "Step"\n', "[owners]\nstep = [3]\n"):
        with pytest.raises(CannotProbeError):
            _ = rule_owners(text, "demo", required=True)
    found = rule_owners('[owners]\nstep = ["Step"]\n', "demo", required=True)
    assert found == {"step": STEP}


def test_completed_nonempty_test_results_are_the_only_model_answers() -> None:
    for failure_code in (1, 101):
        assert completed_test(PASSED, 0, failure_code)
        assert not completed_test(FAILED, failure_code, failure_code)
        for code, output in (
            (126, FAILED),
            (2, FAILED),
            (-9, FAILED),
            (1, PASSED),
            (0, FAILED),
            (0, b""),
            (0, PASSED + PASSED),
            (0, PASSED.replace(b"running 2", b"running 3")),
            (
                0,
                PASSED.replace(b"running 2", b"running 0").replace(
                    b"2 passed", b"0 passed"
                ),
            ),
            (
                0,
                PASSED.replace(b"2 passed", b"0 passed").replace(
                    b"0 ignored", b"2 ignored"
                ),
            ),
        ):
            with pytest.raises(CannotProbeError):
                _ = completed_test(output, code, failure_code)


def test_reference_only_crash_or_launch_failure_cannot_prove_a_parameter(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    text = vector(
        'id = "adjusted"\nentries = ["Step"]\nrules = { step = 1 }\nbefore = 2\nafter = 3\n'
    )
    path = demo(tmp_path, text, None)
    receipt = tmp_path / "receipt"

    def build(_root: Path, _crates: Sequence[str]) -> Mapping[str, Path]:
        built = tmp_path / f"build-{len(list(tmp_path.glob('build-*')))}"
        built.mkdir()
        return {"demo": fake_test(built, path, receipt, "crash")}

    monkeypatch.setattr(shell, "build", build)
    with pytest.raises(CannotProbeError):
        _ = check_domains(tmp_path, ("demo",))
    assert ["rules" in vectors(t)[-1] for _, t in receipts(receipt)] == [
        True,
        True,
        False,
    ]
    assert path.read_text(encoding="utf-8") == text

    def unlaunchable(_root: Path, _crates: Sequence[str]) -> Mapping[str, Path]:
        return {"demo": tmp_path / "missing"}

    monkeypatch.setattr(shell, "build", unlaunchable)
    with pytest.raises(CannotProbeError):
        _ = check_domains(tmp_path, ("demo",))
    assert path.read_text(encoding="utf-8") == text


def test_linux_selects_dedicated_vectors_or_library_adapter(
    tmp_path: Path,
) -> None:
    tests = tmp_path / "crates/clean/demo/tests"
    tests.mkdir(parents=True)
    assert target_of("demo", tmp_path) == ("--lib",)
    _ = (tests / "other.rs").write_text(
        "// Invented target.\n", encoding="utf-8"
    )
    assert target_of("demo", tmp_path) == ("--lib",)
    _ = (tests / "vectors.rs").write_text(
        "// Invented vector adapter.\n", encoding="utf-8"
    )
    assert target_of("demo", tmp_path) == ("--test", "vectors")


def test_nested_vectors_select_the_integration_target(tmp_path: Path) -> None:
    tests = tmp_path / "crates/clean/demo/tests/vectors"
    tests.mkdir(parents=True)
    _ = (tests / "main.rs").write_text(
        "// Invented vector adapter.\n", encoding="utf-8"
    )
    assert target_of("demo", tmp_path) == ("--test", "vectors")


def every_domain(_root: Path) -> FrozenList[str]:
    return FrozenList(("alpha", "beta"))


def selected_by(
    arguments: Sequence[str], root: Path, monkeypatch: pytest.MonkeyPatch
) -> tuple[list[str], list[Sequence[str]]]:
    owners_checked: list[str] = []
    domains_probed: list[Sequence[str]] = []

    def check_owners(_root: Path, name: str) -> None:
        owners_checked.append(name)

    def check_probes(_root: Path, names: Sequence[str]) -> int:
        domains_probed.append(tuple(names))
        return 0

    monkeypatch.setattr(shell, "domains", every_domain)
    monkeypatch.setattr(shell, "check_domain_owners", check_owners)
    monkeypatch.setattr(shell, "check_domains", check_probes)
    assert run(arguments, root) == 0
    return owners_checked, domains_probed


def test_owners_selects_the_check_and_its_domains(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    for arguments, owners, probed in (
        (("--owners",), ["alpha", "beta"], []),
        (("--owners", "beta"), ["beta"], []),
        ((), [], [("alpha", "beta")]),
        (("beta",), [], [("beta",)]),
    ):
        assert selected_by(arguments, tmp_path, monkeypatch) == (owners, probed)


def test_missing_names_each_parameter_entry_no_vector_adjusts_and_fails(
    capsys: pytest.CaptureFixture[str],
) -> None:
    status = run(("--missing", "demo"), FIXTURES)
    assert status == 1
    assert capsys.readouterr().out.splitlines() == [
        "adjusted: demo: Missing has no adjusted vector",
        "adjusted: demo: EmptyInline has no adjusted vector",
        "adjusted: demo: EmptyTable has no adjusted vector",
        "adjusted: demo: 2 of 5 Parameter entries have an adjusted vector",
    ]


def test_a_toml_error_in_the_owners_or_the_vectors_is_a_refusal(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    owners = "tools/cleanroom/adjusted_rule_owners/demo.toml"
    valid = {
        "docs/behaviours/demo.md": SPEC,
        "scenarios/vectors/demo.toml": vector(
            'id = "step"\nentries = ["Step"]\nrules = { step = 3 }\nbefore = 2\nafter = 5\n'
        ),
        owners: '[owners]\nstep = ["Step"]\n',
    }
    for case, broken in enumerate((
        None,
        owners,
        "scenarios/vectors/demo.toml",
        "tools/cleanroom/adjusted_probe_tests.toml",
    )):
        root = tmp_path / str(case)
        for name, text in valid.items():
            (root / name).parent.mkdir(parents=True, exist_ok=True)
            _ = (root / name).write_text(text, encoding="utf-8", newline="\n")
        if broken is not None:
            _ = (root / broken).write_text(
                "[owners\n", encoding="utf-8", newline="\n"
            )
        status = run(("--owners", "demo"), root)
        errors = capsys.readouterr().err
        assert status == (0 if broken is None else 2), errors
        assert bool(errors) == (broken is not None)
        assert not errors or errors.startswith("adjusted_vectors_run: ")
