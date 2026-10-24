"""Eight finite settlement comparisons, existing PTRS1 helpers only."""
import hashlib
import struct
CASES=tuple(range(3900,3908))
RAM_SIZE=131072

def check_capture(data,case,h):
    h.require(case in CASES,'unadmitted settlement case')
    r=h.records(data,b'PTRS1')
    def span(at,b):return struct.pack('<II',at,len(b))+b
    def word(b,at):return struct.unpack_from('<H',b,at)[0]
    def put(b,at,v):struct.pack_into('<H',b,at,v)
    poses=bytearray(24);poses[8]=8;poses[14]=2
    colors=b''.join(struct.pack('<H',0x2000+i) for i in range(16))
    metadata=[h.Record(0,struct.pack('<IB',case,1)),
      h.Record(8,span(0x91b629,poses)+span(0x9b9000,colors)),
      h.Record(10,span(0x91b629,poses)+span(0x91d727,b'\x00\x90')),
      h.Record(12,struct.pack('<II',1048576,1048576))]
    h.require(r[:4]==metadata,'settlement exact authored metadata')
    body=r[4:]
    h.require([x.tag for x in body]==[2,2,5,2]+[3]*16+[5,4,4,2],'settlement exact record order')
    snaps=[]
    for index,phase in [(0,1),(1,220),(3,28),(23,11)]:
        p=body[index].payload
        h.require(len(p)==RAM_SIZE+3 and word(p,0)==phase and p[-1]==0,'settlement exact snapshot/diagnostic')
        snaps.append(p[2:-1])
    h.require(body[2]==h.Record(5,b'\x01\x00\x90') and body[20]==h.Record(5,b'\x04\x00\x00'),'copy then misc')
    for i,rec in enumerate(body[4:20]):
        h.require(rec==h.Record(3,struct.pack('<HIHH',11,0xc180+2*i,0,0x2000+i)),'exact palette publication')
    h.require(body[21]==h.Record(4,struct.pack('<HH',3900,int(case==3904))),'actual damage Result')
    h.require(body[22]==h.Record(4,struct.pack('<HH',3903,1)),'missing complete fullRAM success')
    entry,after_fx,before_copy,final=snaps
    for at,value in [(0xa1c,1),(0xa94,2),(0xa96,0),(0x9c2,3 if case==3905 else 99),
                     (0xa4c,0x10 if case==3905 else 0x2000),
                     (0xa4e,0x20 if case==3905 else 1),
                     (0xa50,0x8000 if case==3904 else 4 if case==3905 else 0),
                     (0x9a2,1 if case==3901 else 0x21 if case==3902 else 0)]:
        h.require(word(entry,at)==value,f'settlement initial {at:x}')
    effect=bytearray(entry);put(effect,0xa94,1)
    if case!=3905:
        put(effect,0xad2,2);put(effect,0xa9c,0 if case==3902 else 2)
        put(effect,0xa4e,0x8001);put(effect,0xa50,0x8001 if case==3904 else 1)
    h.require(after_fx==effect,'actual atmosphere pending production/full unchanged RAM')
    put(effect,0xa72,0x601)
    h.require(before_copy==effect,'palette starts after atmosphere and before damage')
    effect[0xc180:0xc1a0]=colors
    health,fraction={3900:(97,0x9fff),3901:(98,0x6000),3902:(98,0xc000),3905:(0,0)}.get(case,(99,0x2000))
    put(effect,0x9c2,health);put(effect,0xa4c,fraction)
    pending,subpending=(0x8001,0x8001) if case==3904 else (1,0x8001) if case>=3906 else (0,0)
    put(effect,0xa50,pending);put(effect,0xa4e,subpending)
    if case>=3906:
        for at,value in [(0x8b,0x100),(0x8f,0x200),(0xdfe,0x400),(0xe00,0x800)]:put(effect,at,value)
    h.require(final==effect,'settlement complete final RAM mismatch')
    return dict(case=case,status='PASS',bytes=len(data),sha256=hashlib.sha256(data).hexdigest(),
      phases=[1,220,28,11],exclusions=[],cpu_comparison=False)
