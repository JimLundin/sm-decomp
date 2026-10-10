//! Rust code generation.
//!
//! Each function variant becomes a block-dispatch loop:
//! ```ignore
//! pub fn f_808338_m1x1(m: &mut Machine) {
//!     let mut blk = 0usize;
//!     loop { match blk { 0 => { /* insns */ blk = 1; continue; } ... } }
//! }
//! ```
//! It is not pretty, but it is correct by construction for any CFG shape, and
//! rustc/LLVM flatten it well. Readability comes later from overrides + names.

use std::collections::BTreeMap;
use std::fmt::Write;

use anyhow::{Result, bail};

use crate::cfg::{Exit, FunctionCfg};
use crate::decode::Insn;
use crate::opcodes::Mode;

pub fn mx_key(m8: bool, x8: bool) -> u8 {
    ((m8 as u8) << 1) | x8 as u8
}

pub fn variant_name(m8: bool, x8: bool) -> &'static str {
    match (m8, x8) {
        (false, false) => "m0x0",
        (false, true) => "m0x1",
        (true, false) => "m1x0",
        (true, true) => "m1x1",
    }
}

pub fn fn_name(entry: u32, m8: bool, x8: bool) -> String {
    format!("f_{entry:06X}_{}", variant_name(m8, x8))
}

/// Human-readable disassembly for comments and reports.
pub fn disasm(i: &Insn) -> String {
    let o = i.operand;
    let s = match i.mode {
        Mode::Imp => String::new(),
        Mode::Acc => "A".into(),
        Mode::ImmM | Mode::ImmX | Mode::Imm8 | Mode::Imm16 => {
            if i.len == 2 {
                format!("#${o:02X}")
            } else {
                format!("#${o:04X}")
            }
        }
        Mode::Dp => format!("${o:02X}"),
        Mode::DpX => format!("${o:02X},X"),
        Mode::DpY => format!("${o:02X},Y"),
        Mode::DpInd => format!("(${o:02X})"),
        Mode::DpIndX => format!("(${o:02X},X)"),
        Mode::DpIndY => format!("(${o:02X}),Y"),
        Mode::DpIndLong => format!("[${o:02X}]"),
        Mode::DpIndLongY => format!("[${o:02X}],Y"),
        Mode::Abs => format!("${o:04X}"),
        Mode::AbsX => format!("${o:04X},X"),
        Mode::AbsY => format!("${o:04X},Y"),
        Mode::Long => format!("${o:06X}"),
        Mode::LongX => format!("${o:06X},X"),
        Mode::AbsInd => format!("(${o:04X})"),
        Mode::AbsIndX => format!("(${o:04X},X)"),
        Mode::AbsIndLong => format!("[${o:04X}]"),
        Mode::Sr => format!("${o:02X},S"),
        Mode::SrIndY => format!("(${o:02X},S),Y"),
        Mode::Rel8 | Mode::Rel16 => format!("${:04X}", i.rel_target() & 0xffff),
        Mode::Block => format!("${:02X},${:02X}", o >> 8, o & 0xff),
    };
    format!("{} {}", i.mnem, s).trim_end().to_string()
}

/// Effective-address expression for memory modes.
fn ea_expr(i: &Insn) -> Option<String> {
    let o = i.operand;
    Some(match i.mode {
        Mode::Dp => format!("ea::dp(m, 0x{o:02X})"),
        Mode::DpX => format!("ea::dp_x(m, 0x{o:02X})"),
        Mode::DpY => format!("ea::dp_y(m, 0x{o:02X})"),
        Mode::DpInd => format!("ea::dp_ind(m, 0x{o:02X})"),
        Mode::DpIndX => format!("ea::dp_ind_x(m, 0x{o:02X})"),
        Mode::DpIndY => format!("ea::dp_ind_y(m, 0x{o:02X})"),
        Mode::DpIndLong => format!("ea::dp_ind_long(m, 0x{o:02X})"),
        Mode::DpIndLongY => format!("ea::dp_ind_long_y(m, 0x{o:02X})"),
        Mode::Abs => format!("ea::abs(m, 0x{o:04X})"),
        Mode::AbsX => format!("ea::abs_x(m, 0x{o:04X})"),
        Mode::AbsY => format!("ea::abs_y(m, 0x{o:04X})"),
        Mode::Long => format!("ea::long(0x{o:06X})"),
        Mode::LongX => format!("ea::long_x(m, 0x{o:06X})"),
        Mode::Sr => format!("ea::sr(m, 0x{o:02X})"),
        Mode::SrIndY => format!("ea::sr_ind_y(m, 0x{o:02X})"),
        _ => return None,
    })
}

/// Expression for an operand value of the given width.
fn value_expr(i: &Insn, w: bool) -> Result<String> {
    if matches!(i.mode, Mode::ImmM | Mode::ImmX) {
        return Ok(format!("0x{:04X}u16", i.operand));
    }
    match ea_expr(i) {
        Some(ea) => Ok(format!("{{ let a = {ea}; m.read(a, {w}) }}")),
        None => bail!("no value for {} in mode {:?}", i.mnem, i.mode),
    }
}

fn branch_cond(mnem: &str) -> &'static str {
    match mnem {
        "BPL" => "!m.cpu.flag(flag::N)",
        "BMI" => "m.cpu.flag(flag::N)",
        "BVC" => "!m.cpu.flag(flag::V)",
        "BVS" => "m.cpu.flag(flag::V)",
        "BCC" => "!m.cpu.flag(flag::C)",
        "BCS" => "m.cpu.flag(flag::C)",
        "BNE" => "!m.cpu.flag(flag::Z)",
        "BEQ" => "m.cpu.flag(flag::Z)",
        _ => unreachable!(),
    }
}

/// Code for one straight-line instruction (anything that isn't a block exit).
/// Returns Err for control flow, which the block emitter handles itself.
/// Used directly by the fuzz crate, so this is the single source of truth.
pub fn emit_straight(i: &Insn) -> Result<String> {
    let mw = !i.m8; // accumulator / memory op is 16-bit
    let xw = !i.x8; // index op is 16-bit
    let next16 = i.next() & 0xffff;
    let s = match i.mnem {
        // loads / ALU on A
        "LDA" | "ORA" | "AND" | "EOR" | "ADC" | "SBC" => {
            let f = i.mnem.to_lowercase();
            format!("{{ let v = {}; ops::{f}(m, v, {mw}); }}", value_expr(i, mw)?)
        }
        "CMP" => format!("{{ let v = {}; let r = m.cpu.a; ops::cmp(m, r, v, {mw}); }}", value_expr(i, mw)?),
        "BIT" => {
            let imm = i.mode == Mode::ImmM;
            format!("{{ let v = {}; ops::bit(m, v, {mw}, {imm}); }}", value_expr(i, mw)?)
        }
        "LDX" => format!("{{ let v = {}; ops::ldx(m, v, {xw}); }}", value_expr(i, xw)?),
        "LDY" => format!("{{ let v = {}; ops::ldy(m, v, {xw}); }}", value_expr(i, xw)?),
        "CPX" => format!("{{ let v = {}; let r = m.cpu.x; ops::cmp(m, r, v, {xw}); }}", value_expr(i, xw)?),
        "CPY" => format!("{{ let v = {}; let r = m.cpu.y; ops::cmp(m, r, v, {xw}); }}", value_expr(i, xw)?),
        // stores
        "STA" | "STX" | "STY" | "STZ" => {
            let (reg, w) = match i.mnem {
                "STA" => ("m.cpu.a", mw),
                "STX" => ("m.cpu.x", xw),
                "STY" => ("m.cpu.y", xw),
                _ => ("0u16", mw),
            };
            let ea = ea_expr(i).ok_or_else(|| anyhow::anyhow!("store without address"))?;
            format!("{{ let a = {ea}; let v = {reg}; m.write(a, v, {w}); }}")
        }
        // read-modify-write
        "ASL" | "LSR" | "ROL" | "ROR" | "INC" | "DEC" | "TSB" | "TRB" => {
            let f = i.mnem.to_lowercase();
            if i.mode == Mode::Acc {
                format!("{{ let v = ops::get_a(m, {mw}); let r = ops::{f}(m, v, {mw}); ops::set_a(m, r, {mw}); }}")
            } else {
                let ea = ea_expr(i).ok_or_else(|| anyhow::anyhow!("rmw without address"))?;
                format!(
                    "{{ let a = {ea}; let v = m.read(a, {mw}); let r = ops::{f}(m, v, {mw}); m.write(a, r, {mw}); }}"
                )
            }
        }
        "INX" => format!("ops::inx(m, {xw});"),
        "INY" => format!("ops::iny(m, {xw});"),
        "DEX" => format!("ops::dex(m, {xw});"),
        "DEY" => format!("ops::dey(m, {xw});"),
        // transfers
        "TAX" | "TAY" | "TXY" | "TYX" | "TSX" => format!("ops::{}(m, {xw});", i.mnem.to_lowercase()),
        "TXA" | "TYA" => format!("ops::{}(m, {mw});", i.mnem.to_lowercase()),
        "TXS" | "TCD" | "TDC" | "TCS" | "TSC" | "XBA" | "XCE" => format!("ops::{}(m);", i.mnem.to_lowercase()),
        // flags
        "CLC" => "m.cpu.set_flag(flag::C, false);".into(),
        "SEC" => "m.cpu.set_flag(flag::C, true);".into(),
        "CLI" => "m.cpu.set_flag(flag::I, false);".into(),
        "SEI" => "m.cpu.set_flag(flag::I, true);".into(),
        "CLD" => "m.cpu.set_flag(flag::D, false);".into(),
        "SED" => "m.cpu.set_flag(flag::D, true);".into(),
        "CLV" => "m.cpu.set_flag(flag::V, false);".into(),
        "REP" => format!("ops::rep(m, 0x{:02X});", i.operand),
        "SEP" => format!("ops::sep(m, 0x{:02X});", i.operand),
        // stack
        "PHA" | "PLA" => format!("ops::{}(m, {mw});", i.mnem.to_lowercase()),
        "PHX" | "PLX" | "PHY" | "PLY" => format!("ops::{}(m, {xw});", i.mnem.to_lowercase()),
        "PHP" | "PHB" | "PLB" | "PHD" | "PLD" | "PHK" => format!("ops::{}(m);", i.mnem.to_lowercase()),
        "PLP" => format!("ops::plp(m); m.assume_mx({}, 0x{:06X});", crate::emit::mx_key(i.m8, i.x8), i.addr),
        "PEA" => format!("m.push16(0x{:04X});", i.operand),
        "PEI" => {
            format!("{{ let a = ea::dp(m, 0x{:02X}) as u16; let v = m.read16_bank0(a); m.push16(v); }}", i.operand)
        }
        "PER" => format!("m.push16(0x{:04X});", i.rel_target() & 0xffff),
        "MVN" | "MVP" => {
            format!("ops::block_move(m, 0x{:02X}, 0x{:02X}, {});", i.operand & 0xff, i.operand >> 8, i.mnem == "MVN")
        }
        "NOP" => String::new(),
        // calls: push the return address the real CPU would, run the callee,
        // then check the widths we assumed for the code after the call.
        "JSR" | "JSL" => {
            let after = format!("m.assume_mx({}, 0x{:06X});", mx_key(i.m8, i.x8), i.addr);
            match i.mode {
                Mode::Abs => format!(
                    "m.push16(0x{:04X}); m.call(0x{:06X}); {after}",
                    (next16.wrapping_sub(1)) & 0xffff,
                    (i.addr & 0xff_0000) | i.operand
                ),
                Mode::Long => format!(
                    "{{ let pb = m.cpu.pb; m.push8(pb); }} m.push16(0x{:04X}); m.call(0x{:06X}); {after}",
                    (next16.wrapping_sub(1)) & 0xffff,
                    i.operand
                ),
                Mode::AbsIndX => format!(
                    "{{ let t = ea::abs_ind_x_target(m, 0x{:04X}); m.push16(0x{:04X}); m.call(t); }} {after}",
                    i.operand,
                    (next16.wrapping_sub(1)) & 0xffff
                ),
                _ => unreachable!(),
            }
        }
        // block exits and unsupported ops are handled by the caller
        _ => bail!("{} is not a straight-line instruction", i.mnem),
    };
    Ok(s)
}

/// Emit the terminating instruction + exit of a block.
fn emit_exit(out: &mut String, last: &Insn, exit: &Exit) -> Result<()> {
    match exit {
        Exit::Goto(id) => writeln!(out, "                blk = {id}; continue;")?,
        Exit::Cond { taken, fall } => writeln!(
            out,
            "                blk = if {} {{ {taken} }} else {{ {fall} }}; continue;",
            branch_cond(last.mnem)
        )?,
        Exit::Return => {
            let body = match last.mnem {
                "RTS" => "m.pull16();",
                "RTL" => "m.pull16(); m.pull8();",
                _ => "ops::plp(m); m.pull16(); m.pull8();", // RTI (native)
            };
            writeln!(out, "                {body} return;")?
        }
        Exit::Tail(t) => writeln!(out, "                m.tail(0x{t:06X}); return;")?,
        Exit::IndirectTail => {
            let f = match last.mode {
                Mode::AbsInd => "abs_ind_target",
                Mode::AbsIndX => "abs_ind_x_target",
                _ => "abs_ind_long_target",
            };
            writeln!(out, "                let t = ea::{f}(m, 0x{:04X}); m.tail(t); return;", last.operand)?
        }
        Exit::Stop(why) => writeln!(out, "                m.unimplemented(0x{:06X}, {why:?});", last.addr)?,
    }
    Ok(())
}

fn is_exit_insn(i: &Insn, exit: &Exit, is_last: bool) -> bool {
    if !is_last {
        return false;
    }
    match exit {
        // Goto via fallthrough has a normal last instruction; via BRA/JMP it doesn't.
        Exit::Goto(_) => matches!(i.mnem, "BRA" | "BRL" | "JMP" | "JML"),
        // An undecodable byte ends the block *after* a normal instruction.
        Exit::Stop(_) => matches!(i.mnem, "BRK" | "COP" | "STP" | "WAI" | "WDM"),
        _ => true,
    }
}

pub fn emit_function(cfg: &FunctionCfg, name: Option<&str>) -> Result<String> {
    let mut out = String::new();
    let fname = fn_name(cfg.entry, cfg.m8, cfg.x8);
    writeln!(
        out,
        "    /// ${:02X}:{:04X} {} ({})",
        cfg.entry >> 16,
        cfg.entry & 0xffff,
        name.unwrap_or(""),
        variant_name(cfg.m8, cfg.x8)
    )?;
    writeln!(out, "    pub fn {fname}(m: &mut Machine) {{")?;
    writeln!(out, "        let mut blk: usize = 0;")?;
    writeln!(out, "        loop {{")?;
    writeln!(out, "            match blk {{")?;
    for (id, b) in cfg.blocks.iter().enumerate() {
        writeln!(out, "            {id} => {{ // ${:06X} {}", b.start, variant_name(b.m8, b.x8))?;
        let n = b.insns.len();
        if n == 0 {
            writeln!(out, "                m.unimplemented(0x{:06X}, \"empty block\");", b.start)?;
        }
        for (k, i) in b.insns.iter().enumerate() {
            writeln!(out, "                // ${:02X}:{:04X}  {}", i.addr >> 16, i.addr & 0xffff, disasm(i))?;
            if is_exit_insn(i, &b.exit, k + 1 == n) {
                emit_exit(&mut out, i, &b.exit)?;
            } else {
                match emit_straight(i) {
                    Ok(code) if !code.is_empty() => writeln!(out, "                {code}")?,
                    Ok(_) => {}
                    Err(e) => writeln!(out, "                m.unimplemented(0x{:06X}, {:?});", i.addr, e.to_string())?,
                }
                if k + 1 == n {
                    emit_exit(&mut out, i, &b.exit)?;
                }
            }
        }
        writeln!(out, "            }}")?;
    }
    writeln!(out, "            _ => unreachable!(),")?;
    writeln!(out, "            }}")?;
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    Ok(out)
}

/// Assemble every function into one `generated.rs`, grouped by bank.
pub fn emit_crate(funcs: &[(FunctionCfg, Option<String>)]) -> Result<String> {
    let mut by_bank: BTreeMap<u8, Vec<&(FunctionCfg, Option<String>)>> = BTreeMap::new();
    for f in funcs {
        by_bank.entry((f.0.entry >> 16) as u8).or_default().push(f);
    }
    let mut out = String::new();
    writeln!(out, "// Generated by `recomp` from your own ROM. Do not commit or distribute.")?;
    writeln!(out, "use runtime::TableEntry;\n")?;
    for (bank, fs) in &by_bank {
        writeln!(
            out,
            "#[allow(unused_variables, unused_assignments, unused_mut, unreachable_code, unused_braces, clippy::all)]"
        )?;
        writeln!(out, "pub mod bank_{bank:02x} {{")?;
        writeln!(out, "    use runtime::{{ea, flag, ops, Machine}};\n")?;
        for (cfg, name) in fs {
            out.push_str(&emit_function(cfg, name.as_deref())?);
            out.push('\n');
        }
        writeln!(out, "}}\n")?;
    }
    writeln!(out, "pub static TABLE: &[TableEntry] = &[")?;
    for (cfg, _) in funcs {
        writeln!(
            out,
            "    (0x{:06X}, {}, bank_{:02x}::{}),",
            cfg.entry,
            mx_key(cfg.m8, cfg.x8),
            cfg.entry >> 16,
            fn_name(cfg.entry, cfg.m8, cfg.x8)
        )?;
    }
    writeln!(out, "];")?;
    Ok(out)
}
