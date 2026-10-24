"""Finite local movement/owned-call assertion mutations. Root builds snapshots."""
import argparse,ctypes,hashlib,json,os,re,runpy,subprocess,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('verification assertions require unoptimized Python')
HERE=Path(__file__).resolve().parent
EMPTY=hashlib.sha256(b'').hexdigest()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write(p,obj):p.write_text(json.dumps(obj,indent=2),encoding='utf-8')
def modules(root):
    sys.path.insert(0,str(root/'tools'));sys.path.insert(0,str(root/'sm/src/verification'))
    import verify_dll,room_entry_mutations,player_transitions_compare
    return verify_dll,room_entry_mutations,player_transitions_compare

def recipes():
    items=json.loads((HERE/'recipes.json').read_text())['recipes']
    result={m['fault']:m for m in items}
    assert len(items)==len(result)==9
    return result

def child(args):
    verifier,_,_=modules(args.root);case=args.case
    assert case in (36,37,4031,4034,4036,4037,4048)
    sdk=os.add_dll_directory(str(Path(os.environ['LOCALAPPDATA'])/'sm-decomp-tools/SDL2-2.32.10/SDL2-2.32.10/x86_64-w64-mingw32/bin'))
    dll=args.root/'target/verify-dll/sm_verify.dll';source=verifier.source_hash(args.root);binary=sha(dll)
    host=verifier.load_host(dll);verifier.check_source(host,source);verifier.check_library(host,args.root)
    fn=host.sm_verify_room_world_case if case<4000 else host.sm_verify_player_transitions_rust
    fn.argtypes=[ctypes.c_char_p,ctypes.c_uint32,ctypes.c_void_p,ctypes.c_size_t];fn.restype=ctypes.c_size_t
    buffer=ctypes.create_string_buffer(2097152);size=fn(os.fsencode(args.rom),case,buffer,len(buffer));assert 0<=size<=len(buffer)
    data=buffer.raw[:size];(args.out/f'{case}.bin').write_bytes(data)
    assert source==verifier.source_hash(args.root) and binary==sha(dll)
    print(json.dumps(dict(case=case,bytes=size,sha256=hashlib.sha256(data).hexdigest(),source=source,dll_sha256=binary)),flush=True)

def capture(args,case):
    command=[sys.executable,'-B',str(Path(__file__).resolve()),'child','--root',str(args.root),'--out',str(args.out),'--rom',str(args.rom),'--case',str(case)]
    p=subprocess.run(command,capture_output=True,timeout=60,env=dict(os.environ,RUST_BACKTRACE='0'))
    (args.out/f'{case}.stdout.log').write_bytes(p.stdout);(args.out/f'{case}.stderr.log').write_bytes(p.stderr)
    assert p.returncode==0,('nonzero child exit is not a kill',case,p.returncode)
    obs=json.loads(p.stdout.decode('utf-8'));data=(args.out/f'{case}.bin').read_bytes()
    assert obs['case']==case and obs['bytes']==len(data) and obs['sha256']==hashlib.sha256(data).hexdigest()
    return obs,p.stderr.decode('utf-8')

def baseline_check(root,out,case,obs,stderr,entry,comparator):
    assert entry.stderr_matches(stderr,()),'baseline stderr must be clean'
    data=(out/f'{case}.bin').read_bytes()
    if case<4000:
        assert len(data)==131086 and data[131077]==0 and data[131078:]==bytes(8),'collision framing'
        if case==36:
            frozen=json.loads((root/'sm/src/verification/player_collision_dll_baselines.json').read_text())['collision:36']
            assert obs['bytes']==frozen['bytes'] and obs['sha256']==frozen['sha256']
    else:
        check=runpy.run_path(str(HERE.parent/'capture_locomotion.py'))['check']
        check(case,data,comparator)

def main():
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['baseline','mutant','child'])
    for name in ('root','out','rom'):p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--fault');p.add_argument('--baseline',type=Path);p.add_argument('--case',type=int)
    args=p.parse_args();args.root=args.root.resolve();args.out=args.out.resolve();args.rom=args.rom.resolve();assert args.rom.is_file()
    if args.mode=='child':child(args);return
    args.out.mkdir(exist_ok=False);verifier,entry,comparator=modules(args.root);all_recipes=recipes()
    source=verifier.source_hash(args.root);binary=sha(args.root/'target/verify-dll/sm_verify.dll')
    report=dict(status='RUNNING',mode=args.mode,root=str(args.root),source=source,dll_sha256=binary,rom_sha256=sha(args.rom),recipes_sha256=sha(HERE/'recipes.json'),driver_sha256=sha(Path(__file__)),checker_sha256=sha(HERE.parent/'capture_locomotion.py'),captures=[])
    write(args.out/'results.json',report)
    try:
        if args.mode=='baseline':
            for case in sorted({int(m['selected'].split(':')[1]) for m in all_recipes.values()}):
                obs,stderr=capture(args,case);baseline_check(args.root,args.out,case,obs,stderr,entry,comparator)
                assert obs['source']==source and obs['dll_sha256']==binary
                report['captures'].append(obs);write(args.out/'results.json',report)
        else:
            assert args.baseline is not None and args.fault in all_recipes
            base=json.loads((args.baseline/'results.json').read_text());base_root=Path(base['root'])
            assert base['status']=='PASS' and base['mode']=='baseline' and base['rom_sha256']==report['rom_sha256']
            assert base['recipes_sha256']==report['recipes_sha256'] and base['checker_sha256']==report['checker_sha256']
            assert verifier.source_hash(base_root)==base['source'] and sha(base_root/'target/verify-dll/sm_verify.dll')==base['dll_sha256'],'baseline changed'
            recipe=all_recipes[args.fault];case=int(recipe['selected'].split(':')[1]);recorded=next(row for row in base['captures'] if row['case']==case)
            assert recorded['sha256']==sha(args.baseline/f'{case}.bin')
            recheck=args.out/'baseline-recheck';recheck.mkdir();baseline_args=argparse.Namespace(root=base_root,out=recheck,rom=args.rom)
            repeated,stderr=capture(baseline_args,case);baseline_check(base_root,recheck,case,repeated,stderr,entry,comparator)
            assert repeated==recorded and (recheck/f'{case}.bin').read_bytes()==(args.baseline/f'{case}.bin').read_bytes(),'fresh baseline changed'
            assert len(recipe['edits'])==1,'one production fault per variant'
            edit=recipe['edits'][0];assert edit['path'].startswith('sm/src/game/')
            original=(base_root/edit['path']).read_text();changed=(args.root/edit['path']).read_text()
            assert original.count(edit['before'])==1 and changed==original.replace(edit['before'],edit['after'],1),'wrong production edit'
            obs,stderr=capture(args,case);assert obs['bytes']==0 and obs['sha256']==EMPTY,'survived selected assertion'
            assert obs['source']==source and obs['dll_sha256']==binary
            message=recipe['assertion_message']
            if args.fault=='CONTACT_REFRESH':
                text=stderr.replace('\r\n','\n');operands=re.compile(r'(?m)^  left: 0\n right: 4660\n')
                assert len(operands.findall(text))==1,'wrong refresh operands';stderr=operands.sub('',text,count=1)
            elif recipe.get('expected_values'):
                left,right=recipe['expected_values'];assert re.search(r'left: '+re.escape(left)+r'\s+right: '+re.escape(right),stderr),'wrong equality operands'
            footer='Room world: room world assertions failed' if case<4000 else 'defined player transition predecessor: locomotion invocation unexpectedly panicked'
            assert entry.stderr_matches(stderr,((recipe['assertion_source'],message),),footer),'unrelated panic/footer is not a kill'
            report.update(fault=args.fault,recipe=recipe,baseline=str(args.baseline.resolve()),baseline_recheck=repeated);report['captures'].append(obs)
        assert verifier.source_hash(args.root)==source and sha(args.root/'target/verify-dll/sm_verify.dll')==binary
        assert sha(args.rom)==report['rom_sha256'] and sha(HERE/'recipes.json')==report['recipes_sha256'] and sha(Path(__file__))==report['driver_sha256'] and sha(HERE.parent/'capture_locomotion.py')==report['checker_sha256']
        report['status']='PASS'
    except BaseException as error:
        report.update(status='FAILED',error=repr(error));write(args.out/'results.json',report);raise
    write(args.out/'results.json',report);print(json.dumps(report),flush=True)
if __name__=='__main__':main()