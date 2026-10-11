//! Translation-unit reuse; generated candidate units are always rebuilt.
use crate::{CacheRequest, CacheResult, acquire_lease, atomic_toml, cached_artifact};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExecRequest {
    pub command: Vec<String>,
    pub run: PathBuf,
    pub label: String,
    pub root: PathBuf,
    pub workers: usize,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CBuildRequest {
    pub command: Vec<String>,
    pub run: PathBuf,
    pub checkout: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoBuildRequest {
    pub checkout: PathBuf,
    pub root: PathBuf,
    pub run: PathBuf,
    pub package: String,
    pub member: String,
    pub destination: PathBuf,
    pub workers: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ExecutionRecord {
    pub argv: Vec<String>,
    pub seconds: f64,
    pub worker_wait_seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn headers(directory: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let p = entry?.path();
        if p.is_dir() {
            headers(&p, result)?;
        } else if p.extension().and_then(|v| v.to_str()) == Some("h") {
            result.push(p);
        }
    }
    Ok(())
}

pub(crate) fn execute(
    command: &[String],
    run: &Path,
    label: &str,
    root: &Path,
    workers: usize,
) -> Result<ExecutionRecord> {
    execute_timeout(command, run, label, root, workers, 240)
}
pub(crate) fn execute_timeout(
    command: &[String],
    run: &Path,
    label: &str,
    root: &Path,
    workers: usize,
    timeout: u64,
) -> Result<ExecutionRecord> {
    ensure!(!command.is_empty(), "empty process command");
    ensure!(label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'), "unsafe process label");
    fs::create_dir_all(run)?;
    let started = Instant::now();
    let (_held, wait) = acquire_lease(root, workers)?;
    let active = Instant::now();
    let mut child = Command::new(&command[0])
        .args(&command[1..])
        .current_dir(run)
        .stdout(Stdio::from(File::create(run.join(format!("{label}.stdout")))?))
        .stderr(Stdio::from(File::create(run.join(format!("{label}.stderr")))?))
        .spawn()?;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if active.elapsed() > std::time::Duration::from_secs(timeout) {
            child.kill()?;
            child.wait()?;
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
    Ok(record)
}

pub fn exec(request: ExecRequest) -> Result<ExecutionRecord> {
    execute(&request.command, &request.run, &request.label, &request.root, request.workers)
}

fn rust_sources(directory: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            rust_sources(&path, result)?;
        } else if path.extension().is_some_and(|v| v == "rs") {
            result.push(path);
        }
    }
    Ok(())
}

pub fn cargo_build(request: CargoBuildRequest) -> Result<CacheResult> {
    let checkout = fs::canonicalize(request.checkout)?;
    let root = request.root;
    let run = request.run;
    let package = request.package.as_str();
    let member = request.member.as_str();
    ensure!(
        (package == "evidence" && member == "sm-evidence") || (package == "decomp" && member == "libdecomp.a"),
        "Cargo input closure is defined only for evidence and decomp"
    );
    let mut inputs = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"].map(|p| checkout.join(p)).to_vec();
    inputs.push(checkout.join(format!("crates/{package}/Cargo.toml")));
    rust_sources(&checkout.join(format!("crates/{package}/src")), &mut inputs)?;
    for entry in fs::read_dir(checkout.join("crates"))? {
        let manifest = entry?.path().join("Cargo.toml");
        if manifest.is_file() {
            inputs.push(manifest);
        }
    }
    let config = checkout.join(".cargo");
    if config.is_dir() {
        for entry in fs::read_dir(config)? {
            let p = entry?.path();
            if p.is_file() {
                inputs.push(p);
            }
        }
    }
    let external = checkout.join("deps/rustysnes/crates/rustysnes-core/Cargo.toml");
    if external.exists() {
        inputs.push(external);
    }
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
            inputs,
            destination: request.destination,
        },
        |artifact| {
            let target = artifact.parent().unwrap().join("target");
            let mut actual = command.clone();
            actual.extend(["--jobs".into(), "1".into(), "--target-dir".into(), target.to_string_lossy().into_owned()]);
            // Cargo's working directory must select this checkout, while logs stay in the run.
            fs::create_dir_all(&run)?;
            let (_held, _) = acquire_lease(&root, workers)?;
            let status = Command::new(&actual[0])
                .args(&actual[1..])
                .current_dir(&checkout)
                .stdout(File::create(run.join(format!("{package}-build.stdout")))?)
                .stderr(File::create(run.join(format!("{package}-build.stderr")))?)
                .status()?;
            ensure!(status.success(), "{package} build failed: {status}");
            fs::copy(target.join("release").join(member), artifact)?;
            fs::remove_dir_all(target)?;
            Ok(())
        },
    )
}

pub fn c_build(request: CBuildRequest) -> Result<ExecutionRecord> {
    let CBuildRequest { run, checkout, root, workers, command } = request;
    ensure!(!command.is_empty(), "empty C command");
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
    let mut records = Vec::new();
    let mut objects = Vec::new();
    for (index, source) in sources.iter().enumerate() {
        let source = fs::canonicalize(if source.is_absolute() { source.clone() } else { run.join(source) })?;
        let destination = run.join(format!("c-object-{index:03}.o"));
        let mut compile = vec![command[0].clone()];
        compile.extend(flags.clone());
        compile.extend(["-c".into(), source.to_string_lossy().into_owned(), "-o".into(), "{artifact}".into()]);
        let build = |artifact: &Path| {
            let mut actual = compile.clone();
            *actual.last_mut().unwrap() = artifact.to_string_lossy().into_owned();
            execute(&actual, &run, &format!("c-object-{index:03}"), &root, workers).map(|_| ())
        };
        if source.starts_with(&pinned) {
            // Ask the compiler for its complete preprocessor closure, including
            // system headers and textually included sources. Header edits and
            // changed include search paths must invalidate even a warm object.
            let (_held, _) = acquire_lease(&root, workers)?;
            let dependencies =
                Command::new(&command[0]).args(&flags).arg("-M").arg(&source).current_dir(&run).output()?;
            drop(_held);
            ensure!(
                dependencies.status.success(),
                "C dependency scan failed: {}",
                String::from_utf8_lossy(&dependencies.stderr)
            );
            let mut closure = inputs.clone();
            for name in crate::inputs::dep_paths(&String::from_utf8(dependencies.stdout)?)? {
                let p = PathBuf::from(name);
                closure.push(if p.is_absolute() { p } else { run.join(p) });
            }
            closure.push(source);
            records.push(cached_artifact(
                &CacheRequest {
                    root: root.clone(),
                    checkout: checkout.clone(),
                    command: compile.clone(),
                    inputs: closure,
                    destination: destination.clone(),
                },
                build,
            )?);
        } else {
            build(&destination)?;
        }
        objects.push(destination);
    }
    let source_names: Vec<_> = sources.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    let mut linked: Vec<String> = command.into_iter().filter(|arg| !source_names.contains(arg)).collect();
    linked.splice(1..1, objects.iter().map(|p| p.to_string_lossy().into_owned()));
    let result = execute(&linked, &run, "c-link", &root, workers);
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
