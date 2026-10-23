"""Compare original CPU and actual repaired SaveToSram captures; executes no gameplay.
Usage: python compare_flags_save_oracle.py ORIGINAL.bin CANDIDATE.bin
Keep all ROM-derived captures outside the repository.
"""
import hashlib
import json
import sys
from pathlib import Path
RAM = 131072
SRAM = 8192
PCS = [0x818000, 0x818020, 0x81803A, 0x81803D, 0x818050, 0x818068, 0x818084]
FINAL_ABI = {0, 1, 3, 4, 5, 6} | set(range(0x12, 0x1C)) | set(range(0x1FF5, 0x2000))
def exact_mask(stage):
    if stage == 0:
        return set()
    if stage < 3:
        return set(range(0x12, 0x16)) | set(range(0x1FF8, 0x2000))
    return FINAL_ABI

def parse(data, candidate):
    assert data[:5] == b'FSOR1', 'flags.save.magic'
    cursor = 5
    records = []
    while cursor < len(data):
        assert cursor + 5 <= len(data), 'flags.save.record-header'
        tag = data[cursor]
        size = int.from_bytes(data[cursor+1:cursor+5], 'little')
        cursor += 5
        value = data[cursor:cursor+size]
        assert len(value) == size, 'flags.save.record-size'
        cursor += size
        records.append((tag, value))
    assert records[0][0] == 0 and len(records[0][1]) == 4, 'flags.save.case-header'
    case = int.from_bytes(records[0][1], 'little')
    assert case in (0, 1), 'flags.save.case-id'
    stride = 1 if candidate else 2
    assert len(records) == 1 + 7*stride, 'flags.save.exact-seven-stages'
    states = []
    for stage in range(7):
        tag, value = records[1+stage*stride]
        assert tag == 1 and len(value) == 17+RAM+SRAM and value[0] == stage, 'flags.save.stage-shape'
        cpu = value[1:17]
        if candidate:
            assert cpu == bytes(16), 'flags.save.no-fake-cpu-metadata'
        else:
            assert int.from_bytes(cpu[:4], 'little') == PCS[stage], 'flags.save.stage-PC'
            tag, mask = records[2+stage*stride]
            assert tag == 2 and mask[0] == stage and (len(mask)-1)%4 == 0, 'flags.save.mask-shape'
            addresses = [int.from_bytes(mask[i:i+4], 'little') for i in range(1,len(mask),4)]
            assert addresses == sorted(exact_mask(stage)), 'flags.save.exact-executed-mask'
        states.append((value[17:17+RAM], value[17+RAM:]))
    return case, states

def equal(expected, actual, excluded, label):
    assert len(expected) == len(actual), label+'.length'
    differences = [i for i,(a,b) in enumerate(zip(expected,actual)) if a != b and i not in excluded]
    assert not differences, (label, [(hex(i),expected[i],actual[i]) for i in differences[:16]])

def word(data, address):
    return int.from_bytes(data[address:address+2], 'little')

def compare(original, candidate):
    case, expected = parse(original, False)
    other, actual = parse(candidate, True)
    assert case == other, 'flags.save.same-case'
    for stage, ((cpu_ram,cpu_sram),(rust_ram,rust_sram)) in enumerate(zip(expected,actual)):
        equal(cpu_ram,rust_ram,exact_mask(stage),f'flags.save.case{case}.stage{stage}.RAM')
        equal(cpu_sram,rust_sram,set(),f'flags.save.case{case}.stage{stage}.SRAM')
        assert all(rust_ram[i] == actual[0][0][i] for i in FINAL_ABI), 'flags.save.candidate-preserves-ABI-entry'
        assert rust_ram[0x5E7:0x5E9] == actual[0][0][0x5E7:0x5E9] == bytes((0x34,0xA5)), 'flags.save.gameplay-scratch-not-excluded'
    # Independent relationships at real phase boundaries; CPU bytes remain the oracle.
    for states in (expected,actual):
        entry = states[0][0]
        assert states[1][0][0xD7C0:0xD820] == entry[0x9A2:0xA02], 'flags.save.player-copy'
        area = 1 if case == 0 else 6
        start = 0xCD52 + area*256
        assert states[2][0][start:start+256] == entry[0x7F7:0x8F7], 'flags.save.unaligned-map-copy'
        assert states[4][0][0xD916:0xD91A] == entry[0x78B:0x78D]+entry[0x79F:0x7A1], 'flags.save.station-area'
        for stage in range(7):
            assert states[stage][0][0xD820:0xD8F0] == entry[0xD820:0xD8F0], 'flags.save.flags-preserved'
        checksum = sum(word(states[5][0],a) for a in range(0xD7C0,0xDE1C,2)) & 0xFFFF
        slot = 0 if case == 0 else 2
        final = states[6][1]
        assert word(final,2*slot) == word(final,0x1FF0+2*slot) == checksum, 'flags.save.both-checksums'
        assert word(final,8+2*slot) == word(final,0x1FF8+2*slot) == checksum ^ 0xFFFF, 'flags.save.both-complements'
        assert states[0][1] == states[4][1], 'flags.save.no-premature-SRAM-write'
    return {'case':case,'stages':7,'RAM_bytes_per_stage':RAM,'SRAM_bytes_per_stage':SRAM,'exact_ABI_bytes':len(FINAL_ABI),'result':'PASS'}
if __name__ == '__main__':
    original = Path(sys.argv[1]).read_bytes()
    candidate = Path(sys.argv[2]).read_bytes()
    report = compare(original,candidate)
    report['original_sha256'] = hashlib.sha256(original).hexdigest()
    report['candidate_sha256'] = hashlib.sha256(candidate).hexdigest()
    print(json.dumps(report,indent=2))
