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
            violation=verdict['future_poll_assignments'][0]
            self.assertEqual(violation['completion_frame'],272)
            self.assertGreater(violation['assigned_poll_event'],violation['completion_event'])
            ledger=json.loads((run/'controller-causality.json').read_text())
            complete=next(row for row in ledger['events'] if row['kind']=='dispatch-complete')
            self.assertIsNone(complete['preceding_poll'])
            self.assertTrue(verdict['native']['completed_frames'] >= 320)
            self.assertEqual(verdict['candidate']['initializations'], 1)
            self.assertTrue((run / 'native/events.csv').exists())
            self.assertTrue((run / 'candidate/terminal.bin').exists())


class ControllerCausalityTests(unittest.TestCase):
    def test_completion_never_consumes_a_later_poll_and_prelude_remains(self):
        sys.path.insert(0, str(ROOT / 'tools'))
        from decomp_prefix import controller_causality
        def event(number, kind, held=0):
            return {'source_event': number, 'source_frame': 272, 'kind': kind, 'state': {'held': held}}
        events = [event(23, 'controller-return', 0x1000), event(1005, 'boot-clear'),
                  event(1024, 'dispatch-entry'), event(1025, 'dispatch-complete'),
                  event(1030, 'controller-return', 0x80), event(1031, 'dispatch-entry', 0x80),
                  event(1068, 'dispatch-complete', 0x80)]
        rows = controller_causality(events)
        self.assertEqual(rows[0]['source_event'], 23)
        self.assertEqual(rows[0]['phase'], 'prelude')
        self.assertEqual(rows[3]['preceding_poll'], None)
        self.assertEqual(rows[-1]['preceding_poll'], 1030)
        self.assertEqual(rows[1]['cleared_poll'], 23)

if __name__ == '__main__':
    unittest.main()
