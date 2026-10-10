import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name('port_cleanup.py')


class CleanupTests(unittest.TestCase):
    def test_preview_and_apply_preserve_sealed_and_unfinished_builds(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for name in ('completed', 'sealed', 'unfinished'):
                target = root / name / 'target'
                target.mkdir(parents=True)
                (target / 'build.o').write_bytes(b'build')
            (root / 'completed' / 'verdict.json').write_text(json.dumps({'status': 'mixed-C-Rust-integration', 'artifacts': {'terminal.bin': 'digest'}}))
            (root / 'sealed' / 'verdict.json').write_text(json.dumps({'status': 'offline-verified', 'nested': [{'artifacts': {'other/../target/build.o': 'digest'}}]}))
            (root / 'linked').symlink_to(root / 'completed', target_is_directory=True)
            preview = subprocess.run([sys.executable, str(SCRIPT), '--root', str(root)], capture_output=True, text=True)
            self.assertEqual(preview.returncode, 0, preview.stderr)
            self.assertTrue((root / 'completed' / 'target').exists())
            applied = subprocess.run([sys.executable, str(SCRIPT), '--root', str(root), '--apply'], capture_output=True, text=True)
            self.assertEqual(applied.returncode, 0, applied.stderr)
            self.assertFalse((root / 'completed' / 'target').exists())
            self.assertTrue((root / 'sealed' / 'target' / 'build.o').exists())
            self.assertTrue((root / 'unfinished' / 'target' / 'build.o').exists())


if __name__ == '__main__':
    unittest.main()
