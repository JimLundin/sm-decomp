"""Native allocator compared against actual C, including amount context."""

import json
import tempfile
import unittest
from pathlib import Path

from port_family import ROOT, digest
from port_plm import verify, verify_plm_evidence

SOURCE = Path("/home/jim/sm-decomp/deps/snesrev-sm")
ROM = Path("/home/jim/sm-decomp/rom/sm.sfc")


class NativePlmTests(unittest.TestCase):
    def test_real_quicksand_spawn_and_wrong_amount_and_retained_tamper(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            module = tmp / "candidate.rs"
            original = (ROOT / "crates/port/src/plm.rs").read_text()
            module.write_text(original)
            config = {
                "family": "native-test",
                "rust_module": str(module),
                "rom": {"file": str(ROM), "sha256": digest(ROM)},
                "cases": [
                    {
                        "name": "quicksand",
                        "a": 0xB72B,
                        "b": 0x10000,
                        "writes": [[0xB02, [3, 0]]],
                        "expected_flags": 0,
                        "expected_amount": 0x3000,
                    }
                ],
            }
            manifest = tmp / "manifest.json"
            manifest.write_text(json.dumps(config))
            v = verify(manifest, SOURCE, tmp / "good")
            self.assertEqual(v["status"], "offline-verified", v.get("error"))
            path = Path(v["run"]) / "verdict.json"
            verify_plm_evidence(path)
            original_verdict = path.read_text()
            missing = json.loads(original_verdict)
            del missing["artifacts"]["quicksand/rust.stdout"]
            path.write_text(json.dumps(missing))
            with self.assertRaisesRegex(ValueError, "case artifacts"):
                verify_plm_evidence(path)
            changed_pin = json.loads(original_verdict)
            changed_pin["source"]["revision"] = "wrong"
            path.write_text(json.dumps(changed_pin))
            with self.assertRaisesRegex(ValueError, "pinned source"):
                verify_plm_evidence(path)
            path.write_text(original_verdict)
            output = path.parent / "quicksand" / "rust-after.wram"
            output.write_bytes(bytes(0x20000))
            with self.assertRaisesRegex(ValueError, "artifact"):
                verify_plm_evidence(path)
            module.write_text(
                original.replace(
                    "Ok(flags)\n}", "*amount=amount.wrapping_add(1);Ok(flags)\n}"
                )
            )
            self.assertNotEqual(module.read_text(), original)
            v = verify(manifest, SOURCE, tmp / "bad")
            self.assertEqual(v["status"], "rejected")
            self.assertIn("amount", v["error"])
