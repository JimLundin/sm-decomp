//! Per-batch ownership and authenticated native family reuse across worktrees.
use crate::inputs::bounded;
use crate::{atomic_toml, cleanup, digest, family, family_model, live, live_model, plm_fixtures};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum CatalogEntry {
    Kind(String),
    Details(FamilyEntry),
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct FamilyEntry {
    pub kind: String,
    pub manifest: PathBuf,
    pub live: Option<String>,
    pub prepare: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Catalog {
    pub families: BTreeMap<String, CatalogEntry>,
    pub profiles: BTreeMap<String, Vec<String>>,
}
#[derive(Debug, Deserialize, PartialEq)]
struct Discovery {
    rust_module: Option<PathBuf>,
    routines: Option<Vec<DiscoveryRoutine>>,
    #[serde(default)]
    projectile_host: bool,
    #[serde(default)]
    projectile_rendering: bool,
}
#[derive(Debug, Deserialize, PartialEq)]
struct DiscoveryRoutine {
    #[serde(default)]
    symbol: String,
    pointer_bank: Option<u16>,
}
// Catalog discovery is metadata, not fixture admission. Project table-form
// manifests before TOML decoding so ignored multi-megabyte cases never become
// an AST. The family verifier still parses and validates the complete manifest.
// Unusual dotted discovery layouts use the original complete decoder.
fn discovery(text: &str) -> Result<Discovery> {
    let projected = discovery_projection(text)?;
    Ok(toml::from_str(projected.as_deref().unwrap_or(text))?)
}
fn discovery_projection(text: &str) -> Result<Option<String>> {
    let mut projected = String::new();
    let mut routine = false;
    let mut root = true;
    let mut start = 0;
    while start < text.len() {
        bounded::check()?;
        let end = discovery_statement_end(text.as_bytes(), start)?;
        let statement = text[start..end].trim();
        start = end;
        if statement.is_empty() || statement.starts_with('#') {
            continue;
        }
        if statement.starts_with('[') {
            let header = statement[..discovery_delimiter(statement, b'#').unwrap_or(statement.len())].trim_end();
            let array = statement.starts_with("[[");
            let closing = if array { "]]" } else { "]" };
            let Some(key) = header.strip_suffix(closing) else { return Ok(None) };
            let key = toml_edit::Key::parse(&key[if array { 2 } else { 1 }..])?;
            routine = array && key.len() == 1 && key[0].get() == "routines";
            root = false;
            if key
                .first()
                .is_some_and(|key| ["rust_module", "projectile_host", "projectile_rendering"].contains(&key.get()))
                || (!array && key.len() == 1 && key[0].get() == "routines")
                || (key.len() > 1 && key[0].get() == "routines" && ["symbol", "pointer_bank"].contains(&key[1].get()))
            {
                return Ok(None);
            }
            if routine {
                projected.push_str("[[routines]]\n");
            }
            continue;
        }
        if !root && !routine {
            continue;
        }
        let equal = discovery_delimiter(statement, b'=');
        let Some(equal) = equal else { return Ok(None) };
        let key = toml_edit::Key::parse(&statement[..equal])?;
        let Some(first) = key.first() else { return Ok(None) };
        let wanted = if root {
            if first.get() == "routines" {
                if key.len() != 1 {
                    return Ok(None);
                }
                let Some(value) = discovery_inline_routines(&statement[equal + 1..])? else { return Ok(None) };
                projected.push_str("routines = ");
                projected.push_str(&value);
                projected.push('\n');
                continue;
            }
            ["rust_module", "projectile_host", "projectile_rendering"].contains(&first.get())
        } else {
            ["symbol", "pointer_bank"].contains(&first.get())
        };
        if wanted {
            if key.len() != 1 {
                return Ok(None);
            }
            projected.push_str(statement);
            projected.push('\n');
        }
    }
    Ok(Some(projected))
}
fn discovery_inline_routines(value: &str) -> Result<Option<String>> {
    let end = discovery_extent(value.as_bytes(), 0, b"#")?;
    let value = value[..end].trim();
    let Some(values) = value.strip_prefix('[').and_then(|value| value.strip_suffix(']')) else {
        return Ok(None);
    };
    let mut projected = String::from("[");
    let mut start = 0;
    while start < values.len() {
        let end = discovery_extent(values.as_bytes(), start, b",")?;
        let value = values[start..end].trim();
        start = (end + 1).min(values.len());
        if value.is_empty() {
            continue;
        }
        let Some(fields) = value.strip_prefix('{').and_then(|value| value.strip_suffix('}')) else {
            return Ok(None);
        };
        projected.push('{');
        let mut start = 0;
        while start < fields.len() {
            let end = discovery_extent(fields.as_bytes(), start, b",")?;
            let field = fields[start..end].trim();
            start = (end + 1).min(fields.len());
            let Some(equal) = discovery_delimiter(field, b'=') else { return Ok(None) };
            let key = toml_edit::Key::parse(&field[..equal])?;
            if key.first().is_some_and(|key| ["symbol", "pointer_bank"].contains(&key.get())) {
                if key.len() != 1 {
                    return Ok(None);
                }
                if !projected.ends_with('{') {
                    projected.push(',');
                }
                projected.push_str(field);
            }
        }
        projected.push_str("},");
    }
    projected.push(']');
    Ok(Some(projected))
}
fn discovery_delimiter(text: &str, delimiter: u8) -> Option<usize> {
    let mut quote = 0;
    let mut escaped = false;
    text.bytes().position(|byte| {
        if escaped {
            escaped = false;
        } else if quote == b'"' && byte == b'\\' {
            escaped = true;
        } else if quote != 0 {
            if byte == quote {
                quote = 0;
            }
        } else if byte == b'\'' || byte == b'"' {
            quote = byte;
        } else if byte == delimiter {
            return true;
        }
        false
    })
}
// Newlines inside arrays, inline tables and multiline strings are not new
// statements. Track them even in ignored fixture payloads; apparent headers
// inside comments/strings must never change the discovery context.
fn discovery_statement_end(bytes: &[u8], start: usize) -> Result<usize> {
    let end = discovery_extent(bytes, start, b"\n")?;
    Ok((end + 1).min(bytes.len()))
}
fn discovery_extent(bytes: &[u8], start: usize, delimiters: &[u8]) -> Result<usize> {
    let mut index = start;
    let mut quote = 0;
    let mut multiline = false;
    let mut brackets = Vec::new();
    while index < bytes.len() {
        if index.is_multiple_of(64 * 1024) {
            bounded::check()?;
        }
        let byte = bytes[index];
        if quote != 0 {
            if quote == b'"' && byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == quote {
                if multiline {
                    let run = bytes[index..].iter().take_while(|byte| **byte == quote).count();
                    if run >= 3 {
                        index += run;
                        quote = 0;
                        continue;
                    }
                } else {
                    quote = 0;
                }
            } else {
                ensure!(multiline || byte != b'\n', "unterminated discovery string");
            }
        } else {
            if brackets.is_empty() && delimiters.contains(&byte) {
                return Ok(index);
            }
            match byte {
                b'\'' | b'"' => {
                    quote = byte;
                    multiline = bytes.get(index..index + 3).is_some_and(|run| run == [byte; 3]);
                    if multiline {
                        index += 3;
                        continue;
                    }
                }
                b'[' | b'{' => brackets.push(byte),
                b']' | b'}' => {
                    ensure!(
                        brackets.pop() == Some(if byte == b']' { b'[' } else { b'{' }),
                        "unbalanced discovery brackets"
                    );
                }
                b'#' => {
                    index += bytes[index..].iter().position(|byte| *byte == b'\n').unwrap_or(bytes.len() - index);
                    continue;
                }
                _ => {}
            }
        }
        index += 1;
    }
    ensure!(quote == 0 && brackets.is_empty(), "unterminated discovery value");
    Ok(bytes.len())
}
pub fn catalog(checkout: &Path) -> Result<Catalog> {
    bounded::limited(|| catalog_using(checkout, |path| bounded::text(path)))
}

pub(crate) fn catalog_until(checkout: &Path, deadline: u64, max_bytes: u64) -> Result<Catalog> {
    bounded::until(deadline, max_bytes, || catalog_using(checkout, |path| bounded::text(path)))
}

/// Keep catalog discovery identical for the supervisor's bounded source reader.
pub(crate) fn catalog_using(checkout: &Path, mut read: impl FnMut(&Path) -> Result<String>) -> Result<Catalog> {
    let mut config: Catalog = toml::from_str(&read(&checkout.join("tools/port/batch.toml"))?)?;
    let paths = bounded::entries(&checkout.join("tools/port"))?;
    ensure!(
        paths.len() <= 4096 && config.families.len() <= 4096 && config.profiles.len() <= 1024,
        "native catalog discovery inventory exceeded"
    );
    for path in paths {
        bounded::check()?;
        if path.extension().is_none_or(|v| v != "toml") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy();
        if config.families.contains_key(name.as_ref()) {
            continue;
        }
        let manifest =
            discovery(&read(&path)?).with_context(|| format!("invalid discovery manifest {}", path.display()))?;
        if manifest.rust_module.is_none() || manifest.routines.is_none() {
            continue;
        }
        let specialized = manifest.projectile_host
            || manifest.projectile_rendering
            || manifest.routines.unwrap().iter().any(|r| {
                r.pointer_bank.is_some()
                    || ["EprojInstruction", "EprojPreinstruction", "PlmInstruction", "PlmFrame", "PlmControlFrame"]
                        .contains(&r.symbol.as_str())
            });
        config.families.insert(
            name.into_owned(),
            CatalogEntry::Details(FamilyEntry {
                kind: if specialized { "dispatcher" } else { "family" }.into(),
                manifest: path.strip_prefix(checkout)?.into(),
                live: None,
                prepare: None,
            }),
        );
        ensure!(config.families.len() <= 4096, "native catalog family inventory exceeded");
    }
    config.profiles.insert("offline-all".into(), config.families.keys().cloned().collect());
    Ok(config)
}
pub(crate) fn entry(config: &Catalog, name: &str) -> Result<FamilyEntry> {
    Ok(match config.families.get(name).with_context(|| format!("unknown family: {name}"))? {
        CatalogEntry::Kind(kind) => FamilyEntry {
            kind: kind.clone(),
            manifest: format!("tools/port/{name}.toml").into(),
            live: ["family", "callback", "projectile"].contains(&kind.as_str()).then(|| kind.clone()),
            prepare: None,
        },
        CatalogEntry::Details(item) => item.clone(),
    })
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct FamilyRecord {
    pub status: String,
    pub exit_code: i32,
    pub verdict: Option<PathBuf>,
    pub manifest_sha256: Option<String>,
    pub kind: Option<String>,
    pub seconds: f64,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct ReplayRecord {
    pub status: String,
    pub exit_code: i32,
    pub verdict: Option<PathBuf>,
    pub seconds: f64,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Batch {
    pub schema_version: u32,
    pub profile: String,
    pub families: BTreeMap<String, FamilyRecord>,
    pub replays: Vec<ReplayRecord>,
    pub publication_error: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Request {
    pub checkout: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
    pub operation: String,
    pub profile: String,
    pub families: Option<Vec<String>>,
    pub batch: Option<PathBuf>,
    pub jobs: usize,
    pub case_jobs: usize,
    pub live_jobs: usize,
    pub frames: u64,
    pub accepted: Option<PathBuf>,
    pub changed_since: Option<String>,
    pub fresh: bool,
    pub native_plm: bool,
    pub room_plm: bool,
    pub rng_predecessor: Option<PathBuf>,
    pub evidence_store: Option<PathBuf>,
    pub retain_private_stores: bool,
}
impl Default for Request {
    fn default() -> Self {
        Self {
            checkout: PathBuf::new(),
            root: PathBuf::new(),
            workers: 4,
            operation: String::new(),
            profile: "gameplay".into(),
            families: None,
            batch: None,
            jobs: 3,
            case_jobs: 4,
            live_jobs: 2,
            frames: 9000,
            accepted: None,
            changed_since: None,
            fresh: false,
            native_plm: false,
            room_plm: false,
            rng_predecessor: None,
            evidence_store: None,
            retain_private_stores: false,
        }
    }
}
fn current(request: &Request, checkout: &Path, batch: &Batch, config: &Catalog, name: &str) -> Result<PathBuf> {
    let _ = request;
    let item = entry(config, name)?;
    let record =
        batch.families.get(name).with_context(|| format!("{name}: missing passing evidence; run verify first"))?;
    ensure!(record.exit_code == 0, "{name}: missing passing evidence; run verify first");
    let path = record.verdict.clone().with_context(|| format!("{name}: missing passing evidence; run verify first"))?;
    ensure!(
        record.manifest_sha256.as_deref() == Some(digest(&checkout.join(item.manifest))?.as_str()),
        "{name}: manifest changed; run verify first"
    );
    Ok(path)
}
fn replay_request(
    request: &Request,
    checkout: &Path,
    batch: &Batch,
    config: &Catalog,
    names: &[String],
    output: &Path,
) -> Result<live_model::Request> {
    let mut replay = live_model::Request {
        checkout: checkout.into(),
        root: request.root.clone(),
        workers: request.workers,
        frames: request.frames,
        live_jobs: request.live_jobs,
        output: output.into(),
        accepted: request
            .accepted
            .clone()
            .unwrap_or_else(|| checkout.join("out/port/decompressor-offline/decomp-acceptance.toml")),
        room_plm: request.room_plm,
        rng_predecessor: request.rng_predecessor.clone(),
        evidence_store: request.evidence_store.clone(),
        retain_private_stores: request.retain_private_stores,
        ..Default::default()
    };
    let mut contracts = 0;
    for name in names {
        let item = entry(config, name)?;
        if item.live.as_deref() == Some("projectile") {
            ensure!(replay.projectile_verdict.is_none(), "multiple native projectile contracts selected");
            replay.projectile_verdict = Some(current(request, checkout, batch, config, name)?);
            contracts += 1;
            continue;
        }
        let target = match item.live.as_deref() {
            Some("family") => &mut replay.family_verdicts,
            Some("callback") => &mut replay.callback_verdicts,
            Some("dispatcher") => &mut replay.dispatcher_verdicts,
            _ => continue,
        };
        target.push(current(request, checkout, batch, config, name)?);
        contracts += 1;
    }
    ensure!(contracts > 0, "no live-eligible families selected");
    if request.native_plm || request.room_plm {
        replay.plm_verdict = Some(current(request, checkout, batch, config, "plm-spawn")?);
    }
    Ok(replay)
}
fn verify_one(
    request: &Request,
    checkout: &Path,
    batch_path: &Path,
    config: &Catalog,
    name: &str,
    previous: Option<&FamilyRecord>,
) -> Result<FamilyRecord> {
    let item = entry(config, name)?;
    let kind = item.kind;
    ensure!(
        [
            "family",
            "offline-family",
            "dispatcher",
            "plm-instructions",
            "projectile",
            "offline-projectile",
            "callback",
            "plm",
            "power-bomb",
            "ridley-subsystem"
        ]
        .contains(&kind.as_str()),
        "{name}: unsupported native adapter {kind}"
    );
    ensure!(item.prepare.is_none(), "{name}: fixture preparation command unsupported");
    let mut manifest = checkout.join(item.manifest);
    let hash = digest(&manifest)?;
    if !request.fresh
        && let Some(previous) = previous
        && previous.exit_code == 0
        && previous.manifest_sha256.as_deref() == Some(&hash)
        && let Some(verdict) = &previous.verdict
        && family::authenticate(family_model::AuthenticateRequest {
            checkout: checkout.into(),
            verdict: verdict.clone(),
        })
        .is_ok()
    {
        let mut previous = previous.clone();
        previous.status = "reused".into();
        return Ok(previous);
    }
    let started = Instant::now();
    if kind == "plm-instructions" {
        let output = batch_path
            .parent()
            .unwrap()
            .join("families")
            .join(name)
            .join("fixtures")
            .join(SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos().to_string());
        manifest = plm_fixtures::generate(checkout, &output, None)?.manifest;
    }
    let verdict = family::verify(family_model::VerifyRequest {
        execution: String::new(),
        checkout: checkout.into(),
        source: checkout.join("deps/snesrev-sm"),
        root: request.root.clone(),
        workers: request.workers,
        case_jobs: request.case_jobs,
        adapter: kind.clone(),
        manifest,
        output: batch_path.parent().unwrap().join("families").join(name).join("evidence"),
    })?;
    let passed = verdict.status == "offline-verified";
    let path = verdict.run.join("verdict.toml");
    if passed {
        family::authenticate(family_model::AuthenticateRequest { checkout: checkout.into(), verdict: path.clone() })?;
    }
    Ok(FamilyRecord {
        status: verdict.status,
        exit_code: if passed { 0 } else { 1 },
        verdict: Some(path),
        manifest_sha256: Some(hash),
        kind: Some(kind),
        seconds: started.elapsed().as_secs_f64(),
        error: verdict.error,
    })
}
#[derive(Serialize)]
pub(crate) struct Receipt {
    pub status: String,
    pub batch: PathBuf,
    pub selected: Vec<String>,
    pub families: BTreeMap<String, FamilyRecord>,
    pub replays: Vec<ReplayRecord>,
    pub cleanup: Option<cleanup::CleanupReceipt>,
}
#[derive(Serialize, Deserialize)]
struct Latest {
    batch: PathBuf,
}
pub fn operate(request: Request) -> Result<toml::Value> {
    if ["status", "catalog", "plan"].contains(&request.operation.as_str()) {
        return operate_inner(request);
    }
    let destination = request
        .batch
        .as_ref()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| request.checkout.join("out/port/batches"));
    fs::create_dir_all(&destination)?;
    let mut metrics = crate::metrics::Run::start(
        destination.join(format!("batch-{}.metrics.toml", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos())),
        "batch",
    )?;
    metrics.context.configuration("workers", request.workers);
    metrics.context.configuration("case-jobs", request.case_jobs);
    let result = operate_inner(request);
    let success = result.as_ref().is_ok_and(|value| {
        value
            .get("status")
            .and_then(toml::Value::as_str)
            .is_none_or(|status| ["complete", "unchanged"].contains(&status))
    });
    metrics
        .finish(if success { "success" } else { "failed" }, result.as_ref().err().map(|error| format!("{error:#}")))?;
    result
}
fn operate_inner(request: Request) -> Result<toml::Value> {
    let checkout = fs::canonicalize(&request.checkout)?;
    let config = catalog(&checkout)?;
    let command = request.operation.as_str();
    ensure!(
        ["verify", "replay", "run", "status", "catalog", "plan"].contains(&command),
        "unsupported batch operation: {command}"
    );
    if command == "catalog" {
        return Ok(toml::Value::try_from(config)?);
    }
    let mut names = request
        .families
        .clone()
        .or_else(|| config.profiles.get(&request.profile).cloned())
        .context("unknown profile or missing families")?;
    if command == "plan" || request.changed_since.is_some() {
        let report = crate::affected::select(&checkout, &names, request.changed_since.as_deref())?;
        ensure!(
            toml::to_string(&config)? == toml::to_string(&catalog(&checkout)?)?,
            "Offline family catalog changed during selection; retry"
        );
        if command == "plan" {
            return Ok(toml::Value::try_from(report)?);
        }
        names = report.families();
        if names.is_empty() {
            return Ok(
                crate::report!({"status":"unchanged","selected":Vec::<String>::new(),"families":Vec::<String>::new()}),
            );
        }
    }
    if request.native_plm || request.room_plm {
        names.push("plm-spawn".into());
    }
    let mut seen = BTreeSet::new();
    let names = names.into_iter().filter(|n| seen.insert(n.clone())).collect::<Vec<_>>();
    for name in &names {
        ensure!(
            !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
            "unsafe family name"
        );
        entry(&config, name)?;
    }
    let selection = checkout.join("out/port/batches");
    fs::create_dir_all(&selection)?;
    let selecting = OpenOptions::new().create(true).append(true).open(selection.join("selection.lock"))?;
    selecting.lock()?;
    let latest = selection.join("latest.toml");
    let path = if let Some(path) = &request.batch {
        path.clone()
    } else if latest.exists() {
        pipeline_model::read::<Latest>(&latest)?.batch
    } else {
        ensure!(["verify", "run"].contains(&command), "no batch yet");
        selection.join(SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros().to_string()).join("batch.toml")
    };
    ensure!(!path.is_symlink(), "batch record is a symlink");
    fs::create_dir_all(path.parent().context("batch parent missing")?)?;
    let path = fs::canonicalize(path.parent().unwrap())?.join(path.file_name().context("batch filename missing")?);
    let lock = OpenOptions::new().create(true).append(true).open(path.with_extension("lock"))?;
    ensure!(lock.try_lock().is_ok(), "another command is writing this batch; retry after it finishes");
    let mut batch: Batch = if path.exists() {
        let value: Batch = pipeline_model::read(&path)?;
        ensure!(value.schema_version == 2, "unsupported batch version");
        value
    } else {
        ensure!(["verify", "run"].contains(&command), "batch does not exist");
        let value = Batch { schema_version: 2, profile: request.profile.clone(), ..Default::default() };
        atomic_toml(&path, &value)?;
        atomic_toml(&latest, &Latest { batch: path.clone() })?;
        value
    };
    drop(selecting);
    if command == "status" {
        return Ok(toml::Value::try_from(batch)?);
    }
    if ["verify", "run"].contains(&command) {
        ensure!((1..=8).contains(&request.jobs), "batch jobs must be 1..8");
        let state = Mutex::new(&mut batch);
        let next = AtomicUsize::new(0);
        let metrics = crate::metrics::active();
        std::thread::scope(|scope| {
            for _ in 0..request.jobs.min(names.len()) {
                let metrics = metrics.clone();
                let next = &next;
                let state = &state;
                let names = &names;
                let request = &request;
                let checkout = &checkout;
                let path = &path;
                let config = &config;
                scope.spawn(move || {
                    let _entered = metrics.as_ref().map(|metrics| metrics.enter());
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(name) = names.get(index) else {
                            break;
                        };
                        let previous = state.lock().unwrap().families.get(name).cloned();
                        let record = match verify_one(request, checkout, path, config, name, previous.as_ref()) {
                            Ok(record) => record,
                            Err(error) => FamilyRecord {
                                status: "failed".into(),
                                exit_code: 1,
                                error: Some(format!("{error:#}")),
                                ..Default::default()
                            },
                        };
                        eprintln!("{name}: {}", record.status);
                        let mut owner = state.lock().unwrap();
                        owner.families.insert(name.clone(), record);
                        if let Err(error) = atomic_toml(path, &**owner) {
                            owner.publication_error = Some(format!("{error:#}"));
                        }
                    }
                });
            }
        });
        ensure!(batch.publication_error.is_none(), "batch publication failed");
    }
    let verified = names.iter().all(|name| batch.families.get(name).is_some_and(|r| r.exit_code == 0));
    let mut replay_passed = true;
    if ["replay", "run"].contains(&command) && (command == "replay" || verified) {
        let started = Instant::now();
        let directory = path
            .parent()
            .unwrap()
            .join("replays")
            .join(SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos().to_string());
        let record =
            match replay_request(&request, &checkout, &batch, &config, &names, &directory).and_then(live::verify) {
                Ok(verdict) => {
                    let passed = verdict.status == "mixed-C-Rust-integration";
                    ReplayRecord {
                        status: verdict.status,
                        exit_code: if passed { 0 } else { 1 },
                        verdict: Some(verdict.run.join("verdict.toml")),
                        seconds: started.elapsed().as_secs_f64(),
                        error: verdict.error,
                    }
                }
                Err(error) => ReplayRecord {
                    status: "failed".into(),
                    exit_code: 1,
                    error: Some(format!("{error:#}")),
                    seconds: started.elapsed().as_secs_f64(),
                    ..Default::default()
                },
            };
        replay_passed = record.exit_code == 0;
        batch.replays.push(record);
        atomic_toml(&path, &batch)?;
    }
    let context = crate::metrics::active();
    let cleaning = context.as_ref().map(|context| context.stage("batch-scoped-cleanup")).transpose()?;
    let cleaned = cleanup::cleanup(cleanup::CleanupRequest {
        root: path.parent().unwrap().into(),
        apply: true,
        deduplicate: true,
    })?;
    if let Some(cleaning) = cleaning {
        cleaning.finish("success")?;
    }
    let passed = replay_passed && (command == "replay" || verified);
    Ok(toml::Value::try_from(Receipt {
        status: if passed { "complete" } else { "failed" }.into(),
        batch: path,
        selected: names,
        families: batch.families,
        replays: batch.replays,
        cleanup: Some(cleaned),
    })?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_refuses_expired_budget_and_excessive_inventory_before_manifest_reads() -> Result<()> {
        let directory = bounded::tests::Directory::new()?;
        let port = directory.0.join("tools/port");
        fs::create_dir_all(&port)?;
        let config = "[families]\n[profiles]\n";
        fs::write(port.join("batch.toml"), config)?;
        fs::write(port.join("one.toml"), "rust_module='one.rs'\n[[routines]]\nsymbol='One'\n")?;
        assert!(catalog_until(&directory.0, 0, 1024).unwrap_err().to_string().contains("deadline"));
        assert!(catalog_until(&directory.0, bounded::now_unix_ms()? + 1000, config.len() as u64).is_err());
        for number in 0..4096 {
            fs::write(port.join(format!("extra-{number}.toml")), b"")?;
        }
        let mut reads = 0usize;
        let error = catalog_using(&directory.0, |path| {
            reads += 1;
            bounded::text(path)
        })
        .unwrap_err();
        assert!(error.to_string().contains("inventory"));
        assert_eq!(reads, 1);
        Ok(())
    }

    #[test]
    fn discovery_matches_full_decode_across_strings_tables_and_inline_layouts() {
        let manifests = [
            r#"
                "rust_module" = 'crates/port/src/pure.rs'
                projectile_rendering = false
                notes = """Quoted \"value\", # comment, and fake header:
[[routines]]
symbol = 'EprojInstruction'
"""
                [["routines"]] # actual routine, not fake ]]
                'symbol' = "Pure"
                [[routines.cases]]
                text = '''Literal fake header:
[[routines]]
pointer_bank = 134
'''
                writes = [
                    { offset = 1, data = [0, 1, 2] }, # ] " '''
                    { offset = 2, data = [3, 4, 5] },
                ]
                [[routines]]
                symbol = "Later"
                pointer_bank = 134
            "#,
            "rust_module = 'pure.rs'\nprojectile_host = true\n[[routines]]\n",
            "rust_module = 'pure.rs'\nroutines = [{symbol = 'Pure'}, {pointer_bank = 134}]\n",
            "rust_module = 'pure.rs'\nroutines = [{cases = [{text = '}, symbol = fake', data = [0, 1]}], symbol = 'Pure'}, {pointer_bank = 134}] # ]\n",
            "rust_module = 'pure.rs'\n[[routines]]\nsymbol = \"\"\"Pure\"\"\"\n",
            "rust_module = 'pure.rs'\n[rom]\nfile = 'rootless'\n",
        ];
        for manifest in manifests {
            assert_eq!(discovery(manifest).unwrap(), toml::from_str::<Discovery>(manifest).unwrap());
        }
        for invalid in [
            "rust_module = 12\n",
            "rust_module = 'one'\nrust_module = 'two'\n",
            "projectile_host = 'yes'\n",
            "[[routines]]\npointer_bank = 65536\n",
            "[[routines]]\nsymbol = 'unterminated\n",
            "rust_module = 'pure.rs'\nroutines.symbol = 'Pure'\n",
            "[[routines]]\n[routines.symbol]\nvalue = 1\n",
            "[[routines.cases]]\nvalue = [0, 1\n",
            "[[routines.cases]]\nvalue = [0, 1}\n",
            "[[routines.cases]]\nvalue = '''unterminated\n",
        ] {
            assert!(discovery(invalid).is_err(), "invalid discovery accepted: {invalid}");
        }
    }
    #[test]
    fn catalog_discovers_signatures_without_parsing_fixture_payloads() {
        let checkout = std::env::temp_dir().join(format!(
            "sm-batch-catalog-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(checkout.join("tools/port")).unwrap();
        fs::write(checkout.join("tools/port/batch.toml"), "[families]\n[profiles]\n").unwrap();
        let mut manifest = String::from(
            "rust_module = 'crates/port/src/heavy.rs'\n[[routines]]\nsymbol = 'Pure'\n[[routines.cases]]\nname = 'large'\ndata = [\n",
        );
        for _ in 0..2_000_000 {
            manifest.push_str("0,");
        }
        manifest.push_str("\n]\n[[routines]]\nsymbol = 'EprojInstruction'\npointer_bank = 134\n");
        fs::write(checkout.join("tools/port/heavy.toml"), manifest).unwrap();
        let mut manifest = String::from("rust_module = 'crates/port/src/inline.rs'\nroutines = [{cases = [{data = [");
        for _ in 0..2_000_000 {
            manifest.push_str("0,");
        }
        manifest.push_str("]}], symbol = 'Pure'}, {symbol = 'Later', pointer_bank = 134}]\n");
        fs::write(checkout.join("tools/port/inline.toml"), manifest).unwrap();
        let started = Instant::now();
        let catalog = catalog(&checkout).unwrap();
        let elapsed = started.elapsed();
        fs::remove_dir_all(&checkout).unwrap();
        assert_eq!(entry(&catalog, "heavy").unwrap().kind, "dispatcher");
        assert_eq!(entry(&catalog, "inline").unwrap().kind, "dispatcher");
        assert_eq!(catalog.profiles["offline-all"], ["heavy", "inline"]);
        eprintln!("fixture-heavy catalog discovery: {elapsed:?}");
        assert!(elapsed < std::time::Duration::from_secs(2), "catalog parsed fixture payloads: {elapsed:?}");
    }
    #[test]
    #[ignore = "reads the checkout's complete manifest inventory"]
    fn catalog_discovers_retained_checkout_manifests() {
        let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
        let started = Instant::now();
        let catalog = catalog(checkout).unwrap();
        let elapsed = started.elapsed();
        assert_eq!(entry(&catalog, "observed-ridley-instructions-captured").unwrap().kind, "dispatcher");
        assert!(catalog.profiles["offline-all"].contains(&"observed-ridley-instructions-captured".into()));
        eprintln!("retained manifest catalog discovery: {} families, {elapsed:?}", catalog.families.len());
        assert!(elapsed < std::time::Duration::from_secs(10), "retained catalog remained slow: {elapsed:?}");
    }
    #[test]
    fn replay_selection_authenticates_manifests_and_optional_plm() {
        let checkout = std::env::temp_dir().join(format!(
            "sm-batch-selection-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(checkout.join("tools/port")).unwrap();
        let mut config = Catalog { families: BTreeMap::new(), profiles: BTreeMap::new() };
        let mut batch = Batch::default();
        let names = ["pure", "callback", "setup", "plm-spawn", "discovered"].map(str::to_owned);
        for (name, kind) in
            [("pure", "family"), ("callback", "callback"), ("setup", "offline-family"), ("plm-spawn", "plm")]
        {
            config.families.insert(name.into(), CatalogEntry::Kind(kind.into()));
        }
        config.families.insert(
            "discovered".into(),
            CatalogEntry::Details(FamilyEntry {
                kind: "dispatcher".into(),
                manifest: "tools/port/discovered.toml".into(),
                live: None,
                prepare: None,
            }),
        );
        for name in &names {
            let manifest = checkout.join(format!("tools/port/{name}.toml"));
            fs::write(&manifest, "live_eligible=true\n").unwrap();
            batch.families.insert(
                name.clone(),
                FamilyRecord {
                    status: "offline-verified".into(),
                    exit_code: 0,
                    verdict: Some(format!("{name}-verdict.toml").into()),
                    manifest_sha256: Some(digest(&manifest).unwrap()),
                    ..Default::default()
                },
            );
        }
        let request = Request { native_plm: true, room_plm: true, ..Default::default() };
        let replay = replay_request(&request, &checkout, &batch, &config, &names, &checkout.join("replay")).unwrap();
        assert_eq!(replay.family_verdicts, vec![PathBuf::from("pure-verdict.toml")]);
        assert_eq!(replay.callback_verdicts, vec![PathBuf::from("callback-verdict.toml")]);
        assert!(replay.dispatcher_verdicts.is_empty());
        assert_eq!(replay.plm_verdict, Some("plm-spawn-verdict.toml".into()));
        assert!(replay_request(&Request::default(), &checkout, &batch, &config, &["setup".into()], &checkout).is_err());
        for name in ["pure", "callback", "plm-spawn"] {
            let manifest = checkout.join(format!("tools/port/{name}.toml"));
            fs::write(&manifest, "live_eligible=false\n").unwrap();
            assert!(
                replay_request(&request, &checkout, &batch, &config, &names, &checkout)
                    .unwrap_err()
                    .to_string()
                    .contains(&format!("{name}: manifest changed"))
            );
            fs::write(&manifest, "live_eligible=true\n").unwrap();
        }
        batch.families.get_mut("pure").unwrap().exit_code = 1;
        assert!(
            replay_request(&request, &checkout, &batch, &config, &names, &checkout)
                .unwrap_err()
                .to_string()
                .contains("missing passing evidence")
        );
        fs::remove_dir_all(checkout).unwrap();
    }
}
