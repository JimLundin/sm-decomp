"""Minimal existing PTRS1 carrier: fresh baseline or byte-exact owned comparison."""
import argparse,ctypes,hashlib,json,os,struct,subprocess,sys
from pathlib import Path
if sys.flags.optimize: raise RuntimeError('verification requires Python assertions')
CAPACITY=2097152

def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def write(path,value):path.write_text(json.dumps(value,indent=2),encoding='utf-8')
def check(case,data,h):
    assert 4100<=case<=4105
    records=h.records(data,b'PTRS1')
    def span(address,b):return struct.pack('<II',address,len(b))+b
    def word(b,at):return struct.unpack_from('<H',b,at)[0]
    def put(b,at,v):struct.pack_into('<H',b,at,v)
    poses=bytes([8,0,0,0,0,0,2,0])*256
    palette=b''.join(struct.pack('<H',0x2000+i) for i in range(16))
    assert records[:4]==[h.Record(0,struct.pack('<IB',case,1)),h.Record(8,span(0x91b629,poses)+span(0x9b9000,palette)),h.Record(10,span(0x91b629,poses)+span(0x91d727,b'\x00\x90')),h.Record(12,struct.pack('<II',1048576,1048576))],'exact player update metadata'
    body=records[4:]
    if case==4103:
        assert [r.tag for r in body]==[2,2,4,2]
        positions=[(0,1),(1,220),(3,11)]
    else:
        assert [r.tag for r in body]==[2,2,5,2]+[3]*16+[5,4,2]
        positions=[(0,1),(1,220),(3,28),(22,11)]
        assert body[2]==h.Record(5,b'\x01\x00\x90') and body[20]==h.Record(5,b'\x04\x00\x00')
        for i,rec in enumerate(body[4:20]):assert rec==h.Record(3,struct.pack('<HIHH',11,0xc180+2*i,0,0x2000+i))
    snaps=[]
    for index,phase in positions:
        raw=body[index].payload;assert len(raw)==131075 and word(raw,0)==phase and raw[-1]==0
        snaps.append(raw[2:-1])
    assert body[-2]==h.Record(4,struct.pack('<HH',4100,1))
    initial=snaps[0]
    assert word(initial,0xa44)=={4101:0xe7f5,4102:0xe833,4103:0xe8ec}.get(case,0xe725)
    assert word(initial,0xa1c)==(0x29 if case==4105 else 1)
    for at,value in [(0xa1e,0x0908),(0xa58,0xa337),(0x5f7,1),(0xa6e,7),(0xa94,2),(0x9c2,99),(0xa4c,0x2000),(0xa50,1),(0xa4e,0x8001)]:assert word(initial,at)==value,('seed',case,at)
    expected=bytearray(initial);put(expected,0xa6e,0);put(expected,0xa94,15 if case==4105 else 1)
    if case==4105:
        assert word(initial,0xa5a)==0xe41b and word(initial,0xb2e)==4 and word(initial,0xb2c)==0xf800 and word(initial,0xb32)==0x1000
        put(expected,0xb2e,5);put(expected,0xb2c,0x800);put(expected,0xa96,4)
    else:assert word(initial,0xa5a)==(0xe37e if case==4104 else 0xe90e)
    assert snaps[1]==expected,'gamma precedes animation and contact clears before movement'
    if case!=4103:
        put(expected,0xa72,0x601);assert snaps[2]==expected,'palette begins after animation'
        expected[0xc180:0xc1a0]=palette
    if case in (4100,4104,4105):
        put(expected,0x9c2,97);put(expected,0xa4c,0x9fff);put(expected,0xa50,0);put(expected,0xa4e,0)
    if case in (4101,4102):
        for at,value in [(0x8b,0x100),(0x8f,0x200),(0xdfe,0x400),(0xe00,0x800)]:put(expected,at,value)
    assert snaps[-1]==expected,'exact full RAM beta return duties'
    return dict(case=case,bytes=len(data),records=len(records),phases=[p for _,p in positions],sha256=hashlib.sha256(data).hexdigest(),paired_map_path='suppressed-only')

def main():
    p=argparse.ArgumentParser();p.add_argument('--root',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--rom',type=Path,required=True)
    p.add_argument('--first',type=int,default=4100);p.add_argument('--last',type=int,default=4105);p.add_argument('--reference',type=Path);p.add_argument('--child',type=int)
    args=p.parse_args();root=args.root.resolve();out=args.out.resolve();rom=args.rom.resolve();assert 4100<=args.first<=args.last<=4105
    sys.path.insert(0,str(root/'tools'));import verify_dll,player_transitions_compare as comparator
    if args.child is not None:
        case=args.child;assert args.first<=case<=args.last
        ident=json.loads((out/'identity.json').read_text());assert verify_dll.source_hash(root)==ident['source'] and digest(root/'target/verify-dll/sm_verify.dll')==ident['dll_sha256']
        sdk=os.add_dll_directory(str(Path(os.environ['LOCALAPPDATA'])/'sm-decomp-tools/SDL2-2.32.10/SDL2-2.32.10/x86_64-w64-mingw32/bin'))
        host=verify_dll.load_host(root/'target/verify-dll/sm_verify.dll');verify_dll.check_source(host,ident['source']);verify_dll.check_library(host,root)
        fn=host.sm_verify_player_transitions_rust;fn.argtypes=[ctypes.c_char_p,ctypes.c_uint32,ctypes.c_void_p,ctypes.c_size_t];fn.restype=ctypes.c_size_t
        buffer=ctypes.create_string_buffer(CAPACITY);size=fn(os.fsencode(rom),case,buffer,len(buffer));assert 0<size<=CAPACITY,('capture failed',case,size)
        data=buffer.raw[:size];(out/f'{case}.bin').write_bytes(data);check(case,data,comparator)
        assert verify_dll.source_hash(root)==ident['source'] and digest(root/'target/verify-dll/sm_verify.dll')==ident['dll_sha256'];return
    out.mkdir(exist_ok=False)
    ident=dict(root=str(root),source=verify_dll.source_hash(root),dll_sha256=digest(root/'target/verify-dll/sm_verify.dll'),rom_sha256=digest(rom),driver_sha256=digest(Path(__file__)),export='sm_verify_player_transitions_rust')
    write(out/'identity.json',ident);report=dict(status='RUNNING',identity=ident,cases=[]);write(out/'results.json',report)
    reference=None
    if args.reference:
        reference=args.reference.resolve();prior=json.loads((reference/'results.json').read_text());assert prior['status']=='PASS' and prior['identity']['rom_sha256']==ident['rom_sha256']
    for case in range(args.first,args.last+1):
        row=dict(case=case,passed=False);report['cases'].append(row)
        command=[sys.executable,'-B',str(Path(__file__).resolve()),'--root',str(root),'--out',str(out),'--rom',str(rom),'--first',str(args.first),'--last',str(args.last),'--child',str(case)]
        try:
            proc=subprocess.run(command,capture_output=True,timeout=60,env=dict(os.environ,RUST_BACKTRACE='0'))
            (out/f'{case}.stdout.log').write_bytes(proc.stdout);(out/f'{case}.stderr.log').write_bytes(proc.stderr)
            row['exit']=proc.returncode;assert proc.returncode==0 and proc.stdout==b'' and proc.stderr==b'',('unclean capture',case,proc.returncode)
            data=(out/f'{case}.bin').read_bytes();row.update(check(case,data,comparator))
            if reference:
                saved=next(r for r in prior['cases'] if r['case']==case);assert saved['passed'] and saved['sha256']==digest(reference/f'{case}.bin')
                assert data==(reference/f'{case}.bin').read_bytes(),('exact baseline mismatch',case)
            assert verify_dll.source_hash(root)==ident['source'] and digest(root/'target/verify-dll/sm_verify.dll')==ident['dll_sha256'];row['passed']=True
        except Exception as error:
            row['error']=repr(error);report['status']='FAILED';write(out/'results.json',report);raise
        write(out/'results.json',report);print('player update',case,'PASS',flush=True)
    assert digest(rom)==ident['rom_sha256'] and digest(Path(__file__))==ident['driver_sha256']
    report['status']='PASS';write(out/'results.json',report)
if __name__=='__main__':main()