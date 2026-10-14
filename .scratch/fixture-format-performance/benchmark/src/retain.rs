//! Keep exact study sources/readers, then seal retained files after scoped cleanup.
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::{Path, PathBuf}};

fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
fn save(path: &Path, bytes: &[u8]) -> Result<()> {
    if path.exists() { ensure!(fs::read(path)? == bytes, "retained file changed: {}", path.display()); }
    else { fs::create_dir_all(path.parent().context("parent missing")?)?; fs::write(path, bytes)?; }
    Ok(())
}
fn files(root: &Path, output: &mut BTreeSet<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        ensure!(!ty.is_symlink(), "unexpected retained symlink");
        if ty.is_dir() { files(&entry.path(), output)?; }
        else if ty.is_file() { output.insert(entry.path()); }
    }
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(args.len() == 4, "usage: retain-fixture-evidence copy|seal CHECKOUT PRIVATE-TEMP RETAINED");
    let checkout = Path::new(&args[1]); let temporary = Path::new(&args[2]); let retained = Path::new(&args[3]);
    ensure!([checkout, temporary, retained].iter().all(|p| p.is_absolute()), "absolute paths required");
    let effort = checkout.join(".scratch/fixture-format-performance");
    if args[0] == "copy" {
        let mut sources = BTreeSet::new();
        files(&effort.join("benchmark"), &mut sources)?;
        for name in ["family_model.rs", "fixture_format.rs", "fixture_writes.rs", "fixture_cli.rs"] {
            sources.insert(checkout.join("crates/pipeline/src").join(name));
        }
        for source in sources { save(&retained.join("source-final").join(source.strip_prefix(checkout)?), &fs::read(source)?)?; }
        let mut relocated = Vec::new();
        for line in fs::read_to_string(effort.join("retained.sha256"))?.lines() {
            let (expected, original) = line.split_once("  ").context("checksum row malformed")?;
            let source = Path::new(original);
            let bytes = if source == checkout.join("crates/pipeline/src/family_model.rs") {
                fs::read(effort.join("benchmark/src/original_family_model.rs"))?
            } else if source.ends_with("benchmark/src/main.rs") || source.ends_with("benchmark/src/first_study.rs") {
                fs::read_to_string(source)?.replace("original_family_model.rs", "../../../../crates/pipeline/src/family_model.rs").into_bytes()
            } else if source.ends_with("benchmark/Cargo.toml") {
                let text = fs::read_to_string(source)?;
                format!("{}{}", text.split_once("[[bin]]").context("bin section missing")?.0,
                    &text[text.find("[dependencies]").context("dependencies missing")?..]).into_bytes()
            } else { fs::read(source)? };
            ensure!(hash(&bytes) == expected, "original study checksum differs: {}", source.display());
            let destination = if source.extension().is_some_and(|e| e == "rs") || source.ends_with("Cargo.toml") || source.ends_with("Cargo.lock") {
                let destination = retained.join("source-original").join(source.strip_prefix(checkout)?);
                save(&destination, &bytes)?;
                destination
            } else { source.to_path_buf() };
            relocated.push(serde_json::json!({"original_path": source, "retained_path": destination, "sha256": expected}));
        }
        save(&retained.join("original-study-relocations.json"), &serde_json::to_vec_pretty(&relocated)?)?;
        for (source, name) in [
            (temporary.join("target/debug/sm-pipeline"), "sm-pipeline"),
            (temporary.join("study-target/release/production-migrate"), "production-migrate"),
            (temporary.join("study-target/release/final-audit"), "final-audit"),
            (std::env::current_exe()?, "retain-fixture-evidence"),
        ] {
            let target = retained.join("bin").join(name);
            save(&target, &fs::read(&source)?)?;
            fs::set_permissions(&target, fs::metadata(source)?.permissions())?;
        }
        println!("Exact original study sources matched their pre-migration checksums; final source snapshots and readers retained.");
    } else {
        ensure!(args[0] == "seal", "unknown retention operation");
        let mut retained_files = BTreeSet::new();
        files(&checkout.join("out/fixture-format-performance"), &mut retained_files)?;
        files(&effort.join("benchmark"), &mut retained_files)?;
        for entry in fs::read_dir(&effort)? {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == "json" || e == "log") && path.file_name().is_none_or(|n| n != "retention-seal.log") {
                retained_files.insert(path);
            }
        }
        let mut seals = String::new();
        for path in &retained_files { seals.push_str(&format!("{}  {}\n", hash(&fs::read(path)?), path.display())); }
        fs::write(effort.join("retained-final.sha256"), seals)?;
        println!("Sealed {} retained files.", retained_files.len());
    }
    Ok(())
}
