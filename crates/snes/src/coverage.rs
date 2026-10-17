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
    let instruction = recomp::decode::Insn {
        addr: address,
        op: opcode,
        mnem: mnemonic,
        mode,
        len: mode.len(mx & 2 != 0, mx & 1 != 0),
        operand: 0,
        m8: mx & 2 != 0,
        x8: mx & 1 != 0,
    };
    InstructionCount {
        address,
        mx,
        emulation,
        opcode,
        pattern: format!("{mnemonic}/{mode:?}"),
        supported: !emulation && recomp::step::supported(&instruction),
        count,
    }
}

impl Accounting {
    pub(crate) fn checkpoint(&self) -> Vec<(bool, u32, u8, bool, u8, u64)> {
        self.generated
            .iter()
            .map(|(&(a, m, e, o), &n)| (true, a, m, e, o, n))
            .chain(self.interpreted.iter().map(|(&(a, m, e, o), &n)| (false, a, m, e, o, n)))
            .collect()
    }
    pub(crate) fn restore(&mut self, rows: Vec<(bool, u32, u8, bool, u8, u64)>) {
        self.generated.clear();
        self.interpreted.clear();
        for (g, a, m, e, o, n) in rows {
            let table = if g { &mut self.generated } else { &mut self.interpreted };
            table.insert((a, m, e, o), n);
        }
    }
    pub fn record(&mut self, generated: bool, address: u32, mx: u8, emulation: bool, opcode: u8) {
        let counts = if generated { &mut self.generated } else { &mut self.interpreted };
        *counts.entry((address, mx, emulation, opcode)).or_default() += 1;
    }

    pub fn report(
        &self,
        table: impl Iterator<Item = (u32, u8, u8)>,
        generated_dispatches: u64,
        interpreted_dispatches: u64,
    ) -> Coverage {
        let mut compiled: Vec<_> = table
            .map(|(address, mx, opcode)| {
                entry((address, mx, false, opcode), *self.generated.get(&(address, mx, false, opcode)).unwrap_or(&0))
            })
            .collect();
        compiled.sort_by_key(|row| (row.address, row.mx));
        let interpreted = self.interpreted.iter().map(|(&key, &count)| entry(key, count)).collect();
        let generated_instructions = self.generated.values().sum();
        let interpreted_instructions = self.interpreted.values().sum();
        let total = generated_instructions + interpreted_instructions;
        Coverage {
            schema_version: 1,
            compiled,
            interpreted,
            generated_instructions,
            interpreted_instructions,
            generated_dispatches,
            interpreted_dispatches,
            noninstruction_dispatches: generated_dispatches + interpreted_dispatches - total,
            fallback_fraction: if total == 0 { 0.0 } else { interpreted_instructions as f64 / total as f64 },
            observation: "actual executed instructions; hardware interrupt/wait/stop dispatch excluded",
        }
    }
}

/// Observe the first real opcode read without performing a diagnostic read.
/// Interrupt entry and idle steps are disarmed by the existing CPU notifications.
pub(crate) struct OpcodeBus<'a, B> {
    pub inner: B,
    pub opcode: &'a mut Option<u8>,
    pub armed: bool,
}
impl<B: rustysnes_core::cpu::Bus> rustysnes_core::cpu::Bus for OpcodeBus<'_, B> {
    fn begin_cpu_cycle(&mut self, clocks: u32) {
        self.inner.begin_cpu_cycle(clocks);
    }
    fn begin_interrupt(&mut self) {
        self.armed = false;
        self.inner.begin_interrupt();
    }
    fn halt_pending(&self) -> bool {
        self.inner.halt_pending()
    }
    fn interrupt_pending(&self, masked: bool) -> bool {
        self.inner.interrupt_pending(masked)
    }
    fn read24(&mut self, addr: u32) -> u8 {
        let value = self.inner.read24(addr);
        if self.armed {
            *self.opcode = Some(value);
            self.armed = false;
        }
        value
    }
    fn write24(&mut self, addr: u32, value: u8) {
        self.inner.write24(addr, value);
    }
    fn advance(&mut self, clocks: u32) {
        self.inner.advance(clocks);
    }
    fn access_cycles(&self, addr: u32) -> u32 {
        self.inner.access_cycles(addr)
    }
    fn poll_nmi(&mut self) -> bool {
        self.inner.poll_nmi()
    }
    fn poll_irq(&mut self) -> bool {
        self.inner.poll_irq()
    }
}
