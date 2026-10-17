//! Independent synthetic HDMA timing diagnostics, with bounded local workers.
//! These observations diagnose hardware timing; they do not admit game changes.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const SOURCE_TAR_SHA256: &str = "e9c0336218a47586e85b7c38341114995b3b7cdd727baf7c09c307d82072ef25";
const FRAME_CLOCKS: u64 = 357368;
const LINE_CLOCKS: u64 = 1364;
const FRAMES: u64 = 2;
static INTERRUPTED: AtomicBool = AtomicBool::new(false);
extern "C" fn interrupt(_: i32) {
    INTERRUPTED.store(true, Ordering::Relaxed);
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Direct,
    Indirect,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Loop {
    Nop,
    Branch,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Request {
    pub root: PathBuf,
    pub out: PathBuf,
    pub kind: Kind,
    pub instruction_loop: Loop,
    pub phases: usize,
    /// Local phase concurrency; all child processes also take shared leases.
    pub jobs: usize,
    pub timeout_seconds: u64,
}
impl Request {
    fn validate(&self) -> Result<()> {
        ensure!((1..=22).contains(&self.phases), "phases must be between 1 and 22");
        ensure!((1..=64).contains(&self.jobs), "jobs must be between 1 and 64");
        ensure!((1..=3600).contains(&self.timeout_seconds), "timeout must be between 1 and 3600 seconds");
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Row {
    pub pc: u32,
    pub master_clock: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Comparison {
    pub passed: bool,
    pub reference_instructions: usize,
    pub candidate_instructions: usize,
    pub compared_instructions: usize,
    pub original: Option<Row>,
    pub candidate: Option<Row>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Artifact {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Process {
    pub command: Vec<String>,
    pub cwd: PathBuf,
    pub environment: BTreeMap<String, String>,
    pub seconds: f64,
    pub worker_wait_seconds: f64,
    pub complete: bool,
    pub passed: bool,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub stdout: Artifact,
    pub stderr: Artifact,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Phase {
    pub phase: usize,
    pub complete: bool,
    pub comparison: Option<Comparison>,
    pub error: Option<String>,
    pub artifacts: Vec<Artifact>,
    pub processes: Vec<Process>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Origins {
    /// Existing recording executable, as in the legacy black-box diagnostic.
    /// This deliberately does not claim an independently rebuilt oracle.
    pub oracle_method: String,
    pub oracle_binary: Artifact,
    pub oracle_source_archive: Artifact,
    pub oracle_source: Vec<Artifact>,
    pub oracle_libraries: Vec<Artifact>,
    pub candidate_binary: Artifact,
    pub candidate_source: Vec<Artifact>,
    pub movie_template: Artifact,
    pub rustc_version: String,
    pub compiler: Artifact,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub run_id: String,
    pub owner: PathBuf,
    pub kind: Kind,
    pub instruction_loop: Loop,
    pub requested_phases: usize,
    pub frames: u64,
    pub complete: bool,
    pub passed: bool,
    pub error: Option<String>,
    pub origins: Option<Origins>,
    pub setup: Vec<Process>,
    pub phases: Vec<Phase>,
}

/// Literal register program retained byte-for-byte from the independent probe.
pub fn cartridge(kind: Kind, instruction_loop: Loop, phase: usize) -> Result<Vec<u8>> {
    ensure!(phase < 22, "phase index must be less than 22");
    let mut rom = vec![0; 65536];
    let mut code = vec![0x18, 0xfb, 0xe2, 0x30];
    for (reg, value) in [
        (0x4370u16, if kind == Kind::Indirect { 0x40 } else { 0 }),
        (0x4371, 0),
        (0x4372, 0),
        (0x4373, 0x90),
        (0x4374, 0),
        (0x4377, 0),
    ] {
        code.extend([0xa9, value, 0x8d, reg as u8, (reg >> 8) as u8]);
    }
    code.extend([0xad, 0x12, 0x42, 0x10, 0xfb, 0xa9, 0x80, 0x8d, 0x0c, 0x42]);
    code.extend(std::iter::repeat_n(0xea, phase));
    code.extend(match instruction_loop {
        Loop::Nop => &[0xea, 0xea, 0x80, 0xfc][..],
        Loop::Branch => &[0x80, 0xfe],
    });
    rom[..code.len()].copy_from_slice(&code);
    rom[0x1000..0x100b].copy_from_slice(match kind {
        Kind::Indirect => &[3, 0, 0x91, 0x83, 3, 0x91, 0, 0, 0, 0, 0],
        Kind::Direct => &[3, 0x80, 0x83, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0],
    });
    rom[0x1100..0x1106].fill(0x80);
    rom[0x7fc0..0x7fd5].copy_from_slice(b"SM HDMA PROBE        ");
    rom[0x7fd5] = 0x20;
    rom[0x7fd9] = 1;
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    rom[0x7ffc..0x7ffe].copy_from_slice(&[0, 0x80]);
    Ok(rom)
}

fn oracle_rows(path: &Path) -> Result<Vec<Row>> {
    let mut rows = Vec::new();
    let mut frame = 0;
    let mut last = 0;
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        let Some(pc) = line.get(..6).filter(|s| s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
        else {
            continue;
        };
        if line.as_bytes().get(6) != Some(&b' ') {
            continue;
        }
        let Some((_, timing)) = line.rsplit_once(" V:").or_else(|| line.rsplit_once("V:")) else { continue };
        let Some((v, h)) = timing.split_once("H:") else { continue };
        let v = v.trim().parse::<u64>().context("invalid oracle vertical clock")?;
        let h = h.split_whitespace().next().context("missing oracle horizontal clock")?.parse::<u64>()?;
        ensure!(v < 262 && h < LINE_CLOCKS, "oracle is outside the two-frame NTSC convention");
        let clock = v * LINE_CLOCKS + h;
        if clock < last {
            frame += 1;
        }
        ensure!(frame < FRAMES, "oracle contains more than two frames");
        last = clock;
        rows.push(Row { pc: u32::from_str_radix(pc, 16)?, master_clock: clock + frame * FRAME_CLOCKS });
    }
    Ok(rows)
}
fn candidate_rows(path: &Path) -> Result<Vec<Row>> {
    let mut rows = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        let (pc, clock) = line.split_once(',').context("candidate trace must contain pc,clock")?;
        ensure!(pc.len() == 6 && pc.bytes().all(|b| b.is_ascii_hexdigit()), "invalid candidate PC");
        let row = Row { pc: u32::from_str_radix(pc, 16)?, master_clock: clock.parse()? };
        if let Some(last) = rows.last() {
            let last: &Row = last;
            ensure!(row.master_clock > last.master_clock, "candidate clocks do not advance");
        }
        rows.push(row);
    }
    Ok(rows)
}
fn compare(original: &[Row], candidate: &[Row]) -> Result<Comparison> {
    ensure!(!original.is_empty() && candidate.len() >= original.len(), "incomplete timing capture");
    let mismatch = original.iter().zip(candidate).position(|(a, b)| a != b);
    Ok(Comparison {
        passed: mismatch.is_none(),
        reference_instructions: original.len(),
        candidate_instructions: candidate.len(),
        compared_instructions: mismatch.unwrap_or(original.len()),
        original: mismatch.map(|i| original[i].clone()),
        candidate: mismatch.map(|i| candidate[i].clone()),
    })
}
fn artifact(owner: &Path, path: &Path) -> Result<Artifact> {
    Ok(Artifact {
        path: path.strip_prefix(owner)?.to_str().context("artifact path is not UTF-8")?.into(),
        sha256: crate::digest(path)?,
        bytes: path.metadata()?.len(),
    })
}
fn retain(owner: &Path, source: &Path, relative: &str) -> Result<Artifact> {
    let target = crate::readiness_run::safe_artifact(owner, relative)?;
    fs::create_dir_all(target.parent().context("retained parent missing")?)?;
    ensure!(!target.exists(), "retained artifact already exists");
    let before = crate::digest(source)?;
    fs::copy(source, &target)?;
    ensure!(crate::digest(source)? == before && crate::digest(&target)? == before, "input changed while retaining");
    artifact(owner, &target)
}
fn snapshot(owner: &Path, root: &Path, prefix: &str, selected: fn(&Path) -> bool) -> Result<Vec<Artifact>> {
    fn visit(
        owner: &Path,
        root: &Path,
        directory: &Path,
        prefix: &str,
        selected: fn(&Path) -> bool,
        files: &mut Vec<Artifact>,
    ) -> Result<()> {
        let mut entries = fs::read_dir(directory)?.map(|e| e.map(|v| v.path())).collect::<std::io::Result<Vec<_>>>()?;
        entries.sort();
        for path in entries {
            if path.is_symlink() {
                continue;
            }
            if path.is_dir() {
                if !["target", ".git", "generated", "__pycache__"]
                    .contains(&path.file_name().and_then(|s| s.to_str()).unwrap_or(""))
                {
                    visit(owner, root, &path, prefix, selected, files)?;
                }
            } else if selected(&path) {
                let relative = format!("{prefix}/{}", path.strip_prefix(root)?.display());
                files.push(retain(owner, &path, &relative)?);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(owner, root, root, prefix, selected, &mut files)?;
    Ok(files)
}
fn candidate_input(path: &Path) -> bool {
    ["rs", "toml", "lock", "json", "in"].contains(&path.extension().and_then(|s| s.to_str()).unwrap_or(""))
}
fn oracle_input(path: &Path) -> bool {
    ["cpp", "hpp", "c", "h", "in", "mak"].contains(&path.extension().and_then(|s| s.to_str()).unwrap_or(""))
        || ["Makefile", "options.build", "VERSION"].contains(&path.file_name().and_then(|s| s.to_str()).unwrap_or(""))
}
fn movie_entries(template: &Path, rom: &[u8]) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut archive = zip::ZipArchive::new(File::open(template)?)?;
    let mut entries = BTreeMap::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        ensure!(entries.insert(entry.name().to_owned(), bytes).is_none(), "duplicate template ZIP member");
    }
    ensure!(
        ["input", "rom.sha256", "rom.hint"].iter().all(|k| entries.contains_key(*k)),
        "movie lacks required fields"
    );
    entries.insert("input".into(), b"F. 0 0|............\n".repeat(10));
    entries.insert("rom.sha256".into(), format!("{:x}\n", <sha2::Sha256 as sha2::Digest>::digest(rom)).into_bytes());
    entries.insert("rom.hint".into(), b"SM HDMA PROBE\n".to_vec());
    Ok(entries)
}
fn write_movie(template: &Path, path: &Path, rom: &[u8]) -> Result<()> {
    let mut archive = zip::ZipWriter::new(File::create(path)?);
    for (name, bytes) in movie_entries(template, rom)? {
        archive.start_file(name, zip::write::SimpleFileOptions::default())?;
        archive.write_all(&bytes)?;
    }
    archive.finish()?.sync_all()?;
    Ok(())
}
fn observer(path: &Path) -> Result<String> {
    let path = path.to_str().context("oracle output path is not UTF-8")?;
    // Quoting prevents worktree names containing quotes from changing Lua code.
    let quoted = path.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n").replace('\r', "\\r");
    Ok(format!(
        "local out=assert(io.open('{quoted}','w'))\nlocal function trace(cpu,line,real) if real then out:write(line,'\\n') end end\nmemory.registertrace(0,trace)\n"
    ))
}
struct Runtime {
    owner: PathBuf,
    shared: PathBuf,
    workers: usize,
    timeout: Duration,
}
impl Runtime {
    fn process(
        &self,
        label: &str,
        command: Vec<String>,
        cwd: &Path,
        environment: BTreeMap<String, String>,
    ) -> Result<Process> {
        let stdout = self.owner.join(format!("logs/{label}.stdout.log"));
        let stderr = self.owner.join(format!("logs/{label}.stderr.log"));
        let started = Instant::now();
        let (_lease, wait) = crate::acquire_lease(&self.shared, self.workers)?;
        let active = Instant::now();
        ensure!(!INTERRUPTED.load(Ordering::Relaxed), "HDMA probe interrupted");
        let mut child = Command::new(&command[0])
            .args(&command[1..])
            .current_dir(cwd)
            .envs(&environment)
            .process_group(0)
            .stdout(Stdio::from(File::create(&stdout)?))
            .stderr(Stdio::from(File::create(&stderr)?))
            .spawn()?;
        let mut error = None;
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            let interrupted = INTERRUPTED.load(Ordering::Relaxed);
            if interrupted || active.elapsed() >= self.timeout {
                unsafe {
                    kill(-(child.id() as i32), 9);
                }
                error = Some(if interrupted { "process interrupted" } else { "process timed out" }.into());
                break child.wait()?;
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        let report = Process {
            command,
            cwd: cwd.into(),
            environment,
            seconds: started.elapsed().as_secs_f64(),
            worker_wait_seconds: wait,
            complete: status.code().is_some() && error.is_none(),
            passed: status.success() && error.is_none(),
            exit_code: status.code(),
            error,
            stdout: artifact(&self.owner, &stdout)?,
            stderr: artifact(&self.owner, &stderr)?,
        };
        write_new(&self.owner.join(format!("logs/{label}.toml")), &report)?;
        Ok(report)
    }
}
unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
    fn signal(signal: i32, handler: usize) -> usize;
}
fn write_new<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(toml::to_string_pretty(value)?.as_bytes())?;
    file.sync_all()?;
    Ok(())
}
fn oracle_command(owner: &Path, origins: &Origins, directory: &Path) -> Vec<String> {
    vec![
        owner.join(&origins.oracle_binary.path).display().to_string(),
        "--dumper=INTERNAL-NULL".into(),
        format!("--rom-a={}", directory.join("probe.sfc").display()),
        "--length=2".into(),
        format!("--lua={}", directory.join("observer.lua").display()),
        directory.join("probe.lsmv").display().to_string(),
    ]
}
fn oracle_environment(owner: &Path, directory: &Path) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("XDG_CONFIG_HOME".into(), directory.join("config").display().to_string()),
        ("LD_LIBRARY_PATH".into(), owner.join("binaries/oracle-libraries").display().to_string()),
    ])
}
fn oracle_libraries(owner: &Path, root: &Path) -> Result<Vec<Artifact>> {
    let directory = root.join("deps/lsnes-build-deps/usr/lib/x86_64-linux-gnu");
    let output = Command::new("ldd")
        .arg(root.join("deps/lsnes/source/src/util/lsnes-dumpavi.util"))
        .env("LD_LIBRARY_PATH", &directory)
        .output()?;
    ensure!(output.status.success(), "cannot resolve recording executable libraries");
    let listing = String::from_utf8(output.stdout)?;
    ensure!(!listing.contains("not found"), "recording executable dependencies missing");
    let mut libraries = Vec::new();
    let mut names = BTreeSet::new();
    for line in listing.lines() {
        let words = line.split_whitespace().collect::<Vec<_>>();
        if words.len() >= 3 && words[1] == "=>" {
            let name = words[0];
            ensure!(!name.contains('/') && names.insert(name), "invalid/duplicate recording library");
            libraries.push(retain(owner, Path::new(words[2]), &format!("binaries/oracle-libraries/{name}"))?);
        }
    }
    ensure!(!libraries.is_empty(), "no recording libraries resolved");
    Ok(libraries)
}
fn candidate_command(owner: &Path, origins: &Origins, directory: &Path) -> Vec<String> {
    vec![
        owner.join(&origins.candidate_binary.path).display().to_string(),
        directory.join("probe.sfc").display().to_string(),
        directory.join("candidate.csv").display().to_string(),
        "2".into(),
    ]
}
fn phase(runtime: &Runtime, origins: &Origins, kind: Kind, instruction_loop: Loop, phase: usize) -> Phase {
    let mut result =
        Phase { phase, complete: false, comparison: None, error: None, artifacts: Vec::new(), processes: Vec::new() };
    let outcome = (|| -> Result<()> {
        let directory = runtime.owner.join(format!("phases/{phase:02}"));
        fs::create_dir(&directory)?;
        let rom_path = directory.join("probe.sfc");
        let movie = directory.join("probe.lsmv");
        let oracle = directory.join("oracle.txt");
        let lua = directory.join("observer.lua");
        let candidate = directory.join("candidate.csv");
        let rom = cartridge(kind, instruction_loop, phase)?;
        fs::write(&rom_path, &rom)?;
        write_movie(&runtime.owner.join(&origins.movie_template.path), &movie, &rom)?;
        fs::write(&lua, observer(&oracle)?)?;
        for path in [&rom_path, &movie, &lua] {
            result.artifacts.push(artifact(&runtime.owner, path)?);
        }
        let process = runtime.process(
            &format!("phase-{phase:02}-oracle"),
            oracle_command(&runtime.owner, origins, &directory),
            &directory,
            oracle_environment(&runtime.owner, &directory),
        )?;
        let passed = process.passed;
        result.processes.push(process);
        ensure!(passed, "recording core failed or timed out");
        result.artifacts.push(artifact(&runtime.owner, &oracle)?);
        let process = runtime.process(
            &format!("phase-{phase:02}-candidate"),
            candidate_command(&runtime.owner, origins, &directory),
            &directory,
            BTreeMap::new(),
        )?;
        let passed = process.passed;
        result.processes.push(process);
        ensure!(passed, "candidate failed or timed out");
        result.artifacts.push(artifact(&runtime.owner, &candidate)?);
        result.comparison = Some(compare(&oracle_rows(&oracle)?, &candidate_rows(&candidate)?)?);
        result.complete = true;
        Ok(())
    })();
    if let Err(error) = outcome {
        result.error = Some(format!("{error:#}"));
        // A failed or interrupted capture can still contain useful divergence
        // evidence. Seal every materialized phase artifact before returning.
        let directory = runtime.owner.join(format!("phases/{phase:02}"));
        for name in ["probe.sfc", "probe.lsmv", "observer.lua", "oracle.txt", "candidate.csv"] {
            let path = directory.join(name);
            if path.is_file()
                && !result.artifacts.iter().any(|a| a.path == format!("phases/{phase:02}/{name}"))
                && let Ok(item) = artifact(&runtime.owner, &path)
            {
                result.artifacts.push(item);
            }
        }
    }
    result
}

/// Runs each selected phase independently and retains failures alongside passes.
pub fn operate(request: Request) -> Result<Report> {
    request.validate()?;
    INTERRUPTED.store(false, Ordering::Relaxed);
    unsafe {
        signal(2, interrupt as *const () as usize);
    }
    let root = fs::canonicalize(&request.root)?;
    let out = if request.out.is_absolute() { request.out.clone() } else { root.join(&request.out) };
    fs::create_dir_all(&out)?;
    let id = format!("{}-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(), std::process::id());
    let owner = out.join(&id);
    fs::create_dir(&owner)?;
    let owner = fs::canonicalize(owner)?;
    for directory in ["logs", "phases", "source", "binaries"] {
        fs::create_dir(owner.join(directory))?;
    }
    write_new(&owner.join("request.toml"), &request)?;
    let common = Command::new("git").args(["rev-parse", "--git-common-dir"]).current_dir(&root).output()?;
    ensure!(common.status.success(), "cannot find shared worktree worker root");
    let common = fs::canonicalize(root.join(String::from_utf8(common.stdout)?.trim()))?;
    let shared = std::env::var_os("SM_PIPELINE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| common.parent().unwrap().join("out/pipeline"));
    let workers = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".into()).parse()?;
    let runtime =
        Runtime { owner: owner.clone(), shared, workers, timeout: Duration::from_secs(request.timeout_seconds) };
    let mut report = Report {
        schema_version: 1,
        run_id: id,
        owner: owner.clone(),
        kind: request.kind,
        instruction_loop: request.instruction_loop,
        requested_phases: request.phases,
        frames: FRAMES,
        complete: false,
        passed: false,
        error: None,
        origins: None,
        setup: Vec::new(),
        phases: Vec::new(),
    };
    let outcome = (|| -> Result<()> {
        let archive = root.join("deps/lsnes/sources.tar");
        ensure!(
            crate::digest(&archive)? == SOURCE_TAR_SHA256,
            "recording source archive differs from frozen upstream origin"
        );
        let oracle_source_archive = retain(&owner, &archive, "source/original.tar")?;
        let oracle_binary =
            retain(&owner, &root.join("deps/lsnes/source/src/util/lsnes-dumpavi.util"), "binaries/lsnes-dumpavi")?;
        let oracle_source = snapshot(&owner, &root.join("deps/lsnes/source"), "source/oracle", oracle_input)?;
        let oracle_libraries = oracle_libraries(&owner, &root)?;
        ensure!(
            oracle_source.iter().any(|a| a.path.ends_with("bsnes/snes/cpu/cpu.cpp")),
            "recording core source absent"
        );
        let movie_template = retain(&owner, &root.join("movies/super-metroid-any-3653.lsmv"), "source/template.lsmv")?;
        let mut candidate_source = Vec::new();
        for name in ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"] {
            candidate_source.push(retain(&owner, &root.join(name), &format!("source/candidate/{name}"))?);
        }
        candidate_source.extend(snapshot(&owner, &root.join("crates"), "source/candidate/crates", candidate_input)?);
        candidate_source.extend(snapshot(
            &owner,
            &root.join("deps/rustysnes"),
            "source/candidate/deps/rustysnes",
            candidate_input,
        )?);
        if root.join(".cargo").is_dir() {
            candidate_source.extend(snapshot(
                &owner,
                &root.join(".cargo"),
                "source/candidate/.cargo",
                candidate_input,
            )?);
        }
        // `~/.cargo/bin/rustc` can be a rustup shim. Retain the compiler selected
        // by the snapshotted toolchain file, rather than identifying that shim.
        let sysroot = Command::new("rustc").args(["--print", "sysroot"]).current_dir(&root).output()?;
        ensure!(sysroot.status.success(), "cannot resolve Rust compiler sysroot");
        let compiler_path = fs::canonicalize(Path::new(String::from_utf8(sysroot.stdout)?.trim()).join("bin/rustc"))?;
        let compiler = retain(&owner, &compiler_path, "binaries/rustc-origin")?;
        let version = Command::new(&compiler_path).arg("-vV").output()?;
        ensure!(version.status.success(), "cannot identify Rust compiler");
        let rustc_version = String::from_utf8(version.stdout)?;
        let source = owner.join("source/candidate");
        let build = owner.join("build");
        let process = runtime.process(
            "candidate-build",
            [
                "cargo",
                "build",
                "--release",
                "-p",
                "snes",
                "--bin",
                "timing-probe",
                "--locked",
                "--offline",
                "--jobs",
                "1",
            ]
            .map(str::to_owned)
            .to_vec(),
            &source,
            BTreeMap::from([("CARGO_TARGET_DIR".into(), build.display().to_string())]),
        )?;
        let passed = process.passed;
        report.setup.push(process);
        ensure!(passed, "candidate build failed or timed out");
        let candidate_binary = retain(&owner, &build.join("release/timing-probe"), "binaries/timing-probe")?;
        let origins = Origins {
            oracle_method: "retained-existing-recording-executable".into(),
            oracle_binary,
            oracle_source_archive,
            oracle_source,
            oracle_libraries,
            candidate_binary,
            candidate_source,
            movie_template,
            rustc_version,
            compiler,
        };
        for item in &origins.candidate_source {
            verify_artifact(&owner, item)?;
        }
        report.origins = Some(origins.clone());
        let next = AtomicUsize::new(0);
        let phase_results = std::thread::scope(|scope| {
            let mut workers = Vec::new();
            for _ in 0..request.jobs.min(request.phases) {
                workers.push(scope.spawn(|| {
                    let mut results = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        if index >= request.phases {
                            break;
                        }
                        results.push(phase(&runtime, &origins, request.kind, request.instruction_loop, index));
                    }
                    results
                }));
            }
            workers
                .into_iter()
                .map(|worker| worker.join().map_err(|_| anyhow::anyhow!("HDMA phase worker panicked")))
                .collect::<Result<Vec<_>>>()
        })?;
        report.phases = phase_results.into_iter().flatten().collect();
        report.phases.sort_by_key(|p| p.phase);
        report.complete = report.phases.len() == request.phases && report.phases.iter().all(|p| p.complete);
        report.passed =
            report.complete && report.phases.iter().all(|p| p.comparison.as_ref().is_some_and(|c| c.passed));
        Ok(())
    })();
    if let Err(error) = outcome {
        report.error = Some(format!("{error:#}"));
    }
    write_new(&owner.join("report.toml"), &report)?;
    Ok(report)
}
fn verify_artifact(owner: &Path, item: &Artifact) -> Result<PathBuf> {
    let path = crate::readiness_run::safe_artifact(owner, &item.path)?;
    ensure!(
        crate::digest(&path)? == item.sha256 && path.metadata()?.len() == item.bytes,
        "artifact differs: {}",
        item.path
    );
    Ok(path)
}
fn verify_process(owner: &Path, process: &Process, label: &str) -> Result<()> {
    verify_artifact(owner, &process.stdout)?;
    verify_artifact(owner, &process.stderr)?;
    let stored: Process = toml::from_str(&fs::read_to_string(owner.join(format!("logs/{label}.toml")))?)?;
    ensure!(
        &stored == process
            && process.seconds.is_finite()
            && process.seconds >= 0.0
            && process.worker_wait_seconds.is_finite()
            && process.worker_wait_seconds >= 0.0
            && process.stdout.path == format!("logs/{label}.stdout.log")
            && process.stderr.path == format!("logs/{label}.stderr.log"),
        "process result differs from retained record"
    );
    Ok(())
}
/// Recomputes terminal comparisons from retained text/CSV, preserving legacy evidence.
pub fn authenticate(path: &Path) -> Result<Report> {
    ensure!(path.extension().is_some_and(|p| p == "toml"), "HDMA report must use TOML");
    let report: Report = toml::from_str(&fs::read_to_string(path)?)?;
    let owner = fs::canonicalize(path.parent().context("report parent missing")?)?;
    ensure!(
        report.schema_version == 1
            && report.frames == FRAMES
            && report.owner == owner
            && owner.file_name().and_then(|p| p.to_str()) == Some(&report.run_id),
        "HDMA report owner/scope differs"
    );
    ensure!((1..=22).contains(&report.requested_phases), "invalid HDMA phase count");
    let request: Request = toml::from_str(&fs::read_to_string(owner.join("request.toml"))?)?;
    request.validate()?;
    ensure!(
        request.kind == report.kind
            && request.instruction_loop == report.instruction_loop
            && request.phases == report.requested_phases,
        "HDMA request differs from terminal report"
    );
    let origins = report.origins.as_ref().context("diagnostic setup was incomplete")?;
    ensure!(
        origins.oracle_method == "retained-existing-recording-executable"
            && origins.oracle_source_archive.sha256 == SOURCE_TAR_SHA256,
        "oracle origin method differs"
    );
    let mut artifact_paths = BTreeSet::new();
    for item in [
        &origins.oracle_binary,
        &origins.oracle_source_archive,
        &origins.candidate_binary,
        &origins.movie_template,
        &origins.compiler,
    ]
    .into_iter()
    .chain(&origins.oracle_source)
    .chain(&origins.oracle_libraries)
    .chain(&origins.candidate_source)
    {
        ensure!(artifact_paths.insert(&item.path), "duplicate source/origin path");
        verify_artifact(&owner, item)?;
    }
    ensure!(
        origins.oracle_binary.path == "binaries/lsnes-dumpavi"
            && origins.oracle_source_archive.path == "source/original.tar"
            && origins.candidate_binary.path == "binaries/timing-probe"
            && origins.movie_template.path == "source/template.lsmv"
            && origins.compiler.path == "binaries/rustc-origin",
        "diagnostic origin artifact paths differ"
    );
    let source_files = snapshot_names(&owner.join("source/candidate"), candidate_input)?;
    let expected_source = origins
        .candidate_source
        .iter()
        .map(|a| a.path.strip_prefix("source/candidate/").context("candidate source path differs").map(str::to_owned))
        .collect::<Result<BTreeSet<_>>>()?;
    ensure!(source_files == expected_source, "candidate source closure differs");
    let source_files = snapshot_names(&owner.join("source/oracle"), oracle_input)?;
    let expected_source = origins
        .oracle_source
        .iter()
        .map(|a| a.path.strip_prefix("source/oracle/").context("oracle source path differs").map(str::to_owned))
        .collect::<Result<BTreeSet<_>>>()?;
    ensure!(source_files == expected_source, "oracle source closure differs");
    ensure!(report.setup.len() == 1, "candidate build result missing");
    let build_process = &report.setup[0];
    verify_process(&owner, build_process, "candidate-build")?;
    ensure!(
        build_process.complete
            && build_process.passed
            && build_process.exit_code == Some(0)
            && build_process.error.is_none()
            && build_process.command
                == [
                    "cargo",
                    "build",
                    "--release",
                    "-p",
                    "snes",
                    "--bin",
                    "timing-probe",
                    "--locked",
                    "--offline",
                    "--jobs",
                    "1"
                ]
                .map(str::to_owned)
            && build_process.cwd == owner.join("source/candidate")
            && build_process.environment
                == BTreeMap::from([("CARGO_TARGET_DIR".into(), owner.join("build").display().to_string())]),
        "candidate build invocation differs"
    );
    let mut phase_ids = BTreeSet::new();
    for phase in &report.phases {
        ensure!(phase.phase < report.requested_phases && phase_ids.insert(phase.phase), "invalid/duplicate HDMA phase");
        for item in &phase.artifacts {
            verify_artifact(&owner, item)?;
        }
        for (index, process) in phase.processes.iter().enumerate() {
            ensure!(index < 2, "extra phase process");
            verify_process(
                &owner,
                process,
                &format!("phase-{:02}-{}", phase.phase, if index == 0 { "oracle" } else { "candidate" }),
            )?;
        }
        if phase.complete {
            ensure!(
                phase.processes.len() == 2
                    && phase
                        .processes
                        .iter()
                        .all(|p| p.complete && p.passed && p.exit_code == Some(0) && p.error.is_none()),
                "phase lacks two passing processes"
            );
            let directory = owner.join(format!("phases/{:02}", phase.phase));
            ensure!(
                phase.processes[0].command == oracle_command(&owner, origins, &directory)
                    && phase.processes[1].command == candidate_command(&owner, origins, &directory)
                    && phase.processes.iter().all(|p| p.cwd == directory)
                    && phase.processes[0].environment == oracle_environment(&owner, &directory)
                    && phase.processes[1].environment.is_empty(),
                "phase capture invocation differs"
            );
            let expected = ["probe.sfc", "probe.lsmv", "observer.lua", "oracle.txt", "candidate.csv"]
                .map(|p| format!("phases/{:02}/{p}", phase.phase))
                .into_iter()
                .collect::<BTreeSet<_>>();
            ensure!(
                phase.artifacts.iter().map(|a| a.path.clone()).collect::<BTreeSet<_>>() == expected
                    && phase.artifacts.len() == expected.len(),
                "phase artifact closure differs"
            );
            let rom = cartridge(report.kind, report.instruction_loop, phase.phase)?;
            ensure!(fs::read(directory.join("probe.sfc"))? == rom, "synthetic cartridge differs");
            ensure!(
                fs::read_to_string(directory.join("observer.lua"))? == observer(&directory.join("oracle.txt"))?,
                "observer differs"
            );
            let expected = movie_entries(&owner.join(&origins.movie_template.path), &rom)?;
            let mut archive = zip::ZipArchive::new(File::open(directory.join("probe.lsmv"))?)?;
            let mut actual = BTreeMap::new();
            for i in 0..archive.len() {
                let mut member = archive.by_index(i)?;
                let mut bytes = Vec::new();
                member.read_to_end(&mut bytes)?;
                ensure!(actual.insert(member.name().to_owned(), bytes).is_none(), "duplicate generated movie member");
            }
            ensure!(actual == expected, "generated movie differs from synthetic inputs");
            let comparison = compare(
                &oracle_rows(&directory.join("oracle.txt"))?,
                &candidate_rows(&directory.join("candidate.csv"))?,
            )?;
            ensure!(
                phase.comparison.as_ref() == Some(&comparison) && phase.error.is_none(),
                "phase result differs from retained observations"
            );
        } else {
            ensure!(phase.comparison.is_none(), "incomplete phase claims comparison");
        }
    }
    let complete = report.phases.len() == report.requested_phases && report.phases.iter().all(|p| p.complete);
    let passed = complete && report.phases.iter().all(|p| p.comparison.as_ref().is_some_and(|p| p.passed));
    ensure!(report.complete == complete && report.passed == passed, "HDMA terminal status differs");
    Ok(report)
}

fn snapshot_names(root: &Path, selected: fn(&Path) -> bool) -> Result<BTreeSet<String>> {
    fn visit(root: &Path, path: &Path, selected: fn(&Path) -> bool, names: &mut BTreeSet<String>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            ensure!(!path.is_symlink(), "source snapshot symlink");
            if path.is_dir() {
                visit(root, &path, selected, names)?;
            } else if selected(&path) {
                names.insert(path.strip_prefix(root)?.to_str().context("source path UTF-8")?.into());
            }
        }
        Ok(())
    }
    let mut names = BTreeSet::new();
    visit(root, root, selected, &mut names)?;
    Ok(names)
}

pub fn run(args: &[String]) -> Result<()> {
    if args.first().is_some_and(|arg| arg == "authenticate") {
        ensure!(args.len() == 2, "usage: probe-hdma authenticate REPORT.toml");
        let report = authenticate(Path::new(&args[1]))?;
        println!("{}", toml::to_string_pretty(&report)?);
        return Ok(());
    }
    let mut request = Request {
        root: std::env::current_dir()?,
        out: "out/sync/hdma-probes".into(),
        kind: Kind::Indirect,
        instruction_loop: Loop::Nop,
        phases: 8,
        jobs: 4,
        timeout_seconds: 600,
    };
    ensure!(args.len().is_multiple_of(2), "each HDMA option requires a value");
    for [option, value] in args.as_chunks::<2>().0 {
        match option.as_str() {
            "--kind" => {
                request.kind = match value.as_str() {
                    "direct" => Kind::Direct,
                    "indirect" => Kind::Indirect,
                    _ => anyhow::bail!("kind must be direct or indirect"),
                }
            }
            "--loop" => {
                request.instruction_loop = match value.as_str() {
                    "nop" => Loop::Nop,
                    "branch" => Loop::Branch,
                    _ => anyhow::bail!("loop must be nop or branch"),
                }
            }
            "--phases" => request.phases = value.parse()?,
            "--jobs" => request.jobs = value.parse()?,
            "--timeout" => request.timeout_seconds = value.parse()?,
            "--root" => request.root = value.clone().into(),
            "--out" => request.out = value.clone().into(),
            _ => anyhow::bail!("unknown HDMA option: {option}"),
        }
    }
    let report = operate(request)?;
    println!("{}", toml::to_string_pretty(&report)?);
    std::io::stdout().flush()?;
    ensure!(report.complete, "HDMA diagnostic incomplete: {}", report.owner.join("report.toml").display());
    if !report.passed {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cartridge_has_literal_register_program_and_indirect_table() {
        let rom = cartridge(Kind::Indirect, Loop::Branch, 2).unwrap();
        assert_eq!(
            &rom[..44],
            &[
                0x18, 0xfb, 0xe2, 0x30, 0xa9, 0x40, 0x8d, 0x70, 0x43, 0xa9, 0, 0x8d, 0x71, 0x43, 0xa9, 0, 0x8d, 0x72,
                0x43, 0xa9, 0x90, 0x8d, 0x73, 0x43, 0xa9, 0, 0x8d, 0x74, 0x43, 0xa9, 0, 0x8d, 0x77, 0x43, 0xad, 0x12,
                0x42, 0x10, 0xfb, 0xa9, 0x80, 0x8d, 0x0c, 0x42
            ]
        );
        assert_eq!(&rom[44..48], &[0xea, 0xea, 0x80, 0xfe]);
        assert_eq!(&rom[0x1000..0x100b], &[3, 0, 0x91, 0x83, 3, 0x91, 0, 0, 0, 0, 0]);
        assert_eq!(&rom[0x1100..0x1106], &[0x80; 6]);
        assert_eq!(&rom[0x7ffc..0x7ffe], &[0, 0x80]);
        assert_eq!(rom.len(), 65536);
        assert!(cartridge(Kind::Direct, Loop::Nop, 22).is_err());
        let direct = cartridge(Kind::Direct, Loop::Nop, 0).unwrap();
        assert_eq!(direct[5], 0);
        assert_eq!(&direct[44..48], &[0xea, 0xea, 0x80, 0xfc]);
        assert_eq!(&direct[0x1000..0x100b], &[3, 0x80, 0x83, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0]);
    }
    #[test]
    fn incomplete_and_clock_mismatches_are_independent_failures() {
        let original = [Row { pc: 0x008000, master_clock: 186 }, Row { pc: 0x008001, master_clock: 194 }];
        assert!(compare(&[], &original).is_err());
        assert!(compare(&original, &original[..1]).is_err());
        let mut actual = original.to_vec();
        actual[1].master_clock += 2;
        let result = compare(&original, &actual).unwrap();
        assert!(!result.passed);
        assert_eq!(result.compared_instructions, 1);
        actual[1] = original[1].clone();
        actual.push(Row { pc: 0x008002, master_clock: 202 });
        assert!(compare(&original, &actual).unwrap().passed);
    }
    #[test]
    fn trace_rollover_uses_ntsc_frame_clock_and_rejects_third_frame() {
        let temporary = std::env::temp_dir().join(format!(
            "sm-hdma-trace-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&temporary).unwrap();
        let path = temporary.join("oracle.txt");
        fs::write(&path, "ignored\n008000 nop V: 261 H: 1360\n008001 nop V: 0 H: 4\n").unwrap();
        assert_eq!(
            oracle_rows(&path).unwrap(),
            vec![Row { pc: 0x8000, master_clock: 357364 }, Row { pc: 0x8001, master_clock: 357372 }]
        );
        fs::write(
            &path,
            "008000 nop V: 261 H: 1360\n008001 nop V: 0 H: 4\n008002 nop V: 261 H: 1360\n008003 nop V: 0 H: 4\n",
        )
        .unwrap();
        assert!(oracle_rows(&path).is_err());
        fs::write(&path, "008000,186\n008001,186\n").unwrap();
        assert!(candidate_rows(&path).is_err());
        fs::remove_dir_all(temporary).unwrap();
    }
    #[test]
    fn sealed_capture_rejects_changes_and_path_escape() {
        let temporary = std::env::temp_dir().join(format!(
            "sm-hdma-seal-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&temporary).unwrap();
        let path = temporary.join("candidate.csv");
        fs::write(&path, "008000,186\n").unwrap();
        let item = artifact(&temporary, &path).unwrap();
        assert!(verify_artifact(&temporary, &item).is_ok());
        fs::write(&path, "008000,188\n").unwrap();
        assert!(verify_artifact(&temporary, &item).is_err());
        let escaped = Artifact { path: "../candidate.csv".into(), ..item };
        assert!(verify_artifact(&temporary, &escaped).is_err());
        fs::remove_dir_all(temporary).unwrap();
    }
}
