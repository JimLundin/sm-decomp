//! Instruction timing capture for independently assembled diagnostic cartridges.
use anyhow::{Context, Result};
use rustysnes_core::{System, cart::Cart};
use std::{fs, io::Write};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let rom = args.next().context("usage: timing-probe ROM OUTPUT [FRAMES]")?;
    let path = args.next().context("missing output path")?;
    let frames = args.next().map(|n| n.parse::<u64>()).transpose()?.unwrap_or(2);
    let mut system = System::new(0);
    system.bus.cart = Some(Cart::from_rom(&fs::read(rom)?)?);
    system.reset();
    let mut out = fs::File::create(path)?;
    while system.bus.ppu.frame_count() < frames {
        let r = system.cpu.regs;
        let pc = u32::from(r.pbr) << 16 | u32::from(r.pc);
        // This matches the recording core's instruction trace convention.
        // Diagnostic cartridges do not enable NMI or IRQ. TAS checkpoints
        // instead use ObservedBus and the actual opcode-fetch seam.
        writeln!(out, "{pc:06x},{}", system.bus.clock.master)?;
        system.cpu.step(&mut system.bus);
    }
    Ok(())
}
