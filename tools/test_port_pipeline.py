"""Real process/cache checks for cross-worktree scheduling and publication."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from port_pipeline import cache_artifact, lease


class PipelineTests(unittest.TestCase):
    def test_resource_budget_blocks_second_process_and_releases_after_failure(self):
        with tempfile.TemporaryDirectory() as temporary, patch.dict(os.environ, {
            'SM_PIPELINE_ROOT': temporary, 'SM_PIPELINE_WORKERS': '1'}):
            marker = Path(temporary) / 'entered'
            code = 'from port_pipeline import lease; from pathlib import Path; import sys\nwith lease(): Path(sys.argv[1]).write_text("entered")'
            with lease():
                process = subprocess.Popen([sys.executable, '-c', code, str(marker)],
                    env=dict(os.environ, PYTHONPATH=str(Path(__file__).parent)))
                with self.assertRaises(subprocess.TimeoutExpired):
                    process.wait(timeout=0.2)
                self.assertFalse(marker.exists())
            self.assertEqual(process.wait(timeout=10), 0)
            self.assertTrue(marker.exists())
            with self.assertRaises(RuntimeError):
                with lease():
                    raise RuntimeError('worker failed')
            with lease():
                pass

    def test_cache_reuse_changed_inputs_corruption_and_failed_publication(self):
        with tempfile.TemporaryDirectory() as temporary, patch.dict(os.environ, {'SM_PIPELINE_ROOT': temporary}):
            root = Path(temporary)
            source = root / 'source'; source.write_text('one')
            output = root / 'output'
            calls = []
            def build(path):
                calls.append(path)
                path.write_bytes(source.read_bytes())
            command = [sys.executable, 'test-input-copy']
            first = cache_artifact(command, [source], output, build)
            self.assertFalse(first['hit'])
            self.assertTrue(cache_artifact(command, [source], output, build)['hit'])
            self.assertEqual(len(calls), 1)
            (root / 'build-cache' / first['key'] / 'artifact').write_text('corrupted')
            self.assertFalse(cache_artifact(command, [source], output, build)['hit'])
            source.write_text('two')
            self.assertFalse(cache_artifact(command, [source], output, build)['hit'])
            self.assertEqual(output.read_text(), 'two')
            def failed(path):
                path.write_text('partial')
                raise RuntimeError('compiler failed')
            with self.assertRaises(RuntimeError):
                cache_artifact(command + ['failed'], [source], output, failed)
            self.assertFalse(cache_artifact(command + ['failed'], [source], output, build)['hit'])

    def test_two_processes_publish_only_one_build_for_a_key(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);source=root/'source';source.write_text('source')
            script=root/'worker.py'
            script.write_text('''import sys,time
from pathlib import Path
from port_pipeline import cache_artifact
root=Path(sys.argv[1])
def build(path):
 with (root/'builds').open('a') as log:log.write('build\\n')
 time.sleep(0.2)
 path.write_text('artifact')
cache_artifact([sys.executable,'concurrent'],[root/'source'],root/sys.argv[2],build)
''')
            env=dict(os.environ,SM_PIPELINE_ROOT=temporary,PYTHONPATH=str(Path(__file__).parent))
            workers=[subprocess.Popen([sys.executable,str(script),temporary,name],env=env) for name in ('one','two')]
            for worker in workers:self.assertEqual(worker.wait(timeout=15),0)
            self.assertEqual((root/'builds').read_text(),'build\n')
            self.assertEqual((root/'one').read_bytes(),(root/'two').read_bytes())
