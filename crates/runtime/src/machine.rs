use std::collections::{BTreeMap, HashMap};
use std::fmt;

use crate::cpu::Cpu;
use crate::{Mx, mx_name};

/// The non-CPU half of the console. Implement this on top of an emulator core
/// (`snes::SnesBus` wraps RustySNES), and on a sparse RAM map for tests.
pub trait Bus {
    /// Passive notification before interrupt-entry discarded fetch.
    fn begin_interrupt(&mut self) {}
    /// Inspect the latest interrupt sample without consuming a pending edge.
    fn interrupt_pending(&self, _irq_masked: bool) -> bool {
        false
    }
    /// Whether the next cycle is paused by active HDMA.
    fn halt_pending(&self) -> bool {
        false
    }
    /// Final cycle of an implied instruction, with the next program address.
    fn implied_cycle(&mut self, _next_pc: u32, _irq_masked: bool) {
        self.tick(6);
    }
    /// Begin a CPU cycle before its clock phases; allows hardware DMA stalls.
    fn begin_cpu_cycle(&mut self, _clocks: u32) {}
    fn read(&mut self, addr: u32) -> u8;
    fn write(&mut self, addr: u32, v: u8);
    /// Advance the rest of the console by this many master clocks.
    /// Generated code calls this with per-block cycle estimates once timing work starts.
    fn tick(&mut self, _master_cycles: u32) {}
    /// Interpreter timing and interrupt hooks. Test buses can keep the defaults.
    fn access_cycles(&self, _addr: u32) -> u32 {
        6
    }
    fn poll_nmi(&mut self) -> bool {
        false
    }
    fn poll_irq(&mut self) -> bool {
        false
    }
    /// Bus/internal CPU cycles charged by a timed generated execution adapter.
    fn cpu_cycles(&self) -> u64 {
        0
    }
    /// Restore adapter bookkeeping from a verified same-machine checkpoint.
    fn restore_cpu_cycles(&mut self, _cycles: u64) {}
}

/// Executes original ROM code for anything that has no generated function yet.
/// Contract: start at `target` with the current CPU state and run until the
/// subroutine returns (stack pointer climbs back above its value at entry).
/// `snes::InterpreterFallback` drives RustySNES's 65816 core over the same `Bus`.
pub trait Fallback {
    fn run_subroutine(&mut self, cpu: &mut Cpu, bus: &mut dyn Bus, target: u32);
}

pub type GenFn = fn(&mut Machine);

/// One row of the generated function table: (24-bit address, M/X key, function).
pub type TableEntry = (u32, Mx, GenFn);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Miss {
    pub addr: u32,
    pub mx: Mx,
}

impl fmt::Display for Miss {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:02X}:{:04X} {}", self.addr >> 16, self.addr & 0xffff, mx_name(self.mx))
    }
}

pub struct Machine {
    pub cpu: Cpu,
    pub bus: Box<dyn Bus>,
    table: HashMap<(u32, Mx), GenFn>,
    fallback: Option<Box<dyn Fallback>>,
    /// Calls that went to the fallback, with counts. Dump with [`Machine::misses_csv`].
    pub misses: BTreeMap<Miss, u64>,
    /// Calls served by generated code. `generated / (generated + misses)` is the progress metric.
    pub generated_calls: u64,
}

impl Machine {
    pub fn new(bus: Box<dyn Bus>, table: &[TableEntry], fallback: Option<Box<dyn Fallback>>) -> Self {
        Self {
            cpu: Cpu::default(),
            bus,
            table: table.iter().map(|&(a, mx, f)| ((a, mx & 3), f)).collect(),
            fallback,
            misses: BTreeMap::new(),
            generated_calls: 0,
        }
    }

    // ---- memory -------------------------------------------------------------

    #[inline]
    pub fn read8(&mut self, addr: u32) -> u8 {
        self.bus.read(addr & 0xff_ffff)
    }

    #[inline]
    pub fn write8(&mut self, addr: u32, v: u8) {
        self.bus.write(addr & 0xff_ffff, v)
    }

    /// Data read of 1 or 2 bytes. The high byte comes from addr+1 across banks,
    /// which is correct for absolute/long modes. Direct-page wrap is handled in `ea`.
    #[inline]
    pub fn read(&mut self, addr: u32, wide: bool) -> u16 {
        let lo = self.read8(addr) as u16;
        if wide { lo | (self.read8(addr.wrapping_add(1)) as u16) << 8 } else { lo }
    }

    #[inline]
    pub fn write(&mut self, addr: u32, v: u16, wide: bool) {
        self.write8(addr, v as u8);
        if wide {
            self.write8(addr.wrapping_add(1), (v >> 8) as u8);
        }
    }

    /// 16-bit pointer read within bank 0 (direct page / stack pointers).
    pub fn read16_bank0(&mut self, addr: u16) -> u16 {
        let lo = self.read8(addr as u32) as u16;
        lo | (self.read8(addr.wrapping_add(1) as u32) as u16) << 8
    }

    pub fn read24_bank0(&mut self, addr: u16) -> u32 {
        self.read16_bank0(addr) as u32 | (self.read8(addr.wrapping_add(2) as u32) as u32) << 16
    }

    // ---- stack (native mode: full 16-bit S in bank 0) ----------------------

    pub fn push8(&mut self, v: u8) {
        let s = self.cpu.s;
        self.write8(s as u32, v);
        self.cpu.s = s.wrapping_sub(1);
    }

    pub fn pull8(&mut self) -> u8 {
        self.cpu.s = self.cpu.s.wrapping_add(1);
        let s = self.cpu.s;
        self.read8(s as u32)
    }

    pub fn push16(&mut self, v: u16) {
        self.push8((v >> 8) as u8);
        self.push8(v as u8);
    }

    pub fn pull16(&mut self) -> u16 {
        let lo = self.pull8() as u16;
        lo | (self.pull8() as u16) << 8
    }

    // ---- control flow -------------------------------------------------------

    /// JSR/JSL body: the emitter has already pushed the return address.
    /// Runs `target` (generated or fallback) until it returns.
    pub fn call(&mut self, target: u32) {
        let mx = self.cpu.mx();
        let saved_pb = self.cpu.pb;
        self.cpu.pb = (target >> 16) as u8;
        if let Some(&f) = self.table.get(&(target, mx)) {
            self.generated_calls += 1;
            f(self);
        } else {
            *self.misses.entry(Miss { addr: target, mx }).or_default() += 1;
            match self.fallback.take() {
                Some(mut fb) => {
                    fb.run_subroutine(&mut self.cpu, &mut *self.bus, target);
                    self.fallback = Some(fb);
                }
                None => panic!(
                    "no generated function and no fallback for {} — add it to config/functions.toml and regenerate",
                    Miss { addr: target, mx }
                ),
            }
        }
        self.cpu.pb = saved_pb;
    }

    /// JMP/JML to another function, or an indirect jump (jump tables).
    /// The callee's RTS/RTL consumes the return address our caller pushed,
    /// so after this the current generated function simply returns.
    pub fn tail(&mut self, target: u32) {
        self.call(target)
    }

    /// Emitted after every call and after PLP: the recompiler assumed these widths.
    /// A failure here is a pipeline signal, not a crash to paper over.
    #[track_caller]
    pub fn assume_mx(&self, expected: Mx, at: u32) {
        let got = self.cpu.mx();
        if got != expected {
            panic!(
                "width assumption broken at ${:02X}:{:04X}: recompiled as {}, CPU is {} — add a variant or an override",
                at >> 16,
                at & 0xffff,
                mx_name(expected),
                mx_name(got)
            );
        }
    }

    /// Emitted for instructions the emitter does not translate yet.
    #[track_caller]
    pub fn unimplemented(&mut self, at: u32, what: &str) -> ! {
        panic!("unimplemented in recompiler: {what} at ${:02X}:{:04X}", at >> 16, at & 0xffff)
    }

    /// CSV of fallback misses: `addr,mx,count`. Feed this to `trace merge`.
    pub fn misses_csv(&self) -> String {
        let mut s = String::from("addr,mx,count\n");
        for (m, n) in &self.misses {
            s.push_str(&format!("{:06X},{},{}\n", m.addr, mx_name(m.mx), n));
        }
        s
    }
}
