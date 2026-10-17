use std::{env, fs, time::Instant};

use anyhow::{Result, ensure};
use snes::Reference;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let rom = args.next().unwrap_or_else(|| "rom/sm.sfc".into());
    let frames: usize = args.next().unwrap_or_else(|| "120".into()).parse()?;
    ensure!(frames > 0, "frame count must be positive");
    let mut reference = Reference::new(&fs::read(rom)?, 0)?;
    let started = Instant::now();
    for _ in 0..frames {
        reference.run_frame(0)?;
    }
    // A reproducible fingerprint for setup checks, not an accuracy oracle.
    let fingerprint = reference
        .wram()
        .iter()
        .fold(0xcbf29ce484222325u64, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3));
    println!(
        "reference: {frames} frames in {:.2}s; WRAM fingerprint {fingerprint:016x}; {} framebuffer pixels",
        started.elapsed().as_secs_f64(),
        reference.framebuffer().len()
    );
    Ok(())
}
