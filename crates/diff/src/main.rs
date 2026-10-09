//! diff <movie.inputs> — run the lockstep comparison.
//!
//! Not wired up yet: needs the two `Console` implementations.
//!   reference = RustySNES running rom/sm.sfc
//!   subject   = game::machine() on the same RustySNES hardware, CPU replaced
//! See docs/PIPELINE.md, milestone 1–2.

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let path = std::env::args().nth(1).context("usage: diff <movie.inputs>")?;
    let inputs = movie::read_inputs(&std::fs::read_to_string(&path)?)?;
    eprintln!(
        "diff: loaded {} frames from {path}. Reference/subject consoles are not wired up yet \
         (milestone 1–2 in docs/PIPELINE.md).",
        inputs.len()
    );
    Ok(())
}
