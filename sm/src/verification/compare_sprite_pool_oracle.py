"""Compare the52-phase originalCPU and actualRust sprite lifecycle captures.

Usage: python compare_sprite_pool_oracle.py ORIGINAL.bin CANDIDATE.bin
No gameplay is executed; both captures must remain outside the repository.
"""
import hashlib
import json
import sys
from pathlib import Path

RAM_BYTES = 131072
PHASES = 52


def phase(index):
    if index == 0:
        return 0, 0
    if index == 1:
        return 2, 0
    return (1 if index % 2 == 0 else 2), (index - 2) // 2 + 1


def exact_mask(operation, tick):
    # Independently audited executed pushes/stores for the original dud6 fixture
    # at128,96. None of its six spritemaps takes the high-X/large-sprite branches.
    if operation == 0:
        return {0x12, 0x13} | set(range(0x1FF8, 0x2000))
    if operation == 1:
        return set(range(0x1FF8, 0x2000)) | ({0x12, 0x13} if tick == 24 else set())
    if tick < 24:
        return {0, 1, 3, 4, 0x12, 0x13, 0x14, 0x15, 0x18, 0x19} | set(range(0x1FF3, 0x2000))
    return set(range(0x1FF8, 0x2000))


def records(data):
    assert data[:5] == b'PSOR1', 'sprite.pool.oracle.magic'
    cursor = 5
    result = []
    while cursor < len(data):
        assert cursor + 5 <= len(data), 'sprite.pool.oracle.record-header'
        tag = data[cursor]
        count = int.from_bytes(data[cursor + 1:cursor + 5], 'little')
        cursor += 5
        payload = data[cursor:cursor + count]
        assert len(payload) == count, 'sprite.pool.oracle.record-size'
        cursor += count
        result.append((tag, payload))
    return result


def parse(data, candidate):
    items = records(data)
    assert items[0] == (0, bytes(4)), 'sprite.pool.oracle.case-id'
    stride = 2 if candidate else 3
    assert len(items) == 1 + PHASES * stride, 'sprite.pool.oracle.exact52phases'
    phases = []
    for index in range(PHASES):
        operation, tick = phase(index)
        states = []
        for state_index, tag in enumerate((1, 2)):
            actual_tag, payload = items[1 + index * stride + state_index]
            assert actual_tag == tag and len(payload) == 22 + RAM_BYTES
            assert int.from_bytes(payload[:2], 'little') == index
            assert tuple(payload[2:4]) == (operation, tick)
            result = int.from_bytes(payload[4:6], 'little')
            assert result == (62 if tag == 2 and operation == 0 else 0xFFFF), (
                f'sprite.pool.oracle.phase{index}.result', result)
            if candidate:
                assert payload[6:22] == bytes(16), 'sprite.pool.oracle.no-fake-cpu-metadata'
            else:
                pc = int.from_bytes(payload[6:10], 'little')
                entries = (0xB4BC26, 0xB4BC82, 0xB4BD32)
                exits = (0xB4BC81, 0xB4BCEF, 0xB4BD96)
                assert pc == (entries if tag == 1 else exits)[operation]
            states.append(payload[22:])
        mask = set()
        if not candidate:
            tag, payload = items[1 + index * stride + 2]
            assert tag == 3 and len(payload) >= 2 and (len(payload) - 2) % 4 == 0
            assert int.from_bytes(payload[:2], 'little') == index
            addresses = [int.from_bytes(payload[i:i + 4], 'little') for i in range(2, len(payload), 4)]
            assert addresses == sorted(set(addresses)), 'sprite.pool.oracle.ordered-distinct-mask'
            mask = set(addresses)
            assert mask == exact_mask(operation, tick), (
                f'sprite.pool.oracle.phase{index}.audited-exact-mask', sorted(mask))
        phases.append((states[0], states[1], mask))
    return phases


def assert_equal(expected, actual, excluded, label):
    differences = [at for at, (a, b) in enumerate(zip(expected, actual))
                   if a != b and at not in excluded]
    assert not differences, (label, [(hex(at), expected[at], actual[at]) for at in differences[:16]])


def word(data, address):
    return int.from_bytes(data[address:address + 2], 'little')


def check_continuity(phases, label):
    for index in range(1, PHASES):
        expected = bytearray(phases[index - 1][1])
        operation, _ = phase(index)
        if operation == 2:
            # Exactly the shared normal OAM preparation; no hidden sprite reset.
            expected[0x370:0x592] = bytes(0x222)
        assert_equal(expected, phases[index][0], set(), f'{label}.phase{index}.sequential-entry')


def compare(reference, candidate):
    old = parse(reference, False)
    new = parse(candidate, True)
    check_continuity(old, 'sprite.pool.oracle.reference')
    check_continuity(new, 'sprite.pool.oracle.candidate')
    cumulative = set()
    rows = []
    for index, ((old_entry, old_exit, current), (entry, exit_state, _)) in enumerate(zip(old, new)):
        operation, tick = phase(index)
        # Prior CPU-only bytes may differ at the next entry; every other byte,
        # including all actual sprite/OAM/selection state, must already agree.
        assert_equal(old_entry, entry, cumulative, f'sprite.pool.oracle.phase{index}.entry')
        cumulative |= current
        assert_equal(old_exit, exit_state, cumulative, f'sprite.pool.oracle.phase{index}.exit')
        for address in cumulative:
            assert exit_state[address] == entry[address], (
                f'sprite.pool.oracle.phase{index}.preserve-cpu-only', hex(address))
            assert entry[address] == new[0][0][address], (
                f'sprite.pool.oracle.phase{index}.preserve-original-seed', hex(address))
        if operation == 2:
            assert (word(exit_state, 0x590) != 0) == (tick < 24), (
                f'sprite.pool.oracle.phase{index}.actual-oam')
        if operation == 1:
            assert (word(exit_state, 0xEFB6) != 0) == (tick < 24), (
                f'sprite.pool.oracle.phase{index}.retirement')
        rows.append({'phase': index, 'operation': operation, 'tick': tick,
                     'cpu_only_addresses': sorted(current), 'cumulative_count': len(cumulative)})
    return {'passed': True, 'phases': rows,
            'reference_sha256': hashlib.sha256(reference).hexdigest(),
            'candidate_sha256': hashlib.sha256(candidate).hexdigest(),
            'scope': 'original dud6 lifecycle and actual OAM; component oracle, not a movie'}


if __name__ == '__main__':
    print(json.dumps(compare(Path(sys.argv[1]).read_bytes(), Path(sys.argv[2]).read_bytes()), indent=2))
