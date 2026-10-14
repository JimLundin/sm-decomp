//! Read-only phase timing for a retained complete native proof.
//! File timestamp spans include publication and are not kernel CPU timings.
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn modified(path: &Path) -> Result<SystemTime> {
    Ok(path.metadata()?.modified()?)
}
fn span(first: SystemTime, last: SystemTime) -> Option<f64> {
    last.duration_since(first).ok().map(|value| value.as_secs_f64())
}
fn invocations(root: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            invocations(&path, result)?;
        } else if path.to_string_lossy().ends_with(".invocation.toml") {
            result.push(path);
        }
    }
    Ok(())
}
fn field<'a>(text: &'a str, name: &str) -> Result<&'a str> {
    let prefix = format!("{name} = ");
    let values: Vec<_> = text.lines().filter_map(|line| line.strip_prefix(&prefix)).collect();
    if values.len() != 1 {
        return Err("expected one generated flat field".into());
    }
    Ok(values[0])
}
fn main() -> Result<()> {
    let root = PathBuf::from(std::env::args_os().nth(1).ok_or("expected proof root")?);
    let family = root.join("run/family");
    let mut paths = Vec::new();
    invocations(&family, &mut paths)?;
    paths.sort();
    let mut rows = Vec::new();
    let mut sums = BTreeMap::<String, f64>::new();
    let mut latest = SystemTime::UNIX_EPOCH;
    let mut earliest = None;
    for path in paths {
        let invocation = fs::read_to_string(&path)?;
        let key = field(&invocation, "operation_key")?.trim_matches('"');
        if key.len() != 64 || !key.bytes().all(|value| value.is_ascii_hexdigit()) {
            return Err("operation key is not a SHA-256 identity".into());
        }
        let operation = root.join("operations").join(key);
        let timing_path = PathBuf::from(path.to_string_lossy().replace(".invocation.toml", ".timing.toml"));
        let timing = fs::read_to_string(&timing_path)?;
        let kind = if path.parent() == Some(&family) { "compile" } else { "fixture" };
        let mut row = format!("[[producers]]\ninvocation = {:?}\nkind = {kind:?}\n", path.display().to_string());
        let seconds: f64 = field(&timing, "seconds")?.parse()?;
        row.push_str(&format!("producer_seconds = {seconds}\n"));
        *sums.entry(format!("{kind}_producer_seconds")).or_default() += seconds;
        let invocation_time = modified(&path)?;
        let intent = modified(&operation.join("intent.json"))?;
        let sandbox = modified(&operation.join("sandbox.json"))?;
        let result = modified(&operation.join("result.json"))?;
        let publication = modified(&timing_path)?;
        latest = latest.max(publication);
        earliest = Some(earliest.map_or(intent, |previous: SystemTime| previous.min(intent)));
        for (name, first, last) in [
            ("identity_import_seconds", invocation_time, intent),
            ("runner_startup_seconds", intent, sandbox),
            ("sandbox_and_finalization_seconds", sandbox, result),
            ("parent_publication_seconds", result, publication),
        ] {
            if let Some(seconds) = span(first, last) {
                row.push_str(&format!("{name} = {seconds}\n"));
                *sums.entry(format!("{kind}_{name}")).or_default() += seconds;
            } else {
                row.push_str(&format!("{name}_unavailable = true\n"));
            }
        }
        rows.push(row);
    }
    println!("operation_count = {}", rows.len());
    println!(
        "timing_contract = \"File timestamp spans; modified invocation timestamps are omitted, not interpreted as duration. Sandbox span includes result finalization, not just gameplay CPU.\""
    );
    if let Some(first) = earliest {
        println!("producer_span_seconds = {}", span(first, latest).ok_or("reversed producer span")?);
    }
    println!(
        "post_producer_finalization_seconds = {}",
        span(latest, modified(&root.join("run/native-receipt.toml"))?).ok_or("reversed receipt timestamp")?
    );
    println!("\n[totals]");
    for (name, seconds) in sums {
        println!("{name} = {seconds}");
    }
    for row in rows {
        println!("\n{row}");
    }
    Ok(())
}
