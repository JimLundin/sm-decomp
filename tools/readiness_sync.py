#!/usr/bin/env python3
"""Strict, provenance-bound original-core synchronization for readiness."""
import json
import time
from pathlib import Path

from compare_sync import FIELDS, compare, read
from readiness import digest

OFFICIAL_ORIGIN = {'sha256': 'aa56d499d0393e24c8d7c9eaf0ed337916f07c02fdb2d281613bcb1cd052a1d7',
                   'source_tar_sha256': 'e9c0336218a47586e85b7c38341114995b3b7cdd727baf7c09c307d82072ef25'}

OBSERVER_SEMANTICS = 'actual opcode fetch; discarded interrupt fetch excluded'
FRAME_LABELS = 'lsnes currentframe; NTSC callback at V241'
INPUT_KEYS = ('rom_sha256', 'movie_sha256', 'input_sha256', 'frames', 'seed',
              'reset', 'controller', 'region')


def verified_capture(path, role, inputs, frames=18000, origin=None):
    """Require immutable completion and applicable identities, never infer legacy metadata."""
    path = Path(path)
    metadata = json.loads(path.with_suffix('.capture.json').read_text())
    if metadata.get('schema_version') != 1 or metadata.get('role') != role or not metadata.get('run_id'):
        raise ValueError('Missing verified capture schema/role/run identity: ' + str(path))
    if metadata.get('checkpoint_sha256') != digest(path):
        raise ValueError('Capture checksum mismatch: ' + str(path))
    if metadata.get('completed_frames', 0) < frames:
        raise ValueError('Truncated capture completion: ' + str(path))
    if role == 'candidate' and metadata['completed_frames'] != frames:
        raise ValueError('Fresh candidate must complete the exact requested scope')
    for key in INPUT_KEYS:
        if key not in inputs or metadata.get('inputs', {}).get(key) != inputs[key]:
            raise ValueError('Incompatible capture input/settings identity: ' + key)
    observer = metadata.get('observer', {})
    if (observer.get('checkpoint_pc') != '828948' or observer.get('fields') != list(FIELDS)
            or observer.get('semantics') != OBSERVER_SEMANTICS or observer.get('frame_labels') != FRAME_LABELS
            or not observer.get('sha256')):
        raise ValueError('Incompatible actual-fetch observer identity')
    provenance = metadata.get('provenance', {})
    required = ['source_sha256', 'binary_sha256', 'build_log_sha256', 'compiler', 'features']
    required += ['archive_sha256', 'source_tar_sha256'] if role == 'original' else ['hardware_source_sha256', 'lockfile_sha256']
    if provenance.get('method') != 'verified-clean-build' or any(not provenance.get(key) for key in required):
        raise ValueError('Missing verified source/build provenance: ' + str(path))
    rows = read(path)
    if metadata.get('event_count') != len(rows) or rows[-1]['frame'] > metadata['completed_frames'] + 1:
        raise ValueError('Capture event/completion count mismatch')
    verify_artifacts(metadata, origin or OFFICIAL_ORIGIN, path.parent.parent if path.parent.name == 'generated' else path.parent)
    return rows, metadata


def compare_captures(original, candidate, inputs, frames=18000, origin=None):
    started = time.monotonic()
    expected, original_metadata = verified_capture(original, 'original', inputs, frames, origin)
    actual, candidate_metadata = verified_capture(candidate, 'candidate', inputs, frames, origin)
    result = compare(expected, actual, 272, frames, 0)
    result.update(status='pass' if result['status'] == 'PASS' else 'divergence', complete=True,
                  expected_frames=frames, completed_frames=frames,
                  original_completed_frames=original_metadata['completed_frames'],
                  candidate_completed_frames=candidate_metadata['completed_frames'],
                  original=str(original), candidate=str(candidate), checkpoint_pc='828948',
                  original_sha256=digest(original), candidate_sha256=digest(candidate),
                  elapsed_seconds=time.monotonic() - started)
    return result


def observer_script(out):
    """Original black-box actual-fetch observer; a Lua escape produces each LF."""
    if ']]' in str(out):
        raise ValueError('Unsupported output path for Lua long string')
    offsets = (0x998, 0x79b, 0xaf6, 0xafa, 0x9c2, 0x9a4, 0x9da, 0x9dc, 0x9de, 0x9e0)
    return ('local out = assert(io.open([[' + str(out) + "]], 'w'))\n" +
            "out:write('frame,event," + ','.join(FIELDS) + "\\n')\n" +
            'local fields = {' + ','.join(str(offset) for offset in offsets) + '}\n' + r"""
local event = 0
local function checkpoint()
    event = event + 1
    local parts = {tostring(movie.currentframe()), tostring(event)}
    for _,address in ipairs(fields) do
        parts[#parts+1] = string.format('%04X', memory.readword('WRAM', address))
    end
    out:write(table.concat(parts, ','), '\n')
    out:flush()
end
memory.registerexec('BUS', 0x828948, checkpoint)
""")


def build_original(run, jobs=4):
    """Build fresh from independently hash-verified official sources/private packages."""
    import os
    import shutil
    import tarfile
    from setup_lsnes import configure
    from readiness import inventory_digest, write_json

    manifest = json.loads((run.root / 'docs/lsnes-dependencies.json').read_text())
    official = run.root / 'deps/downloads' / manifest['archive']['filename']
    original_tar = run.root / 'deps/lsnes/sources.tar'
    for path, expected in ((official, manifest['archive']['sha256']),
                           (original_tar, manifest['archive']['source_tar_sha256'])):
        if not path.exists() or run.guard_input(path) != expected:
            raise RuntimeError('Missing or invalid original source archive: run just setup-lsnes')
    build = run.path / 'generated/original-build'
    source, libraries = build / 'source', build / 'libraries'
    source.mkdir(parents=True)
    libraries.mkdir()
    # The verified official tar uses relative paths only. Extraction is local,
    # and byte inventories never inspect original emulator execution semantics.
    with tarfile.open(original_tar) as archive:
        for member in archive.getmembers():
            relative = Path(member.name)
            if relative.is_absolute() or '..' in relative.parts or member.issym() or member.islnk():
                raise RuntimeError('Unsafe member in official source tar')
        archive.extractall(source)
    packages = []
    for index, record in enumerate(manifest['packages']):
        path = run.root / 'deps/downloads' / record['filename']
        if not path.exists() or run.guard_input(path) != record['sha256']:
            raise RuntimeError('Missing or invalid original build package: run just setup-lsnes')
        result = run.run_stage('original-package-' + str(index), ['dpkg-deb', '-x', path, libraries])
        if result['status'] != 'pass':
            raise RuntimeError('Original private package extraction failed')
        packages.append({'filename': record['filename'], 'sha256': record['sha256'], 'path': str(path)})
    curl = Path('/usr/lib/x86_64-linux-gnu/libcurl.so.4').resolve()
    for alias, target in ((libraries / 'usr/bin/lua', 'lua5.2'),
                          (libraries / 'usr/bin/luac', 'luac5.2'),
                          (libraries / 'usr/lib/x86_64-linux-gnu/libcurl.so', str(curl))):
        if alias.exists() or alias.is_symlink():
            alias.unlink()
        alias.symlink_to(target)
    configure(source, original_tar, libraries)
    inventory = [{'path': path.relative_to(source).as_posix(), 'sha256': digest(path)}
                 for path in sorted(source.rglob('*')) if path.is_file()]
    inventory_path = build / 'source-inventory.json'
    write_json(inventory_path, {'sha256': inventory_digest(inventory), 'files': inventory})
    # Record compiler and the actual host/private link-library contents, rather
    # than confusing package filenames with the libraries the linker consumed.
    compiler_stage = run.run_stage('original-compiler', ['g++', '--version'])
    if compiler_stage['status'] != 'pass':
        raise RuntimeError('Original compiler unavailable')
    compiler = (run.path / compiler_stage['stdout']).read_text().strip()
    compiler_path = Path(shutil.which('g++')).resolve()
    dependencies = [{'path': str(compiler_path), 'sha256': run.guard_input(compiler_path)},
                    {'path': str(curl), 'sha256': run.guard_input(curl)}]
    for path in sorted(libraries.rglob('*')):
        if path.is_file() and not path.is_symlink():
            dependencies.append({'path': str(path), 'sha256': digest(path)})
    env = os.environ.copy()
    for name in ('MAKEFLAGS', 'MFLAGS', 'LD_PRELOAD', 'CFLAGS', 'CPPFLAGS', 'CXXFLAGS', 'LDFLAGS'):
        env.pop(name, None)
    env.update(PATH=str(libraries / 'usr/bin') + os.pathsep + env.get('PATH', ''),
               PKG_CONFIG_PATH=str(libraries / 'usr/lib/x86_64-linux-gnu/pkgconfig'))
    result = run.run_stage('original-build', ['make', '-j' + str(jobs), 'headless'], cwd=source, env=env)
    if result['status'] != 'pass':
        raise RuntimeError('Original clean build failed; inspect original-build logs')
    for entry in inventory:
        if digest(source / entry['path']) != entry['sha256']:
            raise RuntimeError('Original source changed during build: ' + entry['path'])
    binary = source / 'src/util/lsnes-dumpavi.util'
    run.record_artifact('original-binary', binary)
    run.manifest['identities']['original-binary']['evidence'] = str(binary.relative_to(run.path))
    attestation = {'method': 'verified-clean-build', 'archive_sha256': digest(official),
                   'source_tar_sha256': digest(original_tar), 'source_sha256': inventory_digest(inventory),
                   'binary_sha256': digest(binary), 'build_log_sha256': digest(run.path / result['stdout']),
                   'build_report_sha256': digest(run.path / 'logs/original-build.json'),
                   'build_command': result['command'],
                   'compiler': compiler, 'features': ['bsnes v085 Compatibility', 'NTSC', 'DUMMY graphics/sound/joystick'],
                   'artifacts': {'official_archive': str(official), 'source_tar': str(original_tar),
                                 'source_root': str(source), 'source_inventory': str(inventory_path),
                                 'binary': str(binary), 'build_log': str(run.path / result['stdout']),
                                 'build_report': str(run.path / 'logs/original-build.json'),
                                 'recipe': str(run.path / 'source/files/tools/setup_lsnes.py')},
                   'recipe_sha256': digest(run.path / 'source/files/tools/setup_lsnes.py'),
                   'configured_libraries': str(libraries),
                   'packages': packages, 'build_dependencies': dependencies}
    write_json(build / 'attestation.json', attestation)
    run.manifest['identities']['original-build'] = {'sha256': digest(build / 'attestation.json'),
                                                  'evidence': 'generated/original-build/attestation.json'}
    run.save_manifest()
    return binary, attestation


def capture_original(run, binary, provenance, frames=18000):
    import os
    from readiness import write_json
    out = run.path / 'generated/original.csv'
    script = out.with_suffix('.lua')
    script.write_text(observer_script(out))
    env = os.environ.copy()
    env['XDG_CONFIG_HOME'] = str(run.path / 'generated/original-config')
    movie = run.root / 'movies/super-metroid-any-3653.lsmv'
    rom = run.root / 'rom/sm.sfc'
    result = run.run_stage('original-replay', [binary, '--dumper=INTERNAL-NULL', '--rom-a=' + str(rom),
                           '--length=' + str(frames), '--lua=' + str(script), movie], env=env)
    if result['status'] != 'pass':
        raise RuntimeError('Original replay failed or incomplete')
    log = (run.path / result['stdout']).read_text(errors='replace') + (run.path / result['stderr']).read_text(errors='replace')
    if 'Finished!' not in log or 'bsnes v085 (Compatibility core)' not in log or 'snes_ntsc' not in log:
        raise RuntimeError('Original replay lacks verified completion/core/region diagnostics')
    rows = read(out)
    metadata = capture_metadata(run, 'original', out, rows, provenance, script, frames)
    metadata['capture_report'] = str(run.path / 'logs/original-replay.json')
    metadata['capture_report_sha256'] = digest(run.path / 'logs/original-replay.json')
    bind_capture_command(run, metadata, result, out, script)
    portable_metadata(metadata, run.path)
    write_json(out.with_suffix('.capture.json'), metadata)
    return out


def capture_metadata(run, role, out, rows, provenance, observer, frames):
    return {'schema_version': 1, 'role': role, 'run_id': run.manifest['run_id'],
            'completed_frames': frames, 'checkpoint_sha256': digest(out), 'event_count': len(rows),
            'inputs': {key: run.manifest['identities']['inputs'][key] for key in INPUT_KEYS},
            'observer': {'checkpoint_pc': '828948', 'fields': list(FIELDS), 'semantics': OBSERVER_SEMANTICS,
                         'frame_labels': FRAME_LABELS, 'sha256': digest(observer), 'artifact': str(observer)},
            'provenance': provenance, 'checkpoint_artifact': 'generated/' + out.name}


def verify_artifacts(metadata, expected_origin=None, owner=None):
    def artifact(value):
        path = Path(value)
        return path if path.is_absolute() else Path(owner) / path
    """Rehash linked provenance; strings in a capture sidecar are never attestations."""
    from readiness import inventory_digest
    observer = metadata['observer']
    if digest(artifact(observer['artifact'])) != observer['sha256']:
        raise ValueError('Observer artifact checksum mismatch')
    provenance = metadata['provenance']
    artifacts = provenance['artifacts']
    for name, field in (('binary', 'binary_sha256'), ('build_log', 'build_log_sha256'), ('recipe', 'recipe_sha256')):
        if digest(artifact(artifacts[name])) != provenance[field]:
            raise ValueError('Provenance artifact checksum mismatch: ' + name)
    report = json.loads(artifact(artifacts['build_report']).read_text())
    if digest(artifact(artifacts['build_report'])) != provenance.get('build_report_sha256') or report.get('command') != provenance.get('build_command'):
        raise ValueError('Build process report identity/invocation mismatch')
    if report.get('status') != 'pass' or report.get('complete') is not True or report.get('returncode') != 0:
        raise ValueError('Build did not complete successfully')
    inventory = json.loads(artifact(artifacts['source_inventory']).read_text())
    if inventory_digest(inventory['files']) != provenance['source_sha256'] or inventory['sha256'] != provenance['source_sha256']:
        raise ValueError('Source inventory checksum mismatch')
    for entry in inventory['files']:
        relative = Path(entry['path'])
        if relative.is_absolute() or '..' in relative.parts or digest(artifact(artifacts['source_root']) / relative) != entry['sha256']:
            raise ValueError('Source artifact checksum mismatch')
    if metadata['role'] == 'original':
        artifact_path = artifact
        for artifact_name, field in (('official_archive', 'archive_sha256'), ('source_tar', 'source_tar_sha256')):
            if digest(artifact_path(artifacts[artifact_name])) != provenance[field]:
                raise ValueError('Official original source artifact mismatch')
        if expected_origin and (provenance['archive_sha256'] != expected_origin['sha256'] or
                               provenance['source_tar_sha256'] != expected_origin['source_tar_sha256']):
            raise ValueError('Original archive differs from independently pinned official origin')
        for record in provenance.get('packages', []) + provenance.get('build_dependencies', []):
            if digest(artifact(record['path'])) != record['sha256']:
                raise ValueError('Original build dependency changed')
    capture_report = json.loads(artifact(metadata['capture_report']).read_text())
    if digest(artifact(metadata['capture_report'])) != metadata['capture_report_sha256'] or (
            capture_report.get('status') != 'pass' or capture_report.get('complete') is not True
            or capture_report.get('returncode') != 0):
        raise ValueError('Capture process lacks digest-bound successful completion')
    command = metadata.get('command')
    if command != capture_report.get('command') or not metadata.get('command_files'):
        raise ValueError('Capture command is not bound to completion evidence')
    files = metadata['command_files']
    if any(digest(artifact(record['artifact'])) != record['sha256'] for record in files):
        raise ValueError('Capture command input/artifact changed')
    expected_digests = {metadata['checkpoint_sha256'], observer['sha256'], provenance['binary_sha256'],
                        metadata['inputs']['rom_sha256'], metadata['inputs']['movie_sha256'],
                        metadata['inputs']['input_sha256']}
    if not expected_digests.issubset({record['sha256'] for record in files}):
        raise ValueError('Capture command does not bind all required inputs')
    if command[0] != next(record['command_path'] for record in files if record['sha256'] == provenance['binary_sha256']):
        raise ValueError('Capture used a different executable')
    if metadata['role'] == 'original':
        if '--length=' + str(metadata['completed_frames']) not in command or '--dumper=INTERNAL-NULL' not in command:
            raise ValueError('Original capture invocation scope differs')
        checkpoint_path = next(record['command_path'] for record in files if record['sha256'] == metadata['checkpoint_sha256'])
        observer_path = next(record['command_path'] for record in files if record['sha256'] == observer['sha256'])
        rom_path = next(record['command_path'] for record in files if record['sha256'] == metadata['inputs']['rom_sha256'])
        movie_path = next(record['command_path'] for record in files if record['sha256'] == metadata['inputs']['movie_sha256'])
        if ('--lua=' + observer_path not in command or '--rom-a=' + rom_path not in command
                or movie_path not in command or artifact(observer['artifact']).read_text() != observer_script(checkpoint_path)):
            raise ValueError('Original capture command/observer does not bind exact checkpoint/ROM/movie')
        if provenance['build_command'][0] != 'make' or provenance['build_command'][-1] != 'headless':
            raise ValueError('Original build command does not select Compatibility headless target')
        verify_original_inventory(provenance, artifact)
    else:
        owner_manifest = json.loads((Path(owner) / 'manifest.json').read_text())
        identities = owner_manifest['identities']
        if (metadata['run_id'] != owner_manifest['run_id'] or provenance['source_sha256'] != identities['source']['sha256']
                or provenance['hardware_source_sha256'] != identities['hardware']['source_sha256']
                or provenance['hardware'] != identities['hardware'] or provenance['lockfile_sha256'] != identities['Cargo.lock']['sha256']
                or provenance['compiler'] != identities['compiler']['version']):
            raise ValueError('Candidate provenance differs from owner source/hardware/build identities')
        lockfile = artifact(artifacts['source_root']) / 'Cargo.lock'
        if digest(lockfile) != provenance['lockfile_sha256']:
            raise ValueError('Candidate lockfile artifact changed')
        required_build = ['cargo', 'build', '--release', '-p', 'snes', '--bin', 'reference-replay', '--locked', '--offline']
        if provenance['build_command'] != required_build:
            raise ValueError('Candidate build invocation differs from verified settings')
        if provenance.get('hardware'):
            hardware = provenance['hardware']
            if hardware['source_sha256'] != provenance['hardware_source_sha256'] or inventory_digest(hardware['files']) != hardware['source_sha256']:
                raise ValueError('Candidate hardware inventory identity mismatch')
            for entry in hardware['files']:
                if digest(Path(provenance['hardware_root']) / entry['path']) != entry['sha256']:
                    raise ValueError('Candidate hardware artifact changed')
        if '--frames' not in command or command[command.index('--frames') + 1] != str(metadata['completed_frames']) or '--lsnes-frame-labels' not in command:
            raise ValueError('Candidate capture invocation scope/labels differs')
        for flag, expected_digest in (('--rom', metadata['inputs']['rom_sha256']),
                                      ('--checkpoints', metadata['checkpoint_sha256'])):
            value = next(record['command_path'] for record in files if record['sha256'] == expected_digest)
            if flag not in command or command[command.index(flag) + 1] != value:
                raise ValueError('Candidate command does not bind exact ROM/checkpoint')
        input_path = next(record['command_path'] for record in files if record['sha256'] == metadata['inputs']['input_sha256'])
        if command[1] != input_path:
            raise ValueError('Candidate command does not bind exact canonical input')



def capture_candidate(run, frames=18000):
    """Build and cold replay this maintained source; no older candidate is substituted."""
    import os
    from readiness import write_json
    target = run.path / 'generated/reference-target'
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(target)
    result = run.run_stage('sync-reference-build', ['cargo', 'build', '--release', '-p', 'snes',
                           '--bin', 'reference-replay', '--locked', '--offline'], env=env)
    if result['status'] != 'pass':
        raise RuntimeError('Fresh reference build failed or incomplete')
    binary = target / 'release/reference-replay'
    run.record_artifact('sync-reference-binary', binary)
    observer = run.path / 'source/files/crates/snes/src/checkpoint.rs'
    provenance = {'method': 'verified-clean-build', 'source_sha256': run.manifest['identities']['source']['sha256'],
                  'hardware_source_sha256': run.manifest['identities']['hardware']['source_sha256'],
                  'lockfile_sha256': run.manifest['identities']['Cargo.lock']['sha256'],
                  'hardware': run.manifest['identities']['hardware'],
                  'hardware_root': str((run.root / 'deps/rustysnes').resolve()),
                  'binary_sha256': digest(binary), 'build_log_sha256': digest(run.path / result['stdout']),
                  'build_report_sha256': digest(run.path / 'logs/sync-reference-build.json'),
                  'build_command': result['command'],
                  'compiler': run.manifest['identities']['compiler']['version'],
                  'features': run.manifest['identities']['hardware']['features'],
                  'recipe_sha256': digest(run.path / 'source/files/Cargo.toml'),
                  'artifacts': {'binary': str(binary), 'build_log': str(run.path / result['stdout']),
                                'build_report': str(run.path / 'logs/sync-reference-build.json'),
                                'source_inventory': str(run.path / 'source/inventory.json'),
                                'source_root': str(run.path / 'source/files'),
                                'recipe': str(run.path / 'source/files/Cargo.toml')}}
    out = run.path / 'generated/candidate.csv'
    result = run.run_stage('sync-reference-replay', [binary, run.path / 'generated/movie.inputs',
                           '--rom', run.root / 'rom/sm.sfc', '--frames', str(frames),
                           '--out', run.path / 'generated/reference-fingerprints.csv', '--checkpoints', out,
                           '--lsnes-frame-labels'])
    if result['status'] != 'pass':
        raise RuntimeError('Fresh reference replay failed or incomplete')
    raw = json.loads(out.with_suffix('.capture.json').read_text())
    if raw['completed_frames'] != frames or raw['checkpoint_sha256'] != digest(out):
        raise RuntimeError('Fresh reference completion count/checksum differs from requested capture')
    # Preserve the runner completion sidecar before replacing it with the richer
    # verified envelope, which additionally binds all build/input identities.
    write_json(out.with_suffix('.runner.json'), raw)
    metadata = capture_metadata(run, 'candidate', out, read(out), provenance, observer, frames)
    metadata.update(capture_report=str(run.path / 'logs/sync-reference-replay.json'),
                    capture_report_sha256=digest(run.path / 'logs/sync-reference-replay.json'))
    bind_capture_command(run, metadata, result, out, observer)
    portable_metadata(metadata, run.path)
    write_json(out.with_suffix('.capture.json'), metadata)
    return out


def preserve_debt(run):
    """Historical captures retain identities and limitations; they cannot certify readiness."""
    import shutil
    from readiness import write_json
    directory = run.path / 'diagnostics/historical-debt'
    directory.mkdir()
    artifacts = []
    for name in ('full-any-lsnes.csv', 'full-any-lsnes.capture.json', 'full-any-lsnes.lua', 'full-any-lsnes.log',
                 'irq-mask-checkpoints.csv', 'irq-mask-checkpoints.capture.json',
                 'irq-mask-18000.json', 'irq-mask-36000.json'):
        source = run.root / 'out/sync' / name
        if source.exists():
            identity = run.guard_input(source)
            shutil.copy2(source, directory / name)
            if digest(directory / name) != identity:
                raise RuntimeError('Historical debt artifact changed while copying')
            artifacts.append({'name': name, 'sha256': identity, 'path': 'diagnostics/historical-debt/' + name})
    legacy = next((item for item in artifacts if item['name'] == 'full-any-lsnes.csv'), None)
    legacy_sidecar = directory / 'full-any-lsnes.capture.json'
    legacy_scope_verified = False
    if legacy and legacy_sidecar.exists():
        completion = json.loads(legacy_sidecar.read_text())
        legacy_scope_verified = completion.get('completed_frames') == 129712 and completion.get('checkpoint_sha256') == legacy['sha256']
        if not legacy_scope_verified:
            raise ValueError('Historical original capture completion/hash mismatch')
    record = {'readiness_eligible': False, 'reason': 'Legacy capture lacks complete source/binary/observer provenance',
              'legacy_original_declared_frames': 129712, 'legacy_completion_hash_verified': legacy_scope_verified,
              'later_label_disagreement': {'event': 15292, 'original_frame': 18195, 'candidate_frame': 18196},
              'later_semantic_disagreement': {'event': 24934, 'original_frame': 35203, 'candidate_frame': 35204,
                                             'x': [44, 43]}, 'artifacts': artifacts}
    write_json(directory / 'debt.json', record)
    run.manifest['historical_debt'] = 'diagnostics/historical-debt/debt.json'
    run.save_manifest()
    return record


def synchronization(run, original_run=None, frames=18000, jobs=4):
    """Complete independent gate; accepts original reuse only from verified terminal evidence."""
    from readiness import read_verdict
    started = time.monotonic()
    preserve_debt(run)
    expected_origin = json.loads((run.root / 'docs/lsnes-dependencies.json').read_text())['archive']
    if original_run:
        original_run = Path(original_run).resolve()
        verdict = read_verdict(original_run)
        if verdict['status'] != 'pass' or verdict.get('checks', {}).get('original-attestation', verdict.get('checks', {}).get('synchronization', {})).get('status') != 'pass':
            raise ValueError('Original reuse requires complete passing immutable run')
        original = original_run / 'generated/original.csv'
        _, metadata = verified_capture(original, 'original', run.manifest['identities']['inputs'], frames, expected_origin)
        verify_artifacts(metadata, expected_origin, original_run)
        owner_manifest = json.loads((original_run / 'manifest.json').read_text())
        if metadata['run_id'] != owner_manifest['run_id']:
            raise ValueError('Original capture belongs to a different immutable run')
        run.guard_input(original)
        run.guard_input(original.with_suffix('.capture.json'))
        run.manifest['reused_evidence'].append({'kind': 'original-oracle', 'run_id': metadata['run_id'],
                                               'path': str(original_run), 'checkpoint_sha256': digest(original)})
        run.save_manifest()
    else:
        binary, provenance = build_original(run, jobs)
        original = capture_original(run, binary, provenance, frames)
    candidate = capture_candidate(run, frames)
    original_metadata = json.loads(original.with_suffix('.capture.json').read_text())
    verify_artifacts(original_metadata, expected_origin, original.parent.parent)
    verify_artifacts(json.loads(candidate.with_suffix('.capture.json').read_text()), owner=run.path)
    result = compare_captures(original, candidate, run.manifest['identities']['inputs'], frames)
    result['elapsed_seconds'] = time.monotonic() - started
    run.check_sources()
    run.record_check('synchronization', result)
    return result


def main(argv=None):
    import argparse
    from readiness import Run, preflight
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument('--original-run', type=Path, help='Reuse verified original from a completed passing run')
    parser.add_argument('--jobs', type=int, default=4)
    parser.add_argument('--attest-original', type=Path, help='Preserve a completed original capture in separately verified immutable evidence')
    arguments = parser.parse_args(argv)
    if arguments.attest_original:
        path = attest_original(arguments.root, arguments.attest_original)
        print('Run: ' + str(path))
        print('Brief: ' + str(path / 'brief.md'))
        return 0
    if arguments.jobs < 1:
        parser.error('--jobs must be positive')
    run = Run.create(arguments.root, 'synchronization', {'domain': 'Japan/USA any%', 'frames': 18000,
                                                      'original_labels': [272, 18000]})
    print('Run: ' + str(run.path), flush=True)
    print('Brief: ' + str(run.path / 'brief.md'), flush=True)
    try:
        failures = preflight(run)
        if failures:
            return run.finalize('error', 'preflight', '; '.join(failures), ['preflight', 'synchronization'])
        result = synchronization(run, arguments.original_run, jobs=arguments.jobs)
        return run.finalize(result['status'], 'synchronization', result.get('reason') or
                            'Fresh exact actual-fetch comparison completed; synchronization scope only',
                            ['preflight', 'synchronization'])
    except KeyboardInterrupt:
        return run.finalize('incomplete', 'synchronization', 'Independent synchronization interrupted')
    except Exception as error:
        failed = next((report for report in reversed(list(run.checks.values())) if report['status'] != 'pass'), None)
        return run.finalize(failed['status'] if failed else 'error', 'synchronization', str(error),
                            ['preflight', 'synchronization'])




def bind_capture_command(run, metadata, result, out, observer):
    """Bind the actual invocation's files and settings, retaining command text as evidence."""
    metadata['command'] = result['command']
    metadata['command_files'] = []
    inputs = [(out, metadata['checkpoint_sha256']), (observer, metadata['observer']['sha256']),
              (run.root / 'rom/sm.sfc', metadata['inputs']['rom_sha256']),
              (run.root / 'movies/super-metroid-any-3653.lsmv', metadata['inputs']['movie_sha256']),
              (run.path / 'generated/movie.inputs', metadata['inputs']['input_sha256']),
              (Path(metadata['provenance']['artifacts']['binary']), metadata['provenance']['binary_sha256'])]
    for path, identity in inputs:
        try:
            evidence = path.relative_to(run.path).as_posix()
        except ValueError:
            evidence = str(path.resolve())
        metadata['command_files'].append({'command_path': str(path), 'artifact': evidence, 'sha256': identity})
    metadata['owner_origin'] = str(run.path)


def verify_original_inventory(provenance, artifact):
    """Reconstruct pinned source plus declared build edits without inspecting semantics."""
    import tempfile
    import tarfile
    from setup_lsnes import configure
    from readiness import inventory_digest
    artifacts = provenance['artifacts']
    with tempfile.TemporaryDirectory(prefix='sm-original-origin-') as temporary:
        expected = Path(temporary)
        with tarfile.open(artifact(artifacts['source_tar'])) as archive:
            archive.extractall(expected)
        configure(expected, artifact(artifacts['source_tar']), Path(provenance['configured_libraries']))
        files = [{'path': path.relative_to(expected).as_posix(), 'sha256': digest(path)}
                 for path in sorted(expected.rglob('*')) if path.is_file()]
        if inventory_digest(files) != provenance['source_sha256']:
            raise ValueError('Original source inventory does not match official source plus declared build edits')


def portable_metadata(metadata, owner):
    """Convert owned evidence links to relative paths without altering command history."""
    def relative(value):
        path = Path(value)
        try:
            return path.relative_to(owner).as_posix()
        except ValueError:
            return value
    metadata['observer']['artifact'] = relative(metadata['observer']['artifact'])
    for name, value in metadata['provenance']['artifacts'].items():
        metadata['provenance']['artifacts'][name] = relative(value)
    for record in metadata['provenance'].get('build_dependencies', []):
        record['path'] = relative(record['path'])
    metadata['capture_report'] = relative(metadata['capture_report'])
    return metadata


def attest_original(root, previous):
    """Make a portable original-only attestation from proven captured artifacts.

    The previous whole run is never upgraded or edited. This new run independently
    verifies the official origin, declared build edits and exact capture process.
    """
    import shutil
    from readiness import Run, write_json
    previous = Path(previous).resolve()
    metadata = json.loads((previous / 'generated/original.capture.json').read_text())
    run = Run.create(root, 'original-attestation', {'frames': metadata['completed_frames'], 'checkpoint_pc': '828948'})
    try:
        run.capture_source()
        for relative in ('generated/original-build', 'generated/original.csv', 'generated/original.lua',
                         'generated/movie.inputs'):
            source, destination = previous / relative, run.path / relative
            if source.is_dir():
                shutil.copytree(source, destination, symlinks=True)
            else:
                shutil.copy2(source, destination)
        for stage in ('original-build', 'original-replay', 'original-compiler'):
            for suffix in ('.json', '.stdout.log', '.stderr.log'):
                shutil.copy2(previous / 'logs' / (stage + suffix), run.path / 'logs' / (stage + suffix))
        # Preserve the exact historical build recipe separately from current
        # maintained snapshot: differing harness code never relabels old source.
        recipe = run.path / 'generated/original-recipe.py'
        shutil.copy2(previous / 'source/files/tools/setup_lsnes.py', recipe)
        provenance = metadata['provenance']
        build_report = run.path / 'logs/original-build.json'
        # Preserve the observed report unchanged and bind its bytes in the new attestation.
        provenance['build_report_sha256'] = digest(build_report)
        provenance['build_command'] = json.loads(build_report.read_text())['command']
        provenance['configured_libraries'] = str(previous / 'generated/original-build/libraries')
        for name, value in provenance['artifacts'].items():
            path = Path(value)
            if path.is_absolute():
                try:
                    provenance['artifacts'][name] = path.relative_to(previous).as_posix()
                except ValueError:
                    pass
        provenance['artifacts']['recipe'] = 'generated/original-recipe.py'
        for record in provenance.get('build_dependencies', []):
            try:
                record['path'] = Path(record['path']).relative_to(previous).as_posix()
            except ValueError:
                pass
        metadata['capture_report'] = 'logs/original-replay.json'
        metadata['observer']['artifact'] = 'generated/original.lua'
        # Upgrade missing command binding only from the digest-verified process
        # report and verified inputs; never infer historical executable identity.
        report = json.loads((run.path / metadata['capture_report']).read_text())
        if digest(run.path / metadata['capture_report']) != metadata['capture_report_sha256']:
            raise ValueError('Original process report changed')
        metadata['command'] = report['command']
        metadata['owner_origin'] = str(previous)
        old_manifest = json.loads((previous / 'manifest.json').read_text())
        file_paths = [(previous / 'generated/original.csv', 'generated/original.csv', metadata['checkpoint_sha256']),
                      (previous / 'generated/original.lua', 'generated/original.lua', metadata['observer']['sha256']),
                      (Path(provenance['artifacts']['binary']), provenance['artifacts']['binary'], provenance['binary_sha256']),
                      (previous / 'generated/movie.inputs', 'generated/movie.inputs', metadata['inputs']['input_sha256']),
                      (Path(old_manifest['commands'][0]['cwd']) / 'rom/sm.sfc', str(Path(old_manifest['commands'][0]['cwd']) / 'rom/sm.sfc'), metadata['inputs']['rom_sha256']),
                      (Path(old_manifest['commands'][0]['cwd']) / 'movies/super-metroid-any-3653.lsmv', str(Path(old_manifest['commands'][0]['cwd']) / 'movies/super-metroid-any-3653.lsmv'), metadata['inputs']['movie_sha256'])]
        metadata['command_files'] = [{'command_path': str(path if path.is_absolute() else previous / path),
                                     'artifact': evidence, 'sha256': identity} for path, evidence, identity in file_paths]
        for record in provenance.get('packages', []):
            record['path'] = str(Path(record['path']).resolve())
        for record in provenance.get('build_dependencies', []):
            if Path(record['path']).is_absolute():
                record['path'] = str(Path(record['path']).resolve())
        for name in ('official_archive', 'source_tar'):
            if Path(provenance['artifacts'][name]).is_absolute():
                provenance['artifacts'][name] = str(Path(provenance['artifacts'][name]).resolve())
        for record in metadata['command_files']:
            if Path(record['artifact']).is_absolute():
                record['artifact'] = str(Path(record['artifact']).resolve())
        metadata['run_id'] = run.manifest['run_id']
        out = run.path / 'generated/original.csv'
        write_json(out.with_suffix('.capture.json'), metadata)
        verified_capture(out, 'original', metadata['inputs'], metadata['completed_frames'])
        run.manifest['identities']['inputs'] = metadata['inputs']
        run.manifest['identities']['original-oracle'] = {'sha256': digest(out), 'evidence': 'generated/original.csv'}
        run.manifest['attested_from'] = {'run_id': old_manifest['run_id'], 'path': str(previous),
                                        'capture_report_sha256': metadata['capture_report_sha256']}
        run.save_manifest()
        run.record_check('original-attestation', {'status': 'pass', 'complete': True,
                                                'completed_frames': metadata['completed_frames'],
                                                'events': metadata['event_count'], 'checkpoint_sha256': digest(out)})
        run.finalize('pass', 'original-attestation', 'Independently verified original-only capture; readiness remains pending', ['original-attestation'])
        return run.path
    except Exception as error:
        run.finalize('error', 'original-attestation', str(error))
        raise


if __name__ == '__main__':
    import sys
    sys.exit(main())
