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
    path.write_text(json.dumps(value))


def inventory(path, rows):
    write(path, {"schema_version": 1, "rom_sha256": "rom", "config_sha256": "config", "keys": rows})


def key(address, body="body"):
    return {"address": address, "mx": 3, "opcode": 0xea, "bytes": [0xea], "body_sha256": body}


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
            parent.record_artifact("generated-key-inventory", old)
            candidate = Run.create(root, "parity", {"frames": 2}, parent.manifest["run_id"])
            candidate.capture_source()
            current = candidate.path / "generated/inventory.json"
            inventory(current, [key(0x8002), key(0x8020)])
            profile = candidate.path / "diagnostics/coverage.json"
            write(profile, {"schema_version": 1, "compiled": [dict(key(0x8002), count=40), dict(key(0x8020), count=0)],
                            "interpreted": [], "generated_instructions": 40, "interpreted_instructions": 10})
            report = evaluate_coverage(candidate, profile, current, parent.path)
            self.assertEqual(report["status"], "uncovered")
            self.assertEqual(report["required_keys"], [{"address": 0x8020, "mx": 3, "reason": "added"}])
            self.assertEqual(report["uncovered_keys"], [{"address": 0x8020, "mx": 3}])
