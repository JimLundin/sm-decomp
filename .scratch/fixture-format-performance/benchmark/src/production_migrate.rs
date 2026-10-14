//! One-time migration and equivalence audit using the exact production codec.
#![allow(dead_code)]
#[path = "../../../../crates/pipeline/src/family_model.rs"]
mod family_model;
#[path = "../../../../crates/pipeline/src/fixture_format.rs"]
mod fixture_format;
mod inputs {
    pub mod bounded {
        pub fn check() -> anyhow::Result<()> { Ok(()) }
        pub fn read(path: impl AsRef<std::path::Path>) -> anyhow::Result<Vec<u8>> { Ok(std::fs::read(path)?) }
        pub fn limited<T>(f: impl FnOnce() -> anyhow::Result<T>) -> anyhow::Result<T> { f() }
    }
}
mod family {
    pub fn unique_id() -> usize {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}
use anyhow::{Context, Result, ensure};
use family_model::{Case, Manifest};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{fs, path::{Path, PathBuf}, time::Instant, hint::black_box};

fn cases(m: &Manifest) -> impl Iterator<Item = &Case> { m.cases.iter().chain(m.routines.iter().flat_map(|r| &r.cases)) }
fn strip(mut m: Manifest) -> Manifest {
    for c in m.cases.iter_mut().chain(m.routines.iter_mut().flat_map(|r| &mut r.cases)) { c.writes.clear(); }
    m
}
fn state(c: &Case, seed: u8) -> Vec<u8> {
    // Captured before_file is an unpatchable external input. Vary every input
    // byte for both captured inputs and continuations to verify this contract.
    let mut state = if c.continue_from.is_some() || c.before_file.is_some() {
        (0..0x20000usize).map(|i| (i.wrapping_mul(37) as u8).wrapping_add(seed)).collect()
    } else { vec![c.fill.unwrap_or(0); 0x20000] };
    for w in &c.writes { state[w.offset..w.offset + w.data.len()].copy_from_slice(w.data); }
    state
}
fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
#[derive(Serialize)]
struct Row {
    source: PathBuf, destination: PathBuf, family: String, source_bytes: usize, binary_bytes: usize,
    source_sha256: String, binary_sha256: String, cases: usize, continuations: usize, captured: usize,
    original_writes: usize, binary_spans: usize, compile_ms: f64, load_ms: Vec<f64>,
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    ensure!(args.len() == 2, "usage: production-migrate CHECKOUT NEW-STAGING-DIRECTORY");
    let checkout = Path::new(&args[0]);
    let output = Path::new(&args[1]);
    ensure!(!output.exists(), "staging directory exists");
    fs::create_dir_all(output)?;
    let mut paths: Vec<_> = fs::read_dir(checkout.join("tools/port"))?.map(|p| Ok(p?.path())).collect::<Result<_>>()?;
    paths.retain(|p| p.extension().is_some_and(|v| v == "toml") && !["batch", "source-inventory"].iter().any(|n| p.file_stem().is_some_and(|s| s == *n)));
    paths.sort();
    let mut rows = Vec::new();
    for path in paths {
        let relative = path.strip_prefix(checkout)?;
        let source = fs::read(&path)?;
        let original: Manifest = toml::from_str(std::str::from_utf8(&source)?).with_context(|| relative.display().to_string())?;
        let started = Instant::now();
        let bytes = fixture_format::encode(&original).with_context(|| relative.display().to_string())?;
        let compile_ms = started.elapsed().as_secs_f64() * 1000.0;
        let decoded = fixture_format::decode(bytes.clone()).with_context(|| relative.display().to_string())?;
        ensure!(strip(original.clone()) == strip(decoded.clone()), "metadata changed: {}", relative.display());
        for (old, new) in cases(&original).zip(cases(&decoded)) {
            for seed in [0, 73, 211] { ensure!(state(old, seed) == state(new, seed), "state changed: {} / {} seed {seed}", relative.display(), old.name); }
        }
        ensure!(fixture_format::encode(&decoded)? == bytes, "noncanonical roundtrip: {}", relative.display());
        let destination = relative.with_extension("smf");
        let staged = output.join(&destination);
        fs::create_dir_all(staged.parent().context("staged parent")?)?;
        fs::write(&staged, &bytes)?;
        let mut load_ms = Vec::new();
        if ["floater-spike-motion", "mother-brain-bomb-update", "ridley-subsystem"].contains(&original.family.as_str()) {
            let expected_hash = hash(&bytes);
            for _ in 0..5 {
                let started = Instant::now();
                let bytes = fs::read(&staged)?;
                ensure!(hash(&bytes) == expected_hash, "timed input changed");
                let decoded = fixture_format::decode(bytes)?;
                black_box(&decoded);
                load_ms.push(started.elapsed().as_secs_f64() * 1000.0);
            }
        }
        let row = Row {
            source: relative.into(), destination, family: original.family.clone(), source_bytes: source.len(), binary_bytes: bytes.len(),
            source_sha256: hash(&source), binary_sha256: hash(&bytes), cases: cases(&original).count(),
            continuations: cases(&original).filter(|c| c.continue_from.is_some()).count(), captured: cases(&original).filter(|c| c.before_file.is_some()).count(),
            original_writes: cases(&original).map(|c| c.writes.len()).sum(), binary_spans: cases(&decoded).map(|c| c.writes.len()).sum(), compile_ms, load_ms,
        };
        eprintln!("{}: {} cases, {} -> {} bytes", row.family, row.cases, row.source_bytes, row.binary_bytes);
        rows.push(row);
    }
    fs::write(output.join("migration.json"), serde_json::to_vec_pretty(&rows)?)?;
    Ok(())
}
