"""Compare actual-core observations to29 original65816 cases plus4 injected cases.

Usage: python compare_samus_palette_oracle.py REFERENCE_DIR CANDIDATE_DIR
This never executes gameplay or loads a ROM. Captures/results must remain local.
"""
import hashlib
import json
import sys
from pathlib import Path


def records(data, magic):
    assert data[:5] == magic, 'visual.samus.oracle.framing.magic'
    cursor = 5
    result = []
    while cursor < len(data):
        assert cursor + 5 <= len(data), 'visual.samus.oracle.framing.header'
        tag = data[cursor]
        size = int.from_bytes(data[cursor + 1:cursor + 5], 'little')
        cursor += 5
        payload = data[cursor:cursor + size]
        assert len(payload) == size, 'visual.samus.oracle.framing.payload'
        cursor += size
        result.append((tag, payload))
    return result


def injection(case):
    assert 29 <= case <= 32
    payload = (0x91d712).to_bytes(4, 'little')
    for address, value in [(0xa4a, 0x8000), (0xb62, 9 if case in (30, 32) else 0),
                           (0xad0, 2 if case <= 30 else 1)]:
        payload += address.to_bytes(4, 'little') + value.to_bytes(2, 'little')
    return payload


def parse_oracle(data, case):
    events = []
    phases = {}
    branch = None
    for tag, payload in records(data, b'SPOR1'):
        if tag == 0:
            assert int.from_bytes(payload, 'little') == case
            continue
        if tag == 9:
            assert payload == injection(case)
            events.append(('inject', payload.hex()))
            continue
        assert len(payload) >= 16
        pc = int.from_bytes(payload[:4], 'little')
        a = int.from_bytes(payload[5:7], 'little')
        x = int.from_bytes(payload[7:9], 'little')
        if tag in (1, 2, 4, 5):
            assert tag not in phases
            phases[tag] = payload[16:]
            assert len(phases[tag]) == 131072
            if tag == 2:
                carry = payload[4] & 1
                events.append(('carry', carry))
            else:
                events.append(('phase', tag))
        elif tag == 3:
            assert branch is None
            branch = pc
        elif tag == 6:
            assert pc == 0x91D8A5
            events.append(('misc',))
        elif tag == 7:
            if pc == 0x91DD5B:
                events.append(('copy', x))
            elif pc == 0x91DEBA:
                events.append(('load',))
            elif pc == 0x90F084:
                events.append(('command', a & 255))
            elif pc in (0x80902B, 0x809049):
                events.append(('sound', 0, 6 if pc == 0x809049 else 9, a & 255))
            else:
                raise AssertionError(f'unknown oracle event {pc:06x}')
        else:
            raise AssertionError(f'unknown oracle tag {tag}')
    assert set(phases) == {1, 2, 4, 5}
    assert branch == (0x91D721 if carry else 0x91D717)
    assert events.count(('misc',)) == 2
    assert sum(event[0] == 'inject' for event in events) == (case >= 29)
    return phases, events


def parse_candidate(data, case):
    events = []
    phases = {}
    for tag, payload in records(data, b'SPCR1'):
        if tag == 0:
            assert int.from_bytes(payload, 'little') == case
        elif tag in (1, 2, 4, 5):
            assert tag not in phases, f'visual.samus.oracle.{case}.duplicate-phase'
            if tag == 2:
                assert payload[0] in (0, 1)
                events.append(('carry', payload[0]))
                payload = payload[1:]
            else:
                events.append(('phase', tag))
            assert len(payload) == 131072
            phases[tag] = payload
        elif tag == 6:
            assert payload == b''
            events.append(('misc',))
        elif tag == 7:
            assert len(payload) == 3
            argument = int.from_bytes(payload[1:], 'little')
            if payload[0] == 1:
                events.append(('copy', argument))
            elif payload[0] == 2:
                assert argument == 0
                events.append(('load',))
            elif payload[0] == 3:
                events.append(('command', argument))
            else:
                raise AssertionError('unknown candidate event')
        elif tag == 8:
            assert len(payload) == 3
            events.append(('sound', *payload))
        elif tag == 9:
            assert payload == injection(case)
            events.append(('inject', payload.hex()))
        else:
            raise AssertionError(f'unknown candidate tag {tag}')
    assert set(phases) == {1, 2, 4, 5}, f'visual.samus.oracle.{case}.missing-phase'
    assert sum(event[0] == 'inject' for event in events) == (case >= 29)
    return phases, events


def compare(reference, candidate, case):
    old, old_events = parse_oracle(reference, case)
    new, new_events = parse_candidate(candidate, case)
    assert new_events == old_events, (f'visual.samus.oracle.{case}.events', old_events, new_events)
    excluded = [(0x1FEB, 0x2000)]
    if 21 <= case <= 24:
        excluded.append((0x24, 0x26))
    # No mutation of the expected data and no broad snapshot normalization.
    excluded_bytes = {i for start, end in excluded for i in range(start, end)}
    for phase in (1, 2, 4, 5):
        for start, end in excluded:
            assert new[phase][start:end] == old[1][start:end], (
                f'visual.samus.oracle.{case}.phase{phase}.preserve-excluded', start, end)
        differences = [i for i, (a, b) in enumerate(zip(old[phase], new[phase]))
                       if a != b and i not in excluded_bytes]
        assert not differences, (f'visual.samus.oracle.{case}.phase{phase}.ram',
                                 [(hex(i), old[phase][i], new[phase][i]) for i in differences[:16]])
    return {'id': case, 'passed': True, 'excluded': excluded,
            'family': 'injected-callback-transition' if case >= 29 else 'selector2',
            'candidate_sha256': hashlib.sha256(candidate).hexdigest(),
            'oracle_excluded': {str(phase): [(start, old[phase][start:end].hex())
                                           for start, end in excluded] for phase in (1, 2, 4, 5)}}


def main(reference_dir, candidate_dir):
    frozen = json.loads((reference_dir / 'results.json').read_text())
    expected = {row['id']: row for row in frozen['cases']}
    assert set(expected) in (set(range(29)), set(range(33)))
    assert all(row['passed'] for row in expected.values())
    rows = []
    for case in range(len(expected)):
        reference = (reference_dir / f'{case}.bin').read_bytes()
        assert hashlib.sha256(reference).hexdigest() == expected[case]['sha256']
        rows.append(compare(reference, (candidate_dir / f'{case}.bin').read_bytes(), case))
    print(json.dumps({'source': frozen['source'], 'passed': len(rows), 'cases': rows}, indent=2))


if __name__ == '__main__':
    main(Path(sys.argv[1]), Path(sys.argv[2]))
