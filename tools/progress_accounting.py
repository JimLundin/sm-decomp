#!/usr/bin/env python3
"""Count physical Rust and explicitly mapped superseded sections at a Git pin."""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import hashlib
import sys

sys.dont_write_bytecode = True


def function(text, name, lexer):
    tokens = lexer(text)
    words = [text[a:b] for _, a, b in tokens]
    matches = []
    for i in range(len(tokens) - 2):
        if words[i:i + 2] != ["fn", name]:
            continue
        begin = i
        while begin < len(tokens) and words[begin] not in ("{", ";"):
            begin += 1
        if begin == len(tokens) or words[begin] == ";":
            continue
        end, depth = begin, 1
        while depth and end + 1 < len(tokens):
            end += 1
            depth += (words[end] == "{") - (words[end] == "}")
        assert not depth, f"unclosed {name}"
        lo = text.count("\n", 0, tokens[i][1]) + 1
        hi = text.count("\n", 0, tokens[end][2]) + 1
        lines = text.splitlines(keepends=True)
        section = "".join(lines[lo - 1:hi]).replace("\r\n", "\n")
        matches.append((lo, hi, hashlib.sha256(section.encode()).hexdigest()))
    assert len(matches) <= 1, f"ambiguous {name}"
    return matches[0] if matches else None


def discounts(kept, mapping, lexer):
    covered, counted, results = set(), set(), []
    operations = signatures = 0
    for record in mapping["sections"]:
        for key in ("source_pin", "function", "replacement", "actual_callers", "proof", "retention_reason", "deletion_condition"):
            assert record.get(key), f"missing mapping field {key}"
        assert record["unsafe_operation_sites"] >= 0 and record["unsafe_signatures"] >= 0
        found = function(kept.get(record["file"], ""), record["function"], lexer)
        if found is None:
            results.append({"function": record["function"], "status": "absent; discount removed", "lines": 0})
            continue
        lo, hi, digest = found
        assert digest == record["sha256"], f"changed section requires mapping: {record['function']}"
        assert not any(p in ("verification", "verify") for p in Path(record["file"]).parts) and not Path(record["file"]).name.startswith("verify")
        for replacement in record["replacement"]:
            actual = function(kept.get(replacement["file"], ""), replacement["function"], lexer)
            assert actual and actual[2] == replacement["sha256"], "changed replacement requires review"
        identity = (record["file"], record["function"])
        lines = {(record["file"], line) for line in range(lo, hi + 1)}
        assert identity in counted or not covered.intersection(lines), "overlapping functions require a deduplicated operation map"
        covered.update(lines)
        if identity not in counted:
            operations += record["unsafe_operation_sites"]
            signatures += record["unsafe_signatures"]
            counted.add(identity)
        results.append({"function": record["function"], "status": "retained compatibility", "span": [lo, hi], "lines": hi - lo + 1})
    return len(covered), operations, signatures, results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ref", default="HEAD")
    parser.add_argument("--superseded", type=Path)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    repo = args.repo.resolve()
    pin = subprocess.check_output(["git", "rev-parse", args.ref], cwd=repo, text=True).strip()
    mapping = json.loads(args.superseded.read_text(encoding="utf-8-sig")) if args.superseded else {"sections": []}
    needed = {r["file"] for r in mapping["sections"]}
    needed.update(p["file"] for r in mapping["sections"] for p in r["replacement"])
    kept, totals = {}, dict(production=0, verification=0, legacy=0, unsafe_containing=0)
    rows = subprocess.check_output(["git", "ls-tree", "-r", "-z", pin, "sm/src"], cwd=repo).split(b"\0")
    with subprocess.Popen(["git", "cat-file", "--batch"], cwd=repo, stdin=subprocess.PIPE, stdout=subprocess.PIPE) as batch:
        for row in filter(None, rows):
            meta, name = row.split(b"\t", 1)
            rel = name.decode()
            if not rel.endswith(".rs"):
                continue
            batch.stdin.write(meta.split()[2] + b"\n")
            batch.stdin.flush()
            size = int(batch.stdout.readline().split()[-1])
            data = batch.stdout.read(size)
            assert batch.stdout.read(1) == b"\n"
            parts, lines = Path(rel).parts, data.splitlines()
            verification = any(p in ("verification", "verify") for p in parts) or Path(rel).name.startswith("verify")
            totals["verification" if verification else "production"] += len(lines)
            totals["unsafe_containing"] += sum(b"unsafe" in line for line in lines)
            if Path(rel).parent.as_posix() == "sm/src/game" and Path(rel).name.startswith("sm_"):
                totals["legacy"] += len(lines)
            if rel in needed:
                kept[rel] = data.decode()
        batch.stdin.close()
        assert batch.wait() == 0
    spec = importlib.util.spec_from_file_location("unsafe_lexer", repo / "tools/codemod/strip_unsafe_fixpoint.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    covered, operations, signatures, results = discounts(kept, mapping, module.lex)
    print(json.dumps({"commit": pin, "physical_production": totals["production"], "mapped_superseded_retained": covered,
                      "adjusted_implementation": totals["production"] - covered, "other_physical_counts": totals,
                      "mapped_retained_unsafe_operation_sites": operations, "mapped_retained_unsafe_signatures": signatures,
                      "global_unsafe_operations_and_signatures": "pending mapping", "sections": results,
                      "limits": "Adjustment is an accounting discount, not deletion or inactive-code proof; missing sections lose their discount. Unmapped code remains counted."}, indent=2))


if __name__ == "__main__":
    main()
