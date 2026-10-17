//! One bounded cost measurement using synthetic state, with no game dependency.
use sm_trace::{Identity, Reference, WRAM_BYTES, Writer};
use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(1).map(PathBuf::from).ok_or("provide a fresh trace path")?;
    let identity = Identity {
        reference: "synthetic-cost-only".into(),
        rom: "none".into(),
        host: "hash-cost-example-v1".into(),
        input: "ordinal-low-byte".into(),
        observation: "128KiB-WRAM+64KiB-external+32-byte-effects".into(),
        scenario: "10000-steps".into(),
    };
    let mut wram = vec![0; WRAM_BYTES];
    let external = vec![1; 64 * 1024];
    let effects = [2; 32];
    let start = Instant::now();
    let mut writer = Writer::create(&path, &identity)?;
    for ordinal in 1..=10000 {
        wram[WRAM_BYTES - 1] = ordinal as u8;
        writer.record(ordinal, &wram, &external, &effects)?;
    }
    let summary = writer.finish()?;
    let write_time = start.elapsed();
    let start = Instant::now();
    let reference = Reference::load(&path, &identity)?;
    let load_time = start.elapsed();
    let start = Instant::now();
    let mut comparison = reference.comparison();
    for ordinal in 1..=10000 {
        wram[WRAM_BYTES - 1] = ordinal as u8;
        comparison.check(ordinal, &wram, &external, &effects)?;
    }
    assert_eq!(comparison.finish()?, summary);
    println!(
        "records={} trace_bytes={} write_seconds={:.6} load_seconds={:.6} compare_seconds={:.6}",
        summary.records,
        std::fs::metadata(path)?.len(),
        write_time.as_secs_f64(),
        load_time.as_secs_f64(),
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
