//! Inspect retained execution artifacts; liveness observations still attest caller inspection.
//! Validator fixtures are test data, never evidence that actual Codex sessions ran.
use super::super::{Artifact, Claim, Update, WorkResult, check_artifact, main_checkout, source_fingerprint};
use super::DrillObservation;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Execution {
    pub schema: String,
    pub frozen_input: Artifact,
    pub isolated_bridge: Artifact,
    pub sessions: Vec<SessionFiles>,
    pub controls: BTreeMap<String, Control>,
    pub integrations: Vec<Integration>,
    pub worker_budget: Artifact,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SessionFiles {
    pub launch: Artifact,
    pub log: Artifact,
    pub stderr: Artifact,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Control {
    pub commands: Vec<Artifact>,
    pub observations: Vec<Artifact>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Integration {
    pub result: Artifact,
    pub authentication: Artifact,
    pub cleanup: Artifact,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Frozen {
    pub schema: String,
    pub engine_binary: Artifact,
    pub engine_source_checkout: PathBuf,
    pub engine_source_commit: String,
    pub engine_source_fingerprint: String,
    pub engine_build: Artifact,
    pub isolated_fixture_checkout: PathBuf,
    pub isolated_fixture_baseline: String,
    pub harness_source: Artifact,
    pub created_at: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Build {
    pub schema: String,
    pub source_checkout: PathBuf,
    pub source_commit: String,
    pub source_fingerprint: String,
    pub binary_sha256: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Process {
    pub pid: u32,
    pub boot_id: String,
    pub start_ticks: u64,
    pub kernel_state: Option<String>,
    pub argv: Vec<String>,
    pub cwd: PathBuf,
    pub observed_at: String,
    pub status: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Launch {
    pub schema: String,
    pub name: String,
    pub session_id: String,
    pub pid: u32,
    pub process: Process,
    pub log: PathBuf,
    pub stderr: PathBuf,
    pub prompt: Artifact,
    pub observed_at: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CommandReceipt {
    pub schema: String,
    pub session_id: String,
    pub pid: u32,
    pub process: Process,
    pub route: String,
    pub request: Artifact,
    pub stdout: Artifact,
    pub stderr: Artifact,
    pub exit_code: i32,
    pub observed_at: String,
}

fn read<T: DeserializeOwned>(artifact: &Artifact) -> Result<T> {
    check_artifact(artifact)?;
    toml::from_str(&fs::read_to_string(&artifact.path)?)
        .with_context(|| format!("invalid drill artifact {}", artifact.path.display()))
}
fn seals(artifacts: &[Artifact]) -> Result<()> {
    for artifact in artifacts {
        check_artifact(artifact)?;
    }
    Ok(())
}
fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).current_dir(root).output()?;
    ensure!(output.status.success(), "drill Git evidence is unavailable");
    Ok(String::from_utf8(output.stdout)?.trim().into())
}
fn commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn same_path(left: &Path, right: &Path) -> Result<bool> {
    Ok(fs::canonicalize(left)? == fs::canonicalize(right)?)
}
fn process(process: &Process, root: &Path) -> Result<()> {
    ensure!(
        process.pid > 0
            && super::real_session_id(&process.boot_id)
            && process.start_ticks > 0
            && !process.argv.is_empty()
            && !process.observed_at.is_empty()
            && same_path(&process.cwd, root)?,
        "drill process lacks exact PID/boot/start/argv/cwd identity"
    );
    Ok(())
}
fn same_process(left: &Process, right: &toml::Value) -> Result<()> {
    ensure!(
        right.get("pid").and_then(toml::Value::as_integer) == Some(i64::from(left.pid)),
        "drill observation names another PID"
    );
    // A reaped process has no /proc start metadata; its original launch remains sealed.
    if right.get("status").and_then(toml::Value::as_str) != Some("not-running") {
        let right: Process = right.clone().try_into()?;
        ensure!(
            left.boot_id == right.boot_id
                && left.start_ticks == right.start_ticks
                && left.argv == right.argv
                && left.cwd == right.cwd,
            "drill process identity changed"
        );
    }
    Ok(())
}
fn field<'a>(value: &'a toml::Value, name: &str) -> Result<&'a toml::Value> {
    value.get(name).with_context(|| format!("drill receipt lacks {name}"))
}
fn text<'a>(value: &'a toml::Value, name: &str) -> Result<&'a str> {
    field(value, name)?.as_str().context("drill field must be text")
}
fn artifact(value: &toml::Value, name: &str) -> Result<Artifact> {
    Ok(field(value, name)?.clone().try_into()?)
}
fn list<'a>(value: &'a toml::Value, name: &str) -> Result<&'a Vec<toml::Value>> {
    field(value, name)?.as_array().context("drill field must be a list")
}

struct Executed {
    receipt: CommandReceipt,
    request: toml::Value,
    output: Option<toml::Value>,
    error: String,
}
impl Executed {
    fn operation(&self) -> Result<&str> {
        text(&self.request, "operation")
    }
    fn queue(&self) -> Result<&toml::Value> {
        field(&self.request, "queue")
    }
    fn success(&self) -> bool {
        self.receipt.exit_code == 0
    }
    fn output(&self) -> Result<&toml::Value> {
        self.output.as_ref().context("command has no typed successful output")
    }
    fn update(&self) -> Result<Update> {
        Ok(field(self.queue()?, "update")?.clone().try_into()?)
    }
}
fn command(artifact: &Artifact, frozen: &Frozen, isolated: &Path, sessions: &BTreeSet<String>) -> Result<Executed> {
    let receipt: CommandReceipt = read(artifact)?;
    ensure!(
        receipt.schema == "decomp-drill-command-v1"
            && sessions.contains(&receipt.session_id)
            && receipt.pid == receipt.process.pid
            && !receipt.observed_at.is_empty(),
        "command lacks a launched real session"
    );
    process(&receipt.process, isolated)?;
    ensure!(
        same_path(Path::new(&receipt.process.argv[0]), &frozen.engine_binary.path)?
            && receipt.process.argv.iter().any(|arg| arg == &receipt.route),
        "command did not use frozen engine/route"
    );
    ensure!(["decomp-authority", "decomp-session"].contains(&receipt.route.as_str()), "unknown drill command route");
    let request: toml::Value = read(&receipt.request)?;
    let queue = if receipt.route == "decomp-session" { field(&request, "queue")? } else { &request };
    ensure!(same_path(Path::new(text(queue, "checkout")?), isolated)?, "command used another repository");
    let authority: super::SessionAuthority = field(queue, "authority")?.clone().try_into()?;
    ensure!(
        authority.session_id == receipt.session_id
            && authority.generation > 0
            && !text(queue, "coordinator")?.is_empty(),
        "command authority differs from launched caller"
    );
    check_artifact(&receipt.stdout)?;
    check_artifact(&receipt.stderr)?;
    let error = fs::read_to_string(&receipt.stderr.path)?;
    let output = if receipt.exit_code == 0 {
        Some(read(&receipt.stdout)?)
    } else {
        ensure!(!error.trim().is_empty(), "rejected command lacks diagnostic output");
        None
    };
    Ok(Executed { receipt, request, output, error })
}
fn held(output: &toml::Value) -> Result<usize> {
    Ok(list(output, "jobs")?
        .iter()
        .filter(|job| matches!(job.get("state").and_then(toml::Value::as_str), Some("reserved" | "running")))
        .count())
}
fn deferred(command: &Executed, reason: &str) -> bool {
    command.success()
        && command.operation().ok() == Some("infrastructure-next")
        && command.output.as_ref().is_some_and(|out| {
            out.get("assignment").is_none()
                && out.get("deferred").and_then(toml::Value::as_table).is_some_and(|entries| {
                    entries.values().any(|entry| entry.as_str().is_some_and(|text| text.contains(reason)))
                })
        })
}

pub(crate) fn validate(root: &Path, checkout: &Path, drill: &DrillObservation) -> Result<()> {
    let isolated = fs::canonicalize(&drill.isolated_checkout).context("drill isolated repository is missing")?;
    ensure!(
        isolated != root
            && !isolated.starts_with(root)
            && main_checkout(&isolated)? == isolated
            && main_checkout(&isolated)? != main_checkout(checkout)?,
        "drill must use an independent isolated repository"
    );
    let execution: Execution = read(
        drill
            .execution
            .as_ref()
            .context("drill requires typed execution evidence; boolean/text summaries are insufficient")?,
    )?;
    ensure!(execution.schema == "decomp-drill-execution-v1", "unsupported drill execution schema");
    let frozen: Frozen = read(&execution.frozen_input)?;
    ensure!(
        frozen.schema == "decomp-drill-frozen-input-v1"
            && same_path(&frozen.engine_source_checkout, checkout)?
            && frozen.engine_source_commit == drill.head
            && frozen.engine_source_fingerprint == drill.source_fingerprint
            && source_fingerprint(&frozen.engine_source_checkout)? == frozen.engine_source_fingerprint
            && same_path(&frozen.isolated_fixture_checkout, &isolated)?
            && commit(&frozen.isolated_fixture_baseline)
            && !frozen.created_at.is_empty(),
        "drill frozen source differs from activation source"
    );
    git(&isolated, &["cat-file", "-e", &format!("{}^{{commit}}", frozen.isolated_fixture_baseline)])?;
    ensure!(
        Command::new("git")
            .args(["merge-base", "--is-ancestor", &frozen.isolated_fixture_baseline, "HEAD"])
            .current_dir(&isolated)
            .status()?
            .success(),
        "isolated baseline is not retained"
    );
    check_artifact(&frozen.engine_binary)?;
    check_artifact(&frozen.harness_source)?;
    let build: Build = read(&frozen.engine_build)?;
    ensure!(
        build.schema == "decomp-drill-engine-build-v1"
            && same_path(&build.source_checkout, checkout)?
            && build.source_commit == drill.head
            && build.source_fingerprint == drill.source_fingerprint
            && build.binary_sha256 == frozen.engine_binary.sha256,
        "frozen binary lacks matching retained source/build metadata"
    );
    let bridge: toml::Value = read(&execution.isolated_bridge)?;
    ensure!(
        text(&bridge, "schema")? == "decomp-legacy-owner-bridge-v1"
            && text(&bridge, "source_baseline")? == frozen.isolated_fixture_baseline,
        "drill isolated bridge baseline differs"
    );
    let mut sessions = BTreeSet::new();
    let mut launches = Vec::new();
    for files in &execution.sessions {
        let launch: Launch = read(&files.launch)?;
        ensure!(
            launch.schema == "decomp-drill-session-launch-v1"
                && super::real_session_id(&launch.session_id)
                && launch.pid == launch.process.pid
                && !launch.observed_at.is_empty()
                && !launch.name.is_empty(),
            "invalid real session launch"
        );
        process(&launch.process, &isolated)?;
        ensure!(
            launch.process.argv.iter().any(|arg| arg == "exec")
                && launch.process.argv.iter().any(|arg| arg == "--json")
                && Path::new(&launch.process.argv[0]).file_name().is_some_and(|name| name == "codex"),
            "launch is not a Codex JSON execution"
        );
        check_artifact(&files.log)?;
        check_artifact(&files.stderr)?;
        check_artifact(&launch.prompt)?;
        ensure!(
            same_path(&files.log.path, &launch.log)? && same_path(&files.stderr.path, &launch.stderr)?,
            "launch log seals name other files"
        );
        let mut started = Vec::new();
        for line in fs::read_to_string(&files.log.path)?.lines().filter(|line| !line.trim().is_empty()) {
            let event: serde_json::Value = serde_json::from_str(line).context("malformed sealed Codex JSONL")?;
            if event["type"] == "thread.started" {
                started.push(event["thread_id"].as_str().context("thread.started lacks id")?.to_owned());
            }
        }
        ensure!(started == vec![launch.session_id.clone()], "launch JSONL lacks its exact thread.started");
        sessions.insert(launch.session_id.clone());
        launches.push(launch);
    }
    ensure!(sessions.len() >= 2, "drill requires two distinct launched real sessions");
    let mut all_commands = Vec::new();
    for name in drill.controls.keys() {
        let control = execution.controls.get(name).with_context(|| format!("drill lacks typed control {name}"))?;
        ensure!(!control.commands.is_empty(), "typed drill control lacks executed commands: {name}");
        seals(&control.observations)?;
        let commands: Vec<_> = control
            .commands
            .iter()
            .map(|artifact| command(artifact, &frozen, &isolated, &sessions))
            .collect::<Result<_>>()?;
        let passed = match name.as_str() {
            "independent-sessions" => independent_reservations(&commands)?,
            "writable-overlap" => commands.iter().any(|cmd| deferred(cmd, "writable ownership")),
            "duplicate-ticket" => commands.iter().any(|cmd| deferred(cmd, "ticket ownership")),
            "stale-authority" => {
                commands.iter().any(|cmd| !cmd.success() && cmd.error.contains("stale session authority"))
            }
            "shared-capacity" => commands.iter().any(|cmd| {
                cmd.success()
                    && cmd.operation().ok() == Some("infrastructure-next")
                    && cmd
                        .output
                        .as_ref()
                        .is_some_and(|out| out.get("assignment").is_none() && held(out).is_ok_and(|n| n == 3))
            }),
            "crash-restart" => {
                launches.iter().any(|launch| {
                    launch.process.argv.iter().any(|arg| arg == "resume")
                        && launches.iter().any(|old| old.session_id == launch.session_id && old.pid != launch.pid)
                }) && commands.iter().any(|cmd| {
                    cmd.success()
                        && cmd.operation().ok() == Some("resume")
                        && cmd.output.as_ref().is_some_and(|out| {
                            out.get("status").and_then(toml::Value::as_str) == Some("reviewed-resume")
                        })
                })
            }
            "stopped-transfer" => transfer_control(&commands)?,
            "same-attempt-retry" => retry_control(&commands)?,
            "surviving-child" => surviving_control(&control.observations, &commands)?,
            "cleanup" | "integration-success" => {
                commands.iter().any(|cmd| {
                    cmd.success()
                        && cmd.operation().ok() == Some("infrastructure-record")
                        && matches!(cmd.update(), Ok(Update::Finished { result: WorkResult::Integrated { .. }, .. }))
                }) && !execution.integrations.is_empty()
            }
            _ => false,
        };
        ensure!(passed, "typed drill control did not pass: {name}");
        all_commands.extend(commands);
    }
    integrations(&execution.integrations, &isolated, &all_commands)?;
    budget(&execution.worker_budget, &frozen, &isolated)?;
    Ok(())
}

fn independent_reservations(commands: &[Executed]) -> Result<bool> {
    let mut reserved = Vec::new();
    for command in
        commands.iter().filter(|command| command.success() && command.operation().ok() == Some("infrastructure-next"))
    {
        let Some(assignment) = command.output()?.get("assignment") else {
            continue;
        };
        let claim: Claim = field(assignment, "claim")?.clone().try_into()?;
        let job: super::super::Job = field(assignment, "job")?.clone().try_into()?;
        if claim.id != job.id || claim.revision != job.revision || job.writable_paths.is_empty() {
            continue;
        }
        reserved.push((&command.receipt.session_id, claim, job.writable_paths));
    }
    Ok(reserved.iter().enumerate().any(|(index, (session, claim, paths))| {
        reserved.iter().skip(index + 1).any(|(other_session, other_claim, other_paths)| {
            session != other_session
                && claim != other_claim
                && paths
                    .iter()
                    .all(|path| other_paths.iter().all(|other| !path.starts_with(other) && !other.starts_with(path)))
        })
    }))
}

fn transfer_control(commands: &[Executed]) -> Result<bool> {
    for cmd in commands.iter().filter(|cmd| {
        cmd.success() && cmd.receipt.route == "decomp-authority" && cmd.operation().ok() == Some("transfer")
    }) {
        let transfer: super::RoleTransfer = field(&cmd.request, "transfer")?.clone().try_into()?;
        let proof: super::TransferObservation = read(&transfer.observation)?;
        if !super::stopped(&proof.status) || proof.generation == 0 {
            continue;
        }
        let output: super::Receipt = cmd.output()?.clone().try_into()?;
        if output.roles.iter().any(|role| {
            role.token == transfer.token
                && role.session_id == transfer.session_id
                && role.generation == proof.generation + 1
        }) {
            return Ok(true);
        }
    }
    Ok(false)
}
fn retry_control(commands: &[Executed]) -> Result<bool> {
    for (index, cmd) in commands
        .iter()
        .enumerate()
        .filter(|(_, cmd)| cmd.success() && cmd.operation().ok() == Some("infrastructure-record"))
    {
        let update = cmd.update()?;
        let Update::Finished { claim, result: WorkResult::Submitted { .. } } = &update else {
            continue;
        };
        for other in commands
            .iter()
            .skip(index + 1)
            .filter(|other| other.success() && other.operation().ok() == Some("infrastructure-record"))
        {
            if cmd.request != other.request {
                continue;
            }
            let matches = |out: &toml::Value| -> Result<bool> {
                Ok(list(out, "jobs")?.iter().any(|job| {
                    job.get("claim").cloned().and_then(|value| value.try_into::<Claim>().ok()).as_ref() == Some(claim)
                        && job.get("state").and_then(toml::Value::as_str) == Some("submitted")
                }))
            };
            if matches(cmd.output()?)? && matches(other.output()?)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
fn surviving_control(observations: &[Artifact], commands: &[Executed]) -> Result<bool> {
    for seal in observations {
        let value: toml::Value = read(seal)?;
        if value.get("schema").and_then(toml::Value::as_str) != Some("decomp-drill-surviving-child-v1") {
            continue;
        }
        let parent: Launch = read(&artifact(&value, "parent")?)?;
        let worker: Launch = read(&artifact(&value, "worker")?)?;
        let parent_observation: toml::Value = read(&artifact(&value, "parent_observation")?)?;
        let worker_observation: toml::Value = read(&artifact(&value, "worker_observation")?)?;
        same_process(&parent.process, &parent_observation)?;
        same_process(&worker.process, &worker_observation)?;
        ensure!(
            ["not-running", "completed", "interrupted", "failed", "stopped"]
                .contains(&text(&parent_observation, "status")?)
                && text(&worker_observation, "status")? == "running",
            "surviving-child process observations did not establish stopped parent/live child"
        );
        let queue_seal = artifact(&value, "queue")?;
        let queue: toml::Value = read(&queue_seal)?;
        let claim: Claim = field(&value, "claim")?.clone().try_into()?;
        ensure!(
            commands.iter().any(|cmd| cmd.success() && cmd.receipt.stdout == queue_seal),
            "surviving-child queue lacks executed command"
        );
        if list(&queue, "jobs")?.iter().any(|entry| {
            entry.get("claim").cloned().and_then(|value| value.try_into::<Claim>().ok()).as_ref() == Some(&claim)
                && entry.get("agent").and_then(toml::Value::as_str) == Some(worker.session_id.as_str())
                && entry.get("state").and_then(toml::Value::as_str) == Some("running")
        }) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn integrations(entries: &[Integration], isolated: &Path, commands: &[Executed]) -> Result<()> {
    ensure!(!entries.is_empty(), "drill lacks authenticated infrastructure integration");
    for entry in entries {
        let publication: toml::Value = read(&entry.result)?;
        ensure!(
            text(&publication, "schema")? == "decomp-infrastructure-result-v1",
            "drill integration is not maintained admission"
        );
        let admitted: Update = field(&publication, "admitted")?.clone().try_into()?;
        let requested: Update = field(&publication, "requested")?.clone().try_into()?;
        integration_recorded(&requested, &admitted, commands)?;
        let Update::Finished {
            claim,
            result: WorkResult::Integrated { submissions, evidence, authentication, cleanup },
        } = admitted
        else {
            anyhow::bail!("drill result is not integrated");
        };
        ensure!(
            !submissions.is_empty() && authentication == entry.authentication && cleanup == entry.cleanup,
            "integration pointers differ"
        );
        seals(&evidence)?;
        check_artifact(&artifact(&publication, "scope")?)?;
        let auth: toml::Value = read(&entry.authentication)?;
        ensure!(
            text(&auth, "schema")? == "decomp-infrastructure-authentication-v1"
                && text(&auth, "admission_domain")? == "infrastructure-v1"
                && field(&auth, "claim")?.clone().try_into::<Claim>()? == claim
                && field(&auth, "evidence")?.clone().try_into::<Vec<Artifact>>()? == evidence,
            "integration authentication tuple/evidence differs"
        );
        let source = text(&auth, "source_commit")?;
        ensure!(
            commit(source) && publication.get("source_commit").and_then(toml::Value::as_str) == Some(source),
            "integration source differs"
        );
        git(isolated, &["cat-file", "-e", &format!("{source}^{{commit}}")])?;
        ensure!(
            Command::new("git")
                .args(["merge-base", "--is-ancestor", source, "HEAD"])
                .current_dir(isolated)
                .status()?
                .success(),
            "integrated source ancestry is missing"
        );
        let mut gates = 0;
        for seal in &evidence {
            let Ok(value) = read::<toml::Value>(seal) else {
                continue;
            };
            if value.get("schema").and_then(toml::Value::as_str) == Some("decomp-infrastructure-gate-v1") {
                ensure!(
                    value.get("exit_code").and_then(toml::Value::as_integer) == Some(0)
                        && value.get("timed_out").and_then(toml::Value::as_bool) == Some(false)
                        && text(&value, "source_commit")? == source,
                    "integration gate did not pass on frozen source"
                );
                for key in ["stdout_sha256", "stderr_sha256"] {
                    ensure!(
                        evidence
                            .iter()
                            .any(|seal| Some(seal.sha256.as_str()) == value.get(key).and_then(toml::Value::as_str)),
                        "gate output is unsealed"
                    );
                }
                gates += 1;
            }
        }
        ensure!(gates > 0, "integration lacks real gate receipts");
        let clean: toml::Value = read(&entry.cleanup)?;
        ensure!(
            text(&clean, "status")? == "complete" && clean.get("remaining_action").is_none(),
            "drill cleanup incomplete"
        );
        let receipts: Vec<Artifact> = field(&clean, "receipts")?.clone().try_into()?;
        let dependencies: Vec<Artifact> = field(&clean, "retained_dependencies")?.clone().try_into()?;
        seals(&receipts)?;
        seals(&dependencies)?;
        ensure!(
            receipts.iter().any(|seal| read::<toml::Value>(seal).is_ok_and(|value| value
                .get("apply")
                .and_then(toml::Value::as_bool)
                == Some(true)
                && value.get("bytes").and_then(toml::Value::as_integer).is_some())),
            "cleanup lacks applied utility receipt"
        );
        for submission in submissions {
            let worktree =
                std::env::temp_dir().join(format!("sm-{}-{}-{}", claim.coordinator, submission.id, submission.attempt));
            // Registered worktrees are the authoritative retirement check, rather than an inferred coordinator path.
            let listing = git(isolated, &["worktree", "list", "--porcelain"])?;
            ensure!(
                !listing.lines().any(|line| line.starts_with("worktree ")
                    && line.contains(&format!("-{}-{}", submission.id, submission.attempt))),
                "integrated implementer worktree remains registered"
            );
            let _ = worktree;
        }
    }
    Ok(())
}
fn integration_recorded(requested: &Update, admitted: &Update, commands: &[Executed]) -> Result<()> {
    let Update::Finished { claim, result: WorkResult::Integrated { submissions, .. } } = admitted else {
        anyhow::bail!("drill result is not integrated");
    };
    for command in
        commands.iter().filter(|command| command.success() && command.operation().ok() == Some("infrastructure-record"))
    {
        if command.update()? != *requested {
            continue;
        }
        let jobs = list(command.output()?, "jobs")?;
        let complete = |claim: &Claim| {
            jobs.iter().any(|job| {
                job.get("claim").cloned().and_then(|value| value.try_into::<Claim>().ok()).as_ref() == Some(claim)
                    && job.get("state").and_then(toml::Value::as_str) == Some("complete")
                    && job.get("stale").and_then(toml::Value::as_bool) == Some(false)
                    && job.get("pending_revision").is_none()
            })
        };
        if !complete(claim) || !submissions.iter().all(complete) {
            continue;
        }
        let current: super::SessionAuthority = field(command.queue()?, "authority")?.clone().try_into()?;
        if text(command.queue()?, "coordinator")? != claim.coordinator {
            continue;
        }
        let mut roles = Vec::new();
        for role_receipt in
            commands.iter().filter(|command| command.success() && command.receipt.route == "decomp-authority")
        {
            let receipt: super::Receipt = role_receipt.output()?.clone().try_into()?;
            roles.extend(receipt.roles.into_iter().filter(|role| role.token == claim.coordinator));
        }
        if roles.iter().max_by_key(|role| role.generation).is_some_and(|role| {
            role.kind == super::RoleKind::Integrator
                && role.session_id == current.session_id
                && role.generation == current.generation
        }) {
            return Ok(());
        }
    }
    anyhow::bail!("drill integration lacks executed complete claim and current integrator role")
}
fn budget(seal: &Artifact, frozen: &Frozen, isolated: &Path) -> Result<()> {
    let value: toml::Value = read(seal)?;
    ensure!(
        text(&value, "schema")? == "decomp-drill-worker-budget-v1"
            && value.get("limit").and_then(toml::Value::as_integer) == Some(4)
            && value.get("held").and_then(toml::Value::as_integer) == Some(4)
            && value.get("contended").and_then(toml::Value::as_bool) == Some(true),
        "drill shared process budget did not pass"
    );
    let evidence: Vec<Artifact> = field(&value, "evidence")?.clone().try_into()?;
    seals(&evidence)?;
    let mut snapshot_ok = false;
    let mut waited = false;
    for seal in &evidence {
        let value: toml::Value = read(seal)?;
        match value.get("schema").and_then(toml::Value::as_str) {
            Some("decomp-drill-worker-budget-snapshot-v1") => {
                ensure!(
                    same_path(Path::new(text(&value, "root")?), &isolated.join("out/pipeline"))?
                        && value.get("limit").and_then(toml::Value::as_integer) == Some(4),
                    "lease snapshot used another budget"
                );
                let held = list(&value, "held")?;
                ensure!(held.len() == 4, "lease snapshot lacks four live leases");
                let mut identities = BTreeSet::new();
                for holder in held.iter().chain(std::iter::once(field(&value, "waiter")?)) {
                    let observed: Process = field(holder, "process")?.clone().try_into()?;
                    process(&observed, isolated)?;
                    ensure!(
                        observed.status == "running"
                            && identities.insert((observed.pid, observed.start_ticks))
                            && same_path(Path::new(&observed.argv[0]), &frozen.engine_binary.path)?
                            && observed.argv.iter().any(|arg| arg == "lease"),
                        "invalid live lease identity"
                    );
                    if let Some(receipt) = holder.get("lease_receipt") {
                        let receipt: Artifact = receipt.clone().try_into()?;
                        let lease: toml::Value = read(&receipt)?;
                        ensure!(
                            lease.get("worker_wait_seconds").and_then(toml::Value::as_float).is_some(),
                            "missing acquired native lease receipt"
                        );
                    }
                }
                ensure!(
                    field(&value, "waiter")?.get("acquired").and_then(toml::Value::as_bool) == Some(false),
                    "fifth lease was already acquired"
                );
                snapshot_ok = true;
            }
            Some("decomp-drill-lease-finished-v1") => {
                ensure!(value.get("exit_code").and_then(toml::Value::as_integer) == Some(0), "lease process failed");
                let lease: toml::Value = read(&artifact(&value, "lease_receipt")?)?;
                waited |= lease
                    .get("worker_wait_seconds")
                    .and_then(toml::Value::as_float)
                    .is_some_and(|seconds| seconds > 0.0);
            }
            _ => (),
        }
    }
    ensure!(snapshot_ok && waited, "budget evidence lacks simultaneous contention and completed native wait");
    Ok(())
}
