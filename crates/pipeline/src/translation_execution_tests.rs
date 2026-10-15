use super::*;
use std::{
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

pub(crate) struct Project {
    pub(crate) root: PathBuf,
    pub(crate) installation: Installation,
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("/usr/bin/git")
        .args(["-c", "core.hooksPath=/dev/null"])
        .args(args)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .current_dir(root)
        .output()?;
    ensure!(output.status.success(), "proof Git failed: {}", String::from_utf8_lossy(&output.stderr));
    Ok(String::from_utf8(output.stdout)?.trim().into())
}

fn installed(path: PathBuf) -> Result<InstalledFile> {
    Ok(InstalledFile { sha256: digest(&path)?, path })
}

pub(crate) fn policy(
    source: &Path,
    baseline: String,
    files: &[&str],
    dependencies: String,
    verification: String,
    pinned: String,
) -> Result<FrozenPolicy> {
    Ok(FrozenPolicy {
        schema: "translation-execution-policy-v1".into(),
        version: 1,
        scope_id: "complete-deterministic-supervisor-proof".into(),
        claim: ClaimLineage {
            id: "isolated-proof-original-claim".into(),
            revision: "reviewed-scope-v1".into(),
            attempt: 1,
            coordinator: "isolated-proof-coordinator".into(),
        },
        baseline_commit: baseline,
        pinned_c_sha256: pinned,
        dependencies_sha256: dependencies,
        verification_sha256: verification,
        adapter_sha256: "0".repeat(64),
        approved_files: files
            .iter()
            .map(|name| Ok(((*name).into(), digest(&source.join(name))?)))
            .collect::<Result<_>>()?,
        verification_stages: vec![VerificationStage::Focused],
        budgets: Budgets {
            workers: 4,
            max_attempts: 3,
            max_evidence_bytes: 8 * 1024 * 1024 * 1024,
            gate_timeout_seconds: 360,
        },
        rewrite_limits: RewriteLimits {
            max_context_bytes: 256 * 1024,
            max_response_bytes: 256 * 1024,
            max_file_bytes: 128 * 1024,
            max_total_bytes: 256 * 1024,
        },
    })
}

impl Project {
    pub(crate) fn new() -> Result<Self> {
        let root = std::env::temp_dir().join(format!(
            "sm-complete-supervisor-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let repository = root.join("source");
        fs::create_dir_all(repository.join("src"))?;
        fs::create_dir_all(repository.join("tests"))?;
        fs::write(repository.join("src/domain.rs"), "pub fn doubled(value: u32) -> u32 { value * 2 }\n")?;
        fs::write(
            repository.join("tests/frozen.rs"),
            "#[path=\"../src/domain.rs\"] mod domain;\n#[test] fn doubles_positive(){assert_eq!(domain::doubled(7),14);}\n#[test] fn handles_zero(){assert_eq!(domain::doubled(0),0);}\n",
        )?;
        let source_files = ["src/domain.rs", "tests/frozen.rs"]
            .iter()
            .map(|name| Ok(((*name).into(), digest(&repository.join(name))?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let names = vec!["doubles_positive".into(), "handles_zero".into()];
        let coverage = BTreeMap::from([("arithmetic domain".into(), names.clone())]);
        let provenance = infrastructure::InfrastructureProvenance {
            schema: "translation-infrastructure-provenance-v1".into(),
            scope_id: "arithmetic-supervisor-proof".into(),
            source_files: source_files.clone(),
            approved_files: vec!["src/domain.rs".into()],
            test_root: "tests/frozen.rs".into(),
            test_names: names.clone(),
            coverage: coverage.clone(),
            limitations: vec!["Reviewed in-process infrastructure test acceptance only.".into()],
        };
        let provenance_path = root.join("provenance.json");
        write_once(&provenance_path, &provenance)?;
        let contract = infrastructure::InfrastructureContract {
            schema: "translation-infrastructure-contract-v1".into(),
            scope_id: provenance.scope_id,
            source_files,
            approved_files: provenance.approved_files,
            test_root: provenance.test_root,
            test_names: names,
            coverage,
            provenance: Asset { file: provenance_path.clone(), sha256: digest(&provenance_path)? },
            compiler_limits: infrastructure::InfrastructureLimits {
                timeout_seconds: 120,
                writable_bytes: 32 * 1024 * 1024,
                stdout_bytes: 65536,
                stderr_bytes: 65536,
                stdin_bytes: 0,
            },
            tests_limits: infrastructure::InfrastructureLimits {
                timeout_seconds: 5,
                writable_bytes: 1024 * 1024,
                stdout_bytes: 65536,
                stderr_bytes: 65536,
                stdin_bytes: 0,
            },
        };
        let prepared = infrastructure::prepare(&repository, &contract)?;
        git(&repository, &["init", "-q"])?;
        git(&repository, &["config", "user.name", "Supervisor proof"])?;
        git(&repository, &["config", "user.email", "proof@invalid"])?;
        git(&repository, &["add", "src/domain.rs", "tests/frozen.rs"])?;
        git(&repository, &["commit", "-q", "-m", "Frozen proof source"])?;
        let baseline = git(&repository, &["rev-parse", "HEAD"])?;
        let policy = policy(
            &repository,
            baseline.clone(),
            &["src/domain.rs"],
            prepared.dependencies_sha256,
            prepared.verification_sha256,
            sha256("infrastructure-no-actual-c-v1"),
        )?;
        let helper = std::env::var_os("SM_TRANSLATION_TEST_HELPER")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp/sm-translation-execution-target/debug/sm-translation-sandbox"));
        let rewrite = std::env::var_os("SM_TRANSLATION_RECORDED_REWRITE").map(PathBuf::from).unwrap_or_else(|| {
            PathBuf::from("/tmp/sm-translation-execution-target/debug/sm-translation-recorded-rewrite")
        });
        fs::create_dir_all(root.join("installed"))?;
        fs::copy(&helper, root.join("installed/helper"))?;
        fs::copy(&rewrite, root.join("installed/rewrite"))?;
        fs::copy("/usr/bin/x86_64-linux-gnu-gcc-9", root.join("installed/gcc-9"))?;
        let rustc = Command::new("rustup").args(["which", "rustc"]).output()?;
        ensure!(rustc.status.success(), "installed Rust compiler missing");
        let mut installation = Installation {
            schema: "translation-supervisor-installation-v1".into(),
            id: "arithmetic-proof".into(),
            controller: installed(std::env::current_exe()?)?,
            helper: installed(root.join("installed/helper"))?,
            rustc: installed(PathBuf::from(String::from_utf8(rustc.stdout)?.trim()))?,
            c_compiler: installed(root.join("installed/gcc-9"))?,
            source_repository: repository.clone(),
            integration_repository: repository,
            integration_baseline: baseline,
            pinned_source: root.join("unused-pinned-c"),
            state_root: root.join("state"),
            pipeline_root: "/home/jim/sm-decomp/out/pipeline".into(),
            workers: 4,
            max_gate_bytes: 256 * 1024 * 1024,
            max_checkout_bytes: 64 * 1024 * 1024,
            max_context_bytes: 256 * 1024,
            rewrite: RewriteInstallation {
                executable: installed(root.join("installed/rewrite"))?,
                recorded: true,
                provider: None,
                limits: RunLimits {
                    timeout: Duration::from_secs(10),
                    writable_bytes: 1024 * 1024,
                    stdout_bytes: 256 * 1024,
                    stderr_bytes: 4096,
                    stdin_bytes: 256 * 1024,
                },
            },
            policy,
            contract: Contract::Infrastructure { contract },
            admission: Admission::IsolatedProof {
                limitations: vec!["Isolated complete supervisor proof; no production or gameplay acceptance.".into()],
            },
        };
        installation.policy.adapter_sha256 = installation_identity(&installation)?;
        Ok(Self { root, installation })
    }
    fn adapter(&self) -> Result<Adapter> {
        Adapter::new(self.installation.clone())
    }
    fn open(&self) -> Result<Supervisor<Adapter>> {
        let adapter = self.adapter()?;
        Supervisor::open(&adapter.run_root.join("journal"), self.installation.policy.clone(), adapter)
    }
    fn replacements(&self, view: &View, content: &str) -> SourceReplacements {
        let State::Rewrite { capsule } = &view.state else {
            panic!("expected rewrite capsule: {:?}", view.state);
        };
        SourceReplacements {
            capsule_id: capsule.id.clone(),
            files: vec![Replacement {
                path: capsule.files[0].path.clone(),
                base_sha256: capsule.files[0].sha256.clone(),
                content: content.into(),
            }],
        }
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        if std::thread::panicking() || std::env::var_os("SM_SUPERVISOR_KEEP_INFRA").is_some() {
            eprintln!("Retained infrastructure diagnostic source: {}", self.root.display());
        } else {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

pub(crate) fn records(root: &Path) -> Result<BTreeMap<PathBuf, (String, u64)>> {
    let mut files = BTreeMap::new();
    if !root.is_dir() {
        return Ok(files);
    }
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            files.extend(records(&path)?);
        } else {
            files.insert(path.clone(), (digest(&path)?, fs::metadata(&path)?.len()));
        }
    }
    Ok(files)
}

#[test]
fn closed_installation_rejects_overlapping_roots_and_metadata_budget_before_dispatch() -> Result<()> {
    let project = Project::new()?;
    let mut installation = project.installation.clone();
    installation.pipeline_root = installation.state_root.join("nested");
    installation.policy.adapter_sha256 = installation_identity(&installation)?;
    ensure!(Adapter::new(installation).is_err(), "overlapping protected resource roots admitted");
    let mut installation = project.installation.clone();
    installation.policy.budgets.max_evidence_bytes = 4096;
    installation.policy.adapter_sha256 = installation_identity(&installation)?;
    ensure!(Adapter::new(installation).is_err(), "missing journal resource reservation admitted");
    let mut installation = project.installation.clone();
    installation.policy.budgets.max_evidence_bytes = 200 * 1024 * 1024;
    installation.policy.adapter_sha256 = installation_identity(&installation)?;
    ensure!(Adapter::new(installation).is_err(), "attempt admitted without emergency finalization extent");
    let mut installation = project.installation.clone();
    installation.policy.rewrite_limits.max_total_bytes = installation.max_checkout_bytes;
    installation.policy.adapter_sha256 = installation_identity(&installation)?;
    ensure!(Adapter::new(installation).is_err(), "frozen replacement bound exceeded source/archive budget");
    ensure!(!project.installation.state_root.join("operations").exists(), "prelaunch refusal executed a producer");
    Ok(())
}

#[test]
fn partial_apply_and_incomplete_clone_failures_preserve_source_then_retire_once() -> Result<()> {
    for unborn in [false, true] {
        let project = Project::new()?;
        let adapter = project.adapter()?;
        let mut supervisor = project.open()?;
        let candidate = adapter.candidate(1);
        if unborn {
            fs::create_dir_all(candidate.join("src"))?;
            git(&candidate, &["init", "-q"])?;
            fs::write(candidate.join("src/domain.rs"), "pub fn doubled(value:u32)->u32{value * 3}\n")?;
        } else {
            checkout::create(
                &project.installation.source_repository,
                &candidate,
                &project.installation.policy.baseline_commit,
            )?;
            fs::write(candidate.join("src/domain.rs"), "pub fn doubled(value:u32)->u32{value * 3}\n")?;
        }
        let failed = supervisor.advance()?;
        ensure!(
            matches!(failed.state, State::Stopped { disposition: Disposition::Failed, .. }),
            "partial source failure did not finalize: {failed:?}"
        );
        ensure!(!candidate.exists(), "partial source checkout was not retired");
        let retained = adapter.attempt(1).join("partial-source/candidate/src/domain.rs");
        ensure!(fs::read_to_string(retained)?.contains("value * 3"), "partial replacement bytes lost");
        let before = records(&adapter.run_root)?;
        ensure!(
            project.open()?.advance()? == failed && records(&adapter.run_root)? == before,
            "partial failure resume launched new work or changed diagnosis"
        );
    }
    Ok(())
}

#[test]
fn surviving_creation_lease_prevents_failure_retirement_until_owner_stops() -> Result<()> {
    use std::os::unix::io::AsRawFd;
    let project = Project::new()?;
    let adapter = project.adapter()?;
    let mut supervisor = project.open()?;
    let candidate = adapter.candidate(1);
    fs::create_dir_all(candidate.join("src"))?;
    fs::write(candidate.join("src/domain.rs"), "partial source\n")?;
    let key = sha256(candidate.as_os_str().as_encoded_bytes());
    let lease_path = adapter.attempt(1).join(format!(".translation-create-{key}.lock"));
    let lease = OpenOptions::new().create_new(true).write(true).open(lease_path)?;
    ensure!(
        unsafe { libc::flock(lease.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0,
        "test could not establish exact surviving writer lease"
    );
    let pending = supervisor.cancel("operator stops an incomplete clone")?;
    ensure!(
        matches!(pending.state, State::Finalizing { .. }) && candidate.exists(),
        "surviving source owner was retired: {pending:?}"
    );
    drop(lease);
    let stopped = supervisor.advance()?;
    ensure!(
        matches!(stopped.state, State::Stopped { disposition: Disposition::Cancelled, .. }) && !candidate.exists(),
        "stopped source owner cleanup did not finish: {stopped:?}"
    );
    Ok(())
}

#[test]
fn complete_infrastructure_lifecycle_retires_authenticates_reuses_and_repairs_publication() -> Result<()> {
    let project = Project::new()?;
    let started = Instant::now();
    let mut supervisor = project.open()?;
    let rewrite = supervisor.advance()?;
    let replacement = project.replacements(&rewrite, "pub fn doubled(value:u32)->u32 { value.wrapping_mul(2) }\n");
    let complete = supervisor
        .run_with_rewrite(|adapter, context, capsule| adapter.rewrite(context, capsule, Some(&replacement)))?;
    let State::Complete { completion, publication } = &complete.state else {
        anyhow::bail!("supervisor stopped: {:?}", complete);
    };
    ensure!(completion.gates.len() == 2, "candidate/integrated gates missing");
    let adapter = project.adapter()?;
    let accepted_record: PublicationRecord = read(&adapter.attempt(1).join("publication/accepted.json"))?;
    adapter.authenticate_journal_extent(completion, &accepted_record)?;
    let final_record: FinalRecord = read(&adapter.attempt(1).join("finalization.json"))?;
    let full_manifest: serde_json::Value = read(&final_record.retention.manifest.path)?;
    let closure: Vec<Retained> = serde_json::from_value(full_manifest["retained"].clone())?;
    ensure!(
        closure.iter().any(|item| item.reference.path.starts_with(adapter.run_root.join("journal"))),
        "finalization omitted durable journal evidence bytes"
    );
    ensure!(!adapter.candidate(1).exists() && !adapter.integration(1).exists(), "source checkouts not retired");
    let captured = records(&adapter.run_root)?;
    let resumed = project.open()?.advance()?;
    ensure!(resumed == complete, "complete response retry changed accepted result");
    ensure!(records(&adapter.run_root)? == captured, "complete retry allocated another capture");
    ensure!(supervisor.submit(replacement)? == complete, "identical rewrite submission reran gates");
    let original_gate = adapter.gate_record(&completion.gates[0].seal.operation_key);
    let original_bytes = fs::read(&original_gate)?;
    let mut changed: GateRecord = read(&original_gate)?;
    changed.outcome.passed = false;
    fs::write(&original_gate, serde_json::to_vec(&changed)?)?;
    ensure!(project.open().is_err(), "tampered retained original gate admitted before new work");
    fs::write(&original_gate, original_bytes)?;
    ensure!(project.open()?.advance()? == complete, "valid retired gate could not be recovered");
    let projection = adapter.attempt(1).join("publication/stages.json");
    let original = fs::read(&projection)?;
    fs::remove_file(&projection)?;
    ensure!(project.open()?.inspect().is_err(), "read-only inspection accepted missing projection");
    ensure!(project.open()?.advance()? == complete, "resume could not repair accepted projection");
    ensure!(
        fs::read(&projection)? == original && records(&adapter.run_root)? == captured,
        "publication repair changed original captured evidence"
    );
    fs::write(&projection, b"[]")?;
    ensure!(project.open()?.advance().is_err(), "changed projection was silently repaired");
    fs::write(&projection, &original)?;
    let accepted = adapter.attempt(1).join("publication/accepted.json");
    let accepted_bytes = fs::read(&accepted)?;
    fs::write(&accepted, b"{}")?;
    ensure!(project.open()?.inspect().is_err(), "accepted publication tamper admitted");
    fs::write(&accepted, accepted_bytes)?;
    ensure!(project.open()?.inspect()? == complete, "retained publication failed restored authentication");
    println!(
        "Complete infrastructure supervisor: {:.3}s, {} gates, {} final bytes, {} publication bytes, {} reclaimed bytes",
        started.elapsed().as_secs_f64(),
        completion.gates.len(),
        completion.finalization.retained_bytes,
        publication.retained_bytes,
        completion.finalization.reclaimed_bytes
    );
    Ok(())
}

#[test]
fn failed_candidate_retirement_restores_exact_retry_then_repair_without_automatic_gates() -> Result<()> {
    let project = Project::new()?;
    let mut supervisor = project.open()?;
    let rewrite = supervisor.advance()?;
    let replacements = project.replacements(&rewrite, "pub fn doubled(value:u32)->u32 { value.wrapping_mul(3) }\n");
    let failed = supervisor
        .run_with_rewrite(|adapter, context, capsule| adapter.rewrite(context, capsule, Some(&replacements)))?;
    ensure!(
        matches!(failed.state, State::Stopped { disposition: Disposition::Failed, .. }),
        "failed test did not stop: {failed:?}"
    );
    let adapter = project.adapter()?;
    ensure!(!adapter.candidate(1).exists(), "failed candidate checkout was retained");
    let captured = records(&adapter.run_root)?;
    ensure!(
        project.open()?.advance()? == failed && records(&adapter.run_root)? == captured,
        "stopped resume evaluated unchanged source"
    );
    let retry = supervisor.retry("operator deliberately repeats the failed candidate")?;
    ensure!(
        retry.attempt == 2 && matches!(retry.state, State::Stopped { .. }),
        "operator retry did not preserve failed source: {retry:?}"
    );
    ensure!(!adapter.candidate(2).exists(), "retry checkout not retired");
    let repair = supervisor.repair("operator requests a changed implementation")?;
    let State::Rewrite { capsule } = &repair.state else {
        anyhow::bail!("repair did not restore source: {repair:?}");
    };
    ensure!(capsule.files[0].content.contains("wrapping_mul(3)"), "repair did not restore exact failed implementation");
    let replacements = project.replacements(&repair, "pub fn doubled(value:u32)->u32 { value.wrapping_mul(2) }\n");
    let complete = supervisor
        .run_with_rewrite(|adapter, context, capsule| adapter.rewrite(context, capsule, Some(&replacements)))?;
    ensure!(matches!(complete.state, State::Complete { .. }), "changed repair did not complete: {complete:?}");
    ensure!(!adapter.candidate(3).exists(), "repaired checkout not retired");
    if let State::Complete { completion, .. } = &complete.state {
        let final_record: FinalRecord = read(&adapter.attempt(3).join("finalization.json"))?;
        let intent: FinalIntent = read(&adapter.attempt(3).join("finalization-intent.json"))?;
        ensure!(intent.prior_finalizations.len() == 2, "later success omitted retained failures");
        let first = adapter.attempt(1).join("finalization.json");
        let original = fs::read(&first)?;
        fs::write(&first, b"{}")?;
        ensure!(project.open().is_err(), "later successful history accepted tampered first failure");
        fs::write(&first, original)?;
        adapter.authenticate(&project.installation.policy, &final_record.request, &completion.finalization)?;
        ensure!(project.open()?.advance()? == complete, "repaired terminal retry failed authentication");
    }
    Ok(())
}

#[test]
fn failed_rewrite_and_preparation_failure_finalize_once_without_candidate_acceptance() -> Result<()> {
    let project = Project::new()?;
    let mut supervisor = project.open()?;
    supervisor.advance()?;
    let failed = supervisor.fail_rewrite("bounded restricted worker failure")?;
    ensure!(
        matches!(failed.state, State::Stopped { disposition: Disposition::Failed, .. }),
        "rewrite failure not finalized"
    );
    let adapter = project.adapter()?;
    ensure!(!adapter.candidate(1).exists(), "failed rewrite left checkout");
    assert_stopped_accounting(&adapter, &project.installation.policy)?;
    let captured = records(&adapter.run_root)?;
    ensure!(
        project.open()?.advance()? == failed && records(&adapter.run_root)? == captured,
        "rewrite failure resume wrote another result"
    );
    let mut installation = project.installation.clone();
    installation.state_root = project.root.join("missing-coverage-state");
    if let Contract::Infrastructure { contract } = &mut installation.contract {
        contract.coverage.clear();
    }
    installation.policy.adapter_sha256 = installation_identity(&installation)?;
    let adapter = Adapter::new(installation.clone())?;
    let mut supervisor = Supervisor::open(&adapter.run_root.join("journal"), installation.policy, adapter.clone())?;
    let failed = supervisor.advance()?;
    ensure!(
        matches!(failed.state, State::Stopped { .. }) && !adapter.candidate(1).exists(),
        "missing coverage did not stop before rewrite and cleanup: {failed:?}"
    );
    ensure!(!adapter.attempt(1).join("publication").exists(), "preparation failure published acceptance");
    assert_stopped_accounting(&adapter, &adapter.installed.policy)?;
    Ok(())
}

fn assert_stopped_accounting(adapter: &Adapter, policy: &FrozenPolicy) -> Result<()> {
    let path = adapter.attempt(1).join("finalization.json");
    let record: FinalRecord = read(&path)?;
    ensure!(record.request.disposition != Disposition::Complete, "accounting fixture is not stopped");
    let delta = execution::authenticate_finalization_tail(
        &adapter.run_root.join("journal"),
        policy,
        &record.context,
        &record.request,
        &record.result,
    )?;
    ensure!(
        record.journal_delta_bytes == Some(delta)
            && record.result.retained_bytes
                == record.retention.retained_bytes + record_bytes(&path)?.len() as u64 + delta,
        "stopped finalization omitted actual canonical result or terminal journal bytes"
    );
    adapter.authenticate(policy, &record.request, &record.result)
}

#[test]
fn cancelled_finalization_accounts_for_exact_terminal_bytes_and_refuses_whitespace_tampering() -> Result<()> {
    let project = Project::new()?;
    let mut supervisor = project.open()?;
    let stopped = supervisor.cancel("operator cancels before preparation")?;
    ensure!(
        matches!(stopped.state, State::Stopped { disposition: Disposition::Cancelled, .. }),
        "preparation cancellation did not finish: {stopped:?}"
    );
    let adapter = project.adapter()?;
    assert_stopped_accounting(&adapter, &project.installation.policy)?;
    let captured = records(&adapter.run_root)?;
    ensure!(
        project.open()?.advance()? == stopped && records(&adapter.run_root)? == captured,
        "cancelled resume created a producer or changed evidence"
    );
    let path = adapter.attempt(1).join("finalization.json");
    let record: FinalRecord = read(&path)?;
    let original = record_bytes(&path)?;
    let mut whitespace = original.clone();
    whitespace.push(b' ');
    fs::write(&path, &whitespace)?;
    ensure!(
        adapter.authenticate(&project.installation.policy, &record.request, &record.result).is_err(),
        "noncanonical final record accepted with incorrect retained bytes"
    );
    fs::write(&path, &original)?;
    let tail = fs::read_dir(adapter.run_root.join("journal"))?
        .map(|entry| Ok(entry?.path()))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|path| path.file_name().is_some_and(|name| name.to_string_lossy().starts_with("event-")))
        .max()
        .context("stopped journal terminal event missing")?;
    let terminal = fs::read(&tail)?;
    let mut whitespace = terminal.clone();
    whitespace.push(b' ');
    fs::write(&tail, &whitespace)?;
    ensure!(
        adapter.authenticate(&project.installation.policy, &record.request, &record.result).is_err(),
        "terminal JSON whitespace accepted with changed journal accounting"
    );
    fs::write(&tail, terminal)?;
    ensure!(
        project.open()?.advance()? == stopped && records(&adapter.run_root)? == captured,
        "restored cancellation failed authenticating original evidence"
    );
    Ok(())
}

#[test]
fn preparation_context_and_shared_worker_reservations_obey_inherited_bounds() -> Result<()> {
    let root = std::env::temp_dir().join(format!(
        "sm-supervisor-expired-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    ensure!(
        crate::acquire_lease_until(&root, 4, 0).is_err() && !root.exists(),
        "expired shared worker reservation created state"
    );
    let now = tool_execution::bounded::now_unix_ms()?;
    let worker_root = root.join("workers");
    fs::create_dir_all(&worker_root)?;
    let held = OpenOptions::new().create_new(true).append(true).open(worker_root.join("config.lock"))?;
    held.try_lock()?;
    let started = Instant::now();
    ensure!(
        crate::acquire_lease_until(&root, 4, now + 40).is_err() && started.elapsed() < Duration::from_secs(1),
        "shared configuration wait exceeded inherited deadline"
    );
    drop(held);
    fs::remove_dir_all(&root)?;
    let bank = "uint16 Routine(uint16 k) { return k; }\n";
    ensure!(
        recovered_definition_bounded(bank, "Routine", 8).is_err(),
        "recovered source allocated an over-budget context definition"
    );
    ensure!(
        recovered_definition_bounded(bank, "Routine", 4096)? == bank.trim_end(),
        "bounded recovered context changed source"
    );
    Ok(())
}

pub(crate) fn native_harness_sources() -> Vec<(&'static str, &'static str)> {
    crate::source_stamps::all()
}

pub(crate) fn copy_native_source_snapshot(current: &Path, destination: &Path) -> Result<()> {
    fn copy_port(current: &Path, destination: &Path, relative: &Path, depth: usize) -> Result<()> {
        ensure!(depth <= 32, "native proof source tree is too deep");
        for path in tool_execution::bounded::entries(&current.join(relative))? {
            tool_execution::bounded::safe(&path)?;
            let relative = path.strip_prefix(current)?;
            if path.is_dir() {
                copy_port(current, destination, relative, depth + 1)?;
            } else {
                fs::create_dir_all(destination.join(relative).parent().context("native source parent missing")?)?;
                tool_execution::bounded::immutable_bytes(
                    &destination.join(relative),
                    &tool_execution::bounded::read(&path)?,
                )?;
            }
        }
        Ok(())
    }
    copy_port(current, destination, Path::new("crates/port/src"), 0)?;
    for (name, source) in native_harness_sources() {
        let path = destination.join(name);
        fs::create_dir_all(path.parent().context("native harness parent missing")?)?;
        tool_execution::bounded::immutable_bytes(&path, source.as_bytes())?;
    }
    Ok(())
}

pub(crate) fn native_installation(root: &Path) -> Result<Installation> {
    crate::initialize_test_binary_identity()?;
    let offline = crate::offline::families()?;
    let current = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("workspace missing")?
        .parent()
        .context("workspace missing")?
        .to_path_buf();
    ensure!(!root.exists(), "native supervisor proof root already exists");
    let repository = root.join("source");
    fs::create_dir_all(repository.join("crates/pipeline/src"))?;
    fs::create_dir_all(repository.join("crates/port/src"))?;
    fs::create_dir_all(repository.join("tools/port"))?;
    let manifest_path = Path::new("tools/port/enemy-jump-instructions.smf");
    let manifest = offline_verification::fixtures::read(&current.join(manifest_path))?;
    fs::copy(current.join(manifest_path), repository.join(manifest_path))?;
    copy_native_source_snapshot(&current, &repository)?;
    fs::copy(current.join("tools/dependencies.toml"), repository.join("tools/dependencies.toml"))?;
    fs::write(
        repository.join("tools/port/batch.toml"),
        "[families]\nenemy-jump-instructions = \"family\"\n[profiles]\n",
    )?;
    let pinned = PathBuf::from("/home/jim/sm-decomp/deps/snesrev-sm");
    let inventory = root.join("source-inventory.toml");
    crate::source_inventory::run(&repository, &pinned, &inventory)?;
    let provenance: native::NativeProvenance = tool_execution::bounded::object(
        &current.join(".scratch/translation-scope-execution/native-gates/provenance/provenance.toml"),
    )?;
    let contract = offline.declare_native(native::NativeDeclaration {
        checkout: &repository,
        kind: native::NativeDeclarationKind::Family(manifest_path),
        source_inventory: Asset { file: inventory.clone(), sha256: digest(&inventory)? },
        provenance,
        compiler_limits: native::NativeLimits {
            timeout_seconds: 120,
            writable_bytes: 128 * 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 256 * 1024,
            stdin_bytes: 0,
        },
        fixture_limits: native::NativeLimits {
            timeout_seconds: 5,
            writable_bytes: 512 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            stdin_bytes: 0,
        },
        deadline_unix_ms: None,
        max_input_bytes: tool_execution::bounded::DEFAULT_BYTES,
    })?;
    let prepared = offline.prepare_native(
        native::NativePreparation::new(&repository, &pinned, &contract),
        &crate::translation_native::ScopeApproval,
    )?;
    git(&repository, &["init", "-q"])?;
    git(&repository, &["config", "user.name", "Native supervisor proof"])?;
    git(&repository, &["config", "user.email", "proof@invalid"])?;
    git(&repository, &["add", "crates", "tools"])?;
    git(&repository, &["commit", "-q", "-m", "Frozen complete enemy jump instruction proof"])?;
    let baseline = git(&repository, &["rev-parse", "HEAD"])?;
    let mut policy = policy(
        &repository,
        baseline.clone(),
        &[&manifest.rust_module],
        sealed(&vec![prepared.dependencies_sha256()])?,
        sealed(&vec![prepared.verification_sha256()])?,
        sealed(&vec![prepared.source()])?,
    )?;
    policy.scope_id = "enemy-jump-instructions-complete-supervisor-proof".into();
    policy.verification_stages = vec![VerificationStage::Focused, VerificationStage::Offline];
    policy.budgets.max_attempts = 1;
    policy.budgets.gate_timeout_seconds = 600;
    fs::create_dir_all(root.join("installed"))?;
    for (variable, name) in [("SM_TRANSLATION_TEST_HELPER", "helper"), ("SM_TRANSLATION_RECORDED_REWRITE", "rewrite")] {
        let source = std::env::var_os(variable).map(PathBuf::from).context("native proof installed tool missing")?;
        fs::copy(source, root.join("installed").join(name))?;
    }
    fs::copy("/usr/bin/x86_64-linux-gnu-gcc-9", root.join("installed/gcc-9"))?;
    let rustc = Command::new("rustup").args(["which", "rustc"]).output()?;
    ensure!(rustc.status.success(), "installed Rust compiler missing");
    let mut installation = Installation {
        schema: "translation-supervisor-installation-v1".into(),
        id: "native-complete-proof".into(),
        controller: installed(std::env::current_exe()?)?,
        helper: installed(root.join("installed/helper"))?,
        rustc: installed(PathBuf::from(String::from_utf8(rustc.stdout)?.trim()))?,
        c_compiler: installed(root.join("installed/gcc-9"))?,
        source_repository: repository.clone(),
        integration_repository: repository.clone(),
        integration_baseline: baseline,
        pinned_source: pinned,
        state_root: root.join("state"),
        pipeline_root: "/home/jim/sm-decomp/out/pipeline".into(),
        workers: 4,
        max_gate_bytes: 1024 * 1024 * 1024,
        max_checkout_bytes: 128 * 1024 * 1024,
        max_context_bytes: 256 * 1024,
        rewrite: RewriteInstallation {
            executable: installed(root.join("installed/rewrite"))?,
            recorded: true,
            provider: None,
            limits: RunLimits {
                timeout: Duration::from_secs(20),
                writable_bytes: 1024 * 1024,
                stdout_bytes: 256 * 1024,
                stderr_bytes: 4096,
                stdin_bytes: 256 * 1024,
            },
        },
        policy,
        contract: Contract::Native {
            contracts: vec![contract],
            catalog_sha256: digest(&repository.join("tools/port/batch.toml"))?,
            focused_family: manifest.family,
        },
        admission: Admission::IsolatedProof {
            limitations: vec![
                "Closed enemy-jump-instructions catalog: 16 actual-C fixture pairs over two routines.".into(),
                "Original-ROM parity, Live replay, other families and production claim acceptance remain separate."
                    .into(),
            ],
        },
    };
    installation.policy.adapter_sha256 = installation_identity(&installation)?;
    write_once(&root.join("installation.json"), &installation)?;
    Ok(installation)
}

#[test]
#[ignore = "bounded complete real actual-C supervisor proof; requires pinned source, ROM and confinement"]
fn complete_native_supervisor_rewrite_verify_integrate_retire_publish_and_recover() -> Result<()> {
    crate::initialize_test_binary_identity()?;
    let root = PathBuf::from(std::env::var("SM_SUPERVISOR_NATIVE_PROOF")?);
    let installation = native_installation(&root)?;
    let started = Instant::now();
    let adapter = Adapter::new(installation.clone())?;
    let mut supervisor =
        Supervisor::open(&adapter.run_root.join("journal"), installation.policy.clone(), adapter.clone())?;
    let complete = supervisor.run_with_rewrite(|adapter, context, capsule| {
        let original = &capsule.files[0];
        ensure!(original.content.contains("Ok(BASE + usize::from(k))"), "recorded native transformation missing");
        let replacements = SourceReplacements {
            capsule_id: capsule.id.clone(),
            files: vec![Replacement {
                path: original.path.clone(),
                base_sha256: original.sha256.clone(),
                content: original.content.replace("Ok(BASE + usize::from(k))", "Ok(usize::from(k) + BASE)"),
            }],
        };
        adapter.rewrite(context, capsule, Some(&replacements))
    })?;
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let State::Complete { completion, publication } = &complete.state else {
        write_once(&root.join("stopped.json"), &complete)?;
        bail!("complete native supervisor stopped: {complete:?}");
    };
    ensure!(
        completion.gates.len() == 4 && completion.gates.iter().all(|gate| gate.passed),
        "prescribed candidate/merged gates incomplete"
    );
    let retained: Vec<GateRecord> = read(&adapter.attempt(1).join("retained-gates.json"))?;
    let mut producers = BTreeSet::new();
    let mut referenced_producers = 0;
    let mut cases = 0;
    for gate in &retained {
        ensure!(gate.receipts.len() == 1, "complete family gate missing");
        let Receipt::Native { prepared, source, receipt } = &gate.receipts[0] else {
            bail!("native gate changed domain");
        };
        let authenticated = prepared.authenticate(native::NativeEvidence { checkout: source, receipt })?;
        let verdict = authenticated.report();
        ensure!(verdict.verified_cases == 16 && verdict.verified_routines == 2, "actual-C complete coverage differs");
        cases = verdict.verified_cases;
        referenced_producers += receipt.operation_count()?;
        producers.extend(
            authenticated
                .references()
                .iter()
                .filter(|reference| {
                    reference.path.starts_with(&receipt.operation_root)
                        && reference.path.file_name().is_some_and(|name| name == "result.json")
                })
                .map(|reference| reference.path.clone()),
        );
    }
    ensure!(
        producers.len() == 34 && referenced_producers == 136,
        "candidate/merged gates did not reuse exact fixture pairs"
    );
    let readiness: Vec<Receipt> = read(&adapter.attempt(1).join("retained-readiness.json"))?;
    let readiness_producers = readiness
        .iter()
        .map(|receipt| match receipt {
            Receipt::Native { receipt, .. } => receipt.operation_count(),
            _ => Ok(0),
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<usize>();
    ensure!(!adapter.candidate(1).exists() && !adapter.integration(1).exists(), "native producer checkout not retired");
    let completion_path = adapter.attempt(1).join("publication/completion.json");
    ensure!(
        authenticate_retained_proof(&root.join("installation.json"), &completion_path)? == installation.policy,
        "complete retained native proof failed authentication"
    );
    let captured = records(&adapter.run_root)?;
    ensure!(
        supervisor.advance()? == complete && records(&adapter.run_root)? == captured,
        "complete native resume executed another producer or allocated evidence"
    );
    let stages = adapter.attempt(1).join("publication/stages.json");
    let stage_bytes = fs::read(&stages)?;
    fs::remove_file(&stages)?;
    ensure!(supervisor.inspect().is_err(), "native read-only inspect accepted missing publication");
    ensure!(
        supervisor.advance()? == complete
            && fs::read(&stages)? == stage_bytes
            && records(&adapter.run_root)? == captured,
        "native publication recovery changed captured evidence"
    );
    let allocation = fs::read_dir(adapter.run_root.join("allocations"))?.try_fold(0u64, |total, entry| {
        let record: Allocation = read(&entry?.path())?;
        total.checked_add(record.bytes).context("proof allocation overflow")
    })?;
    let actual_bytes = captured.values().map(|(_, bytes)| *bytes).sum::<u64>();
    write_once(
        &root.join("proof.json"),
        &serde_json::json!({
            "schema": "translation-supervisor-measurement-v1", "elapsed_seconds": elapsed_seconds,
            "fixture_pairs": cases, "routines": 2, "gates": completion.gates.len(),
            "executed_gate_producers": producers.len(), "readiness_producers": readiness_producers,
            "reused_gate_producers": referenced_producers - producers.len(), "restricted_rewrite_producers": 1,
            "reserved_bytes": allocation, "allocated_logical_bytes": actual_bytes,
            "retained_finalization_bytes": completion.finalization.retained_bytes,
            "retained_publication_bytes": publication.retained_bytes,
            "reclaimed_checkout_bytes": completion.finalization.reclaimed_bytes,
            "completion": completion_path, "installation": root.join("installation.json"),
            "resume_allocated_bytes": 0, "original_rom_parity": "not-tested", "live_replay": "not-tested"
        }),
    )?;
    println!(
        "Complete native supervisor: {elapsed_seconds:.3}s, {cases} fixture pairs, {} gate producers, {} reused; {} retained bytes, {} checkout bytes reclaimed",
        producers.len(),
        referenced_producers - producers.len(),
        completion.finalization.retained_bytes,
        completion.finalization.reclaimed_bytes
    );
    Ok(())
}

#[test]
#[ignore = "read-only authentication of the retained complete supervisor proof; no producer execution"]
fn retained_complete_native_supervisor_authenticates_without_producers() -> Result<()> {
    crate::initialize_test_binary_identity()?;
    let root = PathBuf::from(std::env::var("SM_SUPERVISOR_NATIVE_PROOF")?);
    let installation: Installation = read(&root.join("installation.json"))?;
    let run_id = sha256(format!("translation-execution-run-v1:{}", sealed(&installation.policy)?));
    let run_root = installation.state_root.join(run_id);
    let captured = records(&run_root)?;
    authenticate_retained_proof(
        &root.join("installation.json"),
        &run_root.join("attempt-0001/publication/completion.json"),
    )?;
    ensure!(records(&run_root)? == captured, "read-only native proof authentication mutated retained state");
    Ok(())
}
