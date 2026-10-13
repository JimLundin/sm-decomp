#[path = "../../snapshots/pipeline/family_model.rs"]
mod family_model;
#[path = "../../snapshots/pipeline/ridley_subsystem_captured.rs"]
mod captured;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

fn digest(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

fn atomic_toml<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let temporary = path.with_extension("toml.tmp");
    fs::write(&temporary, toml::to_string_pretty(value)?)?;
    fs::rename(temporary, path)?;
    Ok(())
}

mod ridley_subsystem_inventory {
    use super::*;
    pub fn authenticate(checkout: &Path, verdict: &Path) -> Result<()> {
        let executable = std::env::var("RIDLEY_FROZEN_PIPELINE").context("frozen pipeline path missing")?;
        let expected = std::env::var("RIDLEY_FROZEN_SHA256").context("frozen pipeline digest missing")?;
        ensure!(digest(Path::new(&executable))? == expected, "frozen executable changed");
        let output = Command::new(&executable)
            .arg("ridley-subsystem-authenticate")
            .arg(verdict)
            .current_dir(checkout)
            .output()?;
        let receipt = Path::new(env!("CARGO_MANIFEST_DIR")).join("../inventory-authentication.toml");
        fs::write(&receipt, &output.stdout)?;
        fs::write(receipt.with_extension("stderr"), &output.stderr)?;
        ensure!(output.status.success(), "native inventory authentication failed: {}", String::from_utf8_lossy(&output.stderr));
        let value: toml::Value = toml::from_str(std::str::from_utf8(&output.stdout)?)?;
        ensure!(value["status"].as_str() == Some("authenticated"), "native authentication receipt status differs");
        ensure!(value["accepted"].as_bool() == Some(false) && value["live_exercised"].as_bool() == Some(false), "inventory receipt gained acceptance");
        ensure!(digest(Path::new(&executable))? == expected, "frozen executable changed during authentication");
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    ensure!(args.len() == 4, "usage: ridley-natural-producer CHECKOUT INVENTORY_VERDICT OUTPUT");
    let mut manifest = captured::generate(Path::new(&args[1]), Path::new(&args[2]), Path::new(&args[3]))?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().context("producer parent missing")?;
    for name in ["producer/src/main.rs", "snapshots/pipeline/family_model.rs", "snapshots/pipeline/ridley_subsystem_captured.rs"] {
        manifest.rust_dependencies.push(fs::canonicalize(root.join(name))?);
    }
    atomic_toml(Path::new(&args[3]), &manifest)?;
    println!("{} independent natural fixture pairs across {} entries; {} capture dependencies", manifest.routines.iter().map(|r|r.cases.len()).sum::<usize>(), manifest.routines.len(), manifest.rust_dependencies.len());
    Ok(())
}
