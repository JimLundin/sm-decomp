//! Infrastructure claims through the maintained native assignment engine.
use std::{fs, io::{self, Read}, path::Path};
use sha2::{Digest, Sha256};
#[path = "../../crates/pipeline/src/assignment.rs"]
mod assignment;
fn digest(path: &Path) -> anyhow::Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
fn main() -> anyhow::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let request: assignment::Request = toml::from_str(&input)?;
    println!("{}", toml::to_string_pretty(&assignment::operate(request)?)?);
    Ok(())
}
