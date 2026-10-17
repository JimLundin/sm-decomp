//! Frozen native live matrix. Preparation is outside the complete measured boundary.
use anyhow::{Context, Result, bail, ensure};
use pipeline_model::metrics::{Measurement, Receipt};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

mod host;

const ORDER: [(usize, u64); 12] = [
    (2, 1000),
    (2, 45000),
    (4, 1000),
    (4, 45000),
    (4, 1000),
    (4, 45000),
    (2, 1000),
    (2, 45000),
    (2, 1000),
    (2, 45000),
    (4, 1000),
    (4, 45000),
];
const SUBJECTS: [&str; 4] = ["c-0", "c-1", "mixed-0", "mixed-1"];

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub format: String,
    pub pipeline: PathBuf,
    pub output: PathBuf,
    pub request: crate::live_model::Request,
    pub capacity_roots: BTreeMap<String, PathBuf>,
    pub cache_seed: PathBuf,
    pub store_seed: PathBuf,
    /// Seed runs for both extents, sealed by this exact source/reader workload.
    pub seed_verdicts: Vec<PathBuf>,
    #[serde(default)]
    pub frozen: BTreeMap<PathBuf, String>,
    #[serde(default)]
    pub cache_inventory: BTreeMap<String, String>,
    #[serde(default)]
    pub store_inventory: BTreeMap<String, String>,
    #[serde(default)]
    pub source_revision: String,
    #[serde(default)]
    pub environment_sha256: String,
    pub max_rss_bytes: u64,
    pub max_private_staging_bytes: u64,
    pub minimum_free_bytes: u64,
}

#[derive(Serialize, Deserialize)]
struct Sample {
    sequence: usize,
    workers: usize,
    frames: u64,
    comparable: bool,
    measurement_started: bool,
    elapsed_ns: u64,
    last_completed_stage: Option<String>,
    error: Option<String>,
    verdict: Option<PathBuf>,
    metrics: PathBuf,
    cleanup: Vec<crate::cleanup::CleanupReceipt>,
    free_bytes_after: u64,
    #[serde(default)]
    attribution: Vec<StageSummary>,
}
#[derive(Serialize, Deserialize)]
struct StageSummary {
    role: String,
    stage: String,
    count: usize,
    summed_elapsed_ns: u64,
    union_elapsed_ns: Option<u64>,
    union_unavailable_reason: Option<String>,
    semantics: String,
}
fn attribute(receipt: &Receipt) -> Result<Vec<StageSummary>> {
    let roles = std::iter::once((receipt.scope_id.clone(), receipt.role.clone()))
        .chain(receipt.scopes.iter().map(|s| (s.id.clone(), s.role.clone())))
        .collect::<BTreeMap<_, _>>();
    let mut groups: BTreeMap<(String, Option<String>), (u64, u64)> = BTreeMap::new();
    for summary in tool_execution::metrics::stage_summaries(receipt)? {
        let totals = groups.entry((summary.role, summary.name)).or_default();
        totals.0 = totals.0.checked_add(summary.count).context("stage count overflow")?;
        totals.1 = totals.1.checked_add(summary.summed_elapsed_ns).context("stage duration overflow")?;
    }
    groups.into_iter().map(|((role, name), (count, summed_elapsed_ns))| {
        let count = usize::try_from(count)?;
        let mut spans = receipt.stages.iter().filter(|stage| {
            name.as_deref() == Some(stage.name.as_str())
                && stage.parent.as_ref().and_then(|parent| roles.get(parent)) == Some(&role)
        }).map(|stage| (stage.start_ns, stage.end_ns)).collect::<Vec<_>>();
        let complete = name.is_some() && count == spans.len();
        let union_elapsed_ns = complete.then(|| {
            spans.sort_unstable();
            let mut union = 0;
            let mut end = 0;
            for (start, next) in &spans { union += next.saturating_sub(end.max(*start)); end = end.max(*next); }
            union
        });
        Ok(StageSummary {
            role, stage: name.unwrap_or_else(|| "unavailable-stage-labels".into()), count, summed_elapsed_ns,
            union_elapsed_ns,
            union_unavailable_reason: (!complete).then(|| "individual spans or labels omitted by bounded telemetry; summed duration cannot establish a wall-time union".into()),
            semantics: "counts and summed durations cover all observed spans; a union is reported only for a complete trace; other groups can overlap/nest; neither sums nor unions across groups equal total wall or removable critical-path share".into(),
        })
    }).collect()
}
#[derive(Serialize)]
struct Cell {
    workers: usize,
    frames: u64,
    samples_ns: Vec<u64>,
    median_ns: u64,
    minimum_ns: u64,
    maximum_ns: u64,
}
#[derive(Serialize)]
struct Summary {
    format: &'static str,
    source_revision: String,
    config_sha256: String,
    boundary: &'static str,
    cells: Vec<Cell>,
}

pub fn command(args: &[String]) -> Result<()> {
    let _detailed = tool_execution::metrics::detailed();
    ensure!(args.len() == 2, "usage: benchmark prepare|freeze|validate|run CONFIG.toml");
    let path = Path::new(&args[1]);
    let mut config: Config = pipeline_model::read(path)?;
    match args[0].as_str() {
        "prepare" => prepare(&mut config, path),
        "freeze" => {
            ensure!(config.frozen.is_empty(), "config already frozen");
            freeze(&mut config)?;
            crate::atomic_toml(path, &config)?;
            crate::emit(&crate::report!({"status":"frozen", "config":path}))
        }
        "validate" => {
            validate(&config)?;
            authenticate_seeds(&config)?;
            crate::emit(&crate::report!({"status":"validated", "config_sha256":crate::digest(path)?}))
        }
        "run" => run(config, path),
        other => bail!("unknown benchmark operation: {other}"),
    }
}

fn prepare(config: &mut Config, path: &Path) -> Result<()> {
    ensure!(config.frozen.is_empty() && config.seed_verdicts.is_empty(), "seed configuration already prepared");
    let mut preflight = config.clone();
    preflight.seed_verdicts = vec![PathBuf::from("preparation-1000"), PathBuf::from("preparation-45000")];
    structure(&preflight)?;
    fs::create_dir_all(&config.output)?;
    // The live command remains the authority for current-source admission.
    // Preparation is an explicit cold/empty-store control, outside the warm matrix.
    let seeds = config.output.with_file_name(format!(
        "{}-seeds",
        config.output.file_name().context("benchmark output name missing")?.to_string_lossy()
    ));
    fs::create_dir(&seeds)?;
    let root = config.capacity_roots.get("2").context("capacity-two root missing")?.clone();
    ensure!(!root.join("build-cache").exists(), "prepare requires a fresh capacity root cache");
    let mut host = host::Monitor::start(&seeds.join("host.toml"))?;
    host.preflight()?;
    let mut metrics =
        tool_execution::metrics::Run::start(seeds.join("pipeline.metrics.toml"), "benchmark-seed-preparation")?;
    fs::create_dir(seeds.join("live"))?;
    metrics.context.add_live_output(seeds.join("live"));
    metrics.context.sample_disk();
    let store = seeds.join("shared-store");
    let result = (|| -> Result<()> {
        for frames in [1000, 45000] {
            let mut request = config.request.clone();
            request.root = root.clone();
            request.workers = 2;
            request.live_jobs = 2;
            request.frames = frames;
            request.output = seeds.join("live");
            request.evidence_store = Some(store.clone());
            let text = invoke(config, &metrics.context, &mut host, "live-verify", &request, &seeds, true)?;
            let verdict: crate::live_model::Verdict = toml::from_str(&text)?;
            ensure!(verdict.status == "mixed-C-Rust-integration", "seed admission failed");
            config.seed_verdicts.push(verdict.run.join("verdict.toml"));
        }
        Ok(())
    })();
    let error = result.as_ref().err().map(|e| format!("{e:#}"));
    // Authenticate every accepted seed after cleanup, including one retained
    // before a later preparation refusal. Partial rejected runs keep diagnostics.
    for entry in fs::read_dir(seeds.join("live"))? {
        let verdict_path = entry?.path().join("verdict.toml");
        if verdict_path.is_file() {
            let verdict: crate::live_model::Verdict = pipeline_model::read(&verdict_path)?;
            if verdict.status == "mixed-C-Rust-integration" && !config.seed_verdicts.contains(&verdict_path) {
                config.seed_verdicts.push(verdict_path);
            }
        }
    }
    metrics.finish(tool_execution::metrics::outcome(error.as_deref()), error)?;
    host.finish()?;
    for deduplicate in [false, true] {
        let cleanup = crate::cleanup::cleanup(crate::cleanup::CleanupRequest {
            root: seeds.join("live"),
            apply: true,
            deduplicate,
        })?;
        crate::atomic_toml(&seeds.join(if deduplicate { "dedup.toml" } else { "cleanup.toml" }), &cleanup)?;
    }
    for verdict in &config.seed_verdicts {
        crate::observations::authenticate(crate::observations::AuthenticateRequest {
            verdict: verdict.clone(),
            root: root.clone(),
            workers: 2,
        })?;
    }
    result?;
    config.store_seed = store;
    fs::create_dir(&config.cache_seed)?;
    // Copy only cache keys named by these two successfully admitted live runs.
    let mut keys = BTreeSet::new();
    for path in &config.seed_verdicts {
        let verdict: crate::live_model::Verdict = pipeline_model::read(path)?;
        keys.insert(verdict.evidence_cache.context("evidence cache receipt missing")?.key);
        keys.insert(verdict.decompressor_cache.context("decompressor cache receipt missing")?.key);
        #[derive(Deserialize)]
        struct CCache {
            objects: Vec<tool_execution::build_cache::CacheResult>,
        }
        let c: CCache = pipeline_model::read(&verdict.run.join("c-cache.toml"))?;
        keys.extend(c.objects.into_iter().map(|c| c.key));
    }
    for key in keys {
        let source = root.join("build-cache").join(&key);
        let target = config.cache_seed.join(key);
        fs::create_dir(&target)?;
        for name in ["artifact", "seal.toml"] {
            fs::copy(source.join(name), target.join(name))?;
        }
    }
    fs::rename(root.join("build-cache"), seeds.join("native-cache"))?;
    freeze(config)?;
    crate::atomic_toml(path, config)?;
    crate::emit(&crate::report!({"status":"prepared","config":path,"seeds":seeds,
        "scope":"cold/empty-store seed preparation; excluded from warm measured matrix"}))
}

fn files(root: &Path, directory: &Path, output: &mut BTreeMap<String, String>) -> Result<()> {
    ensure!(!directory.is_symlink(), "inventory directory is a symlink");
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        let meta = path.symlink_metadata()?;
        ensure!(!meta.file_type().is_symlink(), "inventory symlink: {}", path.display());
        if meta.is_dir() {
            files(root, &path, output)?;
        } else {
            ensure!(meta.is_file(), "inventory special file");
            let relative = path.strip_prefix(root)?.to_str().context("inventory path is not UTF-8")?;
            // Publication's root lock coordinates writers; it is not an immutable
            // store object and must not enter the seed clone or its byte identity.
            if relative == "pipeline.lock" {
                continue;
            }
            // Seed inventories must contain only immutable native artifacts/seals/packs.
            let allowed = path.extension().is_some_and(|e| e == "pack")
                || path.file_name().is_some_and(|n| n == "artifact" || n == "seal.toml");
            ensure!(allowed, "unsealed seed member {relative}");
            output.insert(relative.into(), crate::digest(&path)?);
        }
    }
    Ok(())
}
fn inventory(path: &Path) -> Result<BTreeMap<String, String>> {
    let mut output = BTreeMap::new();
    files(path, path, &mut output)?;
    ensure!(!output.is_empty(), "empty seed inventory");
    Ok(output)
}
fn freeze(config: &mut Config) -> Result<()> {
    structure(config)?;
    let checkout = &config.request.checkout;
    let clean = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=normal"])
        .current_dir(checkout)
        .output()?;
    ensure!(clean.status.success() && clean.stdout.is_empty(), "commit combined source before freezing benchmark");
    let git = Command::new("git").args(["ls-files", "-z"]).current_dir(checkout).output()?;
    ensure!(git.status.success(), "cannot enumerate combined source");
    for name in git.stdout.split(|b| *b == 0).filter(|b| !b.is_empty()) {
        let name = std::str::from_utf8(name)?;
        // Tracker, documentation and retained evidence are not build inputs.
        if name.starts_with("crates/")
            || name.starts_with("tools/")
            || name.starts_with(".cargo/")
            || ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rustfmt.toml"].contains(&name)
        {
            let path = checkout.join(name);
            if path.is_file() {
                config.frozen.insert(path.clone(), crate::digest(&path)?);
            }
        }
    }
    let revision = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(checkout).output()?;
    ensure!(revision.status.success(), "cannot freeze revision");
    config.source_revision = String::from_utf8(revision.stdout)?.trim().into();
    config.environment_sha256 = environment_digest()?;
    let mut paths = vec![
        config.pipeline.clone(),
        checkout.join("rom/sm.sfc"),
        checkout.join("movies/super-metroid-any-3653.inputs"),
        config.request.accepted.clone(),
        config.request.rng_predecessor.clone().context("RNG predecessor required")?,
    ];
    paths.extend(config.request.family_verdicts.clone());
    for verdict in &config.seed_verdicts {
        paths.extend([verdict.clone(), verdict.parent().context("seed parent missing")?.join("sm-evidence")]);
        let admitted: crate::live_model::Verdict = pipeline_model::read(verdict)?;
        for relative in admitted.rust_dependencies.keys() {
            paths.push(crate::documents::resolve(checkout, relative));
        }
        if let Some(source) = admitted.source {
            paths.extend(source.files.keys().map(|relative| checkout.join("deps/snesrev-sm").join(relative)));
        }
    }
    for compiler in ["cc", "cargo", "rustc"] {
        paths.push(crate::executable(compiler)?);
    }
    for path in paths {
        config.frozen.insert(path.clone(), crate::digest(&path)?);
    }
    config.cache_inventory = inventory(&config.cache_seed)?;
    config.store_inventory = inventory(&config.store_seed)?;
    validate(config)?;
    authenticate_seeds(config)
}

fn structure(config: &Config) -> Result<()> {
    ensure!(config.format == "pipeline-benchmark-v1", "unknown benchmark config");
    ensure!(
        config.pipeline.is_absolute()
            && config.output.is_absolute()
            && config.request.checkout.is_absolute()
            && config.cache_seed.is_absolute()
            && config.store_seed.is_absolute(),
        "benchmark paths must be absolute"
    );
    let r = &config.request;
    ensure!(
        r.family_verdicts.len() == 1
            && r.rng_predecessor.is_some()
            && r.dispatcher_verdicts.is_empty()
            && r.callback_verdicts.is_empty()
            && r.plm_verdict.is_none()
            && r.projectile_verdict.is_none()
            && !r.room_plm
            && !r.capture_active_eprojs
            && !r.retain_private_stores,
        "benchmark workload must be system/decompressor/RNG"
    );
    let family: toml::Value = pipeline_model::read(&r.family_verdicts[0])?;
    ensure!(family.get("family").and_then(toml::Value::as_str) == Some("system"), "system family required");
    ensure!(
        config.max_rss_bytes == 8_589_934_592
            && config.max_private_staging_bytes == 68_719_476_736
            && config.minimum_free_bytes == 53_687_091_200,
        "frozen host budgets differ"
    );
    ensure!(config.capacity_roots.len() == 2, "two separate capacity roots required");
    for capacity in [2, 4] {
        let root = config.capacity_roots.get(&capacity.to_string()).context("capacity root missing")?;
        #[derive(Deserialize)]
        struct Capacity {
            workers: usize,
        }
        let actual: Capacity = pipeline_model::read(&root.join("workers/capacity.toml"))?;
        ensure!(root.is_absolute() && actual.workers == capacity, "wrong capacity root");
    }
    ensure!(config.capacity_roots["2"] != config.capacity_roots["4"], "capacity roots must differ");
    ensure!(config.seed_verdicts.len() == 2, "one seed verdict for each extent required");
    Ok(())
}
fn validate(config: &Config) -> Result<()> {
    structure(config)?;
    ensure!(!config.frozen.is_empty() && !config.source_revision.is_empty(), "unfrozen benchmark");
    ensure!(config.environment_sha256 == environment_digest()?, "compiler environment changed");
    ensure!(std::env::var("SM_PIPELINE_METRICS").as_deref() != Ok("disabled"), "benchmark telemetry must be enabled");
    for (path, hash) in &config.frozen {
        ensure!(crate::digest(path)? == *hash, "frozen input changed: {}", path.display());
    }
    for path in [
        &config.pipeline,
        &config.request.accepted,
        config.request.rng_predecessor.as_ref().unwrap(),
        &config.request.family_verdicts[0],
    ] {
        ensure!(config.frozen.contains_key(path), "required frozen identity missing: {}", path.display());
    }
    ensure!(inventory(&config.cache_seed)? == config.cache_inventory, "cache seed inventory changed");
    ensure!(inventory(&config.store_seed)? == config.store_inventory, "store seed inventory changed");
    evidence::Store::open(&config.store_seed)?.verify_packs()?;
    let git = Command::new("git").args(["ls-files", "-z"]).current_dir(&config.request.checkout).output()?;
    ensure!(git.status.success(), "cannot recheck combined source inventory");
    for name in git.stdout.split(|b| *b == 0).filter(|b| !b.is_empty()) {
        let name = std::str::from_utf8(name)?;
        if name.starts_with("crates/")
            || name.starts_with("tools/")
            || name.starts_with(".cargo/")
            || ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rustfmt.toml"].contains(&name)
        {
            let path = config.request.checkout.join(name);
            ensure!(config.frozen.contains_key(&path), "new unfrozen source: {}", path.display());
        }
    }
    for entry in fs::read_dir(&config.cache_seed)? {
        let entry = entry?.path();
        ensure!(entry.is_dir(), "cache seed must contain entry directories");
        let seal: tool_execution::build_cache::CacheSeal = pipeline_model::read(&entry.join("seal.toml"))?;
        ensure!(
            seal.inputs.version == 4 && crate::digest(&entry.join("artifact"))? == seal.sha256,
            "cache seed seal differs"
        );
    }
    Ok(())
}

fn environment_digest() -> Result<String> {
    use sha2::Digest;
    let values = std::env::vars()
        .filter(|(name, _)| {
            name.starts_with("CARGO_")
                || name.starts_with("RUST")
                || name.starts_with("CC_")
                || [
                    "CC",
                    "AR",
                    "CFLAGS",
                    "CPPFLAGS",
                    "LDFLAGS",
                    "PATH",
                    "CPATH",
                    "C_INCLUDE_PATH",
                    "CPLUS_INCLUDE_PATH",
                    "LIBRARY_PATH",
                    "SOURCE_DATE_EPOCH",
                    "GCC_EXEC_PREFIX",
                    "COMPILER_PATH",
                    "SDKROOT",
                    "MACOSX_DEPLOYMENT_TARGET",
                ]
                .contains(&name.as_str())
        })
        .collect::<BTreeMap<_, _>>();
    Ok(format!("{:x}", sha2::Sha256::digest(toml::to_string(&values)?.as_bytes())))
}
fn authenticate_seeds(config: &Config) -> Result<()> {
    let mut extents = BTreeSet::new();
    let store = evidence::Store::open(&config.store_seed)?;
    for path in &config.seed_verdicts {
        let verdict: crate::live_model::Verdict = pipeline_model::read(path)?;
        ensure!(
            verdict.status == "mixed-C-Rust-integration" && verdict.state_effects_match,
            "seed live verdict is not admitted"
        );
        ensure!(
            !verdict.projectile_native_installed
                && !verdict.room_plm_installed
                && verdict.predecessors.len() == 1
                && verdict.predecessors[0].path == config.request.family_verdicts[0]
                && verdict.decompressor_predecessor_sha256 == crate::digest(&config.request.accepted)?
                && verdict.rng_predecessor_sha256 == crate::digest(config.request.rng_predecessor.as_ref().unwrap())?,
            "seed workload/predecessor identity differs"
        );
        for (relative, hash) in &verdict.rust_dependencies {
            ensure!(
                crate::digest(&crate::documents::resolve(&config.request.checkout, relative))? == *hash,
                "seed current-source closure differs: {relative}"
            );
        }
        extents.insert(verdict.frames);
        crate::observations::authenticate(crate::observations::AuthenticateRequest {
            verdict: path.clone(),
            root: config.capacity_roots["2"].clone(),
            workers: 2,
        })?;
        for name in SUBJECTS {
            let seal = verdict.evidence_storage.subjects.get(name).context("seed subject missing")?;
            let (run, _) = evidence::verified_inventory(&store, &evidence::parse_id(&seal.run)?)?;
            ensure!(evidence::hex(&run.observations) == seal.observations, "seed observations differ");
        }
    }
    ensure!(extents == BTreeSet::from([1000, 45000]), "seed extents differ");
    Ok(())
}

fn clone_seed(source: &Path, destination: &Path, expected: &BTreeMap<String, String>) -> Result<()> {
    fs::create_dir(destination)?;
    let mut buffer = [0u8; 65536];
    for relative in expected.keys() {
        ensure!(
            !relative.is_empty()
                && Path::new(relative).components().all(|c| matches!(c, std::path::Component::Normal(_))),
            "unsafe seed inventory path"
        );
        let target = destination.join(relative);
        fs::create_dir_all(target.parent().context("seed parent missing")?)?;
        fs::copy(source.join(relative), &target)?;
        // Repeatable read warming of exactly the same seed bytes, outside the timer.
        let mut input = File::open(target)?;
        while input.read(&mut buffer)? != 0 {}
    }
    ensure!(inventory(destination)? == *expected, "cloned seed inventory differs");
    Ok(())
}

fn value(measurement: &Measurement, name: &str) -> Result<u64> {
    match measurement {
        Measurement::Measured { value } => Ok(*value),
        Measurement::Unavailable { reason } => bail!("required {name} unavailable: {reason}"),
    }
}
fn budgets(config: &Config, context: &tool_execution::metrics::Context) -> Result<()> {
    let (resources, complete) = context.budget_resources();
    ensure!(complete, "incomplete resource sampling");
    ensure!(value(&resources.aggregate_sampled_rss_bytes, "RSS")? <= config.max_rss_bytes, "RSS budget exceeded");
    // Disk paths may not yet exist during preparation/build; free reserve is always checked.
    if let Measurement::Measured { value: allocation } = resources.combined_allocated_bytes {
        ensure!(allocation <= config.max_private_staging_bytes, "staging/private budget exceeded");
    }
    ensure!(
        tool_execution::resource_sampler::available_bytes(&config.output)? >= config.minimum_free_bytes,
        "free disk reserve breached"
    );
    Ok(())
}

fn invoke<T: Serialize>(
    config: &Config,
    context: &tool_execution::metrics::Context,
    host: &mut host::Monitor,
    operation: &str,
    request: &T,
    sample: &Path,
    enforce: bool,
) -> Result<String> {
    let index = context.snapshot().processes.len();
    let label = format!("{index:03}-{operation}");
    let stdout_path = sample.join(format!("{label}.stdout"));
    let mut command = Command::new(&config.pipeline);
    command
        .arg(operation)
        .current_dir(&config.request.checkout)
        .env("SM_PIPELINE_METRICS", "enabled")
        .stdin(Stdio::piped())
        .stdout(File::create(&stdout_path)?)
        .stderr(File::create(sample.join(format!("{label}.stderr")))?);
    let metrics = sample.join(format!("{label}.metrics.toml"));
    // Only live-verify creates a root receipt. Other public commands are still owned/accounted.
    if operation == "live-verify" {
        context.child_context(&mut command, &metrics, operation)?;
    } else {
        command.env_remove("SM_PIPELINE_METRICS_CONTEXT");
    }
    let mut child = tool_execution::metrics::OwnedChild::spawn(&mut command, context, operation)?;
    child.take_stdin().context("child stdin missing")?.write_all(toml::to_string(request)?.as_bytes())?;
    let mut last = Instant::now() - Duration::from_secs(1);
    let status = loop {
        if let Err(error) = tool_execution::metrics::check_cancelled() {
            child.terminate()?;
            return Err(error);
        }
        if last.elapsed() >= Duration::from_millis(500) {
            let check = (|| -> Result<()> {
                if enforce {
                    budgets(config, context)?;
                }
                host.sample(enforce)
            })();
            if let Err(error) = check {
                child.terminate()?;
                if operation == "live-verify" && metrics.exists() {
                    context.merge_child(&metrics)?;
                }
                return Err(error);
            }
            last = Instant::now();
        }
        match child.poll_exit() {
            Ok(Some(status)) => break status,
            Ok(None) => (),
            Err(error) => {
                child.terminate()?;
                return Err(error);
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    if operation == "live-verify" && metrics.exists() {
        context.merge_child(&metrics)?;
    }
    ensure!(status.success(), "{operation} refused; diagnostics at {}", sample.display());
    Ok(fs::read_to_string(stdout_path)?)
}

fn warm_cache(verdict: &crate::live_model::Verdict, run: &Path) -> Result<()> {
    ensure!(
        verdict.evidence_cache.as_ref().is_some_and(|c| c.hit)
            && verdict.decompressor_cache.as_ref().is_some_and(|c| c.hit),
        "unexpected native Cargo cache miss"
    );
    let cache: toml::Value = pipeline_model::read(&run.join("c-cache.toml"))?;
    let objects = cache.get("objects").and_then(toml::Value::as_array).context("C cache records missing")?;
    // Generated host and wrapped entry units are intentionally rebuilt; reusable units must hit.
    let reusable = objects.iter().filter(|c| c.get("key").is_some()).collect::<Vec<_>>();
    ensure!(
        !reusable.is_empty() && reusable.iter().all(|c| c.get("hit").and_then(toml::Value::as_bool) == Some(true)),
        "unexpected native C cache miss"
    );
    Ok(())
}

fn run(config: Config, config_path: &Path) -> Result<()> {
    validate(&config)?;
    authenticate_seeds(&config)?;
    let config_hash = crate::digest(config_path)?;
    fs::create_dir_all(&config.output)?;
    let plan = config.output.join("benchmark-plan.toml");
    ensure!(!plan.exists(), "benchmark output is immutable; choose a new directory");
    crate::atomic_toml(&plan, &config)?;
    let mut cells: BTreeMap<(usize, u64), Vec<u64>> = BTreeMap::new();
    for (index, (workers, frames)) in ORDER.into_iter().enumerate() {
        validate(&config)?;
        ensure!(crate::digest(config_path)? == config_hash, "benchmark configuration changed");
        let sample = config.output.join(format!("sample-{:02}-{workers}-{frames}", index + 1));
        fs::create_dir(&sample)?;
        fs::create_dir(sample.join("live"))?;
        let root = &config.capacity_roots[&workers.to_string()];
        let metrics_path = sample.join("pipeline.metrics.toml");
        let mut record = Sample {
            sequence: index + 1,
            workers,
            frames,
            comparable: false,
            measurement_started: false,
            elapsed_ns: 0,
            last_completed_stage: None,
            error: None,
            verdict: None,
            metrics: metrics_path.clone(),
            cleanup: vec![],
            free_bytes_after: 0,
            attribution: vec![],
        };
        let cache = root.join("build-cache");
        let store = sample.join("shared-store");
        let preparation = (|| -> Result<host::Monitor> {
            ensure!(!cache.exists(), "capacity root cache is populated; use independent empty roots");
            clone_seed(&config.cache_seed, &cache, &config.cache_inventory)?;
            clone_seed(&config.store_seed, &store, &config.store_inventory)?;
            let cloned = evidence::Store::open(&store)?;
            for path in &config.seed_verdicts {
                let verdict: crate::live_model::Verdict = pipeline_model::read(path)?;
                for subject in verdict.evidence_storage.subjects.values() {
                    evidence::verified_inventory(&cloned, &evidence::parse_id(&subject.run)?)?;
                }
            }
            let mut host = host::Monitor::start(&sample.join("host.toml"))?;
            host.preflight()?;
            Ok(host)
        })();
        let mut host = match preparation {
            Ok(host) => host,
            Err(error) => {
                // No live writer has started. Preserve the refusal and leave any
                // partially cloned cache in place for diagnosis; no timing is admitted.
                record.error = Some(format!("sample preparation: {error:#}"));
                for deduplicate in [false, true] {
                    match crate::cleanup::cleanup(crate::cleanup::CleanupRequest {
                        root: sample.join("live"),
                        apply: true,
                        deduplicate,
                    }) {
                        Ok(receipt) => record.cleanup.push(receipt),
                        Err(error) => record.error.as_mut().unwrap().push_str(&format!("; cleanup: {error:#}")),
                    }
                }
                record.free_bytes_after = tool_execution::resource_sampler::available_bytes(&config.output)?;
                crate::atomic_toml(&sample.join("sample.toml"), &record)?;
                return Err(error);
            }
        };
        let mut metrics = tool_execution::metrics::Run::start(metrics_path, "benchmark")?;
        record.measurement_started = true;
        metrics.context.configuration("config-sha256", &config_hash);
        metrics.context.configuration("source-revision", &config.source_revision);
        metrics
            .context
            .configuration("page-cache", "same seed files read before timer; OS cache not globally controlled");
        let mut request = config.request.clone();
        request.root = root.clone();
        request.workers = workers;
        request.live_jobs = workers;
        request.frames = frames;
        request.output = sample.join("live");
        request.evidence_store = Some(store);
        metrics.context.add_live_output(sample.join("live"));
        metrics.context.sample_disk();
        let result = (|| -> Result<()> {
            let stage = metrics.context.stage("live-verification")?;
            let text = invoke(&config, &metrics.context, &mut host, "live-verify", &request, &sample, true)?;
            let verdict: crate::live_model::Verdict = toml::from_str(&text)?;
            record.verdict = Some(verdict.run.join("verdict.toml"));
            ensure!(
                verdict.status == "mixed-C-Rust-integration" && verdict.frames == frames,
                "live workload/admission differs"
            );
            warm_cache(&verdict, &verdict.run)?;
            stage.finish("success")?;
            for deduplicate in [false, true] {
                let stage = metrics.context.stage(if deduplicate { "deduplication" } else { "cleanup" })?;
                let text = invoke(
                    &config,
                    &metrics.context,
                    &mut host,
                    "cleanup",
                    &crate::cleanup::CleanupRequest { root: sample.join("live"), apply: true, deduplicate },
                    &sample,
                    true,
                )?;
                record.cleanup.push(toml::from_str(&text)?);
                stage.finish("success")?;
            }
            let stage = metrics.context.stage("retained-authentication")?;
            invoke(
                &config,
                &metrics.context,
                &mut host,
                "live-authenticate",
                &crate::observations::AuthenticateRequest {
                    verdict: record.verdict.clone().unwrap(),
                    root: root.clone(),
                    workers,
                },
                &sample,
                true,
            )?;
            stage.finish("success")?;
            budgets(&config, &metrics.context)?;
            let final_resources = metrics.context.snapshot().resources;
            value(&final_resources.combined_allocated_bytes, "complete staging/private peak")?;
            ensure!(
                value(&final_resources.minimum_free_bytes, "minimum free reserve")? >= config.minimum_free_bytes,
                "sampled free reserve breached"
            );
            Ok(())
        })();
        // Freeze the successful timer before post-run checks and receipt writes.
        // Failed-run recovery is recorded separately and never enters a median.
        let measured_end = metrics.context.now()?;
        let result = result.and_then(|()| {
            validate(&config)?;
            ensure!(crate::digest(config_path)? == config_hash, "benchmark configuration changed");
            Ok(())
        });
        record.error = result.as_ref().err().map(|e| format!("{e:#}"));
        if result.is_err() {
            // OwnedChild has stopped/reaped writers before this failure cleanup.
            for deduplicate in [false, true] {
                match crate::cleanup::cleanup(crate::cleanup::CleanupRequest {
                    root: sample.join("live"),
                    apply: true,
                    deduplicate,
                }) {
                    Ok(receipt) => record.cleanup.push(receipt),
                    Err(error) => record.error.as_mut().unwrap().push_str(&format!("; cleanup: {error:#}")),
                }
            }
            // Record retained accepted verdicts even if a later budget/host/cache check refused the sample.
            for entry in fs::read_dir(sample.join("live"))? {
                let path = entry?.path().join("verdict.toml");
                if path.is_file() {
                    let verdict: crate::live_model::Verdict = pipeline_model::read(&path)?;
                    if verdict.status == "mixed-C-Rust-integration" {
                        record.verdict = Some(path.clone());
                        let _ = invoke(
                            &config,
                            &metrics.context,
                            &mut host,
                            "live-authenticate",
                            &crate::observations::AuthenticateRequest { verdict: path, root: root.clone(), workers },
                            &sample,
                            false,
                        )
                        .map_err(|e| {
                            record.error.as_mut().unwrap().push_str(&format!("; retained authentication: {e:#}"))
                        });
                    }
                }
            }
        }
        metrics.finish(tool_execution::metrics::outcome(record.error.as_deref()), record.error.clone())?;
        let receipt = metrics.context.snapshot();
        record.elapsed_ns = measured_end.saturating_sub(receipt.start_ns);
        record.attribution = attribute(&receipt)?;
        record.last_completed_stage = receipt.last_completed_stage;
        record.comparable = result.is_ok();
        record.free_bytes_after = tool_execution::resource_sampler::available_bytes(&config.output)?;
        host.finish()?;
        crate::atomic_toml(&sample.join("sample.toml"), &record)?;
        // Preserve the sample's final cache inventory for warmth audit and diagnostics.
        // The next sample still starts from an independent clone of the original seed.
        fs::rename(&cache, sample.join("native-cache"))?;
        ensure!(record.comparable, "sample {} incomparable: {}", index + 1, record.error.unwrap_or_default());
        cells.entry((workers, frames)).or_default().push(record.elapsed_ns);
        eprintln!(
            "benchmark sample {}: capacity {workers}, {frames} inputs, {:.3}s",
            index + 1,
            record.elapsed_ns as f64 / 1e9
        );
    }
    let mut summaries = vec![];
    for ((workers, frames), times) in cells {
        let mut sorted = times.clone();
        sorted.sort_unstable();
        ensure!(sorted.len() == 3, "incomplete benchmark cell");
        summaries.push(Cell {
            workers,
            frames,
            samples_ns: times,
            median_ns: sorted[1],
            minimum_ns: sorted[0],
            maximum_ns: sorted[2],
        });
    }
    crate::atomic_toml(
        &config.output.join("summary.toml"),
        &Summary {
            format: "pipeline-benchmark-summary-v1",
            source_revision: config.source_revision,
            config_sha256: config_hash,
            boundary: "public live verify through publication, authentication, cleanup/dedup and retained authentication; independent warmed native cache/shared-store seeds; preparation excluded",
            cells: summaries,
        },
    )
}

#[cfg(test)]
mod controls {
    use super::*;
    #[test]
    fn bounded_attribution_retains_complete_sums_and_marks_missing_time_unions() {
        let _detailed = tool_execution::metrics::detailed();
        let path = std::env::temp_dir().join(format!("sm-benchmark-attribution-{}.toml", std::process::id()));
        let mut run = tool_execution::metrics::Run::start(path.clone(), "attribution-control").unwrap();
        let outer = run.context.stage("overlap").unwrap();
        let inner = run.context.stage("overlap").unwrap();
        inner.finish("success").unwrap();
        outer.finish("success").unwrap();
        for _ in 0..5_000 {
            run.context.stage("repeated").unwrap().finish("success").unwrap();
        }
        run.finish("success", None).unwrap();
        let receipt = run.context.snapshot();
        let attribution = attribute(&receipt).unwrap();
        let overlap = attribution.iter().find(|summary| summary.stage == "overlap").unwrap();
        assert_eq!(overlap.count, 2);
        assert!(overlap.union_elapsed_ns.is_some_and(|union| union > 0 && union < overlap.summed_elapsed_ns));
        assert!(overlap.union_unavailable_reason.is_none());
        let repeated = attribution.iter().find(|summary| summary.stage == "repeated").unwrap();
        assert_eq!(repeated.count, 5_000);
        assert!(repeated.summed_elapsed_ns > 0);
        assert!(repeated.union_elapsed_ns.is_none());
        assert!(repeated.union_unavailable_reason.as_deref().is_some_and(|reason| reason.contains("omitted")));
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn real_live_refusal_keeps_diagnostics_and_scoped_cleanup() {
        let _detailed = tool_execution::metrics::detailed();
        let root = std::env::temp_dir().join(format!(
            "sm-benchmark-live-refusal-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let request = crate::live_model::Request {
            checkout: root.clone(),
            output: root.join("live"),
            root: root.join("workers"),
            workers: 2,
            live_jobs: 2,
            frames: 1000,
            ..Default::default()
        };
        let verdict = crate::live::verify(request).unwrap();
        assert_eq!(verdict.status, "rejected");
        assert!(verdict.error.as_deref().is_some_and(|e| !e.is_empty()));
        let metrics: Receipt = pipeline_model::read(&verdict.run.join("pipeline.metrics.toml")).unwrap();
        assert_eq!(metrics.outcome, "failed");
        assert_eq!(metrics.last_completed_stage.as_deref(), Some("scoped-cleanup"));
        assert!(verdict.run.join("verdict.toml").is_file());
        for deduplicate in [false, true] {
            crate::cleanup::cleanup(crate::cleanup::CleanupRequest {
                root: root.join("live"),
                apply: true,
                deduplicate,
            })
            .unwrap();
        }
        assert!(verdict.run.join("verdict.toml").is_file());
        fs::remove_dir_all(root).unwrap();
    }
}
