#!/usr/bin/env python3
"""Compare packed Merkle objects and flat page IDs; not a production store.

Reads WRAM and full frame snapshots, retains only small JSON measurements.
Merkle objects use SHA-256 IDs, 256-byte leaves and fanout 16. The optimistic
physical pack is one zstd frame of objects sorted by hash, with u32 lengths.
This excludes a seek index, per-run catalog and actual filesystem allocation.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

from storage_format_study import Zstd


def digest(payload):
    return hashlib.sha256(payload).digest()


class Merkle:
    def __init__(self):
        self.objects = {}

    def put(self, payload):
        identity = digest(payload)
        if identity in self.objects and self.objects[identity] != payload:
            raise ValueError('hash collision')
        self.objects[identity] = payload
        return identity

    def store(self, data):
        layer = [self.put(b'P' + struct.pack('<H', len(data[i:i+256])) + data[i:i+256])
                 for i in range(0, len(data), 256)]
        if not layer:
            raise ValueError('empty snapshot unsupported in this experiment')
        while len(layer) > 1:
            layer = [self.put(b'N' + struct.pack('<H', len(layer[i:i+16]))
                              + b''.join(layer[i:i+16])) for i in range(0, len(layer), 16)]
        return self.put(b'M' + struct.pack('<Q', len(data)) + layer[0])

    def restore(self, identity):
        payload = self.objects[identity]
        if digest(payload) != identity:
            raise ValueError('object hash differs')
        if payload[:1] == b'P':
            length, = struct.unpack('<H', payload[1:3])
            if len(payload) != length + 3:
                raise ValueError('page length differs')
            return payload[3:]
        if payload[:1] == b'M':
            length, = struct.unpack('<Q', payload[1:9])
            data = self.restore(payload[9:])
            if len(data) != length:
                raise ValueError('snapshot length differs')
            return data
        count, = struct.unpack('<H', payload[1:3])
        if payload[:1] != b'N' or len(payload) != 3 + 32 * count:
            raise ValueError('node length differs')
        return b''.join(self.restore(payload[i:i+32]) for i in range(3, len(payload), 32))

    def stats(self, codec):
        payload = b''.join(identity + struct.pack('<I', len(value)) + value
                           for identity, value in sorted(self.objects.items()))
        packed = codec.compress(payload, 3)
        if codec.decompress(packed, len(payload)) != payload:
            raise ValueError('Merkle object pack round trip differs')
        return {'objects': len(self.objects),
                'pages': sum(p[:1] == b'P' for p in self.objects.values()),
                'nodes': sum(p[:1] == b'N' for p in self.objects.values()),
                'snapshot_roots': sum(p[:1] == b'M' for p in self.objects.values()),
                'object_payload_bytes': sum(map(len, self.objects.values())),
                'pack_bytes_including_object_ids_lengths': len(packed),
                'single_pack_uncompressed_bytes': len(payload)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--capture-root', type=Path, action='append', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    for root in args.capture_root:
        if root.resolve() == args.output.resolve() or root.resolve() in args.output.resolve().parents:
            parser.error('output must be outside source captures')
    codec = Zstd()
    merkle = Merkle()
    pages, page_ids, flat_tables, roots = [], {}, [], []
    report = {'schema': 'merkle-estimate-v2', 'page_bytes': 256, 'fanout': 16,
              'zstd_version': codec.version, 'compression_level': 3,
              'scope': ('all WRAM/full frame snapshots, no traces; whole-object pack; '
                        'physical seek index and run catalog excluded'), 'datasets': []}
    for root in args.capture_root:
        for label, paths in (
            ('wram', sorted((root / 'fixtures').glob('*.wram'))),
            ('frames', sorted(root.glob('frame-*.bin'))),
        ):
            before = merkle.stats(codec)
            raw = 0
            for path in paths:
                data = path.read_bytes()
                identity = merkle.store(data)
                if merkle.restore(identity) != data:
                    raise ValueError('Merkle snapshot round trip differs')
                ids = []
                for offset in range(0, len(data), 256):
                    page = data[offset:offset+256].ljust(256, b'\0')
                    if page not in page_ids:
                        page_ids[page] = len(pages)
                        pages.append(page)
                    ids.append(page_ids[page])
                if b''.join(pages[i] for i in ids)[:len(data)] != data:
                    raise ValueError('flat page snapshot round trip differs')
                flat_tables.append(struct.pack('<QI', len(data), len(ids))
                                   + b''.join(struct.pack('<I', i) for i in ids))
                roots.append({'path': str(path), 'root': identity.hex(),
                              'logical_sha256': digest(data).hex()})
                raw += len(data)
            after = merkle.stats(codec)
            row = {'run': str(root), 'kind': label, 'files': len(paths),
                   'logical_bytes': raw, 'new_objects': after['objects'] - before['objects'],
                   'new_pack_bytes': after['pack_bytes_including_object_ids_lengths']
                                    - before['pack_bytes_including_object_ids_lengths'],
                   'cumulative': after}
            report['datasets'].append(row)
            print(json.dumps(row), flush=True)
    flat = b''.join(pages) + b''.join(flat_tables)
    flat_packed = codec.compress(flat, 3)
    if codec.decompress(flat_packed, len(flat)) != flat:
        raise ValueError('flat page pack round trip differs')
    report.update(roots_sha256=digest(json.dumps(roots, sort_keys=True).encode()).hex(),
                  totals=merkle.stats(codec),
                  global_flat_pages={'unique_pages': len(pages),
                                     'raw_bytes': len(flat), 'zstd_bytes': len(flat_packed)},
                  round_trip=True)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
