#!/usr/bin/env python3
"""Run complete native enabled-pattern vectors through production emitted bodies."""
import argparse
import hashlib
import json
import os
import subprocess
import tarfile
import time
from pathlib import Path

from readiness import Run, digest, write_json, source_inventory, inventory_digest

OPCODES = ("a9", "a2", "c2", "e2", "8d", "ea", "ca")


def boundary_cases():
    """Literal independent vectors for guaranteed boundaries absent in random corpus."""
    cases = []
    def add(name, opcode, pc, p, registers, changes, operand=(), writes=()):
        initial = dict(pc=pc, pbr=0x80, s=0x1ff, p=p, a=0xabcd, x=0x1234,
                       y=0x5678, dbr=0xff, d=0, e=0)
        initial.update(registers)
        addresses = [(0x800000 | ((pc + n) & 0xffff), byte) for n, byte in enumerate((opcode,) + operand)]
        ram = dict(addresses)
        for address, _ in writes:
            ram[address] = 0
        initial["ram"] = sorted(ram.items())
        final = dict(initial, pc=(pc + len(operand) + 1) & 0xffff, **changes)
        ram.update(writes)
        final["ram"] = sorted(ram.items())
        cycles = [[address, byte, "dp-r---"] for address, byte in addresses]
        if opcode in (0xea, 0xca):
            cycles.append([0x800000 | final["pc"], None, "---r---"])
        if opcode in (0xc2, 0xe2):
            cycles.append([addresses[-1][0], None, "---r---"])
        cycles.extend([[address, byte, "d--w---"] for address, byte in writes])
        cases.append(dict(name=name, initial=initial, final=final, cycles=cycles))
    add("boundary LDA8 preserves A high across FFFF",0xa9,0xffff,0x30,{"x":0,"y":0},{"a":0xab80,"p":0xb0},(0x80,))
    add("boundary LDA16 wraps operand bank at FFFE",0xa9,0xfffe,0,{},{"a":0x8000,"p":0x80},(0,0x80))
    add("boundary LDX16 wraps operand bank at FFFE",0xa2,0xfffe,0,{},{"x":0,"p":2},(0,0))
    add("boundary SEP truncates X Y and preserves A",0xe2,0xffff,0,{},{"x":0x34,"y":0x78,"p":0x30},(0x30,))
    add("boundary REP preserves native mode and decimal",0xc2,0xffff,0x38,{"x":0x34,"y":0x78},{"p":8},(0x30,))
    add("boundary STA16 wraps 24 bit data address",0x8d,0xfffe,0,{},{},(0xff,0xff),((0xffffff,0xcd),(0,0xab)))
    add("boundary NOP wraps PC and preserves decimal",0xea,0xffff,8,{},{})
    for p,x,want,flags in [(0,0,0xffff,0x80),(0,1,0,2),(0x10,0,0xff,0x90),(0x10,1,0,0x12)]:
        add("boundary DEX zero sign wrap P{:02X} X{}".format(p,x),0xca,0xffff,p,{"x":x,"y":0},{"x":want,"p":flags})
    return cases


def command(argv, root, output, name, env=None):
    started = time.monotonic()
    with (output / (name + ".stdout.log")).open("wb") as stdout, (output / (name + ".stderr.log")).open("wb") as stderr:
        result = subprocess.run([str(item) for item in argv], cwd=root, env=env, stdout=stdout, stderr=stderr)
    write_json(output / (name + ".json"), {"command": [str(item) for item in argv], "returncode": result.returncode,
                "elapsed_seconds": time.monotonic() - started})
    return result.returncode


def run(corpus, output, opcodes=OPCODES, batch_size=500, timing=False):
    root = Path(__file__).resolve().parent.parent
    output.mkdir(parents=True, exist_ok=False)
    started = time.monotonic()
    report = {"schema_version": 1, "status": "incomplete", "expected": 0, "run": 0,
              "passed": 0, "failed": 0, "skipped": 0, "opcodes": {}, "corpus": [],
              "scope": "complete declared native opcodes" if tuple(opcodes) == OPCODES else "development selected opcodes",
              "expectation_source": "SingleStepTests/65816 native JSON; independent literal boundary fixtures",
              "unsupported_debt": "ADC overflow remains unsupported; docs/SETUP.md and workflows/recomp-iteration.md",
              "conditions": {"native_mx": ["m0x0","m0x1","m1x0","m1x1"], "decimal_flag": [False,True],
                             "preserved_status": "all nonaffected flags", "addressing": ["immediate","absolute","implied"],
                             "inapplicable": ["decimal ALU","branch/direct-page/indexed penalties","emulation compiled execution"]}}
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
    cases = []
    for opcode in opcodes:
        path = corpus / (opcode + ".n.json")
        selected = json.loads(path.read_text())
        report["corpus"].append({"opcode": opcode, "sha256": digest(path), "cases": len(selected)})
        report["opcodes"][opcode] = {"expected": len(selected), "run": 0, "passed": 0, "failed": 0, "skipped": 0}
        for case in selected:
            case["case_sha256"] = hashlib.sha256(json.dumps(case, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
        cases.extend(selected)
    report["expected"] = len(cases)
    if tuple(opcodes) == OPCODES:
        additions = boundary_cases()
        for case in additions:
            case["case_sha256"] = hashlib.sha256(json.dumps(case, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
            pc = 0x800000 | case["initial"]["pc"]
            opcode = "{:02x}".format(dict(case["initial"]["ram"])[pc])
            report["opcodes"][opcode]["expected"] += 1
        cases.extend(additions)
        report["boundary_fixtures"] = len(additions)
        report["expected"] = len(cases)
    with (output/"expected-cases.jsonl").open("w") as inventory:
        for case in cases:
            inventory.write(json.dumps({"name":case["name"],"case_sha256":case["case_sha256"]},sort_keys=True)+"\n")
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
    with (output / "cases.jsonl").open("w") as combined:
        for number, index in enumerate(range(0, len(cases), batch_size)):
            if source_inventory(root) != sources or any(digest(corpus/(entry["opcode"]+".n.json")) != entry["sha256"] for entry in report["corpus"]):
                raise RuntimeError("Source or corpus changed during validation")
            batch = output / ("batch-{:04d}".format(number))
            batch.mkdir()
            input_path = batch / "input.json"
            write_json(input_path, cases[index:index + batch_size])
            bodies, prepared, ledger = batch / "bodies.rs", batch / "prepared.json", batch / "cases.jsonl"
            if command([target / "debug/timed-gen", input_path, bodies, prepared], root, batch, "generate", environment):
                report["status"] = "error"
                break
            environment["SM_TIMED_STEPS"] = str(bodies)
            if command(["cargo", "build", "-p", "fuzz", "--bin", "timed-run", "--locked", "--offline"], root, batch, "build", environment):
                report["status"] = "error"
                break
            code = command([target / "debug/timed-run", prepared, ledger], root, batch, "execute", environment)
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
            write_json(batch / "identities.json", {"bodies_sha256": digest(bodies), "prepared_sha256": digest(prepared),
                "binary_sha256": digest(target / "debug/timed-run")})
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
        code = command(["cargo", "test", "-p", "snes", "--test", "integration", "--test", "irq_mask_timing",
                        "--test", "interrupt_timing", "--test", "dma_timing", "--test", "hdma_timing",
                        "--test", "generated_bus_timing",
                        "--locked", "--offline"], root, output, "timing", environment)
        timing_report={"schema_version": 1, "status": "pass" if code == 0 else "error",
            "complete": code == 0, "returncode": code, "elapsed_seconds": time.monotonic()-timing_started,
            "oracles": {name: digest(root/name) for name in ("crates/snes/tests/hdma_oracle.csv", "crates/snes/tests/irq_mask_timing.rs", "crates/snes/tests/interrupt_timing.rs", "crates/snes/tests/dma_timing.rs")},
            "sources": ["https://wiki.superfamicom.org/timing", "Original rr2-beta23 / bsnes085Compatibility black-box fixtures"],
            "conditions": {"applicable": ["native M/X, D preservation", "6/8/12 access clocks", "program/data bank wrap", "IRQ old-I and NMI phase", "DMA/HDMA pending pause", "refresh", "frame/input rollover and resume"],
                           "inapplicable": ["decimal ALU: no enabled ALU opcode", "branches/direct-page/indexed penalties: no enabled mode", "emulation mode: compiled domain is native"]}}
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
    parser.add_argument("--opcodes", default=",".join(OPCODES))
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
        result = run(args.corpus.resolve(), output, tuple(args.opcodes.split(",")), args.batch_size,args.timing)
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
