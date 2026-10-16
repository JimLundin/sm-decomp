//! Read-only whole-batch planning probes. No verification producers are launched.
use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File},
    path::Path,
    process::Command,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn run(root: &Path, label: &str, binary: &Path, args: &[&str], expected: i32) -> Result<()> {
    let output = File::create(root.join(format!("{label}.stdout.toml")))?;
    let error = File::create(root.join(format!("{label}.stderr.txt")))?;
    let timing = root.join(format!("{label}.timing.txt"));
    let status = Command::new("/usr/bin/time")
        .args(["-f", "wall_seconds=%e user_seconds=%U system_seconds=%S peak_rss_kib=%M", "-o"])
        .arg(&timing)
        .args(["timeout", "--kill-after=3s", "45s"])
        .arg(binary)
        .args(args)
        .stdout(output)
        .stderr(error)
        .status()?;
    println!("{label}: exit={:?} {}", status.code(), fs::read_to_string(timing)?.trim());
    if status.code() != Some(expected) {
        return Err(format!("{label}: expected exit {expected}; inspect retained diagnostics").into());
    }
    Ok(())
}

fn selection(path: &Path) -> Result<Vec<String>> {
    let document = fs::read_to_string(path)?;
    let mut section = "";
    let mut identities = Vec::new();
    let mut selected = 0usize;
    let mut excluded = 0usize;
    let mut reasons = BTreeMap::<String, usize>::new();
    for line in document.lines().map(str::trim) {
        match line {
            "[[selected]]" => { section = "selected"; selected += 1; }
            "[[excluded]]" => { section = "excluded"; excluded += 1; }
            _ => {}
        }
        if line.starts_with("family = ") && !section.is_empty() {
            identities.push(format!("{section} {line}"));
        }
        if section == "selected" && line.starts_with("detail = ") {
            *reasons.entry(line.to_owned()).or_default() += 1;
        }
    }
    if selected + excluded == 0 {
        return Err("no family decisions in planning output".into());
    }
    println!("{}: selected={selected} excluded={excluded}", path.file_name().unwrap().display());
    for (reason, count) in reasons {
        println!("  {count}: {reason}");
    }
    identities.sort();
    Ok(identities)
}

fn main() -> Result<()> {
    let root = std::env::args_os().nth(1).ok_or("usage: probe OUTPUT_ROOT")?;
    let root = fs::canonicalize(root)?;
    let original = root.join("controller-original");
    let stripped = root.join("controller-stripped");
    // Keep the set of untracked paths stable before either planning invocation.
    for label in ["startup-original", "startup-stripped", "catalog-original", "catalog-stripped", "plan-original", "plan-stripped"] {
        for suffix in ["stdout.toml", "stderr.txt", "timing.txt"] {
            File::create(root.join(format!("{label}.{suffix}")))?;
        }
    }
    run(&root, "startup-original", &original, &["__read_only_startup_probe__"], 1)?;
    run(&root, "startup-stripped", &stripped, &["__read_only_startup_probe__"], 1)?;
    run(&root, "catalog-original", &original, &["batch", "catalog"], 0)?;
    run(&root, "catalog-stripped", &stripped, &["batch", "catalog"], 0)?;
    if fs::read(root.join("catalog-original.stdout.toml"))? != fs::read(root.join("catalog-stripped.stdout.toml"))? {
        return Err("catalog decisions differ after debug-section stripping".into());
    }
    let args = ["batch", "plan", "--profile", "offline-all", "--changed-since", "HEAD~1"];
    run(&root, "plan-original", &original, &args, 0)?;
    run(&root, "plan-stripped", &stripped, &args, 0)?;
    if selection(&root.join("plan-original.stdout.toml"))? != selection(&root.join("plan-stripped.stdout.toml"))? {
        return Err("selected/excluded families differ after debug-section stripping".into());
    }
    println!("Catalog bytes and selected/excluded family identities match.");
    Ok(())
}
