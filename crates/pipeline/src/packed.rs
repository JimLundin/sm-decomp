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

struct Children(Vec<std::process::Child>);
impl Drop for Children {
    fn drop(&mut self) {
        for child in &mut self.0 {
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}
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
    let mut timing =
        Timing { path: directory.join("pipeline-timing.toml"), started: Instant::now(), completed: false, wait: None };
    let store = directory.join("store");
    fs::create_dir(&store)?;
    let stage = directory.join(".evidence-stage");
    fs::create_dir(&stage)?;
    fs::create_dir(stage.join("fixtures"))?;
    fs::create_dir(stage.join("saves"))?;
    let fifo = directory.join(".effects.pipe");
    ensure!(Command::new("mkfifo").args(["-m", "600"]).arg(&fifo).status()?.success(), "create capture pipe failed");
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
    let (_lease, wait) = acquire_lease(root, workers)?;
    timing.wait = Some(wait);
    let before = bytes(&store)?;
    // O_RDWR avoids blocking if the producer exits before opening its pipe.
    let sentinel = OpenOptions::new().read(true).write(true).open(&fifo)?;
    let mut children = Children(Vec::new());
    let mut collector = Command::new(reader)
        .arg("capture")
        .arg(&store)
        .arg(&receipt)
        .arg(&stage)
        .arg(&provenance_path)
        .arg(&candidate)
        .stdin(File::open(&fifo)?)
        .stdout(Stdio::piped())
        .stderr(File::create(directory.join("evidence.stderr"))?)
        .spawn()?;
    let stdout = collector.stdout.take().context("collector stdout missing")?;
    children.0.push(collector);
    let (sender, receiver) = std::sync::mpsc::channel();
    let ready = std::thread::spawn(move || {
        let mut buf = String::new();
        let result = std::io::BufReader::new(stdout).read_line(&mut buf).map(|_| buf);
        let _ = sender.send(result);
    });
    let signal = receiver.recv_timeout(Duration::from_secs(60)).context("collector readiness timeout")??;
    ready.join().map_err(|_| anyhow::anyhow!("collector readiness worker panicked"))?;
    ensure!(signal == "ready\n", "collector failed to open store");
    let command = &request.command;
    ensure!(
        !command.is_empty()
            && digest(Path::new(&command[0]))?
                == provenance.binary_sha256.as_deref().context("binary digest missing")?,
        "producer identity differs"
    );
    let active = Instant::now();
    children.0.push(
        Command::new(&command[0])
            .args(&command[1..])
            .current_dir(&directory)
            .env("SM_EVIDENCE_PIPE", &fifo)
            .env("SM_EVIDENCE_STAGING", &stage)
            .stdout(File::create(directory.join("replay.stdout"))?)
            .stderr(File::create(directory.join("replay.stderr"))?)
            .spawn()?,
    );
    let producer_status = loop {
        if let Some(status) = children.0[1].try_wait()? {
            break status;
        }
        ensure!(children.0[0].try_wait()?.is_none(), "collector exited during replay");
        ensure!(active.elapsed() < timeout, "producer timeout");
        std::thread::sleep(Duration::from_millis(50));
    };
    let producer_seconds = active.elapsed().as_secs_f64();
    drop(sentinel);
    let sealing = Instant::now();
    let collector_status = loop {
        if let Some(status) = children.0[0].try_wait()? {
            break status;
        }
        ensure!(sealing.elapsed() < timeout, "collector seal timeout");
        std::thread::sleep(Duration::from_millis(50));
    };
    ensure!(
        producer_status.success() && collector_status.success(),
        "producer/collector failed: {producer_status}/{collector_status}"
    );
    let mut pointer: Pointer = document(&candidate)?;
    ensure!(pointer.provenance == *provenance, "collector producer provenance differs");
    let mut staged = std::collections::BTreeMap::new();
    inventory(&stage, &stage, &mut staged)?;
    let actual = pointer
        .artifacts
        .iter()
        .filter(|(name, _)| name.as_str() != "hardware.log")
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    ensure!(staged == actual, "packed restoration differs from staged observations");
    pointer.store = Some(PathBuf::from("store"));
    pointer.pipeline_seconds = Some(started.elapsed().as_secs_f64());
    pointer.producer_seconds = Some(producer_seconds);
    pointer.seal_seconds = Some(sealing.elapsed().as_secs_f64());
    pointer.stored_added_bytes = Some(bytes(&store)? - before);
    pointer.worker_wait_seconds = Some(wait);
    pointer.store_wait_seconds = Some(0.0);
    atomic_toml(&directory.join("evidence.toml"), &pointer)?;
    drop(_lease);
    observations::subject(&directory, root, workers, None)?;
    fs::remove_dir_all(stage)?;
    for p in [candidate, receipt, provenance_path] {
        fs::remove_file(p)?;
    }
    timing.completed = true;
    Ok(pointer)
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
    let root = request.root.as_path();
    let workers = request.workers;
    let pointer = observations::subject(&directory, root, workers, None)?;
    let private = fs::canonicalize(directory.join(pointer.store.as_ref().context("pointer store missing")?))?;
    let target = request.store.as_path();
    ensure!(!target.is_symlink(), "evidence store is a symlink");
    fs::create_dir_all(target)?;
    let target = fs::canonicalize(target)?;
    if private == target {
        return Ok(pointer);
    }
    let started = Instant::now();
    let guard = OpenOptions::new().create(true).append(true).open(target.join("pipeline.lock"))?;
    guard.lock()?;
    let (_lease, _wait) = acquire_lease(root, workers)?;
    let reader = directory.parent().context("subject parent missing")?.join("sm-evidence");
    let output = Command::new(reader).arg("import").arg(&private).arg(&target).arg(&pointer.run).output()?;
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
    atomic_toml(&directory.join("evidence.toml"), &updated)?;
    drop(_lease);
    drop(guard);
    observations::subject(&directory, root, workers, None)?;
    if private == directory.join("store") {
        fs::remove_dir_all(private)?;
    }
    Ok(updated)
}
