import hashlib
from itertools import starmap
lazy from collections.abc import Mapping, Sequence

import pytest
from ordered import FrozenOrderedDict

from sm_tools.ci.core.check_replay_transition import (
    InvalidTransitionError,
    validate,
)

TEXT = "movement first 4\nmovement second 0\n"
DIGEST = hashlib.sha256(TEXT.encode()).hexdigest()
FLOORS = (("first", 4), ("second", 0))
REQUIREMENT = FrozenOrderedDict[str, object]({
    "source_path": "content-tests.rs",
    "source_sha256": "b" * 64,
    "identity": "content/pose",
    "before_values": [1],
    "meaning": "pose content integrity",
})


def prior(
    requirements: Sequence[Mapping[str, object]] = (),
) -> dict[str, object]:
    snapshot = {
        "path": "floors.txt",
        "text": TEXT,
        "sha256": DIGEST,
        "noncomment_lines": 2,
    }
    found: dict[str, object] = {"source": "fixed", "snapshot": [snapshot]}
    if requirements:
        found["requirement"] = list(requirements)
    return found


def claim(
    name: str,
    count: int,
    disposition: str = "replaced",
    lost: Sequence[str] = (),
) -> dict[str, object]:
    return {
        "source_path": "floors.txt",
        "source_sha256": DIGEST,
        "identity": f"movement/{name}",
        "before_values": [count],
        "disposition": disposition,
        "reason": "Independent scoped witness.",
        "witness_case_ids": ["movement/case"],
        "lost_observations": list(lost),
    }


def claims() -> list[dict[str, object]]:
    return list(starmap(claim, FLOORS))


def transition(
    entries: Sequence[Mapping[str, object]],
    checks: Sequence[Mapping[str, object]] = (),
) -> dict[str, object]:
    found: dict[str, object] = {
        "format": 1,
        "source": "fixed",
        "status": "reviewed",
        "review_sha256": "a" * 64,
        "claim": list(entries),
    }
    if checks:
        found["development_check"] = list(checks)
    return found


def baseline(
    kind: str = "behaviour", status: str = "pass"
) -> dict[str, object]:
    case = {
        "id": "movement/case",
        "claim": kind,
        "expected": {"status": status},
    }
    return {"case": [case]}


def test_zero_floor_still_requires_a_disposition() -> None:
    assert validate(prior(), transition(claims()), baseline()) == 2
    with pytest.raises(InvalidTransitionError):
        _ = validate(prior(), transition(claims()[:1]), baseline())


def test_unsupported_is_not_replacement_behaviour_proof() -> None:
    with pytest.raises(InvalidTransitionError):
        _ = validate(
            prior(), transition(claims()), baseline("boundary", "unsupported")
        )


def test_retirement_requires_an_explicit_loss() -> None:
    retired = [claim("first", 4, "retired"), claim("second", 0)]
    with pytest.raises(InvalidTransitionError):
        _ = validate(prior(), transition(retired), baseline())
    disclosed = [
        claim("first", 4, "retired", ("former complete-row coverage",)),
        claim("second", 0),
    ]
    assert validate(prior(), transition(disclosed), baseline()) == 2


def test_duplicate_and_modified_history_are_rejected() -> None:
    with pytest.raises(InvalidTransitionError):
        _ = validate(
            prior(), transition([*claims(), claim("first", 4)]), baseline()
        )
    with pytest.raises(InvalidTransitionError):
        _ = validate(
            prior(),
            transition([claim("first", 3), claim("second", 0)]),
            baseline(),
        )


def test_internal_content_receipt_requires_a_sha256_pin() -> None:
    internal = {
        **REQUIREMENT,
        "disposition": "internal",
        "reason": "content test retained",
        "witness_case_ids": [],
        "test_ids": ["pose"],
        "lost_observations": [],
    }
    entries = [*claims(), internal]

    def receipt(digest: str) -> list[Mapping[str, object]]:
        return [{"id": "pose", "status": "pass", "receipt_sha256": digest}]

    with pytest.raises(InvalidTransitionError):
        _ = validate(
            prior((REQUIREMENT,)),
            transition(entries, receipt("x" * 64)),
            baseline(),
        )
    assert (
        validate(
            prior((REQUIREMENT,)),
            transition(entries, receipt("c" * 64)),
            baseline(),
        )
        == 3
    )
