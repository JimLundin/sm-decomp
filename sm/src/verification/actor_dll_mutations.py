"""Actual actor DLL faults extending the existing hosted room mutation runner."""
import ctypes
import hashlib
import json
import os
from pathlib import Path
import re
import runpy
import sys

import room_entry_mutations as entry

RECIPES = json.loads(Path(__file__).with_name('actor_dll_mutation_recipes.json').read_text(encoding='utf-8-sig'))
ROOM_PANICS = entry.GUARD_PANICS[:3]
INITIALIZER_PANIC = ('actor_initializer_checks.rs', 'actor initializer fixture deliberate nested dispatch unwind')
GUARD_PANICS = entry.GUARD_PANICS
EMPTY_HASH = hashlib.sha256(b'').hexdigest()


def capture_child(root, selected, rom=None):
    """Reuse source-stamped DLL loading; never build or keep a native host alive here."""
    if selected not in {m['selected'] for m in RECIPES}:
        raise RuntimeError('unknown actor mutation selected case')
    os.environ['RUST_BACKTRACE'] = '0'
    verifier = runpy.run_path(str(root / 'tools/verify_dll.py'))
    host = verifier['load_host'](root / 'target/verify-dll/sm_verify.dll')
    verifier['check_source'](host, verifier['source_hash'](root))
    verifier['check_library'](host, root)
    if selected == 'guard':
        function = host.sm_verify_room_runtime_guard
        function.argtypes = []
        function.restype = ctypes.c_bool
        observation = {'selected': selected, 'passed': bool(function())}
    else:
        case = int(selected.split(':')[1])
        output = ctypes.create_string_buffer(4 * 1024 * 1024)
        if selected.startswith('world:'):
            if rom is None or not Path(rom).is_file():
                raise RuntimeError('initialized actor mutation requires an existing --actor-rom path')
            function = host.sm_verify_room_world_case
            function.argtypes = [ctypes.c_char_p, ctypes.c_uint32, ctypes.c_void_p, ctypes.c_size_t]
            function.restype = ctypes.c_size_t
            size = function(os.fsencode(rom), case, output, len(output))
        else:
            function = host.sm_verify_actor_population_animation_case
            function.argtypes = [ctypes.c_uint32, ctypes.c_void_p, ctypes.c_size_t]
            function.restype = ctypes.c_size_t
            size = function(case, output, len(output))
        if size > len(output):
            raise RuntimeError('actor capture exceeds output capacity')
        data = output.raw[:size]
        if data:
            prefix = b'AKMC1' if selected.startswith('world:') else b'APAN1' + case.to_bytes(4, 'little')
            if not data.startswith(prefix) or (selected.startswith('world:') and len(data) != 131079):
                raise RuntimeError('wrong actor fixture capture header or size')
        observation = {'selected': selected, 'bytes': size, 'sha256': hashlib.sha256(data).hexdigest()}
    print(json.dumps(observation), flush=True)


def capture(copied, selected, run, rom=None):
    args = [sys.executable, '-B', str(Path(__file__).with_name('room_mutations.py').resolve()),
            '--actor-capture', str(copied), selected]
    if rom is not None:
        args.append(str(rom))
    result = run(args, copied, 30)
    if result.returncode != 0:
        raise RuntimeError(f'{selected}: child exit is not a kill\n{result.stdout}\n{result.stderr}')
    try:
        observation = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f'{selected}: invalid capture output\n{result.stdout}\n{result.stderr}') from error
    return observation, result.stderr


def baseline(copied, selected, run, rom=None):
    observation, stderr = capture(copied, selected, run, rom)
    panics = GUARD_PANICS if selected == 'guard' else ()
    if selected == 'guard':
        valid = observation == {'selected': selected, 'passed': True}
    else:
        valid = (observation.get('selected') == selected and observation.get('bytes', 0) > 0
                 and re.fullmatch('[0-9a-f]{64}', observation.get('sha256', '')) is not None)
    if not valid or not entry.stderr_matches(stderr, panics):
        raise RuntimeError(f'{selected}: unchanged selected baseline must pass\n{observation}\n{stderr}')
    return observation


def qualify(mutation, observation, stderr):
    selected, fault = mutation['selected'], mutation['fault']
    if selected == 'guard':
        expected = {'selected': selected, 'passed': False}
        prior = ROOM_PANICS if fault == 'F2' else ROOM_PANICS + (INITIALIZER_PANIC,) if fault in {'F5', 'F6'} else GUARD_PANICS
        footer = None
    else:
        expected = {'selected': selected, 'bytes': 0, 'sha256': EMPTY_HASH}
        prior = ()
        footer = ('Room world: room world assertions failed' if selected.startswith('world:')
                  else f"actor population/animation case {selected.split(':')[1]} assertion failed")
    wanted = prior + ((mutation['assertion_source'], mutation['assertion_message']),)
    if observation != expected or not entry.stderr_matches(stderr, wanted, footer):
        raise RuntimeError(f"{mutation['name']}: survival or unrelated failure, not intended assertion kill\n{observation}\n{stderr}")
    if mutation.get('expected_values'):
        left, right = mutation['expected_values']
        if not re.search(r'left: ' + re.escape(left) + r'\s+right: ' + re.escape(right), stderr):
            raise RuntimeError(f"{mutation['name']}: wrong assertion operands")


def build(root, work, paths, name, run, copy_tracked, edits=()):
    copied = work / name
    copy_tracked(root, copied, paths)
    for edit in edits:
        target = copied / edit['path']
        original = target.read_text(encoding='utf-8')
        if original.count(edit['before']) != 1:
            raise RuntimeError(f"{name}: unique production anchor missing in {edit['path']}")
        target.write_text(original.replace(edit['before'], edit['after'], 1), encoding='utf-8')
    result = run([sys.executable, '-B', 'tools/verify_dll.py', '--build-only', '--direct'], copied, 600)
    if result.returncode != 0:
        raise RuntimeError(f'{name}: compile failure is not a kill\n{result.stdout}\n{result.stderr}')
    return copied


def run_checks(root, work, paths, run, copy_tracked, rom=None, kraid_only=False):
    if os.environ.get('GITHUB_ACTIONS') != 'true' or os.name != 'nt':
        raise RuntimeError('actor DLL mutation builds require Windows GitHub CI')
    selected_mutations = [m for m in RECIPES if m['selected'].startswith('world:') == kraid_only]
    if not selected_mutations:
        raise RuntimeError('no actor mutation recipes selected')
    if kraid_only and (rom is None or not Path(rom).is_file()):
        raise RuntimeError('Kraid mutation suite requires --actor-rom PATH; no cases skipped or credited')
    required = {'sm/src/verification/actor_dll_mutations.py', 'sm/src/verification/actor_dll_mutation_recipes.json'}
    required.update(e['path'] for m in selected_mutations for e in m['edits'])
    if not required.issubset(paths):
        raise RuntimeError('actor mutation sources must be tracked')
    unchanged = work / 'baseline'
    observations = {}
    for mutation in selected_mutations:
        selected = mutation['selected']
        if selected not in observations:
            observations[selected] = baseline(unchanged, selected, run, rom)
    for mutation in selected_mutations:
        # Recheck the exact selected untouched baseline before its independent fault.
        if baseline(unchanged, mutation['selected'], run, rom) != observations[mutation['selected']]:
            raise RuntimeError('selected baseline capture changed between runs')
        copied = build(root, work, paths, mutation['name'], run, copy_tracked, mutation['edits'])
        observation, stderr = capture(copied, mutation['selected'], run, rom)
        qualify(mutation, observation, stderr)
        print(f"{mutation['name']}: KILLED by {mutation['required_assertion']}", flush=True)
    print(f'actor DLL mutations: {len(selected_mutations)}/{len(selected_mutations)} unique faults killed; selected baselines passed', flush=True)
    if not kraid_only:
        print('F9 maps to the detached actor_restore_reuses_live_epoch result; it is not a second independent mutant.', flush=True)
