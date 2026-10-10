"""Public original-boundary capture and independent prefix replay command."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
ASSETS = Path('/home/jim/sm-decomp') if not (ROOT / 'deps').exists() else ROOT

class PrefixCommandTests(unittest.TestCase):
    def test_missing_movie_cannot_accept_a_prefix(self):
        with tempfile.TemporaryDirectory() as output:
            result = subprocess.run([sys.executable, str(ROOT / 'tools/decomp_prefix.py'),
                '--asset-root', str(ASSETS), '--movie', str(Path(output) / 'absent.lsmv'),
                '--output', output], capture_output=True, text=True, timeout=150)
            self.assertNotEqual(result.returncode, 0)
            verdict = json.loads((Path(json.loads(result.stdout)['evidence']) / 'verdict.json').read_text())
            self.assertEqual(verdict['status'], 'error')
            self.assertFalse(verdict['accepted'])

    def test_native_edges_are_retained_and_behavior_change_is_rejected(self):
        with tempfile.TemporaryDirectory() as output:
            result = subprocess.run([sys.executable, str(ROOT / 'tools/decomp_prefix.py'),
                '--asset-root', str(ASSETS), '--output', output],
                capture_output=True, text=True, timeout=240)
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            run = Path(json.loads(result.stdout)['evidence'])
            verdict = json.loads((run / 'verdict.json').read_text())
            self.assertEqual(verdict['status'], 'nonaccepting')
            self.assertFalse(verdict['accepted'])
            self.assertEqual(verdict['first_unsupported']['kind'], 'prelude-controller-poll')
            self.assertEqual(verdict['first_unsupported']['source_frame'], 19)
            self.assertTrue(verdict['changed_gameplay_control']['detected'])
            timeline = json.loads((run / 'timeline.json').read_text())
            polls = [row for row in timeline['events'] if row['kind'] == 'controller-return']
            self.assertGreater(len(polls), 100)
            self.assertEqual([row['held'] for row in polls[:3]], [0x1000, 0x80, 0x1000])
            self.assertTrue(all('source_event' in row and 'classification' in row for row in timeline['events']))
            self.assertTrue(verdict['comparisons'])
            self.assertTrue(verdict['native']['completed_frames'] >= 320)
            self.assertEqual(verdict['candidate']['initializations'], 1)
            self.assertTrue((run / 'native/events.csv').exists())
            self.assertTrue((run / 'candidate/terminal.bin').exists())

if __name__ == '__main__':
    unittest.main()
