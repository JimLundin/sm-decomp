//! Emit one resumable instruction per address/width key. Unsupported
//! instructions stay interpreted, making incremental hardware validation safe.
use std::{collections::BTreeMap, fmt::Write};

use anyhow::Result;

use crate::{cfg::FunctionCfg, decode::Insn, emit, opcodes::Mode};

#[derive(serde::Deserialize)]
struct Declaration {
    schema_version: u32,
    patterns: Vec<Pattern>,
    #[serde(default)]
    prospective_patterns: Vec<Pattern>,
}
#[derive(serde::Deserialize)]
struct Pattern {
    opcode: String,
    mnemonic: String,
    mode: String,
    mx: Vec<u8>,
}

/// The same source-owned declaration drives production support and validation.
pub const PATTERN_DECLARATION: &str = include_str!("timed_patterns.json");

fn patterns() -> &'static Declaration {
    static DECLARATION: std::sync::OnceLock<Declaration> = std::sync::OnceLock::new();
    DECLARATION.get_or_init(|| {
        let declaration: Declaration =
            serde_json::from_str(PATTERN_DECLARATION).expect("valid timed pattern declaration");
        assert_eq!(declaration.schema_version, 2, "unsupported timed declaration version");
        declaration
    })
}

/// Keep this whitelist small until each timing pattern passes the hardware
/// differential tests. Calls, stack operations, interrupts, and indirect modes
/// remain interpreted and can be added without changing the frame interface.
pub fn supported(i: &Insn) -> bool {
    patterns().patterns.iter().any(|pattern| {
        u8::from_str_radix(&pattern.opcode, 16).expect("valid declared opcode") == i.op
            && pattern.mnemonic == i.mnem
            && pattern.mode == format!("{:?}", i.mode)
            && pattern.mx.contains(&emit::mx_key(i.m8, i.x8))
    })
}

/// Complete timed body shared by production tables and instruction validation.
pub fn emit_body(i: &Insn) -> Result<String> {
    anyhow::ensure!(supported(i), "unsupported timed pattern {}", i.mnem);
    emit_timed_body(i)
}

/// Explicit proof path for one source-declared prospective pattern. Normal
/// dispatch remains governed solely by the enabled production patterns.
pub fn emit_prospective_body(i: &Insn, opcode: &str) -> Result<String> {
    anyhow::ensure!(
        patterns().prospective_patterns.iter().any(|pattern| {
            pattern.opcode == opcode
                && u8::from_str_radix(opcode, 16).ok() == Some(i.op)
                && pattern.mnemonic == i.mnem
                && pattern.mode == format!("{:?}", i.mode)
                && pattern.mx.contains(&emit::mx_key(i.m8, i.x8))
        }),
        "undeclared prospective timed pattern {opcode}"
    );
    emit_timed_body(i)
}

fn emit_timed_body(i: &Insn) -> Result<String> {
    let mut body = String::new();
    if matches!(i.mode, Mode::Imp) {
        writeln!(body, "m.bus.implied_cycle(0x{:06x}, m.cpu.flag(flag::I));", i.next())?;
    } else if matches!(i.mode, Mode::Imm8) {
        writeln!(body, "m.bus.tick(6);")?;
    }
    writeln!(body, "{}", emit::emit_straight(i)?)?;
    Ok(body)
}

pub fn emit_table(funcs: &[(FunctionCfg, Option<String>)], read: &dyn Fn(u32) -> Option<u8>) -> Result<String> {
    emit_table_for_proof(funcs, read, &std::collections::HashSet::new(), None)
}

pub fn emit_table_excluding(
    funcs: &[(FunctionCfg, Option<String>)],
    read: &dyn Fn(u32) -> Option<u8>,
    exclusions: &std::collections::HashSet<(u32, u8)>,
) -> Result<String> {
    emit_table_for_proof(funcs, read, exclusions, None)
}

/// Isolated proposal generation. An explicit prospective opcode and exclusion
/// inventory never affect normal compiled support.
pub fn emit_table_for_proof(
    funcs: &[(FunctionCfg, Option<String>)],
    read: &dyn Fn(u32) -> Option<u8>,
    exclusions: &std::collections::HashSet<(u32, u8)>,
    prospective: Option<&str>,
) -> Result<String> {
    let mut instructions = BTreeMap::new();
    for (cfg, _) in funcs {
        for i in cfg
            .blocks
            .iter()
            .flat_map(|b| &b.insns)
            .filter(|i| supported(i) || prospective.is_some_and(|opcode| emit_prospective_body(i, opcode).is_ok()))
        {
            let key = (i.addr, emit::mx_key(i.m8, i.x8));
            if !exclusions.contains(&key) {
                instructions.entry(key).or_insert(i);
            }
        }
    }
    let mut out = String::from(
        "// Generated from your ROM; do not commit or distribute.\nuse runtime::{Machine, StepEntry, ops, flag, ea};\n",
    );
    for ((addr, mx), i) in &instructions {
        writeln!(out, "fn step_{addr:06x}_{mx}(m: &mut Machine) {{")?;
        let body = if supported(i) { emit_body(i)? } else { emit_prospective_body(i, prospective.unwrap())? };
        writeln!(out, "    {body}")?;
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
