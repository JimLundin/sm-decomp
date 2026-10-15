use super::*;
use std::process::Command;

#[test]
fn generated_abi_accepts_canonical_and_renamed_plm_modules() -> Result<()> {
    for name in ["plm", "allocation"] {
        for confined in [false, true] {
            let directory = tool_execution::testing::Directory::new()?;
            let checkout = &directory.0;
            let sources = checkout.join("crates/port/src");
            let run = checkout.join("run");
            fs::create_dir_all(&sources)?;
            fs::create_dir(&run)?;
            let memory = sources.join("memory.rs");
            fs::write(&memory, include_str!("../../port/src/memory.rs"))?;
            let module = sources.join(format!("{name}.rs"));
            fs::write(
                &module,
                r#"
use crate::memory::Wram;
pub fn spawn(ram: &mut Wram, rom: &[u8], header: u16, amount: &mut i32) -> Result<u8, &'static str> {
    ram.set_word(0, header);
    *amount += i32::from(rom[0]);
    Ok(3)
}
pub fn spawn_room(ram: &mut Wram, _rom: &[u8], header: u16) -> Result<(), &'static str> {
    ram.set_word(2, header);
    Ok(())
}
pub fn spawn_hardcoded(ram: &mut Wram, _rom: &[u8], x: u8, y: u8, header: u16) -> Result<(), &'static str> {
    ram.set_word(4, header);
    ram.set_word(6, u16::from_le_bytes([x, y]));
    Ok(())
}
"#,
            )?;
            let generated = if confined {
                inputs::canonical_wrapper(checkout, &run, &[memory, module.clone()])?
            } else {
                inputs::wrapper(crate::authoring::WrapperRequest {
                    checkout: checkout.clone(),
                    run: run.clone(),
                    modules: vec![module.clone()],
                    imports: vec![],
                })?
            };
            let mut wrapper = rust_wrapper(&generated.path, &module, confined);
            if confined {
                // Map the namespace paths locally while preserving canonical
                // crate-root placement and the exact generated ABI wrapper.
                let mapped = run.join("mapped-memory.rs");
                fs::write(
                    &mapped,
                    fs::read_to_string(&generated.path)?.replace("/inputs/rust/", &format!("{}/", checkout.display())),
                )?;
                wrapper = wrapper.replace("\"/inputs/generated/memory.rs\"", &format!("{mapped:?}"));
            }
            wrapper.push_str(
                r#"
fn main() {
    let mut ram = vec![0u8; WRAM_SIZE];
    let rom = [7u8];
    let mut amount = 5;
    unsafe {
        assert_eq!(native_spawn(ram.as_mut_ptr(), ram.len(), rom.as_ptr(), rom.len(), 0x1234, &mut amount), 3);
        assert_eq!(amount, 12);
        assert_eq!(&ram[..2], &[0x34, 0x12]);
        assert_eq!(native_positioned(ram.as_mut_ptr(), ram.len(), rom.as_ptr(), rom.len(), 1, 0x5678, 0), 0);
        assert_eq!(&ram[2..4], &[0x78, 0x56]);
        assert_eq!(native_positioned(ram.as_mut_ptr(), ram.len(), rom.as_ptr(), rom.len(), 2, 0x9abc, 0x4321), 0);
        assert_eq!(&ram[4..8], &[0xbc, 0x9a, 0x21, 0x43]);
        let before = ram.clone();
        assert_eq!(native_spawn(std::ptr::null_mut(), ram.len(), rom.as_ptr(), rom.len(), 0, &mut amount), -1);
        assert_eq!(native_spawn(ram.as_mut_ptr(), ram.len() - 1, rom.as_ptr(), rom.len(), 0, &mut amount), -1);
        assert_eq!(native_spawn(ram.as_mut_ptr(), ram.len(), ram.as_ptr(), 1, 0, &mut amount), -1);
        assert_eq!(native_spawn(ram.as_mut_ptr(), ram.len(), rom.as_ptr(), rom.len(), 0, ram.as_mut_ptr().cast()), -1);
        assert_eq!(native_positioned(ram.as_mut_ptr(), ram.len(), std::ptr::null(), 0, 1, 0, 0), -1);
        assert_eq!(native_positioned(ram.as_mut_ptr(), ram.len(), rom.as_ptr(), rom.len(), 3, 0, 0), -1);
        assert_eq!(ram, before);
        assert_eq!(amount, 12);
    }
}
"#,
            );
            let source = run.join("wrapper.rs");
            let binary = run.join("wrapper");
            fs::write(&source, wrapper)?;
            let compiled =
                Command::new("rustc").args(["--edition=2024"]).arg(&source).arg("-o").arg(&binary).output()?;
            ensure!(
                compiled.status.success(),
                "{name} wrapper (confined={confined}) failed to compile: {}",
                String::from_utf8_lossy(&compiled.stderr)
            );
            let result = Command::new(&binary).output()?;
            ensure!(
                result.status.success(),
                "{name} wrapper (confined={confined}) failed ABI checks: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
    Ok(())
}
