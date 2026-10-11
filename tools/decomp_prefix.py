#!/usr/bin/env python3
"""Investigate an original-ROM update prefix; unsupported boundaries never accept."""
import argparse
import csv
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
REVISION = '578f90b3cc49557bb70060ad033bb90b8cf8ac50'
ROM_HASH = '12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72'
FIELDS = {'game_state':0x998,'room':0x79b,'samus_x':0xaf6,'samus_y':0xafa,'health':0x9c2,
          'items':0x9a4,'equipped_items':0x9a2,'clock_frames':0x9da,'clock_seconds':0x9dc,
          'clock_minutes':0x9de,'clock_hours':0x9e0,'rng':0x5e5,'held':0x8b,'pressed':0x8f,
          'previous':0x97,'accepted_nmi':0x5b6,'all_nmi':0x5b8,'upload':0x617,'waiting':0x5b4}
HOOKS = {'reset':0x80841c,'boot-clear':0x808482,'controller-entry':0x809459,
         'controller-return':0x809496,'dispatch-entry':0x828948,'dispatch-complete':0x82897a,
         'explicit-wait':0x808338,'nmi-entry':0x809583,'message-entry':0x858080,
         'message-return':0x8580ba,'upload-entry':0x808028}
SEMANTIC = tuple(name for name in FIELDS if name not in ('accepted_nmi','all_nmi','waiting'))

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def write(path,value):
    path.write_text(json.dumps(value,indent=2)+'\n')

def controller_causality(events):
    """Retain source order and reset boundaries; never borrow a future poll."""
    rows=[];poll=None;started=False
    for event in events:
        kind=event['kind']
        row={'source_event':event['source_event'],'source_frame':event['source_frame'],'kind':kind}
        if kind=='controller-return':
            poll=event['source_event'];row.update(phase='dispatch' if started else 'prelude',held=event['state']['held'])
        elif kind=='boot-clear':
            row['cleared_poll']=poll;poll=None
        elif kind in ('dispatch-entry','dispatch-complete'):
            started=True;row['preceding_poll']=poll
        else:
            continue
        rows.append(row)
    return rows

def observer(directory):
    # Registration address is captured explicitly: callback address is memoryspace-global.
    text = 'local out=assert(io.open([['+str(directory/'events.csv')+"]],'w'))\n"
    text += "out:write('frame,event,kind,address,value,"+','.join(FIELDS)+"\\n')\nlocal sequence=0\n"
    text += 'local offsets={'+','.join(str(a) for a in FIELDS.values())+'}\n'
    text += r'''
local function observe(kind,address,value)
 sequence=sequence+1
 local parts={tostring(movie.currentframe()),tostring(sequence),kind,tostring(address),tostring(value or 0)}
 for _,a in ipairs(offsets) do
  local v
  if a==0x5b4 then v=memory.readbyte('WRAM',a) else v=memory.readword('WRAM',a) end
  parts[#parts+1]=tostring(v)
 end
 out:write(table.concat(parts,','),'\n');out:flush()
 if kind=='dispatch-complete' or kind=='boot-clear' or kind=='controller-entry' or kind=='controller-return' then
  local file=assert(io.open([['''+str(directory)+r''']]..'/state-'..sequence..'.wram','wb'))
  local bytes=memory.readregion('WRAM',0,131072)
  for i=0,131071 do file:write(string.char(bytes[i])) end
  file:close()
 end
end
function on_frame_emulated() observe('frame-complete',0,0) end
'''
    for kind,address in HOOKS.items():
        text += "memory.registerexec('BUS',"+str(address)+",function(a,v) observe('"+kind+"',"+str(address)+",v) end)\n"
    for address in (0x4218,0x4219,0x421a,0x421b,0x804218,0x804219,0x80421a,0x80421b):
        text += "memory.registerread('BUS',"+str(address)+",function(a,v) observe('controller-register',"+str(address)+",v) end)\n"
    return text

def invoke(command, directory, timeout, env=None):
    with (directory/'stdout.log').open('w') as out, (directory/'stderr.log').open('w') as err:
        subprocess.run(list(map(str,command)),cwd=directory,stdout=out,stderr=err,
                       timeout=timeout,env=env,check=True)

def compare(native, candidate):
    rows=[]
    for ordinal,(a,b) in enumerate(zip(native,candidate),1):
        differences={key:{'original':a['state'][key],'candidate':b['state'][key]}
                     for key in SEMANTIC if a['state'][key]!=b['state'][key]}
        rows.append({'ordinal':ordinal,'source_frame':a['source_frame'],'source_event':a['source_event'],
                     'host_step':b['host_step'],'candidate_event':b['event'],'differences':differences,
                     'original':a,'candidate':b,'scope':'diagnostic ordinal pairing; mapping unaccepted'})
    return rows

def investigate(args,run,verdict):
    assets=args.asset_root.resolve()
    source=assets/'deps/snesrev-sm'
    movie=(args.movie or assets/'movies/super-metroid-any-3653.lsmv').resolve()
    rom=assets/'rom/sm.sfc'
    verdict['movie_sha256']=digest(movie)
    if digest(rom)!=ROM_HASH: raise ValueError('unsupported ROM identity')
    corpus=json.loads((assets/'movies/manifest.json').read_text())
    entry=next((row for row in corpus if row['sha256']==verdict['movie_sha256']),None)
    if not entry or entry['category']!='any%': raise ValueError('unsupported movie identity')
    with zipfile.ZipFile(movie) as archive:
        if archive.read('rom.sha256').decode().strip()!=ROM_HASH: raise ValueError('movie ROM mismatch')
        authored=[]
        letters='BYsSudlrAXLR'
        for line in archive.read('input').decode().splitlines():
            header,pad=line.split('|')
            if header not in ('F.','F. 0 0') or len(pad)!=12:
                raise ValueError('unsupported movie reset/subframe/controller record')
            if any(c not in ('.',' ',letters[i]) for i,c in enumerate(pad)):
                raise ValueError('invalid movie input')
            authored.append(sum(1<<(15-i) for i,c in enumerate(pad) if c==letters[i]))
        verdict['authored_input_sha256']=hashlib.sha256(archive.read('input')).hexdigest()
    verdict.update(rom_sha256=ROM_HASH,source_revision=REVISION)
    pin=next(row for row in json.loads((ROOT/'tools/dependencies.json').read_text()) if row['name']=='snesrev-sm')
    archive_path=assets/'deps/downloads'/pin['archive']
    if pin['revision']!=REVISION or digest(archive_path)!=pin['sha256']: raise ValueError('source archive mismatch')
    with tarfile.open(archive_path) as archive:
        for member in archive.getmembers():
            if member.isfile():
                path=source/Path(*Path(member.name).parts[1:])
                if path.read_bytes()!=archive.extractfile(member).read(): raise ValueError('modified C source: '+str(path))
    verdict['source_archive_sha256']=pin['sha256']
    native=run/'native';native.mkdir()
    lua=native/'observe.lua';lua.write_text(observer(native))
    exe=assets/'deps/lsnes/source/src/util/lsnes-dumpavi.util'
    command=[exe,'--dumper=INTERNAL-NULL','--rom-a='+str(rom),'--length='+str(args.frames),'--lua='+str(lua),movie]
    env=dict(os.environ,LD_LIBRARY_PATH=str(assets/'deps/lsnes-build-deps/usr/lib/x86_64-linux-gnu'),XDG_CONFIG_HOME=str(native/'config'))
    invoke(command,native,args.timeout,env)
    log=(native/'stderr.log').read_text()+(native/'stdout.log').read_text()
    if 'Finished!' not in log or 'bsnes v085 (Compatibility core)' not in log or 'snes_ntsc' not in log:
        raise ValueError('native completion/core/region missing')
    raw=list(csv.DictReader((native/'events.csv').open()))
    if not raw: raise ValueError('empty native capture')
    events=[]
    for index,row in enumerate(raw,1):
        if int(row['event'])!=index: raise ValueError('noncontiguous native events')
        events.append({'source_frame':int(row['frame']),'source_event':index,'kind':row['kind'],
                       'address':int(row['address']),'value':int(row['value']),
                       'state':{key:int(row[key]) for key in FIELDS}})
    frames=[row for row in events if row['kind']=='frame-complete']
    if len(frames)!=args.frames: raise ValueError('truncated native frames: '+str(len(frames)))
    verdict['native']={'command':list(map(str,command)),'executable_sha256':digest(exe),
        'observer_sha256':digest(lua),'events_sha256':digest(native/'events.csv'),
        'stdout_sha256':digest(native/'stdout.log'),'stderr_sha256':digest(native/'stderr.log'),
        'completed_frames':len(frames),'core':'lsnes rr2-beta23 / bsnes v085 Compatibility','region':'snes_ntsc',
        'raw_states':{path.name:digest(path) for path in native.glob('*.wram')}}
    # Every actual poll is retained, including prelude and message polls. We do not infer
    # skipped waits from repeated positions. Hardware-only frames remain source records.
    steps=[];mapped=[];started=False;first=None;last_held=0;hardware=[];poll_entry=None
    for event in events:
        row=dict(event);kind=row['kind']
        row['held']=row['state']['held'];row['pressed']=row['state']['pressed']
        row['released']=last_held & ~row['held'] if kind=='controller-return' else 0
        row['classification']='retained-boundary'
        row['source_authored_pad']=authored[row['source_frame']-1] if 1<=row['source_frame']<=len(authored) else None
        row['source_frame_convention']='lsnes movie.currentframe; 1-based authored frame lookup retained for diagnosis'
        row['host_step']=None
        if kind=='dispatch-entry': started=True
        if kind=='controller-entry':
            if poll_entry is not None: raise ValueError('nested native controller poll')
            poll_entry=row['source_event'];hardware=[]
        if kind=='controller-register' and poll_entry is not None:
            hardware.append({'source_event':row['source_event'],'address':row['address'],'value':row['value']})
        if kind=='controller-return':
            lows=[r for r in hardware if r['address']&0xffff==0x4218]
            highs=[r for r in hardware if r['address']&0xffff==0x4219]
            if poll_entry is None or len(lows)!=1 or len(highs)!=1:
                raise ValueError('native controller return lacks one actual register pair')
            row['hardware_pad']=lows[0]['value']+(highs[0]['value']<<8)
            joy2=[r for r in hardware if r['address']&0xffff in (0x421a,0x421b)]
            if joy2 and (sorted(r['address']&0xffff for r in joy2)!=[0x421a,0x421b] or any(r['value'] for r in joy2)):
                raise ValueError('captured nonzero or incomplete second controller needs register backing support')
            row['controller_entry_event']=poll_entry;row['actual_register_reads']=hardware
            poll_entry=None
            row['classification']='retimed-controller-poll'
            row['host_step']=len(steps)+1
            steps.append({'source_frame':row['source_frame'],'source_event':row['source_event'],'pad':row['held'],
                          'reason':'one retained native poll per diagnostic C host step'})
            last_held=row['held']
            if not started and first is None:
                first={'kind':'prelude-controller-poll','source_frame':row['source_frame'],'source_event':row['source_event'],
                       'reason':'Native input routine runs before main dispatch; C boot coroutine does not expose an equivalent poll seam. Retiming is diagnostic only.'}
        elif kind=='frame-complete': row['classification']='skipped-no-c-step; all boundary records retained'
        elif kind=='explicit-wait': row['classification']='retained-deliberate-wait; unproved-c-coroutine-mapping'
        mapped.append(row)
    if not steps or not first: raise ValueError('bounded capture lacks expected prelude seam')
    timeline={'contract':'decomp-prefix-map-v1','accepted':False,'events':mapped,'host_steps':steps,
              'policy':'retain every actual poll and every boundary; no future-state search, no corrective injection',
              'terminal':'all unfinished native dispatches retained; no completion inferred'}
    write(run/'timeline.json',timeline)
    causality={'accepted':False,'policy':'source-order controller ledger; boot clearing invalidates prior controller ownership; no replay state injection',
               'events':controller_causality(events)}
    write(run/'controller-causality.json',causality)
    verdict['controller_causality_sha256']=digest(run/'controller-causality.json')
    verdict['timeline_sha256']=digest(run/'timeline.json')
    poll_cases=run/'poll-cases.txt'
    polls=[row for row in mapped if row['kind']=='controller-return']
    fixture_lines=[]
    for row in polls:
        entry_state=(native/f"state-{row['controller_entry_event']}.wram").read_bytes()
        return_state=(native/f"state-{row['source_event']}.wram").read_bytes()
        stack=[i for i in range(0x1f00,0x2000) if entry_state[i]!=return_state[i]]
        if stack and (len(stack)!=1 or entry_state[stack[0]]!=0x94 or return_state[stack[0]]!=0x16):
            raise ValueError('native controller call lacks bounded known return-address stack transition')
        fixture_lines.append(f"{row['controller_entry_event']} {row['source_event']} {row['hardware_pad']:04x} {stack[0] if stack else 0xffff:04x}\n")
    poll_cases.write_text(''.join(fixture_lines))
    verdict['poll_cases_sha256']=digest(poll_cases)
    inputs=run/'derived.inputs';inputs.write_text(''.join('%04x\n'%row['pad'] for row in steps))
    verdict['derived_inputs_sha256']=digest(inputs)
    units=sorted((source/'src').glob('sm_*.c'))+[source/'src'/name for name in ('spc_player.c','tracing.c','util.c')]+sorted((source/'src/snes').glob('*.c'))
    host=ROOT/'tools/decomp/prefix_host.c'
    build=["cc","-std=gnu11","-O2","-fno-strict-aliasing","-ffunction-sections","-fdata-sections","-iquote",source/'src',*units,host,
           '-Wl,--gc-sections','-Wl,--wrap=Call','-Wl,--wrap=cpu_runOpcode','-Wl,--wrap=DecompressToMem',
           '-Wl,--wrap=ReadJoypadInputs','-Wl,--wrap=Vector_NMI','-Wl,--wrap=NextRandom','-lm','-o',run/'prefix-host']
    invoke(build,run,args.timeout)
    verdict['build']={'command':list(map(str,build)),'binary_sha256':digest(run/'prefix-host'),
        'compiler':subprocess.check_output(['cc','--version'],text=True).splitlines()[0],
        'host_sha256':digest(host),'base_host_sha256':digest(ROOT/'tools/decomp/headless.c'),
        'frame_header_sha256':digest(ROOT/'tools/decomp/headless_frame.h'),'driver_sha256':digest(Path(__file__)),
        'sources':{str(path.relative_to(source)):digest(path) for path in units}}
    (run/'LICENSE.txt').write_bytes((source/'LICENSE.txt').read_bytes())
    (run/'prefix_host.c').write_bytes(host.read_bytes())
    (run/'headless_frame.h').write_bytes((ROOT/'tools/decomp/headless_frame.h').read_bytes())
    (run/'headless.c').write_bytes((ROOT/'tools/decomp/headless.c').read_bytes())
    (run/'driver.py').write_bytes(Path(__file__).read_bytes())
    primitive=run/'poll-primitive';primitive.mkdir()
    cmd=[run/'prefix-host',rom,native,poll_cases,'poll-fixtures']
    invoke(cmd,primitive,args.timeout)
    primitive_rows=[json.loads(line) for line in (primitive/'stdout.log').read_text().splitlines() if line.startswith('{')]
    if len(primitive_rows)!=len(polls) or not all(row['gameplay_wram_match'] for row in primitive_rows):
        raise ValueError('native captured controller primitive differs from pinned C')
    verdict['poll_primitive']={'scope':'offline routine differential from original entry WRAM and actual hardware read pair; no movie replay state injection',
        'command':list(map(str,cmd)),'cases':len(primitive_rows),'gameplay_wram_match':True,'full_wram_match':all(row['full_wram_match'] for row in primitive_rows),
        'raw_stack_differences':[{key:row[key] for key in ('entry_event','return_event','different_bytes','native_stack_offset')} for row in primitive_rows],
        'excluded_bytes':{'per-case exact stack byte':'only native entry0x94→return0x16 call-address transition; validated unique changed stack byte; C must preserve entry0x94; all other stack bytes compared'},
        'first_source_frame':polls[0]['source_frame'],'stdout_sha256':digest(primitive/'stdout.log'),
        'stderr_sha256':digest(primitive/'stderr.log')}
    poll_control=run/'changed-poll-primitive';poll_control.mkdir()
    control_cmd=[run/'prefix-host',rom,native,poll_cases,'changed-poll-fixtures']
    invoke(control_cmd,poll_control,args.timeout)
    changed_polls=[json.loads(line) for line in (poll_control/'stdout.log').read_text().splitlines() if line.startswith('{')]
    if len(changed_polls)!=len(polls) or not any(not row['gameplay_wram_match'] for row in changed_polls):
        raise ValueError('changed hardware sample escaped polling primitive comparison')
    verdict['poll_primitive']['changed_sample_control']={'detected':True,'change':'toggle actual presented hardware A button bit in separate offline process',
        'command':list(map(str,control_cmd)),'stdout_sha256':digest(poll_control/'stdout.log'),
        'stderr_sha256':digest(poll_control/'stderr.log'),'first_mismatch':next(row for row in changed_polls if not row['gameplay_wram_match'])}
    state_control=run/'changed-state-poll-primitive';state_control.mkdir()
    state_cmd=[run/'prefix-host',rom,native,poll_cases,'changed-state-poll-fixtures']
    invoke(state_cmd,state_control,args.timeout)
    state_rows=[json.loads(line) for line in (state_control/'stdout.log').read_text().splitlines() if line.startswith('{')]
    if len(state_rows)!=len(polls) or not all(not row['gameplay_wram_match'] for row in state_rows):
        raise ValueError('unrelated WRAM error escaped polling primitive comparison')
    verdict['poll_primitive']['unrelated_state_control']={'detected':True,'change':'toggle room byte0x79b after real primitive, separate offline process',
        'command':list(map(str,state_cmd)),'stdout_sha256':digest(state_control/'stdout.log'),'stderr_sha256':digest(state_control/'stderr.log')}
    subjects={}
    for name,mode in (('candidate','unchanged'),('changed-gameplay','changed-rng')):
        directory=run/name;(directory/'fixtures').mkdir(parents=True);(directory/'saves').mkdir()
        cmd=[run/'prefix-host',rom,inputs,str(len(steps)),mode]
        invoke(cmd,directory,args.timeout)
        records=[json.loads(line) for line in (directory/'events.jsonl').read_text().splitlines()]
        complete=[row for row in records if row['kind']=='dispatch-complete']
        summary=[json.loads(line) for line in (directory/'stdout.log').read_text().splitlines() if line.startswith('{')][-1]
        subjects[name]=complete
        verdict[name]={'command':list(map(str,cmd)),**summary,'events_sha256':digest(directory/'events.jsonl'),
                        'terminal_sha256':digest(directory/'terminal.bin'),'rom_view_sha256':digest(directory/'rom-view.bin'),
                        'complete_states':{path.name:digest(path) for path in directory.glob('*.wram')},
                        'stdout_sha256':digest(directory/'stdout.log'),'stderr_sha256':digest(directory/'stderr.log')}
    original=[row for row in mapped if row['kind']=='dispatch-complete']
    # Bounded causal seam: original NMI samples are restored after the completed
    # boundary, while the C-relocated main-entry poll is suppressed. No state loading.
    first_complete=original[0]
    prelude=[row for row in polls if row['source_event']<first_complete['source_event']]
    causal_inputs=run/'causal.inputs'
    after=next(row for row in polls if row['source_event']>first_complete['source_event'])
    causal_inputs.write_text(''.join(f"{row['hardware_pad']:04x}\n" for row in polls))
    causal_results={}
    for name,mode in (('causal-candidate','causal-polling'),('causal-control','changed-causal-rng')):
        directory=run/name;(directory/'fixtures').mkdir(parents=True);(directory/'saves').mkdir()
        cmd=[run/'prefix-host',rom,causal_inputs,str(len(polls)),mode]
        invoke(cmd,directory,args.timeout)
        records=[json.loads(line) for line in (directory/'events.jsonl').read_text().splitlines()]
        completed=[row for row in records if row['kind']=='dispatch-complete']
        if len(completed)!=len(original): raise ValueError('bounded causal seam completion count differs')
        causal_results[name]=completed
        before=[row for row in records if row['kind']=='controller-return' and row['event']<completed[0]['event']]
        if [row['state']['held'] for row in before]!=[row['held'] for row in prelude]:
            raise ValueError('causal prelude polls changed order or sample')
        post=[row for row in records if row['kind']=='controller-return' and row['event']>completed[0]['event']]
        if len(post)!=len(polls)-len(prelude) or [row['state']['held'] for row in post]!=[row['held'] for row in polls[len(prelude):]]:
            raise ValueError('causal post-completion sample boundary differs')
        for ordinal,boundary in enumerate(completed):
            prior=[row for row in records if row['kind']=='controller-return' and row['event']<boundary['event']]
            expected_prior=[row for row in polls if row['source_event']<original[ordinal]['source_event']]
            if [row['state']['held'] for row in prior]!=[row['held'] for row in expected_prior]:
                raise ValueError('causal completion consumed future or reordered poll')
        verdict[name]={'command':list(map(str,cmd)),'events_sha256':digest(directory/'events.jsonl'),
            'terminal_sha256':digest(directory/'terminal.bin'),'complete_states':{path.name:digest(path) for path in directory.glob('complete-*.wram')},
            'stdout_sha256':digest(directory/'stdout.log'),'stderr_sha256':digest(directory/'stderr.log')}
    causal_diff={key:{'original':first_complete['state'][key],'candidate':causal_results['causal-candidate'][0]['state'][key]}
        for key in SEMANTIC if first_complete['state'][key]!=causal_results['causal-candidate'][0]['state'][key]}
    control_diff={key:{'candidate':causal_results['causal-candidate'][0]['state'][key],'changed':causal_results['causal-control'][0]['state'][key]}
        for key in SEMANTIC if causal_results['causal-candidate'][0]['state'][key]!=causal_results['causal-control'][0]['state'][key]}
    if causal_diff or not control_diff: raise ValueError('bounded causal first-completion seam failed parity or control')
    verdict['causal_first_completion']={'semantic_match':True,'source_frame':first_complete['source_frame'],
        'source_event':first_complete['source_event'],'prelude_polls_executed':len(prelude),
        'next_poll_event':after['source_event'],'policy':'real NMI polling after completion; relocated main poll suppressed; real reset clearing; no WRAM loading',
        'candidate':causal_results['causal-candidate'][0],'changed_rng_control':control_diff,
        'inputs_sha256':digest(causal_inputs),'scope':'first completed gameplay dispatch only; boot video timing and later waits unproven'}
    causal_comparisons=compare(original,causal_results['causal-candidate'])
    if any(row['differences'] for row in causal_comparisons):
        raise ValueError('later causal completion semantic mismatch')
    prefix_controls=[{'ordinal':i,'differences':{key:{'candidate':a['state'][key],'changed':b['state'][key]} for key in SEMANTIC if a['state'][key]!=b['state'][key]}}
        for i,(a,b) in enumerate(zip(causal_results['causal-candidate'],causal_results['causal-control']),1)]
    if not any(row['differences'] for row in prefix_controls): raise ValueError('changed causal prefix escaped detection')
    verdict['causal_update_prefix']={'semantic_match':True,'completed_updates':len(causal_comparisons),
        'comparisons':causal_comparisons,'changed_rng_control':prefix_controls,'polls_executed':len(polls),
        'scope':'source-ordered controller and completed-update semantic fields; native waits and video timing remain retained but unmapped',
        'explicit_native_wait_events':[row['source_event'] for row in mapped if row['kind']=='explicit-wait']}
    comparisons=compare(original,subjects['candidate'])
    if not comparisons: raise ValueError('no diagnostic completed-boundary comparisons')
    verdict['comparisons']=comparisons
    verdict['first_semantic_mismatch']=next((row for row in comparisons if row['differences']),None)
    completed_ledger=[row for row in causality['events'] if row['kind']=='dispatch-complete']
    poll_order={row['host_step']:row['source_event'] for row in mapped if row['kind']=='controller-return'}
    causal_violations=[]
    for comparison,ledger in zip(comparisons,completed_ledger):
        assigned=poll_order.get(comparison['host_step'])
        if assigned is not None and assigned>ledger['source_event']:
            causal_violations.append({'completion_event':ledger['source_event'],'completion_frame':ledger['source_frame'],
                                     'candidate_host_step':comparison['host_step'],'assigned_poll_event':assigned,
                                     'reason':'diagnostic poll-as-step assigns input observed after the compared original completion'})
    verdict['future_poll_assignments']=causal_violations
    # Compare changed subject to unchanged subject with the very same timeline. Existing
    # native mapping failure cannot count as detecting this independently introduced error.
    control=[]
    for a,b in zip(subjects['candidate'],subjects['changed-gameplay']):
        changes={key:{'unchanged':a['state'][key],'changed':b['state'][key]} for key in SEMANTIC if a['state'][key]!=b['state'][key]}
        if changes: control.append({'host_step':a['host_step'],'candidate_event':a['event'],'changes':changes})
    if not control: raise ValueError('changed gameplay escaped semantic detection')
    verdict['changed_gameplay_control']={'detected':True,'change':'actual linked NextRandom result toggles one bit, in separate process',
        'comparison':'same fixed source map; changed subject vs unchanged subject, independent of original adapter rejection',
        'first_mismatch':control[0],'corrective_state_injection':False}
    verdict['comparison_exclusions']={'accepted_nmi':'scheduler phases differ; raw counts retained, mapping unaccepted',
        'all_nmi':'hardware lag count retained separately; no inferred normalization','waiting':'comparison captures before C NMI and original tail; raw byte retained'}
    verdict.update(status='nonaccepting',first_unsupported=first,accepted=False,
        original_rom_parity='unproven; diagnostic prefix rejected',initial_state='ordinary reset with zero SRAM once per process')

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--asset-root',type=Path,default=ROOT)
    parser.add_argument('--movie',type=Path)
    parser.add_argument('--frames',type=int,default=320)
    parser.add_argument('--timeout',type=int,default=120)
    parser.add_argument('--output',type=Path,default=ROOT/'out/decomp/prefix')
    args=parser.parse_args()
    if not 320<=args.frames<=2000 or args.timeout<1: parser.error('bounded investigation requires 320..2000 frames and positive timeout')
    run=args.output.resolve()/datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ');run.mkdir(parents=True)
    verdict={'contract':'decomp-original-prefix-v1','status':'error','accepted':False,'requested_frames':args.frames}
    try: investigate(args,run,verdict)
    except (OSError,ValueError,KeyError,subprocess.SubprocessError,zipfile.BadZipFile) as error: verdict['error']=str(error)
    write(run/'verdict.json',verdict)
    print(json.dumps({'status':verdict['status'],'evidence':str(run),'error':verdict.get('error')}))
    return 2 if verdict['status']=='nonaccepting' else 1

if __name__=='__main__': sys.exit(main())
