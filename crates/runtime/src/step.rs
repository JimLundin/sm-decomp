//! Instruction-sized generated execution. Timing is explicit, so callers can
//! resume at each instruction and service interrupts or stop at a frame edge.
use crate::{Bus, Machine, Mx};

pub type StepFn = fn(&mut Machine);

/// Native-mode instruction address, widths, encoded bytes, and compiled body.
pub type StepEntry = (u32, Mx, &'static [u8], StepFn);

/// Fetch the exact compiled bytes, execute their timed body, and advance within
/// the program bank. Interrupt dispatch belongs to the caller before this step.
pub fn execute_step(machine: &mut Machine, pc: u32, bytes: &[u8], body: StepFn) -> Result<u32, u32> {
    for (index, expected) in bytes.iter().enumerate() {
        let address = (pc & 0xff0000) | u32::from((pc as u16).wrapping_add(index as u16));
        if machine.read8(address) != *expected {
            return Err(address);
        }
    }
    body(machine);
    Ok((pc & 0xff0000) | u32::from((pc as u16).wrapping_add(bytes.len() as u16)))
}

/// Bus cycles for generated instructions: reads latch four clocks before the
/// end of a cycle, writes latch at its end, and internal cycles take six clocks.
/// The frame runner uses the same sequencing as the interpreter's bus contract.
pub struct TimedBus<B> {
    pub inner: B,
    pub cycles: u64,
}

impl<B> TimedBus<B> {
    pub fn new(inner: B) -> Self {
        Self { inner, cycles: 0 }
    }
}

impl<B: Bus> Bus for TimedBus<B> {
    fn implied_cycle(&mut self, next_pc: u32, irq_masked: bool) {
        let halt = self.inner.halt_pending();
        self.inner.begin_cpu_cycle(6);
        if halt || self.inner.interrupt_pending(irq_masked) {
            let clocks = self.inner.access_cycles(next_pc);
            self.inner.tick(clocks - 4);
            self.inner.read(next_pc);
            self.inner.tick(4);
        } else {
            self.inner.tick(6);
        }
        self.cycles += 1;
    }
    fn read(&mut self, addr: u32) -> u8 {
        let clocks = self.inner.access_cycles(addr);
        self.inner.begin_cpu_cycle(clocks);
        self.inner.tick(clocks - 4);
        let value = self.inner.read(addr);
        self.inner.tick(4);
        self.cycles += 1;
        value
    }
    fn write(&mut self, addr: u32, value: u8) {
        self.inner.begin_cpu_cycle(self.inner.access_cycles(addr));
        self.inner.tick(self.inner.access_cycles(addr));
        self.inner.write(addr, value);
        self.cycles += 1;
    }
    fn tick(&mut self, clocks: u32) {
        self.inner.begin_cpu_cycle(clocks);
        self.inner.tick(clocks);
        self.cycles += u64::from(clocks / 6);
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
    fn cpu_cycles(&self) -> u64 {
        self.cycles
    }
}
