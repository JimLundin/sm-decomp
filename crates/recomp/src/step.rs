//! Emit one resumable instruction per address/width key. Unsupported
//! instructions stay interpreted, making incremental hardware validation safe.
use std::{collections::BTreeMap, fmt::Write};

use anyhow::Result;

use crate::{cfg::FunctionCfg, decode::Insn, emit, opcodes::Mode};

/// Keep this whitelist small until each timing pattern passes the hardware
/// differential tests. Calls, stack operations, interrupts, and indirect modes
/// remain interpreted and can be added without changing the frame interface.
pub fn supported(i: &Insn) -> bool {
    match i.mode {
        Mode::ImmM => i.mnem == "LDA",
        Mode::ImmX => i.mnem == "LDX",
        Mode::Imm8 => matches!(i.mnem, "REP" | "SEP"),
        Mode::Abs => i.mnem == "STA",
        Mode::Imp => matches!(i.mnem, "NOP" | "DEX"),
        _ => false,
    }
}

pub fn emit_table(funcs: &[(FunctionCfg, Option<String>)], read: &dyn Fn(u32) -> Option<u8>) -> Result<String> {
    let mut instructions = BTreeMap::new();
    for (cfg, _) in funcs {
        for i in cfg.blocks.iter().flat_map(|b| &b.insns).filter(|i| supported(i)) {
            instructions.entry((i.addr, emit::mx_key(i.m8, i.x8))).or_insert(i);
        }
    }
    let mut out = String::from(
        "// Generated from your ROM; do not commit or distribute.\nuse runtime::{Machine, StepEntry, ops, flag, ea};\n",
    );
    for ((addr, mx), i) in &instructions {
        writeln!(out, "fn step_{addr:06x}_{mx}(m: &mut Machine) {{")?;
        if matches!(i.mode, Mode::Imp) {
            let next_pc = (i.addr & 0xff0000) | ((i.addr + i.len) & 0xffff);
            writeln!(out, "    m.bus.implied_cycle(0x{next_pc:06x}, m.cpu.flag(flag::I));")?;
        } else if matches!(i.mode, Mode::Imm8) {
            writeln!(out, "    m.bus.tick(6);")?;
        }
        writeln!(out, "    {}", emit::emit_straight(i)?)?;
        writeln!(out, "}}")?;
    }
    writeln!(out, "pub static STEP_TABLE: &[StepEntry] = &[")?;
    for ((addr, mx), i) in &instructions {
        let bytes = (0..i.len).map(|n| read((addr & 0xff0000) | ((addr + n) & 0xffff)).unwrap()).collect::<Vec<_>>();
        writeln!(out, "    (0x{addr:06x}, {mx}, &{bytes:?}, step_{addr:06x}_{mx}),")?;
    }
    writeln!(out, "];")?;
    Ok(out)
}
