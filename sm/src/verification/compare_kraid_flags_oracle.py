"""Compare original CPU and actual migrated Kraid native-word caller captures.
Usage: python compare_kraid_flags_oracle.py ORIGINAL.bin CANDIDATE.bin
No gameplay execution. ROM-derived captures stay local.
"""
import hashlib
import json
import sys
from pathlib import Path
RAM=131072
SRAM=8192

def params(case):
    assert 0 <= case < 12, 'kraid.native.case'
    return case >= 6, (1,7,65535)[case%6//2], case%2

def mask(fade,stage):
    if stage == 0: return set()
    return set(range(0x1FFD if fade and stage == 1 else 0x1FF6 if fade else 0x1FFE,0x2000))

def parse(data,candidate):
    assert data[:5] == b'NKOR1', 'kraid.native.magic'
    records=[]; cursor=5
    while cursor<len(data):
        assert cursor+5<=len(data), 'kraid.native.header'
        tag=data[cursor]; size=int.from_bytes(data[cursor+1:cursor+5],'little'); cursor+=5
        value=data[cursor:cursor+size]; assert len(value)==size, 'kraid.native.size'
        records.append((tag,value)); cursor+=size
    assert records[0][0]==0 and len(records[0][1])==4, 'kraid.native.case-header'
    case=int.from_bytes(records[0][1],'little'); fade,area,bit=params(case)
    final=(0xA7C842 if bit else 0xA7C83B) if fade else (0xA7A93D if bit else 0xA7A942)
    pcs=[0xA7C815,0xA7C819,0xA7C822,final] if fade else [0xA7A92C,final]
    stride=1 if candidate else 2
    assert len(records)==1+len(pcs)*stride, 'kraid.native.exact-stages'
    states=[]
    for stage,pc in enumerate(pcs):
        tag,value=records[1+stage*stride]
        assert tag==1 and len(value)==19+RAM+SRAM and value[0]==stage, 'kraid.native.stage-shape'
        result=int.from_bytes(value[1:3],'little')
        assert result==(bit if not fade and stage==1 else 0xFFFF), 'kraid.native.actual-return'
        cpu=value[3:19]
        if candidate: assert cpu==bytes(16), 'kraid.native.no-fake-CPU'
        else:
            assert int.from_bytes(cpu[:4],'little')==pc, 'kraid.native.actual-PC'
            tag,raw=records[2+stage*stride]
            assert tag==2 and raw[0]==stage and (len(raw)-1)%4==0, 'kraid.native.mask-shape'
            actual=[int.from_bytes(raw[i:i+4],'little') for i in range(1,len(raw),4)]
            assert actual==sorted(mask(fade,stage)), 'kraid.native.exact-executed-stack'
        states.append((value[19:19+RAM],value[19+RAM:]))
    return case,states

def equal(a,b,excluded,label):
    assert len(a)==len(b), label+'.length'
    differences=[i for i,(x,y) in enumerate(zip(a,b)) if x!=y and i not in excluded]
    assert not differences,(label,[(hex(i),a[i],b[i]) for i in differences[:16]])

def word(data,at): return int.from_bytes(data[at:at+2],'little')

def compare(original,candidate):
    case,expected=parse(original,False); other,actual=parse(candidate,True)
    assert case==other, 'kraid.native.same-case'
    fade,area,bit=params(case); address=0xD828+area
    for stage,((cpu_ram,cpu_sram),(rust_ram,rust_sram)) in enumerate(zip(expected,actual)):
        excluded=mask(fade,stage)
        equal(cpu_ram,rust_ram,excluded,f'kraid.native.case{case}.stage{stage}.RAM')
        equal(cpu_sram,rust_sram,set(),f'kraid.native.case{case}.stage{stage}.SRAM')
        assert rust_sram==actual[0][1], 'kraid.native.no-SRAM-effect'
        assert all(rust_ram[i]==actual[0][0][i] for i in mask(fade,len(actual)-1)), 'kraid.native.candidate-preserves-stack-entry'
        assert rust_ram[0x5E7:0x5E9]==actual[0][0][0x5E7:0x5E9]==bytes((0x34,0xA5)), 'kraid.native.no-scratch-side-effect'
    for states in (expected,actual):
        entry=states[0][0]; final=states[-1][0]
        assert word(entry,address)==0xA65A|bit, 'kraid.native.prepared-native-word'
        assert final[address+1]==entry[address+1] and final[address-1]==entry[address-1] and final[address+2]==entry[address+2], 'kraid.native.adjacent-word-preserved'
        if fade:
            assert word(states[1][0],0xC400)==0 and word(states[1][0],0xC402)==12, 'kraid.native.actual-palette-completion'
            assert states[1][0][0xC000:0xC400]==entry[0xC000:0xC400], 'kraid.native.completed-colors-untouched'
            assert (word(states[2][0],0x619),word(states[2][0],0x629),word(states[2][0],0x639))==(3,8,2), 'kraid.native.actual-music-enqueue'
            assert word(states[2][0],address)==word(entry,address), 'kraid.native.music-before-boss-store'
            assert word(final,address)==0xA65B and word(final,0xFA8)==(0xC851 if bit else 0xC843), 'kraid.native.branch-and-native-store'
        else:
            assert word(final,address)==word(entry,address), 'kraid.native.read-does-not-store'
    return {'case':case,'area':area,'operation':'complete-fade' if fade else 'check-dead','stages':len(actual),'RAM_bytes_per_stage':RAM,'SRAM_bytes_per_stage':SRAM,'result':'PASS'}
if __name__=='__main__':
    original=Path(sys.argv[1]).read_bytes(); candidate=Path(sys.argv[2]).read_bytes()
    report=compare(original,candidate); report['original_sha256']=hashlib.sha256(original).hexdigest(); report['candidate_sha256']=hashlib.sha256(candidate).hexdigest(); print(json.dumps(report,indent=2))
