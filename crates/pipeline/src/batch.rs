//! Per-batch ownership and authenticated native family reuse across worktrees.
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
#[derive(Deserialize)]
struct Discovery {
    rust_module: Option<PathBuf>,
    routines: Option<Vec<DiscoveryRoutine>>,
    #[serde(default)]
    projectile_host: bool,
    #[serde(default)]
    projectile_rendering: bool,
}
#[derive(Deserialize)]
struct DiscoveryRoutine {
    #[serde(default)]
    symbol: String,
    pointer_bank: Option<u16>,
}
pub fn catalog(checkout: &Path) -> Result<Catalog> {
    let mut config: Catalog = pipeline_model::read(&checkout.join("tools/port/batch.toml"))?;
    let mut paths = fs::read_dir(checkout.join("tools/port"))?.map(|e| Ok(e?.path())).collect::<Result<Vec<_>>>()?;
    paths.sort();
    for path in paths {
        if path.extension().is_none_or(|v| v != "toml") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy();
        if config.families.contains_key(name.as_ref()) {
            continue;
        }
        let manifest: Discovery = pipeline_model::read(&path)?;
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
    }
    config.profiles.insert("offline-all".into(), config.families.keys().cloned().collect());
    Ok(config)
}
fn entry(config: &Catalog, name: &str) -> Result<FamilyEntry> {
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
            case_jobs: 1,
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
            "power-bomb"
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
    pub families: BTreeMap<String, FamilyRecord>,
    pub replays: Vec<ReplayRecord>,
    pub cleanup: Option<cleanup::CleanupReceipt>,
}
#[derive(Serialize, Deserialize)]
struct Latest {
    batch: PathBuf,
}
pub fn operate(request: Request) -> Result<toml::Value> {
    let checkout = fs::canonicalize(&request.checkout)?;
    let config = catalog(&checkout)?;
    let command = request.operation.as_str();
    ensure!(
        ["verify", "replay", "run", "status", "catalog"].contains(&command),
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
    if let Some(revision) = &request.changed_since {
        let diff = std::process::Command::new("git")
            .args(["diff", "--name-only", revision, "--"])
            .current_dir(&checkout)
            .output()?;
        ensure!(diff.status.success(), "changed-since revision cannot be resolved");
        let changed = String::from_utf8(diff.stdout)?.lines().map(str::to_owned).collect::<BTreeSet<_>>();
        let global = changed.iter().any(|p| {
            p.starts_with("tools/")
                || p.starts_with("crates/pipeline")
                || p.starts_with(".cargo/")
                || ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "crates/port/src/memory.rs"]
                    .contains(&p.as_str())
        });
        if !global {
            names.retain(|name| {
                let Ok(item) = entry(&config, name) else {
                    return true;
                };
                let Ok(manifest) = pipeline_model::read::<family_model::Manifest>(&checkout.join(&item.manifest))
                else {
                    return true;
                };
                changed.iter().any(|p| {
                    Path::new(p) == item.manifest
                        || Path::new(p) == Path::new(&manifest.rust_module)
                        || manifest.rust_dependencies.iter().any(|v| v == Path::new(p))
                        || p.starts_with("crates/port/src/")
                })
            });
        }
        if names.is_empty() {
            return Ok(crate::report!({"status":"unchanged","families":Vec::<String>::new()}));
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
        std::thread::scope(|scope| {
            for _ in 0..request.jobs.min(names.len()) {
                scope.spawn(|| {
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(name) = names.get(index) else {
                            break;
                        };
                        let previous = state.lock().unwrap().families.get(name).cloned();
                        let record = match verify_one(&request, &checkout, &path, &config, name, previous.as_ref()) {
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
                        if let Err(error) = atomic_toml(&path, &**owner) {
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
    let cleaned = cleanup::cleanup(cleanup::CleanupRequest {
        root: path.parent().unwrap().into(),
        apply: true,
        deduplicate: true,
    })?;
    let passed = replay_passed && (command == "replay" || verified);
    Ok(toml::Value::try_from(Receipt {
        status: if passed { "complete" } else { "failed" }.into(),
        batch: path,
        families: batch.families,
        replays: batch.replays,
        cleanup: Some(cleaned),
    })?)
}

#[cfg(test)]
mod tests {
    use super::*;
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
