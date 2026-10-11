#!/usr/bin/env python3
"""THROWAWAY disk-backed validation storage spike, never a production format.

One command: python3 tools/storage_spike.py benchmark --output <scratch directory>
Requires existing source captures and installed cc, libzstd, zlib and OpenSSL.
Objects live in one append-only pack; SQLite supplies the mutable physical index.
Run/observation manifests are canonical JSON, sealed independently of pack layout.
This single-writer prototype has no garbage collection or concurrency protocol.
"""
import argparse
from collections import OrderedDict
import hashlib
import json
import os
from pathlib import Path
import random
import shutil
import sqlite3
import statistics
import struct
import subprocess
import time

from storage_format_study import Zstd, read_varint

ROOT = Path(__file__).resolve().parents[1]
SOURCE_ROOT = Path('/home/jim/sm-decomp/out/port')
DEFAULT_CAPTURES = [SOURCE_ROOT / 'projectile-movement-live/20261011T061431.667466Z/c-0',
                    SOURCE_ROOT / 'room-plm-current-live/20261010T224032.887967Z/c-0']
NATIVE = Path(__file__).with_name('storage_spike_native.c')
CHUNK_HEADER = struct.Struct('<8Q96s')


def decode_events(data, first_frame=None):
    lines, offset, frame = [], 0, 0
    while offset < len(data):
        delta, offset = read_varint(data, offset)
        frame += -(delta//2)-1 if delta & 1 else delta//2
        if first_frame is not None:
            frame, first_frame = first_frame, None
        tag = data[offset]
        offset += 1
        if tag == 0:
            device, address, value = data[offset:offset+3]
            offset += 3
            lines.append(f'ppu {frame} {device} {address} {value}\n'.encode())
        elif tag == 1:
            address, value = struct.unpack_from('<IB', data, offset)
            offset += 5
            lines.append(f'apu {frame} {address} {value}\n'.encode())
        elif tag == 2:
            length, offset = read_varint(data, offset)
            if offset + length > len(data):
                raise ValueError('literal record truncation')
            lines.append(data[offset:offset+length])
            offset += length
        else:
            raise ValueError('unknown trace tag')
    return b''.join(lines)


def sha(payload):
    return hashlib.sha256(payload).digest()


def canonical(document):
    return json.dumps(document, sort_keys=True, separators=(',', ':')).encode()


def file_sha(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        while True:
            chunk = stream.read(4 * 1024**2)
            if not chunk:
                return digest.hexdigest()
            digest.update(chunk)


def timed(operation):
    wall, cpu = time.perf_counter(), time.process_time()
    value = operation()
    return value, {'wall_seconds': time.perf_counter() - wall,
                   'cpu_seconds': time.process_time() - cpu}


def merkle_root(page_hashes, length):
    layer = page_hashes
    while len(layer) > 1:
        layer = [sha(b'N' + struct.pack('<H', len(layer[i:i+16])) + b''.join(layer[i:i+16]))
                 for i in range(0, len(layer), 16)]
    return sha(b'M' + struct.pack('<Q', length) + layer[0])


class Store:
    """Compact physical IDs; canonical logical identities verified on reading."""
    def __init__(self, directory, create=False):
        self.directory = Path(directory)
        self.codec = Zstd()
        if create:
            self.directory.mkdir(parents=True, exist_ok=False)
        self.db = sqlite3.connect(str(self.directory / 'index.sqlite'))
        if create:
            self.db.executescript('''
                PRAGMA journal_mode=DELETE;
                PRAGMA synchronous=FULL;
                CREATE TABLE objects(id BLOB PRIMARY KEY,kind TEXT,offset INTEGER,
                  packed_len INTEGER,raw_len INTEGER,raw_hash BLOB,packed_hash BLOB);
                CREATE TABLE pages(id INTEGER PRIMARY KEY,hash BLOB UNIQUE,block BLOB,slot INTEGER);
                CREATE TABLE roots(id BLOB PRIMARY KEY,observation BLOB,manifest BLOB);
                CREATE TABLE frames(run BLOB,frame INTEGER,chunk BLOB,ordinal INTEGER);
                CREATE INDEX frame_lookup ON frames(run,frame);
            ''')
        self.pack = (self.directory / 'objects.pack').open('w+b' if create else 'r+b')
        self.cache = OrderedDict()
        self.manifest_cache = {}
        self.frame_lookup = {}
        self.page_lookup = {}
        self.page_rows = {}
        if not create:
            for identity, digest, block, slot in self.db.execute('SELECT id,hash,block,slot FROM pages'):
                self.page_rows[identity] = (digest, block, slot)
        self.pending_pages = []
        self.pending_ids = []
        self.next_page = self.db.execute('SELECT coalesce(max(id)+1,0) FROM pages').fetchone()[0]
        self.read_bytes = 0

    def close(self):
        self.pack.close()
        self.db.close()

    def sync(self):
        self.flush_pages()
        self.pack.flush()
        os.fsync(self.pack.fileno())
        self.db.commit()

    def put(self, kind, identity, raw, packed=None):
        if self.db.execute('SELECT 1 FROM objects WHERE id=?', (identity,)).fetchone():
            return identity
        if packed is None:
            packed = self.codec.compress(raw, 3)
        self.pack.seek(0, 2)
        offset = self.pack.tell()
        self.pack.write(packed)
        self.db.execute('INSERT INTO objects VALUES(?,?,?,?,?,?,?)',
                        (identity, kind, offset, len(packed), len(raw), sha(raw), sha(packed)))
        return identity

    def get(self, identity):
        if identity in self.cache:
            self.cache.move_to_end(identity)
            return self.cache[identity]
        row = self.db.execute('SELECT kind,offset,packed_len,raw_len,raw_hash,packed_hash '
                              'FROM objects WHERE id=?', (identity,)).fetchone()
        if row is None:
            raise ValueError('missing object')
        kind, offset, packed_len, raw_len, raw_hash, packed_hash = row
        if raw_len > 2 * 1024**2 or packed_len > 4 * 1024**2:
            raise ValueError('spike object length exceeds limit')
        self.pack.seek(offset)
        packed = self.pack.read(packed_len)
        self.read_bytes += len(packed)
        if len(packed) != packed_len or sha(packed) != packed_hash:
            raise ValueError('physical object seal differs')
        raw = self.codec.decompress(packed, raw_len)
        if sha(raw) != raw_hash:
            raise ValueError('decoded object seal differs')
        if kind in ('E', 'P') and sha(kind.encode() + raw) != identity:
            raise ValueError('content identity differs')
        self.cache[identity] = raw
        while len(self.cache) > 8:
            self.cache.popitem(last=False)
        return raw

    def page(self, page):
        found = self.page_lookup.get(page)
        if found is not None:
            return found
        digest = sha(b'P' + struct.pack('<H', len(page)) + page)
        row = self.db.execute('SELECT id FROM pages WHERE hash=?', (digest,)).fetchone()
        if row:
            identity = row[0]
        else:
            identity = self.next_page
            self.next_page += 1
            self.pending_pages.append(page.ljust(256, b'\0'))
            self.pending_ids.append((identity, digest))
            self.db.execute('INSERT INTO pages VALUES(?,?,NULL,NULL)', (identity, digest))
            if len(self.pending_pages) == 4096:
                self.flush_pages()
        self.page_lookup[page] = (identity, digest)
        return identity, digest

    def flush_pages(self):
        if not self.pending_pages:
            return
        payload = b''.join(self.pending_pages)
        identity = sha(b'P' + payload)
        self.put('P', identity, payload)
        for slot, (page_id, digest) in enumerate(self.pending_ids):
            self.db.execute('UPDATE pages SET block=?,slot=? WHERE id=?', (identity, slot, page_id))
            self.page_rows[page_id] = (digest, identity, slot)
        self.pending_pages.clear()
        self.pending_ids.clear()

    def snapshot(self, payload):
        pages = [self.page(payload[i:i+256]) for i in range(0, len(payload), 256)]
        root = merkle_root([p[1] for p in pages], len(payload))
        raw = struct.pack('<QI', len(payload), len(pages)) + b''.join(struct.pack('<I', p[0]) for p in pages)
        self.put('S', root, raw)
        return root

    def extract_snapshot(self, identity):
        raw = self.get(identity)
        length, count = struct.unpack_from('<QI', raw)
        if count != (length + 255) // 256 or len(raw) != 12 + count*4:
            raise ValueError('snapshot length/table differs')
        ids = struct.unpack_from('<' + 'I'*count, raw, 12)
        output, digests = bytearray(), []
        # Resolve each page block once per extraction. A small global LRU alone
        # would repeatedly decode scattered blocks when a member spans >8 blocks.
        blocks = {self.page_rows[p][1]: None for p in ids if p in self.page_rows}
        for block in blocks:
            blocks[block] = self.get(block)
        for index, page_id in enumerate(ids):
            if page_id not in self.page_rows:
                raise ValueError('missing page index')
            expected, block, slot = self.page_rows[page_id]
            content = blocks[block][slot*256:(slot+1)*256]
            actual_len = min(256, length - index*256)
            content = content[:actual_len]
            actual = sha(b'P' + struct.pack('<H', len(content)) + content)
            if actual != expected:
                raise ValueError('page content identity differs')
            output.extend(content)
            digests.append(actual)
        if merkle_root(digests, length) != identity:
            raise ValueError('snapshot root differs')
        return bytes(output)

    def import_trace(self, path):
        chunks, footer = [], None
        with path.open('rb') as stream:
            if stream.read(8) != b'SPTRACE1':
                raise ValueError('trace magic')
            while True:
                first_bytes = stream.read(8)
                if len(first_bytes) != 8:
                    raise ValueError('missing trace footer')
                first, = struct.unpack('<Q', first_bytes)
                if first == 2**64-1:
                    total, length, count = struct.unpack('<QQQ', stream.read(24))
                    digest = stream.read(32)
                    if len(digest) != 32 or stream.read(1) or count != len(chunks):
                        raise ValueError('trace completion')
                    footer = {'events': total, 'bytes': length, 'chunks': count, 'sha256': digest.hex()}
                    break
                fields = CHUNK_HEADER.unpack(first_bytes + stream.read(CHUNK_HEADER.size-8))
                first, last, sequence, count, text_len, raw_len, packed_len, frames, seals = fields
                if raw_len > 1024**2 + 256 or packed_len > 2*1024**2 or frames > 1024:
                    raise ValueError('trace chunk bounds')
                indices = [struct.unpack('<IIQ', stream.read(16)) for _ in range(frames)]
                packed = stream.read(packed_len)
                if len(packed) != packed_len or sha(packed) != seals[64:]:
                    raise ValueError('trace physical hash')
                raw = self.codec.decompress(packed, raw_len)
                if sha(b'E' + raw) != seals[:32]:
                    raise ValueError('trace canonical hash')
                identity = seals[:32]
                self.put('E', identity, raw, packed)
                chunks.append({'id': identity.hex(), 'first': first, 'last': last,
                               'sequence': sequence, 'events': count, 'legacy_bytes': text_len,
                               'legacy_sha256': seals[32:64].hex(),
                               'frames': [list(row) for row in indices]})
        if sum(c['events'] for c in chunks) != footer['events'] or sum(c['legacy_bytes'] for c in chunks) != footer['bytes']:
            raise ValueError('trace counts')
        return chunks, footer

    def publish(self, observations, provenance):
        observation_id = sha(b'O' + canonical(observations))
        manifest = {'schema': 'storage-spike-v1', 'complete': True,
                    'observation': observation_id.hex(), 'observations': observations,
                    'provenance': provenance}
        run_id = sha(b'R' + canonical(manifest))
        self.sync()  # Dependencies durable before root publication.
        self.db.execute('INSERT INTO roots VALUES(?,?,?)', (run_id, observation_id, canonical(manifest)))
        for chunk in observations['trace']['chunks']:
            for frame, offset, ordinal in chunk['frames']:
                self.db.execute('INSERT INTO frames VALUES(?,?,?,?)',
                                (run_id, frame, bytes.fromhex(chunk['id']), ordinal))
        self.db.commit()
        manifests = self.directory / 'runs'
        manifests.mkdir(exist_ok=True)
        (manifests / (run_id.hex() + '.json')).write_bytes(canonical(manifest))
        return run_id.hex(), observation_id.hex()

    def manifest(self, root):
        if root in self.manifest_cache:
            return self.manifest_cache[root]
        row = self.db.execute('SELECT manifest FROM roots WHERE id=?', (bytes.fromhex(root),)).fetchone()
        if row is None:
            raise ValueError('missing run')
        raw = row[0]
        if sha(b'R' + raw).hex() != root:
            raise ValueError('run seal differs')
        manifest = json.loads(raw)
        if not manifest['complete'] or sha(b'O' + canonical(manifest['observations'])).hex() != manifest['observation']:
            raise ValueError('observation seal differs')
        self.manifest_cache[root] = manifest
        lookup = {}
        for chunk in manifest['observations']['trace']['chunks']:
            for index, (frame, offset, ordinal) in enumerate(chunk['frames']):
                end = chunk['frames'][index+1][1] if index+1 < len(chunk['frames']) else None
                lookup.setdefault(frame, []).append((chunk['id'], offset, end))
        self.frame_lookup[root] = lookup
        return manifest

    def frame(self, root, frame):
        # Authenticate lookup against sealed manifest; mutable SQL is only a hint.
        manifest = self.manifest(root)
        output = []
        for identity, offset, end in self.frame_lookup[root].get(frame, []):
            raw = self.get(bytes.fromhex(identity))
            output.append(decode_events(raw[offset:end], first_frame=frame))
        return b''.join(output)

    def verify(self, root, snapshots=True):
        manifest = self.manifest(root)
        observations = manifest['observations']
        trace, expected_sequence = hashlib.sha256(), 0
        for chunk in observations['trace']['chunks']:
            if chunk['sequence'] != expected_sequence:
                raise ValueError('sequence continuity')
            raw = self.get(bytes.fromhex(chunk['id']))
            text = decode_events(raw)
            if sha(text).hex() != chunk['legacy_sha256'] or len(text) != chunk['legacy_bytes']:
                raise ValueError('legacy trace mismatch')
            if text.count(b'\n') != chunk['events']:
                raise ValueError('trace event count')
            trace.update(text)
            expected_sequence += chunk['events']
        if trace.hexdigest() != observations['trace']['sha256'] or expected_sequence != observations['trace']['events']:
            raise ValueError('trace completion hash')
        if snapshots:
            for member in observations['snapshots']:
                payload = self.extract_snapshot(bytes.fromhex(member['id']))
                if len(payload) != member['bytes'] or sha(payload).hex() != member['sha256']:
                    raise ValueError('snapshot legacy hash')
        return True


def compile_native(destination):
    subprocess.run(['cc', '-O3', '-Wno-deprecated-declarations', str(NATIVE),
                    '-Wl,-l:libzstd.so.1', '-lz', '-lcrypto', '-o', str(destination)], check=True)


def native_run(binary, args):
    result = subprocess.run([str(binary)] + list(map(str, args)), capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(result.stderr.strip())
    return json.loads(result.stdout)


def disk(directory):
    files = [p for p in Path(directory).rglob('*') if p.is_file()]
    return {'bytes': sum(p.stat().st_size for p in files),
            'allocated_bytes': sum(p.stat().st_blocks*512 for p in files),
            'files': len(files), 'by_file': {str(p.relative_to(directory)): p.stat().st_size for p in files}}


def source_sizes(paths):
    seen = set()
    logical, allocated = 0, 0
    for path in paths:
        stat = path.stat()
        logical += stat.st_size
        inode = stat.st_dev, stat.st_ino
        if inode not in seen:
            allocated += stat.st_blocks*512
            seen.add(inode)
    return {'logical_bytes': logical, 'allocated_bytes_unique_inodes': allocated, 'files': len(paths)}


def controls(destination, binary):
    directory = destination / 'controls'
    directory.mkdir()
    source = directory / 'events.log'
    original = b'ppu 0 1 4 7\nppu 0 1 4 8\nppu 0 1 4 8\napu 4 16777215 255\nppu 0 0 0 0\nroom-plm 2 47083 2056\n'
    source.write_bytes(original)
    packed = directory / 'events.pack'
    native_run(binary, ['convert', source, packed, 3])
    native_run(binary, ['verify', packed])
    store = Store(directory / 'store', create=True)
    chunks, footer = store.import_trace(packed)
    payload = bytes(range(256))*600 + b'partial-page'
    root = store.snapshot(payload)
    other = store.snapshot(payload[:-1] + b'X')
    store.sync()
    if root == other or store.extract_snapshot(root) != payload:
        raise ValueError('snapshot control')
    observation = {'trace': dict(footer, chunks=chunks), 'snapshots': [
        {'name': 'state', 'id': root.hex(), 'sha256': sha(payload).hex(), 'bytes': len(payload)}]}
    run, obs = store.publish(observation, {'producer': 'control-A'})
    run_b, obs_b = store.publish(observation, {'producer': 'control-B'})
    if run == run_b or obs != obs_b or not store.verify(run):
        raise ValueError('run/observation root control')
    if store.frame(run, 4) != b'apu 4 16777215 255\n':
        raise ValueError('frame extraction')
    store.cache.clear()
    page_row = store.db.execute('SELECT id,block FROM pages LIMIT 1').fetchone()
    removed = store.db.execute('SELECT * FROM objects WHERE id=?', (page_row[1],)).fetchone()
    store.db.execute('DELETE FROM objects WHERE id=?', (page_row[1],))
    try:
        store.verify(run)
    except ValueError:
        pass
    else:
        raise ValueError('missing dependency accepted')
    store.db.execute('INSERT INTO objects VALUES(?,?,?,?,?,?,?)', removed)
    store.close()
    data = packed.read_bytes()
    for name, bad in [('truncated', data[:-1]), ('corrupt', data[:200] + bytes([data[200]^1]) + data[201:])]:
        path = directory / name
        path.write_bytes(bad)
        result = subprocess.run([str(binary), 'verify', str(path)], capture_output=True)
        if result.returncode == 0:
            raise ValueError(name + ' accepted')
    # Ending at a complete chunk but omitting the authenticated footer must fail.
    fields = CHUNK_HEADER.unpack(data[8:8+CHUNK_HEADER.size])
    boundary = 8 + CHUNK_HEADER.size + fields[7]*16 + fields[6]
    path = directory / 'no-footer'
    path.write_bytes(data[:boundary])
    if subprocess.run([str(binary), 'verify', str(path)], capture_output=True).returncode == 0:
        raise ValueError('complete-chunk truncation accepted')
    return {'exact_round_trip': True, 'duplicates_resets': True,
            'different_run_equal_observations': True, 'missing_dependency_rejected': True,
            'corruption_truncation_missing_footer_rejected': True,
            'partial_page_and_changed_snapshot': True}


def benchmark(args):
    output = args.output.resolve()
    if output.exists():
        raise ValueError('output must be a new directory')
    captures = args.capture_root or DEFAULT_CAPTURES
    for source in captures:
        if source.resolve() == output or source.resolve() in output.parents:
            raise ValueError('output must be outside source captures')
    output.mkdir(parents=True)
    build = output / 'native-writer'
    compile_native(build)
    report = {'schema': 'storage-spike-benchmark-v1', 'prototype': True,
              'system_zstd': Zstd().version, 'controls': controls(output, build),
              'runs': [], 'native_writer': []}
    sample = output / 'trace-sample.log'
    with (captures[0] / 'hardware.log').open('rb') as source:
        source.seek((captures[0] / 'hardware.log').stat().st_size // 2)
        source.readline()
        payload = source.read(16*1024**2)
        sample.write_bytes(payload[:payload.rfind(b'\n')+1])
    for repeat in range(3):
        for mode in ('null', 'plain', 'gzip', 'zstd-text', 'binary'):
            dest = output / ('writer-' + mode)
            row = native_run(build, ['bench', sample, mode, dest, args.events, 3])
            row['repeat'] = repeat
            report['native_writer'].append(row)
            print('Native writer ' + json.dumps(row), flush=True)
            dest.unlink()
    store = Store(output / 'store', create=True)
    for number, source in enumerate(captures):
        trace_path = source / 'hardware.log'
        if not trace_path.is_file():
            raise ValueError('spike requires plain decimal native trace')
        converted = output / ('trace-' + str(number) + '.pack')
        conversion = native_run(build, ['convert', trace_path, converted, 3])
        verification = native_run(build, ['verify', converted])
        before = disk(store.directory)
        (chunks, footer), import_cost = timed(lambda: store.import_trace(converted))
        verdict_path = source.parent / 'verdict.json'
        verdict_bytes = verdict_path.read_bytes()
        verdict = json.loads(verdict_bytes)
        if conversion['source_sha256'] != verdict['artifacts']['hardware.log']:
            raise ValueError('source trace seal differs')
        paths = sorted((source / 'fixtures').glob('*.wram')) + sorted(source.glob('frame-*.bin'))
        members = []
        def pack_snapshots():
            for path in paths:
                payload = path.read_bytes()
                relative = str(path.relative_to(source))
                digest = sha(payload).hex()
                if digest != verdict['artifacts'][relative]:
                    raise ValueError('source snapshot seal differs')
                root = store.snapshot(payload)
                members.append({'name': relative, 'id': root.hex(), 'bytes': len(payload),
                                'sha256': digest, 'layout': 'opaque-native-v1'})
            store.sync()
        _, snapshot_cost = timed(pack_snapshots)
        observations = {'trace': dict(footer, chunks=chunks), 'snapshots': members,
                        'requested_frames': verdict.get('frames'), 'profile': 'native-decimal-v1'}
        (run_root, observation_root), publish_cost = timed(lambda: store.publish(observations, {
            'source_verdict_sha256': sha(verdict_bytes).hex(), 'source': str(source),
            'producer_sha256': verdict.get('binary_sha256'), 'source_status': verdict.get('status')}))
        if verdict_path.read_bytes() != verdict_bytes:
            raise ValueError('source verdict changed')
        after = disk(store.directory)
        row = {'source': str(source), 'source_size': source_sizes([trace_path] + paths),
               'native_conversion': conversion, 'native_trace_verify': verification,
               'converted_trace_bytes': converted.stat().st_size,
               'import_trace': import_cost, 'snapshot_pack': snapshot_cost,
               'publish': publish_cost, 'run_root': run_root, 'observation_root': observation_root,
               'store_added_bytes': after['bytes']-before['bytes'],
               'store_added_allocated_bytes': after['allocated_bytes']-before['allocated_bytes'],
               'store_after': after, 'snapshots': len(paths),
               'trace_chunks': len(chunks), 'events': footer['events']}
        report['runs'].append(row)
        print('Packed run ' + json.dumps({k:v for k,v in row.items() if k!='store_after'}), flush=True)
        converted.unlink()  # Unsealed conversion intermediate, imported and verified.
    store.sync()
    store.close()
    # Reopen: no writer cache or in-memory object dictionary needed to read.
    store = Store(output / 'store')
    for row in report['runs']:
        manifest = store.manifest(row['run_root'])
        snapshots = manifest['observations']['snapshots']
        frame_members = [m for m in snapshots if m['name'].startswith('frame-')]
        rng = random.Random(7)
        read_samples, frame_samples = [], []
        for member in rng.sample(frame_members, min(10, len(frame_members))):
            store.cache.clear()
            before_bytes = store.read_bytes
            payload, cost = timed(lambda: store.extract_snapshot(bytes.fromhex(member['id'])))
            if sha(payload).hex() != member['sha256']:
                raise ValueError('readback differs')
            _, plain_cost = timed(lambda: (Path(row['source']) / member['name']).read_bytes())
            read_samples.append(dict(cost, bytes=member['bytes'], compressed_read_bytes=store.read_bytes-before_bytes,
                                     plain_read_seconds=plain_cost['wall_seconds']))
        frames = sorted(set(index[0] for c in manifest['observations']['trace']['chunks'] for index in c['frames']))
        for frame in rng.sample(frames, min(10, len(frames))):
            store.cache.clear()
            before_bytes = store.read_bytes
            payload, cost = timed(lambda: store.frame(row['run_root'], frame))
            frame_samples.append(dict(cost, frame=frame, decoded_text_bytes=len(payload),
                                      compressed_read_bytes=store.read_bytes-before_bytes))
        row['snapshot_random_read'] = read_samples
        row['trace_frame_random_read'] = frame_samples
        def verify_snapshots():
            for member in snapshots:
                payload = store.extract_snapshot(bytes.fromhex(member['id']))
                if sha(payload).hex() != member['sha256']:
                    raise ValueError('snapshot verification')
            return True
        _, row['snapshot_verify_all'] = timed(verify_snapshots)
        row['snapshot_verify_all']['logical_bytes'] = sum(m['bytes'] for m in snapshots)
        def verify_stored_trace_objects():
            for chunk in manifest['observations']['trace']['chunks']:
                store.get(bytes.fromhex(chunk['id']))
            return True
        _, row['trace_object_verify_all'] = timed(verify_stored_trace_objects)
        row['trace_object_verify_all']['note'] = 'hashes every stored chunk; original exact text verified by native decoder before import'
        print('Readback run ' + row['run_root'][:12], flush=True)
    report['store'] = disk(store.directory)
    report['store']['pack_sha256'] = file_sha(store.directory / 'objects.pack')
    report['store']['index_sha256'] = file_sha(store.directory / 'index.sqlite')
    report['store']['object_count'] = store.db.execute('SELECT count(*) FROM objects').fetchone()[0]
    report['store']['page_count'] = store.db.execute('SELECT count(*) FROM pages').fetchone()[0]
    report['store']['objects_by_kind'] = dict(store.db.execute('SELECT kind,count(*) FROM objects GROUP BY kind'))
    store.close()
    report['peak_rss_kib'] = __import__('resource').getrusage(__import__('resource').RUSAGE_SELF).ru_maxrss
    report['native_peak_rss_kib'] = __import__('resource').getrusage(__import__('resource').RUSAGE_CHILDREN).ru_maxrss
    report['cleanup_intermediates_bytes'] = 0
    for path in (build, sample, output / 'controls'):
        if path.is_dir():
            report['cleanup_intermediates_bytes'] += disk(path)['bytes']
            shutil.rmtree(path)
        else:
            report['cleanup_intermediates_bytes'] += path.stat().st_size
            path.unlink()
    (output / 'benchmark.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Results: ' + str(output / 'benchmark.json'), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    bench = commands.add_parser('benchmark')
    bench.add_argument('--output', type=Path, required=True)
    bench.add_argument('--capture-root', type=Path, action='append')
    bench.add_argument('--events', type=int, default=5000000)
    verify = commands.add_parser('verify')
    verify.add_argument('--store', type=Path, required=True)
    verify.add_argument('--run', required=True)
    extract = commands.add_parser('extract')
    extract.add_argument('--store', type=Path, required=True)
    extract.add_argument('--run', required=True)
    extract.add_argument('--member', required=True)
    extract.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.command == 'benchmark':
        benchmark(args)
    elif args.command == 'verify':
        store = Store(args.store)
        try:
            print(json.dumps({'verified': store.verify(args.run)}))
        finally:
            store.close()
    else:
        store = Store(args.store)
        try:
            manifest = store.manifest(args.run)
            member = next(m for m in manifest['observations']['snapshots'] if m['name'] == args.member)
            payload = store.extract_snapshot(bytes.fromhex(member['id']))
            if sha(payload).hex() != member['sha256']:
                raise ValueError('member hash')
            with args.output.open('xb') as output:
                output.write(payload)
        finally:
            store.close()


if __name__ == '__main__':
    main()
