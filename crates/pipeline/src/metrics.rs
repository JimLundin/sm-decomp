//! One owner for child exit/resource accounting and bounded per-run sampling.
use anyhow::{Context as _, Result, ensure};
use pipeline_model::metrics::{Clock, Measurement, Process, Receipt, Resources, Stage, Transport};
use std::{
    collections::BTreeSet,
    fs,
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

static NEXT: AtomicU64 = AtomicU64::new(0);
static CANCELLED: AtomicU64 = AtomicU64::new(0);
extern "C" fn cancel(signal: libc::c_int) {
    CANCELLED.store(signal as u64, Ordering::Relaxed);
}
struct Cancellation {
    previous: Vec<(libc::c_int, libc::sigaction)>,
}
impl Cancellation {
    fn install() -> Result<Self> {
        CANCELLED.store(0, Ordering::Relaxed);
        let mut guard = Self { previous: vec![] };
        for signal in [libc::SIGINT, libc::SIGTERM] {
            // SAFETY: sigaction is initialized and only uses a signal-safe atomic handler.
            let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
            action.sa_sigaction = cancel as *const () as usize;
            let mut previous = std::mem::MaybeUninit::uninit();
            if unsafe { libc::sigaction(signal, &action, previous.as_mut_ptr()) } != 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            guard.previous.push((signal, unsafe { previous.assume_init() }));
        }
        Ok(guard)
    }
}
impl Drop for Cancellation {
    fn drop(&mut self) {
        for (signal, action) in &self.previous {
            // SAFETY: restore the action returned by sigaction during installation.
            unsafe { libc::sigaction(*signal, action, std::ptr::null_mut()) };
        }
    }
}
pub(crate) fn outcome(error: Option<&str>) -> &'static str {
    match error {
        None => "success",
        Some(error) if error.contains("interrupted by signal") => "interrupted",
        Some(error) if error.contains("timeout") => "timed-out",
        Some(_) => "failed",
    }
}
pub(crate) fn check_cancelled() -> Result<()> {
    let signal = CANCELLED.load(Ordering::Relaxed);
    ensure!(signal == 0, "interrupted by signal {signal}");
    Ok(())
}
pub(crate) fn lock(file: &fs::File) -> Result<()> {
    loop {
        check_cancelled()?;
        match file.try_lock() {
            Ok(()) => return Ok(()),
            Err(std::fs::TryLockError::WouldBlock) => thread::sleep(Duration::from_millis(20)),
            Err(error) => return Err(error.into()),
        }
    }
}
pub(crate) fn sidecar(directory: &Path, role: &str) -> PathBuf {
    directory.join(format!("{role}-{}-{}.metrics.toml", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)))
}
thread_local! { static ACTIVE: std::cell::RefCell<Option<Context>>=const{std::cell::RefCell::new(None)}; }
pub(crate) fn active() -> Option<Context> {
    ACTIVE.with(|value| value.borrow().clone())
}
pub(crate) fn command_output(command: &mut Command, role: &str) -> Result<std::process::Output> {
    if let Some(context) = active() { context.output(command, role) } else { Ok(command.output()?) }
}
pub(crate) struct ActiveGuard(Option<Context>);
impl Context {
    pub fn snapshot(&self) -> Receipt {
        self.inner.receipt.lock().unwrap().clone()
    }
    pub fn budget_resources(&self) -> (Resources, bool) {
        let receipt = self.inner.receipt.lock().unwrap();
        (
            receipt.resources.clone(),
            receipt.configuration.get("resource-samples-complete").map(String::as_str) == Some("true"),
        )
    }
    pub fn add_live_output(&self, path: PathBuf) {
        self.inner.resources.lock().unwrap().add_live_output(path);
    }
    pub fn status(&self, command: &mut Command, role: &str) -> Result<ExitStatus> {
        let mut child = OwnedChild::spawn(command, self, role)?;
        child.wait()
    }
    pub fn configuration(&self, name: &str, value: impl ToString) {
        self.inner.receipt.lock().unwrap().configuration.insert(name.into(), value.to_string());
    }
    pub fn enabled(&self) -> bool {
        self.inner.receipt.lock().unwrap().enabled
    }
    pub fn transport(&self, path: &Path, logical: u64, pack: u64) -> Result<()> {
        let mut transport = Transport {
            logical_bytes: Measurement::value(logical),
            pack_bytes: Measurement::value(pack),
            ..Default::default()
        };
        if path.exists() {
            #[derive(serde::Deserialize)]
            struct Producer {
                format: String,
                enabled: bool,
                buffer_bytes: u64,
                transport_bytes: u64,
                flush_count: u64,
                write_calls: u64,
                write_elapsed_ns: u64,
                max_flush_ns: u64,
                short_writes: u64,
                interruptions: u64,
                outcome: String,
            }
            let producer: Producer = toml::from_str(&fs::read_to_string(path)?)?;
            ensure!(
                producer.format == "pipeline-producer-metrics-v1"
                    && producer.buffer_bytes <= 65536
                    && producer.outcome == "success",
                "invalid producer metrics"
            );
            ensure!(producer.transport_bytes == logical, "producer/collector transport byte count differs");
            transport.transport_bytes = Measurement::value(producer.transport_bytes);
            transport.flush_count = Measurement::value(producer.flush_count);
            transport.write_calls = Measurement::value(producer.write_calls);
            transport.short_writes = Measurement::value(producer.short_writes);
            transport.interruptions = Measurement::value(producer.interruptions);
            transport.write_elapsed_ns = if producer.enabled {
                Measurement::value(producer.write_elapsed_ns)
            } else {
                Measurement::unavailable("telemetry disabled")
            };
            transport.max_flush_ns = if producer.enabled {
                Measurement::value(producer.max_flush_ns)
            } else {
                Measurement::unavailable("telemetry disabled")
            };
        } else {
            transport.transport_bytes = Measurement::value(logical);
        }
        self.inner.receipt.lock().unwrap().transport = transport;
        Ok(())
    }
    pub fn enter(&self) -> ActiveGuard {
        ActiveGuard(ACTIVE.with(|value| value.replace(Some(self.clone()))))
    }
}
impl Drop for ActiveGuard {
    fn drop(&mut self) {
        ACTIVE.with(|value| {
            value.replace(self.0.take());
        });
    }
}
#[derive(Clone)]
pub(crate) struct Context {
    inner: Arc<Inner>,
}
struct Inner {
    receipt: Mutex<Receipt>,
    path: PathBuf,
    stopped: AtomicBool,
    pids: Mutex<BTreeSet<u32>>,
    wake: Condvar,
    wake_lock: Mutex<()>,
    resources: Arc<Mutex<crate::resource_sampler::ResourceSampler>>,
    last_disk: AtomicU64,
}
pub(crate) struct Run {
    pub context: Context,
    sampler: Option<thread::JoinHandle<()>>,
    finished: bool,
    previous: Option<Context>,
    cancellation: Option<Cancellation>,
}
impl Run {
    pub fn start(path: PathBuf, role: &str) -> Result<Self> {
        let previous = active();
        let inherited_context = if let Some(parent) = &previous {
            let r = parent.inner.receipt.lock().unwrap();
            Some(pipeline_model::metrics::ContextReceipt {
                run_id: r.run_id.clone(),
                scope_id: format!("{}-{}", r.scope_id, NEXT.fetch_add(1, Ordering::Relaxed)),
                parent_scope: Some(r.scope_id.clone()),
                clock: r.clock.clone(),
                enabled: r.enabled,
                path: if path == parent.inner.path {
                    path.with_extension(format!("{}.metrics.toml", NEXT.fetch_add(1, Ordering::Relaxed)))
                } else {
                    path.clone()
                },
            })
        } else {
            pipeline_model::metrics::ContextReceipt::from_env()?
        };
        let clock =
            inherited_context.as_ref().map(|context| context.clock.clone()).map(Ok).unwrap_or_else(Clock::new)?;
        let id = format!("{}-{}-{}", std::process::id(), clock.origin_ns, NEXT.fetch_add(1, Ordering::Relaxed));
        let enabled = std::env::var("SM_PIPELINE_METRICS").as_deref() != Ok("disabled");
        let start_ns = clock.offset()?;
        let receipt = Receipt { format:"pipeline-metrics-v1".into(),run_id:inherited_context.as_ref().map(|c|c.run_id.clone()).unwrap_or_else(||id.clone()),scope_id:inherited_context.as_ref().map(|c|c.scope_id.clone()).unwrap_or(id),parent_scope:inherited_context.as_ref().and_then(|c|c.parent_scope.clone()),role:role.into(),clock,enabled:inherited_context.as_ref().map(|c|c.enabled).unwrap_or(enabled),start_ns,end_ns:start_ns,elapsed_ns:0,outcome:"incomplete".into(),error:None,last_completed_stage:None,stages:vec![],processes:vec![],resources:Resources { rss_interval_ms:100,disk_interval_ms:500,semantics:"wait4 child accounting; aggregate concurrent /proc RSS sampled at 100ms; shared pages may count twice; short-lived/between-sample peaks may be missed".into(),..Default::default()},transport:Transport::default(),configuration:Default::default(),scopes:vec![] };
        let context = Context {
            inner: Arc::new(Inner {
                receipt: Mutex::new(receipt),
                path: inherited_context.as_ref().map(|c| c.path.clone()).unwrap_or(path),
                stopped: AtomicBool::new(false),
                pids: Mutex::new(BTreeSet::new()),
                wake: Condvar::new(),
                wake_lock: Mutex::new(()),
                resources: previous.as_ref().map(|parent| parent.inner.resources.clone()).unwrap_or_else(|| {
                    Arc::new(Mutex::new(crate::resource_sampler::ResourceSampler::new(std::process::id())))
                }),
                last_disk: AtomicU64::new(0),
            }),
        };
        let worker = context.clone();
        // Every top-level native process handles teardown, even when its clock
        // and receipt identity were inherited from a benchmark parent.
        let cancellation = previous.is_none().then(Cancellation::install).transpose()?;
        let sampler = (context.enabled() && inherited_context.is_none()).then(|| {
            thread::spawn(move || {
                while !worker.inner.stopped.load(Ordering::Acquire) {
                    worker.sample();
                    let locked = worker.inner.wake_lock.lock().unwrap();
                    if worker.inner.stopped.load(Ordering::Acquire) {
                        break;
                    }
                    let _ = worker.inner.wake.wait_timeout(locked, Duration::from_millis(100)).unwrap();
                }
            })
        });
        ACTIVE.with(|value| {
            value.replace(Some(context.clone()));
        });
        let run = Self { context, sampler, finished: false, previous, cancellation };
        run.context.write()?;
        Ok(run)
    }
    pub fn finish(&mut self, outcome: &str, error: Option<String>) -> Result<()> {
        self.inner_stop();
        if self.previous.is_none() {
            self.context.sample();
        }
        {
            let mut r = self.context.inner.receipt.lock().unwrap();
            r.end_ns = r.clock.offset()?;
            r.elapsed_ns = r.end_ns - r.start_ns;
            r.outcome = outcome.into();
            r.error = error;
        }
        self.context.write()?;
        if let Some(parent) = &self.previous {
            parent.merge_receipt(self.context.inner.receipt.lock().unwrap().clone())?;
        }
        self.finished = true;
        ACTIVE.with(|value| {
            value.replace(self.previous.take());
        });
        self.cancellation.take();
        Ok(())
    }
    fn inner_stop(&mut self) {
        self.context.inner.stopped.store(true, Ordering::Release);
        self.context.inner.wake.notify_all();
        if let Some(t) = self.sampler.take() {
            let _ = t.join();
        }
    }
}
impl Drop for Run {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.finish("incomplete", Some("scope exited before successful completion".into()));
        }
    }
}
pub(crate) struct StageGuard {
    context: Context,
    id: String,
    name: String,
    start: u64,
    finished: bool,
}
impl Context {
    pub fn add_path(&self, path: PathBuf, kind: crate::resource_sampler::PathKind) {
        self.inner.resources.lock().unwrap().add_path(path, kind);
    }
    pub fn release_path(&self, path: &Path) {
        self.sample_disk();
        self.inner.resources.lock().unwrap().remove_path(path);
    }
    pub fn sample_disk(&self) {
        self.sample_with_disk(true);
    }
    pub fn stage(&self, name: &str) -> Result<StageGuard> {
        let scope = self.inner.receipt.lock().unwrap().scope_id.clone();
        Ok(StageGuard {
            context: self.clone(),
            id: format!("{scope}-stage-{}", NEXT.fetch_add(1, Ordering::Relaxed)),
            name: name.into(),
            start: self.now()?,
            finished: false,
        })
    }
    pub fn now(&self) -> Result<u64> {
        self.inner.receipt.lock().unwrap().clock.offset()
    }
    pub fn child_context(&self, command: &mut Command, path: &Path, role: &str) -> Result<PathBuf> {
        let r = self.inner.receipt.lock().unwrap();
        let context_path = path.with_extension("context.toml");
        let context = pipeline_model::metrics::ContextReceipt {
            run_id: r.run_id.clone(),
            scope_id: format!("{}-{role}-{}", r.scope_id, NEXT.fetch_add(1, Ordering::Relaxed)),
            parent_scope: Some(r.scope_id.clone()),
            clock: r.clock.clone(),
            enabled: r.enabled,
            path: path.to_path_buf(),
        };
        crate::atomic_toml(&context_path, &context)?;
        command.env("SM_PIPELINE_METRICS_CONTEXT", &context_path);
        Ok(context_path)
    }
    pub fn merge_child(&self, path: &Path) -> Result<()> {
        let child: Receipt = toml::from_str(&fs::read_to_string(path)?)?;
        let expected: pipeline_model::metrics::ContextReceipt =
            toml::from_str(&fs::read_to_string(path.with_extension("context.toml"))?)?;
        ensure!(
            child.scope_id == expected.scope_id && child.enabled == expected.enabled,
            "child metrics launch identity differs"
        );
        self.merge_receipt(child)
    }
    fn merge_receipt(&self, child: Receipt) -> Result<()> {
        child.validate()?;
        let mut r = self.inner.receipt.lock().unwrap();
        ensure!(
            child.format == "pipeline-metrics-v1"
                && child.clock.origin_ns == r.clock.origin_ns
                && child.clock.domain == r.clock.domain
                && child.run_id == r.run_id,
            "child metrics clock/run differs"
        );
        ensure!(child.parent_scope.as_deref() == Some(r.scope_id.as_str()), "child metrics parent differs");
        ensure!(
            child.start_ns >= r.start_ns && child.end_ns <= r.clock.offset()?,
            "child metrics outside parent interval"
        );
        ensure!(!r.scopes.iter().any(|scope| scope.id == child.scope_id), "child metrics already merged");
        let parent = r.scope_id.clone();
        r.scopes.push(pipeline_model::metrics::Scope {
            id: child.scope_id.clone(),
            parent,
            role: child.role,
            start_ns: child.start_ns,
            end_ns: child.end_ns,
            outcome: child.outcome,
            error: child.error,
            last_completed_stage: child.last_completed_stage,
            transport: child.transport,
            configuration: child.configuration,
        });
        r.scopes.extend(child.scopes);
        r.stages.extend(child.stages);
        r.processes.extend(child.processes);
        Ok(())
    }
    pub fn output(&self, command: &mut Command, role: &str) -> Result<std::process::Output> {
        use std::io::Read;
        command.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
        let mut child = OwnedChild::spawn(command, self, role)?;
        let mut stdout = child.child.stdout.take().context("missing child stdout")?;
        let mut stderr = child.child.stderr.take().context("missing child stderr")?;
        let out = thread::spawn(move || {
            let mut bytes = vec![];
            stdout.read_to_end(&mut bytes).map(|_| bytes)
        });
        let err = thread::spawn(move || {
            let mut bytes = vec![];
            stderr.read_to_end(&mut bytes).map(|_| bytes)
        });
        let status = child.wait();
        let stdout = out.join().map_err(|_| anyhow::anyhow!("stdout reader panicked"))??;
        let stderr = err.join().map_err(|_| anyhow::anyhow!("stderr reader panicked"))??;
        Ok(std::process::Output { status: status?, stdout, stderr })
    }
    fn write(&self) -> Result<()> {
        let mut receipt = self.inner.receipt.lock().unwrap();
        receipt.stages.sort_by_key(|stage| (stage.start_ns, stage.id.clone()));
        receipt.validate()?;
        crate::atomic_toml(&self.inner.path, &*receipt)
    }
    fn sample(&self) {
        let now = self.now().unwrap_or(0);
        let last = self.inner.last_disk.load(Ordering::Relaxed);
        let disk = now.saturating_sub(last) >= 500_000_000;
        if disk {
            self.inner.last_disk.store(now, Ordering::Relaxed);
        }
        self.sample_with_disk(disk);
    }
    fn sample_with_disk(&self, disk: bool) {
        if !self.inner.receipt.lock().unwrap().enabled {
            return;
        }
        let now = self.now().unwrap_or(0);
        let snapshot = self.inner.resources.lock().unwrap().sample(now, disk);
        let mut r = self.inner.receipt.lock().unwrap();
        r.resources = snapshot.resources;
        r.configuration.insert("resource-samples-complete".into(), snapshot.complete.to_string());
        r.configuration.insert("resource-limitations".into(), snapshot.limitations.join("; "));
        r.configuration.insert("rss-missing-processes".into(), snapshot.rss_missing_processes.to_string());
        r.configuration.insert("disk-missing-paths".into(), snapshot.disk_missing_paths.to_string());
        r.configuration.insert("truncated-scans".into(), snapshot.truncated_scans.to_string());
    }
}
impl StageGuard {
    pub fn finish(mut self, outcome: &str) -> Result<()> {
        self.record(outcome)?;
        self.finished = true;
        Ok(())
    }
    fn record(&self, outcome: &str) -> Result<()> {
        if !self.context.enabled() {
            return Ok(());
        }
        let end = self.context.now()?;
        let mut r = self.context.inner.receipt.lock().unwrap();
        if outcome == "success" {
            r.last_completed_stage = Some(self.name.clone());
        }
        let parent = Some(r.scope_id.clone());
        r.stages.push(Stage {
            id: self.id.clone(),
            parent,
            name: self.name.clone(),
            start_ns: self.start,
            end_ns: end,
            elapsed_ns: end - self.start,
            outcome: outcome.into(),
        });
        Ok(())
    }
}
impl Drop for StageGuard {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.record("incomplete");
        }
    }
}

pub(crate) struct OwnedChild {
    child: Child,
    context: Context,
    role: String,
    argv: Vec<String>,
    start: u64,
    status: Option<ExitStatus>,
    start_ticks: Measurement,
}
impl OwnedChild {
    pub fn poll_exit(&mut self) -> Result<Option<ExitStatus>> {
        self.reap(libc::WNOHANG)
    }
    pub fn terminate(&mut self) -> Result<ExitStatus> {
        // Let native orchestration owners stop and reap their separately grouped
        // producer/collector children before falling back to a forced group stop.
        let sent = unsafe { libc::kill(-(self.child.id() as i32), libc::SIGTERM) };
        if sent != 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
            return Err(std::io::Error::last_os_error().into());
        }
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(status) = self.reap(libc::WNOHANG)? {
                return Ok(status);
            }
            if std::time::Instant::now() >= deadline {
                return self.kill_wait();
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
    pub fn take_stdin(&mut self) -> Option<std::process::ChildStdin> {
        self.child.stdin.take()
    }
    pub fn take_stdout(&mut self) -> Option<std::process::ChildStdout> {
        self.child.stdout.take()
    }
    pub fn spawn(command: &mut Command, context: &Context, role: &str) -> Result<Self> {
        let argv = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        let start = context.now()?;
        use std::os::unix::process::CommandExt;
        command.process_group(0);
        let child = command.spawn()?;
        let start_ticks = fs::read_to_string(format!("/proc/{}/stat", child.id()))
            .ok()
            .and_then(|s| s.rsplit_once(") ")?.1.split_whitespace().nth(19)?.parse::<u64>().ok())
            .map(Measurement::value)
            .unwrap_or_else(|| Measurement::unavailable("child exited before /proc identity read"));
        if let Measurement::Measured { value } = start_ticks {
            context.inner.resources.lock().unwrap().register(child.id(), value);
        }
        context.inner.pids.lock().unwrap().insert(child.id());
        Ok(Self { child, context: context.clone(), role: role.into(), argv, start, status: None, start_ticks })
    }
    pub fn poll(&mut self) -> Result<Option<ExitStatus>> {
        let signal = CANCELLED.load(Ordering::Relaxed);
        if signal != 0 {
            self.kill_wait()?;
            anyhow::bail!("interrupted by signal {signal}");
        }
        self.reap(libc::WNOHANG)
    }
    fn wait(&mut self) -> Result<ExitStatus> {
        loop {
            let signal = CANCELLED.load(Ordering::Relaxed);
            if signal != 0 {
                self.kill_wait()?;
                anyhow::bail!("interrupted by signal {signal}");
            }
            if let Some(status) = self.reap(libc::WNOHANG)? {
                return Ok(status);
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
    fn reap(&mut self, flags: i32) -> Result<Option<ExitStatus>> {
        if self.status.is_some() {
            return Ok(self.status);
        }
        // Observe exit without consuming it so the leader anchors the group identity
        // until all surviving group members have been stopped.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        let options = libc::WEXITED | libc::WNOWAIT | if flags == libc::WNOHANG { libc::WNOHANG } else { 0 };
        let observed = unsafe { libc::waitid(libc::P_PID, self.child.id(), &mut info, options) };
        if observed != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        if unsafe { info.si_pid() } == 0 {
            return Ok(None);
        }
        self.stop_group()?;
        loop {
            let mut status = 0;
            let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
            // SAFETY: wait4 receives this owned child's PID and writable status/rusage.
            let result = unsafe { libc::wait4(self.child.id() as i32, &mut status, flags, usage.as_mut_ptr()) };
            if result == 0 {
                return Ok(None);
            }
            if result < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    if CANCELLED.load(Ordering::Relaxed) != 0 {
                        anyhow::bail!("interrupted by signal {}", CANCELLED.load(Ordering::Relaxed));
                    }
                    continue;
                }
                return Err(error.into());
            }
            // SAFETY: successful wait4 initialized rusage.
            let usage = unsafe { usage.assume_init() };
            let status = ExitStatus::from_raw(status);
            self.status = Some(status);
            self.context.inner.pids.lock().unwrap().remove(&self.child.id());
            self.context.inner.resources.lock().unwrap().unregister(self.child.id());
            let cpu = |t: libc::timeval| -> u64 { (t.tv_sec as u64) * 1_000_000_000 + (t.tv_usec as u64) * 1000 };
            let end_ns = self.context.now()?;
            let scope = self.context.inner.receipt.lock().unwrap().scope_id.clone();
            self.context.inner.receipt.lock().unwrap().processes.push(Process {
                scope,
                pid: self.child.id(),
                parent_pid: std::process::id(),
                start_ticks: self.start_ticks.clone(),
                role: self.role.clone(),
                argv: self.argv.clone(),
                start_ns: self.start,
                end_ns,
                exit_code: status.code(),
                signal: status.signal(),
                outcome: if status.success() { "success" } else { "failed" }.into(),
                reap_count: 1,
                user_cpu_ns: Measurement::value(cpu(usage.ru_utime)),
                system_cpu_ns: Measurement::value(cpu(usage.ru_stime)),
                peak_rss_bytes: Measurement::value(usage.ru_maxrss as u64 * 1024),
                accounting: "Linux wait4 for owned direct child; may include descendants that child reaped".into(),
                error: None,
            });
            return Ok(Some(status));
        }
    }
    pub fn kill_wait(&mut self) -> Result<ExitStatus> {
        if let Some(status) = self.status {
            return Ok(status);
        }
        self.stop_group()?;
        self.reap(0)?.context("blocking wait4 returned no child")
    }
    fn stop_group(&mut self) -> Result<()> {
        // SAFETY: each owned child leads a separate process group established at spawn.
        let killed = unsafe { libc::kill(-(self.child.id() as i32), libc::SIGKILL) };
        let kill_error = (killed != 0).then(std::io::Error::last_os_error);
        if let Some(error) = kill_error
            && error.raw_os_error() != Some(libc::ESRCH)
        {
            return Err(error.into());
        }
        Ok(())
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.status.is_none() {
            let _ = self.kill_wait();
        }
    }
}
