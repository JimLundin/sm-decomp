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
impl<'a> Wram<'a>{fn new(r:&'a mut [u8])->Result<Self,&'static str>{Ok(Self(r))} fn word(&self,p:usize)->u16{u16::from_le_bytes([self.0[p],self.0[p+1]])}}
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
    let source = format!("{stub}{}", include_str!("../src/live_plm.rs.in"));
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
        "#[path={:?}] mod memory;use memory::*;\n{}\n{}",
        root.join("crates/port/src/lib.rs"),
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
