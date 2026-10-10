"""Batch orchestration contracts: resume validation, failure and live eligibility."""
import json
import fcntl
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import port_batch


class PortBatchTests(unittest.TestCase):
    def test_second_command_rejects_while_batch_record_is_locked(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            lock_path = root / 'out/port/batches/runner.lock'
            lock_path.parent.mkdir(parents=True)
            with lock_path.open('a') as held:
                fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
                with patch.object(port_batch, 'ROOT', root), patch.object(port_batch, 'run_main') as run:
                    self.assertEqual(port_batch.main(), 1)
                    run.assert_not_called()

    def test_replay_includes_only_live_families_and_optional_plm(self):
        config = {'families': {'pure': 'family', 'collision': 'callback',
                               'setup': 'offline-family', 'plm-spawn': 'plm'}}
        batch = {'families': {name: {'exit_code': 0, 'verdict': name + '.json'}
                              for name in config['families']}}
        command = port_batch.replay_command(batch, config, list(config['families']), 9000, Path('/tmp/replay'), True)
        self.assertIn('--family-verdict', command)
        self.assertIn('pure.json', command)
        self.assertIn('--collision-verdict', command)
        self.assertIn('collision.json', command)
        self.assertNotIn('setup.json', command)
        self.assertIn('--plm-verdict', command)
        with self.assertRaisesRegex(ValueError, 'No live-eligible'):
            port_batch.replay_command(batch, config, ['setup'], 9000, Path('/tmp/replay'))
        batch['families']['pure']['exit_code'] = 1
        with self.assertRaisesRegex(ValueError, 'missing passing evidence'):
            port_batch.replay_command(batch, config, ['pure'], 9000, Path('/tmp/replay'))

    def test_resume_authenticates_evidence_and_rebuilds_stale_dependency(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = root / 'tools/port/example.json'
            manifest.parent.mkdir(parents=True)
            manifest.write_text('{}')
            path = root / 'batch.json'
            from port_family import digest
            previous = {'status': 'offline-verified', 'exit_code': 0,
                        'verdict': str(root / 'old.json'), 'manifest_sha256': digest(manifest)}
            batch = {'families': {'example': previous}, 'replays': []}
            config = {'families': {'example': 'family'}}
            replacement = {'status': 'offline-verified', 'exit_code': 0, 'verdict': str(root / 'new.json')}
            with patch.object(port_batch, 'ROOT', root), patch.object(port_batch, 'validate') as validate, patch.object(port_batch, 'execute', return_value=replacement) as execute:
                self.assertEqual(port_batch.verify_batch(path, batch, config, ['example']), 0)
                execute.assert_not_called()
                validate.side_effect = [ValueError('dependency changed'), {}]
                self.assertEqual(port_batch.verify_batch(path, batch, config, ['example']), 0)
                execute.assert_called_once()
                self.assertEqual(json.loads(path.read_text())['families']['example']['verdict'], str(root / 'new.json'))

    def test_failed_family_is_recorded_and_returns_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = root / 'tools/port/example.json'
            manifest.parent.mkdir(parents=True)
            manifest.write_text('{}')
            path = root / 'batch.json'
            batch = {'families': {}, 'replays': []}
            with patch.object(port_batch, 'ROOT', root), patch.object(port_batch, 'execute', return_value={'status': 'rejected', 'exit_code': 1}):
                self.assertEqual(port_batch.verify_batch(path, batch, {'families': {'example': 'family'}}, ['example']), 1)
            self.assertEqual(json.loads(path.read_text())['families']['example']['status'], 'rejected')
