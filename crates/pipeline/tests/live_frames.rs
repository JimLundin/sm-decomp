//! Compile the actual frame ABI and prove selection, publication and trace parity.
use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn bounded_and_ridley_abis_preserve_refusal_and_match_whole_attack_frame() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let directory = std::env::temp_dir().join(format!(
        "sm-live-frames-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir(&directory).unwrap();
    let bounded = include_str!("../src/live_bounded_family.rs.in")
        .replace("__MODULE__", &format!("{:?}", root.join("crates/port/src/camera_scroll.rs")))
        .replace("__INDEX__", "0")
        .replace("__NAMES__", "[\"MainScrollingRoutine\"]")
        .replace("__SUPPORT__", "family0::supports(state,name)")
        .replace("__CALL__", "family0::dispatch_rom(&mut Wram::new(bytes).unwrap(),rom,name,a,b)");
    let controls = format!(
        r#"
static mut OPCODES:usize=0;
static mut ALLOCATIONS:usize=0;
#[unsafe(no_mangle)]unsafe extern "C" fn PortRidleyOpcode(_:u32,_:u32,_:u32,_:u32,_:u32){{unsafe{{OPCODES+=1;}}}}
#[unsafe(no_mangle)]unsafe extern "C" fn PortRidleyAllocationBefore(_: *const u8,_: *const u16){{}}
#[unsafe(no_mangle)]unsafe extern "C" fn PortRidleyAllocationAfter(_: *const u8,_: *const u16){{unsafe{{ALLOCATIONS+=1;}}}}
fn initial()->Vec<u8>{{let mut bytes=vec![0;WRAM_SIZE];let mut r=Wram::new(&mut bytes).unwrap();r.set_word(0xf78,0xe13f);r.set_byte(0xfa6,0xa6);r.set_word(0xf92,0xe73a);r.set_word(0xf94,1);r.set_word(0x9c2,99);r.set_word(0xf7a,180);r.set_word(0xf7e,111);r.set_word(0xaf6,80);r.set_word(0xafa,180);bytes}}
fn main(){{let rom=std::fs::read({rom:?}).unwrap();let mut bytes=initial();let mut host=[11u16,22,33,44,55,66,77,88];
 let before=bytes.clone();let original_host=host;unsafe{{assert_eq!(port_ridley_frame(bytes.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),host.as_mut_ptr(),8,0),0);}}assert_eq!(bytes,before);assert_eq!(host,original_host);
 let mut frozen=initial();Wram::new(&mut frozen).unwrap().set_word(0xf8a,4);let saved=frozen.clone();unsafe{{assert_eq!(port_ridley_frame(frozen.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),host.as_mut_ptr(),8,1),0);}}assert_eq!(frozen,saved);assert_eq!(host,original_host);
 unsafe{{assert_eq!(port_ridley_frame(bytes.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),bytes.as_mut_ptr().cast(),8,0),-1);assert_eq!(port_ridley_frame(bytes.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),(usize::MAX-1)as *mut u16,8,0),-1);}}assert_eq!(bytes,before);assert_eq!(host,original_host);
 for fault in 0..4{{let mut invalid=initial();let mut r=Wram::new(&mut invalid).unwrap();match fault{{0=>r.set_byte(0xfa6,0xa5),1=>r.set_word(0xf92,0xe74a),2=>r.set_word(0xf78,0xdead),_=>r.set_word(0xe54,1)}}let saved=invalid.clone();unsafe{{assert_eq!(port_ridley_frame(invalid.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),host.as_mut_ptr(),8,1),if fault==3{{-1}}else{{-2}});}}assert_eq!(invalid,saved);assert_eq!(host,original_host);}}
 let mut expected=initial();let mut context=memory::enemy_projectiles::ProjectileHost::from_words(host);let mut opcode_count=0;let mut allocation_count=0;
 for _ in 0..240{{let report=memory::ridley_enemy_instructions::process_instructions(&mut Wram::new(&mut expected).unwrap(),&rom,&mut context,4096).unwrap();opcode_count+=report.opcodes.len();allocation_count+=report.effects.iter().filter(|e|matches!(e,memory::ridley_enemy_instructions::InstructionEffect::RoomProjectile{{..}})).count();unsafe{{assert_eq!(port_ridley_frame(bytes.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),host.as_mut_ptr(),8,1),0);}}assert_eq!(bytes,expected);assert_eq!(host,context.words());}}
 unsafe{{assert_eq!(*std::ptr::addr_of!(OPCODES),opcode_count);assert_eq!(*std::ptr::addr_of!(ALLOCATIONS),allocation_count);}}assert_eq!(allocation_count,8);
 let mut camera=vec![0;WRAM_SIZE];let saved=camera.clone();unsafe{{assert_eq!(port_family_0(camera.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),0,0,0),-2);}}assert_eq!(camera,saved);let mut r=Wram::new(&mut camera).unwrap();r.set_word(0x7a9,1);r.set_word(0x7ab,1);let mut expected=camera.clone();memory::camera_scroll::dispatch_rom(&mut Wram::new(&mut expected).unwrap(),&rom,"MainScrollingRoutine",0,0).unwrap();unsafe{{assert_eq!(port_family_0(camera.as_mut_ptr(),WRAM_SIZE,rom.as_ptr(),rom.len(),0,0,0),0);}}assert_eq!(camera,expected);
}}
"#,
        rom = root.join("rom/sm.sfc")
    );
    fs::write(
        directory.join("control.rs"),
        format!(
            "#[path={:?}]mod memory;use memory::*;\n{}\n{bounded}\nmod ridley_abi{{use super::*;{}}}use ridley_abi::port_ridley_frame;\n{controls}",
            root.join("crates/port/src/lib.rs"),
            include_str!("../src/live_transaction.rs.in"),
            include_str!("../src/live_ridley.rs.in")
        ),
    )
    .unwrap();
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

#[test]
fn bounded_c_wrapper_refuses_before_c_and_never_retries_accepted_error() {
    let directory = std::env::temp_dir().join(format!("sm-live-frame-wrapper-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let wrapper = include_str!("../src/live_bounded_wrapper.c.in")
        .replace("__SYMBOL__", "Scheduled")
        .replace("__INDEX__", "0")
        .replace("__ACTIVE__", "1")
        .replace("__ABI__", "abi")
        .replace("__ROUTINE__", "0");
    let source = format!(
        r#"
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
static uint8_t g_ram[0x20000],g_rom[8];static struct {{struct {{size_t romSize;}}*cart;}} snes;static typeof(snes)*g_snes=&snes;
static unsigned port_call_depth,port_frame_captures[1];static unsigned long port_outer_hits[1],port_c_hits[1],port_rust_hits[1],port_frame_refused[1],port_frame_active_c[1],port_frame_active_rust[1];static FILE*port_calls;static int port_mixed=1,answer;
static void PortCapture(unsigned i,unsigned long n,const char*p){{}}static void PortFrameCapture(const char*s,unsigned n,const char*p){{}}
static int64_t abi(uint8_t*r,size_t n,const uint8_t*o,size_t m,uint32_t id,int32_t a,int32_t b){{return answer;}}
void C_Scheduled(void){{if(answer==-1){{fprintf(stderr,"C retry occurred\n");exit(17);}}g_ram[0]++;}}
{wrapper}
int main(int argc,char**argv){{typeof(*snes.cart)cart={{8}};snes.cart=&cart;port_calls=tmpfile();assert(port_calls);answer=argc==2?-1:-2;Scheduled();assert(g_ram[0]==1&&port_c_hits[0]==1&&port_frame_refused[0]==1);answer=0;Scheduled();assert(g_ram[0]==1&&port_rust_hits[0]==1);return 0;}}
"#
    );
    fs::write(directory.join("control.c"), source).unwrap();
    let compiled =
        Command::new("cc").arg(directory.join("control.c")).arg("-o").arg(directory.join("control")).output().unwrap();
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    assert!(Command::new(directory.join("control")).status().unwrap().success());
    let failed = Command::new(directory.join("control")).arg("accepted-error").output().unwrap();
    assert_eq!(failed.status.code(), Some(4));
    assert!(!String::from_utf8_lossy(&failed.stderr).contains("C retry"));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn ridley_native_same_bank_bridge_records_real_pointer_kinds_and_host_publication() {
    let directory = std::env::temp_dir().join(format!("sm-live-ridley-bank-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let source = format!(
        r#"
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
typedef uint8_t uint8;typedef uint16_t uint16;typedef uint32_t uint32;typedef int32_t int32;
#define GET_WORD(p) ((uint16)((p)[0]|((uint16)(p)[1]<<8)))
static uint8 g_ram[0x20000],rom[0x400000];static uint8*g_rom=rom;static struct{{struct{{size_t romSize;}}*cart;}}snes;static typeof(snes)*g_snes=&snes;static unsigned snes_frame_counter,port_mixed;static FILE*port_effects;
static int support,executing;static unsigned c_called;static uint16 native_host[8]={{1,2,3,4,5,6,7,8}};static char effects[5000];
static void PortEffect(FILE*f,const char*format,...){{}}
void PortProjectileHostSave(uint16*p){{memcpy(p,native_host,16);}}void PortProjectileHostLoad(const uint16*p){{memcpy(native_host,p,16);}}
static unsigned long port_projectile_capture[3],port_projectile_combat_capture[3],port_projectile_alloc_capture[9],port_projectile_alloc_total[9],port_projectile_alloc_rust[9],port_projectile_total[3],port_projectile_rust[3],port_projectile_active_rust[3],port_projectile_9642_rust[3];
static int PortProjectileFree(void){{return 1;}}static void PortProjectileAllocCapture(unsigned i,unsigned n,const char*p){{}}static void PortProjectileCapture(unsigned i,unsigned n,int c,const char*p){{}}
int32 port_ridley_frame(uint8*r,size_t n,const uint8*o,size_t m,uint16*h,size_t k,uint32 execute){{if(!execute)return support;if(executing<0)return -1;r[0]=91;h[7]=99;return 0;}}
{host}
static uint8 *RomBankBase(uint8 bank){{return rom+0x128000;}}
static void PortC_ProcessEnemyInstructions(void){{c_called++;}}
static const uint16 *PortC_CallEnemyInstr(uint32 a,uint16 k,const uint16*p){{return (const uint16*)0xe7b4;}}
{bank}
int main(void){{typeof(*snes.cart)cart={{sizeof(rom)}};snes.cart=&cart;g_ram[0xe54]=0;g_ram[0xf92]=0x3a;g_ram[0xf93]=0xe7;g_ram[0xf94]=1;support=0;
 port_mixed=0;ProcessEnemyInstructions();assert(c_called==1&&port_ridley_frame_c==1&&port_ridley_active_c==1);
 port_ridley_supported=1;CallEnemyInstr(0xa6e517,0,(const uint16*)(RomBankBase(0xa6)+0xe73c));assert(port_ridley_opcode_c[0]==1);
 port_mixed=1;support=-2;ProcessEnemyInstructions();assert(c_called==2&&port_ridley_frame_refused==1);
 support=0;ProcessEnemyInstructions();assert(c_called==2&&g_ram[0]==91&&native_host[7]==99&&port_ridley_frame_rust==1);
 return 0;}}
"#,
        host = include_str!("../src/live_ridley_host.c.in"),
        bank = include_str!("../src/live_ridley_bank.c.in")
    );
    fs::write(directory.join("control.c"), source).unwrap();
    let compiled =
        Command::new("cc").arg(directory.join("control.c")).arg("-o").arg(directory.join("control")).output().unwrap();
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    fs::create_dir(directory.join("fixtures")).unwrap();
    let result = Command::new(directory.join("control")).current_dir(&directory).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    fs::remove_dir_all(directory).unwrap();
}
