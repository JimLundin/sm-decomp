#!/usr/bin/env python3
"""Preview/remove unsealed Cargo intermediates from finished evidence runs.

Never removes fixtures, verdicts, pointers, binaries or sealed build files.
Unfinished runs and symlinked directories are skipped. Existing evidence is
immutable: an inventory entry anywhere under a target directory protects it.
"""
import argparse
import hashlib
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
        if 'verdict.json' in files:
            for name in ('libfamilies.a', 'librng.a'):
                if name in files and not (base / name).is_symlink():
                    targets.append(base / name)
        for name in files:
            if name not in ('verdict.json', 'acceptance.json') or (base / name).is_symlink():
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
        if target.name != 'target' and data.get('status') != 'mixed-C-Rust-integration':
            continue
        if not protected:
            result.append(target)
    return result


def deduplicate(root, apply):
    """Share storage only for authenticated immutable snapshot bytes."""
    seen = {}
    reclaimed = 0
    for directory, dirs, files in os.walk(root, followlinks=False):
        base = Path(directory)
        dirs[:] = [name for name in dirs if name != 'target' and not (base / name).is_symlink()]
        if 'verdict.json' not in files:
            continue
        try:
            verdict = json.loads((base / 'verdict.json').read_text())
        except (OSError, ValueError):
            continue
        if verdict.get('status') not in ('accepted', 'offline-verified', 'mixed-C-Rust-integration'):
            continue
        inventory = verdict.get('artifacts', {})
        if not isinstance(inventory, dict):
            continue
        for relative, expected in inventory.items():
            source = base / relative
            choices = [source]
            if verdict.get('status') == 'mixed-C-Rust-integration':
                choices = [base / subject / relative for subject in ('c-0', 'c-1', 'mixed-0', 'mixed-1')]
            for path in choices:
                if not (path.suffix == '.wram' or path.name == 'terminal.bin' or path.name == 'rom-view.bin' or path.name.startswith('frame-') and path.suffix == '.bin'):
                    continue
                if not path.is_file() or path.is_symlink() or base not in path.absolute().parents:
                    continue
                payload = path.read_bytes()
                identity = hashlib.sha256(payload).hexdigest()
                if identity != expected:
                    continue
                stat = path.stat()
                key = (stat.st_dev, len(payload), identity)
                previous = seen.get(key)
                if previous is None:
                    seen[key] = path
                    continue
                if previous.stat().st_ino == stat.st_ino:
                    continue
                if previous.read_bytes() != payload:
                    continue
                reclaimed += stat.st_size
                if apply:
                    # Files belong to finished immutable runs; recheck before an
                    # atomic replacement so interruption never loses a snapshot.
                    if path.read_bytes() != payload:
                        raise RuntimeError(f'snapshot changed: {path}')
                    temporary = path.with_name(path.name + '.dedup-link')
                    os.link(previous, temporary)
                    os.replace(temporary, path)
    return reclaimed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT / 'out', help='Evidence tree to scan')
    parser.add_argument('--deduplicate', action='store_true', help='Share storage for identical sealed immutable snapshots')
    parser.add_argument('--apply', action='store_true', help='Remove listed unsealed build directories')
    args = parser.parse_args()
    root = args.root.absolute()
    if root.is_symlink() or not root.is_dir():
        parser.error('root must be an existing real directory')
    if args.deduplicate:
        total = deduplicate(root, args.apply)
        print(f'{"reclaimed" if args.apply else "reclaimable"}: {total} snapshot bytes')
        return 0
    total = 0
    for target in candidates(root):
        size = target.stat().st_size if target.is_file() else sum(path.stat().st_size for path in target.rglob('*') if path.is_file() and not path.is_symlink())
        total += size
        print(f'{"remove" if args.apply else "would remove"}: {target} ({size} bytes)')
        if args.apply:
            # Recheck inventories immediately before each deletion.
            if target not in candidates(root):
                raise RuntimeError(f'cleanup eligibility changed: {target}')
            if target.is_dir():
                shutil.rmtree(target)
            else:
                target.unlink()
    print(f'{"reclaimed" if args.apply else "reclaimable"}: {total} bytes')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
