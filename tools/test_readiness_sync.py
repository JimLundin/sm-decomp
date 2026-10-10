"""Strict independent checkpoint comparison at the confirmed evidence seam."""
import hashlib
import json
import tempfile
import tarfile
import io
import unittest
from pathlib import Path

from compare_sync import FIELDS
from readiness_sync import compare_captures, observer_script
from readiness import digest, inventory_digest
from setup_lsnes import configure

INPUTS = {'rom_sha256': '1' * 64, 'movie_sha256': '2' * 64,
          'input_sha256': '3' * 64, 'frames': 129712, 'seed': 0,
          'reset': 'clean default power-on; no reset/subframe records',
          'controller': 'P1 SNES gamepad', 'region': 'NTSC'}


class ReadinessSyncTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def capture(self, name, role, rows=None):
        path = self.root / (name + '.csv')
        rows = rows or [(272, 1, 44), (272, 2, 45), (18000, 3, 46)]
        path.write_text('frame,event,' + ','.join(FIELDS) + '\n' + ''.join(
            f'{frame},{event},0000,0000,{x:04X},0000,0000,0000,0000,0000,0000,0000\n'
            for frame, event, x in rows))
        directory = self.root / name
        directory.mkdir(exist_ok=True)
        source = directory / 'source'
        source.mkdir(exist_ok=True)
        libraries = directory / 'libraries'
        options_keys = ('CC LD RANLIB CROSS_PREFIX HOSTCC USER_HOSTCCFLAGS USER_CFLAGS USER_LDFLAGS LUA REGEX THREADS GRAPHICS SOUND JOYSTICK BUILD_BSNES BSNES_IS_COMPAT BUILD_GAMBATTE SECRET_RABBIT_CODE OPUS_CODEC OPUS_CODEC_SUPPORTS_SURROUND USE_LIBGCRYPT USE_LIBLZMA BOOST_NEEDS_MT HOST_BOOST_NEEDS_MT').split()
        tar = directory / 'sources.tar'
        initial = {'Cargo.lock': 'fixture lockfile', 'include/library/httpreq.hpp': '// header\n', 'include/library/loadlib.hpp': '// header\n',
                   'bsnes/snes/snes.hpp': '// header\n', 'bsnes/Makefile': 'CFLAGS=-std=gnu++0x\n',
                   'Makefile': '# original makefile\n', 'options.build': ''.join(key + '=\n' for key in options_keys)}
        with tarfile.open(tar, 'w') as archive:
            for relative, contents in initial.items():
                data = contents.encode()
                member = tarfile.TarInfo(relative)
                member.size = len(data)
                archive.addfile(member, io.BytesIO(data))
        with tarfile.open(tar) as archive:
            archive.extractall(source)
        configure(source, tar, libraries)
        inventory = [{'path': item.relative_to(source).as_posix(), 'sha256': digest(item)}
                     for item in sorted(source.rglob('*')) if item.is_file()]
        inventory_path = directory / 'inventory.json'
        inventory_path.write_text(json.dumps({'sha256': inventory_digest(inventory), 'files': inventory}))
        binary, log, recipe, official = [directory / file for file in ('binary', 'build.log', 'recipe.py', 'official.7z')]
        for item in (binary, log, recipe, official):
            item.write_text(item.name)
        build_report = directory / 'build.json'
        build_command = ['make', '-j4', 'headless'] if role == 'original' else ['cargo', 'build', '--release', '-p', 'snes', '--bin', 'reference-replay', '--locked', '--offline']
        build_report.write_text(json.dumps({'status': 'pass', 'complete': True, 'returncode': 0, 'command': build_command}))
        observer = directory / 'observer.lua'
        observer.write_text(observer_script(path))
        provenance = {'method': 'verified-clean-build', 'source_sha256': inventory_digest(inventory),
                      'binary_sha256': digest(binary), 'build_log_sha256': digest(log), 'recipe_sha256': digest(recipe),
                      'build_report_sha256': digest(build_report), 'build_command': build_command, 'compiler': 'fixture compiler', 'features': ['Compatibility' if role == 'original' else 'lsnes-profile'],
                      'artifacts': {'binary': str(binary), 'build_log': str(log), 'recipe': str(recipe),
                                    'source_inventory': str(inventory_path), 'source_root': str(source),
                                    'build_report': str(build_report)}}
        if role == 'original':
            provenance.update(archive_sha256=digest(official), source_tar_sha256=digest(tar),
                              configured_libraries=str(libraries), packages=[], build_dependencies=[])
            provenance['artifacts'].update(official_archive=str(official), source_tar=str(tar))
            self.origin = {'sha256': digest(official), 'source_tar_sha256': digest(tar)}
        else:
            hardware = {'source_sha256': inventory_digest([]), 'files': []}
            provenance.update(hardware_source_sha256=hardware['source_sha256'], hardware=hardware, hardware_root=str(source), lockfile_sha256=digest(source / 'Cargo.lock'))
            (self.root / 'manifest.json').write_text(json.dumps({'run_id': name, 'identities': {'source': {'sha256': inventory_digest(inventory)}, 'hardware': hardware, 'Cargo.lock': {'sha256': digest(source / 'Cargo.lock')}, 'compiler': {'version': 'fixture compiler'}}}))
        inputs = dict(INPUTS)
        files = []
        for kind, key in (('rom', 'rom_sha256'), ('movie', 'movie_sha256'), ('inputs', 'input_sha256')):
            item = directory / kind
            item.write_text(kind)
            inputs[key] = digest(item)
            files.append({'command_path': str(item), 'artifact': str(item), 'sha256': digest(item)})
        self.inputs = inputs
        command = [str(binary), '--length=18000', '--dumper=INTERNAL-NULL', '--lua=' + str(observer), '--rom-a=' + str(directory / 'rom'), str(directory / 'movie')] if role == 'original' else [str(binary), str(directory / 'inputs'), '--frames', '18000', '--lsnes-frame-labels', '--rom', str(directory / 'rom'), '--checkpoints', str(path)]
        capture_report = directory / 'capture.json'
        capture_report.write_text(json.dumps({'status': 'pass', 'complete': True, 'returncode': 0, 'command': command}))
        for item in (path, observer, binary):
            files.append({'command_path': str(item), 'artifact': str(item), 'sha256': digest(item)})
        metadata = {'schema_version': 1, 'role': role, 'run_id': name,
                    'completed_frames': 18000, 'checkpoint_sha256': digest(path),
                    'event_count': len(rows), 'inputs': inputs,
                    'observer': {'checkpoint_pc': '828948', 'fields': list(FIELDS),
                                 'semantics': 'actual opcode fetch; discarded interrupt fetch excluded',
                                 'frame_labels': 'lsnes currentframe; NTSC callback at V241',
                                 'sha256': digest(observer), 'artifact': str(observer)}, 'provenance': provenance,
                    'command': command, 'command_files': files, 'capture_report': str(capture_report),
                    'capture_report_sha256': digest(capture_report)}
        path.with_suffix('.capture.json').write_text(json.dumps(metadata))
        return path

    def test_identity_bound_exact_ordered_visits_pass(self):
        result = compare_captures(self.capture('original', 'original'),
                                  self.capture('candidate', 'candidate'), self.inputs, origin=self.origin)
        self.assertEqual(result['status'], 'pass')
        self.assertEqual(result['events'], 3)
        self.assertEqual(result['expected_frames'], 18000)
        self.assertEqual(result['candidate_frame_offset'], 0)
        self.assertTrue(result['complete'])

    def test_missing_legacy_sidecar_and_wrong_input_cannot_pass(self):
        original = self.capture('original', 'original')
        candidate = self.capture('candidate', 'candidate')
        sidecar = original.with_suffix('.capture.json')
        metadata = json.loads(sidecar.read_text())
        sidecar.unlink()
        with self.assertRaises(OSError):
            compare_captures(original, candidate, self.inputs, origin=self.origin)
        metadata['inputs']['movie_sha256'] = 'c' * 64
        sidecar.write_text(json.dumps(metadata))
        with self.assertRaisesRegex(ValueError, 'identity'):
            compare_captures(original, candidate, self.inputs, origin=self.origin)

    def test_shift_extra_visit_and_first_semantic_disagreement_fail(self):
        original = self.capture('original', 'original')
        shifted = self.capture('shifted', 'candidate', [(273, 1, 44), (273, 2, 45), (18000, 3, 46)])
        self.assertEqual(compare_captures(original, shifted, self.inputs, origin=self.origin)['status'], 'divergence')
        extra = self.capture('extra', 'candidate', [(272, 1, 44), (272, 2, 45), (272, 3, 45), (18000, 4, 46)])
        self.assertEqual(compare_captures(original, extra, self.inputs, origin=self.origin)['status'], 'divergence')
        bad = self.capture('bad', 'candidate', [(272, 1, 44), (272, 2, 43), (18000, 3, 46)])
        result = compare_captures(original, bad, self.inputs, origin=self.origin)
        self.assertEqual(result['candidate_event'], 2)
        self.assertEqual(result['differences'], {'x': [45, 43]})

    def test_stale_digest_incomplete_scope_and_reordered_events_are_rejected(self):
        original = self.capture('original', 'original')
        candidate = self.capture('candidate', 'candidate')
        candidate.write_text(candidate.read_text() + '\n')
        with self.assertRaisesRegex(ValueError, 'checksum'):
            compare_captures(original, candidate, self.inputs, origin=self.origin)
        candidate = self.capture('candidate', 'candidate')
        sidecar = candidate.with_suffix('.capture.json')
        metadata = json.loads(sidecar.read_text())
        metadata['completed_frames'] = 17999
        sidecar.write_text(json.dumps(metadata))
        with self.assertRaisesRegex(ValueError, 'Truncated'):
            compare_captures(original, candidate, self.inputs, origin=self.origin)
        candidate = self.capture('candidate', 'candidate', [(272, 1, 44), (272, 3, 45), (18000, 2, 46)])
        with self.assertRaisesRegex(ValueError, 'reordered'):
            compare_captures(original, candidate, self.inputs, origin=self.origin)

    def test_lua_observer_writes_real_lf_rows(self):
        script = observer_script(self.root / 'checkpoints.csv')
        self.assertIn("'\\n'", script)
        self.assertNotIn("'\\\\n'", script)
        self.assertIn("memory.registerexec('BUS', 0x828948, checkpoint)", script)

    def test_internally_consistent_forged_provenance_cannot_certify_capture(self):
        original = self.capture('original', 'original')
        candidate = self.capture('candidate', 'candidate')
        metadata = json.loads(original.with_suffix('.capture.json').read_text())
        metadata['provenance']['binary_sha256'] = 'f' * 64
        original.with_suffix('.capture.json').write_text(json.dumps(metadata))
        with self.assertRaises((ValueError, OSError, KeyError)):
            compare_captures(original, candidate, self.inputs, origin=self.origin)

    def test_rehashed_modified_source_cannot_impersonate_official_build(self):
        original = self.capture('original', 'original')
        candidate = self.capture('candidate', 'candidate')
        sidecar = original.with_suffix('.capture.json')
        metadata = json.loads(sidecar.read_text())
        source = Path(metadata['provenance']['artifacts']['source_root'])
        (source / 'bsnes/snes/snes.hpp').write_text('// changed emulation source\n')
        inventory = [{'path': item.relative_to(source).as_posix(), 'sha256': digest(item)}
                     for item in sorted(source.rglob('*')) if item.is_file()]
        Path(metadata['provenance']['artifacts']['source_inventory']).write_text(json.dumps(
            {'sha256': inventory_digest(inventory), 'files': inventory}))
        metadata['provenance']['source_sha256'] = inventory_digest(inventory)
        sidecar.write_text(json.dumps(metadata))
        with self.assertRaisesRegex(ValueError, 'official source'):
            compare_captures(original, candidate, self.inputs, origin=self.origin)

    def test_rehashed_capture_command_cannot_claim_different_scope(self):
        original = self.capture('original', 'original')
        candidate = self.capture('candidate', 'candidate')
        sidecar = original.with_suffix('.capture.json')
        metadata = json.loads(sidecar.read_text())
        metadata['command'][metadata['command'].index('--length=18000')] = '--length=100'
        report = Path(metadata['capture_report'])
        record = json.loads(report.read_text())
        record['command'] = metadata['command']
        report.write_text(json.dumps(record))
        metadata['capture_report_sha256'] = digest(report)
        sidecar.write_text(json.dumps(metadata))
        with self.assertRaisesRegex(ValueError, 'scope'):
            compare_captures(original, candidate, self.inputs, origin=self.origin)


if __name__ == '__main__':
    unittest.main()
