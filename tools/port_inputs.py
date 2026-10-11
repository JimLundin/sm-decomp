"""Minimal canonical-memory wrappers and compiler-reported source inventories."""
import json
from pathlib import Path

from port_pipeline import ROOT, sha, within

MEMORY = ROOT / 'crates/port/src/memory.rs'


def wrapper_memory(run, modules, imports=()):
    from port_pipeline import invoke
    result = invoke('wrapper', {'checkout': str(ROOT), 'run': str(Path(run).resolve()),
                               'modules': [str(Path(p).resolve()) for p in modules], 'imports': list(imports)})
    return Path(result['path'])


def compiler_dependencies(wrapper, run, manifest):
    from port_pipeline import invoke
    return invoke('dependencies', {'checkout': str(ROOT), 'run': str(Path(run).resolve()),
                                   'extra': manifest.get('rust_dependencies', [])})


def resolve_source(path, implementation):
    path = Path(path)
    if implementation.get('dependency_contract') != 'compiler-v2':
        return path
    if path.is_absolute() and implementation.get('source_root'):
        old = Path(implementation['source_root'])
        if within(path, old):
            return ROOT / path.relative_to(old)
    return path if path.is_absolute() else ROOT / path


def verify_dependencies(implementation):
    if implementation.get('inputs_harness_sha256') and sha(ROOT / 'tools/port_inputs.py') != implementation['inputs_harness_sha256']:
        raise ValueError('compiler input harness changed')
    for name, expected in implementation.get('pipeline_sources', {}).items():
        if sha(ROOT / name) != expected:
            raise ValueError('Rust compiler input tool changed: ' + name)
    for name, expected in implementation['rust_dependencies'].items():
        path = resolve_source(name, implementation)
        if sha(path) != expected:
            raise ValueError(f'family Rust dependency changed: {path}')
    if implementation.get('dependency_contract') == 'compiler-v2':
        if 'crates/port/src/memory.rs' not in implementation['rust_dependencies']:
            raise ValueError('canonical memory missing from compiler closure')
        module = resolve_source(implementation['module'], implementation)
        key = str(module.relative_to(ROOT)) if within(module, ROOT) else str(module)
        if key not in implementation['rust_dependencies']:
            raise ValueError('candidate missing from compiler closure')


def memory_source(implementation):
    return MEMORY if implementation.get('dependency_contract') == 'compiler-v2' else ROOT / 'crates/port/src/lib.rs'
