#![allow(dead_code)]
use std::{collections::BTreeMap, error::Error, fs, path::Path};
pub use port::{WRAM_SIZE, Wram, power_bomb, projectile_flight, projectile_special, projectiles};
#[path = "/tmp/sm-samus-projectile-frame/crates/port/src/projectile_trails.rs"]
pub mod projectile_trails;
#[path = "/tmp/sm-samus-projectile-frame/crates/port/src/samus_projectile_frame.rs"]
mod frame;
#[path = "/tmp/sm-samus-projectile-frame/crates/pipeline/src/family_model.rs"]
mod model;
use model::{Case, Manifest, Routine, Write};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn word(bytes: &mut [u8], at: usize, value: u16) { bytes[at..at+2].copy_from_slice(&value.to_le_bytes()); }
fn read(bytes: &[u8], at: usize) -> u16 { u16::from_le_bytes([bytes[at], bytes[at+1]]) }
fn writes(bytes: &[u8]) -> Vec<Write> {
    let mut result = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == 0 { at+=1; continue; }
        let start = at;
        while at < bytes.len() && (bytes[at] != 0 || (at+1 < bytes.len() && bytes[at+1] != 0)) { at+=1; }
        result.push(Write { offset: start, data: bytes[start..at].to_vec() });
    }
    result
}
fn writes_with_fill(bytes: &[u8], fill: u8) -> Vec<Write> {
    let mut result=Vec::new();
    let mut at=0;
    while at<bytes.len() {
        if bytes[at]==fill {at+=1;continue;}
        let start=at;
        while at<bytes.len() && (bytes[at]!=fill || (at+1<bytes.len() && bytes[at+1]!=fill)) {at+=1;}
        result.push(Write{offset:start,data:bytes[start..at].to_vec()});
    }
    result
}
fn base() -> Vec<u8> {
    let mut bytes = vec![0; WRAM_SIZE];
    for (at,value) in [(0x7a5,16),(0x7a7,16),(0x7a9,1),(0x7ab,1),(0x7b9,512),(0xa1c,1),(0xaf6,100),(0xafa,100),(0xcce,5),(0xcd2,5)] { word(&mut bytes,at,value); }
    bytes
}
fn initialize(bytes: &mut [u8], rom: &[u8], raw: u16, kind: u16, pre: u16, direction: u16) -> bool {
    let k = usize::from(raw);
    word(bytes,0xc18+k,kind); word(bytes,0xc04+k,direction); word(bytes,0xb64+k,100); word(bytes,0xb78+k,100);
    if projectiles::initialize(&mut Wram::new(bytes).unwrap(), rom, raw).is_err() { return false; }
    word(bytes,0xc68+k,pre); word(bytes,0xc90+k,2); word(bytes,0xc7c+k,32);
    true
}
fn fixture(name: impl Into<String>, bytes: &[u8]) -> Case {
    let fill=if bytes.iter().filter(|&&value|value==0xa5).count()>WRAM_SIZE/2 {0xa5} else {0};
    let writes=if fill==0 {writes(bytes)} else {writes_with_fill(bytes,fill)};
    Case {name:name.into(),fill:Some(fill),writes,expected_return:Some(0),..Default::default()}
}
fn apply(case: &Case) -> Vec<u8> {
    let mut bytes = if let Some(path)=&case.before_file {fs::read(path).unwrap()} else {vec![case.fill.unwrap_or(0); WRAM_SIZE]};
    for w in &case.writes { bytes[w.offset..w.offset+w.data.len()].copy_from_slice(&w.data); }
    bytes
}
fn run() -> Result<()> {
    if std::env::args().nth(1).as_deref()==Some("--candidate-retain") {
        let dest=std::env::args().nth(2).ok_or("candidate snapshot required")?;
        let dest=Path::new(&dest);
        if dest.exists() {return Err("candidate snapshot exists".into());}
        let mut files=BTreeMap::new();
        let root=Path::new("/tmp/sm-samus-projectile-frame");
        let mut bytes=0u64;
        for relative in ["crates/port/src/samus_projectile_frame.rs","crates/port/tests/samus_projectile_frame.rs","tools/port/samus-projectile-frame.toml"] {
            let source=root.join(relative);let target=dest.join(relative);
            fs::create_dir_all(target.parent().unwrap())?;bytes+=fs::copy(&source,&target)?;
            let output=std::process::Command::new("sha256sum").arg(&target).output()?;
            if !output.status.success() {return Err("candidate source hash failed".into());}
            files.insert(relative.to_owned(),String::from_utf8(output.stdout)?.split_whitespace().next().ok_or("hash missing")?.to_owned());
        }
        fs::write(dest.join("retention.toml"),format!("source_root = {:?}\nsnapshot = {:?}\nsource_files = {}\nsource_bytes = {bytes}\n\n{}",root.to_string_lossy(),dest.to_string_lossy(),files.len(),toml::to_string(&BTreeMap::from([("files",files)]))?))?;
        println!("candidate_source_files=3 candidate_source_bytes={bytes} snapshot={}",dest.display());
        return Ok(());
    }
    if std::env::args().nth(1).as_deref()==Some("--progress-input") {
        let registry:serde_json::Value=serde_json::from_str(&fs::read_to_string("/home/jim/sm-decomp/.scratch/decomp-progress/registry.json")?)?;
        let stage=std::env::args().nth(2).ok_or("stage required")?;
        let output=std::env::args().nth(3).ok_or("update output required")?;
        let revision=registry["source"]["revision"].as_str().ok_or("revision missing")?;
        let inventory=registry["source"]["inventory_sha256"].as_str().ok_or("inventory hash missing")?;
        fs::write(output,format!(
            "id = \"samus-projectile-frame-0f59de1-{stage}\"\nscope = \"Complete scheduled Samus projectile update and render passes\"\nagent = \"/root/independent_gameplay\"\nstage = \"{stage}\"\nsource_revision = \"{revision}\"\ninventory_sha256 = \"{inventory}\"\nroutines = [\"HandleProjectile\", \"DrawPlayerExplosions2\", \"sub_9382FD\", \"DrawBombAndProjectileExplosions\"]\ncommit = \"0f59de1c6d8b6fa1bb50b2f66b954424406d0f62\"\nlimitations = [\"476 declared whole-frame fixtures pass actual C; live ownership and merged-source integration remain pending.\", \"PLM and HDMA requests use explicit existing C host dependencies; host-free selection conservatively retains C ownership for callback frames.\", \"Selected Ceres player and explosion drawing branches refuse because pinned C reads uninitialized locals.\", \"Failed diagnostic evidence is retained. Integrated worktree removal remains the coordinator responsibility.\"]\n\n[[evidence]]\npath = \"/home/jim/sm-decomp/out/port/samus-projectile-frame/offline/1791841155185563-2-0/verdict.toml\"\n\n[[evidence]]\npath = \"/home/jim/sm-decomp/.scratch/decomp-port/samus-projectile-frame-476-after-cleanup-authentication.toml\"\n"
        ))?;
        let trail_stage=if stage=="verified" {"verified"} else {"translated"};
        fs::write(format!("/home/jim/sm-decomp/.scratch/decomp-port/projectile-trails-progress-{trail_stage}.toml"),format!(
            "id = \"projectile-trails-cf5f2d8-{trail_stage}\"\nscope = \"Complete native projectile trail script and render frame\"\nagent = \"/root/independent_gameplay\"\nstage = \"{trail_stage}\"\nsource_revision = \"{revision}\"\ninventory_sha256 = \"{inventory}\"\nroutines = [\"HandleProjectileTrails\", \"ProjInstr_MoveLeftProjectileTrailDown\", \"ProjInstr_MoveRightProjectileTrailDown\", \"ProjInstr_MoveLeftProjectileTrailUp\"]\ncommit = \"cf5f2d8\"\nlimitations = [\"3316 controlled actual-C cases and14 naturally active capture fixtures pass; live whole-frame ownership remains pending.\", \"Movement opcodes were covered offline and did not occur in the9000-input natural prefix.\", \"Creation and placement reuse existing projectile_flight ownership.\", \"Integrated worktree removal and fresh combined live replay remain the coordinator responsibility.\"]\n\n[[evidence]]\npath = \"/home/jim/sm-decomp/out/port/projectile-trails/offline/1791838049353069-2-0/verdict.toml\"\n\n[[evidence]]\npath = \"/home/jim/sm-decomp/out/port/projectile-trails/natural-offline/1791838495334191-2-0/verdict.toml\"\n\n[[evidence]]\npath = \"/home/jim/sm-decomp/.scratch/decomp-port/projectile-trails-offline-authentication.toml\"\n\n[[evidence]]\npath = \"/home/jim/sm-decomp/.scratch/decomp-port/projectile-trails-natural-authentication.toml\"\n"
        ))?;
        return Ok(());
    }
    if std::env::args().nth(1).as_deref()==Some("--equivalent") {
        let original:Manifest=toml::from_str(&fs::read_to_string(std::env::args().nth(2).ok_or("original manifest required")?)?)?;
        let current:Manifest=toml::from_str(&fs::read_to_string("/tmp/sm-samus-projectile-frame/tools/port/samus-projectile-frame.toml")?)?;
        if original.routines.len()!=current.routines.len() {return Err("routine count differs".into());}
        let mut checked=0;
        for (a,b) in original.routines.iter().zip(&current.routines) {
            if a.symbol!=b.symbol || a.cases.len()!=b.cases.len() {return Err("routine domain differs".into());}
            for (a,b) in a.cases.iter().zip(&b.cases) {
                if a.name!=b.name || a.continue_from!=b.continue_from || apply(a)!=apply(b) {return Err(format!("input differs: {}",a.name).into());}
                checked+=1;
            }
        }
        println!("equivalent_whole_frame_inputs={checked}");
        return Ok(());
    }
    if std::env::args().nth(1).as_deref()==Some("--retain") {
        let verdict=std::env::args().nth(2).ok_or("verdict required")?;
        let root=std::env::args().nth(3).ok_or("source root required")?;
        let dest=std::env::args().nth(4).ok_or("snapshot required")?;
        let root=Path::new(&root);
        let dest=Path::new(&dest);
        if dest.exists() {return Err("snapshot already exists".into());}
        fn hash(path:&Path)->Result<String>{
            let output=std::process::Command::new("sha256sum").arg(path).output()?;
            if !output.status.success() {return Err(format!("hash failed {}",path.display()).into());}
            Ok(String::from_utf8(output.stdout)?.split_whitespace().next().ok_or("hash missing")?.to_owned())
        }
        let value:toml::Value=toml::from_str(&fs::read_to_string(&verdict)?)?;
        let run=Path::new(&verdict).parent().ok_or("run required")?;
        let implementation=value.get("implementation").ok_or("implementation missing")?;
        let mut files=BTreeMap::new();
        for section in ["rust_dependencies","harness_sources"] {
            for (name,expected) in implementation.get(section).and_then(toml::Value::as_table).ok_or("source seals missing")? {
                let path=Path::new(name);
                if path.is_absolute() {continue;}
                if path.components().any(|c|!matches!(c,std::path::Component::Normal(_))) {return Err("unsafe source path".into());}
                let expected=expected.as_str().ok_or("source hash missing")?;
                let direct=root.join(name);
                let source=if direct.exists() && hash(&direct)?==expected {direct} else {
                    let mut found=None;
                    for entry in fs::read_dir(run)? {
                        let path=entry?.path();
                        if !path.is_file() {continue;}
                        let file=path.file_name().unwrap().to_string_lossy();
                        if value.get("artifacts").and_then(|v|v.get(file.as_ref())).and_then(toml::Value::as_str)==Some(expected) {found=Some(path);break;}
                    }
                    found.ok_or_else(||format!("no sealed source remains for {name}"))?
                };
                files.insert(name.to_owned(),(source,expected.to_owned()));
            }
        }
        let dependencies=root.join("tools/dependencies.toml");
        files.insert("tools/dependencies.toml".to_owned(),(dependencies.clone(),hash(&dependencies)?));
        let mut bytes=0u64;
        let mut hashes=BTreeMap::new();
        for (relative,(source,expected)) in &files {
            if hash(source)?!=*expected {return Err(format!("retained source hash differs: {relative}").into());}
            let target=dest.join(relative);
            fs::create_dir_all(target.parent().unwrap())?;
            bytes+=fs::copy(source,&target)?;
            if hash(&target)?!=*expected {return Err("copied source differs".into());}
            hashes.insert(relative.clone(),expected.clone());
        }
        fs::create_dir_all(dest.join("deps"))?;
        for name in ["snesrev-sm","downloads"] {std::os::unix::fs::symlink(Path::new("/home/jim/sm-decomp/deps").join(name),dest.join("deps").join(name))?;}
        fs::write(dest.join("retention.toml"),format!("source_root = {:?}\nsnapshot = {:?}\nsource_files = {}\nsource_bytes = {bytes}\n\n{}",root.to_string_lossy(),dest.to_string_lossy(),files.len(),toml::to_string(&BTreeMap::from([("files",hashes)]))?))?;
        println!("retained_source_files={} retained_source_bytes={bytes} snapshot={}",files.len(),dest.display());
        return Ok(());
    }
    if std::env::args().nth(1).as_deref()==Some("--audit") {
        let root=std::env::args().nth(2).ok_or("audit run required")?;
        let mut matching=0;
        let mut missing=0;
        let mut mismatching=0;
        for entry in fs::read_dir(root)? {
            let path=entry?.path();
            if !path.join("case.toml").exists() {continue;}
            let c=fs::read(path.join("c-after.wram"));
            let r=fs::read(path.join("rust-after.wram"));
            match (c,r) {
                (Ok(c),Ok(r)) if c==r => matching+=1,
                (Ok(c),Ok(r)) => {mismatching+=1;println!("mismatch={} first={:x?}",path.file_name().unwrap().to_string_lossy(),c.iter().zip(&r).position(|(a,b)|a!=b));},
                _ => {missing+=1;println!("missing={}",path.file_name().unwrap().to_string_lossy());}
            }
        }
        println!("matching={matching} missing={missing} mismatching={mismatching}");
        return Ok(());
    }
    if let Some(path)=std::env::args().nth(1) {
        let mut bytes=fs::read(path)?;
        if let Some(raw)=std::env::args().nth(2) {
            let raw=raw.parse::<usize>()?;
            for k in (0..20).step_by(2) {if k!=raw {word(&mut bytes,0xc40+k,0);}}
        }
        let rom=fs::read("/home/jim/sm-decomp/rom/sm.sfc")?;
        struct DebugHost;
        impl projectile_special::Host for DebugHost {fn spawn_plm(&mut self,_:&mut Wram<'_>,header:u16)->std::result::Result<u8,&'static str>{println!("debug_plm={header:04x}");Ok(0)}}
        impl frame::Host for DebugHost {fn spawn_hdma(&mut self,_:&mut Wram<'_>,script:u16)->std::result::Result<u16,&'static str>{println!("debug_hdma={script:04x}");Ok(0)}}
        for k in (0..20).step_by(2).rev() {if read(&bytes,0xc40+k)!=0 {println!("slot{k} kind={:04x} pre={:04x} ip={:04x} timer={} var={} dir={}",read(&bytes,0xc18+k),read(&bytes,0xc68+k),read(&bytes,0xc40+k),read(&bytes,0xc54+k),read(&bytes,0xc7c+k),read(&bytes,0xc04+k));}}
        if std::env::args().nth(3).as_deref()==Some("prime") {
            for k in (0..20).step_by(2) {if read(&bytes,0xc40+k)!=0 {word(&mut bytes,0xdde,k as u16);word(&mut bytes,0xc54+k,1);projectiles::run_instructions(&mut Wram::new(&mut bytes)?,&rom)?;word(&mut bytes,0xc54+k,2);}}
        }
        let result=frame::dispatch_host(&mut Wram::new(&mut bytes)?,&rom,&mut DebugHost,"HandleProjectile",0,0);
        println!("frame_result={result:?}");
        result?;
        return Ok(());
    }
    let checkout = Path::new("/tmp/sm-samus-projectile-frame");
    let rom = fs::read("/home/jim/sm-decomp/rom/sm.sfc")?;
    let mut scheduler = Vec::new();
    let mut imported = BTreeMap::new();
    let opcodes = [
        ("ProjPreInstr_Beam_NoWaveBeam",0xaef3),("ProjPreInstr_Dir0459",0xaf4a),("ProjPreInstr_Dir1368",0xaf52),("ProjPreInstr_Dir27",0xaf60),
        ("ProjPreInstr_Missile",0xaf68),("ProjPreInstr_Missile_Func0459",0xafc7),("ProjPreInstr_Missile_Func1368",0xafcf),("ProjPreInstr_Missile_Func27",0xafdd),
        ("ProjPreInstr_SuperMissile",0xafe5),("ProjPreInstr_SuperMissile_Func0459",0xb047),("ProjPreInstr_SuperMissile_Func1368",0xb052),("ProjPreInstr_SuperMissile_Func27",0xb06a),
        ("ProjPreInstr_Func1",0xb075),("ProjPreInstr_Bomb",0xb099),("ProjPreInstr_PowerBomb",0xb0ae),
        ("ProjPreInstr_WavePlasmaEtc",0xb0c3),("ProjPreInstr_BeamOrIceWave",0xb0e4),("ProjPreInstr_Wave_Shared",0xb103),("ProjPreInstr_WavePlasmaEtc_0459",0xb13b),("ProjPreInstr_WavePlasmaEtc_1368",0xb143),("ProjPreInstr_WavePlasmaEtc_27",0xb151),("ProjPreInstr_HyperBeam",0xb159),
        ("ProjPreInstr_IceSba",0xcf09),("ProjPreInstr_IceSba2",0xcf7a),("ProjPreInstr_SpeedEcho",0xd4d2),("ProjPreInstr_PlasmaSba",0xd793),
        ("ProjPreInstr_PlasmaSbaFunc_0",0xd7e1),("ProjPreInstr_PlasmaSbaFunc_1",0xd7fa),("ProjPreInstr_PlasmaSbaFunc_2",0xd813),("ProjPreInstr_SpreadBomb",0xd8f7),
        ("ProjPreInstr_WaveSba",0xda08),("ProjPreInstr_SpazerSba",0xdb06),("ProjPreInstr_EndOfSpazerSba",0xdc9c),("ProjPreInstr_UnknownProj8027",0xefd3),
    ];
    for file in ["projectile_flight","projectile_special","power_bomb","projectiles"] {
        let path = checkout.join(format!("tools/port/{file}.toml"));
        if !path.exists() { continue; }
        let manifest: Manifest = toml::from_str(&fs::read_to_string(path)?)?;
        for routine in manifest.routines {
            let Some(&(_,opcode)) = opcodes.iter().find(|(name,_)| *name==routine.symbol) else { continue; };
            let candidates = routine.cases.len();
            let step = candidates.div_ceil(6).max(1);
            for (i,original) in routine.cases.into_iter().enumerate().step_by(step).take(6) {
                let raw = original.a as u16;
                if raw>18 || raw&1!=0 || original.before_file.is_some() || original.continue_from.is_some() { continue; }
                let mut bytes = apply(&original);
                let k = usize::from(raw);
                if read(&bytes,0xdde)!=raw { continue; }
                // Enter each adapted state at the whole scheduler boundary;
                // the original routine inputs and any linked slots are retained.
                if read(&bytes,0xc40+k)==0 {
                    if projectiles::initialize(&mut Wram::new(&mut bytes).unwrap(), &rom, raw).is_err() { continue; }
                }
                word(&mut bytes,0xc68+k,opcode);
                for slot in (0..20).step_by(2) {
                    if slot!=k { word(&mut bytes,0xc40+slot,0); }
                    if read(&bytes,0xc40+slot)!=0 && read(&bytes,0xc68+slot)==0 { word(&mut bytes,0xc68+slot,0xb169); }
                    if read(&bytes,0xc40+slot)!=0 && read(&bytes,0xc54+slot)==0 { word(&mut bytes,0xc54+slot,2); }
                }
                scheduler.push(fixture(format!("native-{file}-{}-{i}",routine.symbol),&bytes));
                *imported.entry(routine.symbol.clone()).or_insert(0usize)+=1;
            }
        }
    }
    scheduler.push(fixture("empty-pool",&base()));
    for direction in 0..10 {
        let mut bytes=base();
        if !initialize(&mut bytes,&rom,0,1,0xb103,direction) {return Err("native wave initialization failed".into());}
        word(&mut bytes,0xc90,1);
        scheduler.push(fixture(format!("direct-wave-shared-direction-{direction}"),&bytes));
    }
    for timer in [0,1,2,0x7fff,0x8000,0xffff] {
        let mut bytes=base();
        for raw in (0..20).step_by(2) { if !initialize(&mut bytes,&rom,raw,0x10,0xb169,2) { return Err("native initialization failed".into()); } word(&mut bytes,0xc54+usize::from(raw),timer); }
        scheduler.push(fixture(format!("all-ten-script-timer-{timer}"),&bytes));
    }
    for &(name,opcode) in &opcodes {
        if imported.contains_key(name) { continue; }
        let mut bytes=base();
        let kind=if opcode==0xb099 {0x300} else if opcode==0xb0ae {0x400} else if name.contains("Missile") {if name.contains("Super") {0x200} else {0x100}} else if name.contains("Sba")||name.contains("SpeedEcho")||name.contains("8027") {0x10} else {0};
        if initialize(&mut bytes,&rom,0,kind,opcode,2) { scheduler.push(fixture(format!("dispatch-{}",name),&bytes)); }
    }
    // Mixed active pools prove scheduler composition and native-ROM script data
    // across ordinary beam, wave, missiles, bomb and inert explosion slots.
    for variant in 0..8 {
        let mut bytes=base();
        for (raw,kind,opcode) in [(0,0,0xaef3),(2,1,0xb0c3),(4,0x100,0xaf68),(6,0x200,0xafe5),(8,0x10,0xb0e4),(10,0x300,0xb099),(12,0x400,0xb0ae),(14,0x500,0xb169),(16,0x700,0xb169),(18,0x800,0xb169)] {
            if initialize(&mut bytes,&rom,raw,kind,opcode,2) {
                // A delayed instruction timer must point past an already
                // consumed native frame before trail creation reads ip-8.
                if variant&1!=0 {
                    word(&mut bytes,0xdde,raw);
                    projectiles::run_instructions(&mut Wram::new(&mut bytes)?,&rom)?;
                }
                word(&mut bytes,0xc54+usize::from(raw), if variant&1==0 {1} else {2});
                word(&mut bytes,0xc90+usize::from(raw),if variant&2==0 {1} else {2});
                // The low byte becomes a linked slot after super-missile
                // acceleration; this deliberately full pool cannot allocate.
                word(&mut bytes,0xc7c+usize::from(raw),if kind==0x200 {2} else if kind==0x100 {240} else if variant&4==0 {2} else {1});
            }
        }
        let first=format!("mixed-pool-{variant}-frame-0");
        scheduler.push(fixture(&first,&bytes));
        for n in 1..=8 { scheduler.push(Case {name:format!("mixed-pool-{variant}-frame-{n}"),continue_from:Some(format!("mixed-pool-{variant}-frame-{}",n-1)),expected_return:Some(0),..Default::default()}); }
    }
    let mut routine_cases = Vec::new();
    for name in ["DrawPlayerExplosions2","sub_9382FD","DrawBombAndProjectileExplosions"] {
        let mut cases=Vec::new();
        for &kind in &[0,1,4,0x10,0x100,0x200,0x300,0x400,0x500,0x700,0x800] {
            for nmi in 0..4 {
                let mut bytes=base();
                word(&mut bytes,0x5b6,nmi);
                for raw in (0..20).step_by(2) { if !initialize(&mut bytes,&rom,raw,kind,0xb169,2) {continue;} projectiles::run_instructions(&mut {word(&mut bytes,0xdde,raw);Wram::new(&mut bytes).unwrap()},&rom).map_err(|e|e.to_owned())?; }
                cases.push(fixture(format!("{}-pool-{kind}-{nmi}",name),&bytes));
            }
        }
        for (i,&(x,y,cursor)) in [(0,0,0),(255,255,508),(256,256,509),(0xffff,0xffff,511),(304,48,512),(0xffd0,8,0xffff),(0xffcf,7,0x203),(303,255,0x81ff),(100,0,0xc45-0x370)].iter().enumerate() {
            let mut bytes=base();
            for raw in (0..20).step_by(2) { initialize(&mut bytes,&rom,raw,0x10,0xb169,2); word(&mut bytes,0xdde,raw); projectiles::run_instructions(&mut Wram::new(&mut bytes).unwrap(),&rom).map_err(|e|e.to_owned())?; word(&mut bytes,0xb64+usize::from(raw),x); word(&mut bytes,0xb78+usize::from(raw),y); }
            word(&mut bytes,0x590,cursor);
            cases.push(fixture(format!("{}-clip-capacity-{i}",name),&bytes));
        }
        // Both ordinary player and explosion Ceres passes use undefined locals
        // in pinned C. The elevated pass has no Mode7 branch and stays valid.
        for &(a,b,c) in &[(0x100,0,0),(0,0x100,0xff00),(0xffff,1,0x8000)] {
            if name!="sub_9382FD" {continue;}
            let mut bytes=base(); word(&mut bytes,0x93f,0x8000); word(&mut bytes,0x78,a);word(&mut bytes,0x7a,b);word(&mut bytes,0x7c,c);word(&mut bytes,0x80,100);word(&mut bytes,0x82,100);
            for raw in (0..20).step_by(2) { let kind=if name=="DrawBombAndProjectileExplosions" {if raw==18 {0x500} else {0x700}} else {0x10}; initialize(&mut bytes,&rom,raw,kind,0xb169,2);word(&mut bytes,0xdde,raw);projectiles::run_instructions(&mut Wram::new(&mut bytes).unwrap(),&rom).map_err(|e|e.to_owned())?; }
            cases.push(fixture(format!("{}-ceres-{a}-{b}-{c}",name),&bytes));
        }
        if name=="DrawPlayerExplosions2" {
            for variant in 0..8 {
                let mut bytes=base();
                word(&mut bytes,0x5b6,variant&1);
                word(&mut bytes,0xac6,1);word(&mut bytes,0xac4,1);
                word(&mut bytes,0xab6,20);word(&mut bytes,0xab4,40);
                word(&mut bytes,0xabe,if variant&2==0 {30} else {0xffff});
                word(&mut bytes,0xabc,if variant&2==0 {50} else {256});
                word(&mut bytes,0xac8,0);word(&mut bytes,0xaca,if variant&4==0 {0} else {1});
                word(&mut bytes,0xa78,1);word(&mut bytes,0xd658,5);
                word(&mut bytes,0xd730,60);word(&mut bytes,0xd778,70);word(&mut bytes,0xd6e8,0x4321);
                cases.push(fixture(format!("player-echo-before-frozen-trails-{variant}"),&bytes));
            }
        }
        routine_cases.push(Routine{symbol:name.to_owned(),args:0,returns:"void".to_owned(),live_eligible:Some(true),cases,..Default::default()});
    }
    let natural=Path::new("/home/jim/sm-decomp/out/port/projectile-trails/natural-9000-corrected");
    for routine in &mut routine_cases {
        for line in fs::read_to_string(natural.join("trail-captures.tsv"))?.lines().skip(1) {
            let fields=line.split('\t').collect::<Vec<_>>();
            let before_file=natural.join(fields[7]);
            let hash=std::process::Command::new("sha256sum").arg(&before_file).output()?;
            if !hash.status.success() {return Err("natural input hash failed".into());}
            let before_sha256=String::from_utf8(hash.stdout)?.split_whitespace().next().ok_or("natural input hash missing")?.to_owned();
            routine.cases.push(Case {
                name:format!("{}-natural-active-{}",routine.symbol,fields[0]),
                before_file:Some(before_file),before_sha256:Some(before_sha256),expected_return:Some(0),..Default::default()
            });
        }
    }
    let mut routines=vec![Routine {symbol:"HandleProjectile".to_owned(),args:0,returns:"void".to_owned(),live_eligible:Some(true),cases:scheduler,..Default::default()}];
    routines.extend(routine_cases);
    let manifest=Manifest {
        family:"samus-projectile-frame".to_owned(),rust_module:checkout.join("crates/port/src/samus_projectile_frame.rs").display().to_string(),
        rust_imports:vec!["power_bomb".to_owned(),"projectile_flight".to_owned(),"projectile_special".to_owned(),"projectile_trails".to_owned(),"projectiles".to_owned()],
        bank:"sm_90.c".to_owned(),rom:Some(model::Asset{file:"/home/jim/sm-decomp/rom/sm.sfc".into(),sha256:"12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72".to_owned()}),
        domain:Some("Whole ten-slot Samus projectile scheduling and both player/explosion render passes, existing Rust flight/bomb/SBA composition, native ROM script chains, full WRAM/return and ordered explicit PLM/HDMA callbacks. Pinned-C Ceres player draw uses an uninitialized Y local and Ceres explosion draw discards its Mode7 point; selected paths refuse before mutation.".to_owned()),
        callback_adapter:Some("power-bomb".to_owned()),plm_amount_context:Some("none".to_owned()),routines,..Default::default()
    };
    fs::write(checkout.join("tools/port/samus-projectile-frame.toml"),toml::to_string(&manifest)?)?;
    for routine in &manifest.routines { println!("{} cases={}",routine.symbol,routine.cases.len()); }
    println!("imported_native_dependency_domains={imported:?}");
    Ok(())
}
fn main(){if let Err(e)=run(){eprintln!("whole-frame fixture adaptation: {e}");std::process::exit(1);}}
