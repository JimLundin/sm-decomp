#!/usr/bin/env python3
"""Authenticate retained live observations without claiming current-source parity."""
import argparse
import json
import shutil
import subprocess
from pathlib import Path

from port_live import stream_digest


def authenticate(verdict_path):
    verdict_path = verdict_path.resolve()
    run = verdict_path.parent
    verdict = json.loads(verdict_path.read_text())
    if not verdict.get('state_effects_match') or not verdict.get('artifacts'):
        raise ValueError('no finished matching observation inventory')
    if stream_digest(run / 'live') != verdict.get('binary_sha256'):
        raise ValueError('retained live binary changed')
    packed = verdict.get('evidence_storage')
    if packed:
        from port_evidence import authenticate_subject, SUBJECTS
        if packed.get('format') != 'sm-evidence-v1' or set(packed['subjects']) != set(SUBJECTS):
            raise ValueError('packed subjects incomplete')
        if stream_digest(run / 'sm-evidence') != verdict.get('evidence_reader_sha256'):
            raise ValueError('retained evidence reader changed')
        for subject in SUBJECTS:
            sealed = packed['subjects'][subject]
            pointer = authenticate_subject(run / subject, pointer_sha256=sealed['pointer_sha256'])
            expected = {k: v for k, v in pointer['artifacts'].items() if Path(k).name != 'calls.jsonl' and Path(k).suffix != '.rom'}
            if expected != verdict['artifacts'] or pointer['run'] != sealed['run'] or pointer['observations'] != sealed['observations']:
                raise ValueError('packed observations differ from verdict')
            if pointer['provenance']['binary_sha256'] != verdict['binary_sha256'] or pointer['provenance']['subject'] != subject:
                raise ValueError('packed producer provenance differs')
        return dict(status='authenticated', artifacts_per_subject=len(verdict['artifacts']),
                    subjects=len(SUBJECTS), binary=True, storage='sm-evidence-v1',
                    verdict_sha256=stream_digest(verdict_path), free_bytes=shutil.disk_usage(run).free,
                    scope='retained frozen binary and observations; current-source admission and original-ROM parity separate')
    subjects = ('c-0', 'c-1', 'mixed-0', 'mixed-1')
    for subject in subjects:
        root = run / subject
        for relative, expected in verdict['artifacts'].items():
            path = root / relative
            if path.is_symlink() or root.resolve() not in path.resolve().parents:
                raise ValueError(f'invalid observation path: {subject}/{relative}')
            if stream_digest(path) != expected:
                raise ValueError(f'observation changed: {subject}/{relative}')
        for expected, row in verdict.get('rom_fixture_cleanup', {}).get(subject, {}).get('retained_views', {}).items():
            path = root / row['retained']
            if path.is_symlink() or root.resolve() not in path.resolve().parents:
                raise ValueError('invalid retained ROM fixture path')
            if stream_digest(path) != expected:
                raise ValueError('retained ROM fixture changed')
    return dict(status='authenticated', artifacts_per_subject=len(verdict['artifacts']),
                subjects=len(subjects), binary=True, free_bytes=shutil.disk_usage(run).free,
                verdict_sha256=stream_digest(verdict_path),
                scope='retained frozen binary and observations; current-source admission and original-ROM parity separate')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('verdict', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    try:
        result = authenticate(args.verdict)
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        print(json.dumps(dict(status='rejected', error=str(error))))
        return 1
    payload = json.dumps(result, indent=2) + '\n'
    if args.output:
        args.output.write_text(payload)
    print(payload, end='')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
