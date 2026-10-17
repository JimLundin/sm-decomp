//! Per-batch ownership and authenticated native family reuse across worktrees.
use crate::atomic_toml;
use crate::cleanup;
use crate::digest;
use crate::live;
use crate::live_model;
use anyhow::{Context, Result, ensure};
use offline_verification::catalog::{Catalog, entry};
#[cfg(test)]
use offline_verification::catalog::{CatalogEntry, FamilyEntry, catalog, catalog_until, catalog_using};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
#[cfg(test)]
use tool_execution::bounded;
use tool_execution::metrics;
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct FamilyRecord {
    pub status: String,
    pub exit_code: i32,
    pub verdict: Option<PathBuf>,
    pub manifest_sha256: Option<String>,
    pub kind: Option<String>,
    /// Elapsed work on this invocation, including unsuccessful reuse attempts.
    pub seconds: f64,
    /// First execution's duration when this row represents retained-result reuse.
    pub original_run_seconds: Option<f64>,
    pub reuse_authentication_seconds: Option<f64>,
    pub reuse_authentication_error: Option<String>,
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
fn family_record(outcome: &offline_verification::FamilyCompletion, previous: Option<&FamilyRecord>) -> FamilyRecord {
    let evaluated = match outcome.result() {
        Ok(evaluated) => evaluated,
        Err(error) => {
            return FamilyRecord {
                status: metrics::outcome(Some(&format!("{error:#}"))).into(),
                exit_code: 1,
                error: Some(format!("{error:#}")),
                seconds: outcome.seconds(),
                ..Default::default()
            };
        }
    };
    if evaluated.run.reused() {
        let mut record = previous.expect("reused family has a supplied previous batch record").clone();
        record.status = "reused".into();
        record.original_run_seconds = Some(record.original_run_seconds.unwrap_or(record.seconds));
        record.seconds = outcome.seconds();
        record.reuse_authentication_seconds = evaluated.reuse_authentication_seconds;
        record.reuse_authentication_error = evaluated.reuse_authentication_error.clone();
        return record;
    }
    let path = evaluated.run.evidence();
    let verdict = evaluated.run.report();
    FamilyRecord {
        exit_code: if verdict.status == "offline-verified" { 0 } else { 1 },
        status: match metrics::outcome(verdict.error.as_deref()) {
            outcome @ ("interrupted" | "timed-out") => outcome.into(),
            _ => verdict.status.clone(),
        },
        verdict: Some(path),
        manifest_sha256: Some(evaluated.manifest_sha256.clone()),
        kind: Some(evaluated.kind.clone()),
        seconds: outcome.seconds(),
        original_run_seconds: None,
        reuse_authentication_seconds: evaluated.reuse_authentication_seconds,
        reuse_authentication_error: evaluated.reuse_authentication_error.clone(),
        error: verdict.error.clone(),
    }
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
    // Metadata commands remain read-only. An existing caller metrics scope can
    // still observe planning inside operate_inner; no default sidecar is created.
    if matches!(request.operation.as_str(), "plan" | "catalog") {
        return operate_inner(request);
    }
    let destination = request
        .batch
        .as_ref()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| request.checkout.join("out/port/batches"));
    fs::create_dir_all(&destination)?;
    let mut metrics = metrics::Run::start(
        destination.join(format!("batch-{}.metrics.toml", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos())),
        "batch",
    )?;
    metrics.context.configuration("workers", request.workers);
    metrics.context.configuration("case-jobs", request.case_jobs);
    metrics.context.configuration("jobs", request.jobs);
    metrics.context.configuration("operation", &request.operation);
    metrics.context.configuration("profile", &request.profile);
    metrics.context.configuration("fresh-required", request.fresh);
    metrics.context.configuration("elapsed-boundary", "after CLI startup/tool authentication through batch cleanup; external wall time includes startup, metrics publication and later independent admission");
    metrics.context.expect_stages(&[
        "batch-planning",
        "batch-selection-lock",
        "batch-record-admission",
        "batch-family-verification",
        "batch-live-replay",
        "batch-publication",
        "batch-scoped-cleanup",
    ]);
    let metrics_path = metrics.context.path().to_path_buf();
    let mut result = operate_inner(request);
    let success = result.as_ref().is_ok_and(|value| {
        value
            .get("status")
            .and_then(toml::Value::as_str)
            .is_none_or(|status| ["complete", "unchanged"].contains(&status))
    });
    let error = if success {
        None
    } else {
        Some(result.as_ref().err().map(|error| format!("{error:#}")).unwrap_or_else(|| {
            let value = result.as_ref().unwrap();
            let errors = value
                .get("selected")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|name| value.get("families")?.get(name.as_str()?)?.get("error")?.as_str())
                .chain(
                    value
                        .get("replays")
                        .and_then(toml::Value::as_array)
                        .and_then(|replays| replays.last())
                        .and_then(|replay| replay.get("error"))
                        .and_then(toml::Value::as_str),
                )
                .collect::<Vec<_>>()
                .join("; ");
            if errors.is_empty() { "batch reports unsuccessful verification".into() } else { errors }
        }))
    };
    metrics.finish(metrics::outcome(error.as_deref()), error)?;
    if let Ok(value) = &mut result
        && let Some(table) = value.as_table_mut()
    {
        table.insert("metrics".into(), toml::Value::try_from(metrics_path)?);
    }
    result
}
fn operate_inner(request: Request) -> Result<toml::Value> {
    let context = metrics::active();
    let planning = context.as_ref().map(|context| context.stage("batch-planning")).transpose()?;
    let checkout = fs::canonicalize(&request.checkout)?;
    let offline = crate::offline::families()?;
    let snapshot = offline.capture_catalog(offline_verification::CatalogRequest::new(&checkout)?)?;
    let config = snapshot.catalog();
    metrics::check_cancelled()?;
    let command = request.operation.as_str();
    ensure!(
        ["verify", "replay", "run", "status", "catalog", "plan"].contains(&command),
        "unsupported batch operation: {command}"
    );
    if command == "catalog" {
        if let Some(planning) = planning {
            planning.finish("success")?;
        }
        return Ok(toml::Value::try_from(config)?);
    }
    let mut names = request
        .families
        .clone()
        .or_else(|| config.profiles.get(&request.profile).cloned())
        .context("unknown profile or missing families")?;
    if command == "plan" || request.changed_since.is_some() {
        let report = crate::affected::select_with_snapshot(&snapshot, &names, request.changed_since.as_deref())?;
        if command == "plan" {
            if let Some(planning) = planning {
                planning.finish("success")?;
            }
            return Ok(toml::Value::try_from(report)?);
        }
        snapshot.validate_current()?;
        names = report.families();
        if names.is_empty() {
            if let Some(planning) = planning {
                planning.finish("success")?;
            }
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
    if let Some(context) = &context {
        context.configuration("selected-families", names.len());
    }
    if let Some(planning) = planning {
        planning.finish("success")?;
    }
    let source = if ["verify", "run", "replay"].contains(&command) {
        Some(fs::canonicalize(checkout.join("deps/snesrev-sm"))?)
    } else {
        None
    };
    let captured_inputs = if ["verify", "run"].contains(&command) {
        Some(offline.capture_batch_inputs(offline_verification::BatchInputRequest {
            catalog: &snapshot,
            source: source.as_ref().unwrap(),
            required: &names,
        })?)
    } else {
        None
    };
    let selection = checkout.join("out/port/batches");
    fs::create_dir_all(&selection)?;
    let selecting = OpenOptions::new().create(true).append(true).open(selection.join("selection.lock"))?;
    metrics::measure("batch-selection-lock", || metrics::lock(&selecting))?;
    let admitting = context.as_ref().map(|context| context.stage("batch-record-admission")).transpose()?;
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
    if let Some(context) = &context {
        context.add_path(path.parent().unwrap().into(), tool_execution::resource_sampler::PathKind::Staging);
        context.configuration(
            "disk-path-accounting",
            "selected batch directory, including retained outputs and temporary files; artifact cache excluded",
        );
    }
    let lock = OpenOptions::new().create(true).append(true).open(path.with_extension("lock"))?;
    ensure!(lock.try_lock().is_ok(), "another command is writing this batch; retry after it finishes");
    snapshot.validate_current()?;
    if let Some(inputs) = &captured_inputs {
        inputs.validate_current()?;
    }
    let mut batch: Batch = if path.exists() {
        let value: Batch = pipeline_model::read(&path)?;
        ensure!(value.schema_version == 2, "unsupported batch version");
        value
    } else {
        ensure!(["verify", "run"].contains(&command), "batch does not exist");
        let value = Batch { schema_version: 2, profile: request.profile.clone(), ..Default::default() };
        metrics::measure("batch-publication", || {
            atomic_toml(&path, &value)?;
            atomic_toml(&latest, &Latest { batch: path.clone() })
        })?;
        value
    };
    drop(selecting);
    if let Some(admitting) = admitting {
        admitting.finish("success")?;
    }
    if command == "status" {
        return Ok(toml::Value::try_from(batch)?);
    }
    if ["verify", "run"].contains(&command) {
        let verifying = context.as_ref().map(|context| context.stage("batch-family-verification")).transpose()?;
        ensure!((1..=8).contains(&request.jobs), "batch jobs must be 1..8");
        let families = names
            .iter()
            .map(|name| {
                let previous =
                    batch.families.get(name).filter(|record| !request.fresh && record.exit_code == 0).and_then(
                        |record| {
                            Some(offline_verification::PreviousFamily {
                                verdict: record.verdict.clone()?,
                                manifest_sha256: record.manifest_sha256.clone()?,
                                kind: record.kind.clone()?,
                            })
                        },
                    );
                offline_verification::FamilyJob {
                    family: name.clone(),
                    output: path.parent().unwrap().join("families").join(name),
                    previous,
                }
            })
            .collect();
        let state = Mutex::new(&mut batch);
        let execution = offline.verify_batch(
            offline_verification::BatchVerification {
                catalog: &snapshot,
                source: source.as_ref().unwrap().clone(),
                root: request.root.clone(),
                workers: request.workers,
                family_jobs: request.jobs,
                case_jobs: request.case_jobs,
                families,
                inputs: captured_inputs.as_ref(),
            },
            |outcome| {
                let mut owner = state.lock().unwrap();
                let record = family_record(outcome, owner.families.get(outcome.family()));
                eprintln!("{}: {}", outcome.family(), record.status);
                owner.families.insert(outcome.family().into(), record);
                if let Err(error) = metrics::measure("batch-publication", || atomic_toml(&path, &**owner)) {
                    owner.publication_error = Some(format!("{error:#}"));
                    return Err(error);
                }
                Ok(())
            },
        );
        if let Err(error) = execution {
            batch.publication_error = Some(format!("{error:#}"));
            atomic_toml(&path, &batch)?;
        }
        ensure!(batch.publication_error.is_none(), "batch publication failed");
        if let Some(verifying) = verifying {
            let errors = names
                .iter()
                .filter_map(|name| batch.families.get(name)?.error.as_deref())
                .collect::<Vec<_>>()
                .join("; ");
            verifying.finish(metrics::outcome((!errors.is_empty()).then_some(errors.as_str())))?;
        }
    }
    let verified = names.iter().all(|name| batch.families.get(name).is_some_and(|r| r.exit_code == 0));
    if command == "replay" {
        let evidence = names
            .iter()
            .map(|name| -> Result<_> {
                let record = batch.families.get(name).context("required batch family missing")?;
                ensure!(record.exit_code == 0, "required batch family has no passing evidence");
                Ok(offline_verification::FamilyEvidence {
                    family: name.clone(),
                    verdict: record.verdict.clone().context("family verdict missing")?,
                    manifest_sha256: record.manifest_sha256.clone().context("family manifest identity missing")?,
                    kind: record.kind.clone().context("family kind missing")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        offline.authenticate_batch(offline_verification::BatchAdmission {
            catalog: &snapshot,
            required: &names,
            source: source.as_ref().unwrap().clone(),
            evidence,
        })?;
    }
    let mut replay_passed = true;
    if ["replay", "run"].contains(&command) && (command == "replay" || verified) {
        let replaying = context.as_ref().map(|context| context.stage("batch-live-replay")).transpose()?;
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
        if let Some(replaying) = replaying {
            replaying.finish(metrics::outcome(record.error.as_deref()))?;
        }
        batch.replays.push(record);
        metrics::measure("batch-publication", || atomic_toml(&path, &batch))?;
    }
    let cleaned = metrics::measure("batch-scoped-cleanup", || {
        cleanup::cleanup(cleanup::CleanupRequest {
            root: path.parent().unwrap().into(),
            apply: true,
            deduplicate: true,
        })
    })?;
    if let Some(context) = &context {
        context.configuration("cleanup-reclaimed-bytes", cleaned.bytes);
        let stats = snapshot.statistics();
        context.configuration("catalog-parses", stats.catalog_parses);
        context.configuration("catalog-manifest-decodes", stats.manifest_decodes);
        context.configuration("catalog-manifest-reads", stats.manifest_reads);
        context.configuration("catalog-freshness-reads", stats.freshness_reads);
        context.configuration("catalog-freshness-bytes", stats.freshness_bytes);
        context.configuration("catalog-retained-byte-reservation", stats.retained_bytes);
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
    fn read_only_catalog_preserves_a_callers_metrics_scope() -> Result<()> {
        crate::initialize_test_binary_identity()?;
        let directory = tool_execution::testing::Directory::new()?;
        let checkout = directory.0.join("checkout");
        fs::create_dir_all(checkout.join("tools/port"))?;
        fs::write(checkout.join("tools/port/batch.toml"), "[families]\n[profiles]\n")?;
        let metrics_path = directory.0.join("caller.metrics.toml");
        let mut caller = metrics::Run::start(metrics_path.clone(), "metadata-caller")?;
        let result =
            operate(Request { checkout: checkout.clone(), operation: "catalog".into(), ..Default::default() })?;
        assert!(result.get("metrics").is_none());
        assert!(!checkout.join("out").exists());
        assert_eq!(metrics::active().unwrap().path(), metrics_path);
        caller.finish("success", None)?;
        let receipt: pipeline_model::metrics::Receipt = pipeline_model::read(&metrics_path)?;
        assert!(receipt.stages.iter().any(|stage| stage.name == "batch-planning" && stage.outcome == "success"));
        assert!(receipt.processes.is_empty(), "metadata planning launched a producer");
        Ok(())
    }

    #[test]
    fn catalog_refuses_expired_budget_and_excessive_inventory_before_manifest_reads() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        let port = directory.0.join("tools/port");
        fs::create_dir_all(&port)?;
        let config = "[families]\n[profiles]\n";
        fs::write(port.join("batch.toml"), config)?;
        offline_verification::fixtures::write(
            &port.join("one.smf"),
            &offline_verification::fixtures::Manifest { rust_module: "one.rs".into(), ..Default::default() },
        )?;
        assert!(catalog_until(&directory.0, 0, 1024).unwrap_err().to_string().contains("deadline"));
        assert!(catalog_until(&directory.0, bounded::now_unix_ms()? + 1000, config.len() as u64).is_err());
        for number in 0..4096 {
            fs::write(port.join(format!("extra-{number}.toml")), b"")?;
        }
        let mut reads = 0usize;
        let error = catalog_using(&directory.0, |path| {
            reads += 1;
            bounded::read(path)
        })
        .unwrap_err();
        assert!(error.to_string().contains("inventory"));
        assert_eq!(reads, 1);
        Ok(())
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
                manifest: "tools/port/discovered.smf".into(),
                live: None,
                prepare: None,
            }),
        );
        for name in &names {
            let manifest = checkout.join(format!("tools/port/{name}.smf"));
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
            let manifest = checkout.join(format!("tools/port/{name}.smf"));
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
