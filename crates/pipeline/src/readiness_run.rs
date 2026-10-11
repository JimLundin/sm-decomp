#![doc = " Immutable run ownership, private source snapshots and retained process outcomes."]
use crate::{atomic_toml, digest};
use anyhow::{Context, Result, ensure};
use pipeline_model::readiness::{
    ArtifactIdentity, EvidenceSeal, ProcessRecord, ProcessSeal, RUN_VERSION, RunManifest, SourceFile, SourceInventory,
    Verdict,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::os::unix::{
    fs::PermissionsExt,
    process::{CommandExt, ExitStatusExt},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use toml::Value;
static INTERRUPTED: AtomicBool = AtomicBool::new(false);
extern "C" fn interrupt(_: i32) {
    INTERRUPTED.store(true, Ordering::Relaxed);
}
pub(crate) fn install_interrupt_handler() {
    unsafe {
        signal(2, interrupt as *const () as usize);
    }
}
pub(crate) fn was_interrupted() -> bool {
    INTERRUPTED.load(Ordering::Relaxed)
}
const EXCLUDED: [&str; 9] = [".git", ".scratch", ".agents", ".codex", ".aws", "deps", "rom", "out", "target"];
pub(crate) fn identity<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    #[derive(Serialize)]
    struct Envelope<'a, T: Serialize + ?Sized> {
        value: &'a T,
    }
    Ok(format!("{:x}", Sha256::digest(toml::to_string(&Envelope { value })?.as_bytes())))
}
fn stamp() -> Result<String> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(utc(time.as_secs(), time.subsec_micros()))
}
fn utc(seconds: u64, micros: u32) -> String {
    let days = (seconds / 86400) as i64 + 719468;
    let era = days / 146097;
    let day = days - era * 146097;
    let year = (day - day / 1460 + day / 36524 - day / 146096) / 365;
    let day_of_year = day - (365 * year + year / 4 - year / 100);
    let month = (5 * day_of_year + 2) / 153;
    let date = day_of_year - (153 * month + 2) / 5 + 1;
    let month = month + if month < 10 { 3 } else { -9 };
    let year = year + era * 400 + i64::from(month <= 2);
    let hour = seconds % 86400 / 3600;
    let minute = seconds % 3600 / 60;
    let second = seconds % 60;
    format!("{year:04}-{month:02}-{date:02}T{hour:02}:{minute:02}:{second:02}.{micros:06}Z")
}
pub(crate) fn safe_artifact(owner: &Path, relative: &str) -> Result<PathBuf> {
    ensure!(
        !relative.is_empty() && Path::new(relative).components().all(|c| matches!(c, Component::Normal(_))),
        "unsafe evidence artifact path"
    );
    let mut path = owner.to_path_buf();
    ensure!(!path.is_symlink(), "evidence owner is a symlink");
    for c in Path::new(relative).components() {
        path.push(c);
        ensure!(!path.is_symlink(), "evidence artifact is a symlink");
    }
    Ok(path)
}
fn collect(root: &Path, dir: &Path, names: &mut BTreeSet<String>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let p = entry?.path();
        let name = p.file_name().unwrap().to_string_lossy();
        if ["target", "generated", "__pycache__"].contains(&name.as_ref()) {
            continue;
        }
        ensure!(!p.is_symlink(), "maintained input is a symlink: {}", p.display());
        if p.is_dir() {
            collect(root, &p, names)?;
        } else if ["rs", "toml", "patch", "lua", "sh", "yml", "yaml", "c", "h", "in", "lock"]
            .contains(&p.extension().and_then(|s| s.to_str()).unwrap_or(""))
            || ["Makefile", "justfile"].contains(&name.as_ref())
        {
            names.insert(p.strip_prefix(root)?.to_string_lossy().into_owned());
        }
    }
    Ok(())
}
pub(crate) fn source_inventory(root: &Path) -> Result<Vec<SourceFile>> {
    let tracked = Command::new("git").args(["ls-files", "-z"]).current_dir(root).output()?;
    ensure!(tracked.status.success(), "cannot inventory maintained Git inputs");
    let mut names = String::from_utf8(tracked.stdout)?
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    for name in ["crates", "tools", "config", ".cargo", ".github"] {
        collect(root, &root.join(name), &mut names)?;
    }
    for name in ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rustfmt.toml", "justfile", "movies/manifest.toml"]
    {
        if root.join(name).is_file() {
            names.insert(name.into());
        }
    }
    let mut files = Vec::new();
    for name in names {
        let relative = Path::new(&name);
        let parts = relative.components().map(|p| p.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>();
        let filename = relative.file_name().context("source filename missing")?.to_string_lossy();
        if EXCLUDED.contains(&parts[0].as_str())
            || parts.iter().any(|p| ["target", "generated", "__pycache__"].contains(&p.as_str()))
            || (parts[0] == "movies" && name != "movies/manifest.toml")
            || [".env", "credentials", "secrets"].iter().any(|prefix| filename.starts_with(prefix))
            || ["credentials", "id_rsa", "id_ed25519"].contains(&filename.as_ref())
            || ["pem", "key", "p12", "sfc", "smc", "lsmv", "inputs"]
                .contains(&relative.extension().and_then(|p| p.to_str()).unwrap_or("").to_ascii_lowercase().as_str())
        {
            continue;
        }
        let path = safe_artifact(root, &name)?;
        if !path.is_file() {
            continue;
        }
        files.push(SourceFile {
            path: name,
            sha256: digest(&path)?,
            mode: path.metadata()?.permissions().mode() & 0o777,
        });
    }
    Ok(files)
}
pub(crate) struct Run {
    pub root: PathBuf,
    pub path: PathBuf,
    pub manifest: RunManifest,
    pub checks: BTreeMap<String, Value>,
    sources: Option<Vec<SourceFile>>,
    guarded: BTreeMap<PathBuf, String>,
    interrupted: bool,
}
impl Run {
    pub fn record_typed_check<T: Serialize>(&mut self, name: &str, report: &T) -> Result<()> {
        self.record_check(name, Value::try_from(report)?)
    }
    pub fn create(root: &Path, kind: &str, scope: impl Into<Value>, parent: Option<&str>) -> Result<Self> {
        install_interrupt_handler();
        let root = fs::canonicalize(root)?;
        let id = format!("{}-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(), std::process::id());
        let path = root.join("out/runs").join(&id);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::create_dir(&path)?;
        for name in ["source", "generated", "logs", "checks", "diagnostics"] {
            fs::create_dir(path.join(name))?;
        }
        let scope = scope.into().as_table().context("run scope must be a TOML table")?.clone();
        let manifest = RunManifest {
            schema_version: RUN_VERSION,
            run_id: id,
            kind: kind.into(),
            scope,
            accepted_parent: parent.map(str::to_owned),
            started_utc: stamp()?,
            finished_utc: None,
            state: "running".into(),
            commands: Vec::new(),
            identities: BTreeMap::new(),
            check_evidence: BTreeMap::new(),
            process_evidence: BTreeMap::new(),
            reused_evidence: Vec::new(),
            build_settings: toml::Table::new(),
            contract_version: None,
            selection_baseline: None,
            increment: None,
            parity_build_inputs: BTreeMap::new(),
            selection_profile_origin: None,
            session_id: None,
            increment_id: None,
            candidate_run_id: None,
            expected_pointer: None,
        };
        let run = Self {
            root,
            path,
            manifest,
            checks: BTreeMap::new(),
            sources: None,
            guarded: BTreeMap::new(),
            interrupted: false,
        };
        run.save()?;
        fs::write(run.path.join("brief.md"), "# Not ready: incomplete\n\nRun allocated; checks have not completed.\n")?;
        Ok(run)
    }
    pub fn ensure_open(&self) -> Result<()> {
        ensure!(
            self.manifest.state == "running" && !self.path.join("verdict.toml").exists(),
            "completed evidence must never be overwritten"
        );
        Ok(())
    }
    pub(crate) fn allocated(owner: &Path, root: &Path) -> Result<Self> {
        let root = fs::canonicalize(root)?;
        let path = fs::canonicalize(owner)?;
        ensure!(path.parent() == Some(root.join("out/runs").as_path()), "allocated run escapes checkout");
        let manifest: RunManifest = pipeline_model::read(&path.join("manifest.toml"))?;
        ensure!(
            manifest.schema_version == RUN_VERSION
                && manifest.state == "running"
                && manifest.commands.is_empty()
                && manifest.identities.is_empty()
                && !path.join("verdict.toml").exists(),
            "validation intent is no longer an empty allocated run"
        );
        Ok(Self {
            root,
            path,
            manifest,
            checks: BTreeMap::new(),
            sources: None,
            guarded: BTreeMap::new(),
            interrupted: false,
        })
    }
    pub fn was_interrupted(&self) -> bool {
        self.interrupted || INTERRUPTED.load(Ordering::Relaxed)
    }
    pub fn save(&self) -> Result<()> {
        self.ensure_open()?;
        atomic_toml(&self.path.join("manifest.toml"), &self.manifest)
    }
    pub fn capture_source(&mut self) -> Result<()> {
        self.ensure_open()?;
        ensure!(self.sources.is_none(), "source snapshot already captured");
        let files = source_inventory(&self.root)?;
        for entry in &files {
            let target = safe_artifact(&self.path.join("source/files"), &entry.path)?;
            fs::create_dir_all(target.parent().unwrap())?;
            fs::copy(self.root.join(&entry.path), &target)?;
            ensure!(digest(&target)? == entry.sha256, "source changed while capturing: {}", entry.path);
        }
        ensure!(source_inventory(&self.root)? == files, "source changed during snapshot");
        let hash = identity(&files)?;
        let inventory = SourceInventory { schema_version: RUN_VERSION, sha256: hash.clone(), files: files.clone() };
        atomic_toml(&self.path.join("source/inventory.toml"), &inventory)?;
        self.manifest
            .identities
            .insert("source".into(), Value::Table(toml::toml! { sha256 = hash inventory = "source/inventory.toml" }));
        self.sources = Some(files);
        self.save()
    }
    pub fn restore(owner: &Path, destination: &Path) -> Result<()> {
        ensure!(!destination.exists(), "restore destination must not already exist");
        let inventory: SourceInventory = pipeline_model::read(&owner.join("source/inventory.toml"))?;
        ensure!(
            inventory.schema_version == RUN_VERSION && identity(&inventory.files)? == inventory.sha256,
            "source inventory corrupt"
        );
        let mut names = BTreeSet::new();
        for entry in &inventory.files {
            ensure!(names.insert(&entry.path), "duplicate source inventory path");
            let path = safe_artifact(&owner.join("source/files"), &entry.path)?;
            ensure!(
                digest(&path)? == entry.sha256 && path.metadata()?.permissions().mode() & 0o777 == entry.mode,
                "source snapshot corrupt: {}",
                entry.path
            );
        }
        fs::create_dir_all(destination)?;
        for entry in inventory.files {
            let target = safe_artifact(destination, &entry.path)?;
            fs::create_dir_all(target.parent().unwrap())?;
            fs::copy(safe_artifact(&owner.join("source/files"), &entry.path)?, &target)?;
            fs::set_permissions(target, fs::Permissions::from_mode(entry.mode))?;
        }
        Ok(())
    }
    pub fn guard(&mut self, path: &Path) -> Result<String> {
        let path = fs::canonicalize(path)?;
        let hash = digest(&path)?;
        if let Some(previous) = self.guarded.get(&path) {
            ensure!(*previous == hash, "input changed; run contaminated: {}", path.display());
        }
        self.guarded.insert(path, hash.clone());
        Ok(hash)
    }
    pub fn check_sources(&self) -> Result<()> {
        ensure!(
            self.sources.as_ref() == Some(&source_inventory(&self.root)?),
            "maintained source/config changed or snapshot absent; run contaminated"
        );
        for (path, hash) in &self.guarded {
            ensure!(path.is_file() && digest(path)? == *hash, "input changed; run contaminated: {}", path.display());
        }
        Ok(())
    }
    pub fn record_check(&mut self, name: &str, report: Value) -> Result<()> {
        self.ensure_open()?;
        ensure!(
            !name.is_empty() && name.bytes().all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c)),
            "unsafe check name"
        );
        ensure!(!self.checks.contains_key(name), "check evidence must not be overwritten");
        ensure!(report.is_table(), "check report must be a TOML table");
        let relative = format!("checks/{name}.toml");
        atomic_toml(&self.path.join(&relative), &report)?;
        self.manifest
            .check_evidence
            .insert(name.into(), EvidenceSeal { path: relative.clone(), sha256: digest(&self.path.join(relative))? });
        self.checks.insert(name.into(), report);
        self.save()
    }
    pub fn record_artifact(&mut self, name: &str, path: &Path, copy_to: Option<&str>) -> Result<ArtifactIdentity> {
        self.ensure_open()?;
        ensure!(!self.manifest.identities.contains_key(name), "artifact identity must not be overwritten");
        let mut record = ArtifactIdentity {
            sha256: digest(path)?,
            bytes: path.metadata()?.len(),
            path: path.to_owned(),
            evidence: None,
        };
        if let Some(relative) = copy_to {
            let target = safe_artifact(&self.path, relative)?;
            ensure!(!target.exists(), "artifact evidence already exists");
            fs::create_dir_all(target.parent().unwrap())?;
            fs::copy(path, &target)?;
            ensure!(digest(&target)? == record.sha256, "artifact changed while copying");
            record.evidence = Some(relative.into());
        } else if let Ok(relative) = path.strip_prefix(&self.path) {
            let relative = relative.to_str().context("artifact path UTF-8")?;
            safe_artifact(&self.path, relative)?;
            record.evidence = Some(relative.into());
        }
        self.manifest.identities.insert(name.into(), Value::try_from(&record)?);
        self.save()?;
        Ok(record)
    }
    pub fn stage(
        &mut self,
        name: &str,
        argv: &[String],
        cwd: &Path,
        env: &BTreeMap<String, String>,
        timeout: Duration,
        exit_statuses: &BTreeMap<i32, String>,
    ) -> Result<Value> {
        self.stage_inner(name, argv, cwd, env, timeout, exit_statuses, true)
    }
    #[doc = " A controller waits on leased child stages and must not hold a worker itself."]
    pub fn stage_controller(
        &mut self,
        name: &str,
        argv: &[String],
        cwd: &Path,
        env: &BTreeMap<String, String>,
        timeout: Duration,
    ) -> Result<Value> {
        self.stage_inner(name, argv, cwd, env, timeout, &BTreeMap::new(), false)
    }
    #[allow(clippy::too_many_arguments)]
    fn stage_inner(
        &mut self,
        name: &str,
        argv: &[String],
        cwd: &Path,
        env: &BTreeMap<String, String>,
        timeout: Duration,
        exit_statuses: &BTreeMap<i32, String>,
        worker: bool,
    ) -> Result<Value> {
        self.ensure_open()?;
        ensure!(!self.was_interrupted(), "run interrupted");
        ensure!(
            !self.checks.contains_key(name)
                && !name.is_empty()
                && name.bytes().all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c)),
            "invalid or duplicate stage"
        );
        ensure!(!argv.is_empty(), "empty stage command");
        let start = Instant::now();
        let mut report = ProcessRecord {
            command: argv.to_vec(),
            cwd: cwd.to_owned(),
            started_utc: stamp()?,
            stdout: format!("logs/{name}.stdout.log"),
            stderr: format!("logs/{name}.stderr.log"),
            returncode: None,
            signal: None,
            complete: false,
            status: "incomplete".into(),
            reason: None,
            worker_wait_seconds: 0.0,
            worker_lease: false,
            elapsed_seconds: 0.0,
            finished_utc: None,
        };
        self.manifest.commands.push(report.clone());
        self.save()?;
        let outcome = (|| -> Result<()> {
            self.check_sources()?;
            let shared = if let Some(path) = std::env::var_os("SM_PIPELINE_ROOT") {
                PathBuf::from(path)
            } else {
                let common =
                    Command::new("git").args(["rev-parse", "--git-common-dir"]).current_dir(&self.root).output()?;
                ensure!(common.status.success(), "cannot locate shared worker root");
                let common = fs::canonicalize(self.root.join(String::from_utf8(common.stdout)?.trim()))?;
                common.parent().context("shared Git parent missing")?.join("out/pipeline")
            };
            let workers = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".into()).parse()?;
            let (held, wait) = if worker {
                let (held, wait) = crate::acquire_lease(&shared, workers)?;
                (Some(held), wait)
            } else {
                (None, 0.0)
            };
            let _lease = held;
            report.worker_wait_seconds = wait;
            report.worker_lease = worker;
            let mut process = Command::new(&argv[0]);
            process
                .args(&argv[1..])
                .current_dir(cwd)
                .envs(env)
                .process_group(0)
                .stdout(Stdio::from(File::create(self.path.join(&report.stdout))?))
                .stderr(Stdio::from(File::create(self.path.join(&report.stderr))?));
            let mut child = process.spawn()?;
            let active = Instant::now();
            let status = loop {
                if let Some(status) = child.try_wait()? {
                    break status;
                }
                let interrupted = INTERRUPTED.load(Ordering::Relaxed);
                if interrupted || active.elapsed() >= timeout {
                    self.interrupted = interrupted;
                    unsafe { kill(-(child.id() as i32), 15) };
                    let grace = Instant::now();
                    while child.try_wait()?.is_none() && grace.elapsed() < Duration::from_secs(2) {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    unsafe { kill(-(child.id() as i32), 9) };
                    let status = child.wait()?;
                    report.reason = Some(if interrupted { "stage interrupted" } else { "stage timed out" }.into());
                    break status;
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            report.returncode = status.code().or_else(|| status.signal().map(|s| -s));
            report.signal = status.signal();
            let complete = status.code().is_some() && report.reason.is_none();
            report.complete = complete;
            report.status = if !complete {
                "incomplete"
            } else if status.success() {
                "pass"
            } else {
                exit_statuses.get(&status.code().unwrap()).map(String::as_str).unwrap_or("error")
            }
            .into();
            if !status.success() && report.reason.is_none() {
                report.reason = Some(format!("command exited with {status}"));
            }
            self.check_sources()?;
            Ok(())
        })();
        if let Err(error) = outcome {
            report.status = "error".into();
            report.complete = false;
            report.reason = Some(format!("{error:#}"));
        }
        report.elapsed_seconds = start.elapsed().as_secs_f64();
        report.finished_utc = Some(stamp()?);
        *self.manifest.commands.last_mut().unwrap() = report.clone();
        atomic_toml(&self.path.join(format!("logs/{name}.toml")), &report)?;
        self.manifest.process_evidence.insert(
            name.into(),
            ProcessSeal {
                report: digest(&self.path.join(format!("logs/{name}.toml")))?,
                stdout: digest(&self.path.join(&report.stdout)).ok(),
                stderr: digest(&self.path.join(&report.stderr)).ok(),
            },
        );
        let value = Value::try_from(&report)?;
        self.record_check(name, value.clone())?;
        Ok(value)
    }
    pub fn finalize(
        &mut self,
        status: &str,
        stage: Option<&str>,
        reason: Option<&str>,
        mandatory: &[String],
    ) -> Result<Value> {
        self.ensure_open()?;
        ensure!(
            ["pass", "divergence", "uncovered", "error", "incomplete"].contains(&status),
            "unknown terminal status"
        );
        let mut status = status.to_owned();
        let mut stage = stage.map(str::to_owned);
        let mut reason = reason.map(str::to_owned);
        if status == "pass"
            && let Some((name, report)) = self
                .checks
                .iter()
                .filter(|(_, r)| {
                    r.get("status").and_then(Value::as_str) != Some("pass")
                        || r.get("complete").and_then(Value::as_bool) != Some(true)
                })
                .min_by_key(|(_, r)| match r.get("status").unwrap_or(&Value::Boolean(false)).as_str() {
                    Some("error") => 0,
                    Some("incomplete") => 1,
                    Some("divergence") => 2,
                    Some("uncovered") => 3,
                    _ => 0,
                })
        {
            status = report["status"]
                .as_str()
                .filter(|s| ["error", "incomplete", "divergence", "uncovered"].contains(s))
                .unwrap_or("incomplete")
                .into();
            stage = Some(name.clone());
            reason = Some(
                report
                    .get("reason")
                    .and_then(Value::as_str)
                    .unwrap_or("recorded check failed or did not complete")
                    .into(),
            );
        }
        if status == "pass"
            && mandatory.iter().any(|name| {
                self.checks.get(name).is_none_or(|r| {
                    r.get("status").and_then(Value::as_str) != Some("pass")
                        || r.get("complete").and_then(Value::as_bool) != Some(true)
                })
            })
        {
            status = "incomplete".into();
            reason = Some("mandatory checks absent or incomplete".into());
        }
        let verdict = Verdict {
            schema_version: RUN_VERSION,
            run_id: self.manifest.run_id.clone(),
            status: status.clone(),
            stage,
            reason: reason.clone(),
            scope: self.manifest.scope.clone(),
            checks: self.checks.clone(),
            mandatory_checks: mandatory.to_vec(),
            acceptance_eligible: false,
            finished_utc: stamp()?,
            evidence: "manifest.toml".into(),
        };
        self.manifest.state = "complete".into();
        self.manifest.finished_utc = Some(verdict.finished_utc.clone());
        atomic_toml(&self.path.join("manifest.toml"), &self.manifest)?;
        fs::write(
            self.path.join("brief.md"),
            format!(
                "# Not ready: {status}\n\n{}\n\n[Manifest](manifest.toml) · [Verdict](verdict.toml) · [Logs](logs/)\n",
                reason.unwrap_or("checks complete".into())
            ),
        )?;
        atomic_toml(&self.path.join("verdict.toml"), &verdict)?;
        let verdict = Value::try_from(&verdict)?;
        Ok(verdict)
    }
}
unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
    fn signal(number: i32, handler: usize) -> usize;
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn project() -> Temp {
        let root = std::env::temp_dir()
            .join(format!("sm-native-readiness-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();
        assert!(Command::new("git").arg("init").arg("-q").arg(&root).status().unwrap().success());
        fs::create_dir_all(root.join("crates/demo/src")).unwrap();
        fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::write(root.join("crates/demo/src/lib.rs"), "// candidate\n").unwrap();
        Temp(root)
    }
    #[test]
    fn timestamps_are_utc_and_preserve_calendar_boundaries() {
        assert_eq!(utc(0, 1), "1970-01-01T00:00:00.000001Z");
        assert_eq!(utc(951782400, 0), "2000-02-29T00:00:00.000000Z");
        assert_eq!(utc(4107542400, 999999), "2100-03-01T00:00:00.999999Z");
    }
    #[test]
    fn snapshots_preserve_dirty_untracked_files_exclude_private_material_and_reject_corruption() {
        let root = project();
        assert!(
            Command::new("git").args(["add", "Cargo.toml", "crates"]).current_dir(&root.0).status().unwrap().success()
        );
        fs::write(root.0.join("crates/demo/src/new.rs"), "// untracked\n").unwrap();
        fs::write(root.0.join("crates/demo/.env"), "private").unwrap();
        fs::write(root.0.join("crates/demo/private.pem"), "private").unwrap();
        let mut run = Run::create(&root.0, "preflight", crate::report!({}), None).unwrap();
        run.capture_source().unwrap();
        assert_eq!(run.sources.as_ref().unwrap().len(), 3);
        let restore = root.0.join("restored");
        Run::restore(&run.path, &restore).unwrap();
        assert_eq!(fs::read_to_string(restore.join("crates/demo/src/new.rs")).unwrap(), "// untracked\n");
        assert!(!restore.join("crates/demo/.env").exists());
        assert!(Run::restore(&run.path, &restore).is_err());
        fs::write(run.path.join("source/files/crates/demo/src/lib.rs"), "corrupt").unwrap();
        assert!(Run::restore(&run.path, &root.0.join("bad")).is_err());
        assert!(!root.0.join("bad").exists());
        fs::write(root.0.join("crates/demo/src/lib.rs"), "contaminated").unwrap();
        assert!(run.check_sources().is_err());
    }
    #[test]
    fn failures_timeouts_mutation_and_missing_checks_prevent_success_and_completed_runs_are_immutable() {
        let root = project();
        let mut run = Run::create(&root.0, "readiness", crate::report!({}), None).unwrap();
        run.capture_source().unwrap();
        let env = BTreeMap::new();
        let report = run
            .stage(
                "failure",
                &["sh".into(), "-c".into(), "echo compiler-output; echo compiler-failure >&2; exit 7".into()],
                &root.0,
                &env,
                Duration::from_secs(5),
                &BTreeMap::new(),
            )
            .unwrap();
        assert_eq!(report["returncode"], toml::Value::try_from(7).unwrap());
        assert_eq!(fs::read_to_string(run.path.join(report["stdout"].as_str().unwrap())).unwrap(), "compiler-output\n");
        let report = run
            .stage(
                "timeout",
                &["sh".into(), "-c".into(), "sleep 5".into()],
                &root.0,
                &env,
                Duration::from_millis(20),
                &BTreeMap::new(),
            )
            .unwrap();
        assert_eq!(report["status"], toml::Value::try_from("incomplete").unwrap());
        run.record_check("apparently-good", crate :: report ! ({ "status" : "pass" , "complete" : true })).unwrap();
        let terminal = run.finalize("pass", None, None, &["apparently-good".into()]).unwrap();
        assert_eq!(terminal["status"], toml::Value::try_from("error").unwrap());
        assert_eq!(terminal["acceptance_eligible"], toml::Value::try_from(false).unwrap());
        let bytes = fs::read(run.path.join("verdict.toml")).unwrap();
        assert!(run.finalize("pass", None, None, &[]).is_err());
        assert_eq!(bytes, fs::read(run.path.join("verdict.toml")).unwrap());
        let mut run = Run::create(&root.0, "readiness", crate::report!({}), None).unwrap();
        run.capture_source().unwrap();
        let report = run
            .stage(
                "mutate",
                &["sh".into(), "-c".into(), "echo changed > crates/demo/src/lib.rs".into()],
                &root.0,
                &env,
                Duration::from_secs(5),
                &BTreeMap::new(),
            )
            .unwrap();
        assert_eq!(report["status"], toml::Value::try_from("error").unwrap());
        assert_eq!(report["complete"], toml::Value::try_from(false).unwrap());
    }
}
