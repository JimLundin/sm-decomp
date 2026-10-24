"""Minimal existing PTRS1 carrier: fresh baseline or byte-exact owned comparison."""
import argparse,ctypes,hashlib,json,os,struct,subprocess,sys
from pathlib import Path
if sys.flags.optimize: raise RuntimeError('verification requires Python assertions')
CAPACITY=2097152

def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def write(path,value):path.write_text(json.dumps(value,indent=2),encoding='utf-8')
def check(case,data,h):
    assert 4000<=case<=4082
    records=h.records(data,b'PTRS1')
    def span(address,b):return struct.pack('<II',address,len(b))+b
    poses=bytes([8,0,0,0,0,0,3,0])*256
    def content(maximum,splash):
        manifest=span(0x91b629,poses)
        table=struct.pack('<6H',0,0,maximum,0,2,0)*28
        for address in (0x90e000,0x909f49,0x90a08d,0x90a1dd):manifest+=span(address,table)
        manifest+=span(0x9b9800,b''.join(struct.pack('<H',0x4100+i) for i in range(16)))
        manifest+=span(0x90a514,bytes([splash])*10)
        if case>=4049:
            decoy=splash==1
            for kind in range(8):
                for address,value in ((0x908b93,0x9040 if decoy else 0x9000),(0x908bef,1 if decoy else 3),(0x908bff,0x9400 if decoy else 0x9200+kind*0x20)):
                    manifest+=span(address+kind*2,struct.pack('<H',value))
                for frame in range(4):manifest+=span(0x909200+kind*0x20+frame*2,struct.pack('<H',0x4000+kind*0x100+frame if decoy else 0x7000))
            for frame in range(256):manifest+=span(0x909000+frame*2,struct.pack('<H',3+frame if decoy else 9))
            for index in range(390,403):manifest+=span(0x92808d+index*2,struct.pack('<H',0xa000))
            manifest+=span(0x92a000,bytes([1,0,0,0,0,0x99 if decoy else 0x21,0x43]))
        return manifest
    assert records[:4]==[h.Record(0,struct.pack('<IB',case,1)),h.Record(8,content(8,1)),h.Record(10,content(6,0)),h.Record(12,struct.pack('<II',2097152,2097152))],'exact content metadata'
    body=records[4:];assert body and body[0].tag==2 and body[-1].tag==2
    phases=[];sounds=[]
    for rec in body:
        if rec.tag==2:
            assert len(rec.payload)==131075 and rec.payload[-1]==0
            phases.append(struct.unpack_from('<H',rec.payload)[0])
        elif rec.tag==3:assert len(rec.payload)==10
        elif rec.tag==4:assert rec==h.Record(4,struct.pack('<HH',4000,1))
        elif rec.tag==5:assert len(rec.payload)==3
        elif rec.tag==6:assert len(rec.payload)==3;sounds.append(list(rec.payload))
        else:raise AssertionError(('unknown locomotion record',rec.tag))
    assert phases[0]==1 and phases[-1]==11 and phases.count(1)==1 and phases.count(11)==1
    assert body[-2]==h.Record(4,struct.pack('<HH',4000,1))
    assert sum(rec.tag==4 for rec in body)==1
    # Exact native queued sound event: observer channel is zero based.
    wanted={4030:[[2,6,6]],4040:[[0,6,47]]}.get(case,[])
    if case>=4049:
        assert [p for p in phases if 450<=p<=453]==list(range(450,454 if case<=4054 else 451)),('missing consumer lifecycle',case,phases)
        if case==4049:wanted=[[2,6,6]]
        elif case==4051:wanted=[[1,6,13]]
        elif case==4052:
            assert sounds[:1]==[[1,6,13]] and sounds[1:] in ([[1,6,15]],[[1,6,17]]),('bubble sound',sounds)
            wanted=sounds
        elif case==4053:wanted=[[1,6,16]]
        elif case==4054:wanted=[[2,6,5]]
    assert sounds==wanted,('unexpected sound effects',case,sounds)
    return dict(case=case,bytes=len(data),records=len(records),phases=phases,sounds=sounds,sha256=hashlib.sha256(data).hexdigest())

def main():
    p=argparse.ArgumentParser();p.add_argument('--root',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--rom',type=Path,required=True)
    p.add_argument('--first',type=int,default=4000);p.add_argument('--last',type=int,default=4048);p.add_argument('--reference',type=Path);p.add_argument('--child',type=int)
    args=p.parse_args();root=args.root.resolve();out=args.out.resolve();rom=args.rom.resolve();assert 4000<=args.first<=args.last<=4082
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
        write(out/'results.json',report);print('locomotion',case,'PASS',flush=True)
    assert digest(rom)==ident['rom_sha256'] and digest(Path(__file__))==ident['driver_sha256']
    report['status']='PASS';write(out/'results.json',report)
if __name__=='__main__':main()