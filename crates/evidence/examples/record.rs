//! Disk-backed native producer microbenchmark, excluding source text parsing.
use anyhow::Result;
use evidence::{Event, Store, TraceWriter, hex};
use std::path::Path;
use std::time::Instant;
#[derive(serde::Serialize)]
struct Report {
    events: u64,
    stream: String,
    seconds: f64,
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    anyhow::ensure!(args.len() == 2, "usage: record NEW_STORE EVENT_COUNT");
    let count: u64 = args[1].parse()?;
    let start = Instant::now();
    let mut store = Store::create(Path::new(&args[0]))?;
    let mut recorder = TraceWriter::new(&mut store);
    for i in 0..count {
        recorder.push(Event::Ppu {
            frame: i / 65536,
            device: 0,
            register: if i % 2 == 0 { 24 } else { 25 },
            value: i % 256,
        })?;
    }
    let stream = recorder.finish()?;
    store.commit()?;
    print!(
        "{}",
        toml::to_string(&Report { events: count, stream: hex(&stream), seconds: start.elapsed().as_secs_f64() })?
    );
    Ok(())
}
