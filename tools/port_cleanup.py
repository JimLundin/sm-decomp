#!/usr/bin/env python3
"""Preview/remove unsealed Cargo intermediates from finished evidence runs.

Never removes fixtures, verdicts, pointers, binaries or sealed build files.
Unfinished runs and symlinked directories are skipped. Existing evidence is
immutable: an inventory entry anywhere under a target directory protects it.
"""
import argparse
import json
import os
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]


def inventories(document):
    if isinstance(document, dict):
        for key, value in document.items():
            if key in ('artifacts', 'inventory') and isinstance(value, dict):
                yield from value
            else:
                yield from inventories(value)
    elif isinstance(document, list):
        for value in document:
            yield from inventories(value)


def candidates(root):
    documents = []
    targets = []
    for directory, dirs, files in os.walk(root, followlinks=False):
        base = Path(directory)
        dirs[:] = [name for name in dirs if not (base / name).is_symlink()]
        if 'target' in dirs:
            targets.append(base / 'target')
            dirs.remove('target')
        for name in files:
            if not name.endswith('.json') or (base / name).is_symlink():
                continue
            try:
                documents.append((base, json.loads((base / name).read_text())))
            except (OSError, ValueError):
                # An unreadable manifest makes cleanup unsafe.
                return []
    result = []
    for target in targets:
        verdict = target.parent / 'verdict.json'
        if not verdict.is_file() or verdict.is_symlink():
            continue
        data = json.loads(verdict.read_text())
        if not isinstance(data, dict) or data.get('status') not in ('accepted', 'offline-verified', 'mixed-C-Rust-integration', 'rejected', 'failed'):
            continue
        protected = False
        for base, document in documents:
            if not isinstance(document, dict):
                continue
            for name in inventories(document):
                path = Path(os.path.abspath(base / name))
                if path == target or target in path.parents:
                    protected = True
        if not protected:
            result.append(target)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT / 'out', help='Evidence tree to scan')
    parser.add_argument('--apply', action='store_true', help='Remove listed unsealed build directories')
    args = parser.parse_args()
    root = args.root.absolute()
    if root.is_symlink() or not root.is_dir():
        parser.error('root must be an existing real directory')
    total = 0
    for target in candidates(root):
        size = sum(path.stat().st_size for path in target.rglob('*') if path.is_file() and not path.is_symlink())
        total += size
        print(f'{"remove" if args.apply else "would remove"}: {target} ({size} bytes)')
        if args.apply:
            # Recheck inventories immediately before each deletion.
            if target not in candidates(root):
                raise RuntimeError(f'cleanup eligibility changed: {target}')
            shutil.rmtree(target)
    print(f'{"reclaimed" if args.apply else "reclaimable"}: {total} bytes')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
