//! One command freezes an approved ordered plan and drives installed supervisors.
//! An interruption resumes the same scope and request. Only explicit repair can
//! authorize a fresh model exchange; plan preparation has no model operation.
use crate::{
    emit, inputs,
    translation_execution::{self as execution, Installation},
};
use ::translation_execution::{
    model::{State, View, digest_valid, sealed, sha256},
    retention::{self, Reference},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    os::unix::{
        fs::{MetadataExt, OpenOptionsExt},
        io::AsRawFd,
    },
    path::{Path, PathBuf},
};

const MAX_RECORD: u64 = 4 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Scope {
    pub id: String,
    /// Dependencies must occur earlier in the declared order.
    pub after: Vec<String>,
    pub installation: Reference,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Plan {
    pub schema: String,
    pub id: String,
    pub state_root: PathBuf,
    pub scopes: Vec<Scope>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Prepared {
    schema: String,
    plan_sha256: String,
    plan: Reference,
    installations: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Outcome {
    schema: String,
    plan_sha256: String,
    scope_id: String,
    installation_sha256: String,
    view: View,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Report {
    pub schema: String,
    pub plan_sha256: String,
    pub state: String,
    pub completed_scopes: Vec<String>,
    pub current_scope: Option<String>,
    pub current: Option<View>,
    pub prepared: Reference,
}

fn name(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.bytes().all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
}

fn read<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T> {
    ensure!(path.is_absolute(), "launch configuration path must be absolute");
    serde_json::from_slice(&inputs::bounded::read(path)?).context("invalid launch JSON")
}

fn immutable<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    ensure!(bytes.len() as u64 <= MAX_RECORD, "launch record extent exceeded");
    inputs::bounded::safe(path)?;
    fs::create_dir_all(path.parent().context("launch record parent missing")?)?;
    inputs::immutable_bytes(path, &bytes)
}

impl Plan {
    pub(crate) fn validate(&self, path: &Path) -> Result<Vec<Installation>> {
        ensure!(
            self.schema == "translation-launch-plan-v1"
                && name(&self.id)
                && !self.scopes.is_empty()
                && self.scopes.len() <= 32,
            "unsupported or empty ordered launch plan"
        );
        inputs::bounded::safe(&self.state_root)?;
        ensure!(
            self.state_root.is_absolute() && !path.starts_with(&self.state_root),
            "launch state overlaps its plan input"
        );
        let mut seen = BTreeSet::new();
        let mut installed = Vec::new();
        let mut resources: Vec<PathBuf> = vec![self.state_root.clone()];
        let mut scopes = BTreeSet::new();
        for scope in &self.scopes {
            ensure!(name(&scope.id) && seen.insert(scope.id.clone()), "duplicate or invalid launch scope id");
            let mut dependencies = BTreeSet::new();
            ensure!(
                scope.after.iter().all(|id| id != &scope.id && seen.contains(id) && dependencies.insert(id)),
                "launch dependency missing, duplicated or out of order"
            );
            ensure!(
                digest_valid(&scope.installation.sha256)
                    && scope.installation.bytes <= MAX_RECORD
                    && !scope.installation.path.starts_with(&self.state_root),
                "launch installation reference extent or path invalid"
            );
            retention::authenticate_reference(&scope.installation, MAX_RECORD)?;
            let installation: Installation = read(&scope.installation.path)?;
            installation.validate()?;
            ensure!(
                installation.rewrite.provider.is_some() && !installation.rewrite.recorded,
                "launch requires a frozen model provider"
            );
            ensure!(
                scopes.insert(installation.policy.scope_id.clone()),
                "launch repeats the same approved Translation scope"
            );
            for root in [
                &installation.source_repository,
                &installation.integration_repository,
                &installation.state_root,
                &installation.pipeline_root,
                &installation.pinned_source,
            ] {
                ensure!(
                    !root.starts_with(&self.state_root) && !self.state_root.starts_with(root),
                    "launch state overlaps installed resources"
                );
            }
            for prior in &resources {
                ensure!(
                    !installation.state_root.starts_with(prior) && !prior.starts_with(&installation.state_root),
                    "launch scopes share a writable controller root"
                );
            }
            resources.push(installation.state_root.clone());
            installed.push(installation);
        }
        Ok(installed)
    }
}

fn lease(path: &Path) -> Result<File> {
    inputs::bounded::safe(path)?;
    fs::create_dir_all(path)?;
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.is_dir() && metadata.uid() == unsafe { libc::geteuid() } && metadata.mode() & 0o022 == 0,
        "launch state is foreign or writable by others"
    );
    let lock = path.join("lock");
    inputs::bounded::safe(&lock)?;
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(lock)?;
    ensure!(file.metadata()?.is_file() && file.metadata()?.nlink() == 1, "launch lease is aliased");
    ensure!(
        unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0,
        "another launch controller owns this plan"
    );
    Ok(file)
}

fn prepare(plan_path: &Path, plan: &Plan) -> Result<Prepared> {
    let plan_sha256 = sealed(plan)?;
    let root = &plan.state_root;
    let destination = root.join("prepared.json");
    let plan_reference = retention::seal_file(plan_path, MAX_RECORD)?;
    if destination.try_exists()? {
        let saved: Prepared = read(&destination)?;
        ensure!(
            saved.schema == "translation-launch-prepared-v1"
                && saved.plan_sha256 == plan_sha256
                && saved.plan == plan_reference
                && saved.installations == plan.scopes.iter().map(|s| s.installation.clone()).collect::<Vec<_>>(),
            "launch preparation differs from original plan"
        );
        for reference in std::iter::once(&saved.plan).chain(&saved.installations) {
            retention::authenticate_reference(reference, MAX_RECORD)?;
        }
        return Ok(saved);
    }
    let prepared = Prepared {
        schema: "translation-launch-prepared-v1".into(),
        plan_sha256,
        plan: plan_reference,
        installations: plan.scopes.iter().map(|s| s.installation.clone()).collect(),
    };
    immutable(&root.join("plan.json"), plan)?;
    immutable(&destination, &prepared)?;
    Ok(prepared)
}

fn authenticate_prepared(plan_path: &Path, plan: &Plan) -> Result<Prepared> {
    let saved: Prepared = read(&plan.state_root.join("prepared.json"))?;
    ensure!(
        saved.schema == "translation-launch-prepared-v1"
            && saved.plan_sha256 == sealed(plan)?
            && saved.plan == retention::seal_file(plan_path, MAX_RECORD)?
            && saved.installations == plan.scopes.iter().map(|s| s.installation.clone()).collect::<Vec<_>>(),
        "launch preparation changed"
    );
    let captured: Plan = read(&plan.state_root.join("plan.json"))?;
    ensure!(captured == *plan, "captured launch plan changed");
    Ok(saved)
}

fn completed_path(plan: &Plan, id: &str) -> PathBuf {
    plan.state_root.join("completed").join(format!("{id}.json"))
}

fn completed(plan: &Plan, scope: &Scope, installation: &Installation) -> Result<Option<View>> {
    let path = completed_path(plan, &scope.id);
    if !path.try_exists()? {
        return Ok(None);
    }
    let outcome: Outcome = read(&path)?;
    ensure!(
        outcome.schema == "translation-launch-scope-outcome-v1"
            && outcome.plan_sha256 == sealed(plan)?
            && outcome.scope_id == scope.id
            && outcome.installation_sha256 == scope.installation.sha256
            && matches!(outcome.view.state, State::Complete { .. }),
        "launch scope completion changed"
    );
    let inspected = execution::execute(installation.clone(), "inspect", None)?;
    ensure!(inspected == outcome.view, "launch completion differs from authenticated supervisor");
    Ok(Some(outcome.view))
}

pub(crate) fn run(plan_path: &Path, action: &str, selected: Option<&str>, reason: Option<&str>) -> Result<Report> {
    let plan: Plan = read(plan_path)?;
    let installations = plan.validate(plan_path)?;
    ensure!(matches!(action, "prepare" | "run" | "inspect" | "repair" | "retry" | "cancel"), "unknown launch action");
    let deliberate = matches!(action, "repair" | "retry" | "cancel");
    ensure!(
        deliberate == (selected.is_some() && reason.is_some())
            && (!deliberate || reason.is_some_and(|s| !s.trim().is_empty() && s.len() <= 4096)),
        "deliberate launch action requires exact scope and reason"
    );
    let _lease = if action == "inspect" { None } else { Some(lease(&plan.state_root)?) };
    let prepared =
        if action == "inspect" { authenticate_prepared(plan_path, &plan)? } else { prepare(plan_path, &plan)? };
    let mut report = Report {
        schema: "translation-launch-report-v1".into(),
        plan_sha256: prepared.plan_sha256,
        state: "prepared".into(),
        completed_scopes: vec![],
        current_scope: None,
        current: None,
        prepared: retention::seal_file(&plan.state_root.join("prepared.json"), MAX_RECORD)?,
    };
    if action == "prepare" {
        return Ok(report);
    }
    let mut completed_ids = BTreeSet::new();
    for (scope, installation) in plan.scopes.iter().zip(&installations) {
        ensure!(scope.after.iter().all(|id| completed_ids.contains(id)), "launch predecessor has not completed");
        if completed(&plan, scope, installation)?.is_some() {
            ensure!(selected != Some(scope.id.as_str()), "cannot retry, repair or cancel a completed launch scope");
            completed_ids.insert(scope.id.clone());
            report.completed_scopes.push(scope.id.clone());
            continue;
        }
        ensure!(
            !deliberate || selected == Some(scope.id.as_str()),
            "operator action must name the first unfinished scope"
        );
        let run_id = sha256(format!("translation-execution-run-v1:{}", sealed(&installation.policy)?));
        let has_journal = installation.state_root.join(run_id).join("journal").is_dir();
        if action == "inspect" && !has_journal {
            report.state = "ready".into();
            report.current_scope = Some(scope.id.clone());
            return Ok(report);
        }
        // Inspect authenticates without repairing; run authenticates completed
        // history and reconciles a surviving rewrite request before any send.
        let mut view = execution::execute(
            installation.clone(),
            if action == "inspect" {
                "inspect"
            } else if deliberate {
                action
            } else {
                "run"
            },
            reason,
        )?;
        if action == "repair" && matches!(view.state, State::Rewrite { .. }) {
            view = execution::execute(installation.clone(), "run", None)?;
        }
        if matches!(view.state, State::Complete { .. }) {
            if action != "inspect" {
                immutable(
                    &completed_path(&plan, &scope.id),
                    &Outcome {
                        schema: "translation-launch-scope-outcome-v1".into(),
                        plan_sha256: sealed(&plan)?,
                        scope_id: scope.id.clone(),
                        installation_sha256: scope.installation.sha256.clone(),
                        view: view.clone(),
                    },
                )?;
            }
            completed_ids.insert(scope.id.clone());
            report.completed_scopes.push(scope.id.clone());
            if deliberate {
                report.state = "ready".into();
                return Ok(report);
            }
        } else {
            report.state = if matches!(view.state, State::Stopped { .. }) { "stopped" } else { "pending" }.into();
            report.current_scope = Some(scope.id.clone());
            report.current = Some(view);
            return Ok(report);
        }
    }
    report.state = "complete".into();
    Ok(report)
}

pub(crate) fn cli(args: &[String]) -> Result<()> {
    if args.first().is_some_and(|value| value == "--request") {
        ensure!(
            args.len() == 3 && matches!(args[2].as_str(), "prepare" | "run" | "inspect"),
            "usage: translation-launch --request /absolute/REQUEST.json prepare|run|inspect"
        );
        let plan = crate::translation_launch_prepare::prepare(Path::new(&args[1]), args[2] == "inspect")?;
        let report = run(&plan.path, &args[2], None, None)?;
        emit(&report)?;
        ensure!(
            args[2] != "run" || report.state == "complete",
            "launch {} at {}; inspect or deliberately repair the retained attempt",
            report.state,
            report.current_scope.as_deref().unwrap_or("none")
        );
        return Ok(());
    }
    ensure!(
        args.len() >= 3 && args[0] == "--plan",
        "usage: translation-launch --plan /absolute/PLAN.json prepare|run|inspect|repair SCOPE REASON|retry SCOPE REASON|cancel SCOPE REASON"
    );
    let deliberate = matches!(args[2].as_str(), "repair" | "retry" | "cancel");
    ensure!(args.len() == if deliberate { 5 } else { 3 }, "unexpected launch arguments");
    let report = run(Path::new(&args[1]), &args[2], args.get(3).map(String::as_str), args.get(4).map(String::as_str))?;
    emit(&report)?;
    ensure!(
        args[2] != "run" || report.state == "complete",
        "launch {} at {}; retained original attempt; inspect or deliberately repair",
        report.state,
        report.current_scope.as_deref().unwrap_or("none")
    );
    Ok(())
}

#[cfg(test)]
#[path = "translation_launch_tests.rs"]
mod tests;
