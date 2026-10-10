"""Changed-key coverage at the confirmed immutable run evidence seam."""
import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from readiness import Run


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.name == "coverage.json":
        run = path.parent.parent
        value.update(complete=True, expected_frames=2, completed_frames=2,
                     candidate_inventory_sha256=hashlib.sha256((run / "generated/inventory.json").read_bytes()).hexdigest())
        if not value["interpreted"]:
            value["interpreted"] = [{"address": 0x8003, "mx": 3, "opcode": 0x80, "count": value["interpreted_instructions"]}]
    path.write_text(json.dumps(value))


def inventory(path, rows):
    run = path.parent.parent
    manifest = json.loads((run / "manifest.json").read_text())
    steps = path.with_name("steps.rs")
    bodies = {}
    for row in rows:
        body = "changed" if row["body_sha256"] == hashlib.sha256(b"\nchanged").hexdigest() else "body"
        bodies[(row["address"], row["mx"])] = body
    source = "".join("fn step_{:06x}_{}(m: &mut Machine) {{\n{}\n}}\n".format(address, mx, body)
                     for (address, mx), body in bodies.items())
    source += "pub static STEP_TABLE: &[StepEntry] = &[\n" + "".join(
        "(0x{:06x}, {}, &[234], step_{:06x}_{}),\n".format(row["address"], row["mx"], row["address"], row["mx"])
        for row in rows) + "];\n"
    steps.write_text(source)
    write(path, {"schema_version": 1, "rom_sha256": "rom", "config_sha256": "config", "keys": rows,
                 "source_sha256": manifest["identities"]["source"]["sha256"],
                 "steps_evidence": steps.relative_to(run).as_posix(),
                 "steps_sha256": hashlib.sha256(steps.read_bytes()).hexdigest()})
    manifest["identities"]["generated-key-inventory"] = {"evidence": path.relative_to(run).as_posix(),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
    write(run / "manifest.json", manifest)


def key(address, body="body"):
    return {"address": address, "mx": 3, "opcode": 0xea, "bytes": [0xea], "body_sha256": hashlib.sha256(("\n" + body).encode()).hexdigest()}


class CoverageTests(unittest.TestCase):
    def test_added_unexecuted_key_is_uncovered_despite_old_execution(self):
        from readiness_coverage import evaluate_coverage
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            parent = Run.create(root, "parity", {"frames": 2})
            parent.capture_source()
            old = parent.path / "generated/inventory.json"
            inventory(old, [key(0x8002)])
            identity = parent.record_artifact("generated-key-inventory", old)
            identity["evidence"] = old.relative_to(parent.path).as_posix()
            parent.save_manifest()
            parent.finalize("pass")
            candidate = Run.create(root, "parity", {"frames": 2}, parent.manifest["run_id"])
            candidate.capture_source()
            current = candidate.path / "generated/inventory.json"
            inventory(current, [key(0x8002), key(0x8020)])
            candidate.record_artifact("generated-key-inventory", current)
            profile = candidate.path / "diagnostics/coverage.json"
            write(profile, {"schema_version": 1, "compiled": [dict(key(0x8002), count=40), dict(key(0x8020), count=0)],
                            "interpreted": [], "generated_instructions": 40, "interpreted_instructions": 10})
            report = evaluate_coverage(candidate, profile, current, parent.path)
            self.assertEqual(report["status"], "uncovered")
            self.assertEqual(report["required_keys"], [{"address": 0x8020, "mx": 3, "reason": "added"}])
            self.assertEqual(report["uncovered_keys"], [{"address": 0x8020, "mx": 3}])

    def test_modified_body_requires_affected_key_execution(self):
        from readiness_coverage import evaluate_coverage
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            parent = Run.create(root, "parity", {"frames": 2})
            parent.capture_source()
            old = parent.path / "generated/inventory.json"
            inventory(old, [key(0x8002)])
            identity = parent.record_artifact("generated-key-inventory", old)
            identity["evidence"] = old.relative_to(parent.path).as_posix()
            parent.save_manifest()
            parent.finalize("pass")
            candidate = Run.create(root, "parity", {"frames": 2}, parent.manifest["run_id"])
            candidate.capture_source()
            current = candidate.path / "generated/inventory.json"
            inventory(current, [key(0x8002, "changed")])
            candidate.record_artifact("generated-key-inventory", current)
            profile = candidate.path / "diagnostics/coverage.json"
            write(profile, {"schema_version": 1, "compiled": [dict(key(0x8002), count=0)],
                            "interpreted": [], "generated_instructions": 0, "interpreted_instructions": 10})
            report = evaluate_coverage(candidate, profile, current, parent.path)
            self.assertEqual(report["status"], "uncovered")
            self.assertEqual(report["required_keys"][0]["reason"], "body changed")

    def test_pattern_promotion_preserves_unaffected_legacy_debt_but_runtime_change_requires_it(self):
        from readiness_coverage import generated_inventory,evaluate_coverage
        from pattern_contract import DECLARATION
        for changed in ('promotion','pattern','runtime'):
            with self.subTest(changed=changed),tempfile.TemporaryDirectory() as temporary:
                root=Path(temporary);subprocess.run(['git','init','-q',str(root)],check=True)
                declaration=json.loads((Path(__file__).resolve().parent.parent/DECLARATION).read_text())
                path=root/DECLARATION;path.parent.mkdir(parents=True);path.write_text(json.dumps(declaration))
                runtime=root/'crates/runtime/src/lib.rs';runtime.parent.mkdir(parents=True);runtime.write_text('unchanged shared runtime')
                (root/'config').mkdir();(root/'config/functions.toml').write_text('config')
                def table(run,extra=False):
                    rows=[(0x8002,0xea)]+([(0x8020,0xad)] if extra else [])
                    text='pub const ROM_SHA256: Option<&str> = Some("'+'a'*64+'");\n'
                    text+=''.join('fn step_{:06x}_3(m: &mut Machine) {{\nbody\n}}\n'.format(addr) for addr,_ in rows)
                    text+='pub static STEP_TABLE: &[StepEntry] = &[\n'+''.join('(0x{:06x}, 3, &[{}], step_{:06x}_3),\n'.format(addr,op,addr) for addr,op in rows)+'];\n'
                    steps=run.path/'generated/steps.rs';steps.write_text(text)
                    return generated_inventory(run,steps)
                parent=Run.create(root,'parity',{'frames':2});parent.capture_source();old=table(parent)
                # Historical inventory retains its original global binding.
                prior=json.loads(old.read_text());prior.pop('execution_binding_version',None);prior.pop('shared_execution_source_sha256',None);prior.pop('declaration_pattern_sha256',None);old.write_text(json.dumps(prior))
                parent.manifest['identities'].pop('generated-key-inventory');identity=parent.record_artifact('generated-key-inventory',old);identity['evidence']=old.relative_to(parent.path).as_posix();parent.save_manifest();parent.finalize('pass')
                if changed=='runtime':runtime.write_text('changed shared runtime')
                else:
                    promoted=declaration['prospective_patterns'].pop(0);declaration['patterns'].append(promoted)
                    if changed=='pattern':next(row for row in declaration['patterns'] if row['opcode']=='ea')['addressing']='changed semantic declaration'
                    path.write_text(json.dumps(declaration))
                candidate=Run.create(root,'parity',{'frames':2},parent.manifest['run_id']);candidate.capture_source();current=table(candidate,True)
                profile=candidate.path/'diagnostics/profile.json'
                write(profile,{'schema_version':1,'complete':True,'expected_frames':2,'completed_frames':2,'candidate_inventory_sha256':hashlib.sha256(current.read_bytes()).hexdigest(),
                    'compiled':[dict(key(0x8002),count=0),dict(key(0x8020),opcode=0xad,bytes=[0xad],count=10)],'interpreted':[], 'generated_instructions':10,'interpreted_instructions':0})
                report=evaluate_coverage(candidate,profile,current,parent.path)
                self.assertEqual(report['status'],'pass' if changed=='promotion' else 'uncovered')
                self.assertEqual(any(row['address']==0x8002 for row in report['required_keys']),changed!='promotion')

    def test_progress_delta_is_absent_when_replay_intervals_differ(self):
        from readiness_coverage import comparable_progress
        parent = {"domain": {"rom": "same", "movie": "same", "profile": "same", "seed": 0, "start": 0, "frames": 300},
                  "generated_instructions": 400, "interpreted_instructions": 1000}
        candidate = dict(parent, domain=dict(parent["domain"], frames=200), generated_instructions=500)
        self.assertFalse(comparable_progress(candidate, parent)["comparable"])
        self.assertNotIn("generated_delta", comparable_progress(candidate, parent))

    def test_initial_legacy_unexecuted_keys_are_inventory_not_progress(self):
        from readiness_coverage import evaluate_coverage
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            run = Run.create(root, "parity", {"frames": 2})
            run.capture_source()
            current = run.path / "generated/inventory.json"
            inventory(current, [key(0x8002), key(0x8020)])
            run.record_artifact("generated-key-inventory", current)
            profile = run.path / "diagnostics/coverage.json"
            write(profile, {"schema_version": 1, "compiled": [dict(key(0x8002), count=40), dict(key(0x8020), count=0)],
                            "interpreted": [], "generated_instructions": 40, "interpreted_instructions": 10})
            report = evaluate_coverage(run, profile, current)
            self.assertEqual(report["status"], "pass")
            self.assertEqual(report["validated_distinct_keys"], 1)
            self.assertEqual(report["unexecuted"][0]["validation"], "unvalidated baseline")
            self.assertEqual(report["required_keys"], [])

    def test_preserved_predecessor_corruption_blocks_changed_key_evidence(self):
        from readiness_coverage import evaluate_coverage
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            parent = Run.create(root, "parity", {"frames": 2})
            parent.capture_source()
            old = parent.path / "generated/inventory.json"
            inventory(old, [key(0x8002)])
            identity = parent.record_artifact("generated-key-inventory", old)
            identity["evidence"] = old.relative_to(parent.path).as_posix()
            parent.save_manifest()
            parent.finalize("pass")
            old.write_text("corrupted inventory")
            candidate = Run.create(root, "parity", {"frames": 2}, parent.manifest["run_id"])
            candidate.capture_source()
            current = candidate.path / "generated/inventory.json"
            inventory(current, [key(0x8002)])
            candidate.record_artifact("generated-key-inventory", current)
            profile = candidate.path / "diagnostics/coverage.json"
            write(profile, {"schema_version": 1, "compiled": [dict(key(0x8002), count=40)],
                            "interpreted": [], "generated_instructions": 40, "interpreted_instructions": 10})
            with self.assertRaisesRegex(RuntimeError, "inventory.*corrupt"):
                evaluate_coverage(candidate, profile, current, parent.path)
