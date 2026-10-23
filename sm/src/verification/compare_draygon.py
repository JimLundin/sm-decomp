"""Compare existing local CPU DSOR1 and actual caller DSCR1 captures; no normalization."""
from pathlib import Path
import struct, sys

def records(path, magic):
    data=Path(path).read_bytes(); assert data[:5]==magic
    p=5; result=[]
    while p<len(data):
        tag,n=struct.unpack_from('<BI',data,p);p+=5
        payload=data[p:p+n];p+=n;assert len(payload)==n
        result.append((tag,payload))
    assert p==len(data)
    return result

def compare(oracle,candidate):
    cpu=records(oracle,b'DSOR1'); actual=records(candidate,b'DSCR1')
    assert cpu[0][0]==0 and actual[0][0]==0
    case=int.from_bytes(cpu[0][1],'little');assert 0<=case<=4
    assert actual[0][1][:4]==cpu[0][1]
    assert cpu[1][0]==1
    initial=cpu[1][1][16:];assert len(initial)==0x20000
    assert actual[0][1][4:]==initial
    expected=[];handle=None;branch=False;sign=0;completed=[]
    scratch_phases=set(); helper_seen=False
    for tag,p in cpu[2:]:
        assert len(p) in (16,16+0x20000)
        pc,P,A,X,Y,SP,DP,DB=struct.unpack_from('<IBHHHHHB',p)
        if tag==2:
            assert pc==0xa5a142
            handle=X;branch=False;sign=0;expected.append((1,handle,0,b''))
        elif tag==3:
            assert handle is not None
            if pc==0xa5a153: assert Y<=0x31f6 # qualified non-bank-wrapping window
            elif pc==0xa5a159:
                assert bool(P&2)==(A==0x8080)
                expected.append((2,handle,A,b''))
            elif pc in (0xa5a15b,0xa5a196):
                assert not branch;branch=True
                expected.append((3,handle,2 if pc==0xa5a15b else 1,b''))
            elif pc==0xa0afea:
                assert branch and sign<2 and A<=255
                expected.append((4+sign,handle,A,b''));sign+=1;helper_seen=True
            else: raise AssertionError(hex(pc))
        elif tag==4:
            assert pc==0xa5a181 and X==handle
            if not branch: expected.append((3,handle,0,b''))
            if helper_seen: scratch_phases.add(len(expected))
            expected.append((6,handle,0,p[16:]));completed.append(handle)
        elif tag==5:
            assert pc==0xa5a195 and SP==0x1fff and Y==0x2468 and not P&1
            if helper_seen: scratch_phases.add(len(expected))
            expected.append((7,0,0,p[16:]))
        else: raise AssertionError(tag)
    assert completed==[62,60,58,56]
    assert len(actual)-1==len(expected),(len(actual)-1,len(expected))
    preserved=set(range(0xe32,0xe34))|set(range(0x1ffb,0x2000))
    phases=0
    for event_index,((tag,p),(wanted,h,v,ram)) in enumerate(zip(actual[1:],expected)):
        assert tag==wanted
        assert len(p)>=4
        got_h,got_v=struct.unpack_from('<HH',p)
        assert (got_h,got_v)==(h,v),(tag,got_h,got_v,h,v)
        observed=p[4:]
        if not ram: assert not observed;continue
        assert len(observed)==len(ram)==len(initial)
        # CPU's sign-extension helper writes E32/E33; JSL/PHY use 1FFB..1FFF.
        # Rust must preserve these exact entry bytes. No other bytes are masked.
        assert all(observed[i]==initial[i] for i in preserved),'candidate touched CPU-only scratch/stack'
        exclusions=set(range(0x1ffb,0x2000))
        if event_index in scratch_phases: exclusions.update(range(0xe32,0xe34))
        mismatch=[i for i in range(len(ram)) if i not in exclusions and observed[i]!=ram[i]]
        assert not mismatch,(case,tag,h,[hex(i) for i in mismatch[:20]])
        phases+=1
    assert phases==5 and expected[-1][0]==7
    print(f'CPU/candidate case {case}: semantic event order + four slot RAM phases + final RAM PASS')

if __name__=='__main__':
    assert len(sys.argv)==3,'compare_draygon.py oracle.bin candidate.bin'
    compare(*sys.argv[1:])
