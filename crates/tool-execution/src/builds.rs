//! Authenticated compilation and execution; callers supply their fixed recipes.
use crate::{
    acquire_lease, atomic_toml, bounded,
    build_cache::{CacheRequest, CacheResult, Preparation, cached_artifact},
};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Instant,
};
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecRequest {
    pub command: Vec<String>,
    pub run: PathBuf,
    pub label: String,
    pub root: PathBuf,
    pub workers: usize,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub sampler_paths: Vec<SamplerPath>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct SamplerPath {
    pub path: PathBuf,
    pub kind: crate::resource_sampler::PathKind,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CBuildRequest {
    pub command: Vec<String>,
    pub run: PathBuf,
    pub checkout: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
}
fn default_timeout() -> u64 {
    240
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CargoBuildRequest {
    pub checkout: PathBuf,
    pub root: PathBuf,
    pub run: PathBuf,
    pub package: String,
    pub member: String,
    pub destination: PathBuf,
    pub workers: usize,
    /// Complete input closure selected by the application owning this artifact.
    pub inputs: Vec<PathBuf>,
}

/// One fixed standalone output selected by its application owner.
pub struct StandaloneArtifact {
    pub source: PathBuf,
    pub name: String,
    pub label: String,
    pub flags: Vec<String>,
}
/// At most three standalone outputs share one frozen compiler preparation.
pub struct StandaloneBuildRequest {
    pub checkout: PathBuf,
    pub root: PathBuf,
    pub run: PathBuf,
    pub workers: usize,
    pub artifacts: Vec<StandaloneArtifact>,
}
pub fn standalone_builds(request: StandaloneBuildRequest) -> Result<Vec<CacheResult>> {
    let StandaloneBuildRequest { checkout, root, run, workers, artifacts: recipes } = request;
    ensure!(!recipes.is_empty() && recipes.len() <= 3, "invalid standalone build extent");
    fs::create_dir_all(&run)?;
    let checkout = fs::canonicalize(checkout)?;
    // Resolve the project's selected toolchain once. A run outside checkout
    // must not silently choose a different rustup override for scan or build.
    execute_at(
        &["rustc".into(), "--print".into(), "sysroot".into()],
        &run,
        &checkout,
        "helper-toolchain",
        &root,
        workers,
        240,
    )?;
    let sysroot = fs::read_to_string(run.join("helper-toolchain.stdout"))?;
    let compiler = fs::canonicalize(Path::new(sysroot.trim()).join("bin/rustc"))?;
    let compiler = compiler.display().to_string();
    let preparation = Preparation::new(&root, &checkout, &compiler)?;
    let mut artifacts = Vec::new();
    for recipe in recipes {
        let StandaloneArtifact { source, name, label, flags } = recipe;
        let source = checkout.join(source);
        preparation.freeze(std::slice::from_ref(&source))?;
        let mut command = vec![compiler.clone(), "--edition=2024".into(), "-O".into()];
        command.extend(flags);
        command.push(source.display().to_string());
        // Always rediscover the actual compiler closure, including include! and
        // cfg-dependent modules. A cache hit must not hide a newly added input.
        let dep_file = run.join(format!("{label}.compiler.d"));
        let mut dependency = command.clone();
        dependency.push(format!("--emit=dep-info={}", dep_file.display()));
        execute(&dependency, &run, &format!("{label}-dependencies"), &root, workers)?;
        let dependency_text = fs::read_to_string(&dep_file)?;
        // These standalone recipes currently use no env!/option_env! input.
        // Fail closed if that changes instead of hiding an arbitrary variable
        // behind the cache's established compiler-environment allowlist.
        ensure!(
            !dependency_text.lines().any(|line| line.starts_with("# env-dep:")),
            "standalone helper has an unsupported build-time environment dependency"
        );
        let mut inputs: Vec<_> = dep_paths(&dependency_text)?
            .into_iter()
            .map(|path| {
                let path = PathBuf::from(path);
                if path.is_absolute() { path } else { run.join(path) }
            })
            .collect();
        inputs.push(source);
        command.extend(["-o".into(), "{artifact}".into()]);
        let result = preparation.artifact(&command, &inputs, &run.join(name), |artifact| {
            let mut actual = command.clone();
            *actual.last_mut().unwrap() = artifact.display().to_string();
            let actual_dependencies = artifact.with_extension("d");
            actual.push(format!("--emit=link,dep-info={}", actual_dependencies.display()));
            execute(&actual, &run, &label, &root, workers)?;
            let actual_text = fs::read_to_string(&actual_dependencies)?;
            ensure!(
                !actual_text.lines().any(|line| line.starts_with("# env-dep:")),
                "standalone helper has an unsupported build-time environment dependency"
            );
            authenticate_dependency_closure(&inputs, &actual_text, &run)?;
            fs::remove_file(actual_dependencies)?;
            Ok(())
        })?;
        atomic_toml(&run.join(format!("{label}.cache.toml")), &result)?;
        artifacts.push(result);
    }
    preparation.finish()?;
    Ok(artifacts)
}

fn dependency_paths(text: &str, run: &Path) -> Result<Vec<PathBuf>> {
    dep_paths(text)?
        .into_iter()
        .map(|name| {
            let path = PathBuf::from(name);
            Ok(fs::canonicalize(if path.is_absolute() { path } else { run.join(path) })?)
        })
        .collect()
}

fn authenticate_dependency_closure(expected: &[PathBuf], actual: &str, run: &Path) -> Result<()> {
    let expected: std::collections::BTreeSet<_> =
        expected.iter().map(fs::canonicalize).collect::<std::io::Result<_>>()?;
    let actual: std::collections::BTreeSet<_> = dependency_paths(actual, run)?.into_iter().collect();
    ensure!(expected == actual, "compiler dependency closure changed during build");
    Ok(())
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExecutionRecord {
    pub argv: Vec<String>,
    pub seconds: f64,
    pub worker_wait_seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn headers(directory: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    source_files(directory, "h", result)
}

pub fn execute(command: &[String], run: &Path, label: &str, root: &Path, workers: usize) -> Result<ExecutionRecord> {
    execute_timeout(command, run, label, root, workers, 240)
}
pub fn execute_timeout(
    command: &[String],
    run: &Path,
    label: &str,
    root: &Path,
    workers: usize,
    timeout: u64,
) -> Result<ExecutionRecord> {
    execute_at(command, run, run, label, root, workers, timeout)
}

fn execute_at(
    command: &[String],
    run: &Path,
    cwd: &Path,
    label: &str,
    root: &Path,
    workers: usize,
    timeout: u64,
) -> Result<ExecutionRecord> {
    execute_at_checked(command, run, cwd, label, root, workers, timeout, || Ok(()))
}

fn execute_at_checked(
    command: &[String],
    run: &Path,
    cwd: &Path,
    label: &str,
    root: &Path,
    workers: usize,
    timeout: u64,
    ready: impl FnOnce() -> Result<()>,
) -> Result<ExecutionRecord> {
    ensure!(!command.is_empty(), "empty process command");
    ensure!(label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'), "unsafe process label");
    fs::create_dir_all(run)?;
    let mut metrics = crate::metrics::Run::start(run.join(format!("{label}.metrics.toml")), label)?;
    let result = (|| -> Result<ExecutionRecord> {
        let started = Instant::now();
        let waiting = metrics.context.stage("worker-wait")?;
        let (_held, wait) = acquire_lease(root, workers)?;
        waiting.finish("success")?;
        ready()?;
        let building = metrics.context.stage("execute")?;
        let active = Instant::now();
        let mut actual = Command::new(&command[0]);
        actual
            .args(&command[1..])
            .current_dir(cwd)
            .stdout(Stdio::from(File::create(run.join(format!("{label}.stdout")))?))
            .stderr(Stdio::from(File::create(run.join(format!("{label}.stderr")))?));
        let mut child = crate::metrics::OwnedChild::spawn(&mut actual, &metrics.context, label)?;
        let status = loop {
            if let Some(status) = child.poll()? {
                break status;
            }
            if active.elapsed() > std::time::Duration::from_secs(timeout) {
                child.kill_wait()?;
                atomic_toml(
                    &run.join(format!("{label}.timing.toml")),
                    &ExecutionRecord {
                        argv: command.to_vec(),
                        seconds: started.elapsed().as_secs_f64(),
                        worker_wait_seconds: wait,
                        exit_code: None,
                        error: Some("compiler timeout".into()),
                    },
                )?;
                bail!("{label} timeout after {timeout}s");
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        let record = ExecutionRecord {
            argv: command.to_vec(),
            seconds: started.elapsed().as_secs_f64(),
            worker_wait_seconds: wait,
            exit_code: status.code(),
            error: None,
        };
        atomic_toml(&run.join(format!("{label}.timing.toml")), &record)?;
        ensure!(status.success(), "{label} failed: {status}");
        building.finish("success")?;
        Ok(record)
    })();
    let error = result.as_ref().err().map(|e| format!("{e:#}"));
    metrics.finish(crate::metrics::outcome(error.as_deref()), error)?;
    result
}

pub fn exec(request: ExecRequest) -> Result<ExecutionRecord> {
    let mut metrics = crate::metrics::Run::start(request.run.join(format!("{}.metrics.toml", request.label)), "exec")?;
    for path in request.sampler_paths {
        metrics.context.add_path(path.path, path.kind);
    }
    metrics.context.sample_disk();
    let result = execute_timeout(
        &request.command,
        &request.run,
        &request.label,
        &request.root,
        request.workers,
        request.timeout,
    );
    let error = result.as_ref().err().map(|e| format!("{e:#}"));
    let finish = metrics.finish(crate::metrics::outcome(error.as_deref()), error);
    if result.is_ok() {
        finish?;
    }
    result
}

pub fn source_files(directory: &Path, extension: &str, result: &mut Vec<PathBuf>) -> Result<()> {
    bounded::limited(|| {
        let mut pending = vec![(directory.to_path_buf(), 0usize)];
        while let Some((directory, depth)) = pending.pop() {
            bounded::check()?;
            ensure!(depth <= 32, "compiler source discovery depth exceeded");
            for path in bounded::entries(&directory)? {
                bounded::safe(&path)?;
                let metadata = path.symlink_metadata()?;
                if metadata.is_dir() {
                    pending.push((path, depth + 1));
                } else if path.extension().is_some_and(|value| value == extension) {
                    ensure!(metadata.is_file(), "compiler source is not a regular file");
                    ensure!(result.len() < 4096, "compiler source discovery file count exceeded");
                    bounded::account(metadata.len())?;
                    result.push(path);
                }
            }
        }
        Ok(())
    })
}

/// Build one application-selected Cargo artifact inside the caller's accounting
/// scope. The caller also accounts for selecting and discovering its inputs.
pub fn cargo_build(request: CargoBuildRequest, metrics: &crate::metrics::Context) -> Result<CacheResult> {
    let _entered = metrics.enter();
    let checkout = fs::canonicalize(request.checkout)?;
    let root = request.root;
    let run = request.run;
    let package = request.package.as_str();
    let member = request.member.as_str();
    let command = vec![
        "cargo".to_owned(),
        "build".into(),
        "--locked".into(),
        "--offline".into(),
        "--release".into(),
        "-p".into(),
        package.into(),
    ];
    let workers = request.workers;
    cached_artifact(
        &CacheRequest {
            root: root.clone(),
            checkout: checkout.clone(),
            command: command.clone(),
            inputs: request.inputs,
            destination: request.destination,
        },
        |artifact| {
            let target = artifact.parent().unwrap().join("target");
            let mut actual = command.clone();
            actual.extend(["--jobs".into(), "1".into(), "--target-dir".into(), target.to_string_lossy().into_owned()]);
            // Cargo's working directory must select this checkout, while logs stay in the run.
            fs::create_dir_all(&run)?;
            let waiting = metrics.stage("cargo-worker-wait")?;
            let (_held, _) = acquire_lease(&root, workers)?;
            waiting.finish("success")?;
            let mut command = Command::new(&actual[0]);
            command
                .args(&actual[1..])
                .current_dir(&checkout)
                .stdout(File::create(run.join(format!("{package}-build.stdout")))?)
                .stderr(File::create(run.join(format!("{package}-build.stderr")))?);
            let status = metrics.status(&mut command, "cargo-build")?;
            ensure!(status.success(), "{package} build failed: {status}");
            fs::copy(target.join("release").join(member), artifact)?;
            fs::remove_dir_all(target)?;
            Ok(())
        },
    )
}

pub fn c_build(request: CBuildRequest) -> Result<ExecutionRecord> {
    c_build_with_inputs(request, &|_| Ok(()))
}

/// Compile the application's fixed C recipe and admit every compiler-discovered
/// input through its policy. All units, including uncached generated units,
/// receive fresh discovery and emitted-closure equality checks. The validator
/// must be read-only and thread safe; it is also called after capacity waits.
/// Compiler/header byte identities, cache locks and artifact authentication
/// remain owned by this module rather than delegated to the validator.
pub fn c_build_with_inputs(
    request: CBuildRequest,
    validate_inputs: &(dyn Fn(&[PathBuf]) -> Result<()> + Sync),
) -> Result<ExecutionRecord> {
    fs::create_dir_all(&request.run)?;
    // Install cancellation once on the parent. Thread-local child scopes then
    // inherit this context rather than racing to replace signal handlers.
    let mut metrics = crate::metrics::Run::start(request.run.join("c-build.metrics.toml"), "c-build")?;
    metrics.context.configuration("workers", request.workers);
    let result = bounded::limited(|| c_build_prepared(request, validate_inputs));
    let error = result.as_ref().err().map(|error| format!("{error:#}"));
    let finished = metrics.finish(crate::metrics::outcome(error.as_deref()), error);
    if result.is_ok() {
        finished?;
    }
    result
}

fn c_build_prepared(
    request: CBuildRequest,
    validate_inputs: &(dyn Fn(&[PathBuf]) -> Result<()> + Sync),
) -> Result<ExecutionRecord> {
    let CBuildRequest { run, checkout, root, workers, command } = request;
    ensure!(!command.is_empty(), "empty C command");
    ensure!((1..=64).contains(&workers), "invalid C worker count");
    let mut flags = Vec::new();
    let mut sources = Vec::new();
    let mut inputs = Vec::new();
    let mut i = 1;
    while i < command.len() {
        let arg = &command[i];
        if arg.ends_with(".c") {
            sources.push(PathBuf::from(arg));
        } else if ["-I", "-iquote", "-isystem"].contains(&arg.as_str()) {
            let include = command.get(i + 1).context("include directory missing")?;
            headers(Path::new(include), &mut inputs)?;
            flags.extend([arg.clone(), include.clone()]);
            i += 1;
        } else if arg == "-o" {
            ensure!(i + 1 < command.len(), "output missing");
            i += 1;
        } else if arg.starts_with("-Wl,") || arg.starts_with("-l") || arg.ends_with(".a") || arg.ends_with(".o") {
        } else if arg.starts_with("-O")
            || arg.starts_with("-f")
            || arg.starts_with("-D")
            || arg.starts_with("-U")
            || arg.starts_with("-std=")
            || arg.starts_with("-m")
            || arg.starts_with("-W")
            || arg == "-pthread"
            || arg == "-g"
        {
            flags.push(arg.clone());
        } else {
            bail!("unsupported C build option: {arg}");
        }
        i += 1;
    }
    ensure!(!sources.is_empty(), "C build has no translation units");
    let pinned = fs::canonicalize(checkout.join("deps"))?;
    let preparation = Preparation::new(&root, &checkout, &command[0])?;
    validate_inputs(&inputs)?;
    preparation.freeze(&inputs)?;
    let objects: Vec<_> = (0..sources.len()).map(|index| run.join(format!("c-object-{index:03}.o"))).collect();
    let compile_unit = |index: usize| -> Result<Option<CacheResult>> {
        let source = &sources[index];
        let source = fs::canonicalize(if source.is_absolute() { source.clone() } else { run.join(source) })?;
        let destination = &objects[index];
        let mut compile = vec![command[0].clone()];
        compile.extend(flags.clone());
        compile.extend(["-c".into(), source.to_string_lossy().into_owned(), "-o".into(), "{artifact}".into()]);
        preparation.freeze(std::slice::from_ref(&source))?;
        validate_inputs(std::slice::from_ref(&source))?;
        // Discover every actual preprocessor closure. A generated wrapper can
        // include pinned source, system headers, or macro-selected paths too.
        let (_held, _) = acquire_lease(&root, workers)?;
        preparation.validate(std::slice::from_ref(&source))?;
        preparation.validate(&inputs)?;
        validate_inputs(std::slice::from_ref(&source))?;
        let dependencies = crate::metrics::command_output(
            Command::new(&command[0]).args(&flags).arg("-M").arg(&source).current_dir(&run),
            "c-dependency-scan",
        )?;
        drop(_held);
        ensure!(
            dependencies.status.success(),
            "C dependency scan failed: {}",
            String::from_utf8_lossy(&dependencies.stderr)
        );
        let discovered = dependency_paths(&String::from_utf8(dependencies.stdout)?, &run)?;
        validate_inputs(&discovered)?;
        let mut closure = inputs.clone();
        closure.extend(discovered.clone());
        closure.push(source.clone());
        closure.sort();
        closure.dedup();
        preparation.freeze(&closure)?;
        preparation.validate(&closure)?;
        let build = |artifact: &Path| -> Result<()> {
            let mut actual = compile.clone();
            *actual.last_mut().unwrap() = artifact.to_string_lossy().into_owned();
            let dep_file = artifact.with_extension("d");
            actual.extend(["-MD".into(), "-MF".into(), dep_file.display().to_string()]);
            execute_at_checked(&actual, &run, &run, &format!("c-object-{index:03}"), &root, workers, 240, || {
                preparation.validate(&closure)?;
                validate_inputs(&closure)
            })?;
            let actual = fs::read_to_string(&dep_file)?;
            authenticate_dependency_closure(&discovered, &actual, &run)?;
            validate_inputs(&dependency_paths(&actual, &run)?)?;
            preparation.validate(&closure)?;
            fs::remove_file(dep_file)?;
            Ok(())
        };
        if source.starts_with(&pinned) {
            let result = preparation.artifact(&compile, &closure, destination, build)?;
            validate_inputs(&closure)?;
            Ok(Some(result))
        } else {
            build(destination)?;
            Ok(None)
        }
    };
    // A fixed number of local workers feed the same cross-process leases.
    // Waiting on one cache key cannot prevent unrelated objects progressing.
    let next = AtomicUsize::new(0);
    let failed = AtomicBool::new(false);
    let context = crate::metrics::active();
    let input_scope = bounded::shared_scope()?;
    let mut completed = std::thread::scope(|scope| -> Result<Vec<_>> {
        let handles: Vec<_> = (0..workers.min(sources.len()))
            .map(|_| {
                scope.spawn(|| {
                    let _active = context.as_ref().map(|context| context.enter());
                    let mut completed = Vec::new();
                    while !failed.load(Ordering::Acquire) {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        if index >= sources.len() {
                            break;
                        }
                        let result = input_scope.enter(|| compile_unit(index));
                        if result.is_err() {
                            preparation.abort();
                            failed.store(true, Ordering::Release);
                        }
                        completed.push((index, result));
                    }
                    completed
                })
            })
            .collect();
        let mut completed = Vec::new();
        for handle in handles {
            completed.extend(handle.join().map_err(|_| anyhow::anyhow!("C build worker panicked"))?);
        }
        Ok(completed)
    })?;
    completed.sort_by_key(|(index, _)| *index);
    // A lower-index sibling cancelled during a lock wait must not hide the
    // compiler diagnostic that actually failed. Preserve source-order errors.
    if let Some(index) = completed.iter().position(|(_, result)| {
        result.as_ref().err().is_some_and(|error| !error.is::<crate::build_cache::PreparationAborted>())
    }) {
        return Err(completed.remove(index).1.unwrap_err());
    }
    let records: Vec<_> =
        completed.into_iter().map(|(_, result)| result).collect::<Result<Vec<_>>>()?.into_iter().flatten().collect();
    preparation.finish()?;
    let source_names: Vec<_> = sources.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    let mut linked: Vec<String> = command.into_iter().filter(|arg| !source_names.contains(arg)).collect();
    linked.splice(1..1, objects.iter().map(|p| p.to_string_lossy().into_owned()));
    let result = execute(&linked, &run, "c-link", &root, workers);
    preparation.finish()?;
    if result.is_ok() {
        for p in objects {
            fs::remove_file(p)?;
        }
    }
    #[derive(Serialize)]
    struct CacheRecords {
        objects: Vec<CacheResult>,
    }
    atomic_toml(&run.join("c-cache.toml"), &CacheRecords { objects: records })?;
    result
}

pub fn dep_paths(text: &str) -> Result<Vec<String>> {
    bounded::check()?;
    ensure!(text.len() as u64 <= bounded::MAX_DOCUMENT_BYTES, "compiler dependency rule extent exceeded");
    let joined = text.replace("\\\n", "");
    let rule =
        joined.split_once(": ").context("invalid compiler dependency rule")?.1.lines().next().unwrap_or_default();
    let mut result = Vec::new();
    let mut token = String::new();
    let mut escaped = false;
    for c in rule.chars() {
        if escaped {
            token.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c.is_whitespace() {
            if !token.is_empty() {
                result.push(std::mem::take(&mut token));
                ensure!(result.len() <= 65536, "compiler dependency path count exceeded");
                bounded::check()?;
            }
        } else {
            token.push(c);
        }
    }
    ensure!(!escaped, "truncated compiler dependency escape");
    if !token.is_empty() {
        result.push(token);
    }
    ensure!(result.len() <= 65536, "compiler dependency path count exceeded");
    bounded::check()?;
    Ok(result)
}
