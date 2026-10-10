use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use anyhow::{Result, ensure};
use runtime::{Bus, Cpu, Fallback, Miss};
use rustysnes_core::{
    System,
    cpu::{Bus as CpuBus, Cpu as Interpreter, Regs, Status},
};

/// Shared SNES hardware for generated CPU code and the interpreter fallback.
/// Hardware stepping uses RustySNES's CPU-side bus interface, including DMA.
#[derive(Clone)]
pub struct SnesBus {
    hardware: Rc<RefCell<rustysnes_core::Bus>>,
}

impl Bus for SnesBus {
    fn halt_pending(&self) -> bool {
        self.hardware.borrow().halt_pending()
    }
    fn interrupt_pending(&self, irq_masked: bool) -> bool {
        self.hardware.borrow().interrupt_pending(irq_masked)
    }
    fn begin_cpu_cycle(&mut self, clocks: u32) {
        self.hardware.borrow_mut().begin_cpu_cycle(clocks);
    }
    fn read(&mut self, addr: u32) -> u8 {
        self.hardware.borrow_mut().read24(addr)
    }
    fn write(&mut self, addr: u32, value: u8) {
        self.hardware.borrow_mut().write24(addr, value);
    }
    fn tick(&mut self, clocks: u32) {
        self.hardware.borrow_mut().advance(clocks);
    }
    fn access_cycles(&self, addr: u32) -> u32 {
        self.hardware.borrow().access_cycles(addr)
    }
    fn poll_nmi(&mut self) -> bool {
        self.hardware.borrow_mut().poll_nmi()
    }
    fn poll_irq(&mut self) -> bool {
        self.hardware.borrow_mut().poll_irq()
    }
}

// The interpreter always borrows the runtime's actual bus, including in a
// generated -> fallback -> generated transition. No second memory map exists.
struct InterpreterBus<'a>(&'a mut dyn Bus);
impl CpuBus for InterpreterBus<'_> {
    fn halt_pending(&self) -> bool {
        self.0.halt_pending()
    }
    fn interrupt_pending(&self, irq_masked: bool) -> bool {
        self.0.interrupt_pending(irq_masked)
    }
    fn begin_cpu_cycle(&mut self, clocks: u32) {
        self.0.begin_cpu_cycle(clocks);
    }
    fn read24(&mut self, addr: u32) -> u8 {
        self.0.read(addr)
    }
    fn write24(&mut self, addr: u32, value: u8) {
        self.0.write(addr, value);
    }
    fn advance(&mut self, clocks: u32) {
        self.0.tick(clocks);
    }
    fn access_cycles(&self, addr: u32) -> u32 {
        self.0.access_cycles(addr)
    }
    fn poll_nmi(&mut self) -> bool {
        self.0.poll_nmi()
    }
    fn poll_irq(&mut self) -> bool {
        self.0.poll_irq()
    }
}

// A generated candidate must poll interrupts before fetching its opcode. If
// the interpreter takes over, replay those exact polls rather than losing NMI.
struct PolledBus<'a> {
    bus: InterpreterBus<'a>,
    nmi: bool,
    irq: bool,
}
impl CpuBus for PolledBus<'_> {
    fn halt_pending(&self) -> bool {
        self.bus.halt_pending()
    }
    fn interrupt_pending(&self, irq_masked: bool) -> bool {
        self.bus.interrupt_pending(irq_masked)
    }
    fn begin_cpu_cycle(&mut self, clocks: u32) {
        self.bus.begin_cpu_cycle(clocks);
    }
    fn read24(&mut self, addr: u32) -> u8 {
        self.bus.read24(addr)
    }
    fn write24(&mut self, addr: u32, value: u8) {
        self.bus.write24(addr, value);
    }
    fn advance(&mut self, clocks: u32) {
        self.bus.advance(clocks);
    }
    fn access_cycles(&self, addr: u32) -> u32 {
        self.bus.access_cycles(addr)
    }
    fn poll_nmi(&mut self) -> bool {
        self.nmi
    }
    fn poll_irq(&mut self) -> bool {
        self.irq
    }
}

fn runtime_regs(r: Regs) -> Cpu {
    Cpu { a: r.a, x: r.x, y: r.y, s: r.s, d: r.d, db: r.dbr, pb: r.pbr, p: r.p.bits(), e: r.emulation }
}
fn interpreter_regs(r: &Cpu, pc: u16) -> Regs {
    Regs {
        a: r.a,
        x: r.x,
        y: r.y,
        s: r.s,
        d: r.d,
        dbr: r.db,
        pbr: r.pb,
        pc,
        p: Status::from_bits_retain(r.p),
        emulation: r.e,
    }
}

/// Execute missing native-mode subroutines on the caller's hardware and stack.
/// Requires a balanced conventional RTS/RTL return. Routines that pop or replace
/// their own return address need overrides; generated frame dispatch is separate.
#[derive(Default)]
pub struct InterpreterFallback;

impl Fallback for InterpreterFallback {
    fn run_subroutine(&mut self, cpu: &mut Cpu, bus: &mut dyn Bus, target: u32) {
        assert!(!cpu.e, "subroutine fallback requires native mode; boot runs in the frame interpreter");
        let mut interpreter = Interpreter::new();
        interpreter.regs = Regs {
            a: cpu.a,
            x: cpu.x,
            y: cpu.y,
            s: cpu.s,
            d: cpu.d,
            dbr: cpu.db,
            pbr: (target >> 16) as u8,
            pc: target as u16,
            p: Status::from_bits_retain(cpu.p),
            emulation: cpu.e,
        };
        let entry_stack = cpu.s;
        let mut bus = InterpreterBus(bus);
        for _ in 0..2_000_000 {
            interpreter.step(&mut bus);
            // The caller has already pushed its return address. Nested calls
            // and interrupts return to this depth; RTS/RTL of our entry climbs above it.
            if interpreter.regs.s > entry_stack {
                let regs = interpreter.regs;
                *cpu = Cpu {
                    a: regs.a,
                    x: regs.x,
                    y: regs.y,
                    s: regs.s,
                    d: regs.d,
                    db: regs.dbr,
                    pb: regs.pbr,
                    p: regs.p.bits(),
                    e: regs.emulation,
                };
                return;
            }
            assert!(
                !interpreter.stopped,
                "fallback stopped at ${:02X}:{:04X}",
                interpreter.regs.pbr, interpreter.regs.pc
            );
        }
        panic!("fallback budget exceeded at ${target:06X}; routine did not return");
    }
}

/// The subject with an empty generated table: interpreter CPU through SnesBus.
/// This proves hardware/frame integration before cycle-aware generated dispatch.
pub struct Subject {
    bus: SnesBus,
    interpreter: Interpreter,
    misses: BTreeMap<Miss, u64>,
    wram: Vec<u8>,
    machine: runtime::Machine,
    steps: HashMap<(u32, u8), (&'static [u8], runtime::StepFn)>,
    accounting: super::coverage::Accounting,
    pub generated_steps: u64,
    pub interpreted_steps: u64,
}

/// Passive instruction-boundary values for independently specified timing tests.
#[derive(Clone, Copy, Debug)]
pub struct InstructionBoundary {
    pub pc: u32,
    pub cpu_cycles: u64,
    pub master_clock: u64,
}

impl Subject {
    /// Construct only for the base NTSC SNES hardware used by Super Metroid.
    pub fn fallback_only(rom: &[u8], seed: u64) -> Result<Self> {
        Self::with_steps(rom, seed, &[])
    }

    pub fn with_steps(rom: &[u8], seed: u64, steps: &[runtime::StepEntry]) -> Result<Self> {
        let reference = super::Reference::new(rom, seed)?;
        let cart = reference.system.bus.cart.as_ref().unwrap();
        ensure!(
            cart.header.coprocessor == rustysnes_core::cart::header::Coprocessor::None,
            "subject does not support cartridge coprocessors"
        );
        ensure!(
            cart.header.region == rustysnes_core::cart::header::Region::Ntsc,
            "subject currently requires an NTSC cartridge"
        );
        let System { bus, cpu, .. } = reference.system;
        let wram = bus.wram().to_vec();
        let bus = SnesBus { hardware: Rc::new(RefCell::new(bus)) };
        let machine = runtime::Machine::new(Box::new(runtime::TimedBus::new(bus.clone())), &[], None);
        Ok(Self {
            bus,
            interpreter: cpu,
            misses: BTreeMap::new(),
            wram,
            machine,
            steps: steps.iter().map(|&(addr, mx, bytes, f)| ((addr, mx), (bytes, f))).collect(),
            accounting: super::coverage::Accounting::default(),
            generated_steps: 0,
            interpreted_steps: 0,
        })
    }

    /// Advance a frame through the runtime bus, collecting actual JSR/JSL targets.
    pub fn run_frame(&mut self, pad: u16) -> Result<()> {
        self.run_frame_inspected(pad, |_, _| {})
    }

    /// Inspect existing instruction boundaries without reading the emulated bus,
    /// ticking hardware, or polling/consuming an interrupt.
    pub fn run_frame_inspected(&mut self, pad: u16, mut inspect: impl FnMut(InstructionBoundary, &[u8])) -> Result<()> {
        self.bus.hardware.borrow_mut().set_joypad(0, pad);
        let frame = self.bus.hardware.borrow().ppu.frame_count();
        for _ in 0..2_000_000 {
            {
                let hardware = self.bus.hardware.borrow();
                let regs = self.interpreter.regs;
                inspect(
                    InstructionBoundary {
                        pc: (u32::from(regs.pbr) << 16) | u32::from(regs.pc),
                        cpu_cycles: self.interpreter.cycles,
                        master_clock: hardware.clock.master,
                    },
                    hardware.wram(),
                );
            }
            let before = self.interpreter.regs;
            let waiting = self.interpreter.waiting;
            let stopped = self.interpreter.stopped;
            let interrupts = self.interpreter.interrupts_taken;
            let opcode = self.bus.hardware.borrow_mut().peek((u32::from(before.pbr) << 16) | u32::from(before.pc));
            let addr = (u32::from(before.pbr) << 16) | u32::from(before.pc);
            let mx = (u8::from(before.m8()) << 1) | u8::from(before.x8());
            let candidate = self.steps.get(&(addr, mx)).copied().filter(|(bytes, _)| {
                !before.emulation
                    && !waiting
                    && !stopped
                    && bytes.iter().enumerate().all(|(index, value)| {
                        self.bus
                            .hardware
                            .borrow_mut()
                            .peek((addr & 0xff0000) | (u32::from(before.pc.wrapping_add(index as u16))))
                            == *value
                    })
            });
            let mut executed_generated = false;
            if let Some((bytes, body)) = candidate {
                let nmi = self.bus.poll_nmi();
                let irq = self.bus.poll_irq();
                if nmi || (irq && !self.interpreter.irq_masked()) {
                    self.interpreter.step(&mut PolledBus { bus: InterpreterBus(&mut self.bus), nmi, irq });
                    self.interpreted_steps += 1;
                } else {
                    self.interpreter.take_irq_mask();
                    self.machine.cpu = runtime_regs(before);
                    let cycles_before = self.machine.bus.cpu_cycles();
                    // Fetch even constant-folded ROM operands: these bus accesses
                    // preserve open bus, CPU/DMA timing, and hardware progression.
                    let next = runtime::step::execute_step(&mut self.machine, addr, bytes, body)
                        .map_err(|changed| anyhow::anyhow!("compiled instruction changed at ${changed:06X}"))?;
                    self.interpreter.regs = interpreter_regs(&self.machine.cpu, next as u16);
                    if matches!(bytes[0], 0x58 | 0x78 | 0x28 | 0xc2 | 0xe2) {
                        self.interpreter.record_irq_mask(before.p.contains(Status::I));
                    }
                    self.interpreter.cycles += self.machine.bus.cpu_cycles() - cycles_before;
                    self.generated_steps += 1;
                    executed_generated = true;
                }
            } else {
                self.interpreter.step(&mut InterpreterBus(&mut self.bus));
                self.interpreted_steps += 1;
            }
            if !waiting && !stopped && self.interpreter.interrupts_taken == interrupts {
                self.accounting.record(executed_generated, addr, mx, before.emulation, opcode);
            }
            if !before.emulation
                && !waiting
                && !stopped
                && self.interpreter.interrupts_taken == interrupts
                && matches!(opcode, 0x20 | 0x22 | 0xfc)
            {
                let regs = self.interpreter.regs;
                let miss = Miss {
                    addr: (u32::from(regs.pbr) << 16) | u32::from(regs.pc),
                    mx: (u8::from(regs.m8()) << 1) | u8::from(regs.x8()),
                };
                *self.misses.entry(miss).or_default() += 1;
            }
            let hardware = self.bus.hardware.borrow();
            if hardware.ppu.frame_count() != frame {
                ensure!(hardware.ppu.frame_count() == frame + 1, "subject skipped a frame");
                self.wram.copy_from_slice(hardware.wram());
                return Ok(());
            }
        }
        anyhow::bail!("subject exceeded instruction budget at frame {frame}");
    }

    pub fn wram(&self) -> &[u8] {
        &self.wram
    }

    /// Bus handle for native compiled/fallback routine integration checks.
    pub fn bus(&self) -> SnesBus {
        self.bus.clone()
    }

    pub fn state_bytes(&self) -> Vec<u8> {
        super::state_bytes(&self.interpreter, &self.bus.hardware.borrow())
    }
    pub fn framebuffer_bytes(&self) -> Vec<u8> {
        self.bus.hardware.borrow().framebuffer().iter().flat_map(|pixel| pixel.to_le_bytes()).collect()
    }

    /// Actual instruction counts and complete compiled-key inventory, excluding
    /// hardware interrupt dispatch and idle wait/stop steps.
    pub fn coverage(&self) -> super::coverage::Coverage {
        self.accounting.report(self.steps.iter().map(|(&(address, mx), (bytes, _))| (address, mx, bytes[0])),
                               self.generated_steps, self.interpreted_steps)
    }

    pub fn misses_csv(&self) -> String {
        let mut text = String::from("addr,mx,count\n");
        for (miss, count) in &self.misses {
            text.push_str(&format!("{:06X},{},{count}\n", miss.addr, runtime::mx_name(miss.mx)));
        }
        text
    }
}
