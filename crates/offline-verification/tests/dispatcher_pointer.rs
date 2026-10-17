use std::{fs, process::Command};
use tool_execution::testing::Directory;

#[test]
fn dispatcher_pointer_banks_normalize_raw_and_host_pointers_and_reject_other_banks() {
    let directory = Directory::new().unwrap();
    let function = include_str!("../src/dispatcher_pointer.c");
    fs::write(
        directory.0.join("pointer.c"),
        format!(
            "#include <stdint.h>\n#include <stdlib.h>\n#include <assert.h>\nstatic uint8_t rom[0x400000];static uint8_t *g_rom=rom;\n{function}\nint main(int argc,char **argv){{if(argc>1){{PortPointerResult((void*)(rom+0x190000),0x86);return 1;}}for(int bank=0x84;bank<=0xb2;bank++){{if(bank!=0x84&&bank!=0x86&&bank!=0xb2)continue;assert(PortPointerResult((void*)(rom+((bank&0x7f)*0x8000)+0x1234),bank)==((bank<<16)|0x9234));assert(PortPointerResult((void*)0x9234,bank)==((bank<<16)|0x9234));assert(PortPointerResult(0,bank)==0);}}return 0;}}"
        ),
    )
    .unwrap();
    assert!(
        Command::new("cc")
            .arg(directory.0.join("pointer.c"))
            .arg("-o")
            .arg(directory.0.join("pointer"))
            .status()
            .unwrap()
            .success()
    );
    assert!(Command::new(directory.0.join("pointer")).status().unwrap().success());
    assert_eq!(Command::new(directory.0.join("pointer")).arg("wrong-bank").status().unwrap().code(), Some(3));
}
