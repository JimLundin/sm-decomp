#!/usr/bin/env python3
"""Run complete native enabled-pattern vectors through production emitted bodies."""
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tarfile
import time
from pathlib import Path

from readiness import Run, digest, write_json, source_inventory, inventory_digest
from pattern_contract import load_declaration, identity, pattern_conditions, DECLARATION, FIXTURES

OPCODES = ("a9", "a2", "c2", "e2", "8d", "ea", "ca")


def boundary_cases():
    """Source-owned independent expectations; never generated from the emitter."""
    root = Path(__file__).resolve().parent.parent
    return json.loads((root / "tools/pattern_fixtures.json").read_text())


def command(argv, root, output, name, env=None):
    started = time.monotonic()
    with (output / (name + ".stdout.log")).open("wb") as stdout, (output / (name + ".stderr.log")).open("wb") as stderr:
        result = subprocess.run([str(item) for item in argv], cwd=root, env=env, stdout=stdout, stderr=stderr)
    write_json(output / (name + ".json"), {"command": [str(item) for item in argv], "returncode": result.returncode,
                "elapsed_seconds": time.monotonic() - started, "stdout":name+".stdout.log", "stderr":name+".stderr.log", "complete":result.returncode>=0, "status":"pass" if result.returncode==0 else "divergence" if result.returncode==1 else "error", "signal":-result.returncode if result.returncode<0 else None})
    return result.returncode


def run(corpus, output, opcodes=None, batch_size=500, timing=False, declaration_path=None):
    root = Path(__file__).resolve().parent.parent
    output.mkdir(parents=True, exist_ok=False)
    started = time.monotonic()
    declaration = load_declaration(root, declaration_path)
    shutil.copy2(root / DECLARATION, output / 'declaration.json')
    shutil.copy2(root / FIXTURES, output / 'independent-fixtures.json')
    declared = tuple(pattern["opcode"] for pattern in declaration["patterns"])
    opcodes = declared if opcodes is None else tuple(opcodes)
    if not opcodes or len(set(opcodes)) != len(opcodes) or not set(opcodes) <= set(declared):
        raise ValueError("Selected opcodes outside production declaration")
    patterns = {pattern["opcode"]: pattern for pattern in declaration["patterns"]}
    report = {"schema_version": 2, "status": "incomplete", "expected": 0, "run": 0,
              "passed": 0, "failed": 0, "skipped": 0, "opcodes": {}, "corpus": [],
              "scope": "complete production declaration" if set(opcodes) == set(declared) else "development selected opcodes",
              "declaration": declaration, "declaration_sha256": identity(declaration),
              "declaration_file_sha256": digest(root / DECLARATION),
              "declaration_path": DECLARATION, "fixtures_path": FIXTURES,
              "fixtures_sha256": digest(root / FIXTURES), "coverage": {},
              "corpus_verified": False,
              "expectation_source": "SingleStepTests/65816 native JSON; independent literal boundary fixtures",
              "unsupported_debt": "ADC overflow remains unsupported; docs/SETUP.md and workflows/recomp-iteration.md"}
    write_json(output / "instruction.json", report)
    sources = source_inventory(root)
    report["source_sha256"] = inventory_digest(sources)
    report["source_inventory"] = sources
    compiler = subprocess.run(["rustc", "--version", "--verbose"], cwd=root,capture_output=True,text=True,check=True)
    report["compiler"] = compiler.stdout
    report["build_settings"] = ["cargo build", "--locked", "--offline", "debug", "isolated target"]
    dependency = next(item for item in json.loads((root / "tools/dependencies.json").read_text()) if item["name"] == "65816")
    report["pinned_dependency"] = dependency
    if corpus == (root / "deps/65816/v1").resolve():
        archive = root / "deps/downloads" / dependency["archive"]
        if not archive.is_file() or digest(archive) != dependency["sha256"]:
            raise RuntimeError("Complete corpus requires the verified pinned download; run just setup")
        remaining=set(opcodes)
        with tarfile.open(archive,"r|gz") as tree:
            for member in tree:
                opcode=Path(member.name).name.partition(".")[0]
                if opcode in remaining and member.name.endswith("/v1/"+opcode+".n.json"):
                    if hashlib.sha256(tree.extractfile(member).read()).hexdigest() != digest(corpus/(opcode+".n.json")):
                        raise RuntimeError("Consumed corpus file differs from pinned archive: "+opcode)
                    remaining.remove(opcode)
                    if not remaining:
                        break
        if remaining:
            raise RuntimeError("Pinned archive lacks declared opcodes: "+",".join(sorted(remaining)))
        report['corpus_verified'] = True
        report['corpus_archive_sha256'] = digest(archive)
    cases = []
    (output / 'corpus').mkdir()
    for opcode in opcodes:
        path = corpus / (opcode + ".n.json")
        selected = json.loads(path.read_text())
        shutil.copy2(path, output / 'corpus' / path.name)
        if not selected:
            raise ValueError("Complete declared corpus is empty: " + opcode)
        report["corpus"].append({"opcode": opcode, "sha256": digest(path), "cases": len(selected)})
        report["opcodes"][opcode] = {"expected": len(selected), "run": 0, "passed": 0, "failed": 0, "skipped": 0}
        for case in selected:
            if case["initial"]["e"] != 0 or case["final"]["e"] != 0 or ((case["initial"]["p"] >> 4) & 3) not in patterns[opcode]["mx"]:
                raise ValueError("Declared native corpus contains unsupported mode/width: " + opcode)
            case["case_sha256"] = hashlib.sha256(json.dumps(case, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
            case["opcode"] = opcode
        cases.extend(selected)
    report["expected"] = len(cases)
    fixtures = {case["name"]: case for case in boundary_cases()}
    additions = []
    for opcode in opcodes:
        report["opcodes"][opcode]["independent_fixtures"] = patterns[opcode]["independent_fixtures"]
        for name in patterns[opcode]["independent_fixtures"]:
            if name not in fixtures:
                raise ValueError("Declared independent fixture missing: " + name)
            case = fixtures[name]
            pc = (case["initial"]["pbr"] << 16) | case["initial"]["pc"]
            if dict(case["initial"]["ram"])[pc] != int(opcode, 16):
                raise ValueError("Independent fixture opcode differs")
            case["case_sha256"] = identity(case)
            case["opcode"] = opcode
            case["independent_fixture"] = True
            additions.append(case)
            report["opcodes"][opcode]["expected"] += 1
    cases.extend(additions)
    report["boundary_fixtures"] = len(additions)
    report["expected"] = len(cases)
    for opcode in opcodes:
        report["coverage"][opcode] = {condition: {"expected": 0, "passed": 0} for condition in
            patterns[opcode]["required_conditions"] + patterns[opcode]["timing_branches"]}
    for case in cases:
        case["conditions"] = pattern_conditions(case, patterns[case["opcode"]])
        for condition in case["conditions"]:
            report["coverage"][case["opcode"]][condition]["expected"] += 1
    with (output/"expected-cases.jsonl").open("w") as inventory:
        for case in cases:
            inventory.write(json.dumps({"name":case["name"],"case_sha256":case["case_sha256"],"opcode":case["opcode"],"conditions":case["conditions"]},sort_keys=True)+"\n")
    report["expected_inventory_sha256"]=digest(output/"expected-cases.jsonl")
    write_json(output / "instruction.json", report)
    if not cases:
        return 2
    target = output / "build"
    environment = dict(os.environ, CARGO_TARGET_DIR=str(target))
    if command(["cargo", "build", "-p", "fuzz", "--bin", "timed-gen", "--locked", "--offline"], root, output, "generator-build", environment):
        report["status"] = "error"
        write_json(output / "instruction.json", report)
        return 2
    generator=output/"timed-gen";shutil.copy2(target/"debug/timed-gen",generator)
    report["generator"]={"path":"timed-gen","sha256":digest(generator)}
    with (output / "cases.jsonl").open("w") as combined:
        for number, index in enumerate(range(0, len(cases), batch_size)):
            if source_inventory(root) != sources or any(digest(corpus/(entry["opcode"]+".n.json")) != entry["sha256"] for entry in report["corpus"]):
                raise RuntimeError("Source or corpus changed during validation")
            batch = output / ("batch-{:04d}".format(number))
            batch.mkdir()
            input_path = batch / "input.json"
            write_json(input_path, cases[index:index + batch_size])
            bodies, prepared, ledger = batch / "bodies.rs", batch / "prepared.json", batch / "cases.jsonl"
            if command([generator, input_path, bodies, prepared], root, batch, "generate", environment):
                report["status"] = "error"
                break
            environment["SM_TIMED_STEPS"] = str(bodies)
            if command(["cargo", "build", "-p", "fuzz", "--bin", "timed-run", "--locked", "--offline"], root, batch, "build", environment):
                report["status"] = "error"
                break
            executable=batch/"timed-run";shutil.copy2(target/"debug/timed-run",executable)
            code = command([executable, prepared, ledger], root, batch, "execute", environment)
            if code not in (0, 1) or not ledger.is_file():
                report["status"] = "incomplete"
                break
            rows = [json.loads(line) for line in ledger.read_text().splitlines()]
            if len(rows) != len(cases[index:index + batch_size]):
                report["status"] = "incomplete"
                break
            for row in rows:
                position = report["run"]
                expected_case = cases[position]
                if (row.get("name") != expected_case["name"] or row.get("case_sha256") != expected_case["case_sha256"]
                        or row.get("status") not in ("pass", "divergence")):
                    report["status"] = "incomplete"
                    raise RuntimeError("Case ledger identity, order or terminal status mismatch")
                combined.write(json.dumps(row, sort_keys=True) + "\n")
                passed = row["status"] == "pass"
                report["run"] += 1
                report["passed" if passed else "failed"] += 1
                count = report["opcodes"][row["opcode"]]
                count["run"] += 1
                count["passed" if passed else "failed"] += 1
                if passed:
                    for condition in expected_case["conditions"]:
                        report["coverage"][row["opcode"]][condition]["passed"] += 1
            write_json(batch / "identities.json", {"bodies_sha256": digest(bodies), "prepared_sha256": digest(prepared),
                "binary": "timed-run", "binary_sha256": digest(executable), "input_sha256":digest(input_path), "ledger_sha256":digest(ledger)})
            combined.flush()
            write_json(output / "instruction.json", report)
        else:
            report["status"] = "divergence" if report["failed"] else "pass"
    report["elapsed_seconds"] = time.monotonic() - started
    if source_inventory(root) != sources:
        raise RuntimeError("Source changed during validation")
    write_json(output / "instruction.json", report)
    if timing:
        timing_started = time.monotonic()
        timing_command = ["cargo", "test", "-p", "snes"]
        for test in declaration["timing_tests"]:
            timing_command.extend(["--test", test])
        timing_command.extend(["--locked", "--offline"])
        code = command(timing_command, root, output, "timing-execution", environment)
        timing_report={"schema_version": 2, "status": "pass" if code == 0 else "error",
            "complete": code == 0, "returncode": code, "process_evidence":"timing-execution.json", "process_sha256":digest(output/"timing-execution.json"), "elapsed_seconds": time.monotonic()-timing_started,
            "scope": report["scope"], "source_sha256": report["source_sha256"],
            "declaration": declaration, "declaration_sha256": identity(declaration),
            "oracles": {name: digest(root/name) for name in declaration["timing_oracles"]},
            "sources": declaration["timing_sources"], "tests": declaration["timing_tests"],
            "conditions": {"applicable": declaration["hardware_conditions"], "inapplicable": declaration["exclusions"]}}
        write_json(output / "timing.json", timing_report)
        if code:
            return 2
        if source_inventory(root) != sources:
            raise RuntimeError("Source changed during timing validation")
    return 0 if report["status"] == "pass" else 1 if report["status"] == "divergence" else 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, default=Path("deps/65816/v1"))
    parser.add_argument("--out", type=Path)
    parser.add_argument("--batch-size", type=int, default=500)
    parser.add_argument("--opcodes", help="development subset of production-declared opcodes")
    parser.add_argument("--declaration", type=Path, help="must match exact production source declaration")
    parser.add_argument("--timing", action="store_true")
    args = parser.parse_args()
    if args.batch_size <= 0:
        parser.error("batch size must be positive")
    evidence_run = None
    if args.out is None:
        evidence_run = Run.create(Path(__file__).resolve().parent.parent,"patterns",{"opcodes":args.opcodes,"native":True})
        print("Run: "+str(evidence_run.path),flush=True)
        print("Brief: "+str(evidence_run.path/"brief.md"),flush=True)
        evidence_run.capture_source()
        output = evidence_run.path / "checks/patterns"
    else:
        output = args.out.resolve()
    if output.exists():
        parser.error("output must not already exist; completed evidence is immutable")
    try:
        result = run(args.corpus.resolve(), output, tuple(args.opcodes.split(",")) if args.opcodes else None, args.batch_size,args.timing,args.declaration)
        if evidence_run:
            evidence_run.check_sources()
            report=json.loads((output/"instruction.json").read_text())
            evidence_run.record_check("instruction",dict(report,complete=report["run"]==report["expected"]))
            if args.timing and (output/"timing.json").exists():
                evidence_run.record_check("timing",json.loads((output/"timing.json").read_text()))
            return evidence_run.finalize("pass" if result==0 else "divergence" if result==1 else "error",
                                         "patterns","Instruction/timing scope only; readiness gates remain pending",
                                         ["instruction","timing"] if args.timing else ["instruction"])
        return result
    except (Exception, KeyboardInterrupt) as error:
        report_path = output / "instruction.json"
        report = json.loads(report_path.read_text()) if report_path.exists() else {"schema_version": 1}
        report.update(status="incomplete" if isinstance(error, KeyboardInterrupt) else "error", reason=str(error))
        write_json(report_path, report)
        if evidence_run:
            return evidence_run.finalize(report["status"],"patterns",str(error))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
