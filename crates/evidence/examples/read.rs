//! Warm-store read timings, including full logical identity authentication.
use anyhow::{Context, Result};
use evidence::*;
use std::io;
use std::path::Path;
use std::time::Instant;
#[derive(serde::Serialize)]
struct FrameReport {
    frame: u64,
    events: u64,
    seconds: f64,
}
#[derive(serde::Serialize)]
struct Report {
    open_seconds: f64,
    frames: Vec<FrameReport>,
    snapshot_seconds: Vec<f64>,
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    anyhow::ensure!(args.len() == 2, "usage: read STORE RUN_ID");
    let start = Instant::now();
    let store = Store::open(Path::new(&args[0]))?;
    let opened = start.elapsed().as_secs_f64();
    let run = read_run(&store, &parse_id(&args[1])?)?;
    let entries = read_tree(&store, &run.observations)?;
    let stream = entries.iter().find(|e| e.name == "hardware.log").context("missing hardware.log")?;
    let snapshot = entries.iter().find(|e| e.name.starts_with("frame-")).context("missing frame snapshot")?;
    let mut frames = Vec::new();
    let mut images = Vec::new();
    for frame in [60, 614, 9156, 1408, 72226] {
        let start = Instant::now();
        let events = write_frame(&store, &stream.id, frame, &mut io::sink())?;
        frames.push(FrameReport { frame, events, seconds: start.elapsed().as_secs_f64() });
    }
    for _ in 0..5 {
        let start = Instant::now();
        write_entry(&store, snapshot.kind, &snapshot.id, &mut io::sink())?;
        images.push(start.elapsed().as_secs_f64());
    }
    print!("{}", toml::to_string(&Report { open_seconds: opened, frames, snapshot_seconds: images })?);
    Ok(())
}
