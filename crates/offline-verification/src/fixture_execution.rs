//! Dependency admission, worker leases and isolated fixture execution.
use crate::{
    atomic_toml,
    metrics::{self, Context, OwnedChild},
};
use anyhow::{Context as _, Result, bail, ensure};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Condvar, Mutex},
    time::{Duration, Instant},
};
use tool_execution::builds::ExecutionRecord;

pub(crate) type Task<'a> = (&'a crate::family_model::Routine, &'a crate::family_model::Case, usize);

fn predecessors(tasks: &[Task<'_>]) -> Result<Vec<Option<usize>>> {
    let names: BTreeMap<_, _> = tasks.iter().enumerate().map(|(i, t)| (t.1.name.as_str(), i)).collect();
    ensure!(names.len() == tasks.len(), "duplicate fixture task name");
    tasks
        .iter()
        .enumerate()
        .map(|(i, t)| {
            t.1.continue_from
                .as_deref()
                .map(|name| {
                    let p = *names.get(name).context("unknown predecessor")?;
                    ensure!(p < i, "predecessor must occur first");
                    Ok(p)
                })
                .transpose()
        })
        .collect()
}

/// Stable bounded waves for the separately confined native adapter. A whole
/// wave joins before the caller checks its comparisons or admits the next one.
pub(crate) fn waves(tasks: &[Task<'_>], jobs: usize) -> Result<Vec<Vec<usize>>> {
    ensure!((1..=64).contains(&jobs), "case jobs must be 1..64");
    let predecessors = predecessors(tasks)?;
    let mut ready = BTreeSet::new();
    let mut successors = vec![Vec::new(); tasks.len()];
    for (index, predecessor) in predecessors.into_iter().enumerate() {
        if let Some(predecessor) = predecessor {
            successors[predecessor].push(index);
        } else {
            ready.insert(index);
        }
    }
    let mut remaining = tasks.len();
    let mut waves = Vec::new();
    while remaining > 0 {
        let mut wave = Vec::new();
        for _ in 0..jobs {
            let Some(index) = ready.pop_first() else {
                break;
            };
            wave.push(index);
        }
        ensure!(!wave.is_empty(), "fixture dependency admission cannot progress");
        for &index in &wave {
            ready.extend(successors[index].iter().copied());
        }
        remaining -= wave.len();
        waves.push(wave);
    }
    Ok(waves)
}

/// Run all admitted workers to completion, including a wave containing a
/// failure. Results keep declaration order and no later wave starts on failure.
pub(crate) fn run_waves<T: Send>(
    tasks: &[Task<'_>],
    jobs: usize,
    begin: impl Fn() -> Result<()>,
    finish: impl Fn() -> Result<()>,
    adapter: impl Fn(Task<'_>) -> Result<T> + Sync,
) -> Result<Vec<(usize, Result<T>)>> {
    let shared = decomp_source::context::shared_scope()?;
    let mut outcomes = Vec::new();
    for wave in waves(tasks, jobs)? {
        if let Err(error) = begin() {
            outcomes.push((tasks.len(), Err(error)));
            break;
        }
        let completed = std::thread::scope(|scope| {
            let handles = wave
                .into_iter()
                .map(|task| {
                    let shared = &shared;
                    let adapter = &adapter;
                    (task, scope.spawn(move || shared.enter(|| adapter(tasks[task]))))
                })
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|(task, handle)| {
                    (
                        task,
                        handle.join().unwrap_or_else(|_| {
                            Err(translation_execution::ExecutionPending {
                                reason: "native fixture worker panicked; retain its operation ownership".into(),
                            }
                            .into())
                        }),
                    )
                })
                .collect::<Vec<_>>()
        });
        let failed = completed.iter().any(|(_, outcome)| outcome.is_err());
        outcomes.extend(completed);
        if let Err(error) = finish() {
            outcomes.push((tasks.len(), Err(error)));
            break;
        }
        if failed {
            break;
        }
    }
    outcomes.sort_by_key(|(task, _)| *task);
    Ok(outcomes)
}

struct Queue {
    // Lowest declared index wins admission, including blocked descendants.
    ready: BTreeMap<usize, bool>,
    remaining: usize,
}
pub(crate) fn run<T: Send>(
    tasks: &[Task<'_>],
    jobs: usize,
    run: &Path,
    root: &Path,
    capacity: usize,
    fork: bool,
    adapter: impl Fn(Task<'_>, &mut Worker) -> Result<T> + Sync,
) -> Result<Vec<(usize, Result<T>)>> {
    ensure!((1..=64).contains(&jobs), "case jobs must be 1..64");
    ensure!((1..=64).contains(&capacity), "worker capacity must be 1..64");
    let predecessors = predecessors(tasks)?;
    let mut dependents = vec![Vec::new(); tasks.len()];
    let mut ready = BTreeMap::new();
    for (index, predecessor) in predecessors.iter().enumerate() {
        if let Some(predecessor) = predecessor {
            dependents[*predecessor].push(index);
        } else {
            ready.insert(index, false);
        }
    }
    let queue = Mutex::new(Queue { ready, remaining: tasks.len() });
    let changed = Condvar::new();
    let outcomes = Mutex::new(Vec::new());
    let context = metrics::active().context("missing fixture metrics scope")?;
    context.configuration("fixture-execution", "dependency-v1");
    context.configuration("fixture-process", if fork { "fork" } else { "fresh" });
    std::thread::scope(|scope| {
        for _ in 0..jobs.min(tasks.len()).min(capacity) {
            scope.spawn(|| {
                let _entered = context.enter();
                let mut worker = Worker::new(run, fork, &context);
                loop {
                    let selected = {
                        let mut q = queue.lock().unwrap();
                        loop {
                            if q.remaining == 0 {
                                break None;
                            }
                            if let Some(selected) = q.ready.pop_first() {
                                break Some(selected);
                            }
                            q = changed.wait(q).unwrap();
                        }
                    };
                    let Some((index, blocked)) = selected else {
                        break;
                    };
                    let outcome = (|| -> Result<T> {
                        if blocked {
                            let dir = run.join(&tasks[index].1.name);
                            fs::create_dir_all(&dir)?;
                            #[derive(Serialize)]
                            struct Blocked<'a> {
                                blocked_by: &'a str,
                            }
                            let previous = &tasks[predecessors[index].unwrap()].1.name;
                            atomic_toml(&dir.join("blocked.toml"), &Blocked { blocked_by: previous })?;
                            bail!("fixture {} blocked by {previous}", tasks[index].1.name);
                        }
                        metrics::check_cancelled()?;
                        // A lease covers one ready Fixture pair, including
                        // Rust admission and C execution. Idle pristine helpers
                        // remain reusable without reserving execution capacity.
                        let (_lease, wait) = crate::acquire_lease(root, capacity).context("fixture worker startup")?;
                        worker.wait = wait;
                        adapter(tasks[index], &mut worker)
                    })();
                    let mut q = queue.lock().unwrap();
                    for &dependent in &dependents[index] {
                        q.ready.insert(dependent, outcome.is_err());
                    }
                    q.remaining -= 1;
                    outcomes.lock().unwrap().push((index, outcome));
                    changed.notify_all();
                }
                if let Err(error) = worker.finish() {
                    outcomes.lock().unwrap().push((tasks.len(), Err(error)));
                }
            });
        }
    });
    let mut outcomes = outcomes.into_inner().unwrap();
    outcomes.sort_by_key(|(i, _)| *i);
    Ok(outcomes)
}

pub(crate) struct Worker {
    wait: f64,
    context: Context,
    helpers: BTreeMap<String, Helper>,
    fork: bool,
    run: PathBuf,
}
impl Worker {
    fn new(run: &Path, fork: bool, context: &Context) -> Self {
        Self { wait: 0., context: context.clone(), helpers: BTreeMap::new(), fork, run: run.into() }
    }
    pub fn execute(&mut self, args: &[String], dir: &Path, label: &str) -> Result<()> {
        self.execute_timeout(args, dir, label, Duration::from_secs(240))
    }
    fn execute_timeout(&mut self, args: &[String], dir: &Path, label: &str, timeout: Duration) -> Result<()> {
        ensure!(!args.is_empty() && ["rust", "c"].contains(&label), "invalid fixture invocation");
        let started = Instant::now();
        let result = (|| -> Result<_> {
            if self.fork {
                if !self.helpers.contains_key(label) {
                    self.helpers.insert(label.into(), Helper::spawn(&self.run, label, &self.context)?);
                }
                let result = self.helpers.get_mut(label).unwrap().invoke(args, dir, label, timeout);
                if result.is_err() {
                    self.helpers.remove(label);
                }
                result
            } else {
                let mut command = Command::new(&args[0]);
                command
                    .args(&args[1..])
                    .current_dir(dir)
                    .stdout(File::create(dir.join(format!("{label}.stdout")))?)
                    .stderr(File::create(dir.join(format!("{label}.stderr")))?);
                let mut child = OwnedChild::spawn(&mut command, &self.context, label)?;
                loop {
                    if let Some(status) = child.poll()? {
                        return Ok(status);
                    }
                    if started.elapsed() >= timeout {
                        child.kill_wait()?;
                        bail!("fixture timeout");
                    }
                    std::thread::sleep(Duration::from_micros(200));
                }
            }
        })();
        let record = ExecutionRecord {
            argv: args.to_vec(),
            seconds: started.elapsed().as_secs_f64(),
            worker_wait_seconds: self.wait,
            exit_code: result.as_ref().ok().and_then(|s| s.code()),
            error: result.as_ref().err().map(|e| format!("{e:#}")),
        };
        self.wait = 0.;
        atomic_toml(&dir.join(format!("{label}.timing.toml")), &record)?;
        ensure!(result?.success(), "{label} fixture refused or failed");
        Ok(())
    }
    fn finish(&mut self) -> Result<()> {
        for helper in self.helpers.values_mut() {
            helper.stop()?;
        }
        Ok(())
    }
}

struct Helper {
    child: OwnedChild,
    input: Option<std::process::ChildStdin>,
    output: std::process::ChildStdout,
    next: u64,
    context: Context,
    finished: bool,
}
fn nonblocking(fd: i32) -> Result<()> {
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        ensure!(flags >= 0 && libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) >= 0, "pipe nonblocking failed");
    }
    Ok(())
}
fn ready(fd: i32, event: i16, deadline: Instant, cancel: bool) -> Result<()> {
    loop {
        if cancel {
            metrics::check_cancelled()?;
        }
        ensure!(Instant::now() < deadline, "fixture helper I/O timeout");
        let mut poll = libc::pollfd { fd, events: event, revents: 0 };
        let result = unsafe { libc::poll(&mut poll, 1, 10) };
        if result < 0 {
            let e = std::io::Error::last_os_error();
            if e.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(e.into());
        }
        if result > 0 {
            return Ok(());
        }
    }
}
impl Helper {
    fn spawn(run: &Path, label: &str, context: &Context) -> Result<Self> {
        let mut command = Command::new(run.join(label));
        command
            .arg("--fixture-worker")
            .arg(run.join("rom.bin"))
            .current_dir(run)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(File::create(run.join(format!("helper-{label}-{}.stderr", tool_execution::unique_id())))?);
        let mut child = OwnedChild::spawn(&mut command, context, &format!("fixture-helper-{label}"))?;
        let input = child.take_stdin().context("helper stdin missing")?;
        let output = child.take_stdout().context("helper stdout missing")?;
        nonblocking(input.as_raw_fd())?;
        nonblocking(output.as_raw_fd())?;
        let mut helper = Self { child, input: Some(input), output, next: 0, context: context.clone(), finished: false };
        let mut magic = [0; 8];
        helper.read(&mut magic, Instant::now() + Duration::from_secs(10), true)?;
        ensure!(&magic == b"SMFX0001", "invalid fixture helper handshake");
        Ok(helper)
    }
    fn read(&mut self, bytes: &mut [u8], deadline: Instant, cancel: bool) -> Result<()> {
        let mut offset = 0;
        while offset < bytes.len() {
            ready(self.output.as_raw_fd(), libc::POLLIN, deadline, cancel)?;
            match self.output.read(&mut bytes[offset..]) {
                Ok(0) => bail!("truncated fixture helper reply"),
                Ok(n) => offset += n,
                Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted) => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
    fn invoke(
        &mut self,
        args: &[String],
        dir: &Path,
        label: &str,
        timeout: Duration,
    ) -> Result<std::process::ExitStatus> {
        let id = self.next;
        self.next += 1;
        let mut bytes = vec![];
        let word = |out: &mut Vec<u8>, n: u64| out.extend(n.to_le_bytes());
        word(&mut bytes, id);
        word(&mut bytes, timeout.as_millis() as u64);
        for s in [dir.to_str().context("non-UTF8 fixture path")?, label] {
            word(&mut bytes, s.len() as u64);
            bytes.extend(s.as_bytes());
        }
        word(&mut bytes, args.len() as u64);
        for s in args {
            word(&mut bytes, s.len() as u64);
            bytes.extend(s.as_bytes());
        }
        ensure!(bytes.len() <= 65536, "fixture request exceeds 64 KiB");
        let mut frame = (bytes.len() as u32).to_le_bytes().to_vec();
        frame.extend(bytes);
        let deadline = Instant::now() + timeout + Duration::from_secs(5);
        let start = self.context.now()?;
        let input = self.input.as_mut().context("helper closed")?;
        let mut offset = 0;
        while offset < frame.len() {
            ready(input.as_raw_fd(), libc::POLLOUT, deadline, true)?;
            match input.write(&frame[offset..]) {
                Ok(0) => bail!("fixture helper write closed"),
                Ok(n) => offset += n,
                Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted) => {}
                Err(e) => return Err(e.into()),
            }
        }
        let mut reply = [0; 72];
        self.read(&mut reply, deadline, true)?;
        let values = reply.chunks_exact(8).map(|v| u64::from_le_bytes(v.try_into().unwrap())).collect::<Vec<_>>();
        ensure!(
            values[0] == id
                && (1..=i32::MAX as u64).contains(&values[1])
                && values[2] <= u32::MAX as u64
                && values[7] <= 1,
            "invalid fixture reply identity"
        );
        use std::os::unix::process::ExitStatusExt;
        let status = std::process::ExitStatus::from_raw(values[2] as i32);
        self.context.fixture_process(
            args,
            label,
            values[1] as u32,
            self.child.id(),
            start,
            status,
            values[4],
            values[5],
            values[6],
        )?;
        #[derive(Serialize)]
        struct ForkReceipt {
            request_id: u64,
            pid: u32,
            status: i32,
            elapsed_ns: u64,
            user_cpu_ns: u64,
            system_cpu_ns: u64,
            peak_rss_bytes: u64,
            timed_out: bool,
            descendants_reaped: u64,
        }
        atomic_toml(
            &dir.join(format!("{label}.fork.toml")),
            &ForkReceipt {
                request_id: id,
                pid: values[1] as u32,
                status: values[2] as i32,
                elapsed_ns: values[3],
                user_cpu_ns: values[4],
                system_cpu_ns: values[5],
                peak_rss_bytes: values[6],
                timed_out: values[7] != 0,
                descendants_reaped: values[8],
            },
        )?;
        ensure!(values[7] == 0, "fixture helper child timeout");
        Ok(status)
    }
    fn stop(&mut self) -> Result<()> {
        self.input.take();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(status) = self.child.poll_exit()? {
                self.finished = true;
                ensure!(status.success(), "fixture helper failed at shutdown");
                return Ok(());
            }
            ensure!(Instant::now() < deadline, "fixture helper shutdown timeout");
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
impl Drop for Helper {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        self.input.take();
        let _ = self.child.terminate_with_timeout(Duration::from_secs(3));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_waves_join_failures_and_preserve_continuations_and_order() -> Result<()> {
        use crate::family_model::{Case, Routine};
        use std::sync::{
            Barrier,
            atomic::{AtomicUsize, Ordering},
        };
        use tool_execution::bounded;
        let routine = Routine::default();
        let cases = [
            Case { name: "bad".into(), ..Default::default() },
            Case { name: "bad-child".into(), continue_from: Some("bad".into()), ..Default::default() },
            Case { name: "good".into(), ..Default::default() },
            Case { name: "good-child".into(), continue_from: Some("good".into()), ..Default::default() },
            Case { name: "later".into(), ..Default::default() },
        ];
        let tasks = cases.iter().enumerate().map(|(index, case)| (&routine, case, index)).collect::<Vec<_>>();
        assert_eq!(waves(&tasks, 2)?, vec![vec![0, 2], vec![1, 3], vec![4]]);
        assert_eq!(waves(&tasks, 1)?, vec![vec![0], vec![1], vec![2], vec![3], vec![4]]);
        assert_eq!(waves(&tasks, 4)?, vec![vec![0, 2, 4], vec![1, 3]]);
        let joined = AtomicUsize::new(0);
        let begins = AtomicUsize::new(0);
        let finishes = AtomicUsize::new(0);
        let barrier = Barrier::new(2);
        decomp_source::context::until(bounded::now_unix_ms()? + 1000, 8, || {
            let outcomes = run_waves(
                &tasks,
                2,
                || {
                    begins.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                },
                || {
                    assert_eq!(joined.load(Ordering::Relaxed), 2, "wave reconciled before both workers finished");
                    finishes.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                },
                |(_, case, index)| {
                    bounded::account(1)?;
                    barrier.wait();
                    joined.fetch_add(1, Ordering::Relaxed);
                    ensure!(case.name != "bad", "comparison refused");
                    Ok(index)
                },
            )?;
            assert_eq!(outcomes.len(), 2, "later waves ran after a comparison failure");
            assert_eq!((outcomes[0].0, outcomes[1].0), (0, 2));
            assert!(outcomes[0].1.is_err());
            assert_eq!(*outcomes[1].1.as_ref().unwrap(), 2);
            assert_eq!(bounded::charged_bytes(), 2);
            Ok(())
        })?;
        assert_eq!(begins.load(Ordering::Relaxed), 1);
        assert_eq!(finishes.load(Ordering::Relaxed), 1);
        Ok(())
    }

    #[test]
    fn real_worker_isolates_mutation_crash_timeout_and_reaps_descendants() {
        let root = std::env::temp_dir().join(format!(
            "sm-fixture-worker-{}-{}",
            std::process::id(),
            tool_execution::unique_id()
        ));
        fs::create_dir(&root).unwrap();
        let source = format!(
            r#"#[path={:?}]mod worker;
unsafe extern "C"{{fn fork()->i32;fn getpid()->i32;fn kill(pid:i32,sig:i32)->i32;}}
static mut COUNT:u8=0;
fn main(){{let args:Vec<String>=std::env::args().collect();if args.get(1).is_some_and(|s|s=="--fixture-worker"){{
worker::serve(|a|{{
let p=std::path::Path::new(&a[2]);std::fs::write(p.join("child.pid"),std::process::id().to_string()).unwrap();
match a[1].as_str(){{
"mutate"=>unsafe{{COUNT=99;}},"check"=>{{assert_eq!(unsafe{{COUNT}},0);println!("pristine");}},
"crash"=>unsafe{{kill(getpid(),9);}},
"descendant"=>{{let pid=unsafe{{fork()}};assert!(pid>=0);if pid==0{{std::fs::write(p.join("descendant.pid"),std::process::id().to_string()).unwrap();}}std::thread::sleep(std::time::Duration::from_secs(30));}},_=>panic!("action")}}0}}).unwrap();}}}}
"#,
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixture_worker.rs")
        );
        fs::write(root.join("worker.rs"), source).unwrap();
        let output = Command::new("rustc")
            .args(["--edition=2024", "-O"])
            .arg(root.join("worker.rs"))
            .arg("-o")
            .arg(root.join("c"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let mut metrics = metrics::Run::start(root.join("test.metrics.toml"), "fixture-test").unwrap();
        let (_lease, _) = crate::acquire_lease(&root.join("budget"), 1).unwrap();
        let mut worker = Worker::new(&root, true, &metrics.context);
        for (i, action) in ["mutate", "check", "crash", "check", "descendant", "check"].into_iter().enumerate() {
            let dir = root.join(format!("case-{i}"));
            fs::create_dir(&dir).unwrap();
            let args = vec![root.join("c").display().to_string(), action.into(), dir.display().to_string()];
            let result = worker.execute_timeout(&args, &dir, "c", Duration::from_millis(100));
            assert_eq!(result.is_ok(), !["crash", "descendant"].contains(&action));
            let pid: i32 = fs::read_to_string(dir.join("child.pid")).unwrap().parse().unwrap();
            assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
            if action == "descendant" {
                let pid: i32 = fs::read_to_string(dir.join("descendant.pid")).unwrap().parse().unwrap();
                assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
            }
        }
        worker.finish().unwrap();
        drop(worker);
        metrics.finish("success", None).unwrap();
        for deduplicate in [false, true] {
            crate::cleanup::cleanup(crate::cleanup::CleanupRequest { root: root.clone(), apply: true, deduplicate })
                .unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn incomplete_reply_and_oversized_request_are_rejected() {
        let root = std::env::temp_dir().join(format!(
            "sm-fixture-framing-{}-{}",
            std::process::id(),
            tool_execution::unique_id()
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("broken.rs"),r#"use std::io::{Read,Write};fn main(){let mut out=std::io::stdout().lock();out.write_all(b"SMFX0001").unwrap();out.flush().unwrap();drop(out);let mut header=[0;4];std::io::stdin().read_exact(&mut header).unwrap();let mut bytes=vec![0;u32::from_le_bytes(header) as usize];std::io::stdin().read_exact(&mut bytes).unwrap();std::io::stdout().write_all(&[0;3]).unwrap();}"#).unwrap();
        assert!(
            Command::new("rustc").arg(root.join("broken.rs")).arg("-o").arg(root.join("c")).status().unwrap().success()
        );
        let mut run = metrics::Run::start(root.join("metrics.toml"), "framing-control").unwrap();
        let mut helper = Helper::spawn(&root, "c", &run.context).unwrap();
        assert!(
            helper
                .invoke(&["c".into()], &root, "c", Duration::from_millis(100))
                .unwrap_err()
                .to_string()
                .contains("truncated")
        );
        drop(helper);
        let mut helper = Helper::spawn(&root, "c", &run.context).unwrap();
        assert!(
            helper
                .invoke(&["x".repeat(65536)], &root, "c", Duration::from_millis(100))
                .unwrap_err()
                .to_string()
                .contains("64 KiB")
        );
        drop(helper);
        run.finish("success", None).unwrap();
        for deduplicate in [false, true] {
            crate::cleanup::cleanup(crate::cleanup::CleanupRequest { root: root.clone(), apply: true, deduplicate })
                .unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }
}
