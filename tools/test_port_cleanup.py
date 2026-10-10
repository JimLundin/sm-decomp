import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name('port_cleanup.py')


class CleanupTests(unittest.TestCase):
    def test_deduplicate_preserves_snapshot_bytes_and_seals(self):
        from port_cleanup import deduplicate
        import hashlib
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            payload = bytes(0x20000)
            digest = hashlib.sha256(payload).hexdigest()
            for name in ('one', 'two'):
                run = root / name
                run.mkdir()
                (run / 'before.wram').write_bytes(payload)
                (run / 'verdict.json').write_text(json.dumps({'status': 'offline-verified', 'artifacts': {'before.wram': digest}}))
            self.assertEqual(deduplicate(root, False), len(payload))
            self.assertNotEqual((root / 'one/before.wram').stat().st_ino, (root / 'two/before.wram').stat().st_ino)
            self.assertEqual(deduplicate(root, True), len(payload))
            self.assertEqual((root / 'one/before.wram').read_bytes(), payload)
            self.assertEqual((root / 'two/before.wram').read_bytes(), payload)
            self.assertEqual((root / 'one/before.wram').stat().st_ino, (root / 'two/before.wram').stat().st_ino)

    def test_preview_and_apply_preserve_sealed_and_unfinished_builds(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for name in ('completed', 'sealed', 'unfinished'):
                target = root / name / 'target'
                target.mkdir(parents=True)
                (target / 'build.o').write_bytes(b'build')
            (root / 'completed' / 'verdict.json').write_text(json.dumps({'status': 'mixed-C-Rust-integration', 'artifacts': {'terminal.bin': 'digest'}}))
            (root / 'sealed' / 'verdict.json').write_text(json.dumps({'status': 'offline-verified', 'nested': [{'artifacts': {'other/../target/build.o': 'digest'}}]}))
            (root / 'completed' / 'libfamilies.a').write_bytes(b'archive')
            (root / 'completed' / 'live').write_bytes(b'linked binary')
            (root / 'linked').symlink_to(root / 'completed', target_is_directory=True)
            preview = subprocess.run([sys.executable, str(SCRIPT), '--root', str(root)], capture_output=True, text=True)
            self.assertEqual(preview.returncode, 0, preview.stderr)
            self.assertTrue((root / 'completed' / 'target').exists())
            applied = subprocess.run([sys.executable, str(SCRIPT), '--root', str(root), '--apply'], capture_output=True, text=True)
            self.assertEqual(applied.returncode, 0, applied.stderr)
            self.assertFalse((root / 'completed' / 'target').exists())
            self.assertFalse((root / 'completed' / 'libfamilies.a').exists())
            self.assertTrue((root / 'completed' / 'live').exists())
            self.assertTrue((root / 'sealed' / 'target' / 'build.o').exists())
            self.assertTrue((root / 'unfinished' / 'target' / 'build.o').exists())


if __name__ == '__main__':
    unittest.main()
