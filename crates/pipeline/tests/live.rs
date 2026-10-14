//! ABI controls for live replacement transactionality, pointer separation and panics.
use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
#[test]
fn family_abi_rejects_extents_aliases_unknown_routines_and_contains_panics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let directory = std::env::temp_dir().join(format!("sm-live-family-abi-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("panic.rs"),"use crate::Wram;pub fn dispatch(r:&mut Wram,_:&str,_:u16,_:u16)->Result<u32,&'static str>{r.set_byte(1,99);panic!(\"candidate panic\")}").unwrap();
    let abi = include_str!("../src/live_family.rs.in")
        .replace("__MODULE__", &format!("{:?}", directory.join("panic.rs")))
        .replace("__INDEX__", "0")
        .replace("__NAMES__", "[\"PanicControl\"]")
        .replace("__CALL__", "family0::dispatch(&mut Wram::new(bytes).unwrap(),name,a as u16,b as u16)");
    fs::write(
        directory.join("abi.rs"),
        format!("#[path={:?}] mod memory;use memory::*;\n{abi}", root.join("crates/port/src/lib.rs")),
    )
    .unwrap();
    let result = Command::new("rustc")
        .args(["--edition=2024", "--crate-type=staticlib"])
        .arg(directory.join("abi.rs"))
        .arg("-o")
        .arg(directory.join("libabi.a"))
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    fs::write(
        directory.join("control.c"),
        r#"
#include <stdint.h>
#include <stddef.h>
#include <string.h>
#include <assert.h>
static unsigned char ram[0x20000],rom[32];
extern int64_t port_family_0(uint8_t*,size_t,const uint8_t*,size_t,uint32_t,int32_t,int32_t);
int main(void){memset(ram,165,sizeof(ram));
 assert(port_family_0(0,sizeof(ram),rom,sizeof(rom),0,0,0)==-1);
 assert(port_family_0(ram,1,rom,sizeof(rom),0,0,0)==-1);
 assert(port_family_0(ram,sizeof(ram),0,sizeof(rom),0,0,0)==-1);
 assert(port_family_0(ram,sizeof(ram),rom,0,0,0,0)==-1);
 assert(port_family_0(ram,sizeof(ram),rom,0x400001,0,0,0)==-1);
 assert(port_family_0(ram,sizeof(ram),ram+1,32,0,0,0)==-1);
 assert(port_family_0(ram,sizeof(ram),rom,sizeof(rom),999,0,0)==-1);
 for(size_t i=0;i<sizeof(ram);i++)assert(ram[i]==165);
 assert(port_family_0(ram,sizeof(ram),rom,sizeof(rom),0,0,0)==-1);return 0;}
"#,
    )
    .unwrap();
    let result = Command::new("cc")
        .arg(directory.join("control.c"))
        .arg(directory.join("libabi.a"))
        .args(["-ldl", "-lpthread", "-lm", "-o"])
        .arg(directory.join("control"))
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let result = Command::new(directory.join("control")).output().unwrap();
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("candidate panic"));
    fs::remove_dir_all(directory).unwrap();
}
#[test]
fn native_plm_abi_commits_only_success_and_rejects_aliases_extents_and_panics() {
    let directory = std::env::temp_dir().join(format!(
        "sm-live-abi-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir(&directory).unwrap();
    let stub = r#"
const WRAM_SIZE:usize=0x20000;
struct Wram<'a>(&'a mut [u8]);
impl<'a> Wram<'a>{fn new(r:&'a mut [u8])->Result<Self,&'static str>{Ok(Self(r))} fn word(&self,p:usize)->u16{u16::from_le_bytes([self.0[p],self.0[p+1]])} fn byte(&self,p:usize)->u8{self.0[p]}}
mod memory {pub mod plm {
 use crate::Wram;
 pub fn supports_header(_: &[u8],header:u16)->Result<bool,&'static str>{Ok(header!=0)}
 pub fn supports_positioned_header(_: &[u8],header:u16)->Result<bool,&'static str>{Ok(header!=0)}
 pub fn spawn(r:&mut Wram,_:&[u8],header:u16,amount:&mut i32)->Result<u8,&'static str>{r.0[0]=99;*amount=77;match header{1=>Err("setup error"),3=>panic!("setup panic"),_=>Ok(3)}}
 pub fn room_entry(_: &Wram,_:&[u8],a:u16)->Result<(u16,u16),&'static str>{Ok((a,0))}
 pub fn spawn_room(r:&mut Wram,_:&[u8],a:u16)->Result<(),&'static str>{r.0[0]=88;if a==1{Err("setup error")}else{Ok(())}}
 pub fn spawn_hardcoded(r:&mut Wram,o:&[u8],_:u8,_:u8,a:u16)->Result<(),&'static str>{spawn_room(r,o,a)}
}}
"#;
    let source =
        format!("{stub}{}{}", include_str!("../src/live_transaction.rs.in"), include_str!("../src/live_plm.rs.in"));
    fs::write(directory.join("abi.rs"), source).unwrap();
    let result = Command::new("rustc")
        .args(["--edition=2024", "--crate-type=staticlib"])
        .arg(directory.join("abi.rs"))
        .arg("-o")
        .arg(directory.join("libabi.a"))
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    fs::write(
        directory.join("control.c"),
        r#"
#include <stdint.h>
#include <stddef.h>
#include <assert.h>
static uint8_t ram[0x20000],rom[8];
extern int32_t port_native_plm(uint8_t*,size_t,const uint8_t*,size_t,uint16_t,int32_t*);
extern int32_t port_native_positioned_plm(uint8_t*,size_t,const uint8_t*,size_t,uint16_t,uint16_t,uint16_t);
int main(void){int32_t amount=12;
 assert(port_native_plm(ram,sizeof(ram),rom,sizeof(rom),0,&amount)==-2);assert(!ram[0]&&amount==12);
 assert(port_native_plm(ram,sizeof(ram),rom,sizeof(rom),1,&amount)==-1);assert(!ram[0]&&amount==12);
 assert(port_native_plm(ram,sizeof(ram),rom,sizeof(rom),3,&amount)==-1);assert(!ram[0]&&amount==12);
 assert(port_native_plm(ram,sizeof(ram),rom,sizeof(rom),2,(int32_t*)(ram+1))==-1);
 assert(port_native_plm(ram,sizeof(ram),ram+1,8,2,&amount)==-1);
 assert(port_native_plm(ram,sizeof(ram)-1,rom,sizeof(rom),2,&amount)==-1);
 assert(port_native_plm(ram,sizeof(ram),rom,sizeof(rom),2,&amount)==3);assert(ram[0]==99&&amount==77);
 ram[0]=0;
 assert(port_native_positioned_plm(ram,sizeof(ram),ram,sizeof(ram),2,2,0)==-1);assert(!ram[0]);
 assert(port_native_positioned_plm(ram,sizeof(ram),rom,sizeof(rom),2,1,0)==-1);assert(!ram[0]);
 assert(port_native_positioned_plm(ram,sizeof(ram),rom,sizeof(rom),2,0,0)==-2);assert(!ram[0]);
 assert(port_native_positioned_plm(ram,sizeof(ram),rom,sizeof(rom),9,2,0)==-1);assert(!ram[0]);
 assert(port_native_positioned_plm(ram,sizeof(ram),rom,sizeof(rom),2,2,0)==1);assert(ram[0]==88);
 for(size_t i=0;i<40;i++){ram[0x1c37+i*2]=1;ram[0x1c38+i*2]=0;}
 amount=12;assert(port_native_plm(ram,sizeof(ram),rom,sizeof(rom),2,&amount)==0);assert(ram[0]==88&&amount==12);
 assert(port_native_plm(ram,sizeof(ram),rom,sizeof(rom),0,&amount)==-2);assert(ram[0]==88&&amount==12);
 assert(port_native_positioned_plm(ram,sizeof(ram),rom,sizeof(rom),9,0,0)==0);assert(ram[0]==88);
 return 0;}
"#,
    )
    .unwrap();
    let result = Command::new("cc")
        .arg(directory.join("control.c"))
        .arg(directory.join("libabi.a"))
        .args(["-ldl", "-lpthread", "-lm", "-o"])
        .arg(directory.join("control"))
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert!(Command::new(directory.join("control")).status().unwrap().success());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn actual_positioned_plm_abi_keeps_failed_allocation_transactional() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let directory = std::env::temp_dir().join(format!("sm-live-positioned-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let source = format!(
        "#[path={:?}] mod memory;use memory::*;\n{}\n{}\n{}",
        root.join("crates/port/src/lib.rs"),
        include_str!("../src/live_transaction.rs.in"),
        include_str!("../src/live_plm.rs.in"),
        r#"
fn main(){let mut ram=vec![0u8;WRAM_SIZE];let mut rom=vec![0u8;0x300000];rom[0x21000..0x21004].copy_from_slice(&[0x27,0xab,0,0x80]);Wram::new(&mut ram).unwrap().set_word(0x7a5,16);
unsafe{assert_eq!(port_native_positioned_plm(ram.as_mut_ptr(),WRAM_SIZE,ram.as_ptr(),WRAM_SIZE,2,0x9000,0x0203),-1);
assert_eq!(port_native_positioned_plm(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),2,0x9000,0x0203),1);
assert_eq!(Wram::new(&mut ram).unwrap().word(0x1c87+78),70);let before=ram.clone();rom[0x21000..0x21002].copy_from_slice(&[255,255]);
assert_eq!(port_native_positioned_plm(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),2,0x9000,0x0203),-2);assert_eq!(ram,before);
assert_eq!(port_native_positioned_plm(ram.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),1,0xffff,0),-1);assert_eq!(ram,before);}}
"#
    );
    fs::write(directory.join("control.rs"), source).unwrap();
    let result = Command::new("rustc")
        .args(["--edition=2024"])
        .arg(directory.join("control.rs"))
        .arg("-o")
        .arg(directory.join("control"))
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert!(Command::new(directory.join("control")).status().unwrap().success());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn bounded_transactions_isolate_nested_callbacks_threads_and_failed_reuse() {
    let directory = std::env::temp_dir().join(format!("sm-live-nested-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("candidate.rs"),
        r#"
use crate::{Wram, WRAM_SIZE};
unsafe extern "C" { fn Nested(marker:u8)->u8; }
pub fn supports(_: &Wram, a:i32)->bool { match a {3=>false,4=>panic!("selector panic"),_=>true} }
pub fn dispatch(r:&mut Wram,a:i32,b:i32)->Result<u32,&'static str>{
 r.set_byte(0,b as u8);r.set_byte(WRAM_SIZE-1,(b as u8)^255);
 match a {1=>return Err("partial failure"),2=>panic!("dispatch panic"),5=>{
  let returned=unsafe{Nested(b as u8)};assert_eq!(returned, b as u8+1);
  assert_eq!(r.byte(0),b as u8);assert_eq!(r.byte(WRAM_SIZE-1),(b as u8)^255);
 },_=>{}}
 Ok(0)
}
"#,
    )
    .unwrap();
    let bounded = include_str!("../src/live_bounded_family.rs.in")
        .replace("__MODULE__", &format!("{:?}", directory.join("candidate.rs")))
        .replace("__INDEX__", "0")
        .replace("__NAMES__", "[\"Control\"]")
        .replace("__SUPPORT__", "family0::supports(state,a)")
        .replace("__CALL__", "family0::dispatch(&mut Wram::new(bytes)?,a,b)");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    fs::write(directory.join("abi.rs"), format!(
        "#[path={:?}] mod memory;use memory::*;\n{}\n{bounded}\n{}",
        root.join("crates/port/src/memory.rs"), include_str!("../src/live_transaction.rs.in"),
        r#"
use std::{alloc::{GlobalAlloc,Layout,System},sync::atomic::{AtomicU64,Ordering}};
struct CountAlloc;static ALLOCATIONS:AtomicU64=AtomicU64::new(0);
unsafe impl GlobalAlloc for CountAlloc {
 unsafe fn alloc(&self,l:Layout)->*mut u8{if l.size()==WRAM_SIZE{ALLOCATIONS.fetch_add(1,Ordering::Relaxed);}unsafe{System.alloc(l)}}
 unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)}}
}
#[global_allocator]static ALLOCATOR:CountAlloc=CountAlloc;
#[unsafe(no_mangle)]pub extern "C" fn WramAllocations()->u64{ALLOCATIONS.load(Ordering::Relaxed)}
"#
    )).unwrap();
    let compiled = Command::new("rustc")
        .args(["--edition=2024", "--crate-type=staticlib", "-O"])
        .arg(directory.join("abi.rs"))
        .arg("-o")
        .arg(directory.join("libabi.a"))
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    fs::write(
        directory.join("control.c"),
        r#"
#include <stdint.h>
#include <stddef.h>
#include <string.h>
#include <assert.h>
#include <pthread.h>
static const uint8_t rom[8]={0};
static _Thread_local uint8_t ram[0x20000],nested[0x20000];
extern int64_t port_family_0(uint8_t*,size_t,const uint8_t*,size_t,uint32_t,int32_t,int32_t);
extern uint64_t WramAllocations(void);
static int64_t call(uint8_t*r,int a,int b){return port_family_0(r,0x20000,rom,sizeof(rom),0,a,b);}
uint8_t Nested(uint8_t marker){memset(nested,marker+1,sizeof(nested));assert(call(nested,0,marker+1)==0);
 assert(nested[0]==marker+1&&nested[0x1ffff]==((marker+1)^255));return nested[0];}
static void unchanged(uint8_t value){for(size_t i=0;i<sizeof(ram);i++)assert(ram[i]==value);}
static void *worker(void*p){uint8_t marker=(uintptr_t)p;
 for(int j=0;j<80;j++){memset(ram,marker,sizeof(ram));assert(call(ram,1,marker+5)==-1);unchanged(marker);
  assert(call(ram,3,marker+5)==-2);unchanged(marker);assert(call(ram,5,marker)==0);
  for(size_t i=0;i<sizeof(ram)-1;i++)assert(ram[i]==marker);assert(ram[0x1ffff]==(marker^255));}
 return 0;}
int main(void){memset(ram,19,sizeof(ram));
 assert(call(ram,2,25)==-1);unchanged(19);assert(call(ram,4,25)==-1);unchanged(19);
 assert(port_family_0(ram,sizeof(ram),ram+1,8,0,0,0)==-1);unchanged(19);
 assert(port_family_0(ram,sizeof(ram),(uint8_t*)(UINTPTR_MAX-3),8,0,0,0)==-1);unchanged(19);
 assert(port_family_0((uint8_t*)(UINTPTR_MAX-3),sizeof(ram),rom,8,0,0,0)==-1);
 assert(worker((void*)75)==0);uint64_t warm=WramAllocations();assert(warm==2);
 pthread_t threads[4];for(uintptr_t i=0;i<4;i++)assert(!pthread_create(&threads[i],0,worker,(void*)(20+i*10)));
 for(int i=0;i<4;i++)assert(!pthread_join(threads[i],0));assert(WramAllocations()==warm+8);
 assert(worker((void*)75)==0);assert(WramAllocations()==warm+8);return 0;}
"#,
    )
    .unwrap();
    let compiled = Command::new("cc")
        .arg(directory.join("control.c"))
        .arg(directory.join("libabi.a"))
        .args(["-ldl", "-lpthread", "-lm", "-o"])
        .arg(directory.join("control"))
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    let executed = Command::new(directory.join("control")).output().unwrap();
    assert!(executed.status.success(), "{}", String::from_utf8_lossy(&executed.stderr));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn actual_projectile_no_host_paths_preserve_parameter_writes_and_admission() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let directory = std::env::temp_dir().join(format!("sm-live-projectile-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let source = format!(
        "#[path={:?}] mod memory;use memory::*;\n{}\n{}\n{}",
        root.join("crates/port/src/lib.rs"),
        include_str!("../src/live_transaction.rs.in"),
        include_str!("../src/live_projectile.rs.in"),
        r#"
fn main(){
 let mut ram=vec![0;WRAM_SIZE];let mut host=[11u16,22,33,44,55,66,77,88];let initial_host=host;
 let before=ram.clone();unsafe{assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,std::ptr::null(),0,std::ptr::null_mut(),0,2,0,0),0);}assert_eq!(ram,before);
 for i in 0..18{Wram::new(&mut ram).unwrap().set_word(0x1997+i*2,0xffff);}
 for (kind,a,b,name,pa,pb) in [(1,0xffff,37,"SpawnEprojWithRoomGfx",0xffff,37),(3,19,0,"SpawnEprojWithGfx",19,0)]{
  let mut expected=ram.clone();let mut context=enemy_projectiles::ProjectileHost::from_words(host);
  observed_projectile_ownership::dispatch_host(&mut Wram::new(&mut expected).unwrap(),&[],&mut context,name,pa,pb).unwrap();
  unsafe{assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,std::ptr::null(),0,std::ptr::null_mut(),0,kind,a,b),0);}
  assert_eq!(ram,expected);assert_eq!(host,context.words());assert_eq!(Wram::new(&mut ram).unwrap().word(0x1993),if kind==1{37}else{19});
 }
 let before=ram.clone();unsafe{assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,std::ptr::null(),0,std::ptr::null_mut(),0,3,19,1),-2);}assert_eq!(ram,before);
 ram.fill(0);Wram::new(&mut ram).unwrap().set_word(0x198d,0x8000);let before=ram.clone();
 unsafe{assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,std::ptr::null(),0,std::ptr::null_mut(),0,2,0,0),-1);
 assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,std::ptr::null(),0,ram.as_mut_ptr().cast(),8,2,0,0),-1);}assert_eq!(ram,before);
 let mut expected=ram.clone();let mut context=enemy_projectiles::ProjectileHost::from_words(host);
 observed_projectile_ownership::dispatch_host(&mut Wram::new(&mut expected).unwrap(),&[],&mut context,"EprojRunAll",0,0).unwrap();
 unsafe{assert_eq!(port_native_projectile(ram.as_mut_ptr(),WRAM_SIZE,std::ptr::null(),0,host.as_mut_ptr(),8,2,0,0),0);}assert_eq!(ram,expected);assert_eq!(host,context.words());assert_eq!(host,initial_host);
}
"#
    );
    fs::write(directory.join("control.rs"), source).unwrap();
    let compiled = Command::new("rustc")
        .args(["--edition=2024", "-Awarnings"])
        .arg(directory.join("control.rs"))
        .arg("-o")
        .arg(directory.join("control"))
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    let executed = Command::new(directory.join("control")).output().unwrap();
    assert!(executed.status.success(), "{}", String::from_utf8_lossy(&executed.stderr));
    fs::remove_dir_all(directory).unwrap();
}
