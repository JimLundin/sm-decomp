//! Replay movie inputs, recording periodic fingerprints and game state.
use std::{fs, path::PathBuf, time::Instant};

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use snes::Reference;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().context("usage: reference-replay <movie> [--frames N] [--rom PATH] [--out CSV] [--checkpoints CSV] [--lsnes-frame-labels]")?;
    let mut rom = PathBuf::from("rom/sm.sfc");
    let mut out = PathBuf::from("out/reference-replay.csv");
    let mut limit = None;
    let mut checkpoints = None::<PathBuf>;
    let mut lsnes_frame_labels = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--frames" => limit = Some(args.next().context("--frames needs a number")?.parse::<usize>()?),
            "--rom" => rom = args.next().context("--rom needs a path")?.into(),
            "--out" => out = args.next().context("--out needs a path")?.into(),
            "--checkpoints" => checkpoints = Some(args.next().context("--checkpoints needs a path")?.into()),
            "--lsnes-frame-labels" => lsnes_frame_labels = true,
            _ => anyhow::bail!("unknown argument {arg}"),
        }
    }
    let movie = movie::load(path.as_ref())?;
    let rom = fs::read(rom)?;
    snes::verify_movie_rom(&movie, &rom)?;
    let mut reference = Reference::new(&rom, 0)?;
    let count = limit.unwrap_or(movie.frames.len());
    ensure!(count <= movie.frames.len(), "requested {count} frames, movie contains only {}", movie.frames.len());
    ensure!(count > 0, "no frames selected");
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(out)?;
    use std::io::Write;
    writeln!(file, "frame,game_state,room,wram_sha256,framebuffer_sha256")?;
    let started = Instant::now();
    let capture_metadata = checkpoints.as_ref().map(|path| path.with_extension("capture.json"));
    if let Some(path) = &capture_metadata
        && path.exists()
    {
        fs::remove_file(path)?;
    }
    let mut checkpoint_file = if let Some(path) = checkpoints {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(path)?;
        writeln!(file, "{}", snes::checkpoint::CSV_HEADER)?;
        Some(file)
    } else {
        None
    };
    let mut event = 0;
    let mut checkpoint_digest = Sha256::new();
    checkpoint_digest.update(snes::checkpoint::CSV_HEADER.as_bytes());
    checkpoint_digest.update(b"\n");
    for (index, pad) in movie.frames[..count].iter().enumerate() {
        if let Some(file) = &mut checkpoint_file {
            let mut error = None;
            reference.run_frame_observed(*pad, snes::checkpoint::CHECKPOINT_PC, |ram, scanline| {
                event += 1;
                let frame =
                    if lsnes_frame_labels { snes::checkpoint::lsnes_frame(index + 1, scanline) } else { index + 1 };
                let row = snes::checkpoint::csv_row(frame, event, ram);
                checkpoint_digest.update(row.as_bytes());
                if let Err(e) = file.write_all(row.as_bytes()) {
                    error = Some(e);
                }
            })?;
            if let Some(e) = error {
                return Err(e.into());
            }
        } else {
            reference.run_frame(*pad)?;
        }
        if index == 0 || (index + 1) % 600 == 0 || index + 1 == count {
            let ram = reference.wram();
            let word = |offset| u16::from_le_bytes([ram[offset], ram[offset + 1]]);
            let mut pixels = Sha256::new();
            for pixel in reference.framebuffer() {
                pixels.update(pixel.to_le_bytes());
            }
            writeln!(
                file,
                "{},{:04X},{:04X},{:x},{:x}",
                index + 1,
                word(0x998),
                word(0x79b),
                Sha256::digest(ram),
                pixels.finalize()
            )?;
            file.flush()?;
            eprintln!(
                "reference: {}/{} frames, game_state=${:04X}, room=${:04X} ({:.1}s)",
                index + 1,
                count,
                word(0x998),
                word(0x79b),
                started.elapsed().as_secs_f64()
            );
        }
    }
    // Publish coverage only after successful completion. Bind it to the exact
    // CSV so a stale capture cannot certify a truncated/changed checkpoint file.
    if let Some(mut file) = checkpoint_file {
        file.flush()?;
    }
    if let Some(path) = capture_metadata {
        fs::write(
            path,
            format!(
                "{{\"completed_frames\":{count},\"checkpoint_sha256\":\"{:x}\",\"core\":\"RustySNES with lsnes-profile\"}}\n",
                checkpoint_digest.finalize()
            ),
        )?;
    }
    Ok(())
}
