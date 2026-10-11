//! Execute the actual native reset coroutine without powering down hardware.
use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn soft_reset_enters_actual_vector_at_manual_position_three() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    if !root.join("deps/snesrev-sm/src/sm_80.c").is_file() {
        eprintln!("actual reset control requires pinned C source");
        return;
    }
    let source = fs::read_to_string(root.join("deps/snesrev-sm/src/sm_80.c")).unwrap();
    let reset = source.split_once("CoroutineRet Vector_RESET_Async(void) {").unwrap().1.split_once("\n}\n").unwrap().0;
    let reset = reset.split_once("  APU_UploadBank(0xCF8000);").unwrap().0;
    let control = format!(
        r#"#include <stdint.h>
#include <string.h>
#include "types.h"
static uint8_t g_ram[0x20000],coroutine_state_0,reg_MEMSEL;
static uint16_t game_state,bug_fix_counter;
static unsigned io_writes,logo_calls,uploads;
struct Cpu{{uint8_t k;uint16_t pc;}} cpu;
struct Snes{{struct Cpu *cpu;}} snes={{&cpu}},*g_snes=&snes;
#define MEMSEL 0x420d
static void WriteReg(int a,int b){{(void)a;(void)b;io_writes++;}}
static uint8_t InitializeIoDisplayLogo_Async(void){{logo_calls++;return 0;}}
#include "headless_frame.h"
CoroutineRet Vector_RESET_Async(void){{{reset}
 uploads++;return 0;}}}}
int main(void){{
 memset(g_ram,0xa5,sizeof(g_ram));game_state=0xffff;coroutine_state_0=0;
 HeadlessPrepareFrame();Vector_RESET_Async();
 if(uploads!=1||logo_calls||io_writes||g_ram[0]!=0xa5||cpu.k!=0x80||cpu.pc!=0x8462)return 1;
 game_state=8;coroutine_state_0=12;HeadlessPrepareFrame();
 if(coroutine_state_0!=12)return 2;
 return 0;}}
"#
    );
    struct Temporary(std::path::PathBuf);
    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let run = Temporary(std::env::temp_dir().join(format!(
        "sm-soft-reset-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    )));
    fs::create_dir(&run.0).unwrap();
    fs::write(run.0.join("control.c"), control).unwrap();
    let result = Command::new("cc")
        .arg("-iquote")
        .arg(root.join("deps/snesrev-sm/src"))
        .arg("-iquote")
        .arg(root.join("tools/decomp"))
        .arg(run.0.join("control.c"))
        .arg("-o")
        .arg(run.0.join("control"))
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let result = Command::new(run.0.join("control")).output().unwrap();
    assert!(
        result.status.success(),
        "soft-reset control exited {:?}: {}",
        result.status.code(),
        String::from_utf8_lossy(&result.stderr)
    );
}
