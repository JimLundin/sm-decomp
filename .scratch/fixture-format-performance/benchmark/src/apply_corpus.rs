//! Install only the corpus whose metadata and state equivalence audit passed.
use anyhow::{Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{fs, path::{Path, PathBuf}, process::Command};
#[derive(Deserialize)]
struct Row {
    source: PathBuf, destination: PathBuf, family: String, source_sha256: String, binary_sha256: String,
    source_bytes: u64, binary_bytes: u64, cases: usize, continuations: usize, captured: usize,
    original_writes: usize, binary_spans: usize, load_ms: Vec<f64>,
}
fn hash(path: &Path) -> Result<String> { Ok(format!("{:x}", Sha256::digest(fs::read(path)?))) }
fn main() -> Result<()> {
    let staging = Path::new("/tmp/sm-binary-fixtures-j6gExW/staging-v2");
    let rows: Vec<Row> = serde_json::from_slice(&fs::read(staging.join("migration.json"))?)?;
    ensure!(rows.len() == 177, "incomplete source corpus");
    for row in &rows {
        ensure!(hash(&row.source)? == row.source_sha256, "source changed: {}", row.source.display());
        ensure!(hash(&staging.join(&row.destination))? == row.binary_sha256, "staged fixture changed");
        ensure!(!row.destination.exists(), "binary destination already exists");
    }
    // Every source and destination has been authenticated before the first edit.
    for row in &rows {
        fs::copy(staging.join(&row.destination), &row.destination)?;
        fs::remove_file(&row.source)?;
    }
    let inventory = Command::new("git").args(["ls-files", "-z"]).output()?;
    ensure!(inventory.status.success(), "tracked source inventory failed");
    for name in inventory.stdout.split(|&b| b == 0).filter(|b| !b.is_empty()) {
        let name = std::str::from_utf8(name)?;
        if !(name.starts_with("crates/") || name.starts_with("docs/") || name.starts_with("tools/") || ["justfile", "README.md"].contains(&name)) { continue; }
        let path = Path::new(name);
        if !path.is_file() { continue; }
        let Ok(mut text) = fs::read_to_string(path) else { continue };
        let original = text.clone();
        for row in &rows {
            let old = row.source.file_name().unwrap().to_str().unwrap();
            let new = row.destination.file_name().unwrap().to_str().unwrap();
            for prefix in ["/", "\"", "'"] { text = text.replace(&format!("{prefix}{old}"), &format!("{prefix}{new}")); }
        }
        for token in ["{name}", "{family}", "{}"] {
            text = text.replace(&format!("tools/port/{token}.toml"), &format!("tools/port/{token}.smf"));
        }
        text = text.replace(".scratch/translation-native/plm-instructions/manifest.toml", ".scratch/translation-native/plm-instructions/manifest.smf");
        if text != original { fs::write(path, text)?; }
    }
    fs::copy(staging.join("migration.json"), ".scratch/fixture-format-performance/results-production-migration.json")?;
    println!("{} families; {} -> {} bytes; {} cases ({} continuations, {} captured); {} writes -> {} spans", rows.len(),
        rows.iter().map(|r| r.source_bytes).sum::<u64>(), rows.iter().map(|r| r.binary_bytes).sum::<u64>(),
        rows.iter().map(|r| r.cases).sum::<usize>(), rows.iter().map(|r| r.continuations).sum::<usize>(),
        rows.iter().map(|r| r.captured).sum::<usize>(), rows.iter().map(|r| r.original_writes).sum::<usize>(),
        rows.iter().map(|r| r.binary_spans).sum::<usize>());
    for row in &rows {
        if row.load_ms.is_empty() { continue; }
        let mut times = row.load_ms.clone(); times.sort_by(f64::total_cmp);
        println!("{}: {} bytes, {:.3} ms median authenticated loading", row.family, row.binary_bytes, times[times.len()/2]);
    }
    Ok(())
}
