//! THROWAWAY: a single-threaded preloaded image, fresh fork per invocation.
#![allow(dead_code, unused_imports, unused_variables)]
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{ffi::CString, io::{Read, Write}, path::Path, time::{Duration, Instant}};
use std::sync::atomic::{AtomicBool, Ordering};
static CANCELLED: AtomicBool = AtomicBool::new(false);
extern "C" fn cancel(_: libc::c_int) { CANCELLED.store(true, Ordering::Relaxed); }

#[path="/tmp/sm-pipeline-upgrade-integration/out/ticket-03/final-projectile/1791813609706347-2-0/memory.rs"]
mod memory;
use memory::*;

#[link(name="fixture_oracle",kind="static")]
unsafe extern "C" {
    fn PortPrototypeInvoke(argc: libc::c_int, argv: *mut *mut libc::c_char) -> libc::c_int;
    static mut g_rom: *const u8;
    static mut g_ram: [u8;0x20000];
    fn fflush(stream: *mut libc::FILE) -> libc::c_int;
}

#[derive(Deserialize)]
struct Request {
    id: u64,
    subject: String,
    argv: Vec<String>,
    directory: String,
    action: String,
    timeout_ms: u64,
}
#[derive(Serialize)]
struct Reply {
    id: u64,
    pid: u32,
    status: i32,
    elapsed_ns: u64,
    cpu_ns: u64,
    reaps: u32,
    timed_out: bool,
    parent_pristine: bool,
    descendants_reaped: u32,
}
fn emit(value: &impl Serialize) -> Result<()> {
    let bytes=toml::to_string(value)?.into_bytes();
    let mut out=std::io::stdout().lock();
    out.write_all(&(bytes.len() as u32).to_le_bytes())?;out.write_all(&bytes)?;out.flush()?;
    Ok(())
}
fn read_request() -> Result<Option<Request>> {
    let mut input=std::io::stdin().lock(); let mut header=[0;4];
    match input.read_exact(&mut header) {
        Err(e) if e.kind()==std::io::ErrorKind::UnexpectedEof => return Ok(None),
        result => result?,
    }
    let n=u32::from_le_bytes(header) as usize;ensure!(n<=65536,"request too large");
    let mut bytes=vec![0;n];input.read_exact(&mut bytes)?;
    Ok(Some(toml::from_str(std::str::from_utf8(&bytes)?)?))
}
fn invoke_rust(args: &[String], rom: &[u8]) -> Result<()> {
    let mut bytes=std::fs::read(&args[4])?;
    let raw=std::fs::read(&args[7])?;ensure!(raw.len()==16,"host extent");
    let mut words=[0;8];for (i,w) in words.iter_mut().enumerate(){*w=u16::from_le_bytes([raw[i*2],raw[i*2+1]]);}
    let mut host=memory::enemy_projectiles::ProjectileHost::from_words(words);
    let value=memory::observed_projectile_ownership::dispatch_host(
        &mut Wram::new(&mut bytes).map_err(anyhow::Error::msg)?,rom,&mut host,&args[1],
        args[2].parse::<i64>()? as i32,args[3].parse::<i64>()? as i32,
    ).map_err(anyhow::Error::msg)?;
    std::fs::write(&args[8],host.words().into_iter().flat_map(u16::to_le_bytes).collect::<Vec<_>>())?;
    std::fs::write(&args[5],bytes)?;println!("{value}");Ok(())
}
fn child(request:&Request,rom:&[u8],args:&mut [*mut libc::c_char])->Result<i32>{
    let directory=Path::new(&request.directory);
    let stdout=std::fs::File::create(directory.join(format!("{}.stdout",request.subject)))?;
    let stderr=std::fs::File::create(directory.join(format!("{}.stderr",request.subject)))?;
    use std::os::fd::AsRawFd;
    unsafe {ensure!(libc::dup2(stdout.as_raw_fd(),1)>=0 && libc::dup2(stderr.as_raw_fd(),2)>=0,"redirect failed");}
    std::env::set_current_dir(directory)?;
    std::fs::write(directory.join("child.pid"),std::process::id().to_string())?;
    if request.action=="crash" {unsafe{libc::raise(libc::SIGKILL);}return Ok(99);}
    if request.action=="sleep" {std::thread::sleep(Duration::from_secs(30));return Ok(99);}
    if request.action=="descendant" {
        let pid=unsafe{libc::fork()};ensure!(pid>=0,"descendant fork failed");
        if pid==0 { unsafe { libc::prctl(libc::PR_SET_PDEATHSIG,libc::SIGKILL,0,0,0); }
            std::fs::write(directory.join("descendant.pid"),std::process::id().to_string())?;
            std::thread::sleep(Duration::from_secs(30));unsafe{libc::_exit(0);}
        }
        std::thread::sleep(Duration::from_secs(30));return Ok(99);
    }
    if request.action=="mutate" {
        unsafe {g_ram[17]=99;*(g_rom as *mut u8)=rom[0]^255;}
        return Ok(0);
    }
    if request.subject=="c" {Ok(unsafe {PortPrototypeInvoke(9,args.as_mut_ptr())})}
    else {invoke_rust(&request.argv,rom)?;Ok(0)}
}
fn main()->Result<()> {
    let rom=std::fs::read(std::env::args().nth(1).context("ROM required")?)?;
    ensure!(!rom.is_empty() && rom.len()<=0x400000,"ROM extent");
    let original_first=rom[0];
    let mut c_rom=rom.clone();
    unsafe{g_rom=c_rom.as_mut_ptr();ensure!(libc::prctl(libc::PR_SET_CHILD_SUBREAPER,1,0,0,0)==0,"subreaper failed");
        let mut action:libc::sigaction=std::mem::zeroed();action.sa_sigaction=cancel as *const () as usize;
        ensure!(libc::sigaction(libc::SIGTERM,&action,std::ptr::null_mut())==0,"signal handler failed");}
    emit(&toml::Value::Table(toml::Table::from_iter([("ready".into(),true.into()),("pid".into(),(std::process::id() as i64).into())])))?;
    while let Some(request)=read_request()? {
        if CANCELLED.load(Ordering::Relaxed){break;}
        if request.action=="truncate" {let mut out=std::io::stdout().lock();out.write_all(&16u32.to_le_bytes())?;out.write_all(b"xx")?;out.flush()?;return Ok(());}
        ensure!(request.subject=="rust" || request.subject=="c","unknown subject");
        ensure!(request.argv.len()==9 && request.timeout_ms<=60000,"invalid invocation");
        let strings=request.argv.iter().map(|s|CString::new(s.as_bytes())).collect::<std::result::Result<Vec<_>,_>>()?;
        let mut args=strings.iter().map(|s|s.as_ptr() as *mut libc::c_char).collect::<Vec<_>>();args.push(std::ptr::null_mut());
        std::io::stdout().flush()?;std::io::stderr().flush()?;unsafe{fflush(std::ptr::null_mut());}
        let started=Instant::now();let parent=unsafe{libc::getpid()};
        let pid=unsafe{libc::fork()};ensure!(pid>=0,"fork failed");
        if pid==0 {
            unsafe{libc::signal(libc::SIGTERM,libc::SIG_DFL);libc::setpgid(0,0);libc::prctl(libc::PR_SET_PDEATHSIG,libc::SIGKILL,0,0,0);if libc::getppid()!=parent{libc::_exit(98);}}
            let result=child(&request,&rom,&mut args);
            let code=match result{Ok(code)=>code,Err(error)=>{eprintln!("{error:#}");4}};
            let _=std::io::stdout().flush();let _=std::io::stderr().flush();unsafe{fflush(std::ptr::null_mut());libc::_exit(code);}
        }
        unsafe{libc::setpgid(pid,pid);}
        let mut timed_out=false;let mut status=0;let mut usage=unsafe{std::mem::zeroed::<libc::rusage>()};
        loop {
            let result=unsafe{libc::wait4(pid,&mut status,libc::WNOHANG,&mut usage)};
            if result==pid{break;}
            if result<0 {ensure!(std::io::Error::last_os_error().raw_os_error()==Some(libc::EINTR),"wait4 failed");}
            if CANCELLED.load(Ordering::Relaxed) || started.elapsed()>=Duration::from_millis(request.timeout_ms){timed_out=true;unsafe{libc::kill(-pid,libc::SIGKILL);libc::kill(pid,libc::SIGKILL);}}
            std::thread::sleep(Duration::from_micros(200));
        }
        unsafe{libc::kill(-pid,libc::SIGKILL);}
        let mut descendants_reaped=0;let deadline=Instant::now()+Duration::from_secs(1);
        loop {let mut ignored=0;let result=unsafe{libc::waitpid(-1,&mut ignored,libc::WNOHANG)};
            if result>0{descendants_reaped+=1;continue;}
            if result<0{ensure!(std::io::Error::last_os_error().raw_os_error()==Some(libc::ECHILD),"descendant wait failed");break;}
            ensure!(Instant::now()<deadline,"descendant reap timeout");std::thread::sleep(Duration::from_micros(200));}
        let cpu=|t:libc::timeval|t.tv_sec as u64*1_000_000_000+t.tv_usec as u64*1000;
        let pristine=rom[0]==original_first && c_rom[0]==original_first && unsafe{g_ram[17]}==0;
        if CANCELLED.load(Ordering::Relaxed){break;}
        emit(&Reply{id:request.id,pid:pid as u32,status,elapsed_ns:started.elapsed().as_nanos() as u64,cpu_ns:cpu(usage.ru_utime)+cpu(usage.ru_stime),reaps:1,timed_out,parent_pristine:pristine,descendants_reaped})?;
        ensure!(pristine,"parent mutated");
    }
    Ok(())
}
