"""Command seam: actual decoded/emitted/compiled step against independent vector."""
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class PatternCommandTests(unittest.TestCase):
    def test_production_declaration_cannot_omit_an_enabled_pattern(self):
        root = Path(__file__).resolve().parent.parent
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            declaration = directory / "reduced.toml"
            declaration.write_text('schema_version = 2\npatterns = []\n')
            result = subprocess.run([sys.executable, str(root / "tools/readiness_patterns.py"),
                "--declaration", str(declaration), "--out", str(directory / "evidence")],
                cwd=root, capture_output=True, text=True)
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            report = json.loads((directory / "evidence/instruction.json").read_text())
            self.assertIn("declaration", report["reason"])

    def test_real_step_runner_keeps_case_ledger_and_rejects_incorrect_expectation(self):
        root = Path(__file__).resolve().parent.parent
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            corpus = directory / "corpus"
            corpus.mkdir()
            initial = {"pc": 0xffff, "pbr": 0x80, "s": 0x1ff, "p": 0x30, "a": 0xab00,
                       "x": 0, "y": 0, "dbr": 0, "d": 0, "e": 0,
                       "ram": [[0x80ffff, 0xa9], [0x800000, 0x80]]}
            final = dict(initial, pc=1, a=0xab80, p=0xb0)
            case = {"name": "independent LDA wraps PC and preserves high A", "initial": initial,
                    "final": final, "cycles": [[0x80ffff, 0xa9, "dp-r-mx-"],
                                                 [0x800000, 0x80, "-p-r-mx-"]]}
            (corpus / "a9.n.json").write_text(json.dumps([case]))
            command = [sys.executable, str(root / "tools/readiness_patterns.py"),
                       "--corpus", str(corpus), "--opcodes", "a9", "--out", str(directory / "good")]
            result = subprocess.run(command, cwd=root, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr + json.loads((directory / "good/instruction.json").read_text()).get("reason", ""))
            report = json.loads((directory / "good/instruction.json").read_text())
            self.assertEqual((report["expected"], report["passed"], report["skipped"]), (3, 3, 0))
            rows = [json.loads(line) for line in (directory / "good/cases.jsonl").read_text().splitlines()]
            self.assertEqual(rows[0]["name"], case["name"])
            self.assertEqual(rows[0]["status"], "pass")
            identities=json.loads((directory/"good/batch-0000/identities.json").read_text())
            executable=directory/"good/batch-0000"/identities["binary"]
            self.assertTrue(executable.is_file())
            from readiness import digest
            self.assertEqual(digest(executable),identities["binary_sha256"])
            execution=json.loads((directory/"good/batch-0000/execute.json").read_text())
            self.assertEqual(Path(execution["command"][0]),executable)

            manifest={"identities":{"source":{"sha256":json.loads((directory/"good/instruction.json").read_text())["source_sha256"]},"compiler":{"version":json.loads((directory/"good/instruction.json").read_text())["compiler"]}}}
            (directory/"manifest.json").write_text(json.dumps(manifest))
            verify=[sys.executable,str(root/"tools/readiness_orchestration.py"),"verify-patterns","--run",str(directory),"--output",str(directory/"good"),"--development-scope"]
            result=subprocess.run(verify,cwd=root,capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr)
            strict = [argument for argument in verify if argument != "--development-scope"]
            result = subprocess.run(strict, cwd=root, capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertIn("Development", result.stderr)
            # A report cannot remove a width/branch or substitute its declaration.
            report_path = directory / "good/instruction.json"
            original = report_path.read_text()
            altered = json.loads(original)
            altered["declaration"]["patterns"][0]["mx"] = [3]
            report_path.write_text(json.dumps(altered))
            result = subprocess.run(verify, cwd=root, capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertIn("declaration", result.stderr)
            report_path.write_text(original)
            executable.write_bytes(executable.read_bytes()+b"corrupt")
            result=subprocess.run(verify,cwd=root,capture_output=True,text=True)
            self.assertEqual(result.returncode,2)
            final["a"] = 0xab81
            (corpus / "a9.n.json").write_text(json.dumps([case]))
            command[-1] = str(directory / "bad")
            result = subprocess.run(command, cwd=root, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            report = json.loads((directory / "bad/instruction.json").read_text())
            self.assertEqual(report["failed"], 1)


if __name__ == "__main__":
    unittest.main()
