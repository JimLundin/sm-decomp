use std::{collections::HashSet, path::PathBuf};

// The integration fixture goes through the real decoder/emitter without a
// commercial ROM or the game's generated files.
fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    let program = [
        0x18, 0xfb, 0xc2, 0x30, // native mode, 16-bit
        0xa9, 0x34, 0x12, 0x8d, 0x10, 0x00, 0xe2, 0x20, 0xa9, 0x80, 0x8d, 0x00, 0x42, // enable NMI
        0xa2, 0x03, 0x00, 0xca, 0xd0, 0xfd, 0xa9, 0xab, 0x8d, 0x12, 0x00, 0x80, 0xf3,
    ];
    let read = |addr: u32| program.get(addr.wrapping_sub(0x8000) as usize).copied();
    let cfg = recomp::cfg::build(&read, 0x8000, true, true, &HashSet::new())?;
    let src = recomp::step::emit_table(&[(cfg, None)], &read)?;
    std::fs::write(PathBuf::from(std::env::var("OUT_DIR")?).join("test_steps.rs"), src)?;
    let nmi_program = [0x18, 0xfb, 0xe2, 0x30, 0xa9, 0x80, 0x8d, 0, 0x42, 0xea, 0xea, 0xea, 0x80, 0xfc];
    let read = |addr: u32| nmi_program.get(addr.wrapping_sub(0x8000) as usize).copied();
    let cfg = recomp::cfg::build(&read, 0x8000, true, true, &HashSet::new())?;
    let src = recomp::step::emit_table(&[(cfg, None)], &read)?;
    std::fs::write(PathBuf::from(std::env::var("OUT_DIR")?).join("nmi_steps.rs"), src)?;
    let hdma_program = [0xea, 0xea, 0x80, 0xfc];
    let read = |addr: u32| hdma_program.get(addr.wrapping_sub(0x8027) as usize).copied();
    let cfg = recomp::cfg::build(&read, 0x8027, true, true, &HashSet::new())?;
    let src = recomp::step::emit_table(&[(cfg, None)], &read)?;
    std::fs::write(PathBuf::from(std::env::var("OUT_DIR")?).join("hdma_steps.rs"), src)?;
    let irq_program = [0xea, 0x80, 0xfe];
    let read = |addr: u32| irq_program.get(addr.wrapping_sub(0x80807f) as usize).copied();
    let cfg = recomp::cfg::build(&read, 0x80807f, true, true, &HashSet::new())?;
    let src = recomp::step::emit_table(&[(cfg, None)], &read)?;
    std::fs::write(PathBuf::from(std::env::var("OUT_DIR")?).join("irq_steps.rs"), src)?;
    for (name, address, program) in [
        ("irq_rep_steps.rs", 0x80807e, [0xc2, 4, 0xea, 0x80, 0xfe]),
        ("irq_sep_steps.rs", 0x80807f, [0xe2, 4, 0xea, 0x80, 0xfe]),
    ] {
        let read = |addr: u32| program.get(addr.wrapping_sub(address) as usize).copied();
        let cfg = recomp::cfg::build(&read, address, true, true, &HashSet::new())?;
        let src = recomp::step::emit_table(&[(cfg, None)], &read)?;
        std::fs::write(PathBuf::from(std::env::var("OUT_DIR")?).join(name), src)?;
    }
    for phase in 0..8 {
        let mut program = vec![0x18, 0xfb, 0xe2, 0x30, 0xa9, 0x80, 0x8d, 0, 0x42];
        program.extend(std::iter::repeat_n(0xea, phase));
        program.extend([0xea, 0xea, 0x80, 0xfc]);
        let read = |addr: u32| program.get(addr.wrapping_sub(0x8000) as usize).copied();
        let cfg = recomp::cfg::build(&read, 0x8000, true, true, &HashSet::new())?;
        let src = recomp::step::emit_table(&[(cfg, None)], &read)?;
        std::fs::write(PathBuf::from(std::env::var("OUT_DIR")?).join(format!("nmi_phase_{phase}.rs")), src)?;
    }
    let program = [0x18, 0xfb, 0xa9, 0x12, 0x8d, 0x10, 0, 0xa9, 0, 0x8d, 0x10, 0, 0x80, 0xfe];
    let read = |addr: u32| {
        if addr == 0x8100 { Some(0xea) } else { program.get(addr.wrapping_sub(0x8000) as usize).copied() }
    };
    let mut functions = Vec::new();
    for address in [0x8000, 0x8100] {
        functions.push((recomp::cfg::build(&read, address, true, true, &HashSet::new())?, None));
    }
    std::fs::write(
        PathBuf::from(std::env::var("OUT_DIR")?).join("feedback_steps.rs"),
        recomp::step::emit_table(&functions, &read)?,
    )?;
    let program = [0xa9, 1, 0x8d, 0x0b, 0x42, 0xea, 0xea];
    let read = |addr: u32| program.get(addr.wrapping_sub(0x8000) as usize).copied();
    let cfg = recomp::cfg::build(&read, 0x8000, true, true, &HashSet::new())?;
    let src = recomp::step::emit_table(&[(cfg, None)], &read)?;
    std::fs::write(PathBuf::from(std::env::var("OUT_DIR")?).join("dma_steps.rs"), src)?;
    Ok(())
}
