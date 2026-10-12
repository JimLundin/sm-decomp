//! Private capture stores let independent subjects produce concurrently.
use crate::{acquire_lease, atomic_toml, digest, observations};
use anyhow::{Context, Result, ensure};
use evidence::{ImportReceipt, Pointer, Provenance};
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
pub struct CaptureRequest {
    pub directory: PathBuf,
    pub reader: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
    pub command: Vec<String>,
    pub provenance: Provenance,
    pub timeout: f64,
}
#[derive(Deserialize)]
pub(crate) struct CaptureManyRequest {
    pub captures: Vec<CaptureRequest>,
    pub jobs: usize,
}
pub(crate) fn capture_parallel(captures: &[CaptureRequest], jobs: usize) -> Result<Vec<Pointer>> {
    ensure!((1..=4).contains(&jobs) && !captures.is_empty(), "invalid parallel capture extent");
    let next = std::sync::atomic::AtomicUsize::new(0);
    let outcomes = std::sync::Mutex::new(Vec::new());
    let context = crate::metrics::active();
    std::thread::scope(|scope| {
        for _ in 0..jobs {
            let context = context.clone();
            let next = &next;
            let outcomes = &outcomes;
            scope.spawn(move || {
                let _entered = context.as_ref().map(|context| context.enter());
                loop {
                    let index = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(request) = captures.get(index) else {
                        break;
                    };
                    let outcome = capture(request.clone());
                    outcomes.lock().unwrap().push((index, outcome));
                }
            });
        }
    });
    let mut outcomes = outcomes.into_inner().unwrap();
    outcomes.sort_by_key(|(index, _)| *index);
    let mut pointers = Vec::new();
    let mut errors = Vec::new();
    for (index, outcome) in outcomes {
        match outcome {
            Ok(pointer) => pointers.push(pointer),
            Err(error) => errors.push(format!("subject {index}: {error:#}")),
        }
    }
    ensure!(errors.is_empty(), "{}", errors.join("; "));
    Ok(pointers)
}
pub(crate) fn capture_many(request: CaptureManyRequest) -> Result<Vec<Pointer>> {
    let directory =
        request.captures.first().and_then(|capture| capture.directory.parent()).context("capture parent missing")?;
    let mut metrics = crate::metrics::Run::start(directory.join("captures.metrics.toml"), "captures")?;
    let result = capture_parallel(&request.captures, request.jobs);
    let error = result.as_ref().err().map(|error| format!("{error:#}"));
    metrics.finish(crate::metrics::outcome(error.as_deref()), error)?;
    result
}
#[derive(Serialize, Deserialize)]
pub struct MaterializeRequest {
    pub directory: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
    pub pointer_sha256: Option<String>,
    pub destination: PathBuf,
    pub member: Option<String>,
    pub frame: Option<u64>,
}
#[derive(Serialize, Deserialize)]
pub struct PublishRequest {
    pub directory: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
    pub store: PathBuf,
}
#[derive(Serialize)]
pub struct MaterializeReceipt {
    pub status: String,
    pub destination: PathBuf,
}
#[derive(Serialize)]
struct TimingReceipt {
    completed: bool,
    pipeline_seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    worker_wait_seconds: Option<f64>,
}
use std::{
    fs::{self, File, OpenOptions},
    io::BufRead,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

struct Children(Vec<crate::metrics::OwnedChild>);
struct Pipe(PathBuf);
impl Drop for Pipe {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
struct Timing {
    path: PathBuf,
    started: Instant,
    completed: bool,
    wait: Option<f64>,
}
impl Drop for Timing {
    fn drop(&mut self) {
        let _ = atomic_toml(
            &self.path,
            &TimingReceipt {
                completed: self.completed,
                pipeline_seconds: self.started.elapsed().as_secs_f64(),
                worker_wait_seconds: self.wait,
            },
        );
    }
}
fn document<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}
fn bytes(store: &Path) -> Result<u64> {
    let mut n = 0;
    for entry in fs::read_dir(store)? {
        let p = entry?.path();
        if p.extension().is_some_and(|s| s == "pack") {
            n += fs::metadata(p)?.len();
        }
    }
    Ok(n)
}
fn inventory(root: &Path, path: &Path, out: &mut std::collections::BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let p = entry?.path();
        ensure!(!p.is_symlink(), "staged evidence is a symlink");
        if p.is_dir() {
            inventory(root, &p, out)?;
        } else {
            out.insert(p.strip_prefix(root)?.to_string_lossy().into_owned(), digest(&p)?);
        }
    }
    Ok(())
}

pub fn capture(request: CaptureRequest) -> Result<Pointer> {
    let directory = fs::canonicalize(&request.directory)?;
    let mut metrics = crate::metrics::Run::start(directory.join("capture.metrics.toml"), "capture")?;
    metrics.context.configuration("subject", request.provenance.subject.as_deref().unwrap_or("unspecified"));
    metrics
        .context
        .configuration("reader-sha256", request.provenance.reader_sha256.as_deref().unwrap_or("unspecified"));
    metrics
        .context
        .configuration("producer-sha256", request.provenance.binary_sha256.as_deref().unwrap_or("unspecified"));
    metrics.context.configuration("workers", request.workers);
    if let Some(source) = &request.provenance.source {
        metrics.context.configuration("source-revision", &source.revision);
        metrics.context.configuration("source-archive-sha256", &source.archive_sha256);
    }
    let result = (|| -> Result<Pointer> {
        let mut timing = Timing {
            path: directory.join("pipeline-timing.toml"),
            started: Instant::now(),
            completed: false,
            wait: None,
        };
        let store = directory.join("store");
        fs::create_dir(&store)?;
        metrics.context.add_path(store.clone(), crate::resource_sampler::PathKind::PrivateStore);
        let stage = directory.join(".evidence-stage");
        fs::create_dir(&stage)?;
        metrics.context.add_path(stage.clone(), crate::resource_sampler::PathKind::Staging);
        metrics.context.sample_disk();
        fs::create_dir(stage.join("fixtures"))?;
        fs::create_dir(stage.join("saves"))?;
        let fifo = directory.join(".effects.pipe");
        ensure!(
            metrics
                .context
                .status(Command::new("mkfifo").args(["-m", "600"]).arg(&fifo), "capture-fifo-create")?
                .success(),
            "create capture pipe failed"
        );
        let _pipe = Pipe(fifo.clone());
        let reader = request.reader.as_path();
        let provenance = &request.provenance;
        ensure!(
            digest(reader)? == provenance.reader_sha256.as_deref().context("reader digest missing")?,
            "capture reader identity differs"
        );
        let provenance_path = directory.join("evidence-provenance.toml");
        atomic_toml(&provenance_path, provenance)?;
        let candidate = directory.join(".evidence-candidate.toml");
        let receipt = directory.join("trace-receipt.toml");
        let timeout = Duration::from_secs_f64(request.timeout);
        let started = Instant::now();
        let root = request.root.as_path();
        let workers = request.workers;
        let waiting = metrics.context.stage("worker-wait")?;
        let (_lease, wait) = acquire_lease(root, workers)?;
        waiting.finish("success")?;
        timing.wait = Some(wait);
        let before = bytes(&store)?;
        // O_RDWR avoids blocking if the producer exits before opening its pipe.
        let sentinel = OpenOptions::new().read(true).write(true).open(&fifo)?;
        let mut children = Children(Vec::new());
        let collector_ready = metrics.context.stage("collector-ready")?;
        let child_metrics = directory.join("collector.metrics.toml");
        let mut collector_command = Command::new(reader);
        collector_command
            .arg("capture")
            .arg(&store)
            .arg(&receipt)
            .arg(&stage)
            .arg(&provenance_path)
            .arg(&candidate)
            .stdin(File::open(&fifo)?)
            .stdout(Stdio::piped())
            .stderr(File::create(directory.join("evidence.stderr"))?);
        let child_context = metrics.context.child_context(&mut collector_command, &child_metrics, "collector")?;
        let mut collector = crate::metrics::OwnedChild::spawn(&mut collector_command, &metrics.context, "collector")?;
        let stdout = collector.take_stdout().context("collector stdout missing")?;
        children.0.push(collector);
        let (sender, receiver) = std::sync::mpsc::channel();
        let ready = std::thread::spawn(move || {
            let mut buf = String::new();
            let result = std::io::BufReader::new(stdout).read_line(&mut buf).map(|_| buf);
            let _ = sender.send(result);
        });
        let ready_started = Instant::now();
        let signal = loop {
            match receiver.recv_timeout(Duration::from_millis(50)) {
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
                    if ready_started.elapsed() < Duration::from_secs(60)
                        && crate::metrics::check_cancelled().is_ok() =>
                {
                    continue;
                }
                result => break result,
            }
        };
        if signal.is_err() {
            children.0[0].kill_wait()?;
        }
        ready.join().map_err(|_| anyhow::anyhow!("collector readiness worker panicked"))?;
        crate::metrics::check_cancelled()?;
        let signal = signal.context("collector readiness timeout")??;
        ensure!(signal == "ready\n", "collector failed to open store");
        collector_ready.finish("success")?;
        let command = &request.command;
        ensure!(
            !command.is_empty()
                && digest(Path::new(&command[0]))?
                    == provenance.binary_sha256.as_deref().context("binary digest missing")?,
            "producer identity differs"
        );
        let active = Instant::now();
        let replay = metrics.context.stage("replay")?;
        let mut producer = Command::new(&command[0]);
        producer
            .args(&command[1..])
            .current_dir(&directory)
            .env("SM_EVIDENCE_PIPE", &fifo)
            .env("SM_EVIDENCE_STAGING", &stage)
            .env("SM_PIPELINE_METRICS_PRODUCER_PATH", directory.join("producer.metrics.toml"))
            .env("SM_PIPELINE_METRICS", if metrics.context.enabled() { "enabled" } else { "disabled" })
            .stdout(File::create(directory.join("replay.stdout"))?)
            .stderr(File::create(directory.join("replay.stderr"))?);
        children.0.push(crate::metrics::OwnedChild::spawn(&mut producer, &metrics.context, "producer")?);
        let producer_status = loop {
            if let Some(status) = children.0[1].poll()? {
                break status;
            }
            ensure!(children.0[0].poll()?.is_none(), "collector exited during replay");
            ensure!(active.elapsed() < timeout, "producer timeout");
            std::thread::sleep(Duration::from_millis(50));
        };
        let producer_seconds = active.elapsed().as_secs_f64();
        replay.finish(if producer_status.success() { "success" } else { "failed" })?;
        drop(sentinel);
        let sealing = Instant::now();
        let drain = metrics.context.stage("fifo-drain-and-seal")?;
        let collector_status = loop {
            if let Some(status) = children.0[0].poll()? {
                break status;
            }
            ensure!(sealing.elapsed() < timeout, "collector seal timeout");
            std::thread::sleep(Duration::from_millis(50));
        };
        drain.finish(if collector_status.success() { "success" } else { "failed" })?;
        metrics.context.merge_child(&child_metrics)?;
        ensure!(
            producer_status.success() && collector_status.success(),
            "producer/collector failed: {producer_status}/{collector_status}"
        );
        let mut pointer: Pointer = document(&candidate)?;
        metrics.context.configuration("subject", provenance.subject.as_deref().unwrap_or("unspecified"));
        metrics.context.configuration("reader-sha256", provenance.reader_sha256.as_deref().unwrap_or("unspecified"));
        metrics.context.configuration("producer-sha256", provenance.binary_sha256.as_deref().unwrap_or("unspecified"));
        metrics.context.configuration("evidence-run", &pointer.run);
        metrics.context.configuration("observations", &pointer.observations);
        metrics.context.configuration("workers", workers);
        metrics.context.configuration("worker-root", root.display());
        ensure!(pointer.provenance == *provenance, "collector producer provenance differs");
        let mut staged = std::collections::BTreeMap::new();
        let comparison = metrics.context.stage("staged-comparison")?;
        inventory(&stage, &stage, &mut staged)?;
        let actual = pointer
            .artifacts
            .iter()
            .filter(|(name, _)| name.as_str() != "hardware.log")
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        ensure!(staged == actual, "packed restoration differs from staged observations");
        comparison.finish("success")?;
        pointer.store = Some(PathBuf::from("store"));
        pointer.pipeline_seconds = Some(started.elapsed().as_secs_f64());
        pointer.producer_seconds = Some(producer_seconds);
        pointer.seal_seconds = Some(sealing.elapsed().as_secs_f64());
        pointer.stored_added_bytes = Some(bytes(&store)? - before);
        pointer.worker_wait_seconds = Some(wait);
        pointer.store_wait_seconds = Some(0.0);
        metrics.context.transport(
            &directory.join("producer.metrics.toml"),
            *pointer.sizes.get("hardware.log").context("hardware bytes missing")?,
            pointer.stored_added_bytes.unwrap_or(0),
        )?;
        let pointer_fsync = metrics.context.stage("pointer-fsync")?;
        atomic_toml(&directory.join("evidence.toml"), &pointer)?;
        pointer_fsync.finish("success")?;
        metrics.context.configuration("pointer-sha256", digest(&directory.join("evidence.toml"))?);
        drop(_lease);
        let authentication = metrics.context.stage("subject-authentication")?;
        observations::subject_with_metrics(&directory, root, workers, None, Some(&metrics.context))?;
        authentication.finish("success")?;
        let release = metrics.context.stage("staging-release")?;
        metrics.context.release_path(&stage);
        fs::remove_dir_all(stage)?;
        for p in [candidate, receipt, provenance_path, child_context] {
            fs::remove_file(p)?;
        }
        timing.completed = true;
        release.finish("success")?;
        Ok(pointer)
    })();
    let error = result.as_ref().err().map(|e| format!("{e:#}"));
    let child_metrics = directory.join("collector.metrics.toml");
    if result.is_err() && child_metrics.exists() {
        let _ = metrics.context.merge_child(&child_metrics);
    }
    let outcome = crate::metrics::outcome(error.as_deref());
    let finished = metrics.finish(outcome, error);
    if result.is_ok() {
        finished?;
    }
    result
}

pub fn materialize(request: MaterializeRequest) -> Result<MaterializeReceipt> {
    let directory = fs::canonicalize(&request.directory)?;
    let root = request.root.as_path();
    let workers = request.workers;
    let pointer = observations::subject(&directory, root, workers, request.pointer_sha256.as_deref())?;
    let store = fs::canonicalize(directory.join(pointer.store.as_ref().context("pointer store missing")?))?;
    let destination = request.destination.as_path();
    let reader = directory.parent().context("subject parent missing")?.join("sm-evidence");
    let (_lease, _wait) = acquire_lease(root, workers)?;
    let mut command = Command::new(reader);
    if let Some(member) = request.member.as_deref() {
        command.arg(if request.frame.is_some() { "frame" } else { "cat" }).arg(store).arg(&pointer.run).arg(member);
        if let Some(frame) = request.frame {
            command.arg(frame.to_string());
        }
        command.stdout(OpenOptions::new().write(true).create_new(true).open(destination)?);
    } else {
        ensure!(request.frame.is_none(), "frame requires member");
        command.arg("extract").arg(store).arg(&pointer.run).arg(destination);
    }
    let output = command.output()?;
    ensure!(output.status.success(), "packed extraction failed: {}", String::from_utf8_lossy(&output.stderr));
    Ok(MaterializeReceipt { status: "extracted".into(), destination: destination.to_path_buf() })
}

pub fn publish(request: PublishRequest) -> Result<Pointer> {
    let directory = fs::canonicalize(&request.directory)?;
    let mut metrics = crate::metrics::Run::start(directory.join("publication.metrics.toml"), "publication")?;
    let result = (|| -> Result<Pointer> {
        let root = request.root.as_path();
        let workers = request.workers;
        let authentication = metrics.context.stage("initial-subject-authentication")?;
        let pointer = observations::subject_with_metrics(&directory, root, workers, None, Some(&metrics.context))?;
        authentication.finish("success")?;
        let private = fs::canonicalize(directory.join(pointer.store.as_ref().context("pointer store missing")?))?;
        let target = request.store.as_path();
        ensure!(!target.is_symlink(), "evidence store is a symlink");
        fs::create_dir_all(target)?;
        let target = fs::canonicalize(target)?;
        if private == target {
            return Ok(pointer);
        }
        metrics.context.add_path(private.clone(), crate::resource_sampler::PathKind::PrivateStore);
        metrics.context.sample_disk();
        let started = Instant::now();
        let guard = OpenOptions::new().create(true).append(true).open(target.join("pipeline.lock"))?;
        let waiting = metrics.context.stage("store-wait")?;
        crate::metrics::lock(&guard)?;
        waiting.finish("success")?;
        let waiting = metrics.context.stage("worker-wait")?;
        let (_lease, _wait) = acquire_lease(root, workers)?;
        waiting.finish("success")?;
        let reader = directory.parent().context("subject parent missing")?.join("sm-evidence");
        let child_metrics = directory.join("import.metrics.toml");
        let mut command = Command::new(reader);
        command.arg("import").arg(&private).arg(&target).arg(&pointer.run);
        let child_context = metrics.context.child_context(&mut command, &child_metrics, "import")?;
        let output = metrics.context.output(&mut command, "shared-import")?;
        metrics.context.merge_child(&child_metrics)?;
        ensure!(output.status.success(), "shared evidence import failed: {}", String::from_utf8_lossy(&output.stderr));
        let imported: ImportReceipt = toml::from_str(std::str::from_utf8(&output.stdout)?)?;
        ensure!(
            imported.run == pointer.run
                && imported.artifacts == pointer.artifacts
                && imported.observations == pointer.observations
                && imported.provenance == pointer.provenance
                && imported.bytes == pointer.bytes
                && imported.files == pointer.files,
            "shared import differs"
        );
        let mut updated = pointer;
        updated.store = Some(target.clone());
        updated.publication_seconds = Some(started.elapsed().as_secs_f64());
        let fsync = metrics.context.stage("pointer-fsync")?;
        atomic_toml(&directory.join("evidence.toml"), &updated)?;
        fsync.finish("success")?;
        drop(_lease);
        drop(guard);
        let authentication = metrics.context.stage("subject-authentication")?;
        observations::subject_with_metrics(&directory, root, workers, None, Some(&metrics.context))?;
        authentication.finish("success")?;
        let release = metrics.context.stage("private-store-release")?;
        if private == directory.join("store") {
            metrics.context.release_path(&private);
            fs::remove_dir_all(private)?;
        }
        release.finish("success")?;
        fs::remove_file(child_context)?;
        Ok(updated)
    })();
    let error = result.as_ref().err().map(|e| format!("{e:#}"));
    metrics.finish(crate::metrics::outcome(error.as_deref()), error)?;
    result
}
