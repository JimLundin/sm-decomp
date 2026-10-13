//! Translation-unit reuse; generated candidate units are always rebuilt.
use crate::inputs::bounded;
use crate::{CacheRequest, CacheResult, acquire_lease, atomic_toml, cached_artifact};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use translation_execution::{
    OperationContext,
    confinement::{self, Program, ReadOnlyBind, RunLimits},
};

/// Internal installed execution context. Legacy callers do not acquire this
/// capability. Every compiler and fixture receives a separate bounded /work;
/// no writable verifier directory is mounted into candidate execution.
pub(crate) struct StrictExecution {
    pub helper: PathBuf,
    pub rustc: PathBuf,
    pub c_compiler: PathBuf,
    pub compiler_runtime: Vec<ReadOnlyBind>,
    pub operation_root: PathBuf,
    pub context: OperationContext,
    pub root: PathBuf,
    pub workers: usize,
    pub compiler_limits: RunLimits,
    pub fixture_limits: RunLimits,
    pub rust_inputs: Vec<PathBuf>,
    pub c_inputs: Vec<PathBuf>,
    pub deadline_unix_ms: u64,
    /// Entire gate reservation includes live quotas, durable results and the
    /// parent copies. Reserving before launch bounds peak production bytes.
    pub max_output_bytes: u64,
    pub reserved_bytes: RefCell<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StrictInvocation {
    pub context: OperationContext,
    pub request_sha256: String,
    pub program: Program,
    pub requested_limits: RunLimits,
    pub limits: RunLimits,
    pub reserved_bytes: u64,
}

impl StrictExecution {
    pub(crate) fn compile(
        &self,
        command: &[String],
        run: &Path,
        label: &str,
        checkout: &Path,
        source: &Path,
    ) -> Result<ExecutionRecord> {
        let rust = command.first().is_some_and(|name| name == "rustc");
        let executable = if rust { self.rustc.clone() } else { self.c_compiler.clone() };
        let mut inputs = Vec::new();
        for path in if rust { &self.rust_inputs } else { &self.c_inputs } {
            let relative = path.strip_prefix(if rust { checkout } else { source })?;
            inputs.push(ReadOnlyBind {
                source: path.clone(),
                destination: Path::new(if rust { "/inputs/rust" } else { "/inputs/c" }).join(relative),
            });
        }
        let generated = if rust { vec!["wrapper.rs", "memory.rs"] } else { vec!["oracle.c"] };
        for name in generated {
            inputs
                .push(ReadOnlyBind { source: run.join(name), destination: Path::new("/inputs/generated").join(name) });
        }
        let outputs = if rust { vec!["rust".into(), "compiler.d".into()] } else { vec!["c".into()] };
        let mut arguments = Vec::new();
        for argument in &command[1..] {
            let mapped = if argument.starts_with("--emit=link,dep-info=") {
                "--emit=link,dep-info=/work/compiler.d".to_owned()
            } else {
                let path = Path::new(argument);
                if let Ok(relative) = path.strip_prefix(run) {
                    if ["rust", "c", "compiler.d"].iter().any(|name| relative == Path::new(name)) {
                        Path::new("/work").join(relative).display().to_string()
                    } else {
                        Path::new("/inputs/generated").join(relative).display().to_string()
                    }
                } else if let Ok(relative) = path.strip_prefix(source) {
                    Path::new("/inputs/c").join(relative).display().to_string()
                } else if let Ok(relative) = path.strip_prefix(checkout) {
                    Path::new("/inputs/rust").join(relative).display().to_string()
                } else {
                    argument.clone()
                }
            };
            arguments.push(mapped);
        }
        if rust {
            arguments.extend([
                "--sysroot".into(),
                "/toolchain".into(),
                "-C".into(),
                "linker=/usr/bin/cc".into(),
                "-C".into(),
                "link-arg=-B/usr/lib/gcc/x86_64-linux-gnu/9/".into(),
                "-C".into(),
                "link-arg=-B/usr/bin/".into(),
                "-C".into(),
                "linker-features=-lld".into(),
            ]);
        } else {
            // GCC normally locates subprocesses relative to argv[0] or PATH.
            // /program is deliberately not the host compiler path.
            arguments.extend(["-B/usr/lib/gcc/x86_64-linux-gnu/9/".into(), "-B/usr/bin/".into()]);
        }
        self.execute(
            &Program {
                executable,
                arguments,
                inputs,
                runtime: self.compiler_runtime.clone(),
                helper: self.helper.clone(),
                outputs,
            },
            &self.compiler_limits,
            run,
            label,
        )
    }

    pub(crate) fn fixture(&self, command: &[String], directory: &Path, subject: &str) -> Result<ExecutionRecord> {
        ensure!(command.len() == 6 || command.len() == 7, "confined plain fixture argument contract differs");
        let executable = PathBuf::from(&command[0]);
        let mut arguments = command[1..4].to_vec();
        arguments.extend(["/inputs/before.wram".into(), "/work/after.wram".into()]);
        let mut inputs =
            vec![ReadOnlyBind { source: PathBuf::from(&command[4]), destination: "/inputs/before.wram".into() }];
        if command.len() == 7 {
            inputs.push(ReadOnlyBind { source: PathBuf::from(&command[6]), destination: "/inputs/rom.bin".into() });
            arguments.push("/inputs/rom.bin".into());
        }
        self.execute(
            &Program {
                executable,
                arguments,
                inputs,
                runtime: confinement::helper_runtime()?,
                helper: self.helper.clone(),
                outputs: vec!["after.wram".into()],
            },
            &self.fixture_limits,
            directory,
            subject,
        )
    }

    fn execute(&self, program: &Program, limits: &RunLimits, run: &Path, label: &str) -> Result<ExecutionRecord> {
        let started = Instant::now();
        let mut context = self.context.clone();
        context.operation_key = translation_execution::model::sealed(&(&self.context.operation_key, run, label))?;
        let request_sha256 = translation_execution::model::sealed(&(program, limits))?;
        let invocation_path = run.join(format!("{label}.invocation.toml"));
        let resumed = invocation_path.is_file();
        if !resumed {
            self.check_deadline()?;
        }
        let (_held, wait) = if resumed {
            (None, 0.0)
        } else {
            let (held, wait) = self.lease()?;
            (Some(held), wait)
        };
        let invocation = if resumed {
            let saved: StrictInvocation = crate::family::object(&invocation_path)?;
            ensure!(
                saved.context == context
                    && saved.request_sha256 == request_sha256
                    && saved.program == *program
                    && saved.requested_limits == *limits,
                "native invocation retried with changed inputs"
            );
            saved
        } else {
            self.check_deadline()?;
            let reserved_bytes =
                translation_execution::operation::reserve_bytes_until(program, limits, 0, self.deadline_unix_ms)?
                    .checked_add(limits.writable_bytes)
                    .and_then(|n| n.checked_add(64 * 1024))
                    .context("native producer reservation overflow")?;
            let remaining = self
                .deadline_unix_ms
                .saturating_sub(u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?);
            ensure!(remaining > 0, "native family aggregate time budget exhausted");
            let mut actual_limits = limits.clone();
            actual_limits.timeout = actual_limits.timeout.min(Duration::from_millis(remaining));
            let invocation = StrictInvocation {
                context: context.clone(),
                request_sha256,
                program: program.clone(),
                requested_limits: limits.clone(),
                limits: actual_limits,
                reserved_bytes,
            };
            self.reserve(invocation.reserved_bytes)?;
            atomic_toml(&invocation_path, &invocation)?;
            invocation
        };
        // Completed invocations consume the same reserved slot on recovery.
        // They never launch again and never rewrite sealed input metadata.
        if resumed {
            self.reserve(invocation.reserved_bytes)?;
        }
        let operation_directory = translation_execution::operation::directory(&self.operation_root, &context)?;
        if resumed && !operation_directory.join("intent.json").exists() {
            bail!("native invocation was interrupted before durable launch; explicit operator retry required");
        }
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
        let output = if now >= self.deadline_unix_ms {
            ensure!(
                operation_directory.join("intent.json").is_file(),
                "native family aggregate time budget exhausted before launch"
            );
            if !operation_directory.join("result.json").exists() {
                translation_execution::operation::stop(&self.operation_root, &context)?;
            }
            translation_execution::operation::authenticate_request(
                &self.operation_root,
                &context,
                program,
                &invocation.limits,
                &[],
            )?
        } else {
            translation_execution::operation::run_until(
                &self.operation_root,
                &context,
                program,
                &invocation.limits,
                &[],
                self.deadline_unix_ms,
            )?
        };
        let publish = || -> Result<ExecutionRecord> {
            immutable_output(&run.join(format!("{label}.stdout")), &output.stdout)?;
            immutable_output(&run.join(format!("{label}.stderr")), &output.stderr)?;
            if !output.diagnostic.is_empty() {
                immutable_output(&run.join(format!("{label}.diagnostic.txt")), output.diagnostic.as_bytes())?;
            }
            for (name, bytes) in output.outputs {
                let destination = if name == Path::new("after.wram") {
                    run.join(format!("{label}-after.wram"))
                } else {
                    run.join(name)
                };
                immutable_output(&destination, &bytes)?;
                #[cfg(unix)]
                if destination.file_name().is_some_and(|name| name == "rust" || name == "c") {
                    use std::os::unix::fs::PermissionsExt;
                    if fs::metadata(&destination)?.permissions().mode() & 0o777 != 0o500 {
                        fs::set_permissions(&destination, fs::Permissions::from_mode(0o500))?;
                    }
                }
            }
            let record = ExecutionRecord {
                argv: std::iter::once(program.executable.display().to_string())
                    .chain(program.arguments.clone())
                    .collect(),
                seconds: started.elapsed().as_secs_f64(),
                worker_wait_seconds: wait,
                exit_code: output.status_raw.and_then(|raw| {
                    use std::os::unix::process::ExitStatusExt;
                    std::process::ExitStatus::from_raw(raw).code()
                }),
                error: (!output.passed).then(|| output.diagnostic.clone()),
            };
            let timing = run.join(format!("{label}.timing.toml"));
            if timing.is_file() {
                let saved: ExecutionRecord = crate::family::object(&timing)?;
                ensure!(
                    saved.argv == record.argv && saved.exit_code == record.exit_code && saved.error == record.error,
                    "native retained execution record differs"
                );
            } else {
                atomic_toml(&timing, &record)?;
            }
            ensure!(output.passed, "confined {label} failed: {}", output.diagnostic);
            Ok(record)
        };
        if u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())? >= self.deadline_unix_ms {
            bounded::reconcile(8 * bounded::DEFAULT_BYTES, publish)
        } else {
            publish()
        }
    }

    fn check_deadline(&self) -> Result<()> {
        ensure!(
            u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())? < self.deadline_unix_ms,
            "native family aggregate time budget exhausted before producer preparation"
        );
        Ok(())
    }

    fn reserve(&self, bytes: u64) -> Result<()> {
        let mut used = self.reserved_bytes.borrow_mut();
        let next = used.checked_add(bytes).context("native gate reservation overflow")?;
        ensure!(
            next <= self.max_output_bytes,
            "native gate aggregate output reservation exhausted ({next} > {})",
            self.max_output_bytes
        );
        *used = next;
        Ok(())
    }

    fn lease(&self) -> Result<(File, f64)> {
        ensure!((1..=64).contains(&self.workers), "native worker count missing");
        let started = Instant::now();
        let workers = self.root.join("workers");
        fs::create_dir_all(&workers)?;
        let guard = fs::OpenOptions::new().create(true).append(true).open(workers.join("config.lock"))?;
        loop {
            if guard.try_lock().is_ok() {
                break;
            }
            let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
            ensure!(now < self.deadline_unix_ms, "native gate deadline exhausted waiting for shared configuration");
            std::thread::sleep(Duration::from_millis(10));
        }
        let config = workers.join("capacity.toml");
        #[derive(Serialize, Deserialize)]
        struct Capacity {
            workers: usize,
        }
        if config.exists() {
            ensure!(
                crate::family::object::<Capacity>(&config)?.workers == self.workers,
                "shared native worker budget differs"
            );
        } else {
            atomic_toml(&config, &Capacity { workers: self.workers })?;
        }
        drop(guard);
        loop {
            crate::metrics::check_cancelled()?;
            for slot in 0..self.workers {
                let held =
                    fs::OpenOptions::new().create(true).append(true).open(workers.join(format!("{slot}.lock")))?;
                if held.try_lock().is_ok() {
                    return Ok((held, started.elapsed().as_secs_f64()));
                }
            }
            let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
            ensure!(now < self.deadline_unix_ms, "native gate deadline exhausted waiting for shared worker");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

#[cfg(test)]
mod strict_tests {
    use super::*;

    #[test]
    fn expired_gate_cannot_reserve_runtime_or_publish_a_producer_invocation() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "sm-expired-producer-{}-{}",
            std::process::id(),
            crate::family::unique_id()
        ));
        let limits = RunLimits {
            timeout: Duration::from_secs(1),
            writable_bytes: 4096,
            stdout_bytes: 0,
            stderr_bytes: 0,
            stdin_bytes: 0,
        };
        let execution = StrictExecution {
            helper: root.join("helper"),
            rustc: root.join("rustc"),
            c_compiler: root.join("cc"),
            compiler_runtime: Vec::new(),
            operation_root: root.join("operations"),
            context: OperationContext {
                run_id: translation_execution::model::sha256("expired"),
                attempt: 1,
                policy_sha256: translation_execution::model::sha256("policy"),
                operation_key: translation_execution::model::sha256("operation"),
                phase: translation_execution::model::ExecutionPhase::Candidate,
            },
            root: root.join("pipeline"),
            workers: 4,
            compiler_limits: limits.clone(),
            fixture_limits: limits.clone(),
            rust_inputs: Vec::new(),
            c_inputs: Vec::new(),
            deadline_unix_ms: 0,
            max_output_bytes: 4096,
            reserved_bytes: RefCell::new(0),
        };
        let program = Program {
            executable: root.join("program"),
            arguments: Vec::new(),
            inputs: Vec::new(),
            runtime: Vec::new(),
            helper: root.join("helper"),
            outputs: Vec::new(),
        };
        let error = execution.execute(&program, &limits, &root.join("run"), "c-build").unwrap_err();
        ensure!(
            error.to_string().contains("time budget exhausted"),
            "expired gate attempted runtime reservation: {error}"
        );
        ensure!(
            !root.exists() && *execution.reserved_bytes.borrow() == 0,
            "expired gate created a producer or resource reservation"
        );
        Ok(())
    }

    #[test]
    fn compiler_runtime_and_source_discovery_share_inherited_bounds() -> Result<()> {
        let directory = bounded::tests::Directory::new()?;
        let error = strict_compiler_runtime_until(&directory.0.join("unavailable-rustc"), 0, 1024).unwrap_err();
        assert!(error.to_string().contains("deadline"));
        fs::write(directory.0.join("source.rs"), b"12345")?;
        let error =
            bounded::until(bounded::now_unix_ms()? + 1000, 4, || source_files(&directory.0, "rs", &mut Vec::new()))
                .unwrap_err();
        assert!(error.to_string().contains("budget"));
        let mut path = directory.0.clone();
        for _ in 0..34 {
            path.push("nested");
            fs::create_dir(&path)?;
        }
        assert!(source_files(&directory.0, "h", &mut Vec::new()).unwrap_err().to_string().contains("depth"));
        Ok(())
    }
}

fn immutable_output(path: &Path, bytes: &[u8]) -> Result<()> {
    crate::inputs::immutable_bytes(path, bytes)
}

/// Read-only files needed by the host's installed compiler/linker. This exposes
/// neither a host checkout nor a controller executable. Runtime imports are
/// frozen by the durable operation runner before executing rewritten Rust.
pub(crate) fn strict_compiler_runtime(rustc: &Path) -> Result<Vec<ReadOnlyBind>> {
    bounded::limited(|| strict_compiler_runtime_bounded(rustc))
}

pub(crate) fn strict_compiler_runtime_until(rustc: &Path, deadline: u64, max_bytes: u64) -> Result<Vec<ReadOnlyBind>> {
    bounded::until(deadline, max_bytes, || strict_compiler_runtime_bounded(rustc))
}

fn strict_compiler_runtime_bounded(rustc: &Path) -> Result<Vec<ReadOnlyBind>> {
    let rustc = fs::canonicalize(rustc)?;
    let toolchain = rustc.parent().and_then(Path::parent).context("compiler toolchain root missing")?;
    ensure!(toolchain.join("bin/rustc") == rustc && toolchain.join("lib/rustlib").is_dir(), "installed rustc required");
    let mut bindings = confinement::helper_runtime()?;
    for (source, destination) in [
        ("/usr/bin/x86_64-linux-gnu-gcc-9", "/usr/bin/cc"),
        ("/usr/bin/x86_64-linux-gnu-as", "/usr/bin/as"),
        ("/usr/bin/x86_64-linux-gnu-ld.bfd", "/usr/bin/ld"),
        ("/usr/lib/gcc/x86_64-linux-gnu/9/cc1", "/usr/lib/gcc/x86_64-linux-gnu/9/cc1"),
        ("/usr/lib/gcc/x86_64-linux-gnu/9/collect2", "/usr/lib/gcc/x86_64-linux-gnu/9/collect2"),
        ("/usr/lib/gcc/x86_64-linux-gnu/9/include", "/usr/lib/gcc/x86_64-linux-gnu/9/include"),
        ("/usr/include", "/usr/include"),
    ] {
        bounded::check()?;
        bindings.push(ReadOnlyBind { source: fs::canonicalize(source)?, destination: destination.into() });
    }
    // GCC's non-LTO link closure; enumerate files so unrelated installed shared
    // libraries, system programs and package caches are inaccessible.
    for directory in ["/usr/lib/gcc/x86_64-linux-gnu/9", "/usr/lib/x86_64-linux-gnu"] {
        for path in bounded::entries(Path::new(directory))? {
            let name = path.file_name().context("runtime filename missing")?.to_string_lossy();
            if (name.starts_with("crt") && name.ends_with(".o"))
                || ["Scrt1.o", "rcrt1.o"].contains(&name.as_ref())
                || [
                    "libgcc.a",
                    "libgcc_eh.a",
                    "libgcc_s.so",
                    "liblto_plugin.so",
                    "libc.so",
                    "libc.a",
                    "libc_nonshared.a",
                    "libm.so",
                    "libm.a",
                    "libmvec.so",
                    "libmvec.a",
                    "libpthread.so",
                    "libpthread.a",
                    "libpthread_nonshared.a",
                    "libdl.so",
                    "libdl.a",
                    "librt.so",
                    "librt.a",
                    "libutil.so",
                    "libutil.a",
                ]
                .contains(&name.as_ref())
            {
                bindings.push(ReadOnlyBind { source: fs::canonicalize(&path)?, destination: path });
            }
        }
    }
    // Dynamic linker/search aliases referenced by libc and gcc's link scripts.
    for path in [
        "/usr/lib/x86_64-linux-gnu/libgcc_s.so.1",
        "/lib/x86_64-linux-gnu/libc.so.6",
        "/lib/x86_64-linux-gnu/libm.so.6",
        "/lib/x86_64-linux-gnu/libmvec.so.1",
        "/lib/x86_64-linux-gnu/libpthread.so.0",
        "/lib/x86_64-linux-gnu/libdl.so.2",
        "/lib/x86_64-linux-gnu/librt.so.1",
        "/lib/x86_64-linux-gnu/libutil.so.1",
        "/lib/x86_64-linux-gnu/libz.so.1",
        "/lib/x86_64-linux-gnu/libmpfr.so.6",
        "/lib/x86_64-linux-gnu/libmpc.so.3",
        "/lib/x86_64-linux-gnu/libgmp.so.10",
        "/lib/x86_64-linux-gnu/libisl.so.22",
        "/lib/x86_64-linux-gnu/libexpat.so.1",
        "/lib/x86_64-linux-gnu/libzstd.so.1",
        "/lib/x86_64-linux-gnu/libstdc++.so.6",
        "/lib/x86_64-linux-gnu/libtinfo.so.6",
        "/lib/x86_64-linux-gnu/libxml2.so.2",
        "/lib/x86_64-linux-gnu/liblzma.so.5",
    ] {
        bounded::check()?;
        if Path::new(path).exists() && !bindings.iter().any(|binding| binding.destination == Path::new(path)) {
            bindings.push(ReadOnlyBind { source: fs::canonicalize(path)?, destination: path.into() });
        }
    }
    for path in [
        "/lib/x86_64-linux-gnu/libopcodes-2.34-system.so",
        "/lib/x86_64-linux-gnu/libbfd-2.34-system.so",
        "/lib/x86_64-linux-gnu/libctf.so.0",
    ] {
        bounded::check()?;
        bindings.push(ReadOnlyBind { source: fs::canonicalize(path)?, destination: path.into() });
    }
    bindings.push(ReadOnlyBind {
        source: fs::canonicalize("/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2")?,
        destination: "/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2".into(),
    });
    bindings.push(ReadOnlyBind { source: toolchain.join("lib"), destination: "/toolchain/lib".into() });
    ensure!(bindings.len() <= 1024, "installed compiler runtime binding inventory exceeded");
    bounded::count(bindings.len())?;
    Ok(bindings)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExecRequest {
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
pub(crate) struct SamplerPath {
    path: PathBuf,
    kind: crate::resource_sampler::PathKind,
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
fn default_timeout() -> u64 {
    240
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
    source_files(directory, "h", result)
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
    let mut metrics = crate::metrics::Run::start(run.join(format!("{label}.metrics.toml")), label)?;
    let result = (|| -> Result<ExecutionRecord> {
        let started = Instant::now();
        let waiting = metrics.context.stage("worker-wait")?;
        let (_held, wait) = acquire_lease(root, workers)?;
        waiting.finish("success")?;
        let building = metrics.context.stage("execute")?;
        let active = Instant::now();
        let mut actual = Command::new(&command[0]);
        actual
            .args(&command[1..])
            .current_dir(run)
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

fn rust_sources(directory: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    source_files(directory, "rs", result)
}

fn source_files(directory: &Path, extension: &str, result: &mut Vec<PathBuf>) -> Result<()> {
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

pub fn cargo_build(request: CargoBuildRequest) -> Result<CacheResult> {
    fs::create_dir_all(&request.run)?;
    let mut metrics =
        crate::metrics::Run::start(request.run.join(format!("{}-cache.metrics.toml", request.package)), "cargo-cache")?;
    let result = (|| -> Result<CacheResult> {
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
        if package == "evidence" {
            rust_sources(&checkout.join("crates/pipeline-model/src"), &mut inputs)?;
        }
        for path in bounded::entries(&checkout.join("crates"))? {
            let manifest = path.join("Cargo.toml");
            if manifest.is_file() {
                inputs.push(manifest);
            }
        }
        let config = checkout.join(".cargo");
        if config.is_dir() {
            for p in bounded::entries(&config)? {
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
                actual.extend([
                    "--jobs".into(),
                    "1".into(),
                    "--target-dir".into(),
                    target.to_string_lossy().into_owned(),
                ]);
                // Cargo's working directory must select this checkout, while logs stay in the run.
                fs::create_dir_all(&run)?;
                let waiting = metrics.context.stage("cargo-worker-wait")?;
                let (_held, _) = acquire_lease(&root, workers)?;
                waiting.finish("success")?;
                let mut command = Command::new(&actual[0]);
                command
                    .args(&actual[1..])
                    .current_dir(&checkout)
                    .stdout(File::create(run.join(format!("{package}-build.stdout")))?)
                    .stderr(File::create(run.join(format!("{package}-build.stderr")))?);
                let status = metrics.context.status(&mut command, "cargo-build")?;
                ensure!(status.success(), "{package} build failed: {status}");
                fs::copy(target.join("release").join(member), artifact)?;
                fs::remove_dir_all(target)?;
                Ok(())
            },
        )
    })();
    if let Ok(cache) = &result {
        metrics.context.configuration("cache-key", &cache.key);
        metrics.context.configuration("cache-hit", cache.hit);
    }
    let error = result.as_ref().err().map(|e| format!("{e:#}"));
    let finished = metrics.finish(crate::metrics::outcome(error.as_deref()), error);
    if result.is_ok() {
        finished?;
    }
    result
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
