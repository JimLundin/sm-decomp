//! movie <in.lsmv|in.bk2|Input Log.txt> <out.inputs>

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let mut a = std::env::args().skip(1);
    let input = a.next().context("usage: movie <in.lsmv|in.bk2> <out.inputs>")?;
    if ["fetch-corpus", "import-corpus"].contains(&input.as_str()) {
        let args = a.collect::<Vec<_>>();
        anyhow::ensure!(args.iter().all(|arg| arg == "--offline"), "unknown corpus option");
        println!(
            "{}",
            toml::to_string_pretty(&movie::corpus::corpus(
                &std::env::current_dir()?,
                &input,
                args.iter().any(|arg| arg == "--offline")
            )?)?
        );
        return Ok(());
    }
    let output = a.next().context("usage: movie <in.lsmv|in.bk2> <out.inputs>")?;
    let movie = movie::load(input.as_ref())?;
    std::fs::write(&output, movie::write_inputs(&movie.frames))?;
    eprintln!("movie: {} frames → {output}", movie.frames.len());
    if let Some(hash) = movie.rom_sha256 {
        eprintln!("recorded ROM SHA-256: {hash}");
    }
    Ok(())
}
