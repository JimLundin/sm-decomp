"""Exercise the public pinned C build/replay evidence command (requires local ROM)."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
INPUT_ROOT = ROOT


class BaselineCommandTests(unittest.TestCase):
    def test_missing_movie_produces_error_evidence_without_replay_acceptance(self):
        with tempfile.TemporaryDirectory() as output:
            result = subprocess.run(
                [sys.executable, str(ROOT / 'tools/decomp_baseline.py'), '--inputs', str(Path(output) / 'missing.inputs'),
                 '--rom', str(INPUT_ROOT / 'rom/sm.sfc'), '--output', output],
                capture_output=True, text=True, timeout=150)
            self.assertNotEqual(result.returncode, 0)
            verdict = json.loads((Path(json.loads(result.stdout)['evidence']) / 'verdict.json').read_text())
            self.assertEqual(verdict['status'], 'error')
            self.assertIn('missing.inputs', verdict['error'])
            self.assertNotIn('terminal', verdict)

    def test_replay_preserves_structured_calls_and_exercises_observation_controls(self):
        with tempfile.TemporaryDirectory() as output:
            result = subprocess.run(
                [sys.executable, str(ROOT / 'tools/decomp_baseline.py'), '--frames', '1',
                 '--inputs', str(INPUT_ROOT / 'movies/super-metroid-any-3653.inputs'),
                 '--rom', str(INPUT_ROOT / 'rom/sm.sfc'), '--output', output],
                capture_output=True, text=True, timeout=150)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            run = Path(json.loads(result.stdout)['evidence'])
            verdict = json.loads((run / 'verdict.json').read_text())
            self.assertEqual(verdict['observation_controls']['fallback_calls'], 1)
            self.assertGreater(verdict['observation_controls']['fallback_instructions'], 0)
            self.assertEqual(verdict['observation_controls']['decompression_calls'], 1)
            calls = json.loads((run / 'replay/fixtures/manifest.json').read_text())['calls']
            self.assertTrue(calls)
            for sequence, call in enumerate(calls, 1):
                self.assertEqual(call['sequence'], sequence)
                self.assertTrue(0x8000 <= (call['source_address'] & 0xffff))
                self.assertTrue(0 <= call['destination_offset'] < 0x20000)
                rom_view = run / 'replay/fixtures' / call['rom_view_file']
                self.assertEqual(hashlib.sha256(rom_view.read_bytes()).hexdigest(), call['rom_view_sha256'])
                for phase in ('before', 'after'):
                    self.assertEqual((run / 'replay/fixtures' / call[phase]['file']).stat().st_size, 0x20000)
                    self.assertEqual(len(call[phase]['sha256']), 64)
            self.assertGreater(verdict['in_memory_rom_patches']['changed_bytes'], 0)
            self.assertEqual(verdict['original_rom_parity'], 'not-tested')


if __name__ == '__main__':
    unittest.main()
