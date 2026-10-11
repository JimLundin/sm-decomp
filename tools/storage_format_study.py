#!/usr/bin/env python3
"""Read-only, bounded storage experiments; not a production artifact format.

Uses the installed libzstd through ctypes solely for measurement. All candidates
are decoded and compared with original bytes. Saves JSON measurements, never
packed captures. Historical verdicts and artifacts are never changed.
"""
import argparse
import ctypes
import ctypes.util
import gzip
import hashlib
import json
from pathlib import Path
import statistics
import struct
import time


class Zstd:
    def __init__(self):
        self.lib = ctypes.CDLL(ctypes.util.find_library('zstd'))
        for name, args, result in (
            ('ZSTD_compressBound', [ctypes.c_size_t], ctypes.c_size_t),
            ('ZSTD_compress', [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p,
                               ctypes.c_size_t, ctypes.c_int], ctypes.c_size_t),
            ('ZSTD_decompress', [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p,
                                 ctypes.c_size_t], ctypes.c_size_t),
            ('ZSTD_isError', [ctypes.c_size_t], ctypes.c_uint),
            ('ZSTD_getErrorName', [ctypes.c_size_t], ctypes.c_char_p),
            ('ZSTD_versionString', [], ctypes.c_char_p),
        ):
            fn = getattr(self.lib, name)
            fn.argtypes, fn.restype = args, result
        self.version = self.lib.ZSTD_versionString().decode()

    def checked(self, size):
        if self.lib.ZSTD_isError(size):
            raise ValueError(self.lib.ZSTD_getErrorName(size).decode())
        return size

    def compress(self, data, level=3):
        out = ctypes.create_string_buffer(self.lib.ZSTD_compressBound(len(data)))
        size = self.checked(self.lib.ZSTD_compress(out, len(out), data, len(data), level))
        return out.raw[:size]

    def decompress(self, data, length):
        out = ctypes.create_string_buffer(length)
        size = self.checked(self.lib.ZSTD_decompress(out, length, data, len(data)))
        if size != length:
            raise ValueError('decoded length differs')
        return out.raw[:size]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def varint(value):
    if not 0 <= value < 2**64:
        raise ValueError('value outside unsigned 64-bit range')
    result = bytearray()
    while value >= 128:
        result.append((value & 127) | 128)
        value >>= 7
    result.append(value)
    return result


def read_varint(data, offset):
    result = 0
    for shift in range(0, 70, 7):
        if offset >= len(data):
            raise ValueError('truncated varint')
        value = data[offset]
        offset += 1
        if shift == 63 and value > 1:
            raise ValueError('overflow varint')
        result |= (value & 127) << shift
        if not value & 128:
            return result, offset
    raise ValueError('overflow varint')


def encode_events(text):
    result = bytearray()
    previous = 0
    count = 0
    for line in text.splitlines(keepends=True):
        columns = line.split()
        tag = columns[0]
        frame = int(columns[1])
        # Signed deltas also preserve resets; no monotonic-frame assumption.
        delta = frame - previous
        result.extend(varint(delta * 2 if delta >= 0 else -delta * 2 - 1))
        if tag == b'ppu' and len(columns) == 5:
            result.append(0)
            result.extend(bytes(map(int, columns[2:])))
        elif tag == b'apu' and len(columns) == 4:
            result.append(1)
            result.extend(struct.pack('<IB', int(columns[2]), int(columns[3])))
        else:
            raise ValueError('unsupported event grammar')
        previous = frame
        count += 1
    return bytes(result), count


def decode_events(data):
    offset = 0
    frame = 0
    lines = []
    while offset < len(data):
        delta, offset = read_varint(data, offset)
        frame += delta // 2 if delta % 2 == 0 else -(delta // 2) - 1
        tag = data[offset]
        offset += 1
        if tag == 0:
            if offset + 3 > len(data):
                raise ValueError('truncated ppu record')
            device, address, value = data[offset:offset + 3]
            offset += 3
            lines.append(f'ppu {frame} {device} {address} {value}\n'.encode())
        elif tag == 1:
            address, value = struct.unpack_from('<IB', data, offset)
            offset += 5
            lines.append(f'apu {frame} {address} {value}\n'.encode())
        else:
            raise ValueError('unknown tag')
    return b''.join(lines)


def codecs(data, zstd):
    rows = {}
    for codec in ('gzip-1', 'zstd-1', 'zstd-3', 'zstd-9'):
        start = time.perf_counter()
        packed = (gzip.compress(data, compresslevel=1, mtime=0) if codec == 'gzip-1'
                  else zstd.compress(data, int(codec.split('-')[1])))
        compressed_at = time.perf_counter()
        decoded = (gzip.decompress(packed) if codec == 'gzip-1'
                   else zstd.decompress(packed, len(data)))
        finished = time.perf_counter()
        if decoded != data:
            raise ValueError('codec round trip differs')
        rows[codec] = {'bytes': len(packed), 'ratio': len(data) / max(1, len(packed)),
                       'compress_seconds': compressed_at - start,
                       'decompress_seconds': finished - compressed_at}
    return rows


def trace_sample(path, fraction, size):
    with path.open('rb') as stream:
        stream.seek(int(path.stat().st_size * fraction))
        if fraction:
            stream.readline()  # Skip the possible partial first record.
        offset = stream.tell()
        text = stream.read(size)
    end = text.rfind(b'\n') + 1
    return offset, text[:end]


def trace_measure(path, fraction, size, zstd):
    offset, text = trace_sample(path, fraction, size)
    start = time.perf_counter()
    binary, count = encode_events(text)
    encoded_at = time.perf_counter()
    if decode_events(binary) != text:
        raise ValueError('event grammar is not byte-exact')
    decoded_at = time.perf_counter()
    result = {'path': str(path), 'offset': offset, 'source_file_bytes': path.stat().st_size,
              'text_bytes': len(text), 'text_sha256': sha(text), 'records': count,
              'binary_bytes': len(binary), 'encode_seconds': encoded_at - start,
              'decode_seconds': decoded_at - encoded_at, 'text_codecs': codecs(text, zstd),
              'binary_codecs': codecs(binary, zstd), 'round_trip': True}
    # Frame-oriented chunks, capped at 1 MiB text input; a large emulated frame
    # can span multiple chunks. Each binary chunk starts with an absolute frame
    # (delta from zero), so no preceding chunk is needed to decode it.
    chunks = []
    current = bytearray()
    group = None
    for line in text.splitlines(keepends=True):
        frame_group = int(line.split()[1]) // 64
        if current and (frame_group != group or len(current) >= 1024**2):
            chunks.append(bytes(current))
            current.clear()
        current.extend(line)
        group = frame_group
    if current:
        chunks.append(bytes(current))
    total = 0
    text_total = 0
    start = time.perf_counter()
    for chunk in chunks:
        encoded, _ = encode_events(chunk)
        packed = zstd.compress(encoded)
        decoded = zstd.decompress(packed, len(encoded))
        if decode_events(decoded) != chunk:
            raise ValueError('chunk event round trip differs')
        total += len(packed)
        plain_packed = zstd.compress(chunk)
        if zstd.decompress(plain_packed, len(chunk)) != chunk:
            raise ValueError('chunk text round trip differs')
        text_total += len(plain_packed)
    result['chunked_zstd_3'] = {
        'chunks': len(chunks), 'frames_per_group': 64, 'text_input_cap': 1024**2,
        'binary_bytes': total, 'text_bytes': text_total,
        'binary_ratio': len(text) / total, 'text_ratio': len(text) / text_total,
        'index_bytes_estimate': len(chunks) * 64,
        'round_trip_seconds': time.perf_counter() - start}
    with path.open('rb') as stream:
        stream.seek(offset)
        if stream.read(len(text)) != text:
            raise ValueError('source trace changed during experiment')
    print(f'Trace {path.name} at {fraction:.0%}: {len(text)} bytes, {count} records', flush=True)
    return result


def manifests(paths):
    return json.dumps([{'name': path.name, 'bytes': path.stat().st_size,
                        'sha256': sha(path.read_bytes())} for path in paths],
                      separators=(',', ':')).encode()


def snapshots_measure(paths, label, zstd, chunk_members=32):
    # Canonical filename order for a fixed offline sample. A production writer
    # should instead retain capture sequence and distinguish streams/layouts.
    manifest = manifests(paths)
    members = [path.read_bytes() for path in paths]
    raw = sum(map(len, members))
    unique = set(members)
    result = {'label': label, 'files': len(paths), 'raw_bytes': raw,
              'unique_whole_files': len(unique), 'whole_file_dedup_bytes': sum(map(len, unique)),
              'manifest_sha256': sha(manifest), 'manifest_zstd_bytes': len(zstd.compress(manifest)),
              'chunk_members': chunk_members, 'candidates': {}}
    if len(members) > 1:
        changed = [sum(a != b for a, b in zip(previous, current)) / max(len(previous), len(current))
                   for previous, current in zip(members, members[1:])]
        result['median_changed_byte_fraction'] = statistics.median(changed)
    for level in (1, 3, 9):
        total = 0
        start = time.perf_counter()
        for member in members:
            packed = zstd.compress(member, level)
            if zstd.decompress(packed, len(member)) != member:
                raise ValueError('individual snapshot differs')
            total += len(packed)
        result['candidates'][f'individual-zstd-{level}'] = {
            'bytes': total, 'ratio': raw / total,
            'round_trip_seconds': time.perf_counter() - start}
    # Bounded independent groups: 32 snapshots. Dictionary never grows forever.
    for page_size in (256, 1024, 4096):
        total = 0
        unique_pages = 0
        table_bytes = 0
        start = time.perf_counter()
        for first in range(0, len(members), chunk_members):
            block = members[first:first + chunk_members]
            dictionary = {}
            pages = []
            tables = []
            for member in block:
                ids = []
                for offset in range(0, len(member), page_size):
                    # Padding is reversed using the original member length.
                    page = member[offset:offset + page_size].ljust(page_size, b'\0')
                    if page not in dictionary:
                        dictionary[page] = len(pages)
                        pages.append(page)
                    ids.append(dictionary[page])
                tables.append(ids)
            pool = b''.join(pages)
            index = b''.join(struct.pack('<I', identity) for table in tables for identity in table)
            unique_pages += len(pages)
            table_bytes += len(index)
            packed = zstd.compress(pool + index)
            decoded = zstd.decompress(packed, len(pool) + len(index))
            if decoded != pool + index:
                raise ValueError('page payload differs')
            cursor = len(pool)
            for member, table in zip(block, tables):
                restored = bytearray()
                for _ in table:
                    identity, = struct.unpack_from('<I', decoded, cursor)
                    cursor += 4
                    restored.extend(decoded[identity * page_size:(identity + 1) * page_size])
                if restored[:len(member)] != member:
                    raise ValueError('page snapshot differs')
            total += len(packed)
        result['candidates'][f'pages-{page_size}-zstd-3'] = {
            'bytes': total, 'ratio': raw / total, 'unique_pages_across_chunks': unique_pages,
            'raw_page_table_bytes': table_bytes,
            'round_trip_seconds': time.perf_counter() - start}
    for method in ('concatenated', 'xor'):
        total = 0
        start = time.perf_counter()
        for first in range(0, len(members), chunk_members):
            block = members[first:first + chunk_members]
            if method == 'concatenated':
                payload = b''.join(block)
            else:
                previous = b''
                deltas = []
                for member in block:
                    if previous and len(member) != len(previous):
                        raise ValueError('XOR experiment requires a fixed state layout')
                    deltas.append(bytes(a ^ b for a, b in zip(member, previous)) if previous else member)
                    previous = member
                payload = b''.join(deltas)
            packed = zstd.compress(payload)
            decoded = zstd.decompress(packed, len(payload))
            cursor = 0
            previous = b''
            for member in block:
                restored = decoded[cursor:cursor + len(member)]
                if method == 'xor' and previous:
                    restored = bytes(a ^ b for a, b in zip(restored, previous))
                if restored != member:
                    raise ValueError('chunk snapshot differs')
                previous = restored
                cursor += len(member)
            total += len(packed)
        result['candidates'][f'{method}-zstd-3'] = {
            'bytes': total, 'ratio': raw / total,
            'round_trip_seconds': time.perf_counter() - start}
    result['round_trip'] = True
    if manifests(paths) != manifest:
        raise ValueError('source snapshot changed during experiment')
    print(f'{label}: {len(paths)} files, {raw} bytes', flush=True)
    return result


def controls(zstd):
    original = b'ppu 0 1 4 7\nppu 0 1 4 8\nppu 0 1 4 8\napu 4 16777215 255\nppu 0 0 0 0\n'
    binary, count = encode_events(original)
    assert count == 5 and decode_events(binary) == original
    assert zstd.compress(binary) == zstd.compress(binary)
    changed = original.replace(b'4 7', b'4 9')
    reordered = b''.join(original.splitlines(keepends=True)[1:3] + original.splitlines(keepends=True)[:1]
                         + original.splitlines(keepends=True)[3:])
    assert sha(encode_events(changed)[0]) != sha(binary)
    assert sha(encode_events(reordered)[0]) != sha(binary)
    for invalid in (b'\x80', b'\xff' * 10):
        try:
            read_varint(invalid, 0)
        except ValueError:
            pass
        else:
            raise ValueError('bad varint accepted')
    packed = zstd.compress(binary)
    try:
        zstd.decompress(packed[:-1], len(binary))
    except ValueError:
        pass
    else:
        raise ValueError('truncated compressed frame accepted')
    return {'duplicates_order_changes_resets': True, 'varint_truncation_overflow': True,
            'compressed_frame_truncation': True, 'same_process_determinism': True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--capture-root', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--trace-mib', type=int, default=4)
    parser.add_argument('--snapshot-limit', type=int, default=1000)
    parser.add_argument('--only-traces', action='store_true')
    args = parser.parse_args()
    if not 1 <= args.trace_mib <= 64 or not 1 <= args.snapshot_limit <= 2000:
        parser.error('trace-mib must be 1..64; snapshot-limit must be 1..2000')
    root = args.capture_root.resolve()
    if root == args.output.resolve() or root in args.output.resolve().parents:
        parser.error('output must be outside the source capture')
    zstd = Zstd()
    report = {'schema': 'storage-study-v1', 'source_root': str(root), 'zstd_version': zstd.version,
              'levels': [1, 3, 9], 'controls': controls(zstd), 'traces': [], 'snapshots': []}
    trace = root / 'hardware.log'
    verdict_path = root.parent / 'verdict.json'
    if verdict_path.is_file():
        verdict_bytes = verdict_path.read_bytes()
        verdict = json.loads(verdict_bytes)
        report['capture_verdict'] = {'path': str(verdict_path), 'sha256': sha(verdict_bytes),
                                     'status': verdict.get('status'),
                                     'stored_trace_sha256': verdict.get('artifacts', {}).get('hardware.log')}
    if trace.is_file():
        for fraction in (0, 0.5, 0.9):
            report['traces'].append(trace_measure(trace, fraction, args.trace_mib * 1024**2, zstd))
    for label, paths in (
        ('wram', sorted((root / 'fixtures').glob('*.wram'))),
        ('frames', sorted(root.glob('frame-*.bin'))),
        ('rom-views', sorted((root / 'fixtures').glob('*.rom'))),
    ):
        if args.only_traces:
            continue
        selected = paths[:min(args.snapshot_limit, 32) if label == 'rom-views' else args.snapshot_limit]
        if selected:
            measured = snapshots_measure(selected, label, zstd)
            measured.update(available_files=len(paths), sample_order='filename lexical')
            report['snapshots'].append(measured)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(f'Measurements: {args.output}', flush=True)


if __name__ == '__main__':
    main()
