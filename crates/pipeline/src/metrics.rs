//! One owner for child exit/resource accounting and bounded per-run sampling.
use anyhow::{Context as _, Result, ensure};
use pipeline_model::metrics::{Clock, Measurement, Process, Receipt, Resources, Stage, Transport};
use std::{
    collections::BTreeSet,
    fs,
    os::unix::{fs::MetadataExt, process::ExitStatusExt},
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
thread_local! { static ACTIVE: std::cell::RefCell<Option<Context>>=const{std::cell::RefCell::new(None)}; }
pub(crate) fn active() -> Option<Context> {
    ACTIVE.with(|value| value.borrow().clone())
}
pub(crate) struct ActiveGuard(Option<Context>);
impl Context {
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
}
pub(crate) struct Run {
    pub context: Context,
    sampler: Option<thread::JoinHandle<()>>,
    finished: bool,
    previous: Option<Context>,
    inherited: bool,
}
impl Run {
    pub fn start(path: PathBuf, role: &str) -> Result<Self> {
        let previous = active();
        if let Some(context) = previous.clone() {
            return Ok(Self { context, sampler: None, finished: false, previous, inherited: true });
        }
        let inherited_context = pipeline_model::metrics::ContextReceipt::from_env()?;
        let clock =
            inherited_context.as_ref().map(|context| context.clock.clone()).map(Ok).unwrap_or_else(Clock::new)?;
        let id = format!("{}-{}-{}", std::process::id(), clock.origin_ns, NEXT.fetch_add(1, Ordering::Relaxed));
        let enabled = std::env::var("SM_PIPELINE_METRICS").as_deref() != Ok("disabled");
        let start_ns = clock.offset()?;
        let receipt = Receipt { format:"pipeline-metrics-v1".into(),run_id:inherited_context.as_ref().map(|c|c.run_id.clone()).unwrap_or_else(||id.clone()),scope_id:inherited_context.as_ref().map(|c|c.scope_id.clone()).unwrap_or(id),parent_scope:inherited_context.as_ref().and_then(|c|c.parent_scope.clone()),role:role.into(),clock,enabled:inherited_context.as_ref().map(|c|c.enabled).unwrap_or(enabled),start_ns,end_ns:start_ns,elapsed_ns:0,outcome:"incomplete".into(),error:None,last_completed_stage:None,stages:vec![],processes:vec![],resources:Resources { rss_interval_ms:100,disk_interval_ms:500,semantics:"wait4 child accounting; aggregate concurrent /proc RSS sampled at 100ms; shared pages may count twice; short-lived/between-sample peaks may be missed".into(),..Default::default()},transport:Transport::default(),configuration:Default::default() };
        let context = Context {
            inner: Arc::new(Inner {
                receipt: Mutex::new(receipt),
                path: inherited_context.as_ref().map(|c| c.path.clone()).unwrap_or(path),
                stopped: AtomicBool::new(false),
                pids: Mutex::new(BTreeSet::new()),
                wake: Condvar::new(),
                wake_lock: Mutex::new(()),
            }),
        };
        let worker = context.clone();
        let sampler = (enabled && inherited_context.is_none()).then(|| {
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
        Ok(Self { context, sampler, finished: false, previous, inherited: false })
    }
    pub fn finish(&mut self, outcome: &str, error: Option<String>) -> Result<()> {
        if self.inherited {
            self.finished = true;
            return Ok(());
        }
        self.inner_stop();
        self.context.sample();
        {
            let mut r = self.context.inner.receipt.lock().unwrap();
            r.end_ns = r.clock.offset()?;
            r.elapsed_ns = r.end_ns - r.start_ns;
            r.outcome = outcome.into();
            r.error = error;
        }
        self.context.write()?;
        self.finished = true;
        ACTIVE.with(|value| {
            value.replace(self.previous.take());
        });
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
        if r.stages.iter().any(|stage| child.stages.iter().any(|other| stage.id == other.id)) {
            return Ok(());
        }
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
        let status = child.reap(0)?.context("child wait missing")?;
        Ok(std::process::Output {
            status,
            stdout: out.join().map_err(|_| anyhow::anyhow!("stdout reader panicked"))??,
            stderr: err.join().map_err(|_| anyhow::anyhow!("stderr reader panicked"))??,
        })
    }
    fn write(&self) -> Result<()> {
        crate::atomic_toml(&self.inner.path, &*self.inner.receipt.lock().unwrap())
    }
    fn sample(&self) {
        let pids = self.inner.pids.lock().unwrap().clone();
        let mut total = 0;
        let mut incomplete = 0;
        for pid in pids {
            match fs::read_to_string(format!("/proc/{pid}/status")) {
                Ok(text) => {
                    if let Some(kb) = text.lines().find_map(|line| {
                        line.strip_prefix("VmRSS:").and_then(|s| s.split_whitespace().next()?.parse::<u64>().ok())
                    }) {
                        total += kb * 1024;
                    }
                }
                Err(_) => incomplete += 1,
            }
        }
        let mut r = self.inner.receipt.lock().unwrap();
        if !r.enabled {
            r.resources.aggregate_sampled_rss_bytes = Measurement::unavailable("telemetry disabled");
            return;
        }
        r.resources.rss_samples += 1;
        r.resources.incomplete_samples += incomplete;
        let previous = match r.resources.aggregate_sampled_rss_bytes {
            Measurement::Measured { value } => value,
            _ => 0,
        };
        if total >= previous {
            r.resources.aggregate_sampled_rss_bytes = Measurement::value(total);
            r.resources.rss_peak_at_ns = r.clock.offset().unwrap_or(0);
        }
    }
}
impl StageGuard {
    pub fn finish(mut self, outcome: &str) -> Result<()> {
        self.record(outcome)?;
        self.finished = true;
        Ok(())
    }
    fn record(&self, outcome: &str) -> Result<()> {
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
    pub fn take_stdout(&mut self) -> Option<std::process::ChildStdout> {
        self.child.stdout.take()
    }
    pub fn spawn(command: &mut Command, context: &Context, role: &str) -> Result<Self> {
        let argv = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        let start = context.now()?;
        let child = command.spawn()?;
        let start_ticks = fs::read_to_string(format!("/proc/{}/stat", child.id()))
            .ok()
            .and_then(|s| s.rsplit_once(") ")?.1.split_whitespace().nth(19)?.parse::<u64>().ok())
            .map(Measurement::value)
            .unwrap_or_else(|| Measurement::unavailable("child exited before /proc identity read"));
        context.inner.pids.lock().unwrap().insert(child.id());
        Ok(Self { child, context: context.clone(), role: role.into(), argv, start, status: None, start_ticks })
    }
    pub fn poll(&mut self) -> Result<Option<ExitStatus>> {
        self.reap(libc::WNOHANG)
    }
    fn reap(&mut self, flags: i32) -> Result<Option<ExitStatus>> {
        if self.status.is_some() {
            return Ok(self.status);
        }
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
                    continue;
                }
                return Err(error.into());
            }
            // SAFETY: successful wait4 initialized rusage.
            let usage = unsafe { usage.assume_init() };
            let status = ExitStatus::from_raw(status);
            self.status = Some(status);
            self.context.inner.pids.lock().unwrap().remove(&self.child.id());
            let cpu = |t: libc::timeval| -> u64 { (t.tv_sec as u64) * 1_000_000_000 + (t.tv_usec as u64) * 1000 };
            let end_ns = self.context.now()?;
            self.context.inner.receipt.lock().unwrap().processes.push(Process {
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
        if let Some(status) = self.poll()? {
            return Ok(status);
        }
        let killed = self.child.kill();
        let status = self.reap(0)?.context("blocking wait4 returned no child")?;
        if let Err(error) = killed {
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error.into());
            }
        }
        Ok(status)
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.status.is_none() {
            let _ = self.kill_wait();
        }
    }
}
