import hashlib
from dataclasses import dataclass
from string import hexdigits
lazy from collections.abc import Mapping

from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

from sm_tools.ci.core.decoded import (
    array,
    field,
    is_table,
    optional_tables,
    optional_texts,
    table,
    tables,
    text,
    texts,
    toml,
)

DESCRIPTION = (
    "Check the historical claim ledger, without executing any game or replay."
)
LEDGER = FrozenList(("prior-claims.toml", "transition.toml", "baseline.toml"))
DISPOSITIONS = FrozenOrderedSet((
    "preserved",
    "replaced",
    "retired",
    "unsupported",
    "internal",
))
WITNESSED = FrozenOrderedSet(("preserved", "replaced"))
LOSSY = FrozenOrderedSet(("retired", "unsupported"))
LOWER_HEX = FrozenOrderedSet(hexdigits.lower())
SHA256_DIGITS = 64

type Key = tuple[str, str]
type Atom = tuple[object, FrozenList[object]]
type Entry = tuple[str, FrozenList[object]]


class InvalidTransitionError(ValueError):
    pass


@dataclass(frozen=True, slots=True)
class Outcome:
    status: int
    line: str


def require(identity: str, *, holds: bool) -> None:
    if not holds:
        raise InvalidTransitionError(identity)


def valid_sha256(value: object) -> bool:
    return (
        isinstance(value, str)
        and len(value) == SHA256_DIGITS
        and all(character in LOWER_HEX for character in value)
    )


def flattened(
    values: Mapping[str, object], prefix: str = ""
) -> FrozenList[tuple[str, object]]:
    return FrozenList(
        pair
        for key, value in values.items()
        for pair in (
            flattened(table(value), f"{prefix}.{key}" if prefix else key)
            if is_table(value)
            else ((f"{prefix}.{key}" if prefix else key, value),)
        )
    )


def toml_entries(path: str, source: str) -> FrozenList[Entry]:
    values = toml(source)
    groups = (
        FrozenList((field(row, "row"), row) for row in tables(values, "row"))
        if "row" in values
        else FrozenList(values.items())
    )
    entries: list[Entry] = []
    for group, fields in groups:
        for name, value in flattened(table(fields)):
            if name != "row":
                require(
                    f"snapshot field: {path}/{group}/{name}",
                    holds=type(value) is int,
                )
                entries.append((f"{group}/{name}", FrozenList((value,))))
    return FrozenList(entries)


def line_identity(path: str, parts: FrozenList[str]) -> str:
    if path.endswith("coverage.txt"):
        match parts[1]:
            case "entry":
                suffix = parts[2:3]
            case "field":
                suffix = FrozenList((" ".join(parts[4:]),))
            case _:
                suffix = FrozenList[str]()
        return "/".join((*parts[:2], *suffix))
    if path.endswith("lockstep.ratchet.txt"):
        return "/".join(parts[:3] if parts[0] == "diff" else parts[:2])
    return "/".join(parts[:2])


def line_entry(path: str, parts: FrozenList[str]) -> Entry:
    counts = FrozenList[object](
        int(part) for part in parts[2:] if part.isdecimal()
    )
    return (line_identity(path, parts), counts)


def prior_atoms(prior: Mapping[str, object]) -> FrozenOrderedDict[Key, Atom]:
    atoms: dict[Key, Atom] = {}
    for snapshot in tables(prior, "snapshot"):
        path = text(snapshot["path"])
        source = text(snapshot["text"])
        digest = hashlib.sha256(source.encode("utf-8")).hexdigest()
        require(f"snapshot pin: {path}", holds=digest == snapshot["sha256"])
        lines = FrozenList(
            line
            for line in source.splitlines()
            if line.strip() and not line.startswith("#")
        )
        require(
            f"snapshot count: {path}",
            holds=len(lines) == snapshot["noncomment_lines"],
        )
        entries = (
            toml_entries(path, source)
            if path.endswith(".toml")
            else FrozenList(
                line_entry(path, FrozenList(line.split())) for line in lines
            )
        )
        for identity, counts in entries:
            key = (path, identity)
            require(
                f"duplicate historical identity: {path}/{identity}",
                holds=key not in atoms,
            )
            atoms[key] = (digest, counts)
    for requirement in optional_tables(prior, "requirement"):
        key = (text(requirement["source_path"]), text(requirement["identity"]))
        require(
            f"duplicate explicit requirement: {'/'.join(key)}",
            holds=key not in atoms,
        )
        require(
            "explicit requirement meaning",
            holds=bool(text(requirement["meaning"]).strip()),
        )
        atoms[key] = (
            requirement["source_sha256"],
            array(requirement["before_values"]),
        )
    require("empty historical claims", holds=bool(atoms))
    return FrozenOrderedDict(atoms)


def check_claim(
    claim: Mapping[str, object],
    label: str,
    floors: Mapping[str, Mapping[str, object]],
    internal: Mapping[str, Mapping[str, object]],
) -> None:
    disposition = claim["disposition"]
    require(
        f"unresolved disposition: {label}", holds=disposition in DISPOSITIONS
    )
    require(
        f"missing reason: {label}", holds=bool(text(claim["reason"]).strip())
    )
    witnesses = texts(claim, "witness_case_ids")
    require(
        f"duplicate witness: {label}",
        holds=len(FrozenOrderedSet(witnesses)) == len(witnesses),
    )
    for witness in witnesses:
        require(
            f"missing replacement case: {label}/{witness}",
            holds=witness in floors,
        )
    lost = bool(claim["lost_observations"])
    if disposition in WITNESSED:
        require(f"replacement without witness: {label}", holds=bool(witnesses))
        require(
            f"replacement without passing behaviour: {label}",
            holds=any(
                floors[witness]["claim"] == "behaviour"
                and field(floors[witness]["expected"], "status") == "pass"
                for witness in witnesses
            ),
        )
    if disposition in LOSSY:
        require(f"undisclosed loss: {label}", holds=lost)
    if disposition == "preserved":
        require(f"preserved claim with loss: {label}", holds=not lost)
    if disposition == "internal":
        require(
            f"gameplay claim cannot become a content check: {label}",
            holds=text(claim["identity"]).startswith("content/")
            and not witnesses,
        )
        checks = optional_texts(claim, "test_ids")
        require(
            f"missing content proof: {label}", holds=bool(checks) and not lost
        )
        for check in checks:
            require(
                f"unmeasured content check: {label}/{check}",
                holds=check in internal
                and internal[check]["status"] == "pass"
                and valid_sha256(internal[check]["receipt_sha256"]),
            )


def by_id(
    rows: FrozenList[Mapping[str, object]], identity: str
) -> FrozenOrderedDict[str, Mapping[str, object]]:
    keyed = FrozenOrderedDict((text(row["id"]), row) for row in rows)
    require(identity, holds=len(keyed) == len(rows))
    return keyed


def validate(
    prior: Mapping[str, object],
    transition: Mapping[str, object],
    baseline: Mapping[str, object],
) -> int:
    require("transition format", holds=transition["format"] == 1)
    require("historical source", holds=transition["source"] == prior["source"])
    require(
        "transition independent review pending",
        holds=transition["status"] == "reviewed",
    )
    require(
        "review receipt pin", holds=valid_sha256(transition["review_sha256"])
    )
    atoms = prior_atoms(prior)
    floors = by_id(tables(baseline, "case"), "duplicate replacement case")
    internal = by_id(
        optional_tables(transition, "development_check"),
        "duplicate content check",
    )
    seen: dict[Key, None] = {}
    for claim in tables(transition, "claim"):
        key = (text(claim["source_path"]), text(claim["identity"]))
        label = "/".join(key)
        require(
            f"unknown or duplicate claim: {label}",
            holds=key in atoms and key not in seen,
        )
        seen[key] = None
        require(
            f"changed historical claim: {label}",
            holds=(claim["source_sha256"], array(claim["before_values"]))
            == atoms[key],
        )
        check_claim(claim, label, floors, internal)
    require("unclassified historical claims", holds=seen.keys() == atoms.keys())
    return len(seen)


def checked(sources: FrozenList[str]) -> Outcome:
    try:
        prior, transition, baseline = (toml(source) for source in sources)
        count = validate(prior, transition, baseline)
    except (ValueError, KeyError, TypeError, IndexError) as error:
        return failed(error)
    return Outcome(
        0, f"REPLAY TRANSITION OK: {count} historical claims accounted for"
    )


def failed(error: Exception) -> Outcome:
    return Outcome(1, f"REPLAY TRANSITION FAIL: {error}")
