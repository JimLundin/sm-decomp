//! movie <in.lsmv|in.bk2|Input Log.txt> <out.inputs>

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let mut a = std::env::args().skip(1);
    let input = a.next().context("usage: movie <in.lsmv|in.bk2> <out.inputs>")?;
    let output = a.next().context("usage: movie <in.lsmv|in.bk2> <out.inputs>")?;
    let movie = movie::load(input.as_ref())?;
    std::fs::write(&output, movie::write_inputs(&movie.frames))?;
    eprintln!("movie: {} frames → {output}", movie.frames.len());
    if let Some(hash) = movie.rom_sha256 {
        eprintln!("recorded ROM SHA-256: {hash}");
    }
    Ok(())
}
