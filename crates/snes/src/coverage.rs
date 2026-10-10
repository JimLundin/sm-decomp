//! Passive accounting of instructions that actually executed, separate from calls.
use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct InstructionCount {
    pub address: u32,
    pub mx: u8,
    pub emulation: bool,
    pub opcode: u8,
    pub pattern: String,
    pub supported: bool,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct Coverage {
    pub schema_version: u32,
    pub compiled: Vec<InstructionCount>,
    pub interpreted: Vec<InstructionCount>,
    pub generated_instructions: u64,
    pub interpreted_instructions: u64,
    pub generated_dispatches: u64,
    pub interpreted_dispatches: u64,
    pub noninstruction_dispatches: u64,
    pub fallback_fraction: f64,
    pub observation: &'static str,
}

#[derive(Default)]
pub(crate) struct Accounting {
    generated: BTreeMap<(u32, u8, bool, u8), u64>,
    interpreted: BTreeMap<(u32, u8, bool, u8), u64>,
}

fn entry(key: (u32, u8, bool, u8), count: u64) -> InstructionCount {
    let (address, mx, emulation, opcode) = key;
    let (mnemonic, mode) = recomp::opcodes::OPCODES[opcode as usize];
    let instruction = recomp::decode::Insn { addr: address, op: opcode, mnem: mnemonic, mode,
        len: mode.len(mx & 2 != 0, mx & 1 != 0), operand: 0, m8: mx & 2 != 0, x8: mx & 1 != 0 };
    InstructionCount { address, mx, emulation, opcode, pattern: format!("{mnemonic}/{mode:?}"),
        supported: !emulation && recomp::step::supported(&instruction), count }
}

impl Accounting {
    pub fn record(&mut self, generated: bool, address: u32, mx: u8, emulation: bool, opcode: u8) {
        let counts = if generated { &mut self.generated } else { &mut self.interpreted };
        *counts.entry((address, mx, emulation, opcode)).or_default() += 1;
    }

    pub fn report(&self, table: impl Iterator<Item = (u32, u8, u8)>, generated_dispatches: u64,
                  interpreted_dispatches: u64) -> Coverage {
        let mut compiled: Vec<_> = table.map(|(address, mx, opcode)| {
            entry((address, mx, false, opcode), *self.generated.get(&(address, mx, false, opcode)).unwrap_or(&0))
        }).collect();
        compiled.sort_by_key(|row| (row.address, row.mx));
        let interpreted = self.interpreted.iter().map(|(&key, &count)| entry(key, count)).collect();
        let generated_instructions = self.generated.values().sum();
        let interpreted_instructions = self.interpreted.values().sum();
        let total = generated_instructions + interpreted_instructions;
        Coverage { schema_version: 1, compiled, interpreted, generated_instructions, interpreted_instructions,
            generated_dispatches, interpreted_dispatches,
            noninstruction_dispatches: generated_dispatches + interpreted_dispatches - total,
            fallback_fraction: if total == 0 { 0.0 } else { interpreted_instructions as f64 / total as f64 },
            observation: "actual executed instructions; hardware interrupt/wait/stop dispatch excluded" }
    }
}
