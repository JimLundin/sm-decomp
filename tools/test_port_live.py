"""Live port contract controls through the public command."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class LivePortTests(unittest.TestCase):
    def test_missing_family_evidence_cannot_start_replay(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'output'
            result = subprocess.run([
                'python3', str(ROOT / 'tools/port_live.py'), '--family-verdict',
                str(Path(temporary) / 'missing.json'), '--output', str(output),
            ], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            verdicts = list(output.glob('*/verdict.json'))
            self.assertEqual(len(verdicts), 1)
            verdict = json.loads(verdicts[0].read_text())
            self.assertEqual(verdict['status'], 'rejected')
            self.assertFalse(verdict['live_exercised'])
            self.assertIn('error', verdict)


if __name__ == '__main__':
    unittest.main()
