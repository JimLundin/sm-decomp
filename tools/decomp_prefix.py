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
 if kind=='dispatch-complete' or kind=='boot-clear' then
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
    for address in (0x4218,0x4219,0x804218,0x804219):
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
            if lows[0]['value']+(highs[0]['value']<<8)!=row['held']:
                raise ValueError('native consumed input differs from actual read pair')
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
    verdict['timeline_sha256']=digest(run/'timeline.json')
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
        'host_sha256':digest(host),'base_host_sha256':digest(ROOT/'tools/decomp/headless.c'),'driver_sha256':digest(Path(__file__)),
        'sources':{str(path.relative_to(source)):digest(path) for path in units}}
    (run/'LICENSE.txt').write_bytes((source/'LICENSE.txt').read_bytes())
    (run/'prefix_host.c').write_bytes(host.read_bytes())
    (run/'headless.c').write_bytes((ROOT/'tools/decomp/headless.c').read_bytes())
    (run/'driver.py').write_bytes(Path(__file__).read_bytes())
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
    comparisons=compare(original,subjects['candidate'])
    if not comparisons: raise ValueError('no diagnostic completed-boundary comparisons')
    verdict['comparisons']=comparisons
    verdict['first_semantic_mismatch']=next((row for row in comparisons if row['differences']),None)
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
