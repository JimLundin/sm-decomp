//! SNES hardware integration, kept separate from the generated game and CPU runtime.
//!
//! The reference runs RustySNES's CPU and hardware together. The subject's
//! `runtime::Bus` and `runtime::Fallback` adapters share that hardware; CPU
//! instruction semantics and ROM-derived Rust stay in their own crates.

use anyhow::{Result, ensure};
use rustysnes_core::{System, cart::Cart};

pub mod checkpoint;
mod subject;
pub use subject::{InterpreterFallback, SnesBus, Subject};

fn state_bytes(cpu: &rustysnes_core::cpu::Cpu, bus: &rustysnes_core::Bus) -> Vec<u8> {
    let r = cpu.regs;
    let mut bytes = Vec::new();
    for value in [r.a, r.x, r.y, r.s, r.d, r.pc] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend([
        r.dbr,
        r.pbr,
        r.p.bits(),
        u8::from(r.emulation),
        u8::from(cpu.waiting),
        u8::from(cpu.stopped),
        u8::from(cpu.irq_masked()),
    ]);
    for value in [cpu.cycles, cpu.interrupts_taken, bus.clock.master] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// Match a container's recorded ROM before executing any of its input.
pub fn verify_movie_rom(movie: &movie::Movie, rom: &[u8]) -> Result<()> {
    use sha2::{Digest, Sha256};
    if let Some(expected) = &movie.rom_sha256 {
        ensure!(format!("{:x}", Sha256::digest(rom)) == *expected, "movie ROM SHA-256 does not match the supplied ROM");
    }
    Ok(())
}

/// A deterministic, headless reference console, powered on from a ROM and seed.
pub struct Reference {
    system: System,
}

impl Reference {
    /// Save the complete hardware and CPU state between instructions.
    pub fn save_state(&self) -> Vec<u8> {
        self.system.save_state()
    }

    /// Load a ROM and reset the machine. Use the same seed and inputs for replays.
    pub fn new(rom: &[u8], seed: u64) -> Result<Self> {
        let cart = Cart::from_rom(rom)?;
        let mut system = System::new(seed);
        system.bus.cart = Some(cart);
        system.reset();
        Ok(Self { system })
    }

    /// Advance one frame with the SNES P1 joypad bitmask.
    pub fn run_frame(&mut self, pad: u16) -> Result<()> {
        let before = self.system.bus.ppu.frame_count();
        self.system.bus.set_joypad(0, pad);
        self.system.run_frame();
        ensure!(self.system.bus.ppu.frame_count() == before + 1, "reference failed to advance exactly one frame");
        Ok(())
    }

    /// Replay one frame, observing a software checkpoint before executing the
    /// instruction at `pc`. The callback sees RAM without changing hardware.
    pub fn run_frame_observed(&mut self, pad: u16, pc: u32, mut observe: impl FnMut(&[u8], u16)) -> Result<()> {
        let frame = self.system.bus.ppu.frame_count();
        ensure!(
            self.system.bus.cart.as_ref().unwrap().header.coprocessor
                == rustysnes_core::cart::header::Coprocessor::None,
            "observed replay requires a base cartridge"
        );
        self.system.bus.set_joypad(0, pad);
        for _ in 0..2_000_000 {
            let regs = self.system.cpu.regs;
            let armed = !self.system.cpu.waiting
                && !self.system.cpu.stopped
                && (u32::from(regs.pbr) << 16 | u32::from(regs.pc)) == pc;
            self.system.cpu.step(&mut checkpoint::ObservedBus {
                bus: &mut self.system.bus,
                pc,
                observe: &mut observe,
                armed,
            });
            if self.system.bus.ppu.frame_count() != frame {
                ensure!(self.system.bus.ppu.frame_count() == frame + 1, "reference skipped a frame");
                return Ok(());
            }
        }
        anyhow::bail!("reference exceeded frame budget");
    }

    /// Read the complete 128 KiB work RAM without affecting the emulated bus.
    pub fn wram(&self) -> &[u8] {
        self.system.bus.wram()
    }

    /// Read the hardware's current framebuffer.
    pub fn framebuffer(&self) -> &[u16] {
        self.system.bus.framebuffer()
    }
    pub fn state_bytes(&self) -> Vec<u8> {
        state_bytes(&self.system.cpu, &self.system.bus)
    }
    pub fn framebuffer_bytes(&self) -> Vec<u8> {
        self.framebuffer().iter().flat_map(|pixel| pixel.to_le_bytes()).collect()
    }
}
