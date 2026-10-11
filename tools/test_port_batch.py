"""Batch orchestration contracts: resume validation, failure and live eligibility."""
import json
import fcntl
from pathlib import Path
import tempfile
import unittest
import sys
from unittest.mock import patch

import port_batch


class PortBatchTests(unittest.TestCase):
    def test_second_command_rejects_while_batch_record_is_locked(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            lock_path = root / 'batch/batch.lock'
            lock_path.parent.mkdir(parents=True)
            with lock_path.open('a') as held:
                fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
                with patch.object(port_batch, 'ROOT', root), patch.object(port_batch, 'CONFIG', Path(port_batch.CONFIG)), patch.object(port_batch, 'read_batch', return_value=(root / 'batch/batch.json', {'families':{},'replays':[]})), patch.object(port_batch, 'operate') as run, patch.object(sys, 'argv', ['port_batch.py','status']):
                    self.assertEqual(port_batch.main(), 1)
                    run.assert_not_called()

    def test_independent_batch_is_not_blocked_by_another_record(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            lock = root / 'first/batch.lock';lock.parent.mkdir()
            other = root / 'second/batch.json';other.parent.mkdir()
            with lock.open('a') as held:
                fcntl.flock(held, fcntl.LOCK_EX)
                with patch.object(port_batch, 'read_batch', return_value=(other,{})), patch.object(port_batch, 'operate', return_value=0) as run, patch.object(sys,'argv',['port_batch.py','status']):
                    self.assertEqual(port_batch.main(),0)
                    run.assert_called_once()

    def test_explicit_dispatcher_adapter_and_discovery_stay_separate(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); manifests=root/'tools/port';manifests.mkdir(parents=True)
            manifest=manifests/'leaves.json';manifest.write_text('{"rust_module":"leaf.rs","routines":[{"symbol":"EprojInstruction"}]}')
            config={'families':{},'profiles':{}}
            with patch.object(port_batch,'ROOT',root):
                discovered=port_batch.catalog(config)
                self.assertEqual(discovered['families']['leaves']['kind'],'dispatcher')
                self.assertIsNone(discovered['families']['leaves']['live'])
                discovered['families']['leaves']['live']='dispatcher'
                from port_family import digest
                record={'exit_code':0,'verdict':'leaves-verdict.json','manifest_sha256':digest(manifest)}
                command=port_batch.replay_command({'families':{'leaves':record}},discovered,['leaves'],1000,root/'replay')
                self.assertIn('--dispatcher-verdict',command)

    def test_replay_includes_only_live_families_and_optional_plm(self):
        from port_family import digest
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            config = {'families': {'pure': 'family', 'collision': 'callback',
                                   'setup': 'offline-family', 'plm-spawn': 'plm'}}
            records = {}
            for name in config['families']:
                manifest = root / 'tools/port' / (name + '.json')
                manifest.parent.mkdir(parents=True, exist_ok=True)
                manifest.write_text('{}')
                records[name] = {'exit_code': 0, 'verdict': name + '.json',
                                 'manifest_sha256': digest(manifest)}
            batch = {'families': records}
            with patch.object(port_batch, 'ROOT', root):
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

    def test_replay_rejects_changed_eligibility_and_native_plm_contract(self):
        from port_family import digest
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            records = {}
            manifests = {}
            config = {'families': {'pure': 'family', 'collision': 'callback', 'plm-spawn': 'plm'}}
            for name in config['families']:
                manifest = root / 'tools/port' / (name + '.json')
                manifest.parent.mkdir(parents=True, exist_ok=True)
                manifest.write_text('{"routines":[{"symbol":"Partial","live_eligible":true}]}')
                manifests[name] = manifest
                records[name] = {'exit_code': 0, 'verdict': name + '.json',
                                 'manifest_sha256': digest(manifest)}
            batch = {'families': records}
            with patch.object(port_batch, 'ROOT', root):
                for name in config['families']:
                    with self.subTest(family=name):
                        original = manifests[name].read_text()
                        manifests[name].write_text(original.replace('true', 'false'))
                        names = ['pure'] if name == 'plm-spawn' else [name]
                        with self.assertRaisesRegex(ValueError, name + ': manifest changed.*run verify'):
                            port_batch.replay_command(batch, config, names, 9000, Path('/tmp/replay'), name == 'plm-spawn')
                        manifests[name].write_text(original)

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
