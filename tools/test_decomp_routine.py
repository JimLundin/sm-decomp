"""Exercise the public offline job against independent whole-WRAM expectations."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]

class SyntheticJob(unittest.TestCase):
    def test_all_synthetic_commands_are_verified_without_routine_acceptance(self):
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(['python3', 'tools/decomp_routine.py',
                                     '--output', directory], capture_output=True, text=True, cwd=ROOT)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            verdict = json.loads(next(Path(directory).glob('*/verdict.json')).read_text())
            self.assertEqual(verdict['status'], 'partial')
            self.assertFalse(verdict['routine_accepted'])
            manifest = json.loads((ROOT / 'tools/decomp/decompress-job.json').read_text())
            self.assertEqual(verdict['observed_cases'], sorted(manifest['required_cases']))
            self.assertTrue(verdict['synthetic_verified'])
            self.assertEqual(verdict['missing_cases'], [])
            self.assertTrue(all(f['status'] == 'matched' for f in verdict['fixtures']))
            for fixture in verdict['fixtures']:
                self.assertEqual(fixture['c_ordered_writes_sha256'], fixture['rust_ordered_writes_sha256'])
                self.assertGreaterEqual(fixture['write_count'], 0)
                self.assertEqual(len(fixture['write_set']), fixture['write_count'])
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

    def test_malformed_streams_are_rejected_before_oracle_build(self):
        examples = [([0xe0], 4096), ([0x40, 0x12], 4096),
                    ([0xc0, 1, 255], 0), ([0x80, 1, 0, 255], 0x1ffff),
                    ([0, 0x12], 4096), ([255, 0], 4096)]
        for stream, destination in examples:
            with self.subTest(stream=stream), tempfile.TemporaryDirectory() as directory:
                manifest = json.loads((ROOT / 'tools/decomp/decompress-job.json').read_text())
                manifest['fixtures'][0].update(stream=stream, destination=destination)
                path = Path(directory) / 'invalid.json'
                path.write_text(json.dumps(manifest))
                result = subprocess.run(['python3', 'tools/decomp_routine.py', '--manifest', str(path),
                                         '--output', directory], capture_output=True, text=True, cwd=ROOT)
                self.assertEqual(result.returncode, 1)
                verdict = json.loads(next(Path(directory).glob('*/verdict.json')).read_text())
                self.assertEqual(verdict['stages'], [])
                self.assertFalse(verdict['routine_accepted'])

class CapturedJob(unittest.TestCase):
    def test_missing_capture_preserves_a_failing_verdict(self):
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(['python3', 'tools/decomp_routine.py', '--captures',
                                     str(Path(directory) / 'missing.json'), '--output', directory],
                                    capture_output=True, text=True, cwd=ROOT)
            self.assertEqual(result.returncode, 1)
            verdict = json.loads(next(Path(directory).glob('*/verdict.json')).read_text())
            self.assertEqual(verdict['status'], 'error')
            self.assertIn('FileNotFoundError', verdict['error'])
            self.assertEqual(verdict['stages'], [])

    def test_changed_captured_state_digest_fails_before_oracle_build(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            rom = bytes([0, 42, 255]) + bytes(0x400000 - 3)
            before = bytes(0x20000)
            after = bytearray(before); after[4096] = 42
            for name, data in [('rom.bin', rom), ('before.wram', before), ('after.wram', after)]:
                (base / name).write_bytes(data)
            sha = lambda data: hashlib.sha256(data).hexdigest()
            job = json.loads((ROOT / 'tools/decomp/decompress-job.json').read_text())
            capture = {'contract': 'decomp-c-captures-v1', 'routine': 'DecompressToMem',
                       'routine_address': 0x80b119, 'source_revision': job['source_revision'],
                       'rom_view_file': 'rom.bin', 'rom_view_sha256': sha(rom), 'rom_view_size': len(rom),
                       'calls': [{'sequence': 1, 'fixture_id': 'literal-capture', 'source_address': 0x808000,
                                  'destination_offset': 4096, 'rom_view_file': 'rom.bin', 'rom_view_sha256': sha(rom),
                                  'before': {'file': 'before.wram', 'sha256': sha(before), 'size': len(before)},
                                  'after': {'file': 'after.wram', 'sha256': sha(after), 'size': len(after)}}]}
            captures = base / 'captures.json'; captures.write_text(json.dumps(capture))
            (base / 'after.wram').write_bytes(before)
            result = subprocess.run(['python3', 'tools/decomp_routine.py', '--captures', str(captures),
                                     '--output', directory], capture_output=True, text=True, cwd=ROOT)
            self.assertEqual(result.returncode, 1)
            verdict = json.loads(next(base.glob('*/verdict.json')).read_text())
            self.assertIn('capture artifact identity differs: after.wram', verdict['error'])
            self.assertEqual(verdict['stages'], [])

    def test_real_capture_replay_has_separate_bounded_original_evidence(self):
        captures = Path('/home/jim/sm-decomp/out/decomp/ticket01/20261010T181241.402731Z/replay/fixtures/manifest.json')
        rom = Path('/home/jim/sm-decomp/rom/sm.sfc')
        if not captures.exists() or not rom.exists():
            self.skipTest('local commercial capture and unmodified ROM are unavailable')
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(['python3', 'tools/decomp_routine.py', '--captures', str(captures),
                                     '--original-rom', str(rom), '--output', directory],
                                    capture_output=True, text=True, cwd=ROOT)
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
            verdict = json.loads(next(Path(directory).glob('*/verdict.json')).read_text())
            self.assertFalse(verdict['routine_accepted'])
            self.assertEqual(verdict['captured_calls_verified'], 23)
            self.assertEqual(verdict['original_rom_parity']['status'], 'bounded-routine-matched')
            self.assertEqual(len(verdict['original_rom_parity']['fixtures']), 3)
            for fixture in verdict['fixtures']:
                self.assertEqual(fixture['c_repeat_sha256'], fixture['c_sha256'])
                self.assertEqual(fixture['rust_repeat_sha256'], fixture['rust_sha256'])

    def test_capture_identity_error_is_retained_before_build(self):
        with tempfile.TemporaryDirectory() as directory:
            captures = Path(directory) / 'captures.json'
            captures.write_text(json.dumps({'contract': 'decomp-c-captures-v1',
                                           'source_revision': 'wrong'}))
            result = subprocess.run(['python3', 'tools/decomp_routine.py', '--captures', str(captures),
                                     '--output', directory], capture_output=True, text=True, cwd=ROOT)
            verdicts = list(Path(directory).glob('*/verdict.json'))
            self.assertEqual(len(verdicts), 1, result.stderr)
            verdict = json.loads(verdicts[0].read_text())
            self.assertEqual(result.returncode, 1)
            self.assertIn('capture source identity', verdict['error'])
            self.assertEqual(verdict['stages'], [])

if __name__ == '__main__':
    unittest.main()

class AcceptanceControls(unittest.TestCase):
    def test_transient_store_is_rejected_even_when_final_wram_matches(self):
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(['python3', 'tools/decomp_routine.py', '--fault', 'write-restore',
                                     '--output', directory], capture_output=True, text=True, cwd=ROOT)
            self.assertEqual(result.returncode, 1)
            verdict = json.loads(next(Path(directory).glob('*/verdict.json')).read_text())
            self.assertIn('ordered WRAM stores differ', verdict['error'])
            self.assertEqual(verdict['fixtures'][0]['c_sha256'], verdict['fixtures'][0]['rust_sha256'])
            self.assertFalse(verdict['routine_accepted'])

    def test_exercised_acceptance_seals_coverage_and_rejects_real_controls(self):
        captures = Path('/home/jim/sm-decomp/out/decomp/ticket01/20261010T181241.402731Z/replay/fixtures/manifest.json')
        rom = Path('/home/jim/sm-decomp/rom/sm.sfc')
        if not captures.exists() or not rom.exists(): self.skipTest('local captures unavailable')
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(['python3', 'tools/decomp_routine.py', '--accept', '--captures', str(captures),
                                     '--original-rom', str(rom), '--output', directory], capture_output=True, text=True, cwd=ROOT)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            pointer = json.loads((Path(directory) / 'decomp-acceptance.json').read_text())
            verdict = json.loads((Path(pointer['run']) / 'verdict.json').read_text())
            self.assertTrue(verdict['routine_accepted'])
            from tools.decomp_routine import verify_accepted
            self.assertEqual(len(verify_accepted(Path(directory) / 'decomp-acceptance.json')['fixtures']), 63)
            self.assertEqual(verdict['status'], 'accepted')
            self.assertTrue(all(control['rejected'] for control in verdict['negative_controls']))
            self.assertEqual(len(verdict['negative_controls']), 11)
            self.assertEqual(len(verdict['fixtures']), 63)
            failure = subprocess.run(['python3', 'tools/decomp_routine.py', '--fault', 'crash', '--output', directory], capture_output=True, text=True, cwd=ROOT)
            self.assertEqual(failure.returncode, 1)
            self.assertEqual(json.loads((Path(directory) / 'decomp-acceptance.json').read_text()), pointer)
            self.assertTrue(json.loads((Path(pointer['run']) / 'verdict.json').read_text())['routine_accepted'])
            self.assertFalse(json.loads((Path(directory) / 'latest-attempt.json').read_text())['routine_accepted'])
