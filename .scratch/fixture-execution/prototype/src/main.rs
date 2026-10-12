//! THROWAWAY: bounded real-fixture experiment, no production admission changes.
#![allow(dead_code)]
use anyhow::{Context,Result,ensure};
use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha256};
use std::{collections::{BTreeMap},fs::{self,File,OpenOptions},io::{Read,Write},os::{fd::AsRawFd,unix::process::CommandExt},path::{Path,PathBuf},process::{Child,Command,Stdio},sync::{Mutex,atomic::{AtomicUsize,Ordering}},time::{Duration,Instant}};
#[path="/home/jim/sm-decomp/crates/pipeline/src/metrics.rs"] mod metrics;
#[path="/home/jim/sm-decomp/crates/pipeline/src/resource_sampler.rs"] mod resource_sampler;
const ORIGINAL:&str="/tmp/sm-pipeline-upgrade-integration/out/ticket-03/final-projectile/1791813609706347-2-0";
const PROTOTYPE:&str="/tmp/sm-fixture-execution-prototype";
fn atomic_toml(path:&Path,value:&impl Serialize)->Result<()>{
    let pending=path.with_extension("pending");let mut f=OpenOptions::new().create_new(true).write(true).open(&pending)?;
    f.write_all(toml::to_string_pretty(value)?.as_bytes())?;f.sync_all()?;fs::rename(pending,path)?;File::open(path.parent().unwrap())?.sync_all()?;Ok(())
}
fn digest(path:&Path)->Result<String>{Ok(format!("{:x}",Sha256::digest(fs::read(path)?)))}
#[derive(Clone)]struct Fixture{name:String,symbol:String,value:toml::Value,previous:Option<String>,expected:Option<u64>}
fn fixtures()->Result<Vec<Vec<Fixture>>>{
    let manifest:toml::Value=toml::from_str(&fs::read_to_string(Path::new(ORIGINAL).join("manifest.toml"))?)?;
    let mut fixtures=BTreeMap::new();
    for routine in manifest["routines"].as_array().context("routines")?{
        for value in routine["cases"].as_array().context("cases")?{
            let name=value["name"].as_str().unwrap().to_owned();
            fixtures.insert(name.clone(),Fixture{name,symbol:routine["symbol"].as_str().unwrap().into(),value:value.clone(),previous:value.get("continue_from").and_then(|v|v.as_str()).map(str::to_owned),expected:value.get("expected_return").and_then(|v|v.as_integer()).map(|n|n as u64)});
        }
    }
    let mut chains:Vec<Vec<Fixture>>=vec![];let mut owners=BTreeMap::new();
    for name in manifest["case_order"].as_array().context("order")?{
        let fixture=fixtures.remove(name.as_str().unwrap()).context("fixture")?;
        let owner=if let Some(previous)=&fixture.previous{*owners.get(previous).context("predecessor")?}else{chains.push(vec![]);chains.len()-1};
        owners.insert(fixture.name.clone(),owner);chains[owner].push(fixture);
    }
    ensure!(fixtures.is_empty(),"missing fixtures");Ok(chains)
}
fn prepare(fixture:&Fixture,run:&Path)->Result<PathBuf>{
    let dir=run.join(&fixture.name);fs::create_dir(&dir)?;
    let original=Path::new(ORIGINAL).join(&fixture.name);
    let (mut before,host)=if let Some(previous)=&fixture.previous{
        let previous=run.join(previous);let c=fs::read(previous.join("c-after.wram"))?;
        ensure!(c==fs::read(previous.join("rust-after.wram"))?,"inherited WRAM differs");
        let h=fs::read(previous.join("c-host-after.bin"))?;ensure!(h==fs::read(previous.join("rust-host-after.bin"))?,"inherited host differs");(c,h)
    }else{(fs::read(original.join("before.wram"))?,fs::read(original.join("host-before.bin"))?)};
    if fixture.previous.is_some(){
        for write in fixture.value.get("writes").and_then(|v|v.as_array()).into_iter().flatten(){
            let offset=write[0].as_integer().context("write offset")? as usize;
            let data=write[1].as_array().context("write data")?;ensure!(offset+data.len()<=before.len(),"write extent");
            for (i,value) in data.iter().enumerate(){before[offset+i]=value.as_integer().unwrap() as u8;}
        }
    }
    ensure!(before==fs::read(original.join("before.wram"))? && host==fs::read(original.join("host-before.bin"))?,"input differs from retained fixture");
    fs::write(dir.join("before.wram"),before)?;fs::write(dir.join("host-before.bin"),host)?;
    atomic_toml(&dir.join("case.toml"),&fixture.value)?;Ok(dir)
}
fn argv(f:&Fixture,dir:&Path,subject:&str)->Vec<String>{
    let number=|key:&str|f.value.get(key).and_then(|v|v.as_integer()).unwrap_or(0).to_string();
    vec![Path::new(ORIGINAL).join(subject).display().to_string(),f.symbol.clone(),number("a"),number("b"),dir.join("before.wram").display().to_string(),dir.join(format!("{subject}-after.wram")).display().to_string(),Path::new(ORIGINAL).join("rom.bin").display().to_string(),dir.join("host-before.bin").display().to_string(),dir.join(format!("{subject}-host-after.bin")).display().to_string()]
}
#[derive(Serialize,Deserialize)]struct Invoke{id:u64,subject:String,argv:Vec<String>,directory:String,action:String,timeout_ms:u64}
#[derive(Serialize,Deserialize)]struct Reply{id:u64,pid:u32,status:i32,elapsed_ns:u64,cpu_ns:u64,reaps:u32,timed_out:bool,parent_pristine:bool,descendants_reaped:u32}
struct Helper{child:Child,next:u64}
impl Helper{
    fn spawn()->Result<Self>{
        let child=Command::new(Path::new(PROTOTYPE).join("target/release/fixture-helper"))
            .arg(Path::new(ORIGINAL).join("rom.bin")).process_group(0).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn()?;
        let mut this=Self{child,next:0};let ready:toml::Value=this.receive(Instant::now()+Duration::from_secs(5))?;
        ensure!(ready.get("ready").and_then(|v|v.as_bool())==Some(true),"helper not ready");Ok(this)
    }
    fn receive<T:serde::de::DeserializeOwned>(&mut self,deadline:Instant)->Result<T>{
        fn read(reader:&mut std::process::ChildStdout,bytes:&mut [u8],deadline:Instant)->Result<()>{
            let mut offset=0;while offset<bytes.len(){
                metrics::check_cancelled()?;ensure!(Instant::now()<deadline,"helper reply timeout");
                let mut poll=libc::pollfd{fd:reader.as_raw_fd(),events:libc::POLLIN,revents:0};
                let result=unsafe{libc::poll(&mut poll,1,10)};if result<0{continue;}if result==0{continue;}
                let n=reader.read(&mut bytes[offset..])?;ensure!(n>0,"truncated helper reply");offset+=n;
            }Ok(())
        }
        let out=self.child.stdout.as_mut().unwrap();let mut header=[0;4];read(out,&mut header,deadline)?;
        let n=u32::from_le_bytes(header) as usize;ensure!(n<=65536,"oversized reply");let mut bytes=vec![0;n];read(out,&mut bytes,deadline)?;
        Ok(toml::from_str(std::str::from_utf8(&bytes)?)?)
    }
    fn request(&mut self,args:Vec<String>,dir:&Path,subject:&str,action:&str,timeout:u64)->Result<Reply>{
        let id=self.next;self.next+=1;let value=Invoke{id,subject:subject.into(),argv:args,directory:dir.display().to_string(),action:action.into(),timeout_ms:timeout};
        let bytes=toml::to_string(&value)?.into_bytes();let input=self.child.stdin.as_mut().unwrap();input.write_all(&(bytes.len() as u32).to_le_bytes())?;input.write_all(&bytes)?;input.flush()?;
        let reply:Reply=self.receive(Instant::now()+Duration::from_millis(timeout+2000))?;
        ensure!(reply.id==id && reply.reaps==1 && reply.parent_pristine,"invalid helper identity/state");Ok(reply)
    }
    fn stop(&mut self)->Result<()>{self.child.stdin.take();let deadline=Instant::now()+Duration::from_secs(2);loop{if let Some(status)=self.child.try_wait()?{ensure!(status.success(),"helper failed");return Ok(());}ensure!(Instant::now()<deadline,"helper exit timeout");std::thread::sleep(Duration::from_millis(1));}}
}
impl Drop for Helper{fn drop(&mut self){
    if matches!(self.child.try_wait(),Ok(Some(_))){return;}
    unsafe{libc::kill(self.child.id() as i32,libc::SIGTERM);}self.child.stdin.take();
    let deadline=Instant::now()+Duration::from_secs(2);
    loop{if matches!(self.child.try_wait(),Ok(Some(_))){return;}if Instant::now()>=deadline{break;}std::thread::sleep(Duration::from_millis(1));}
    unsafe{libc::kill(-(self.child.id() as i32),libc::SIGKILL);}let _=self.child.wait();
}}
struct Lease(File);
impl Lease{fn new()->Result<Self>{let root=Path::new("/home/jim/sm-decomp/out/pipeline/workers");fs::create_dir_all(root)?;
    let capacity:toml::Value=toml::from_str(&fs::read_to_string(root.join("capacity.toml"))?)?;
    let n=capacity["workers"].as_integer().unwrap();let started=Instant::now();loop{metrics::check_cancelled()?;for i in 0..n{let file=OpenOptions::new().create(true).append(true).open(root.join(format!("{i}.lock")))?;if file.try_lock().is_ok(){return Ok(Self(file));}}ensure!(started.elapsed()<Duration::from_secs(10),"worker wait exceeds bounded experiment");std::thread::sleep(Duration::from_millis(1));}}}
fn execute(args:&[String],dir:&Path,subject:&str,baseline:bool)->Result<u64>{
    let mut scope=baseline.then(||metrics::Run::start(dir.join(format!("{subject}.metrics.toml")),subject)).transpose()?;
    let context=scope.as_ref().map(|s|s.context.clone()).or_else(metrics::active).context("missing metrics scope")?;
    let mut command=Command::new(&args[0]);command.args(&args[1..]).current_dir(dir).stdout(File::create(dir.join(format!("{subject}.stdout")))?).stderr(File::create(dir.join(format!("{subject}.stderr")))?);
    let start=Instant::now();let mut child=metrics::OwnedChild::spawn(&mut command,&context,subject)?;
    let status=loop{if let Some(status)=child.poll()?{break status;}ensure!(start.elapsed()<Duration::from_secs(2),"fixture timeout");std::thread::sleep(if baseline{Duration::from_millis(20)}else{Duration::from_micros(200)});};
    atomic_toml(&dir.join(format!("{subject}.timing.toml")),&toml::Value::Table(toml::Table::from_iter([("argv".into(),toml::Value::try_from(args)?),("seconds".into(),start.elapsed().as_secs_f64().into()),("exit_code".into(),(status.code().unwrap_or(-1) as i64).into())])))?;
    if let Some(scope)=&mut scope{scope.finish(if status.success(){"success"}else{"failed"},None)?;}
    ensure!(status.success(),"{subject} refused or failed");let snapshot=context.snapshot();let cpu=snapshot.processes.iter().rev().find(|p|p.argv==args).context("missing child CPU")?;
    let value=|v:&pipeline_model::metrics::Measurement|match v{pipeline_model::metrics::Measurement::Measured{value}=>*value,_=>0};Ok(value(&cpu.user_cpu_ns)+value(&cpu.system_cpu_ns))
}
fn verify(f:&Fixture,dir:&Path)->Result<BTreeMap<String,String>>{
    let mut result=BTreeMap::new();let original=Path::new(ORIGINAL).join(&f.name);
    for suffix in ["after.wram","host-after.bin"]{
        let c=fs::read(dir.join(format!("c-{suffix}")))?;let rust=fs::read(dir.join(format!("rust-{suffix}")))?;
        ensure!(c==rust && c==fs::read(original.join(format!("c-{suffix}")))?,"state mismatch {} {suffix}",f.name);
    }
    for subject in ["rust","c"]{let stdout=fs::read_to_string(dir.join(format!("{subject}.stdout")))?;
        let n=stdout.lines().last().context("return missing")?.parse::<u64>()?;
        let expected=fs::read_to_string(original.join(format!("{subject}.stdout")))?;ensure!(stdout==expected,"return text differs");
        if let Some(value)=f.expected{ensure!(value==n,"independent expected return failed");}
        for suffix in ["after.wram","host-after.bin","stdout","stderr"]{let name=format!("{subject}-{suffix}");let path=if ["stdout","stderr"].contains(&suffix){dir.join(format!("{subject}.{suffix}"))}else{dir.join(&name)};result.insert(path.file_name().unwrap().to_string_lossy().to_string(),digest(&path)?);}
    }
    result.insert("before.wram".into(),digest(&dir.join("before.wram"))?);result.insert("host-before.bin".into(),digest(&dir.join("host-before.bin"))?);Ok(result)
}
fn pair(f:&Fixture,dir:&Path,mode:&str,helper:&mut Option<Helper>,cpu:&mut u64)->Result<()> {
    for subject in ["rust","c"] {
        let args=argv(f,dir,subject);
        if let Some(helper)=helper {
            let reply=helper.request(args.clone(),dir,subject,"invoke",2000)?;*cpu+=reply.cpu_ns;
            let timing=toml::Value::Table(toml::Table::from_iter([("argv".into(),toml::Value::try_from(&args)?),("reply".into(),toml::Value::try_from(&reply)?)]));
            atomic_toml(&dir.join(format!("{subject}.timing.toml")),&timing)?;
            ensure!(libc::WIFEXITED(reply.status)&&libc::WEXITSTATUS(reply.status)==0&&!reply.timed_out,"subject failure");
        }else{*cpu+=execute(&args,dir,subject,mode=="baseline")?;}
    }
    Ok(())
}
#[derive(Serialize)]struct Sample{mode:String,jobs:usize,fixtures:usize,elapsed_ns:u64,cpu_ns:u64,equality:bool,signatures:BTreeMap<String,BTreeMap<String,String>>,load_before:String,load_after:String,scope:String}
fn experiment(chains:&[Vec<Fixture>],path:&Path,mode:&str,jobs:usize)->Result<Sample>{
    fs::create_dir(path)?;let load_before=fs::read_to_string("/proc/loadavg")?;let next=AtomicUsize::new(0);let outcomes=Mutex::new(Vec::new());let started=Instant::now();
    let context=metrics::active().context("missing parent scope")?;
    std::thread::scope(|scope|{for _ in 0..jobs{scope.spawn(||{let _entered=context.enter();let result=(||->Result<_>{let _lease=Lease::new()?;let mut helper=if mode=="fork"{Some(Helper::spawn()?)}else{None};let mut completed=BTreeMap::new();let mut cpu=0;
        loop{metrics::check_cancelled()?;ensure!(started.elapsed()<Duration::from_secs(55),"bounded experiment timeout");let index=next.fetch_add(1,Ordering::Relaxed);let Some(chain)=chains.get(index)else{break;};let mut failure:Option<String>=None;
            for f in chain{if let Some(previous)=&failure{let dir=path.join(&f.name);fs::create_dir(&dir)?;atomic_toml(&dir.join("blocked.toml"),&toml::Value::Table(toml::Table::from_iter([("blocked_by".into(),toml::Value::String(previous.clone()))])))?;continue;}
                let result=(||->Result<_>{let dir=prepare(f,path)?;pair(f,&dir,mode,&mut helper,&mut cpu)?;verify(f,&dir)})();
                match result{Ok(record)=>{completed.insert(f.name.clone(),record);},Err(error)=>{eprintln!("{}: {error:#}",f.name);failure=Some(f.name.clone());}}
            }
        }
        if let Some(helper)=&mut helper{helper.stop()?;}Ok((completed,cpu))})();outcomes.lock().unwrap().push(result);});}});
    cleanup(path)?;
    let elapsed_ns=started.elapsed().as_nanos() as u64;let mut signatures=BTreeMap::new();let mut cpu_ns=0;for result in outcomes.into_inner().unwrap(){let (record,cpu)=result?;signatures.extend(record);cpu_ns+=cpu;}
    let total=chains.iter().map(Vec::len).sum();ensure!(signatures.len()==total,"missing or blocked fixtures");
    let report=Sample{mode:mode.into(),jobs,fixtures:total,elapsed_ns,cpu_ns,equality:true,signatures,load_before,load_after:fs::read_to_string("/proc/loadavg")?,scope:"prepared binaries; includes helper startup/ROM load, worker wait, fixture prepare/execution/comparison, per-case evidence/timing writes, helper shutdown and cleanup/dedup; excludes compilation and final summary writing".into()};atomic_toml(&path.join("sample.toml"),&report)?;println!("{} jobs={} fixtures={} wall={:.3}s CPU={:.3}s",mode,jobs,total,elapsed_ns as f64/1e9,cpu_ns as f64/1e9);Ok(report)
}
fn cleanup(root:&Path)->Result<()>{
    for deduplicate in [false,true]{let mut child=Command::new("/tmp/sm-pipeline-upgrade-integration/out/ticket-03/retained-tools/sm-pipeline-final").arg("cleanup").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
        let request=toml::Value::Table(toml::Table::from_iter([("root".into(),root.display().to_string().into()),("apply".into(),true.into()),("deduplicate".into(),deduplicate.into())]));child.stdin.take().unwrap().write_all(toml::to_string(&request)?.as_bytes())?;
        let output=child.wait_with_output()?;ensure!(output.status.success(),"cleanup failed: {}",String::from_utf8_lossy(&output.stderr));fs::write(root.join(if deduplicate{"dedup.toml"}else{"cleanup.toml"}),output.stdout)?;
    }Ok(())
}
fn controls(chains:&[Vec<Fixture>],root:&Path)->Result<()>{
    let _lease=Lease::new()?;let mut helper=Helper::spawn()?;let f=&chains[0][0];
    for (id,action) in ["mutate","invoke","crash","invoke","sleep","invoke","descendant","invoke"].into_iter().enumerate(){let dir=root.join(format!("control-{id}"));fs::create_dir(&dir)?;fs::copy(Path::new(ORIGINAL).join(&f.name).join("before.wram"),dir.join("before.wram"))?;fs::copy(Path::new(ORIGINAL).join(&f.name).join("host-before.bin"),dir.join("host-before.bin"))?;
        let reply=helper.request(argv(f,&dir,"c"),&dir,"c",action,100)?;atomic_toml(&dir.join("reply.toml"),&reply)?;
        if action=="invoke"{ensure!(reply.status==0,"recovery failed");ensure!(fs::read(dir.join("c-after.wram"))?==fs::read(Path::new(ORIGINAL).join(&f.name).join("c-after.wram"))?,"leak after control");}else if action=="sleep"{ensure!(reply.timed_out,"timeout not recorded");}else if action=="crash"{ensure!(libc::WIFSIGNALED(reply.status),"crash not recorded");}
        if action=="descendant"{ensure!(reply.descendants_reaped==1 && reply.timed_out,"descendant not reaped");}
        ensure!(unsafe{libc::kill(reply.pid as i32,0)}<0,"child survives reply");
    }
    let dir=root.join("rust-refusal");fs::create_dir(&dir)?;let mut args=argv(f,&dir,"rust");args[1]="UnsupportedPrototypeRoutine".into();fs::copy(Path::new(ORIGINAL).join(&f.name).join("before.wram"),dir.join("before.wram"))?;fs::copy(Path::new(ORIGINAL).join(&f.name).join("host-before.bin"),dir.join("host-before.bin"))?;
    let reply=helper.request(args,&dir,"rust","invoke",100)?;ensure!(libc::WIFEXITED(reply.status)&&libc::WEXITSTATUS(reply.status)!=0,"refusal not rejected");ensure!(!dir.join("c.stdout").exists(),"C ran after refusal");atomic_toml(&dir.join("reply.toml"),&reply)?;helper.stop()?;
    let mut refused=f.clone();refused.symbol="UnsupportedPrototypeRoutine".into();let dir=root.join("pair-refusal");fs::create_dir(&dir)?;
    fs::copy(Path::new(ORIGINAL).join(&f.name).join("before.wram"),dir.join("before.wram"))?;fs::copy(Path::new(ORIGINAL).join(&f.name).join("host-before.bin"),dir.join("host-before.bin"))?;
    let mut helper=Some(Helper::spawn()?);ensure!(pair(&refused,&dir,"fork",&mut helper,&mut 0).is_err(),"pair refusal admitted");ensure!(!dir.join("c.stdout").exists(),"pair ran C after refusal");drop(helper);
    let dir=root.join("truncated-reply");fs::create_dir(&dir)?;let mut helper=Helper::spawn()?;
    let error=helper.request(argv(f,&dir,"rust"),&dir,"rust","truncate",100).err().context("truncated reply admitted")?;
    ensure!(error.to_string().contains("truncated"),"unexpected framing error");drop(helper);
    let dir=root.join("cancelled-helper");fs::create_dir(&dir)?;let mut helper=Helper::spawn()?;
    let request=Invoke{id:0,subject:"c".into(),argv:argv(f,&dir,"c"),directory:dir.display().to_string(),action:"descendant".into(),timeout_ms:2000};
    let bytes=toml::to_string(&request)?.into_bytes();let input=helper.child.stdin.as_mut().unwrap();input.write_all(&(bytes.len() as u32).to_le_bytes())?;input.write_all(&bytes)?;input.flush()?;
    let deadline=Instant::now()+Duration::from_secs(2);while !dir.join("descendant.pid").exists(){ensure!(Instant::now()<deadline,"descendant failed to start");std::thread::sleep(Duration::from_millis(1));}
    let helper_pid=helper.child.id();drop(helper);
    for path in ["child.pid","descendant.pid"]{let pid:i32=fs::read_to_string(dir.join(path))?.parse()?;ensure!(unsafe{libc::kill(pid,0)}<0,"{path} survives cancellation");}
    ensure!(unsafe{libc::kill(helper_pid as i32,0)}<0,"helper survives cancellation");
    let dir=root.join("orchestrator-cancel");let mut process=Command::new(std::env::current_exe()?).arg("cancel-case").arg(&dir).stdout(File::create(root.join("cancel.stdout"))?).stderr(File::create(root.join("cancel.stderr"))?).spawn()?;
    let deadline=Instant::now()+Duration::from_secs(3);while !dir.join("descendant.pid").exists(){ensure!(Instant::now()<deadline,"cancellation fixture failed to start");std::thread::sleep(Duration::from_millis(1));}
    unsafe{libc::kill(process.id() as i32,libc::SIGTERM);}let status=process.wait()?;ensure!(!status.success(),"cancelled orchestrator admitted success");
    for path in ["helper.pid","child.pid","descendant.pid"]{let pid:i32=fs::read_to_string(dir.join(path))?.parse()?;ensure!(unsafe{libc::kill(pid,0)}<0,"{path} survives orchestrator cancellation");}
    let mut blocked=chains.iter().take(2).map(|c|c.iter().take(3).cloned().collect::<Vec<_>>()).collect::<Vec<_>>();blocked[0][0].expected=Some(u64::MAX);
    ensure!(experiment(&blocked,&root.join("blocked-control"),"fork",2).is_err(),"failed fixture admitted");
    ensure!(root.join("blocked-control").join(&blocked[0][1].name).join("blocked.toml").is_file(),"descendant ran");
    ensure!(root.join("blocked-control").join(&blocked[1][2].name).join("c.stdout").is_file(),"independent chain stopped");
    atomic_toml(&root.join("controls.toml"),&toml::Value::Table(toml::Table::from_iter([("isolation_crash_timeout_refusal_blocking".into(),true.into())])))?;println!("isolation/crash/timeout/refusal/blocked-descendant controls passed");Ok(())
}
fn main()->Result<()>{
    let operation=std::env::args().nth(1).unwrap_or_else(||"run".into());let output=PathBuf::from(std::env::args().nth(2).unwrap_or_else(||format!("{PROTOTYPE}/runs")));fs::create_dir_all(&output)?;
    let mut run=metrics::Run::start(output.join("prototype.metrics.toml"),"prototype")?;
    let all=fixtures()?;
    let result=(||->Result<()>{
        let verdict:toml::Value=toml::from_str(&fs::read_to_string(Path::new(ORIGINAL).join("verdict.toml"))?)?;
        for name in ["c","rust"]{ensure!(digest(&Path::new(ORIGINAL).join(name))?==verdict["implementation"]["binaries"][name].as_str().unwrap(),"retained binary changed");}
        ensure!(digest(&Path::new(ORIGINAL).join("rom.bin"))?==verdict["rom"]["sha256"].as_str().unwrap(),"ROM changed");
        if operation=="controls"{return controls(&all,&output);}
        if operation=="cancel-case" {let _lease=Lease::new()?;let mut helper=Helper::spawn()?;fs::write(output.join("helper.pid"),helper.child.id().to_string())?;helper.request(argv(&all[0][0],&output,"c"),&output,"c","descendant",2000)?;anyhow::bail!("cancellation fixture returned unexpectedly");}
        if operation=="full"{experiment(&all,&output.join("full-fork-2"),"fork",2)?;return Ok(());}
        let mut sampled=all.iter().map(|chain|chain.iter().take(6).cloned().collect::<Vec<_>>()).collect::<Vec<_>>();
        for chain in all.iter().take(12){for index in [70,100]{let mut fixture=chain[index].clone();fixture.previous=None;sampled.push(vec![fixture]);}}
        let order=[("baseline",1),("fork",2),("fresh",2),("fresh",1),("fork",1),("fork",2),("baseline",1)];let mut original=None;
        for (index,(mode,jobs)) in order.into_iter().enumerate(){let report=experiment(&sampled,&output.join(format!("sample-{index}-{mode}-{jobs}")),mode,jobs)?;if let Some(expected)=&original{ensure!(&report.signatures==expected,"cross-mode evidence differs");}else{original=Some(report.signatures.clone());}}
        let mut reversed=sampled;reversed.reverse();let report=experiment(&reversed,&output.join("reordered-fork-2"),"fork",2)?;ensure!(Some(&report.signatures)==original.as_ref(),"order changed results");Ok(())
    })();run.finish(if result.is_ok(){"success"}else{"failed"},result.as_ref().err().map(|e|format!("{e:#}")))?;cleanup(&output)?;result
}
