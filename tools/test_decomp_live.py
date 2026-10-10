"""Public live verdict rejects missing offline acceptance before compiling."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

class LiveJob(unittest.TestCase):
    def test_missing_acceptance_cannot_start_replay(self):
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(['python3', 'tools/decomp_live.py', '--accepted',
                directory + '/absent.json', '--output', directory], capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            verdict = json.loads(next(Path(directory).glob('*/verdict.json')).read_text())
            self.assertEqual(verdict['status'], 'error')
            self.assertEqual(verdict['stages'], [])
            self.assertFalse(verdict['mixed_integration'])

    @unittest.skipUnless(__import__('os').environ.get('DECOMP_ACCEPTED'), 'set DECOMP_ACCEPTED for the sealed live replay')
    def test_live_replay_matches_and_errors_stop_without_fallback(self):
        import os
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(['python3', 'tools/decomp_live.py', '--accepted',
                os.environ['DECOMP_ACCEPTED'], '--output', directory], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            verdict = json.loads(next(Path(directory).glob('*/verdict.json')).read_text())
            self.assertTrue(verdict['mixed_integration'])
            self.assertEqual(verdict['status'], 'mixed-C-Rust-integration')
            self.assertEqual(verdict['original_rom_parity'], 'not-tested')
            self.assertEqual(verdict['error_control'], {'exit': 4, 'rust_hits': 0, 'c_hits': 0, 'calls': 1})
            for row in verdict['subjects']['mixed-0']:
                self.assertEqual(row['rust_hits'], 23)
                self.assertEqual(row['c_hits'], 0)
