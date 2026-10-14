//! Local process leases and authenticated cache publication for decomp workers.
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use toml::Value;

mod affected;
mod assignment;
mod baseline;
mod batch;
mod benchmark;
mod builds;
mod callback;
mod checkpoints;
mod cleanup;
mod coordination;
mod coverage;
mod decomp_acceptance;
mod decomp_job;
mod decomp_model;
mod decomp_prefix;
mod diagnostics;
mod enemy_instruction_inventory;
mod family;
mod family_model;
mod fixture_execution;
mod fixtures;
mod hdma_probe;
mod inputs;
mod iteration_demonstrations;
mod iteration_diagnosis;
mod iteration_increment;
mod iteration_pattern;
mod iteration_selection;
mod iteration_session;
mod live;
mod live_model;
mod lsnes_configure;
mod metrics;
mod observations;
mod packed;
mod paired_verify;
mod parity;
mod pattern_contract;
mod patterns;
mod plm_fixtures;
mod plm_spawn;
mod preflight;
mod progress;
mod projectile_inventory;
mod readiness;
mod readiness_contract;
mod readiness_feedback;
mod readiness_metadata;
mod readiness_origins;
mod readiness_reuse;
mod readiness_run;
mod readiness_sync;
mod replay;
mod resource_sampler;
mod ridley_subsystem_adapter;
mod ridley_subsystem_fixtures;
mod ridley_subsystem_inventory;
mod rng_job;
mod routine_live;
mod session;
mod source_inventory;
mod storage_studies;
mod translation;
mod translation_cargo;
mod translation_compact;
mod translation_continuation;
mod translation_execution;
mod translation_infrastructure;
mod translation_launch;
mod translation_launch_prepare;
mod translation_legacy;
mod translation_migration;
mod translation_migration_builder;
mod translation_migration_guard;
mod translation_native;
mod translation_original;
mod translation_original_prepare;
mod translation_production;
mod translation_production_prepare;

static BINARY_SHA256: std::sync::OnceLock<String> = std::sync::OnceLock::new();

fn binary_digest() -> Result<&'static str> {
    BINARY_SHA256.get().map(String::as_str).context("binary identity was not initialized")
}

fn digest(path: &Path) -> Result<String> {
    let mut hash = Sha256::new();
    let mut input = File::open(path)?;
    let mut buffer = [0u8; 65536];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn lock(path: &Path) -> Result<File> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    metrics::lock(&file)?;
    Ok(file)
}

fn emit<T: Serialize>(value: &T) -> Result<()> {
    print!("{}", toml::to_string_pretty(value)?);
    std::io::stdout().flush()?;
    Ok(())
}

fn set_option<T: Serialize>(options: &mut Value, name: impl AsRef<str>, value: T) -> Result<()> {
    let table = options.as_table_mut().context("CLI options must be a TOML table")?;
    table.insert(name.as_ref().to_owned(), Value::try_from(value)?);
    Ok(())
}

fn request<T: serde::de::DeserializeOwned>() -> Result<T> {
    let mut input = String::new();
    std::io::stdin().lock().take(1024 * 1024 + 1).read_to_string(&mut input)?;
    ensure!(input.len() <= 1024 * 1024, "request exceeds 1 MiB");
    toml::from_str(&input).context("invalid TOML request")
}

// Cache and lease are interactive. Each TOML document is prefixed by its
// byte length so embedded newlines never split a request or reply.
fn framed_request<T: serde::de::DeserializeOwned>() -> Result<T> {
    let mut input = std::io::stdin().lock();
    let mut header = [0u8; 4];
    input.read_exact(&mut header)?;
    let length = u32::from_le_bytes(header) as usize;
    ensure!(length <= 1024 * 1024, "cache request exceeds 1 MiB");
    let mut bytes = vec![0; length];
    input.read_exact(&mut bytes)?;
    toml::from_str(std::str::from_utf8(&bytes)?).context("invalid framed TOML request")
}
fn framed_emit<T: Serialize>(value: &T) -> Result<()> {
    let bytes = toml::to_string(value)?.into_bytes();
    let mut output = std::io::stdout().lock();
    output.write_all(&u32::try_from(bytes.len())?.to_le_bytes())?;
    output.write_all(&bytes)?;
    output.flush()?;
    Ok(())
}

fn line() -> Result<String> {
    let mut value = String::new();
    ensure!(std::io::stdin().lock().read_line(&mut value)? > 0, "client closed without commit");
    Ok(value)
}

fn acquire_lease(root: &Path, requested: usize) -> Result<(File, f64)> {
    let now = u64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis())?;
    acquire_lease_until(root, requested, now.checked_add(3_600_000).context("worker deadline overflow")?)
}

fn acquire_lease_until(root: &Path, requested: usize, deadline_unix_ms: u64) -> Result<(File, f64)> {
    ensure!((1..=64).contains(&requested), "worker count must be 1..64");
    let started = Instant::now();
    let now = u64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis())?;
    ensure!(now < deadline_unix_ms, "shared worker time budget exhausted before reservation");
    let deadline = started + Duration::from_millis(deadline_unix_ms - now).min(Duration::from_secs(3600));
    let root = root.join("workers");
    fs::create_dir_all(&root)?;
    let guard = OpenOptions::new()
        .create(true)
        .append(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(root.join("config.lock"))?;
    ensure!(guard.metadata()?.is_file(), "worker configuration lock is not regular");
    loop {
        metrics::check_cancelled()?;
        ensure!(Instant::now() < deadline, "shared worker time budget exhausted waiting for configuration");
        match guard.try_lock() {
            Ok(()) => break,
            Err(std::fs::TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(10)),
            Err(std::fs::TryLockError::Error(error)) => return Err(error.into()),
        }
    }
    let config = root.join("capacity.toml");
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Capacity {
        workers: usize,
    }
    let capacity = if config.exists() {
        pipeline_model::read::<Capacity>(&config)?.workers
    } else {
        pipeline_model::write(&config, &Capacity { workers: requested })?;
        requested
    };
    ensure!(capacity == requested, "shared worker budget already configured to {capacity}");
    drop(guard);
    let held = loop {
        metrics::check_cancelled()?;
        ensure!(Instant::now() < deadline, "shared worker time budget exhausted waiting for a slot");
        let mut selected = None;
        for slot in 0..capacity {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                .open(root.join(format!("{slot}.lock")))?;
            ensure!(file.metadata()?.is_file(), "worker slot lock is not regular");
            match file.try_lock() {
                Ok(()) => {
                    selected = Some(file);
                    break;
                }
                Err(std::fs::TryLockError::WouldBlock) => {}
                Err(std::fs::TryLockError::Error(error)) => return Err(error.into()),
            }
        }
        if let Some(file) = selected {
            break file;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    Ok((held, started.elapsed().as_secs_f64()))
}

fn lease(root: &Path, requested: usize) -> Result<()> {
    let (held, wait) = acquire_lease(root, requested)?;
    #[derive(Serialize)]
    struct LeaseReceipt {
        worker_wait_seconds: f64,
    }
    framed_emit(&LeaseReceipt { worker_wait_seconds: wait })?;
    // EOF releases a lease too, so killed clients cannot strand worker capacity.
    let _ = line();
    drop(held);
    Ok(())
}

fn atomic_toml<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    ensure!(path.extension().is_some_and(|e| e == "toml"), "metadata must use .toml");
    let pending = path.with_extension(format!("pending-{}", std::process::id()));
    let result = (|| -> Result<()> {
        let mut out = OpenOptions::new().create_new(true).write(true).open(&pending)?;
        out.write_all(toml::to_string_pretty(value)?.as_bytes())?;
        out.sync_all()?;
        fs::rename(&pending, path)?;
        File::open(path.parent().context("path parent missing")?)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&pending);
    }
    result
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CacheRequest {
    root: PathBuf,
    checkout: PathBuf,
    command: Vec<String>,
    inputs: Vec<PathBuf>,
    destination: PathBuf,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
struct CacheResult {
    key: String,
    hit: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
struct CompilerIdentity {
    sha256: String,
    version: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
struct CacheIdentity {
    version: u32,
    command: Vec<String>,
    sources: std::collections::BTreeMap<String, String>,
    compiler: CompilerIdentity,
    environment_sha256: String,
    platform: [String; 2],
    #[serde(skip_serializing_if = "Option::is_none")]
    rustc: Option<String>,
}
#[derive(Deserialize, Serialize)]
struct CacheSeal {
    inputs: CacheIdentity,
    sha256: String,
}

fn executable(name: &str) -> Result<PathBuf> {
    if name.contains('/') {
        return Ok(PathBuf::from(name));
    }
    for directory in std::env::split_paths(&std::env::var_os("PATH").context("PATH missing")?) {
        let candidate = directory.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    bail!("compiler not found: {name}")
}

fn cached_artifact(request: &CacheRequest, build: impl FnOnce(&Path) -> Result<()>) -> Result<CacheResult> {
    let context = metrics::active();
    let lookup = context.as_ref().map(|context| context.stage("cache-lookup")).transpose()?;
    let root = request.root.clone();
    let checkout = request.checkout.clone();
    let common = metrics::command_output(
        Command::new("git").args(["rev-parse", "--git-common-dir"]).current_dir(&checkout),
        "cache-repository-identity",
    )?;
    let repository = if common.status.success() {
        let common = PathBuf::from(String::from_utf8(common.stdout)?.trim());
        let common = fs::canonicalize(checkout.join(common))?;
        common.parent().context("git common parent missing")?.to_path_buf()
    } else {
        checkout.clone()
    };
    let inputs = &request.inputs;
    let command = &request.command;
    let compiler_name = command.first().context("compiler missing")?;
    let compiler = executable(compiler_name)?;
    let version = metrics::command_output(
        Command::new(&compiler).arg("--version").current_dir(&checkout),
        "cache-compiler-identity",
    )?;
    ensure!(version.status.success(), "compiler version failed");
    let mut sources = std::collections::BTreeMap::new();
    let mut originals = Vec::new();
    for name in inputs {
        let path = fs::canonicalize(name)?;
        let key = path
            .strip_prefix(&checkout)
            .or_else(|_| path.strip_prefix(&repository))
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        let hash = digest(&path)?;
        originals.push((path, hash.clone()));
        sources.insert(key, hash);
    }
    let environment: std::collections::BTreeMap<_, _> = std::env::vars()
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
        .collect();
    let env_hash = format!("{:x}", Sha256::digest(toml::to_string(&environment)?.as_bytes()));
    let canonical_command: Vec<_> = command
        .iter()
        .map(|arg| {
            arg.replace(checkout.to_string_lossy().as_ref(), "{checkout}")
                .replace(repository.to_string_lossy().as_ref(), "{checkout}")
        })
        .collect();
    let mut identity = CacheIdentity {
        version: 4,
        command: canonical_command,
        sources,
        compiler: CompilerIdentity { sha256: digest(&compiler)?, version: String::from_utf8(version.stdout)? },
        environment_sha256: env_hash,
        platform: [std::env::consts::ARCH.into(), std::env::consts::OS.into()],
        rustc: None,
    };
    if ["cargo", "rustc"].contains(&Path::new(compiler_name).file_name().and_then(|v| v.to_str()).unwrap_or_default()) {
        let info =
            metrics::command_output(Command::new("rustc").arg("-vV").current_dir(&checkout), "cache-rustc-identity")?;
        ensure!(info.status.success(), "rustc version failed");
        identity.rustc = Some(String::from_utf8(info.stdout)?);
    }
    let key = format!("{:x}", Sha256::digest(toml::to_string(&identity)?.as_bytes()));
    let root = root.join("build-cache");
    fs::create_dir_all(&root)?;
    let waiting = context.as_ref().map(|context| context.stage("cache-lock-wait")).transpose()?;
    let _held = lock(&root.join(format!("{key}.lock")))?;
    if let Some(waiting) = waiting {
        waiting.finish("success")?;
    }
    let entry = root.join(&key);
    for (path, expected) in &originals {
        ensure!(digest(path)? == *expected, "compiler input changed while waiting for cache");
    }
    let hit = (|| -> Result<bool> {
        ensure!(!entry.symlink_metadata()?.file_type().is_symlink(), "cache symlink");
        let seal: CacheSeal = pipeline_model::read(&entry.join("seal.toml"))?;
        ensure!(!entry.join("artifact").symlink_metadata()?.file_type().is_symlink(), "artifact symlink");
        Ok(seal.inputs == identity && seal.sha256 == digest(&entry.join("artifact"))?)
    })()
    .unwrap_or(false);
    if let Some(lookup) = lookup {
        lookup.finish("success")?;
    }
    if let Some(context) = &context {
        context.configuration(&format!("cache-{key}-hit"), hit);
    }
    let stage = root.join(format!("{key}-{}.pending", std::process::id()));
    if !hit {
        if stage.exists() {
            fs::remove_dir_all(&stage)?;
        }
        fs::create_dir(&stage)?;
    }
    if !hit {
        let result = (|| -> Result<()> {
            let building = context.as_ref().map(|context| context.stage("cache-build")).transpose()?;
            build(&stage.join("artifact"))?;
            if let Some(building) = building {
                building.finish("success")?;
            }
            for (path, expected) in originals {
                ensure!(digest(&path)? == expected, "compiler input changed during build");
            }
            ensure!(!stage.join("artifact").symlink_metadata()?.file_type().is_symlink(), "build artifact symlink");
            let hash = digest(&stage.join("artifact"))?;
            File::open(stage.join("artifact"))?.sync_all()?;
            atomic_toml(&stage.join("seal.toml"), &CacheSeal { inputs: identity, sha256: hash })?;
            if entry.exists() {
                fs::remove_dir_all(&entry)?;
            }
            fs::rename(&stage, &entry)?;
            File::open(&root)?.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(&stage);
        }
        result?;
    }
    let destination = request.destination.clone();
    let restoring = context.as_ref().map(|context| context.stage("cache-restore-authenticate")).transpose()?;
    fs::create_dir_all(destination.parent().context("destination parent missing")?)?;
    fs::copy(entry.join("artifact"), &destination)?;
    ensure!(digest(&destination)? == digest(&entry.join("artifact"))?, "copied artifact differs");
    if let Some(restoring) = restoring {
        restoring.finish("success")?;
    }
    Ok(CacheResult { key, hit })
}

fn cache(request: CacheRequest) -> Result<()> {
    #[derive(Serialize)]
    struct CacheOffer<'a> {
        hit: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        artifact: Option<&'a Path>,
    }
    let mut missed = false;
    let result = cached_artifact(&request, |artifact| {
        missed = true;
        framed_emit(&CacheOffer { hit: false, artifact: Some(artifact) })?;
        ensure!(line()?.trim() == "commit", "cache client aborted");
        Ok(())
    })?;
    if !missed {
        framed_emit(&CacheOffer { hit: true, artifact: None })?;
    }
    framed_emit(&result)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    // The controller already sealed this binary before confinement. Producers
    // have no proc filesystem and never initialize the controller's ambient
    // executable identity or enter its broader command dispatcher.
    match args.first().map(String::as_str) {
        Some("translation-cargo-producer") => return translation_cargo::producer_cli(&args[1..]),
        Some("translation-cargo-environment-producer") => {
            return translation_cargo::environment_producer_cli(&args[1..]);
        }
        Some("translation-compact-producer") => return translation_compact::producer(&args[1..]),
        _ => {}
    }
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) if std::env::var_os("SM_PIPELINE_CONFINED_CARGO").as_deref() == Some(std::ffi::OsStr::new("1")) => {
            let path = PathBuf::from(std::env::args_os().next().context("confined Cargo executable path missing")?);
            inputs::bounded::safe(&path)?;
            ensure!(
                path.is_absolute()
                    && path.starts_with("/work/tmp/target")
                    && path.symlink_metadata()?.is_file()
                    && fs::canonicalize(&path)? == path,
                "confined Cargo executable is outside its exact build directory: {error}"
            );
            path
        }
        Err(error) => return Err(error.into()),
    };
    BINARY_SHA256.set(digest(&executable)?).map_err(|_| anyhow::anyhow!("binary identity already initialized"))?;
    match args.first().map(String::as_str) {
        Some("translation-execute") => translation_execution::cli(&args[1..]),
        Some("translation-launch") => translation_launch::cli(&args[1..]),
        Some("translation-production") => translation_production::cli(&args[1..]),
        Some("translation-original") => {
            ensure!(
                args.len() == 4 && args[1] == "--checkout",
                "usage: translation-original --checkout CHECKOUT run|status|recover"
            );
            let report = translation_continuation::run(Path::new(&args[2]), &args[3])?
                .context("reviewed original production installation is absent")?;
            emit(&report)
        }
        Some("translation-original-install") => {
            ensure!(
                args.len() == 3 && args[1] == "--config",
                "usage: translation-original-install --config CONFIG.json"
            );
            emit(&translation_continuation::install(Path::new(&args[2]))?)
        }
        Some("translation-original-prepare") => {
            ensure!(
                args.len() == 3 && args[1] == "--config",
                "usage: translation-original-prepare --config CONFIG.json"
            );
            emit(&translation_original_prepare::prepare(Path::new(&args[2]))?)
        }
        Some("translation-production-prepare") => {
            ensure!(
                args.len() == 3 && args[1] == "--config",
                "usage: translation-production-prepare --config CONFIG.json"
            );
            emit(&translation_production_prepare::prepare(Path::new(&args[2]))?)
        }
        Some("translation-migration") if args.get(1).is_some_and(|value| value == "build-preparation") => {
            ensure!(
                args.len() == 4 && args[2] == "--request",
                "usage: translation-migration build-preparation --request REQUEST.json"
            );
            emit(&translation_migration_builder::build_from_request(Path::new(&args[3]))?)
        }
        Some("translation-migration") => translation_migration::cli(&args[1..]),
        Some("progress") => progress::cli(&args[1..]),
        Some("storage-study") => storage_studies::run(&args[1..]),
        Some("decomp-prefix") => decomp_prefix::run(&args[1..]),
        Some("decomp-prefix-authenticate") => {
            ensure!(args.len() == 2, "usage: decomp-prefix-authenticate RUN");
            decomp_prefix::authenticate(Path::new(&args[1]))?;
            println!("Prefix diagnostic authenticated; acceptance remains false");
            Ok(())
        }
        Some("source-inventory") => {
            let root = std::env::current_dir()?;
            let mut source = root.join("deps/snesrev-sm");
            let mut output = root.join("tools/port/source-inventory.toml");
            for pair in args[1..].chunks(2) {
                ensure!(pair.len() == 2, "inventory option missing");
                match pair[0].as_str() {
                    "--source" => source = pair[1].clone().into(),
                    "--output" => output = pair[1].clone().into(),
                    other => bail!("unknown inventory option {other}"),
                }
            }
            source_inventory::run(&root, &source, &output)
        }
        Some("iteration-diagnosis") => iteration_diagnosis::run(&args[1..]),
        Some("probe-hdma") => hdma_probe::run(&args[1..]),
        Some("recomp-iteration") => {
            let mut request = crate::report!({});
            let mut index = 1;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--root" => "checkout",
                    "--count" => "count",
                    "--target-generated" => "target_generated",
                    "--resume" => "resume",
                    "--timeout" => "timeout",
                    "--original-run" => "original_run",
                    other => bail!("unknown session option {other}"),
                };
                let value = args.get(index + 1).context("session option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if ["count", "target_generated", "timeout"].contains(&key) {
                        crate::report!(value.parse::<u64>()?)
                    } else {
                        crate::report!(value)
                    },
                )?;
                index += 2;
            }
            let state = iteration_session::operate(request)?;
            println!("Session: {}: {} accepted increments", state.session_id, state.accepted_count);
            std::process::exit(if state.status == iteration_session::Status::Complete {
                0
            } else if matches!(
                state.status,
                iteration_session::Status::RepairNeeded
                    | iteration_session::Status::DecisionRequired
                    | iteration_session::Status::Stopped
            ) {
                1
            } else {
                2
            });
        }
        Some("ridley-subsystem-authenticate") if args.len() == 2 => {
            emit(&ridley_subsystem_inventory::authenticate(&std::env::current_dir()?, Path::new(&args[1]))?)
        }
        Some("ridley-subsystem-fixtures") => {
            let mut checkout = std::env::current_dir()?;
            let mut rom = checkout.join("rom/sm.sfc");
            let mut output = checkout.join("tools/port/ridley-subsystem.toml");
            for pair in args[1..].chunks(2) {
                ensure!(pair.len() == 2, "Ridley fixture option value missing");
                match pair[0].as_str() {
                    "--root" => checkout = PathBuf::from(&pair[1]),
                    "--rom" => rom = PathBuf::from(&pair[1]),
                    "--output" => output = PathBuf::from(&pair[1]),
                    other => bail!("unknown Ridley fixture option {other}"),
                }
            }
            let manifest = ridley_subsystem_fixtures::generate(&checkout, &rom, &output)?;
            emit(&family_model::Generation {
                manifest: output,
                family: manifest.family,
                cases: manifest.routines.iter().map(|routine| routine.cases.len()).sum(),
            })
        }
        Some("ridley-subsystem-inventory") if args.len() == 1 => {
            emit(&ridley_subsystem_inventory::operate(request()?)?)
        }
        Some("ridley-subsystem-inventory") => {
            let mut request = crate::report!({});
            let mut index = 1;
            while index < args.len() {
                if ["--prepare-only", "--observation-control"].contains(&args[index].as_str()) {
                    let key = if args[index] == "--prepare-only" { "prepare_only" } else { "observation_control" };
                    set_option(&mut request, key, true)?;
                    index += 1;
                    continue;
                }
                let key = match args[index].as_str() {
                    "--root" => "checkout",
                    "--source" => "source",
                    "--asset-root" => "asset_root",
                    "--frames" => "frames",
                    "--output" => "output",
                    "--timeout" => "timeout",
                    "--capture-limit" => "capture_limit",
                    "--captures-per-key" => "captures_per_key",
                    "--pipeline-root" => "pipeline_root",
                    "--workers" => "workers",
                    other => bail!("unknown Ridley inventory option {other}"),
                };
                let value = args.get(index + 1).context("Ridley inventory option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if ["frames", "timeout", "capture_limit", "captures_per_key", "workers"].contains(&key) {
                        crate::report!(value.parse::<u64>()?)
                    } else {
                        crate::report!(value)
                    },
                )?;
                index += 2;
            }
            let verdict = ridley_subsystem_inventory::operate(request)?;
            emit(&verdict)?;
            ensure!(
                matches!(
                    verdict.get("status").and_then(Value::as_str),
                    Some("diagnostic-complete" | "diagnostic-prepared")
                ),
                "Ridley inventory failed"
            );
            Ok(())
        }
        Some("enemy-instruction-inventory") => {
            let mut request = crate::report!({});
            let mut index = 1;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--root" => "checkout",
                    "--source" => "source",
                    "--asset-root" => "asset_root",
                    "--frames" => "frames",
                    "--output" => "output",
                    "--timeout" => "timeout",
                    other => bail!("unknown enemy inventory option {other}"),
                };
                let value = args.get(index + 1).context("enemy inventory option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if ["frames", "timeout"].contains(&key) {
                        crate::report!(value.parse::<u64>()?)
                    } else {
                        crate::report!(value)
                    },
                )?;
                index += 2;
            }
            let verdict = enemy_instruction_inventory::operate(request)?;
            emit(&verdict)?;
            ensure!(
                verdict.get("status").and_then(Value::as_str) == Some("diagnostic-complete"),
                "enemy instruction inventory failed"
            );
            Ok(())
        }
        Some("projectile-inventory") => {
            let mut request = crate::report!({});
            let mut index = 1;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--root" => "checkout",
                    "--frames" => "frames",
                    "--output" => "output",
                    other => bail!("unknown inventory option {other}"),
                };
                let value = args.get(index + 1).context("inventory option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if key == "frames" { crate::report!(value.parse::<u64>()?) } else { crate::report!(value) },
                )?;
                index += 2;
            }
            let verdict = projectile_inventory::operate(request)?;
            emit(&verdict)?;
            ensure!(
                verdict.get("status").and_then(Value::as_str) == Some("diagnostic-complete"),
                "projectile inventory failed"
            );
            Ok(())
        }
        Some("readiness-feedback-case") => {
            ensure!(
                args.len() == 5 && args[1] == "--case" && args[3] == "--out",
                "usage: readiness-feedback-case --case CASE --out NEW_OUTPUT"
            );
            readiness_feedback::lifecycle(&args[2], Path::new(&args[4]))?;
            Ok(())
        }
        Some("readiness-feedback-authenticate") => {
            ensure!(args.len() == 2, "usage: readiness-feedback-authenticate RUN");
            let owner = fs::canonicalize(&args[1])?;
            let (manifest, _) = readiness_origins::snapshot(&owner)?;
            readiness_origins::owned(&owner, &manifest)?;
            let report = readiness_origins::document(&owner.join("checks/feedback-demonstrations.toml"))?;
            readiness_feedback::verify(&owner, &manifest, &report)?;
            emit(&crate::report!({"authenticated":true,"run":owner}))
        }
        Some("readiness-accept") => {
            let result = (|| -> Result<Value> {
                ensure!(args.len() == 3 && args[1] == "--run", "usage: readiness-accept --run RUN");
                let owner = fs::canonicalize(&args[2])?;
                let root =
                    owner.parent().and_then(Path::parent).and_then(Path::parent).context("run checkout missing")?;
                readiness_reuse::publish(&owner, root)
            })();
            match result {
                Ok(value) => emit(&value)?,
                Err(error) => {
                    eprintln!("readiness acceptance refused: {error:#}");
                    std::process::exit(2);
                }
            }
            Ok(())
        }
        Some("paired-authenticate") => {
            ensure!(args.len() == 3, "usage: paired-authenticate CACHE FOCUS-REPORT");
            let focus = readiness_origins::document(Path::new(&args[2]))?;
            paired_verify::verify_files(Path::new(&args[1]), &focus)?;
            emit(&crate::report!({"authenticated":true}))
        }
        Some("sync-authenticate") => {
            ensure!(args.len() == 2, "usage: sync-authenticate RUN");
            let owner = Path::new(&args[1]);
            let (manifest, _) = readiness_origins::snapshot(owner)?;
            readiness_origins::owned(owner, &manifest)?;
            let verdict = readiness_origins::document(&owner.join("verdict.toml"))?;
            ensure!(
                manifest.get("state").and_then(Value::as_str) == Some("complete")
                    && verdict.get("status").and_then(Value::as_str) == Some("pass")
                    && manifest["run_id"] == verdict["run_id"],
                "synchronization did not complete"
            );
            let frames =
                u64::try_from(manifest["scope"]["frames"].as_integer().context("synchronization scope missing")?)?;
            let computed = readiness_sync::compare(
                &owner.join("generated/original.csv"),
                &owner.join("generated/candidate.csv"),
                &manifest["identities"]["inputs"],
                frames,
            )?;
            for field in [
                "status",
                "complete",
                "events",
                "expected_frames",
                "completed_frames",
                "original_sha256",
                "candidate_sha256",
            ] {
                ensure!(
                    computed[field] == verdict["checks"]["synchronization"][field],
                    "synchronization terminal report differs"
                );
            }
            emit(&crate::report!({"authenticated":true,"frames":frames,"events":computed["events"]}))
        }
        Some("iteration-demonstrations") => iteration_demonstrations::run(&args[1..]),
        Some("iteration-pattern-authenticate") => {
            ensure!(args.len() == 2, "usage: iteration-pattern-authenticate RUN");
            println!("{}", toml::to_string_pretty(&iteration_pattern::verify_proof(Path::new(&args[1]))?)?);
            Ok(())
        }
        Some("iteration-pattern") => {
            let mut request =
                crate::report!({"command":args.get(1).context("usage: iteration-pattern prove|promote [OPTIONS]")?});
            ensure!(matches!(request["command"].as_str(), Some("prove" | "promote")), "unknown pattern operation");
            let mut index = 2;
            while index < args.len() {
                if args[index] == "--prepare-only" || args[index] == "--recover" {
                    set_option(
                        &mut request,
                        if args[index] == "--recover" { "recover" } else { "prepare_only" },
                        true,
                    )?;
                    index += 1;
                    continue;
                }
                let key = match args[index].as_str() {
                    "--root" => "checkout",
                    "--opcode" => "opcode",
                    "--selection" => "selection",
                    "--proof" => "proof",
                    "--preparation" => "preparation",
                    "--timeout" => "timeout",
                    "--original-run" => "original_run",
                    other => bail!("unknown pattern option {other}"),
                };
                let value = args.get(index + 1).context("pattern option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if key == "timeout" { crate::report!(value.parse::<u64>()?) } else { crate::report!(value) },
                )?;
                index += 2;
            }
            let verdict = iteration_pattern::operate(request)?;
            emit(&verdict)?;
            ensure!(verdict.get("status").and_then(Value::as_str) == Some("pass"), "pattern operation failed");
            Ok(())
        }
        Some(
            "readiness"
            | "readiness-reuse"
            | "readiness-feedback"
            | "iteration-selection"
            | "supported-increment"
            | "validate-iteration",
        ) => {
            let mut request = crate::report!({});
            let mut index = 1;
            while index < args.len() {
                if args[index] == "--prepare-only" {
                    set_option(&mut request, "prepare_only", true)?;
                    index += 1;
                    continue;
                }
                let key = match args[index].as_str() {
                    "--root" => "checkout",
                    "--predecessor" => "predecessor",
                    "--original-run" => "original_run",
                    "--timeout" => "timeout",
                    "--selection" => "selection",
                    "--preparation" => "preparation",
                    other => bail!("unknown readiness option {other}"),
                };
                let value = args.get(index + 1).context("readiness option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if key == "timeout" { crate::report!(value.parse::<u64>()?) } else { crate::report!(value) },
                )?;
                index += 2;
            }
            let verdict = match args[0].as_str() {
                "readiness" => readiness::operate(request)?,
                "readiness-reuse" => readiness_reuse::operate(request)?,
                "iteration-selection" => iteration_selection::operate(request)?,
                "supported-increment" => iteration_increment::operate(request)?,
                "validate-iteration" => {
                    let root = request
                        .get("checkout")
                        .and_then(Value::as_str)
                        .map(PathBuf::from)
                        .unwrap_or(std::env::current_dir()?);
                    let owner = Path::new(family::field(&request, "preparation")?);
                    let increment = iteration_increment::apply(&root, owner)?;
                    set_option(&mut request, "predecessor", crate::report!(&increment.baseline_path))?;
                    set_option(&mut request, "increment", Value::try_from(increment)?)?;
                    readiness::operate(request)?
                }
                _ => readiness_feedback::operate(request)?,
            };
            emit(&verdict)?;
            std::process::exit(match verdict["status"].as_str() {
                Some("pass") => 0,
                Some("divergence" | "uncovered") => 1,
                _ => 2,
            });
        }
        Some("readiness-sync") => {
            let mut request = crate::report!({});
            let mut index = 1;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--root" => "checkout",
                    "--frames" => "frames",
                    "--original-run" => "original_run",
                    "--attest-original" => "attest_original",
                    other => bail!("unknown synchronization option {other}"),
                };
                let value = args.get(index + 1).context("synchronization option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if key == "frames" { crate::report!(value.parse::<u64>()?) } else { crate::report!(value) },
                )?;
                index += 2;
            }
            let verdict = readiness_sync::operate(request)?;
            emit(&verdict)?;
            ensure!(verdict.get("status").and_then(Value::as_str) == Some("pass"), "synchronization failed");
            Ok(())
        }
        Some("decomp-baseline") => {
            let checkout = std::env::current_dir()?;
            let common = Command::new("git").args(["rev-parse", "--git-common-dir"]).output()?;
            ensure!(common.status.success(), "shared Git directory missing");
            let common = fs::canonicalize(checkout.join(String::from_utf8(common.stdout)?.trim()))?;
            let root = std::env::var_os("SM_PIPELINE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| common.parent().unwrap().join("out/pipeline"));
            let workers = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".into()).parse::<usize>()?;
            let mut request = crate::report!({"checkout":checkout,"root":root,"workers":workers,"output":checkout.join("out/decomp/baseline")});
            let mut index = 1;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--frames" => "frames",
                    "--inputs" => "inputs",
                    "--rom" => "rom",
                    "--output" => "output",
                    "--timeout" => "timeout",
                    option => bail!("unknown baseline option {option}"),
                };
                let value = args.get(index + 1).context("baseline option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if ["frames", "timeout"].contains(&key) {
                        crate::report!(value.parse::<u64>()?)
                    } else {
                        crate::report!(value)
                    },
                )?;
                index += 2;
            }
            let verdict = baseline::operate(request.try_into()?)?;
            emit(&verdict)?;
            ensure!(verdict.status == "replay-completed", "baseline replay failed");
            Ok(())
        }
        Some("readiness-diagnose") => {
            let mut request = crate::report!({});
            let mut index = 1;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--root" => "checkout",
                    "--frames" => "frames",
                    "--mode" => "mode",
                    "--start-input" => "start_input",
                    "--start-instruction" => "start_instruction",
                    "--instructions" => "instructions",
                    "--timeout" => "timeout",
                    "--cache" => "cache",
                    "--until-frame-end" => "until_frame_end",
                    "--keys" => "keys",
                    other => bail!("unknown diagnostic option {other}"),
                };
                if key == "until_frame_end" {
                    set_option(&mut request, key, true)?;
                    index += 1;
                } else {
                    let value = args.get(index + 1).context("diagnostic option value missing")?;
                    set_option(
                        &mut request,
                        key,
                        if ["frames", "start_input", "start_instruction", "instructions", "timeout"].contains(&key) {
                            crate::report!(value.parse::<u64>()?)
                        } else if key == "keys" {
                            toml::from_str(&fs::read_to_string(value)?)?
                        } else {
                            crate::report!(value)
                        },
                    )?;
                    index += 2;
                }
            }
            let verdict = diagnostics::operate(request)?;
            emit(&verdict)?;
            std::process::exit(match verdict["status"].as_str() {
                Some("pass") => 0,
                Some("divergence" | "uncovered") => 1,
                _ => 2,
            });
        }
        Some("readiness-parity") => {
            let mut request = crate::report!({});
            let mut index = 1;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--root" => "checkout",
                    "--mode" => "mode",
                    "--frames" => "frames",
                    "--timeout" => "timeout",
                    "--predecessor" => "predecessor",
                    option => bail!("unknown parity option {option}"),
                };
                let value = args.get(index + 1).context("parity option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if ["frames", "timeout"].contains(&key) {
                        crate::report!(value.parse::<u64>()?)
                    } else {
                        crate::report!(value)
                    },
                )?;
                index += 2;
            }
            let verdict = parity::operate(request)?;
            emit(&verdict)?;
            std::process::exit(match verdict["status"].as_str() {
                Some("pass") => 0,
                Some("divergence" | "uncovered") => 1,
                _ => 2,
            });
        }
        Some("pattern-check" | "pattern-authenticate") => {
            let mut request = crate::report!({});
            let mut index = 1;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--corpus" => "corpus",
                    "--out" => "output",
                    "--batch-size" => "batch_size",
                    "--opcodes" => "opcodes",
                    "--declaration" => "declaration",
                    "--prospective" => "prospective",
                    "--run" => "run",
                    "--timing" => "timing",
                    "--development-scope" => "development_scope",
                    option => bail!("unknown pattern option {option}"),
                };
                if ["timing", "development_scope"].contains(&key) {
                    set_option(&mut request, key, true)?;
                    index += 1;
                } else {
                    let value = args.get(index + 1).context("pattern option value missing")?;
                    set_option(
                        &mut request,
                        key,
                        if key == "batch_size" { crate::report!(value.parse::<u64>()?) } else { crate::report!(value) },
                    )?;
                    index += 2;
                }
            }
            if args[0] == "pattern-authenticate" {
                emit(&patterns::authenticate(request)?)
            } else {
                let verdict = patterns::operate(request)?;
                emit(&verdict)?;
                std::process::exit(match verdict["status"].as_str() {
                    Some("pass") => 0,
                    Some("divergence") => 1,
                    _ => 2,
                });
            }
        }
        Some("readiness-preflight") => {
            let request = if args.len() == 1 {
                crate::report!({})
            } else {
                ensure!(args.len() == 3 && args[1] == "--root", "usage: readiness-preflight [--root CHECKOUT]");
                crate::report!({"checkout":args[2]})
            };
            let verdict = preflight::operate(request)?;
            emit(&verdict)?;
            ensure!(verdict.get("status").and_then(Value::as_str) == Some("pass"), "readiness preflight failed");
            Ok(())
        }
        Some("readiness-restore") => {
            ensure!(args.len() == 3, "usage: readiness-restore RUN DESTINATION");
            readiness_run::Run::restore(Path::new(&args[1]), Path::new(&args[2]))?;
            emit(&crate::report!({"status":"restored","destination":args[2]}))
        }
        Some("rng-authenticate") if args.len() == 1 => emit(&rng_job::authenticate(request()?)?),
        Some("rng-authenticate") if args.len() == 2 => emit(&rng_job::authenticate(rng_job::AuthenticateRequest {
            checkout: std::env::current_dir()?,
            verdict: PathBuf::from(&args[1]),
        })?),
        Some("decomp-jobs") if args.len() == 1 => {
            let verdict = rng_job::operate(request()?)?;
            emit(&verdict)?;
            ensure!(verdict.status == "accepted", "routine jobs failed");
            Ok(())
        }
        Some("decomp-jobs") => {
            let checkout = std::env::current_dir()?;
            let common = Command::new("git").args(["rev-parse", "--git-common-dir"]).output()?;
            ensure!(common.status.success(), "cannot locate shared pipeline root");
            let common = fs::canonicalize(checkout.join(String::from_utf8(common.stdout)?.trim()))?;
            let root = std::env::var_os("SM_PIPELINE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| common.parent().unwrap().join("out/pipeline"));
            let workers = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".into()).parse::<usize>()?;
            let mut request = crate::report!({"checkout":checkout,"root":root,"workers":workers,"count":1,"job":"next-random","accepted":checkout.join("out/decomp/routine/decomp-acceptance.toml"),"output":checkout.join("out/decomp/jobs")});
            let mut index = 1;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--job" => "job",
                    "--count" => "count",
                    "--accepted" => "accepted",
                    "--output" => "output",
                    "--timeout" => "timeout",
                    "--live-jobs" => "live_jobs",
                    option => bail!("unknown job option: {option}"),
                };
                index += 1;
                let value = args.get(index).context("job option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if ["count", "timeout", "live_jobs"].contains(&key) {
                        crate::report!(value.parse::<i64>()?)
                    } else {
                        crate::report!(value)
                    },
                )?;
                index += 1;
            }
            let verdict = rng_job::operate(request.try_into()?)?;
            emit(&verdict)?;
            ensure!(verdict.status == "accepted", "routine jobs failed");
            Ok(())
        }
        Some("routine-live") if args.len() == 1 => {
            let verdict = routine_live::verify(request()?)?;
            emit(&verdict)?;
            ensure!(verdict.mixed_integration, "routine live verification failed");
            Ok(())
        }
        Some("routine-live") => {
            let checkout = std::env::current_dir()?;
            let common = Command::new("git").args(["rev-parse", "--git-common-dir"]).output()?;
            ensure!(common.status.success(), "cannot locate shared pipeline root");
            let common = fs::canonicalize(checkout.join(String::from_utf8(common.stdout)?.trim()))?;
            let root = std::env::var_os("SM_PIPELINE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| common.parent().unwrap().join("out/pipeline"));
            let workers = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".into()).parse::<usize>()?;
            let mut request = crate::report!({"checkout":checkout,"root":root,"workers":workers,"accepted":checkout.join("out/decomp/routine/decomp-acceptance.toml"),"output":checkout.join("out/decomp/live")});
            let mut index = 1;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--accepted" => "accepted",
                    "--output" => "output",
                    "--frames" => "frames",
                    "--timeout" => "timeout",
                    "--live-jobs" => "live_jobs",
                    "--rom" => "rom",
                    "--inputs" => "inputs",
                    "--next-random-library" => "next_random_library",
                    option => bail!("unknown routine live option: {option}"),
                };
                index += 1;
                let value = args.get(index).context("routine live option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if ["frames", "timeout", "live_jobs"].contains(&key) {
                        crate::report!(value.parse::<u64>()?)
                    } else {
                        crate::report!(value)
                    },
                )?;
                index += 1;
            }
            let verdict = routine_live::verify(request.try_into()?)?;
            emit(&verdict)?;
            ensure!(verdict.mixed_integration, "routine live verification failed");
            Ok(())
        }
        Some("fixture-sequence") => emit(&fixtures::generate_sequence(request::<fixtures::SequenceRequest>()?)?),
        Some("fixtures") if args.len() == 3 => {
            emit(&fixtures::generate(&std::env::current_dir()?, &args[1], Path::new(&args[2]))?)
        }
        Some("decomp-verify") if args.len() == 1 => {
            let verdict = decomp_job::verify(request()?)?;
            emit(&verdict)?;
            ensure!(verdict.status != "error", "decomp verification failed");
            Ok(())
        }
        Some("decomp-verify") => {
            let checkout = std::env::current_dir()?;
            let common = Command::new("git").args(["rev-parse", "--git-common-dir"]).output()?;
            ensure!(common.status.success(), "cannot locate shared pipeline root");
            let common = fs::canonicalize(checkout.join(String::from_utf8(common.stdout)?.trim()))?;
            let root = std::env::var_os("SM_PIPELINE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| common.parent().unwrap().join("out/pipeline"));
            let workers = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".into()).parse::<usize>()?;
            let mut request =
                crate::report!({"checkout":checkout,"root":root,"workers":workers,"output":"out/decomp/routine"});
            let mut index = 1;
            while index < args.len() {
                if args[index] == "--accept" {
                    set_option(&mut request, "accept", true)?;
                    index += 1;
                    continue;
                }
                let key = match args[index].as_str() {
                    "--manifest" => "manifest",
                    "--output" => "output",
                    "--captures" => "captures",
                    "--original-rom" => "original_rom",
                    "--fault" => "fault",
                    "--timeout" => "timeout",
                    option => bail!("unknown decomp option: {option}"),
                };
                index += 1;
                let value = args.get(index).context("decomp option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if key == "timeout" { crate::report!(value.parse::<u64>()?) } else { crate::report!(value) },
                )?;
                index += 1;
            }
            let verdict = decomp_job::verify(request.try_into()?)?;
            emit(&verdict)?;
            ensure!(verdict.status != "error", "decomp verification failed");
            Ok(())
        }
        Some("callback-abi") => {
            let request: Value = request()?;
            emit(
                &crate::report!({"source":callback::abi(&request["manifest"].clone().try_into()?,Path::new(family::field(&request,"module")?))?}),
            )
        }
        Some("packed-materialize") if args.len() == 1 => emit(&packed::materialize(request()?)?),
        Some("packed-materialize") => {
            ensure!(args.len() >= 3, "usage: packed-materialize SUBJECT DESTINATION [--member NAME] [--frame N]");
            let checkout = std::env::current_dir()?;
            let mut request = crate::report!({"directory":fs::canonicalize(&args[1])?,"destination":checkout.join(&args[2]),
                "root":std::env::var_os("SM_PIPELINE_ROOT").map(PathBuf::from).unwrap_or(checkout.join("out/pipeline")),
                "workers":std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_|"4".into()).parse::<usize>()?});
            for pair in args[3..].chunks(2) {
                ensure!(pair.len() == 2, "materialize option value missing");
                match pair[0].as_str() {
                    "--member" => set_option(&mut request, "member", &pair[1])?,
                    "--frame" => set_option(&mut request, "frame", pair[1].parse::<u64>()?)?,
                    other => bail!("unknown materialize option {other}"),
                }
            }
            ensure!(request.get("frame").is_none() || request.get("member").is_some(), "--frame requires --member");
            emit(&packed::materialize(request.try_into()?)?)
        }
        Some("benchmark") => benchmark::command(&args[1..]),
        Some("live-generate") => emit(&live::generate(request()?)?),
        Some("live-verify") if args.len() == 1 => {
            let verdict = live::verify(request()?)?;
            emit(&verdict)?;
            ensure!(verdict.status == "mixed-C-Rust-integration", "live verification rejected");
            Ok(())
        }
        Some("live-verify") => {
            let checkout = std::env::current_dir()?;
            let common = Command::new("git").args(["rev-parse", "--git-common-dir"]).output()?;
            ensure!(common.status.success(), "cannot locate shared pipeline root");
            let common = fs::canonicalize(checkout.join(String::from_utf8(common.stdout)?.trim()))?;
            let root = std::env::var_os("SM_PIPELINE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| common.parent().unwrap().join("out/pipeline"));
            let workers = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".into()).parse::<usize>()?;
            let mut request = crate::report!({"checkout":checkout,"root":root,"workers":workers,"accepted":checkout.join("out/port/decompressor-offline/decomp-acceptance.toml")});
            let mut index = 1;
            while index < args.len() {
                if ["--room-plm", "--retain-private-stores", "--capture-active-eprojs"].contains(&args[index].as_str())
                {
                    let key = match args[index].as_str() {
                        "--room-plm" => "room_plm",
                        "--capture-active-eprojs" => "capture_active_eprojs",
                        _ => "retain_private_stores",
                    };
                    set_option(&mut request, key, true)?;
                    index += 1;
                    continue;
                }
                let key = match args[index].as_str() {
                    "--family-verdict" => "family_verdicts",
                    "--dispatcher-verdict" => "dispatcher_verdicts",
                    "--collision-verdict" => "callback_verdicts",
                    "--plm-verdict" => "plm_verdict",
                    "--projectile-verdict" => "projectile_verdict",
                    "--accepted" => "accepted",
                    "--rng-predecessor" => "rng_predecessor",
                    "--output" => "output",
                    "--frames" => "frames",
                    "--live-jobs" => "live_jobs",
                    "--evidence-store" => "evidence_store",
                    option => bail!("unknown live option: {option}"),
                };
                index += 1;
                let value = args.get(index).context("live option value missing")?;
                if key.ends_with("_verdicts") {
                    if request.get(key).is_none() {
                        set_option(&mut request, key, Value::Array(Vec::new()))?;
                    }
                    request[key].as_array_mut().unwrap().push(crate::report!(value));
                } else {
                    set_option(
                        &mut request,
                        key,
                        if ["frames", "live_jobs"].contains(&key) {
                            crate::report!(value.parse::<u64>()?)
                        } else {
                            crate::report!(value)
                        },
                    )?;
                }
                index += 1;
            }
            let verdict = live::verify(request.try_into()?)?;
            emit(&verdict)?;
            ensure!(verdict.status == "mixed-C-Rust-integration", "live verification rejected");
            Ok(())
        }
        Some("decomp-authenticate") if args.len() == 1 => emit(&decomp_acceptance::authenticate(request()?)?),
        Some("decomp-authenticate") if args.len() == 2 => {
            emit(&decomp_acceptance::authenticate(decomp_acceptance::AuthenticateRequest {
                checkout: std::env::current_dir()?,
                pointer: args[1].clone().into(),
            })?)
        }
        Some("packed-capture") if args.len() == 1 => emit(&packed::capture(request()?)?),
        Some("packed-capture-many") if args.len() == 1 => {
            let pointers = packed::capture_many(request()?)?;
            emit(&crate::report!({"subjects":pointers}))
        }
        Some("packed-publish") if args.len() == 1 => emit(&packed::publish(request()?)?),
        Some("packed-subject") if args.len() == 1 => {
            let request: Value = request()?;
            emit(&observations::subject(
                Path::new(request["directory"].as_str().context("directory missing")?),
                Path::new(request["root"].as_str().context("root missing")?),
                request["workers"].as_integer().context("workers missing")? as usize,
                request.get("pointer_sha256").and_then(Value::as_str),
            )?)
        }
        Some("checkpoints") if args.len() == 1 => {
            let report = checkpoints::operate(request()?)?;
            emit(&report)?;
            ensure!(report.status == "PASS", "checkpoint comparison failed");
            Ok(())
        }
        Some("checkpoints") => {
            ensure!(args.len() >= 3, "baseline and candidate paths missing");
            let mut request = crate::report!({"baseline":args[1],"candidate":args[2],"output":"out/sync/report.toml"});
            let mut index = 3;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--start-frame" => "start_frame",
                    "--end-frame" => "end_frame",
                    "--candidate-frame-offset" => "frame_offset",
                    "--out" => "output",
                    option => bail!("unknown checkpoint option: {option}"),
                };
                index += 1;
                let value = args.get(index).context("checkpoint option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if key == "output" { crate::report!(value) } else { crate::report!(value.parse::<i64>()?) },
                )?;
                index += 1;
            }
            let report = checkpoints::operate(request.try_into()?)?;
            emit(&report)?;
            ensure!(report.status == "PASS", "checkpoint comparison failed");
            Ok(())
        }
        Some("lsnes-replay") if args.len() == 1 => emit(&replay::lsnes(request()?)?),
        Some("lsnes-replay") => {
            let mut request = crate::report!({"movie":args[1]});
            let mut index = 2;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--frames" => "frames",
                    "--rom" => "rom",
                    "--out" => "output",
                    "--timeout" => "timeout",
                    option => bail!("unknown replay option: {option}"),
                };
                index += 1;
                let value = args.get(index).context("replay option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if ["frames", "timeout"].contains(&key) {
                        crate::report!(value.parse::<u64>()?)
                    } else {
                        crate::report!(value)
                    },
                )?;
                index += 1;
            }
            emit(&replay::lsnes(request.try_into()?)?)
        }
        Some("mesen-replay") if args.len() == 1 => emit(&replay::mesen(request()?)?),
        Some("mesen-replay") => {
            let mut request = crate::report!({"inputs":args[1]});
            let mut index = 2;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--frames" => "frames",
                    "--out" => "output",
                    option => bail!("unknown replay option: {option}"),
                };
                index += 1;
                let value = args.get(index).context("replay option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if key == "frames" { crate::report!(value.parse::<u64>()?) } else { crate::report!(value) },
                )?;
                index += 1;
            }
            emit(&replay::mesen(request.try_into()?)?)
        }
        Some("plm-fixtures") => {
            let mut checkout = std::env::current_dir()?;
            let mut output = checkout.join("out/port/plm-fixtures");
            let mut manifest = None;
            let mut index = 1;
            while index < args.len() {
                let key = &args[index];
                index += 1;
                let value = PathBuf::from(args.get(index).context("fixture option value missing")?);
                match key.as_str() {
                    "--checkout" => checkout = value,
                    "--output-dir" => output = value,
                    "--manifest" => manifest = Some(value),
                    option => bail!("unknown fixture option: {option}"),
                };
                index += 1;
            }
            emit(&plm_fixtures::generate(&checkout, &output, manifest.as_deref())?)
        }
        Some("lease") if args.len() == 3 => lease(Path::new(&args[1]), args[2].parse()?),
        Some("cache") if args.len() == 1 => cache(framed_request()?),
        Some("wrapper") if args.len() == 1 => emit(&inputs::wrapper(request()?)?),
        Some("dependencies") if args.len() == 1 => emit(&inputs::dependencies(request()?)?),
        Some("c-build") if args.len() == 1 => emit(&builds::c_build(request()?)?),
        Some("exec") if args.len() == 1 => emit(&builds::exec(request()?)?),
        Some("cargo-build") if args.len() == 1 => emit(&builds::cargo_build(request()?)?),
        Some("live-authenticate") if args.len() == 1 => emit(&observations::authenticate(request()?)?),
        Some("live-authenticate") if args.len() == 2 => {
            let checkout = std::env::current_dir()?;
            let common = Command::new("git").args(["rev-parse", "--git-common-dir"]).output()?;
            ensure!(common.status.success(), "cannot locate shared pipeline root");
            let common = fs::canonicalize(checkout.join(String::from_utf8(common.stdout)?.trim()))?;
            let root = std::env::var_os("SM_PIPELINE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| common.parent().unwrap().join("out/pipeline"));
            let workers = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".to_owned()).parse::<usize>()?;
            emit(&observations::authenticate(observations::AuthenticateRequest {
                verdict: args[1].clone().into(),
                root,
                workers,
            })?)
        }
        Some("family-verify") if args.len() == 1 => {
            let verdict = family::verify(request()?)?;
            emit(&verdict)?;
            ensure!(verdict.status == "offline-verified", "family verification rejected");
            Ok(())
        }
        Some("family-authenticate") if args.len() == 1 => emit(&family::authenticate(request()?)?),
        Some("decomp-session") if args.len() == 1 => emit(&session::operate(request()?)?),
        Some("decomp-session") => {
            ensure!(
                args.len() == 4 && args[2] == "--request",
                "usage: sm-pipeline decomp-session OP --request REQUEST.toml"
            );
            let request: session::Request = toml::from_str(&fs::read_to_string(&args[3])?)?;
            ensure!(request.operation() == args[1], "session operation differs from typed request");
            emit(&session::operate(request)?)
        }
        Some("decomp-authority") if args.len() == 1 => emit(&assignment::authority::operate(request()?)?),
        Some("decomp-authority") => {
            ensure!(
                args.len() == 3 && args[1] == "--request",
                "usage: sm-pipeline decomp-authority --request REQUEST.toml"
            );
            emit(&assignment::authority::operate(toml::from_str(&fs::read_to_string(&args[2])?)?)?)
        }
        Some("assign") if args.len() == 1 => emit(&coordination::operate(request()?)?),
        Some("assign") => {
            let workers = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".into()).parse::<usize>()?;
            let mut options =
                crate::report!({"checkout":std::env::current_dir()?,"operation":args[1],"workers":workers});
            if let Some(root) = std::env::var_os("SM_PIPELINE_ROOT") {
                set_option(&mut options, "pipeline_root", PathBuf::from(root))?;
            }
            let mut index = 2;
            while index < args.len() {
                if args[index] == "--no-translations" {
                    set_option(&mut options, "translations", false)?;
                    index += 1;
                    continue;
                }
                let key = match args[index].as_str() {
                    "--checkout" => "checkout",
                    "--effort" => "effort",
                    "--coordinator" => "coordinator",
                    "--authority" => "authority",
                    "--capacity" => "capacity",
                    "--pipeline-root" => "pipeline_root",
                    "--workers" => "workers",
                    "--available-slots" => "available_slots",
                    "--profile" => "profile",
                    "--changed-since" => "changed_since",
                    "--source-inventory" => "source_inventory",
                    "--exercise" => "exercise",
                    "--batch-result" => "batch_result",
                    "--scope" => "scope",
                    "--update" => "update",
                    "--families" => "families",
                    option => bail!("unknown assign option: {option}"),
                };
                index += 1;
                if key == "families" {
                    let mut names = Vec::new();
                    while index < args.len() && !args[index].starts_with("--") {
                        names.push(args[index].clone());
                        index += 1;
                    }
                    ensure!(!names.is_empty(), "families missing");
                    set_option(&mut options, key, names)?;
                } else {
                    let value = args.get(index).context("assign option value missing")?;
                    if key == "update" || key == "authority" {
                        set_option(&mut options, key, toml::from_str::<Value>(&fs::read_to_string(value)?)?)?;
                    } else if ["capacity", "available_slots", "workers"].contains(&key) {
                        set_option(&mut options, key, value.parse::<usize>()?)?;
                    } else {
                        set_option(&mut options, key, value)?;
                    }
                    index += 1;
                }
            }
            emit(&coordination::operate(options.try_into()?)?)
        }
        Some("batch") if args.len() == 1 => {
            let result = batch::operate(request()?)?;
            emit(&result)?;
            ensure!(result.get("status").and_then(Value::as_str) != Some("failed"), "batch failed");
            Ok(())
        }
        Some("batch") => {
            let checkout = std::env::current_dir()?;
            let common = Command::new("git").args(["rev-parse", "--git-common-dir"]).output()?;
            ensure!(common.status.success(), "cannot locate shared pipeline root");
            let common = fs::canonicalize(checkout.join(String::from_utf8(common.stdout)?.trim()))?;
            let root = std::env::var_os("SM_PIPELINE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| common.parent().unwrap().join("out/pipeline"));
            let workers = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".to_owned()).parse::<usize>()?;
            let mut request = crate::report!({"checkout":checkout,"root":root,"workers":workers,"operation":args[1]});
            let mut index = 2;
            while index < args.len() {
                if ["--fresh", "--native-plm", "--room-plm", "--retain-private-stores"].contains(&args[index].as_str())
                {
                    set_option(&mut request, args[index].trim_start_matches("--").replace('-', "_"), true)?;
                    index += 1;
                    continue;
                }
                let key = match args[index].as_str() {
                    "--batch" => "batch",
                    "--profile" => "profile",
                    "--checkout" => "checkout",
                    "--root" => "root",
                    "--jobs" => "jobs",
                    "--case-jobs" => "case_jobs",
                    "--live-jobs" => "live_jobs",
                    "--frames" => "frames",
                    "--accepted" => "accepted",
                    "--changed-since" => "changed_since",
                    "--rng-predecessor" => "rng_predecessor",
                    "--evidence-store" => "evidence_store",
                    "--families" => "families",
                    option => bail!("unknown batch option: {option}"),
                };
                index += 1;
                if key == "families" {
                    let mut names = Vec::new();
                    while index < args.len() && !args[index].starts_with("--") {
                        names.push(args[index].clone());
                        index += 1;
                    }
                    ensure!(!names.is_empty(), "families missing");
                    set_option(&mut request, key, crate::report!(names))?;
                } else {
                    let value = args.get(index).context("batch option value missing")?;
                    set_option(
                        &mut request,
                        key,
                        if ["jobs", "case_jobs", "live_jobs", "frames"].contains(&key) {
                            crate::report!(value.parse::<usize>()?)
                        } else {
                            crate::report!(value)
                        },
                    )?;
                    index += 1;
                }
            }
            let result = batch::operate(request.try_into()?)?;
            emit(&result)?;
            ensure!(result.get("status").and_then(Value::as_str) != Some("failed"), "batch failed");
            Ok(())
        }
        Some("family-verify" | "family-authenticate") => {
            let checkout = std::env::current_dir()?;
            let common = Command::new("git").args(["rev-parse", "--git-common-dir"]).output()?;
            ensure!(common.status.success(), "cannot locate shared pipeline root");
            let common = fs::canonicalize(checkout.join(String::from_utf8(common.stdout)?.trim()))?;
            let root = std::env::var_os("SM_PIPELINE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| common.parent().unwrap().join("out/pipeline"));
            let workers = std::env::var("SM_PIPELINE_WORKERS")
                .unwrap_or_else(|_| {
                    std::thread::available_parallelism().map(|n| n.get().min(4)).unwrap_or(1).to_string()
                })
                .parse::<usize>()?;
            let mut request = crate::report!({"checkout":checkout,"source":checkout.join("deps/snesrev-sm"),"root":root,"workers":workers,"case_jobs":workers});
            let mut index = 1;
            while index < args.len() {
                let key = match args[index].as_str() {
                    "--manifest" => "manifest",
                    "--source" => "source",
                    "--output" => "output",
                    "--checkout" => "checkout",
                    "--root" => "root",
                    "--case-jobs" => "case_jobs",
                    "--verdict" => "verdict",
                    "--adapter" => "adapter",
                    "--execution" => "execution",
                    value => bail!("unknown family option: {value}"),
                };
                index += 1;
                let value = args.get(index).context("family option value missing")?;
                set_option(
                    &mut request,
                    key,
                    if key == "case_jobs" { crate::report!(value.parse::<usize>()?) } else { crate::report!(value) },
                )?;
                index += 1;
            }
            if args[0] == "family-authenticate" {
                emit(&family::authenticate(request.try_into()?)?)
            } else {
                let verdict = family::verify(request.try_into()?)?;
                emit(&verdict)?;
                ensure!(verdict.status == "offline-verified", "family verification rejected");
                Ok(())
            }
        }
        Some("cleanup") if args.len() == 1 => emit(&cleanup::cleanup(request()?)?),
        Some("cleanup") => {
            let mut request = crate::report!({"root":"out"});
            let mut index = 1;
            while index < args.len() {
                match args[index].as_str() {
                    "--root" => {
                        index += 1;
                        set_option(&mut request, "root", args.get(index).context("missing cleanup root")?)?;
                    }
                    "--apply" => set_option(&mut request, "apply", true)?,
                    "--deduplicate" => set_option(&mut request, "deduplicate", true)?,
                    option => bail!("unknown cleanup option: {option}"),
                }
                index += 1;
            }
            emit(&cleanup::cleanup(request.try_into()?)?)
        }
        _ => bail!(
            "usage: sm-pipeline lease ROOT WORKERS | cache | wrapper | dependencies | exec | cargo-build | c-build | cleanup | family-verify | family-authenticate | batch | live-authenticate (TOML request on stdin); see crates/pipeline/README.md for command options"
        ),
    }
}
