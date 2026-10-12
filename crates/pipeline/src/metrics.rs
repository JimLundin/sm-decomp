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
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

static NEXT: AtomicU64 = AtomicU64::new(0);
#[derive(Clone)]
pub(crate) struct Context {
    inner: Arc<Inner>,
}
struct Inner {
    receipt: Mutex<Receipt>,
    path: PathBuf,
    stopped: AtomicBool,
    pids: Mutex<BTreeSet<u32>>,
}
pub(crate) struct Run {
    pub context: Context,
    sampler: Option<thread::JoinHandle<()>>,
    finished: bool,
}
impl Run {
    pub fn start(path: PathBuf, role: &str) -> Result<Self> {
        let clock = Clock::new()?;
        let id = format!("{}-{}-{}", std::process::id(), clock.origin_ns, NEXT.fetch_add(1, Ordering::Relaxed));
        let enabled = std::env::var("SM_PIPELINE_METRICS").as_deref() != Ok("disabled");
        let receipt = Receipt { format:"pipeline-metrics-v1".into(),run_id:id.clone(),scope_id:id,parent_scope:None,role:role.into(),clock,enabled,start_ns:0,end_ns:0,elapsed_ns:0,outcome:"incomplete".into(),error:None,last_completed_stage:None,stages:vec![],processes:vec![],resources:Resources { rss_interval_ms:100,disk_interval_ms:500,semantics:"wait4 child accounting; aggregate concurrent /proc RSS sampled at 100ms; shared pages may count twice; short-lived/between-sample peaks may be missed".into(),..Default::default()},transport:Transport::default(),configuration:Default::default() };
        let context = Context {
            inner: Arc::new(Inner {
                receipt: Mutex::new(receipt),
                path,
                stopped: AtomicBool::new(false),
                pids: Mutex::new(BTreeSet::new()),
            }),
        };
        let worker = context.clone();
        let sampler = enabled.then(|| {
            thread::spawn(move || {
                while !worker.inner.stopped.load(Ordering::Acquire) {
                    worker.sample();
                    thread::sleep(Duration::from_millis(100));
                }
            })
        });
        Ok(Self { context, sampler, finished: false })
    }
    pub fn finish(&mut self, outcome: &str, error: Option<String>) -> Result<()> {
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
        Ok(())
    }
    fn inner_stop(&mut self) {
        self.context.inner.stopped.store(true, Ordering::Release);
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
        Ok(StageGuard {
            context: self.clone(),
            id: format!("stage-{}", NEXT.fetch_add(1, Ordering::Relaxed)),
            name: name.into(),
            start: self.now()?,
            finished: false,
        })
    }
    pub fn now(&self) -> Result<u64> {
        self.inner.receipt.lock().unwrap().clock.offset()
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
        r.stages.push(Stage {
            id: self.id.clone(),
            parent: None,
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
}
impl OwnedChild {
    pub fn spawn(command: &mut Command, context: &Context, role: &str) -> Result<Self> {
        let argv = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        let start = context.now()?;
        let child = command.spawn()?;
        context.inner.pids.lock().unwrap().insert(child.id());
        Ok(Self { child, context: context.clone(), role: role.into(), argv, start, status: None })
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
