import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

class BoundedJobs(unittest.TestCase):
    def test_unknown_job_and_exhausted_count_cannot_accept(self):
        for flags in (['--job', 'unknown'], ['--count', '2'], ['--count', '0']):
            with tempfile.TemporaryDirectory() as directory:
                result = subprocess.run([sys.executable, 'tools/decomp_jobs.py', '--output', directory, *flags], capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                verdict = json.loads((Path(directory) / 'progress.json').read_text())
                self.assertEqual(verdict['status'], 'error')
                self.assertEqual(verdict['accepted_jobs'], [])
