#!/usr/bin/env python3
"""Check the historical claim ledger, without executing any game or replay."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import sys
import tomllib
from typing import Any


class InvalidTransition(ValueError):
    """A historical claim or its replacement evidence is missing or inconsistent."""


def require(condition: bool, identity: str) -> None:
    if not condition:
        raise InvalidTransition(identity)


def valid_sha256(value: Any) -> bool:
    return (isinstance(value, str) and len(value) == 64
            and all(character in "0123456789abcdef" for character in value))


def flattened(values: dict[str, Any], prefix: str = "") -> list[tuple[str, Any]]:
    result: list[tuple[str, Any]] = []
    for key, value in values.items():
        name = f"{prefix}.{key}" if prefix else key
        if isinstance(value, dict):
            result.extend(flattened(value, name))
        else:
            result.append((name, value))
    return result


def prior_atoms(prior: dict[str, Any]) -> dict[tuple[str, str], tuple[str, list[int]]]:
    """Recover each obligation from the pinned original metadata, including zero floors."""
    atoms: dict[tuple[str, str], tuple[str, list[int]]] = {}
    for snapshot in prior["snapshot"]:
        path, text = snapshot["path"], snapshot["text"]
        digest = hashlib.sha256(text.encode("utf-8")).hexdigest()
        require(digest == snapshot["sha256"], f"snapshot pin: {path}")
        lines = [line for line in text.splitlines() if line.strip() and not line.startswith("#")]
        require(len(lines) == snapshot["noncomment_lines"], f"snapshot count: {path}")
        entries: list[tuple[str, list[int]]] = []
        if path.endswith(".toml"):
            values = tomllib.loads(text)
            groups = ([(row["row"], row) for row in values["row"]]
                      if "row" in values else list(values.items()))
            for group, fields in groups:
                for field, value in flattened(fields):
                    if field != "row":
                        require(type(value) is int, f"snapshot field: {path}/{group}/{field}")
                        entries.append((f"{group}/{field}", [value]))
        else:
            for line in lines:
                parts = line.split()
                if path.endswith("coverage.txt"):
                    suffix = (parts[2:3] if parts[1] == "entry" else
                              [" ".join(parts[4:])] if parts[1] == "field" else [])
                    identity = "/".join(parts[:2] + suffix)
                elif path.endswith("lockstep.ratchet.txt"):
                    identity = "/".join(parts[:3] if parts[0] == "diff" else parts[:2])
                else:
                    identity = "/".join(parts[:2])
                entries.append((identity, [int(part) for part in parts[2:] if part.isdecimal()]))
        for identity, counts in entries:
            key = (path, identity)
            require(key not in atoms, f"duplicate historical identity: {path}/{identity}")
            atoms[key] = (digest, counts)
    for requirement in prior.get("requirement", []):
        key = (requirement["source_path"], requirement["identity"])
        require(key not in atoms, f"duplicate explicit requirement: {'/'.join(key)}")
        require(bool(requirement["meaning"].strip()), "explicit requirement meaning")
        atoms[key] = (requirement["source_sha256"], requirement["before_values"])
    require(bool(atoms), "empty historical claims")
    return atoms


def validate(prior: dict[str, Any], transition: dict[str, Any], baseline: dict[str, Any]) -> int:
    require(transition["format"] == 1, "transition format")
    require(transition["source"] == prior["source"], "historical source")
    require(transition["status"] == "reviewed", "transition independent review pending")
    review = transition["review_sha256"]
    require(valid_sha256(review), "review receipt pin")
    atoms = prior_atoms(prior)
    floors = {case["id"]: case for case in baseline["case"]}
    require(len(floors) == len(baseline["case"]), "duplicate replacement case")
    internal = {check["id"]: check for check in transition.get("development_check", [])}
    require(len(internal) == len(transition.get("development_check", [])), "duplicate content check")
    seen: set[tuple[str, str]] = set()
    for claim in transition["claim"]:
        key = (claim["source_path"], claim["identity"])
        label = "/".join(key)
        require(key in atoms and key not in seen, f"unknown or duplicate claim: {label}")
        seen.add(key)
        require((claim["source_sha256"], claim["before_values"]) == atoms[key],
                f"changed historical claim: {label}")
        disposition = claim["disposition"]
        require(disposition in {"preserved", "replaced", "retired", "unsupported", "internal"},
                f"unresolved disposition: {label}")
        require(bool(claim["reason"].strip()), f"missing reason: {label}")
        witnesses = claim["witness_case_ids"]
        require(len(set(witnesses)) == len(witnesses), f"duplicate witness: {label}")
        for witness in witnesses:
            require(witness in floors, f"missing replacement case: {label}/{witness}")
        if disposition in {"preserved", "replaced"}:
            require(bool(witnesses), f"replacement without witness: {label}")
            require(any(floors[witness]["claim"] == "behaviour"
                        and floors[witness]["expected"]["status"] == "pass"
                        for witness in witnesses), f"replacement without passing behaviour: {label}")
        if disposition in {"retired", "unsupported"}:
            require(bool(claim["lost_observations"]), f"undisclosed loss: {label}")
        if disposition == "preserved":
            require(not claim["lost_observations"], f"preserved claim with loss: {label}")
        if disposition == "internal":
            require(claim["identity"].startswith("content/") and not witnesses,
                    f"gameplay claim cannot become a content check: {label}")
            checks = claim.get("test_ids", [])
            require(bool(checks) and not claim["lost_observations"], f"missing content proof: {label}")
            for check in checks:
                require(check in internal and internal[check]["status"] == "pass"
                        and valid_sha256(internal[check]["receipt_sha256"]),
                        f"unmeasured content check: {label}/{check}")
    require(seen == set(atoms), "unclassified historical claims")
    return len(seen)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    arguments = parser.parse_args()
    directory = arguments.root / "scenarios/replay"
    try:
        def read(name: str) -> dict[str, Any]:
            with (directory / name).open("rb") as source:
                return tomllib.load(source)
        count = validate(read("prior-claims.toml"), read("transition.toml"), read("baseline.toml"))
    except (OSError, ValueError, KeyError, TypeError, IndexError) as error:
        print(f"REPLAY TRANSITION FAIL: {error}", file=sys.stderr)
        return 1
    print(f"REPLAY TRANSITION OK: {count} historical claims accounted for")
    return 0


if __name__ == "__main__":
    sys.exit(main())
