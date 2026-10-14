//! Authenticate the installed corpus against the completed migration audit.
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
use anyhow::{Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, hint::black_box, path::{Path, PathBuf}, time::Instant};

#[derive(Deserialize)]
struct Row {
    source: PathBuf, destination: PathBuf, binary_sha256: String,
    source_bytes: u64, cases: usize, continuations: usize, captured: usize,
    original_writes: usize, binary_spans: usize,
}
fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    ensure!(args.len() == 3, "usage: final-audit CHECKOUT AUDITED-STAGING REPORT.json");
    let checkout = Path::new(&args[0]);
    let staging = Path::new(&args[1]);
    let rows: Vec<Row> = serde_json::from_slice(&fs::read(staging.join("migration.json"))?)?;
    ensure!(rows.len() == 177, "incomplete migration audit");
    let mut files = Vec::new();
    let mut redirects = Vec::new();
    let mut binary_bytes = 0u64;
    for row in &rows {
        ensure!(!checkout.join(&row.source).exists(), "text fixture source still exists");
        let audited = fs::read(staging.join(&row.destination))?;
        ensure!(hash(&audited) == row.binary_sha256, "audited staging changed");
        let current = fs::read(checkout.join(&row.destination))?;
        let sha256 = hash(&current);
        if sha256 != row.binary_sha256 {
            ensure!(row.destination == Path::new("tools/port/ridley-subsystem-captured.smf"), "unexpected fixture edit");
            let mut expected = fixture_format::decode(audited)?;
            let before = PathBuf::from("/home/jim/sm-decomp/tools/port/ridley-subsystem.toml");
            let after = PathBuf::from("/home/jim/sm-decomp/tools/port/ridley-subsystem.smf");
            let mut count = 0;
            for dependency in &mut expected.rust_dependencies {
                if dependency == &before { *dependency = after.clone(); count += 1; }
            }
            ensure!(count == 1, "expected exactly one dependency redirect");
            ensure!(fixture_format::decode(current.clone())? == expected, "redirect changed other metadata or writes");
            ensure!(fixture_format::encode(&expected)? == current, "redirect is not canonical");
            redirects.push(serde_json::json!({
                "file": row.destination, "before": before, "after": after,
                "audited_sha256": row.binary_sha256, "final_sha256": sha256,
                "other_metadata_and_state_unchanged": true,
            }));
        }
        let mut load_ms = Vec::new();
        if row.destination.file_stem().is_some_and(|name|
            ["floater-spike-motion", "mother-brain-bomb-update", "ridley-subsystem"].iter().any(|selected| name == *selected)
        ) {
            for _ in 0..5 {
                let started = Instant::now();
                let bytes = fs::read(checkout.join(&row.destination))?;
                ensure!(hash(&bytes) == sha256, "timed input changed");
                let decoded = fixture_format::decode(bytes)?;
                black_box(&decoded);
                load_ms.push(started.elapsed().as_secs_f64() * 1000.0);
            }
            let mut sorted = load_ms.clone();
            sorted.sort_by(f64::total_cmp);
            println!("{}: {:.3} ms median warm read/SHA-256/decode", row.destination.display(), sorted[2]);
        }
        binary_bytes += current.len() as u64;
        files.push(serde_json::json!({"file": row.destination, "bytes": current.len(), "sha256": sha256, "load_ms": load_ms}));
    }
    let actual: BTreeSet<_> = fs::read_dir(checkout.join("tools/port"))?
        .map(|entry| Ok(entry?.path())).collect::<Result<Vec<_>>>()?.into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "smf"))
        .map(|path| path.strip_prefix(checkout).unwrap().to_path_buf()).collect();
    ensure!(actual == rows.iter().map(|row| row.destination.clone()).collect(), "binary corpus inventory differs");
    ensure!(redirects.len() == 1, "dependency redirect missing");
    let report = serde_json::json!({
        "schema": "fixture-final-corpus-audit-v1", "original_revision": "f39947c4a1e45c382deb5c842934c92bb28b1fe4",
        "families": rows.len(), "source_bytes": rows.iter().map(|r| r.source_bytes).sum::<u64>(),
        "binary_bytes": binary_bytes, "cases": rows.iter().map(|r| r.cases).sum::<usize>(),
        "continuations": rows.iter().map(|r| r.continuations).sum::<usize>(),
        "captured": rows.iter().map(|r| r.captured).sum::<usize>(),
        "original_writes": rows.iter().map(|r| r.original_writes).sum::<usize>(),
        "binary_spans": rows.iter().map(|r| r.binary_spans).sum::<usize>(),
        "redirects": redirects, "files": files,
        "measurement": "Five warm file read/SHA-256/production-decode repetitions for three families; excludes state preparation, actual-C verification and gameplay; other machine work may be active",
        "machine_loadavg": fs::read_to_string("/proc/loadavg").ok(),
    });
    fs::write(&args[2], serde_json::to_vec_pretty(&report)?)?;
    println!("177 families authenticated; {} -> {} bytes; one dependency filename redirect; all other metadata and state unchanged",
        rows.iter().map(|r| r.source_bytes).sum::<u64>(), binary_bytes);
    Ok(())
}
