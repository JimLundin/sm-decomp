#!/usr/bin/env python3
"""Build a self-contained, current-source atlas. No dependencies or network access.

References are conservative lexical Rust module/path references, not a call graph.
All tracked files are inventory nodes; file contents/assets are never embedded.
"""
import argparse, collections, datetime, hashlib, io, json, re, subprocess, tarfile
from pathlib import Path

GROUPS = [
 ('game','Gameplay','Owned domains and remaining bank routines','teal'),
 ('state','State & compatibility','Session edges, WRAM layout, content and snapshots','amber'),
 ('app','Application & devices','Entry points, windowing, configuration and native adapters','blue'),
 ('snes','Emulated backend','CPU, PPU, DMA, audio hardware and cartridge','violet'),
 ('verify','Verification','Oracle, scenarios, captures and mutation checks','pink'),
 ('build','Build & tools','Cargo, CI, installers and development scripts','slate'),
 ('docs','Documents & inputs','Plans, architecture, replay inputs and repository metadata','gray'),
]
STATE={'game_state.rs','ram_map.rs','rtl.rs','rom.rs','recording.rs','snapshot.rs','native_player_ram.rs','cpu_infra.rs','word_transfers.rs'}
APP={'main.rs','host.rs','config.rs','native_audio.rs','spc_player.rs','audio.rs','features.rs'}
def group(path):
 p=Path(path);name=p.name
 if '/verification/' in path or name.startswith('verify') or '/examples/' in path: return 'verify'
 if path.startswith('sm/src/game/'): return 'game'
 if path.startswith('sm/src/snes/'): return 'snes'
 if path.startswith('sm/src/platform/') or name in APP or path.startswith('sm/src/native_') or path.startswith('sm/native/'): return 'app'
 if path.startswith('sm/src/') and name in STATE: return 'state'
 if path.startswith('sm/src/'): return 'state'
 if path.startswith('tools/') or path.startswith('.github/') or name.startswith('Cargo.') or name in {'build.rs','rust-toolchain.toml'}: return 'build'
 return 'docs'
RESPONSIBILITIES = [
 ('movement','Samus movement'),('samus','Samus actions'),('enemies','Enemies'),
 ('bosses','Bosses'),('rooms','Rooms & doors'),('projectiles','Projectiles'),
 ('hud','HUD & scenes'),('graphics','Sprites & palettes'),('audio','Audio'),('engine','Engine & devices'),
]
ALIASES = {'kraid':['sm/src/game/sm_a7.rs'], 'grapple':['sm/src/game/grapple_swing.rs','sm/src/game/sm_9b.rs'],
 'doors':['sm/src/game/room_entry/mod.rs','sm/src/game/sm_8f.rs','sm/src/game/room_entry/'], 'ridley':['sm/src/game/sm_a6.rs'],
 'draygon':['sm/src/game/sm_a5.rs'], 'mother brain':['sm/src/game/sm_a9.rs'],
 'minimap':['sm/src/game/player_update/minimap.rs'], 'samus':['sm/src/game/player_movement/']}
def responsibility(path):
 name=Path(path).name;low=path.lower()
 if re.fullmatch(r'sm_a[45679]\.rs',name):return 'bosses'
 if any(x in low for x in ('spc','audio','dsp','music','sound')):return 'audio'
 if any(x in low for x in ('player_movement','player_collision')):return 'movement'
 if any(x in low for x in ('projectile','beam_')) or name=='sm_93.rs':return 'projectiles'
 if any(x in low for x in ('minimap','hud','menu','message_box','intro_','dialogue')) or name in ('sm_81.rs','sm_8b.rs','sm_8c.rs','sm_8d.rs','sm_8e.rs'):return 'hud'
 if any(x in low for x in ('room_','door','mechanism','elevator')) or name in ('sm_82.rs','sm_8f.rs'):return 'rooms'
 if any(x in low for x in ('samus','player_','grapple')) or name in ('sm_90.rs','sm_91.rs','sm_92.rs','sm_94.rs','sm_9b.rs'):return 'samus'
 if '/actors/' in low or 'enemy' in name or re.fullmatch(r'sm_(a[0238]|b[23])\.rs',name):return 'enemies'
 if any(x in low for x in ('palette','sprite','animated_tiles','drawing','blend','hdma','mode7','liquid_effect')) or name in ('sm_83.rs','sm_88.rs'):return 'graphics'
 return 'engine'
def source_status(path,text,clean,markers):
 unsafe=any(m['kind']=='unsafe' for m in markers);raw=any(m['kind']=='WRAM' for m in markers)
 forbid=any(m['kind']=='forbid unsafe' for m in markers)
 whole=bool(re.search(r'^#!\[forbid\(unsafe_code\)\]',text,re.M))
 if re.fullmatch(r'sm_[0-9a-f]{2}\.rs',Path(path).name):return 'legacy'
 if whole and not unsafe and not raw and not re.search(r'\bGame\b|\.foreign\s*\(',clean):return 'owned'
 if unsafe and not forbid:return 'legacy'
 return 'partial'
TOKEN=re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|r\#*"[\s\S]*?"\#*|"(?:\\.|[^"\\])*"|\b[a-zA-Z_]\w*\b|::|[^\s]')
def tokens(text): return [(m.group(),m.start()) for m in TOKEN.finditer(text) if not m.group().startswith(('//','/*'))]
def physical_module(path):
 parts=list(Path(path).relative_to('sm/src').with_suffix('').parts)
 if parts[-1] in ('lib','main','mod'): parts.pop()
 return tuple(parts)
def clean_source(text):
 # Preserve character offsets and newlines for source-line evidence.
 return re.sub(r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"',lambda m:re.sub(r'[^\n]',' ',m.group()),text)
def module_map(sources):
 mapping={};declared=[];contexts=collections.defaultdict(set);visited=set()
 def walk(path,base):
  if (path,base) in visited:return
  visited.add((path,base));contexts[path].add(base);mapping[base]=path
  text=sources[path];ts=tokens(text);stack=[];inline=[];i=0
  while i<len(ts):
   tok,pos=ts[i]
   if tok=='mod' and i+2<len(ts) and re.match(r'^\w+$',ts[i+1][0]):
    name=ts[i+1][0];kind=ts[i+2][0];key=base+tuple(inline)+(name,)
    if kind=='{':
     mapping[key]=path;stack.append(name);inline.append(name);i+=3;continue
    if kind==';':
     parent=Path(path).parent if Path(path).stem in ('lib','main','mod') else Path(path).with_suffix('')
     for scope in inline:parent/=scope
     pre=text[max(0,pos-300):pos]
     attr=re.search(r'#\[path\s*=\s*"([^"]+)"\](?:\s*#\[[^\]]*\])*\s*(?:pub(?:\([^)]*\))?\s*)?$',pre)
     # An external #[path] on a non-inline module is relative to the
     # containing source file, even when that file is not lib.rs/mod.rs.
     # Inside an inline module, Rust retains its module-directory prefix.
     if attr and not inline:parent=Path(path).parent
     opts=[parent/attr.group(1)] if attr else [parent/(name+'.rs'),parent/name/'mod.rs']
     for candidate in opts:
      normalized=str(Path(__import__('os').path.normpath(candidate))).replace('\\','/')
      if normalized in sources:
       mapping[key]=normalized;declared.append((path,normalized,text.count('\n',0,pos)+1));walk(normalized,key);break
   if tok=='{':stack.append(None)
   elif tok=='}' and stack:
    scope=stack.pop()
    if scope is not None:inline.pop()
   i+=1
 walk('sm/src/lib.rs',())
 # Application paths prefixed sm:: resolve against the library. Main has no fabricated module scope.
 contexts['sm/src/main.rs'].add(())
 # Exact module declarations outside the primary library remain visible, but
 # crate-relative references in independent detached roots are deliberately omitted.
 for path,text in sources.items():
  if not path.endswith('.rs') or path in contexts:continue
  for m in re.finditer(r'#\[path\s*=\s*"([^"]+)"\]\s*(?:pub(?:\([^)]*\))?\s*)?mod\s+\w+\s*;',text):
   target=str(Path(__import__('os').path.normpath(Path(path).parent/m.group(1)))).replace('\\','/')
   if target in sources:declared.append((path,target,text.count('\n',0,m.start())+1))
 return mapping,declared,contexts

def expand_use(s):
 ts=re.findall(r'[A-Za-z_]\w*|::|[{},*]',s);out=[];i=0
 def walk(prefix):
  nonlocal i
  path=list(prefix)
  while i<len(ts):
   tok=ts[i]
   if tok in (',','}'): return path
   if tok=='as':
    i+=2;return path
   if tok=='{':
    i+=1
    while i<len(ts) and ts[i]!='}':
     child=walk(path)
     if child:out.append(child)
     if i<len(ts) and ts[i]==',':i+=1
     elif i<len(ts) and ts[i]!='}':i+=1
    i+=1;return []
   if tok!='::':path.append(tok)
   i+=1
  return path
 tail=walk([])
 if tail:out.append(tail)
 return out

def generate(root, template, output, revision):
 commit=subprocess.check_output(['git','rev-parse',revision+'^{commit}'],cwd=root,text=True).strip()
 tracked=subprocess.check_output(['git','ls-tree','-r','--name-only',commit],cwd=root,text=True).splitlines()
 archive=tarfile.open(fileobj=io.BytesIO(subprocess.check_output(['git','archive',commit],cwd=root)))
 sources={}; files=[]
 for path in tracked:
  p=root/path
  raw=archive.extractfile(path).read();text=raw.decode('utf-8',errors='replace') if b'\0' not in raw[:4096] else ''
  if p.suffix in ('.rs','.py','.c','.h','.sh','.ps1','.toml','.yml','.yaml','.js','.html','.css'):sources[path]=text
  docs=re.findall(r'^//!\s*(.*)',text,re.M)
  clean=clean_source(text)
  markers=[]
  for title,pattern in [('unsafe',r'\bunsafe\b'),('WRAM',r'\bg_ram\b'),('native ABI',r'extern\s+"C"'),('forbid unsafe',r'forbid\(unsafe_code\)')]:
   matches=list(re.finditer(pattern,text if title in ('native ABI','forbid unsafe') else clean)) if p.suffix=='.rs' else []
   if matches:markers.append({'kind':title,'count':len(matches),'line':text.count('\n',0,matches[0].start())+1})
  production=path.startswith('sm/src/') and p.suffix=='.rs' and not re.search(r'/(verification|verify)/|/verify[^/]*[.]rs$',path)
  gameplay=production or (path.startswith('sm/') and p.suffix in ('.c','.h'))
  files.append(dict(id=path,path=path,name=p.name,group=group(path),lines=len(text.splitlines()) if text else None,bytes=len(raw),summary=' '.join(docs)[:330],markers=markers,ext=p.suffix or 'file',production=production,gameplay=gameplay,responsibility=responsibility(path),status=source_status(path,text,clean,markers) if production else 'legacy'))
 mapping,declared,contexts=module_map(sources); edge_map={}; occurrences=set()
 def add(a,b,kind,line):
  if a==b:return
  if (a,b,kind,line) in occurrences:return
  occurrences.add((a,b,kind,line))
  key=(a,b,kind)
  if key not in edge_map:edge_map[key]=dict(source=a,target=b,kind=kind,line=line,count=0)
  edge_map[key]['count']+=1
 for a,b,line in declared:add(a,b,'module',line)
 unresolved=0
 for path,text in sources.items():
  if path not in contexts:continue
  clean=clean_source(text)
  def resolve(parts,line):
   nonlocal unresolved
   if not parts:return
   resolved=False
   for base in contexts[path]:
    if parts[0] in ('crate','sm'):full=tuple(parts[1:])
    elif parts[0]=='self':full=base+tuple(parts[1:])
    elif parts[0]=='super':
     n=0
     while n<len(parts) and parts[n]=='super':n+=1
     full=base[:-n]+tuple(parts[n:])
    else:
     full=base+tuple(parts)
     if not any(full[:n] in mapping for n in range(len(base)+1,len(full)+1)):
      full=base[:-1]+tuple(parts)
      if not any(full[:n] in mapping for n in range(max(1,len(base)),len(full)+1)):continue
    for n in range(len(full),-1,-1):
     if full[:n] in mapping:
      add(path,mapping[full[:n]],'reference',line);resolved=True;break
   if not resolved:unresolved+=1
  spans=[]
  for m in re.finditer(r'\buse\s+([^;]+);',clean,re.S):
   spans.append((m.start(),m.end()))
   for parts in expand_use(m.group(1)):resolve(parts,clean.count('\n',0,m.start())+1)
  outside=list(clean)
  for start,end in spans:
   for i in range(start,end):
    if outside[i]!='\n':outside[i]=' '
  outside=''.join(outside)
  for m in re.finditer(r'\b(?:crate|self|super|sm)(?:::[A-Za-z_]\w*)+',outside):resolve(m.group().split('::'),outside.count('\n',0,m.start())+1)

 totals={status:sum(f['lines'] or 0 for f in files if f['production'] and f['status']==status) for status in ('owned','partial','legacy')}
 data=dict(commit=commit,generated=datetime.datetime.now(datetime.timezone.utc).isoformat(),groups=[dict(id=g,title=t,description=d,color=c) for g,t,d,c in GROUPS],responsibilities=[dict(id=i,title=t) for i,t in RESPONSIBILITIES],aliases=ALIASES,production=totals,files=files,edges=list(edge_map.values()),unresolved=unresolved,tensions=[],stale_tensions=[])
 tensions_path=template.parent/'tensions.json'
 if tensions_path.exists():
  for tension in json.loads(tensions_path.read_text(encoding='utf-8-sig')):
   try:
    for evidence in tension['evidence']:
     lines=sources[evidence['path']].splitlines();expected=evidence.get('expected')
     if not expected:raise ValueError('missing expected source')
     if evidence['line']>len(lines) or lines[evidence['line']-1].strip()!=expected:
      matches=[i+1 for i,line in enumerate(lines) if line.strip()==expected]
      if len(matches)!=1:raise ValueError('changed or ambiguous source')
      evidence['line']=matches[0]
    data['tensions'].append(tension)
   except (KeyError,ValueError):
    # Automatic HEAD builds must never publish an outdated architecture claim.
    data['stale_tensions'].append(tension['title'])

 payload=json.dumps(data,ensure_ascii=False,separators=(',',':')).replace('</','<\\/')
 output.parent.mkdir(parents=True,exist_ok=True)
 output.write_text(template.read_text(encoding='utf-8').replace('/*__ATLAS_DATA__*/',payload),encoding='utf-8')
 (output.parent/'atlas-data.json').write_text(json.dumps(data,ensure_ascii=False,indent=2),encoding='utf-8')
 print(json.dumps(dict(files=len(files),edges=len(edge_map),module_paths=len(mapping),commit=commit,output=str(output))))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--root',type=Path,default=Path.cwd());p.add_argument('--revision',default='HEAD');p.add_argument('--template',type=Path,default=Path(__file__).with_name('atlas.template.html'));p.add_argument('--output',type=Path,default=Path(__file__).with_name('index.html'));a=p.parse_args();generate(a.root.resolve(),a.template.resolve(),a.output.resolve(),a.revision)
