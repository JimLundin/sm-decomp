"""Exercise the public offline job against independent whole-WRAM expectations."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]

class LiteralJob(unittest.TestCase):
    def test_literal_and_terminator_are_partial_not_routine_acceptance(self):
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(['python3', 'tools/decomp_routine.py',
                                     '--output', directory], capture_output=True, text=True, cwd=ROOT)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            verdict = json.loads(next(Path(directory).glob('*/verdict.json')).read_text())
            self.assertEqual(verdict['status'], 'partial')
            self.assertFalse(verdict['routine_accepted'])
            self.assertEqual(verdict['observed_cases'], ['literal-short', 'terminator'])
            self.assertEqual([f['status'] for f in verdict['fixtures']], ['matched', 'matched'])
            for fixture in verdict['fixtures']:
                self.assertEqual(fixture['c_sha256'], fixture['expected_sha256'])
                self.assertEqual(fixture['rust_sha256'], fixture['expected_sha256'])

    def test_invalid_fixture_never_enters_the_unsafe_c_oracle(self):
        with tempfile.TemporaryDirectory() as directory:
            manifest = json.loads((ROOT / 'tools/decomp/decompress-job.json').read_text())
            manifest['fixtures'][0]['destination'] = 0x20000 - 1
            path = Path(directory) / 'invalid.json'
            path.write_text(json.dumps(manifest))
            result = subprocess.run(['python3', 'tools/decomp_routine.py',
                                     '--manifest', str(path), '--output', directory], capture_output=True, text=True, cwd=ROOT)
            self.assertEqual(result.returncode, 1)
            verdict = json.loads(next(Path(directory).glob('*/verdict.json')).read_text())
            self.assertEqual(verdict['status'], 'error')
            self.assertEqual(verdict['stages'], [])
            self.assertIn('fixture destination', verdict['error'])

if __name__ == '__main__':
    unittest.main()
