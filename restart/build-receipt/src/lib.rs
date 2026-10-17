//! A prospective local Cargo build transaction with immutable, bounded evidence.
use sha2::{Digest, Sha256};
use sm_binary_evidence::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
mod controls;
pub use controls::focused_controls;
mod reader;
pub use reader::authenticate;
mod field_controls;
mod host;
mod request;
pub use request::{decode_request, encode_request, read_request};
mod diagnostics;
pub use diagnostics::PhaseObservation;
use diagnostics::{Measurements, RetentionMeasurements};

pub type Result<T> = std::result::Result<T, String>;
const MAX_FILES: usize = 20_000;
const MAX_VISITS: usize = 40_000;
const MAX_DEPTH: usize = 64;
const MAX_TOTAL_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const SHARD_FILES: usize = 256;

/// A declared source/configuration/tool input tree. Exclusions are absolute
/// directories under this root, for example mutable Cargo target directories.
#[derive(Clone, Debug)]
pub struct InputRoot {
    pub path: PathBuf,
    pub excluded: Vec<PathBuf>,
}

/// Complete caller-declared transaction scope. Environment is the entire child
/// environment, not overrides over an unknown inherited process environment.
#[derive(Clone, Debug)]
pub struct BuildRequest {
    pub checkout: PathBuf,
    pub output: PathBuf,
    pub cargo: PathBuf,
    pub environment: BTreeMap<OsString, OsString>,
    pub inputs: Vec<InputRoot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Identity {
    size: u64,
    sha256: String,
}

/// Successful transaction result. No receipt is returned for a failed build,
/// input drift, omitted dependency, failed retention or malformed dep-info.
#[derive(Clone, Debug)]
pub struct BuildReceipt {
    pub receipt: PathBuf,
    pub dll: PathBuf,
    pub dll_sha256: String,
}

fn error(path: &Path, e: impl std::fmt::Display) -> String {
    format!("{}: {e}", path.display())
}

fn absolute(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err(error(path, "absolute path required"));
    }
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir))
        || path.to_str().is_some_and(|text| text.split(['/', '\\']).any(|part| part == ".."))
    {
        return Err(error(path, "parent components are forbidden"));
    }
    path.to_str().ok_or_else(|| error(path, "UTF-8 path required"))?;
    if path.as_os_str().len() > 32768 {
        return Err(error(path, "path length budget exceeded"));
    }
    Ok(())
}

fn inspect(path: &Path) -> Result<Identity> {
    let mut file = File::open(path).map_err(|e| error(path, e))?;
    let before = file.metadata().map_err(|e| error(path, e))?;
    if !before.is_file() {
        return Err(error(path, "regular file required"));
    }
    if before.len() > MAX_TOTAL_BYTES {
        return Err(error(path, "file byte budget exceeded"));
    }
    let mut hash = Sha256::new();
    let mut size = 0_u64;
    let mut buf = [0; 65_536];
    loop {
        let n = file.read(&mut buf).map_err(|e| error(path, e))?;
        if n == 0 {
            break;
        }
        size = size.checked_add(n as u64).ok_or("file extent overflow")?;
        if size > MAX_TOTAL_BYTES {
            return Err(error(path, "file byte budget exceeded"));
        }
        hash.update(&buf[..n]);
    }
    if size != before.len() || file.metadata().map_err(|e| error(path, e))?.len() != size {
        return Err(error(path, "file extent changed while reading"));
    }
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let sha256 = hash
        .finalize()
        .iter()
        .flat_map(|byte| [char::from(HEX[usize::from(*byte >> 4)]), char::from(HEX[usize::from(*byte & 15)])])
        .collect();
    Ok(Identity { size, sha256 })
}

fn walk(root: &InputRoot, path: &Path, depth: usize, files: &mut BTreeSet<PathBuf>, visits: &mut usize) -> Result<()> {
    *visits += 1;
    if *visits > MAX_VISITS {
        return Err("aggregate directory entry budget exceeded".into());
    }
    if root.excluded.iter().any(|skip| path.starts_with(skip)) {
        return Ok(());
    }
    if depth > MAX_DEPTH {
        return Err(error(path, "directory depth budget exceeded"));
    }
    let metadata = fs::symlink_metadata(path).map_err(|e| error(path, e))?;
    if reparse(&metadata) {
        return Err(error(path, "symlink/reparse input is unsupported"));
    }
    if metadata.is_file() {
        files.insert(path.to_path_buf());
        if files.len() > MAX_FILES {
            return Err("aggregate file budget exceeded".into());
        }
    } else if metadata.is_dir() {
        // Stream entries: never allocate a directory-sized intermediate list.
        for entry in fs::read_dir(path).map_err(|e| error(path, e))? {
            let entry = entry.map_err(|e| error(path, e))?;
            walk(root, &entry.path(), depth + 1, files, visits)?;
        }
    } else {
        return Err(error(path, "nonregular input is unsupported"));
    }
    Ok(())
}

fn membership(roots: &[InputRoot]) -> Result<BTreeSet<PathBuf>> {
    let mut files = BTreeSet::new();
    let mut visits = 0;
    for root in roots {
        walk(root, &root.path, 0, &mut files, &mut visits)?;
    }
    if files.is_empty() {
        return Err("input inventory is empty".into());
    }
    Ok(files)
}

fn reparse(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn canonical(path: &Path) -> Result<PathBuf> {
    absolute(path)?;
    fs::canonicalize(path).map_err(|e| error(path, e))
}

// GCC and ld response-file readers reject Windows verbatim paths. Preserve
// canonical identities in receipts; use the equivalent absolute spelling only
// at the external Cargo target-directory boundary.
fn cargo_path(path: &Path) -> Result<String> {
    absolute(path)?;
    let text = path.to_str().ok_or("non-UTF8 Cargo path")?;
    #[cfg(windows)]
    {
        if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
            return Ok(format!(r"\\{unc}"));
        }
        if let Some(drive) = text.strip_prefix(r"\\?\") {
            let bytes = drive.as_bytes();
            if bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'\\' {
                return Ok(drive.to_owned());
            }
            return Err("unsupported verbatim Cargo path".into());
        }
    }
    Ok(text.to_owned())
}

fn normalized(request: &BuildRequest) -> Result<BuildRequest> {
    if request.inputs.is_empty()
        || request.inputs.len() > 64
        || request.inputs.iter().any(|root| root.excluded.len() > 64)
    {
        return Err("input root/exclusion budget exceeded".into());
    }
    let mut result = request.clone();
    result.checkout = canonical(&request.checkout)?;
    result.cargo = canonical(&request.cargo)?;
    absolute(&request.output)?;
    if request.output.exists() {
        return Err("build receipt output must be fresh".into());
    }
    let parent = request.output.parent().ok_or("output parent missing")?;
    result.output = canonical(parent)?.join(request.output.file_name().ok_or("output name missing")?);
    for root in &mut result.inputs {
        root.path = canonical(&root.path)?;
        for skip in &mut root.excluded {
            *skip = canonical(skip)?;
        }
    }
    Ok(result)
}

fn unchanged(expected: &BTreeMap<PathBuf, Identity>) -> Result<()> {
    unchanged_observed(expected, "unchanged", &mut Measurements::disabled())
}

fn unchanged_observed(
    expected: &BTreeMap<PathBuf, Identity>,
    phase: &'static str,
    measurements: &mut Measurements<'_>,
) -> Result<()> {
    let start = measurements.start();
    let mut count = 0_u64;
    let mut bytes = 0_u64;
    let result = (|| -> Result<()> {
        for (path, identity) in expected {
            let inspected = inspect(path);
            if start.is_some() {
                count = count.checked_add(1).ok_or("measurement count overflow")?;
                bytes = bytes
                    .checked_add(inspected.as_ref().map_or(0, |identity| identity.size))
                    .ok_or("measurement byte overflow")?;
            }
            if inspected? != *identity {
                return Err(error(path, "retained file changed"));
            }
        }
        Ok(())
    })();
    let reported = measurements.finish(phase, start, count, bytes, result.is_ok());
    combine_observation(result, reported)
}

fn inventory(roots: &[InputRoot]) -> Result<BTreeMap<PathBuf, Identity>> {
    inventory_observed(roots, ["inventory_membership", "inventory_hash"], &mut Measurements::disabled())
}

fn inventory_observed(
    roots: &[InputRoot],
    phases: [&'static str; 2],
    measurements: &mut Measurements<'_>,
) -> Result<BTreeMap<PathBuf, Identity>> {
    let start = measurements.start();
    let membership = membership(roots);
    let count = if start.is_some() { membership.as_ref().map_or(0, |files| files.len() as u64) } else { 0 };
    let reported = measurements.finish(phases[0], start, count, 0, membership.is_ok());
    let files = combine_observation(membership, reported)?;
    let start = measurements.start();
    let mut entries = BTreeMap::new();
    let mut total = 0_u64;
    let result = (|| -> Result<()> {
        for path in files {
            let identity = inspect(&path)?;
            total = total.checked_add(identity.size).ok_or("aggregate extent overflow")?;
            if total > MAX_TOTAL_BYTES {
                return Err("aggregate byte budget exceeded".into());
            }
            entries.insert(path, identity);
        }
        Ok(())
    })();
    let count = if start.is_some() { entries.len() as u64 } else { 0 };
    let reported = measurements.finish(phases[1], start, count, total, result.is_ok());
    combine_observation(result, reported)?;
    Ok(entries)
}

// Observer failure cannot erase the original operation diagnostic.
fn combine_observation<T>(operation: Result<T>, reported: Result<()>) -> Result<T> {
    match (operation, reported) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(observer)) => Err(format!("{error}; {observer}")),
    }
}

fn immutable_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path).map_err(|e| error(path, e))?;
    file.write_all(bytes).and_then(|()| file.sync_all()).map_err(|e| error(path, e))
}

fn evidence(path: &Path, value: Value) -> Result<Value> {
    immutable_bytes(path, &sm_binary_evidence::encode(&value)?)?;
    let identity = inspect(path)?;
    Ok(Value::Table(Map::from([
        ("path".into(), path.to_string_lossy().as_ref().into()),
        ("size".into(), (identity.size as i64).into()),
        ("sha256".into(), identity.sha256.into()),
    ])))
}

fn retain(path: &Path, destination: &Path, expected: &Identity) -> Result<()> {
    retain_observed(path, destination, expected, None)
}

fn retain_observed(
    path: &Path,
    destination: &Path,
    expected: &Identity,
    mut statistics: Option<&mut RetentionMeasurements>,
) -> Result<()> {
    let mut source = File::open(path).map_err(|e| error(path, e))?;
    let mut output =
        OpenOptions::new().write(true).create_new(true).open(destination).map_err(|e| error(destination, e))?;
    // Bound a concurrently growing source instead of unbounded io::copy.
    let start = statistics.as_ref().map(|_| Instant::now());
    let copied =
        std::io::copy(&mut (&mut source).take(expected.size + 1), &mut output).map_err(|e| error(destination, e));
    let reported = if let (Some(statistics), Some(start)) = (statistics.as_mut(), start) {
        statistics.record(0, start, copied.as_ref().copied().unwrap_or(0))
    } else {
        Ok(())
    };
    combine_observation(copied, reported)?;
    let start = statistics.as_ref().map(|_| Instant::now());
    let synced = output.sync_all().map_err(|e| error(destination, e));
    let reported = if let (Some(statistics), Some(start)) = (statistics.as_mut(), start) {
        statistics.record(1, start, 0)
    } else {
        Ok(())
    };
    combine_observation(synced, reported)?;
    let start = statistics.as_ref().map(|_| Instant::now());
    let retained = inspect(destination);
    let reported = if let (Some(statistics), Some(start)) = (statistics.as_mut(), start) {
        statistics.record(2, start, retained.as_ref().map_or(0, |identity| identity.size))
    } else {
        Ok(())
    };
    let retained = combine_observation(retained, reported)?;
    if retained != *expected {
        return Err(error(path, "input changed during snapshot retention"));
    }
    let start = statistics.as_ref().map(|_| Instant::now());
    let original = inspect(path);
    let reported = if let (Some(statistics), Some(start)) = (statistics.as_mut(), start) {
        statistics.record(3, start, original.as_ref().map_or(0, |identity| identity.size))
    } else {
        Ok(())
    };
    let original = combine_observation(original, reported)?;
    if original != *expected {
        return Err(error(path, "input changed during snapshot retention"));
    }
    Ok(())
}

fn snapshots(output: &Path, name: &str, entries: &BTreeMap<PathBuf, Identity>) -> Result<Value> {
    snapshots_observed(output, name, entries, &mut Measurements::disabled())
}

fn snapshots_observed(
    output: &Path,
    name: &str,
    entries: &BTreeMap<PathBuf, Identity>,
    measurements: &mut Measurements<'_>,
) -> Result<Value> {
    let mut statistics = measurements.start().map(|_| RetentionMeasurements::default());
    let result = (|| -> Result<Value> {
        let directory = output.join(name);
        fs::create_dir(&directory).map_err(|e| error(&directory, e))?;
        let mut records = Vec::with_capacity(SHARD_FILES);
        let mut shards = Vec::new();
        for (index, (path, identity)) in entries.iter().enumerate() {
            let snapshot = directory.join(format!("{index:05}.bin"));
            retain_observed(path, &snapshot, identity, statistics.as_mut())?;
            records.push(Value::Table(Map::from([
                ("path".into(), path.to_string_lossy().as_ref().into()),
                ("snapshot".into(), snapshot.to_string_lossy().as_ref().into()),
                ("size".into(), (identity.size as i64).into()),
                ("sha256".into(), identity.sha256.clone().into()),
            ])));
            if records.len() == SHARD_FILES || index + 1 == entries.len() {
                let shard = directory.join(format!("inventory-{:03}.sme", shards.len()));
                let start = statistics.as_ref().map(|_| Instant::now());
                let record = evidence(&shard, Value::Array(std::mem::take(&mut records)));
                let reported = if let (Some(statistics), Some(start)) = (statistics.as_mut(), start) {
                    let bytes = record
                        .as_ref()
                        .ok()
                        .and_then(|record| record.get("size"))
                        .and_then(Value::as_integer)
                        .unwrap_or(0) as u64;
                    statistics.record(4, start, bytes)
                } else {
                    Ok(())
                };
                shards.push(combine_observation(record, reported)?);
            }
        }
        Ok(Value::Array(shards))
    })();
    if let Some(statistics) = statistics {
        let phases = if name == "inputs" {
            ["input_copy", "input_sync", "input_destination_hash", "input_source_hash", "input_shard_metadata"]
        } else {
            [
                "generated_copy",
                "generated_sync",
                "generated_destination_hash",
                "generated_source_hash",
                "generated_shard_metadata",
            ]
        };
        let reported = statistics.publish(measurements, phases, result.is_ok());
        return combine_observation(result, reported);
    }
    result
}

fn timestamp() -> Result<Value> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
    Ok(i64::try_from(millis).map_err(|_| "timestamp overflow")?.into())
}

fn cargo(request: &BuildRequest, argv: &[String], stdout: &Path, stderr: &Path) -> Result<std::process::ExitStatus> {
    let out = OpenOptions::new().write(true).create_new(true).open(stdout).map_err(|e| error(stdout, e))?;
    let err = OpenOptions::new().write(true).create_new(true).open(stderr).map_err(|e| error(stderr, e))?;
    Command::new(&request.cargo)
        .args(argv)
        .current_dir(&request.checkout)
        .env_clear()
        .envs(&request.environment)
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .status()
        .map_err(|e| error(&request.cargo, e))
}

/// Parse a Make dep-info first rule, including escaped spaces, backslashes,
/// continuations and dollar escaping. Targets are not counted as inputs.
pub fn dependency_paths(bytes: &[u8], dependency_base: &Path) -> Result<BTreeSet<PathBuf>> {
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("dep-info byte budget exceeded".into());
    }
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?.replace("\\\r\n", " ").replace("\\\n", " ");
    let line = text.lines().find(|line| !line.trim().is_empty()).ok_or("empty dep-info")?;
    let (_, dependencies) = line.split_once(": ").ok_or("dep-info rule separator missing")?;
    let mut paths = BTreeSet::new();
    let mut token = String::new();
    let mut chars = dependencies.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.peek().copied() {
                Some(' ' | '\\' | '#' | ':') => token.push(chars.next().ok_or("invalid escape")?),
                Some(_) => token.push('\\'),
                None => return Err("trailing dep-info escape".into()),
            },
            '$' if chars.peek() == Some(&'$') => {
                chars.next();
                token.push('$');
            }
            ' ' | '\t' => {
                add_dependency(&mut paths, &mut token, dependency_base)?;
            }
            '#' => return Err("unescaped dep-info comment".into()),
            c => token.push(c),
        }
    }
    add_dependency(&mut paths, &mut token, dependency_base)?;
    if paths.is_empty() {
        return Err("dep-info has no inputs".into());
    }
    Ok(paths)
}

fn add_dependency(paths: &mut BTreeSet<PathBuf>, token: &mut String, checkout: &Path) -> Result<()> {
    if token.is_empty() {
        return Ok(());
    }
    let path = PathBuf::from(std::mem::take(token));
    let path = if path.is_absolute() { path } else { checkout.join(path) };
    let path = fs::canonicalize(&path).map_err(|e| error(&path, e))?;
    paths.insert(path);
    if paths.len() > MAX_FILES {
        return Err("dep-info file budget exceeded".into());
    }
    Ok(())
}

/// Runs Cargo synchronously. The external supervisor must enforce the shared
/// producer slot and a deadline for this process and its complete child tree.
pub fn build(request: &BuildRequest) -> Result<BuildReceipt> {
    build_with(request, cargo)
}

/// Execute the same build with optional aggregate diagnostics.
/// The observer must not modify inputs or output, reenter the transaction, or
/// write within captured roots/receipt output. Keep its sidecar log separately
/// authenticated. Observer errors/panics are terminal before publication; no
/// observer is called after the success receipt is published. Timing excludes
/// observer calls. Disabled `build` does not collect diagnostic timers/counters.
pub fn build_observed(
    request: &BuildRequest,
    observer: &mut dyn FnMut(PhaseObservation) -> Result<()>,
) -> Result<BuildReceipt> {
    build_with_observed(request, cargo, &mut Measurements::enabled(observer))
}

fn build_with<F>(request: &BuildRequest, run: F) -> Result<BuildReceipt>
where
    F: FnMut(&BuildRequest, &[String], &Path, &Path) -> Result<std::process::ExitStatus>,
{
    build_with_observed(request, run, &mut Measurements::disabled())
}

fn build_with_observed<F>(
    request: &BuildRequest,
    mut run: F,
    measurements: &mut Measurements<'_>,
) -> Result<BuildReceipt>
where
    F: FnMut(&BuildRequest, &[String], &Path, &Path) -> Result<std::process::ExitStatus>,
{
    let request = measurements.measure("normalization_admission", || {
        let request = normalized(request)?;
        validate(&request)?;
        Ok(request)
    })?;
    let before = inventory_observed(&request.inputs, ["initial_membership", "initial_hash"], measurements)?;
    if !before.contains_key(&request.cargo) {
        return Err("Cargo executable is absent from frozen inputs".into());
    }
    let manifest = canonical(&request.checkout.join("restart/Cargo.toml"))?;
    let lock = canonical(&request.checkout.join("restart/Cargo.lock"))?;
    if !before.contains_key(&manifest) || !before.contains_key(&lock) {
        return Err("restart manifest/lock missing from input scope".into());
    }
    fs::create_dir(&request.output).map_err(|e| error(&request.output, e))?;
    let source = snapshots_observed(&request.output, "inputs", &before, measurements)?;
    let frozen_snapshots = inventory_observed(
        &[InputRoot { path: request.output.join("inputs"), excluded: Vec::new() }],
        ["frozen_snapshots_membership", "frozen_snapshots_hash"],
        measurements,
    )?;
    let graph_args = vec![
        "tree",
        "--manifest-path",
        "restart/Cargo.toml",
        "-p",
        "sm-restart-dll-host",
        "--features",
        "private-checks,native-preparation",
        "--target",
        "x86_64-pc-windows-gnu",
        "--locked",
        "--offline",
        "--edges",
        "normal,build",
        "--prefix",
        "none",
        "--format",
        "{p} features={f}",
    ];
    let graph_args: Vec<String> = graph_args.into_iter().map(String::from).collect();
    let graph_out = request.output.join("graph.stdout");
    let graph_err = request.output.join("graph.stderr");
    measurements.measure("cargo_graph", || {
        if !run(&request, &graph_args, &graph_out, &graph_err)?.success() {
            return Err("Cargo graph inspection failed; diagnostics retained".into());
        }
        Ok(())
    })?;
    let environment = request
        .environment
        .iter()
        .map(|(key, value)| {
            Ok((
                key.to_str().ok_or("non-UTF8 environment key")?.into(),
                value.to_str().ok_or("non-UTF8 environment value")?.into(),
            ))
        })
        .collect::<Result<Map>>()?;
    let target = request.output.join("target");
    let mut argv: Vec<String> = [
        "rustc",
        "--manifest-path",
        "restart/Cargo.toml",
        "-p",
        "sm-restart-dll-host",
        "--features",
        "private-checks,native-preparation",
        "--target",
        "x86_64-pc-windows-gnu",
        "--locked",
        "--offline",
        "--jobs",
        "1",
        "--target-dir",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    argv.push(cargo_path(&target)?);
    argv.extend(["--", "-C", "link-self-contained=yes"].into_iter().map(String::from));
    let pre = evidence(
        &request.output.join("pre.sme"),
        Value::Table(Map::from([
            ("schema".into(), "sm-build-inputs-v1".into()),
            ("inputs".into(), source),
            ("environment".into(), Value::Table(environment)),
            ("argv".into(), argv.clone().into()),
            ("cargo".into(), request.cargo.to_string_lossy().as_ref().into()),
            ("cwd".into(), request.checkout.to_string_lossy().as_ref().into()),
            ("graph".into(), file_reference(&graph_out)?),
            ("graph_stderr".into(), file_reference(&graph_err)?),
            ("graph_argv".into(), graph_args.into()),
            ("target_initially_absent".into(), (!target.exists()).into()),
            ("started_unix_ms".into(), timestamp()?),
        ])),
    )?;
    if inventory_observed(&request.inputs, ["prebuild_membership", "prebuild_hash"], measurements)? != before {
        return Err("inputs changed before build".into());
    }
    let started = timestamp()?;
    let timer = Instant::now();
    let observer_started = measurements.observer_elapsed();
    let stdout = request.output.join("build.stdout");
    let stderr = request.output.join("build.stderr");
    if target.exists() {
        return Err("fresh target appeared before build".into());
    }
    let frozen_output = inventory_observed(
        &[InputRoot { path: request.output.clone(), excluded: Vec::new() }],
        ["frozen_output_membership", "frozen_output_hash"],
        measurements,
    )?;
    let cargo_timer = measurements.start();
    let executed = run(&request, &argv, &stdout, &stderr);
    let cargo_elapsed = cargo_timer.map(|timer| timer.elapsed());
    let status = match executed {
        Ok(status) => status,
        Err(error) => {
            let reported = if let Some(elapsed) = cargo_elapsed {
                measurements.emit("cargo_build", elapsed, 1, 0, false)
            } else {
                Ok(())
            };
            return combine_observation(Err(error), reported);
        }
    };
    let observation = evidence(
        &request.output.join("build.sme"),
        Value::Table(Map::from([
            ("schema".into(), "sm-build-observation-v1".into()),
            ("started_unix_ms".into(), started),
            ("ended_unix_ms".into(), timestamp()?),
            (
                "elapsed_seconds".into(),
                timer.elapsed().saturating_sub(measurements.observer_elapsed() - observer_started).as_secs_f64().into(),
            ),
            ("success".into(), status.success().into()),
            ("exit_code".into(), status.code().map(i64::from).map(Value::from).into_iter().collect::<Vec<_>>().into()),
            ("stdout".into(), file_reference(&stdout)?),
            ("stderr".into(), file_reference(&stderr)?),
        ])),
    )?;
    let result =
        if status.success() { Ok(()) } else { Err("Cargo build failed; observation and diagnostics retained".into()) };
    let reported = if let Some(elapsed) = cargo_elapsed {
        measurements.emit("cargo_build", elapsed, 1, 0, status.success())
    } else {
        Ok(())
    };
    combine_observation(result, reported)?;
    let build_record = request.output.join("build.sme");
    let observed_files = [&stdout, &stderr, &build_record]
        .into_iter()
        .map(|path| Ok((path.clone(), inspect(path)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    if inventory_observed(&request.inputs, ["postbuild_membership", "postbuild_hash"], measurements)? != before {
        return Err("build input membership or content changed".into());
    }
    let mut known = BTreeSet::new();
    for path in before.keys() {
        known.insert(fs::canonicalize(path).map_err(|e| error(path, e))?);
    }
    let generated = inventory_observed(
        &[InputRoot { path: target.clone(), excluded: Vec::new() }],
        ["generated_membership", "generated_hash"],
        measurements,
    )?;
    for path in generated.keys() {
        known.insert(fs::canonicalize(path).map_err(|e| error(path, e))?);
    }
    let generated_records = snapshots_observed(&request.output, "generated", &generated, measurements)?;
    let retained_generated = inventory_observed(
        &[InputRoot { path: request.output.join("generated"), excluded: Vec::new() }],
        ["retained_generated_membership", "retained_generated_hash"],
        measurements,
    )?;
    let dep_timer = measurements.start();
    let dep_result = (|| -> Result<_> {
        let mut dependencies = Vec::new();
        let mut retained_dependencies = BTreeMap::new();
        let dependency_info: Vec<_> =
            generated.keys().filter(|path| path.extension().is_some_and(|ext| ext == "d")).collect();
        if dependency_info.is_empty() || dependency_info.len() > 256 {
            return Err("produced dep-info count must be 1..256".into());
        }
        let produced_dll = target.join("x86_64-pc-windows-gnu/debug/sm_restart_dll_host.dll");
        let final_dep_info = produced_dll.with_extension("d");
        if !generated.contains_key(&final_dep_info) {
            return Err("final DLL dep-info missing".into());
        }
        for (index, path) in dependency_info.iter().enumerate() {
            let bytes = sm_binary_evidence::read_bytes(path)?;
            let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
            let first = text.lines().find(|line| !line.trim().is_empty()).ok_or("empty dep-info")?;
            let (targets, _) = first.split_once(": ").ok_or("dep-info target separator missing")?;
            let target_paths = dependency_paths(format!("rule: {targets}").as_bytes(), &request.checkout)?;
            if target_paths.iter().any(|path| !path.starts_with(&target)) {
                return Err("dep-info target outside fresh target directory".into());
            }
            if **path == final_dep_info && !target_paths.contains(&canonical(&produced_dll)?) {
                return Err("final dep-info does not identify produced DLL".into());
            }
            // Cargo's workspace build emits relative local source paths from the
            // manifest workspace root, independently of the cargo process cwd.
            for input in dependency_paths(&bytes, manifest.parent().ok_or("manifest parent missing")?)? {
                if !known.contains(&input) {
                    return Err(error(&input, "compiled dep-info input omitted from inventory"));
                }
            }
            let retained = request.output.join(format!("dep-info-{index:03}.d"));
            immutable_bytes(&retained, &bytes)?;
            dependencies.push(file_reference(&retained)?);
            retained_dependencies.insert(retained.clone(), inspect(&retained)?);
        }
        Ok((dependencies, retained_dependencies, produced_dll))
    })();
    let reported = measurements.finish("dependency_info", dep_timer, 1, 0, dep_result.is_ok());
    let (dependencies, retained_dependencies, produced_dll) = combine_observation(dep_result, reported)?;
    let dll_identity = measurements.measure("dll_hash", || inspect(&produced_dll))?;
    let dll = request.output.join(format!("{}.dll", dll_identity.sha256));
    measurements.measure("dll_retention", || retain(&produced_dll, &dll, &dll_identity))?;
    unchanged_observed(&frozen_output, "final_frozen_output", measurements)?;
    unchanged_observed(&frozen_snapshots, "final_frozen_snapshots", measurements)?;
    unchanged_observed(&observed_files, "final_observed_files", measurements)?;
    unchanged_observed(&retained_dependencies, "final_retained_dependencies", measurements)?;
    if inventory_observed(&request.inputs, ["final_inputs_membership", "final_inputs_hash"], measurements)? != before {
        return Err("inputs changed during postbuild retention".into());
    }
    if inventory_observed(
        &[InputRoot { path: target, excluded: Vec::new() }],
        ["final_generated_membership", "final_generated_hash"],
        measurements,
    )? != generated
    {
        return Err("generated outputs changed during retention".into());
    }
    if inventory_observed(
        &[InputRoot { path: request.output.join("generated"), excluded: Vec::new() }],
        ["final_retained_membership", "final_retained_hash"],
        measurements,
    )? != retained_generated
    {
        return Err("retained generated snapshots changed".into());
    }
    let receipt = request.output.join("receipt.sme");
    let receipt_value = Value::Table(Map::from([
        ("schema".into(), "sm-build-receipt-v1".into()),
        ("pre".into(), pre),
        ("build".into(), observation),
        ("generated".into(), generated_records),
        ("dependency_info".into(), dependencies.into()),
        ("dll".into(), file_reference(&dll)?),
        ("inputs_unchanged".into(), true.into()),
        (
            "closure_scope".into(),
            "declared roots and all fresh-target dep-info; independent external-input completeness review required"
                .into(),
        ),
    ]));
    // Authenticate the identities written at creation, including original
    // generated/input shard records, before publishing the success pointer.
    let pending = request.output.join("receipt-pending.sme");
    measurements.measure("pending_authentication", || {
        evidence(&pending, receipt_value.clone())?;
        authenticate(&pending)
    })?;
    measurements.measure("publication_ready", || Ok(()))?;
    evidence(&receipt, receipt_value)?;
    Ok(BuildReceipt { receipt, dll, dll_sha256: dll_identity.sha256 })
}

fn file_reference(path: &Path) -> Result<Value> {
    let identity = inspect(path)?;
    Ok(Value::Table(Map::from([
        ("path".into(), path.to_string_lossy().as_ref().into()),
        ("size".into(), (identity.size as i64).into()),
        ("sha256".into(), identity.sha256.into()),
    ])))
}

fn validate(request: &BuildRequest) -> Result<()> {
    for path in [&request.checkout, &request.output, &request.cargo] {
        absolute(path)?;
    }
    if request.output.exists() {
        return Err("build receipt output must be fresh".into());
    }
    if request.inputs.is_empty() || request.inputs.len() > 64 {
        return Err("input root count must be 1..64".into());
    }
    for (index, root) in request.inputs.iter().enumerate() {
        absolute(&root.path)?;
        if request.output.starts_with(&root.path) || root.path.starts_with(&request.output) {
            return Err("receipt output overlaps a captured root".into());
        }
        for other in &request.inputs[..index] {
            if root.path.starts_with(&other.path) || other.path.starts_with(&root.path) {
                return Err("captured roots overlap".into());
            }
        }
        if root.excluded.len() > 64 {
            return Err("exclusion count budget exceeded".into());
        }
        for (index, skip) in root.excluded.iter().enumerate() {
            absolute(skip)?;
            if skip == &root.path || !skip.starts_with(&root.path) {
                return Err("exclusion must be below its root".into());
            }
            if root.excluded[..index].iter().any(|other| skip.starts_with(other) || other.starts_with(skip)) {
                return Err("exclusions overlap".into());
            }
        }
    }
    validate_environment(request.environment.iter().map(|(key, value)| {
        Ok((key.to_str().ok_or("non-UTF8 environment key")?, value.to_str().ok_or("non-UTF8 environment value")?))
    }))
}

fn validate_environment<'a>(entries: impl IntoIterator<Item = Result<(&'a str, &'a str)>>) -> Result<()> {
    let mut environment_names = BTreeSet::new();
    let mut environment_bytes = 0_usize;
    for (index, entry) in entries.into_iter().enumerate() {
        if index >= 256 {
            return Err("environment entry budget exceeded".into());
        }
        let (key, value) = entry?;
        if key.is_empty() || key.contains(['=', '\0']) || value.contains('\0') || key.len() + value.len() > 32768 {
            return Err("invalid environment entry".into());
        }
        if !environment_names.insert(key.to_ascii_uppercase()) {
            return Err("ambiguous Windows environment keys".into());
        }
        environment_bytes =
            environment_bytes.checked_add(key.len() + value.len()).ok_or("environment extent overflow")?;
        if environment_bytes > 1024 * 1024 {
            return Err("aggregate environment budget exceeded".into());
        }
    }
    Ok(())
}
