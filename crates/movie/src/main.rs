//! movie <in.bk2|Input Log.txt> <out.inputs>

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let mut a = std::env::args().skip(1);
    let input = a.next().context("usage: movie <in.bk2> <out.inputs>")?;
    let output = a.next().context("usage: movie <in.bk2> <out.inputs>")?;
    let log = movie::read_input_log(input.as_ref())?;
    let frames = movie::parse_bk2_log(&log)?;
    std::fs::write(&output, movie::write_inputs(&frames))?;
    eprintln!("movie: {} frames → {output}", frames.len());
    Ok(())
}
