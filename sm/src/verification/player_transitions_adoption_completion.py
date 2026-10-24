"""Finite adoption14 envelope checks; complete candidate/reference byte equality is mandatory in driver.
Composed fixture checks every RAM byte against authored literals before its success scalar.
Caller fixture checks explicit control/palette/timer outcomes and release before success.
"""
import hashlib
import struct
CASES=(2759,2763,3004,3036,3062,3086,3700,3701,3702,3800,3801,3802,3803,3804)
RAM_SIZE=0x20000

def spans(rows):
    return b''.join(struct.pack('<II',at,len(data))+data for at,data in rows)

def check_capture(data,case,h,selected):
    if case in selected.CASES:
        return selected.check_capture(data,case,h)
    h.require(case in CASES,'unadmitted adoption case')
    r=h.records(data,b'PTRS1')
    caller=case>=3800
    palette=b''.join(struct.pack('<H',0x2000+i) for i in range(16))
    if caller:
        poses=bytearray(24);poses[8]=8;poses[14]=2
        native=spans([(0x91b629,poses),(0x9b9000,palette)])
        loaded=spans([(0x91b629,poses),(0x91d727,b'\x00\x90')])
    else:
        n=bytearray(256*8);l=bytearray(256*8)
        for i in range(256):
            n[i*8]=8;n[i*8+6]=2
            l[i*8]=4;l[i*8+1]=7;l[i*8+6]=2
        n[0x37*8+1]=15;n[0x4b*8+1]=2;n[0x19*8+1]=3
        native=spans([(0x91b629,n),(0x919000,bytes([7])*8)])
        loaded=spans([(0x91b629,l),(0x91b010,b'\x00\x90'*256)])
    expected=[h.Record(0,struct.pack('<IB',case,1)),h.Record(8,native),h.Record(10,loaded),h.Record(12,struct.pack('<II',0x100000,0x100000))]
    h.require(r[:4]==expected,'exact authored metadata differs')
    body=r[4:]
    phases=[];scalars=[]
    for rec in body:
        p=rec.payload
        if rec.tag==2:
            h.require(len(p)==RAM_SIZE+3 and p[-1]==0,'full RAM snapshot or diagnostic differs')
            phases.append(struct.unpack_from('<H',p)[0])
        elif rec.tag==3:
            h.require(len(p)==10,'publication extent')
            kind,at,old,new=struct.unpack('<HIHH',p)
            h.require((kind==11 and 0xc180<=at<=0xc19e and at%2==0) if caller else (kind in (1,2) and at==0xa1e),'unexpected publication site')
        elif rec.tag==4:
            h.require(len(p)==4,'scalar extent');scalars.append(struct.unpack('<HH',p))
        elif rec.tag==5:
            h.require(caller and p in (b'\x01\x00\x90',b'\x04\x00\x00'),'unexpected palette observation')
        elif rec.tag==6:
            h.require(case==3702 and p==bytes([1,6,15]),'unexpected sound observation')
        else:
            h.require(False,f'unknown observation tag {rec.tag}')
    h.require(body[0].tag==2 and phases[0]==1 and body[-1].tag==2 and phases[-1]==11,'entry/final order')
    h.require(phases.count(1)==1 and phases.count(11)==1,'duplicate boundary')
    marker=3800 if caller else 3702
    h.require(body[-2]==h.Record(4,struct.pack('<HH',marker,1)),'missing completed-check scalar')
    h.require(scalars.count((marker,1))==1,'duplicate success scalar')
    if caller:
        h.require(scalars==[(3800,1)],'caller scalar stream')
        if case==3802:
            h.require(phases==[1,11] and len(body)==3,'disabled intro emitted internal effects')
            h.require(body[0].payload[2:]==body[-1].payload[2:],'disabled intro changed RAM')
        else:
            h.require(phases==[1,220,28,11],'caller exact phases')
            h.require([x.tag for x in body]==[2,2,5,2]+[3]*16+[5,4,2],'caller exact event order')
            h.require(body[2]==h.Record(5,b'\x01\x00\x90') and body[-3]==h.Record(5,b'\x04\x00\x00'),'palette copy/misc ordering')
            for i,rec in enumerate(body[4:20]):
                h.require(rec==h.Record(3,struct.pack('<HIHH',11,0xc180+2*i,0,0x2000+i)),'palette publication content')
            final=body[-1].payload[2:-1]
            h.require(final[0xc180:0xc1a0]==palette,'final palette')
            h.require(struct.unpack_from('<H',final,0xa72)[0]==0x601,'visor timer tail')
            h.require(struct.unpack_from('<HH',final,0xa94)==(1,0),'caller timer/frame')
    else:
        # Exact actual operation order, including every metadata publication.
        def phase(value): return (2,struct.pack('<H',value))
        def scalar(kind,value): return (4,struct.pack('<HH',kind,value))
        def publication(kind,before,after): return (3,struct.pack('<HIHH',kind,0xa1e,before,after))
        expected=[phase(1)]
        if case==3702:
            expected += [phase(3700),phase(3701),(6,bytes([1,6,15]))]
        expected += [phase(220),phase(2),publication(1,8,8 if case==3700 else 0xf08),phase(3)]
        if case==3700:
            expected += [phase(20),scalar(1,0),phase(4),phase(10)]
        else:
            expected += [scalar(1,1),phase(4),phase(5),publication(2,0xf08,8),phase(6),phase(10)]
        expected += [scalar(3702,1),phase(11)]
        actual=[(rec.tag,rec.payload[:2] if rec.tag==2 else rec.payload) for rec in body]
        h.require(actual==expected,'composed exact phase/scalar/publication/sound order')
    return dict(case=case,status='PASS',bytes=len(data),sha256=hashlib.sha256(data).hexdigest(),phases=phases,exclusions=[],cpu_comparison=False)
