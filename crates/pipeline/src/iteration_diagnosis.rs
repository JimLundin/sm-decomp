use crate::readiness_metadata::toml_value_decode;
use crate::{
    digest, family, lock, readiness_contract as contract, readiness_origins as origins,
    readiness_run::{Run, safe_artifact},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use toml::Value;
const VERSION: u32 = 1;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ObservationKind {
    FirstMismatch,
    Controls,
    Coverage,
    IndependentHardware,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Descriptor {
    pub run_path: PathBuf,
    pub evidence: String,
    pub sha256: String,
    pub process: String,
    pub process_sha256: String,
    pub executable: String,
    pub executable_sha256: String,
    pub kind: ObservationKind,
}
#[doc = " A canonical observable scalar. Its type prevents textual collisions such as"]
#[doc = " a numeric bus value and a string containing that number."]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fact {
    pub field: String,
    pub scalar_type: String,
    pub value: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Facts {
    pub kind: ObservationKind,
    pub values: Vec<Fact>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Observation {
    pub descriptor: Descriptor,
    pub facts: Facts,
    pub fingerprint: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Route {
    SetupBuildBudget,
    Coverage,
    IndependentHardwareInput,
    FailureNotReproduced,
    Increment,
    SharedPathsInputCacheHarness,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Failure {
    pub status: String,
    pub stage: String,
    pub route: Route,
    pub earliest_disagreement: Vec<Fact>,
    pub conclusion: String,
    pub next_experiment: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    DecisionRequired,
    PruneUnexecutedKeys,
    IndependentObservation,
    LocalizeFailure,
    RepairAndValidate,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Experiment {
    pub schema_version: u32,
    pub session_id: String,
    pub increment_id: String,
    pub sequence: u64,
    pub previous_sha256: Option<String>,
    pub candidate_run_id: String,
    pub candidate_path: PathBuf,
    pub candidate_manifest_sha256: String,
    pub candidate_verdict_sha256: String,
    pub candidate_source_sha256: String,
    pub experiment_source_sha256: String,
    pub accepted_parent: Option<String>,
    pub hypothesis: String,
    pub predicted_discriminator: String,
    pub result: String,
    pub observations: Vec<Observation>,
    pub new_discriminating_evidence: bool,
    pub consecutive_without_evidence: u64,
    pub decision_required: bool,
    pub failure: Failure,
    pub next_action: Action,
    pub choices: Vec<String>,
    pub run_path: PathBuf,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Reference {
    schema_version: u32,
    sequence: u64,
    run_path: PathBuf,
    manifest_sha256: String,
    verdict_sha256: String,
    experiment_sha256: String,
    previous_sha256: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Lineage {
    schema_version: u32,
    session_id: String,
    increment_id: String,
    accepted_parent: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Head {
    schema_version: u32,
    sequence: u64,
    sha256: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct History {
    pub increment_id: String,
    pub experiments: Vec<Experiment>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub checkout: PathBuf,
    pub increment_id: String,
    #[serde(default)]
    pub history: bool,
    pub session_id: Option<String>,
    pub candidate: Option<PathBuf>,
    pub hypothesis: Option<String>,
    pub discriminator: Option<String>,
    pub result: Option<String>,
    #[doc = " A TOML array of descriptors, supplied through the typed request."]
    #[serde(default)]
    pub evidence: Vec<Descriptor>,
}
#[derive(Serialize, Deserialize)]
struct Check {
    status: String,
    complete: bool,
    sequence: u64,
    sha256: String,
}
#[derive(Serialize)]
struct SourceChange {
    path: String,
    before: Option<String>,
    after: Option<String>,
}
#[derive(Serialize)]
struct SourceDiff {
    candidate_source_sha256: String,
    current_source_sha256: String,
    changed: Vec<SourceChange>,
}
fn safe_id(id: &str) -> Result<()> {
    ensure!(
        !id.is_empty() && id.len() <= 96 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b)),
        "session/increment ID must be stable safe text"
    );
    Ok(())
}
fn directory(root: &Path, increment: &str) -> Result<PathBuf> {
    safe_id(increment)?;
    safe_artifact(root, &format!("out/iteration-history/{increment}"))
}
fn write<T: Serialize>(path: &Path, value: &T, replace: bool) -> Result<()> {
    ensure!(!path.is_symlink(), "history metadata is a symlink");
    let text = toml::to_string_pretty(value)?;
    let temporary = path.with_extension(format!("{}.pending", std::process::id()));
    let mut output = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    let outcome = (|| -> Result<()> {
        output.write_all(text.as_bytes())?;
        output.sync_all()?;
        if replace {
            fs::rename(&temporary, path)?;
        } else {
            fs::hard_link(&temporary, path).context("immutable history metadata already exists")?;
            fs::remove_file(&temporary)?;
        }
        File::open(path.parent().context("metadata parent missing")?)?.sync_all()?;
        Ok(())
    })();
    if outcome.is_err() {
        let _ = fs::remove_file(temporary);
    }
    outcome
}
fn ignored(key: &str) -> bool {
    [
        "run_id",
        "candidate_run_id",
        "utc",
        "started_utc",
        "finished_utc",
        "elapsed_seconds",
        "total_seconds",
        "warm_replay_seconds",
        "binary_sha256",
        "source_sha256",
        "rerun",
        "evidence",
        "command",
        "path",
        "cache",
    ]
    .contains(&key)
}
fn facts(value: &Value) -> Vec<Fact> {
    fn visit(value: &Value, path: String, result: &mut Vec<Fact>) {
        match value {
            Value::Table(values) => {
                result.push(Fact { field: path.clone(), scalar_type: "object".into(), value: String::new() });
                for (key, value) in values {
                    if !ignored(key) {
                        let key = key.replace('~', "~0").replace('/', "~1");
                        visit(value, format!("{path}/{key}"), result);
                    }
                }
            }
            Value::Array(values) => {
                result.push(Fact { field: path.clone(), scalar_type: "array".into(), value: values.len().to_string() });
                for (index, value) in values.iter().enumerate() {
                    visit(value, format!("{path}/{index}"), result);
                }
            }
            _ => {
                let (scalar_type, value) = match value {
                    Value::Boolean(value) => ("bool", value.to_string()),
                    Value::Integer(value) => ("integer", value.to_string()),
                    Value::Float(value) => ("float", value.to_string()),
                    Value::Datetime(value) => ("datetime", value.to_string()),
                    Value::String(value) => ("string", value.clone()),
                    _ => unreachable!(),
                };
                result.push(Fact { field: path, scalar_type: scalar_type.into(), value });
            }
        }
    }
    let mut result = Vec::new();
    visit(value, String::new(), &mut result);
    result
}
fn fingerprint(facts: &Facts) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(toml::to_string(facts)?.as_bytes())))
}
fn terminal(owner: &Path) -> Result<(Value, Value, Value)> {
    let (manifest, inventory) = origins::snapshot(owner)?;
    let verdict = origins::document(&safe_artifact(owner, "verdict.toml")?)?;
    ensure!(
        manifest["state"] == crate::readiness_metadata::value(&"complete").expect("validated native metadata")
            && verdict["run_id"] == manifest["run_id"]
            && [Some("pass"), Some("divergence"), Some("uncovered"), Some("error"), Some("incomplete")]
                .contains(&verdict["status"].as_str()),
        "run lacks a preserved terminal verdict"
    );
    origins::owned(owner, &manifest)?;
    for (name, report) in verdict["checks"].as_table().context("terminal checks missing")? {
        let entry = &manifest["check_evidence"][name];
        let relative = family::field(entry, "path")?;
        let path = safe_artifact(owner, relative)?;
        let expected = family::field(entry, "sha256")?;
        ensure!(
            digest(&path)? == expected && origins::document(&path)? == *report,
            "terminal check differs from sealed evidence: {name}"
        );
    }
    Ok((manifest, inventory, verdict))
}
fn passing(report: &Value) -> bool {
    report.get("status").and_then(Value::as_str) == Some("pass")
        && report.get("complete").and_then(Value::as_bool) == Some(true)
}
fn reproduced(report: &Value) -> bool {
    report.get("status").and_then(Value::as_str) == Some("divergence")
        && report.get("complete").and_then(Value::as_bool) == Some(true)
        && report.get("first_mismatch").and_then(Value::as_table).is_some_and(|v| !v.is_empty())
}
fn matched(report: &Value, candidate: &Value) -> bool {
    candidate.get("start_input_index").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).is_some()
        && candidate.get("start_instruction").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).is_some()
        && candidate.get("identities").and_then(Value::as_table).is_some_and(|ids| !ids.is_empty())
        && ["start_input_index", "start_instruction", "identities"]
            .iter()
            .all(|key| report.get(key) == candidate.get(key))
        && if candidate.get("scope").and_then(Value::as_str) == Some("until-frame-end") {
            report
                .get("completed_instructions")
                .and_then(Value::as_integer)
                .and_then(|n| u64::try_from(n).ok())
                .zip(
                    candidate
                        .get("completed_instructions")
                        .and_then(Value::as_integer)
                        .and_then(|n| u64::try_from(n).ok()),
                )
                .is_some_and(|(a, b)| a >= b)
        } else {
            ["expected_instructions", "scope"]
                .iter()
                .all(|key| candidate.get(key).is_some() && report.get(key) == candidate.get(key))
        }
}
fn route(status: &str, stage: &str, candidate: &Value, predecessor: &Value, disabled: &Value) -> Route {
    if ["error", "incomplete"].contains(&status) {
        Route::SetupBuildBudget
    } else if stage == "synchronization" {
        Route::IndependentHardwareInput
    } else if status == "uncovered" {
        Route::Coverage
    } else if !reproduced(candidate) {
        Route::FailureNotReproduced
    } else if passing(predecessor)
        && passing(disabled)
        && matched(predecessor, candidate)
        && matched(disabled, candidate)
    {
        Route::Increment
    } else {
        Route::SharedPathsInputCacheHarness
    }
}
fn description(route: &Route) -> (&'static str, &'static str) {
    match route {
        Route::SetupBuildBudget => (
            "Candidate validation did not complete",
            "Repair the first retained setup/build/budget failure and rerun fresh candidate validation",
        ),
        Route::Coverage => (
            "Required changed keys did not execute",
            "Use a covered interval within the validated domain, or remove/defer unexecuted candidate keys",
        ),
        Route::IndependentHardwareInput => (
            "Shared-backend passing controls do not establish the recording core's hardware/input behavior",
            "Capture original and candidate actual-fetch IRQ/DMA timing around the first checkpoint disagreement",
        ),
        Route::FailureNotReproduced => (
            "Candidate focus did not reproduce an actual instruction/state/bus mismatch; passing controls cannot attribute the cold failure",
            "Compare the candidate cold from the recorded failed input frame through its first rollover; extend independent hardware/video observation if that frame matches",
        ),
        Route::Increment => (
            "Passing predecessor and disabled-entry controls support attribution to the increment",
            "Probe changed instruction semantics, width dispatch and ordered bus timing at the first disagreement",
        ),
        Route::SharedPathsInputCacheHarness => (
            "Control failure, incomplete controls or unmatched input leaves increment attribution unresolved",
            "Verify input/profile, preserved predecessor, cache fidelity and shared hardware at the first disagreement",
        ),
    }
}
fn summarize(verdict: &Value) -> Result<Failure> {
    let checks = &verdict["checks"];
    let names = ["instruction-diagnosis", "generated-parity", "synchronization", "changed-entry-coverage"];
    let selected = names.iter().find(|name| {
        [Some("divergence"), Some("uncovered"), Some("error"), Some("incomplete")]
            .contains(&checks.get(**name).and_then(|report| report.get("status")).and_then(Value::as_str))
    });
    let report = selected.map(|name| &checks[*name]).unwrap_or(verdict);
    let stage = selected.copied().unwrap_or(verdict.get("stage").and_then(Value::as_str).unwrap_or("validation"));
    let status = family::field(report, "status")?;
    let controls = checks
        .get("failure-controls")
        .and_then(|report| report.get("controls"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let empty = Value::Table(toml::Table::new());
    let first = controls.first().unwrap_or(&empty);
    let candidate = first.get("candidate").unwrap_or(report);
    let routing = route(
        status,
        stage,
        candidate,
        first.get("predecessor").unwrap_or(&empty),
        first.get("disabled").unwrap_or(&empty),
    );
    let earliest = controls
        .iter()
        .find_map(|row| row.get("candidate").and_then(|candidate| candidate.get("first_mismatch")))
        .or_else(|| report.get("first_mismatch"))
        .or_else(|| report.get("first_divergence"))
        .or_else(|| {
            report
                .get("reports")
                .and_then(Value::as_array)
                .and_then(|rows| rows.iter().find_map(|row| row.get("first_mismatch")))
        });
    let (conclusion, next) = description(&routing);
    Ok(Failure {
        status: status.into(),
        stage: stage.into(),
        route: routing,
        earliest_disagreement: earliest.map(facts).unwrap_or_default(),
        conclusion: conclusion.into(),
        next_experiment: next.into(),
    })
}
fn owned_hash(owner: &Path, relative: &str, expected: &str, manifest: &Value) -> Result<PathBuf> {
    let path = safe_artifact(owner, relative)?;
    ensure!(digest(&path)? == expected, "observation artifact checksum differs: {relative}");
    let native = manifest["identities"].as_table().is_some_and(|rows| {
        rows.values().any(|row| {
            row.get("evidence").and_then(Value::as_str) == Some(relative)
                && row.get("sha256").and_then(Value::as_str) == Some(expected)
        })
    }) || manifest["check_evidence"].as_table().is_some_and(|rows| {
        rows.values().any(|row| {
            row["path"] == crate::readiness_metadata::value(&relative).expect("validated native metadata")
                && row["sha256"] == crate::readiness_metadata::value(&expected).expect("validated native metadata")
        })
    });
    ensure!(native, "observation artifact lacks immutable owner seal: {relative}");
    Ok(path)
}
fn execution(descriptor: &Descriptor) -> Result<(Value, Value, PathBuf)> {
    ensure!(!descriptor.run_path.is_symlink(), "observation owner is a symlink");
    let owner = fs::canonicalize(&descriptor.run_path)?;
    ensure!(owner == descriptor.run_path, "observation owner must be canonical");
    let (manifest, _, _) = terminal(&owner)?;
    ensure!(
        Path::new(&descriptor.evidence).extension().is_some_and(|extension| extension == "toml"),
        "diagnostic observations must use TOML"
    );
    let artifact = owned_hash(&owner, &descriptor.evidence, &descriptor.sha256, &manifest)?;
    let process_path = safe_artifact(&owner, &descriptor.process)?;
    ensure!(digest(&process_path)? == descriptor.process_sha256, "observation process checksum differs");
    let process = origins::document(&process_path)?;
    ensure!(
        process["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
            && process.get("signal").is_none()
            && process["returncode"].as_integer().is_some_and(|code| [0, 1].contains(&code))
            && family::array(&manifest, "commands")?.contains(&process),
        "observation execution did not complete with preserved provenance"
    );
    let name = Path::new(&descriptor.process)
        .file_stem()
        .and_then(|name| name.to_str())
        .context("observation process stage missing")?;
    ensure!(descriptor.process == format!("logs/{name}.toml"), "observation process path differs");
    origins::process(&owner, name, &manifest, None, &[0, 1])?;
    let binary = owned_hash(&owner, &descriptor.executable, &descriptor.executable_sha256, &manifest)?;
    ensure!(binary.metadata()?.permissions().mode() & 0o111 != 0, "observation executable is not executable");
    let command = family::array(&process, "command")?;
    ensure!(
        command.first().and_then(Value::as_str).is_some_and(|path| Path::new(path) == binary),
        "observation is not bound to executed preserved binary"
    );
    let output_bound = command.iter().any(|value| value.as_str().is_some_and(|value| Path::new(value) == artifact))
        || command.windows(2).any(|pair| {
            pair[0] == crate::readiness_metadata::value(&"--out").expect("validated native metadata")
                && pair[1].as_str().is_some_and(|value| Some(Path::new(value)) == artifact.parent())
        });
    ensure!(output_bound, "observation output is not bound to process invocation");
    let actual = origins::document(&artifact)?;
    if let Some(hash) = actual.get("binary_sha256") {
        ensure!(hash.as_str() == Some(&descriptor.executable_sha256), "observation report binary identity differs");
    }
    Ok((actual, manifest, owner))
}
pub fn verify_observation(descriptor: &Descriptor) -> Result<Observation> {
    let (actual, manifest, owner) = execution(descriptor)?;
    let value = match descriptor.kind {
        ObservationKind::FirstMismatch => {
            ensure!(reproduced(&actual), "observation lacks an actual complete first mismatch");
            let mismatch = &actual["first_mismatch"];
            ensure!(
                mismatch["instruction"].as_integer().and_then(|n| u64::try_from(n).ok()).is_some()
                    && mismatch["reference"].is_table()
                    && mismatch["subject"].is_table()
                    && mismatch["reference"] != mismatch["subject"],
                "observation mismatch lacks unequal execution effects"
            );
            crate :: report ! ({ "mismatch" : mismatch , })
        }
        ObservationKind::Controls => {
            let controls = family::array(&actual, "controls")?;
            ensure!(
                !controls.is_empty() && controls.iter().any(|row| reproduced(&row["candidate"])),
                "passing controls do not reproduce a failure"
            );
            let mut values = Vec::new();
            for control in controls {
                let mut row = crate :: report ! ({ "interval" : control ["interval"] , });
                for name in ["candidate", "predecessor", "disabled"] {
                    let report = &control[name];
                    ensure!(
                        report["complete"]
                            == crate::readiness_metadata::value(&true).expect("validated native metadata")
                            && [Some("pass"), Some("divergence")].contains(&report["status"].as_str())
                            && matched(report, &control["candidate"]),
                        "incomplete/unmatched controls are not discriminating evidence"
                    );
                    row.as_table_mut().context("control summary table missing")?.insert(
                        name.into(),
                        crate::report!({ "status": report["status"], "first_mismatch": report.get("first_mismatch") }),
                    );
                    let descriptor: Descriptor = toml_value_decode(report["observation"].clone())
                        .context("controls require process-bound observation descriptors")?;
                    ensure!(
                        descriptor.kind == ObservationKind::FirstMismatch
                            || report["status"]
                                == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
                        "control observation kind differs"
                    );
                    if report["status"]
                        == crate::readiness_metadata::value(&"divergence").expect("validated native metadata")
                    {
                        let observation = verify_observation(&descriptor)?;
                        ensure!(
                            observation.facts.values
                                == facts(&crate :: report ! ({ "mismatch" : report ["first_mismatch"] })),
                            "control mismatch differs from executed observation"
                        );
                    } else {
                        let (executed, _, _) = execution(&descriptor)?;
                        ensure!(
                            passing(&executed) && matched(&executed, report),
                            "passing control differs from process-bound evidence"
                        );
                    }
                }
                values.push(row);
            }
            crate :: report ! ({ "controls" : values , })
        }
        ObservationKind::Coverage => {
            ensure!(
                actual["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
                    && actual["uncovered_keys"].is_array(),
                "observation lacks complete changed-key coverage"
            );
            let mut executed = Vec::new();
            let mut keys = BTreeSet::new();
            for row in family::array(&actual, "executed")? {
                let address = row["address"]
                    .as_integer()
                    .and_then(|n| u64::try_from(n).ok())
                    .filter(|n| *n <= 0xffffff)
                    .context("invalid observed coverage address")?;
                let mx = row["mx"]
                    .as_integer()
                    .and_then(|n| u64::try_from(n).ok())
                    .filter(|n| *n < 4)
                    .context("invalid observed coverage width")?;
                let count = row["count"]
                    .as_integer()
                    .and_then(|n| u64::try_from(n).ok())
                    .context("invalid observed execution count")?;
                ensure!(keys.insert((address, mx)), "duplicate observed coverage key");
                executed.push(crate :: report ! ({ "address" : address , "mx" : mx , "count" : count , }));
            }
            crate :: report ! ({ "uncovered_keys" : actual ["uncovered_keys"] , "executed" : executed , })
        }
        ObservationKind::IndependentHardware => {
            ensure!(
                actual["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
                    && actual["independent_source"].as_str().is_some_and(|source| !source.trim().is_empty())
                    && actual["observations"].as_array().is_some_and(|values| !values.is_empty()),
                "independent hardware observation provenance/effects missing"
            );
            let capture = safe_artifact(&owner, family::field(&actual, "capture")?)?;
            let frames = actual["completed_frames"]
                .as_integer()
                .and_then(|n| u64::try_from(n).ok())
                .context("independent capture frame scope missing")?;
            crate::readiness_sync::verify_capture(&capture, "original", &manifest["identities"]["inputs"], frames)?;
            let recorded = crate::checkpoints::read(&capture)?;
            ensure!(
                actual["observations"]
                    == crate::readiness_metadata::value(&toml::Value::try_from(&recorded)?)
                        .expect("validated native metadata"),
                "independent observations differ from authenticated original capture"
            );
            crate :: report ! ({ "independent_source" : actual ["independent_source"] , "observations" : actual ["observations"] , })
        }
    };
    let facts = Facts { kind: descriptor.kind.clone(), values: facts(&value) };
    Ok(Observation { descriptor: descriptor.clone(), fingerprint: fingerprint(&facts)?, facts })
}
fn action(route: &Route, decision: bool) -> Action {
    if decision {
        return Action::DecisionRequired;
    }
    match route {
        Route::Coverage => Action::PruneUnexecutedKeys,
        Route::IndependentHardwareInput => Action::IndependentObservation,
        Route::FailureNotReproduced => Action::LocalizeFailure,
        _ => Action::RepairAndValidate,
    }
}
fn verify_unlocked(root: &Path, increment: &str) -> Result<Vec<Experiment>> {
    let directory = directory(root, increment)?;
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(&directory)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".toml") && name.trim_end_matches(".toml").bytes().all(|b| b.is_ascii_digit()) {
            paths.push(entry.path());
        } else {
            ensure!(
                !name.ends_with(".toml") || !name.trim_end_matches(".toml").bytes().all(|b| b.is_ascii_digit()),
                "legacy history must be imported explicitly; refusing to reset it"
            );
        }
    }
    paths.sort();
    if paths.is_empty() {
        if directory.join("head.toml").exists() {
            let head: Head = pipeline_model::read(&safe_artifact(&directory, "head.toml")?)?;
            ensure!(
                head.schema_version == VERSION && head.sequence == 0 && head.sha256.is_none(),
                "history tail was removed"
            );
        }
        return Ok(Vec::new());
    }
    let lineage: Lineage = pipeline_model::read(&safe_artifact(&directory, "lineage.toml")?)?;
    let head: Head = pipeline_model::read(&safe_artifact(&directory, "head.toml")?)?;
    ensure!(
        lineage.schema_version == VERSION && lineage.increment_id == increment && head.schema_version == VERSION,
        "history lineage/version differs"
    );
    safe_id(&lineage.session_id)?;
    let mut previous = None;
    let mut records = Vec::new();
    let mut hashes = vec![None];
    let mut seen = BTreeSet::new();
    let mut streak = 0u64;
    for (index, path) in paths.iter().enumerate() {
        let sequence = index as u64 + 1;
        ensure!(path == &directory.join(format!("{sequence:04}.toml")), "history ordering differs");
        ensure!(!path.is_symlink(), "history reference is a symlink");
        let reference: Reference = pipeline_model::read(path)?;
        ensure!(
            reference.schema_version == VERSION
                && reference.sequence == sequence
                && reference.previous_sha256 == previous,
            "history ordering/hash chain differs"
        );
        let owner = &reference.run_path;
        ensure!(fs::canonicalize(owner)? == *owner && !owner.is_symlink(), "experiment owner differs");
        ensure!(
            digest(&owner.join("manifest.toml"))? == reference.manifest_sha256
                && digest(&owner.join("verdict.toml"))? == reference.verdict_sha256
                && digest(&owner.join("experiment.toml"))? == reference.experiment_sha256,
            "history experiment checksum differs"
        );
        let (manifest, _, verdict) = terminal(owner)?;
        let record: Experiment = pipeline_model::read(&safe_artifact(owner, "experiment.toml")?)?;
        ensure!(
            manifest["type"]
                == crate::readiness_metadata::value(&"iteration-experiment").expect("validated native metadata")
                && verdict["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
                && record.schema_version == VERSION
                && record.sequence == sequence
                && record.previous_sha256 == previous
                && record.run_path == *owner
                && record.session_id == lineage.session_id
                && record.increment_id == lineage.increment_id
                && record.accepted_parent == lineage.accepted_parent
                && manifest["session_id"]
                    == crate::readiness_metadata::value(&record.session_id).expect("validated native metadata")
                && manifest["increment_id"]
                    == crate::readiness_metadata::value(&record.increment_id).expect("validated native metadata")
                && manifest.get("accepted_parent").and_then(Value::as_str) == record.accepted_parent.as_deref()
                && manifest["candidate_run_id"]
                    == crate::readiness_metadata::value(&record.candidate_run_id).expect("validated native metadata")
                && manifest["identities"]["source"]["sha256"]
                    == crate::readiness_metadata::value(&record.experiment_source_sha256)
                        .expect("validated native metadata"),
            "history experiment identity/lineage differs"
        );
        ensure!(
            digest(&record.candidate_path.join("manifest.toml"))? == record.candidate_manifest_sha256
                && digest(&record.candidate_path.join("verdict.toml"))? == record.candidate_verdict_sha256,
            "preserved candidate checksum differs"
        );
        let (candidate, _, candidate_verdict) = terminal(&record.candidate_path)?;
        ensure!(
            candidate["run_id"]
                == crate::readiness_metadata::value(&record.candidate_run_id).expect("validated native metadata")
                && candidate["identities"]["source"]["sha256"]
                    == crate::readiness_metadata::value(&record.candidate_source_sha256)
                        .expect("validated native metadata")
                && candidate.get("accepted_parent").and_then(Value::as_str) == record.accepted_parent.as_deref()
                && record.failure == summarize(&candidate_verdict)?,
            "preserved candidate identity/failure differs"
        );
        for observation in &record.observations {
            ensure!(verify_observation(&observation.descriptor)? == *observation, "substantive observation changed");
        }
        let fingerprints = record.observations.iter().map(|row| row.fingerprint.clone()).collect::<BTreeSet<_>>();
        let novel = fingerprints.iter().any(|hash| !seen.contains(hash));
        streak = if novel { 0 } else { streak.checked_add(1).context("evidence streak overflow")? };
        ensure!(
            record.new_discriminating_evidence == novel
                && record.consecutive_without_evidence == streak
                && record.decision_required == (streak >= 3)
                && record.next_action == action(&record.failure.route, streak >= 3),
            "history novelty/streak/decision differs from execution evidence"
        );
        let check: Check = pipeline_model::read(&safe_artifact(owner, "checks/experiment-record.toml")?)?;
        ensure!(
            check.status == "pass"
                && check.complete
                && check.sequence == sequence
                && check.sha256 == reference.experiment_sha256,
            "history experiment check differs"
        );
        seen.extend(fingerprints);
        previous = Some(digest(path)?);
        hashes.push(previous.clone());
        records.push(record);
    }
    ensure!(
        head.sequence <= records.len() as u64
            && records.len() as u64 - head.sequence <= 1
            && hashes[head.sequence as usize] == head.sha256,
        "history committed head differs or tail was removed"
    );
    Ok(records)
}
pub fn verify_history(root: &Path, increment: &str) -> Result<Vec<Experiment>> {
    let directory = directory(root, increment)?;
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let _guard = lock(&safe_artifact(&directory, "history.lock")?)?;
    verify_unlocked(root, increment)
}
fn candidate_observations(owner: &Path, manifest: &Value, verdict: &Value) -> Result<Vec<Descriptor>> {
    let checks = verdict.get("checks").context("terminal checks missing")?;
    let reports = checks
        .get("instruction-diagnosis")
        .and_then(|report| report.get("reports"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let controls = checks
        .get("failure-controls")
        .and_then(|report| report.get("controls"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let mut descriptors = Vec::new();
    for report in reports.iter().chain(controls.iter().filter_map(|row| row.get("candidate"))) {
        if !reproduced(report) {
            continue;
        }
        let Some(relative) = report.get("evidence").and_then(Value::as_str) else {
            continue;
        };
        let Some(prefix) = Path::new(relative)
            .parent()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix("instructions"))
        else {
            continue;
        };
        let process = format!("logs/{prefix}focused-replay.toml");
        let executable = format!("generated/{prefix}focused-replay");
        if !owner.join(&process).is_file() || !owner.join(&executable).is_file() {
            continue;
        }
        ensure!(
            manifest["state"] == crate::readiness_metadata::value(&"complete").expect("validated native metadata"),
            "candidate observation still running"
        );
        descriptors.push(Descriptor {
            run_path: owner.into(),
            evidence: relative.into(),
            sha256: digest(&safe_artifact(owner, relative)?)?,
            process_sha256: digest(&safe_artifact(owner, &process)?)?,
            executable_sha256: digest(&safe_artifact(owner, &executable)?)?,
            process,
            executable,
            kind: ObservationKind::FirstMismatch,
        });
    }
    Ok(descriptors)
}
fn required<'a>(value: &'a Option<String>, field: &str) -> Result<&'a str> {
    value.as_deref().filter(|text| !text.trim().is_empty()).with_context(|| format!("{field} required"))
}
pub fn operate(request: Request) -> Result<History> {
    let root = fs::canonicalize(&request.checkout)?;
    safe_id(&request.increment_id)?;
    if request.history {
        return Ok(History {
            increment_id: request.increment_id.clone(),
            experiments: verify_history(&root, &request.increment_id)?,
        });
    }
    let session = required(&request.session_id, "session ID")?;
    safe_id(session)?;
    let hypothesis = required(&request.hypothesis, "hypothesis")?;
    let discriminator = required(&request.discriminator, "predicted discriminator")?;
    let result = required(&request.result, "experiment result")?;
    let candidate = fs::canonicalize(request.candidate.as_ref().context("failed candidate required")?)?;
    let (manifest, inventory, verdict) = terminal(&candidate)?;
    ensure!(
        [Some("divergence"), Some("uncovered"), Some("error"), Some("incomplete")]
            .contains(&verdict["status"].as_str())
            && manifest["type"] == crate::readiness_metadata::value(&"iteration").expect("validated native metadata"),
        "diagnosis requires a preserved terminal failed candidate"
    );
    for (field, expected) in [("session_id", session), ("increment_id", request.increment_id.as_str())] {
        if let Some(actual) = manifest.get("increment").and_then(|increment| increment.get(field)) {
            ensure!(actual.as_str() == Some(expected), "candidate session/increment lineage differs");
        }
    }
    let parent = manifest.get("accepted_parent").and_then(Value::as_str).map(str::to_owned);
    let directory = directory(&root, &request.increment_id)?;
    fs::create_dir_all(&directory)?;
    let _guard = lock(&safe_artifact(&directory, "history.lock")?)?;
    let mut history = verify_unlocked(&root, &request.increment_id)?;
    let lineage = Lineage {
        schema_version: VERSION,
        session_id: session.into(),
        increment_id: request.increment_id.clone(),
        accepted_parent: parent.clone(),
    };
    if directory.join("lineage.toml").exists() {
        ensure!(
            pipeline_model::read::<Lineage>(&directory.join("lineage.toml"))? == lineage,
            "unresolved increment session/predecessor changed"
        );
    } else {
        write(&directory.join("lineage.toml"), &lineage, false)?;
    }
    let sequence = history.len() as u64 + 1;
    let previous =
        if sequence > 1 { Some(digest(&directory.join(format!("{:04}.toml", sequence - 1)))?) } else { None };
    write(
        &directory.join("head.toml"),
        &Head { schema_version: VERSION, sequence: sequence - 1, sha256: previous.clone() },
        true,
    )?;
    let mut observations = Vec::new();
    for descriptor in candidate_observations(&candidate, &manifest, &verdict)?.iter().chain(request.evidence.iter()) {
        let observation = verify_observation(descriptor)?;
        if !observations.contains(&observation) {
            observations.push(observation);
        }
    }
    let seen =
        history.iter().flat_map(|row| row.observations.iter().map(|row| &row.fingerprint)).collect::<BTreeSet<_>>();
    let novel = observations.iter().any(|row| !seen.contains(&row.fingerprint));
    let streak = if novel {
        0
    } else {
        history
            .last()
            .map(|row| row.consecutive_without_evidence)
            .unwrap_or(0)
            .checked_add(1)
            .context("evidence streak overflow")?
    };
    let failure = summarize(&verdict)?;
    let mut run = Run::create(&root, "iteration-experiment", contract::domain(), parent.as_deref())?;
    eprintln!("Experiment: {}", run.path.display());
    let outcome = (|| -> Result<Experiment> {
        run.capture_source()?;
        run.manifest.session_id = Some(session.into());
        run.manifest.increment_id = Some(request.increment_id.clone());
        run.manifest.candidate_run_id = Some(family::field(&manifest, "run_id")?.into());
        run.save()?;
        run.record_artifact(
            "failed-candidate-verdict",
            &candidate.join("verdict.toml"),
            Some("diagnostics/candidate-verdict.toml"),
        )?;
        let current = origins::document(&run.path.join("source/inventory.toml"))?;
        let map = |inventory: &Value| -> Result<BTreeMap<String, String>> {
            family::array(inventory, "files")?
                .iter()
                .map(|row| Ok((family::field(row, "path")?.into(), family::field(row, "sha256")?.into())))
                .collect()
        };
        let before = map(&inventory)?;
        let after = map(&current)?;
        let changed = before
            .keys()
            .chain(after.keys())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|path| before.get(*path) != after.get(*path))
            .map(|path| SourceChange {
                path: path.clone(),
                before: before.get(path).cloned(),
                after: after.get(path).cloned(),
            })
            .collect();
        let path = run.path.join("diagnostics/source-diff.toml");
        pipeline_model::write(
            &path,
            &SourceDiff {
                candidate_source_sha256: family::field(&manifest["identities"]["source"], "sha256")?.into(),
                current_source_sha256: family::field(&run.manifest.identities["source"], "sha256")?.into(),
                changed,
            },
        )?;
        run.record_artifact("source-diff", &path, None)?;
        for (index, observation) in observations.iter().enumerate() {
            let path = run.path.join(format!("diagnostics/observation-{:02}.toml", index + 1));
            pipeline_model::write(&path, observation)?;
            run.record_artifact(&format!("observation-{}", index + 1), &path, None)?;
        }
        let choices = if streak >= 3 {
            vec![
                failure.next_experiment.clone(),
                "Inspect preserved candidate source and paired input/cache/hardware assumptions".into(),
                "Defer this increment and retain accepted predecessor".into(),
            ]
        } else {
            Vec::new()
        };
        let record = Experiment {
            schema_version: VERSION,
            session_id: session.into(),
            increment_id: request.increment_id.clone(),
            sequence,
            previous_sha256: previous.clone(),
            candidate_run_id: family::field(&manifest, "run_id")?.into(),
            candidate_path: candidate.clone(),
            candidate_manifest_sha256: digest(&candidate.join("manifest.toml"))?,
            candidate_verdict_sha256: digest(&candidate.join("verdict.toml"))?,
            candidate_source_sha256: family::field(&manifest["identities"]["source"], "sha256")?.into(),
            experiment_source_sha256: family::field(&run.manifest.identities["source"], "sha256")?.into(),
            accepted_parent: parent,
            hypothesis: hypothesis.into(),
            predicted_discriminator: discriminator.into(),
            result: result.into(),
            observations,
            new_discriminating_evidence: novel,
            consecutive_without_evidence: streak,
            decision_required: streak >= 3,
            next_action: action(&failure.route, streak >= 3),
            failure,
            choices,
            run_path: run.path.clone(),
        };
        let path = run.path.join("experiment.toml");
        pipeline_model::write(&path, &record)?;
        run.record_artifact("experiment", &path, None)?;
        let brief = format!(
            "{}\n\nUnresolved increment: {}. Candidate: {}. Accepted predecessor: {}.\n\n{}\n\nHypothesis: {}. Predicted discriminator: {}. Result: {}.\n\nConsecutive experiments without new evidence: {}.\n\nNext experiment: {}.\n\n[Immutable record](experiment.toml) · [Candidate]({}/brief.md) · [Evidence](diagnostics/)\n",
            if record.decision_required { "Decision required" } else { "Routine diagnosis continues" },
            record.increment_id,
            record.candidate_run_id,
            record.accepted_parent.as_deref().unwrap_or("none"),
            record.failure.conclusion,
            record.hypothesis,
            record.predicted_discriminator,
            record.result,
            streak,
            record.failure.next_experiment,
            candidate.display()
        );
        if record.decision_required {
            let decision = run.path.join("diagnostics/decision.toml");
            pipeline_model::write(&decision, &record)?;
            run.record_artifact("decision", &decision, None)?;
            let markdown = run.path.join("diagnostics/decision.md");
            fs::write(
                &markdown,
                format!(
                    "{brief}\nChoices:\n{}",
                    record.choices.iter().map(|choice| format!("- {choice}\n")).collect::<String>()
                ),
            )?;
            run.record_artifact("decision-brief", &markdown, None)?;
        }
        run.record_typed_check(
            "experiment-record",
            &Check { status: "pass".into(), complete: true, sequence, sha256: digest(&path)? },
        )?;
        run.check_sources()?;
        run.finalize(
            "pass",
            Some("experiment-record"),
            Some("Diagnostic experiment retained; does not certify acceptance"),
            &["experiment-record".into()],
        )?;
        fs::write(run.path.join("brief.md"), &brief)?;
        let reference = Reference {
            schema_version: VERSION,
            sequence,
            run_path: run.path.clone(),
            manifest_sha256: digest(&run.path.join("manifest.toml"))?,
            verdict_sha256: digest(&run.path.join("verdict.toml"))?,
            experiment_sha256: digest(&path)?,
            previous_sha256: previous,
        };
        let reference_path = directory.join(format!("{sequence:04}.toml"));
        write(&reference_path, &reference, false)?;
        write(
            &directory.join("head.toml"),
            &Head { schema_version: VERSION, sequence, sha256: Some(digest(&reference_path)?) },
            true,
        )?;
        Ok(record)
    })();
    match outcome {
        Ok(record) => {
            history.push(record);
            Ok(History { increment_id: request.increment_id, experiments: history })
        }
        Err(error) => {
            if run.manifest.state == "running" {
                run.finalize("error", Some("experiment-record"), Some(&format!("{error:#}")), &[])?;
            }
            Err(error)
        }
    }
}
#[doc = " CLI metadata is TOML. `--evidence` imports a native table containing"]
#[doc = " `[[evidence]]` descriptors."]
pub fn run(args: &[String]) -> Result<()> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Evidence {
        evidence: Vec<Descriptor>,
    }
    let mut request = Request {
        checkout: std::env::current_dir()?,
        increment_id: String::new(),
        history: false,
        session_id: None,
        candidate: None,
        hypothesis: None,
        discriminator: None,
        result: None,
        evidence: Vec::new(),
    };
    let mut index = 0;
    while index < args.len() {
        let option = args[index].as_str();
        if option == "--history" {
            request.history = true;
            index += 1;
            continue;
        }
        let value = args.get(index + 1).with_context(|| format!("{option} needs a value"))?;
        match option {
            "--root" | "--checkout" => request.checkout = value.into(),
            "--increment-id" => request.increment_id = value.clone(),
            "--session-id" => request.session_id = Some(value.clone()),
            "--candidate" => request.candidate = Some(value.into()),
            "--hypothesis" => request.hypothesis = Some(value.clone()),
            "--discriminator" => request.discriminator = Some(value.clone()),
            "--result" => request.result = Some(value.clone()),
            "--evidence" => {
                let path = Path::new(value);
                ensure!(
                    path.extension().is_some_and(|extension| extension == "toml"),
                    "diagnostic evidence descriptors must use TOML"
                );
                request.evidence = pipeline_model::read::<Evidence>(path)?.evidence;
            }
            _ => anyhow::bail!("unknown iteration-diagnosis option: {option}"),
        }
        index += 2;
    }
    let inspect = request.history;
    let history = operate(request)?;
    println!("{}", toml::to_string_pretty(&history)?);
    ensure!(
        inspect || !history.experiments.last().is_some_and(|record| record.decision_required),
        "decision required after three experiments without new evidence"
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        process::Command,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn project() -> Temp {
        let path = std::env::temp_dir()
            .join(format!("sm-native-diagnosis-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&path).unwrap();
        assert!(Command::new("git").args(["init", "-q"]).arg(&path).status().unwrap().success());
        fs::create_dir(path.join("crates")).unwrap();
        fs::write(path.join("crates/control.rs"), "// candidate 0").unwrap();
        Temp(path)
    }
    fn candidate(root: &Path, number: u64) -> Run {
        fs::write(root.join("crates/control.rs"), format!("// candidate {number}")).unwrap();
        let mut run = Run::create(root, "iteration", contract::domain(), Some("accepted-parent")).unwrap();
        run.capture_source().unwrap();
        run . record_check ("generated-parity" , crate :: report ! ({ "status" : "divergence" , "complete" : true , "first_divergence" : crate :: report ! ({ "frame" : 7 , }) , }) ,) . unwrap () ;
        run.finalize("divergence", Some("generated-parity"), None, &[]).unwrap();
        run
    }
    fn request(root: &Path, candidate: &Path) -> Request {
        Request {
            checkout: root.into(),
            increment_id: "increment".into(),
            history: false,
            session_id: Some("session".into()),
            candidate: Some(candidate.into()),
            hypothesis: Some("Store timing changes effect".into()),
            discriminator: Some("Independent ordered bus trace differs".into()),
            result: Some("Same mismatch".into()),
            evidence: Vec::new(),
        }
    }
    fn observation(root: &Path, value: u64, elapsed: u64) -> Descriptor {
        let mut run =
            Run::create(root, "diagnostic", crate :: report ! ({ "instructions" : 1 , }), Some("accepted-parent"))
                .unwrap();
        run.capture_source().unwrap();
        let source = run.path.join("generated/probe.rs");
        let text = format!(
            "status = \"divergence\"\ncomplete = true\nelapsed_seconds = {elapsed}\n[first_mismatch]\ninstruction = 3\nregion = \"ordered bus\"\n[first_mismatch.reference]\nvalue = 1\n[first_mismatch.subject]\nvalue = {value}\n"
        );
        fs::write(
            &source,
            format!("fn main() {{ std::fs::write(std::env::args().nth(1).unwrap(), {text:?}).unwrap(); }}"),
        )
        .unwrap();
        let binary = run.path.join("generated/probe");
        assert!(
            Command::new("rustc")
                .arg(&source)
                .args(["--crate-name", "diagnosis_probe", "-o"])
                .arg(&binary)
                .status()
                .unwrap()
                .success()
        );
        run.record_artifact("probe", &binary, None).unwrap();
        run.record_artifact("probe-source", &source, None).unwrap();
        let evidence = run.path.join("diagnostics/observation.toml");
        run.stage(
            "probe",
            &[binary.to_str().unwrap().into(), evidence.to_str().unwrap().into()],
            root,
            &Default::default(),
            Duration::from_secs(30),
            &Default::default(),
        )
        .unwrap();
        run.record_artifact("probe-output", &evidence, None).unwrap();
        run.finalize("pass", Some("probe"), None, &["probe".into()]).unwrap();
        Descriptor {
            run_path: run.path.clone(),
            evidence: "diagnostics/observation.toml".into(),
            sha256: digest(&evidence).unwrap(),
            process: "logs/probe.toml".into(),
            process_sha256: digest(&run.path.join("logs/probe.toml")).unwrap(),
            executable: "generated/probe".into(),
            executable_sha256: digest(&binary).unwrap(),
            kind: ObservationKind::FirstMismatch,
        }
    }
    #[test]
    fn retries_preserve_candidate_and_escalate_on_third_without_evidence() {
        let root = project();
        for number in 0..3 {
            let candidate = candidate(&root.0, number);
            let before = fs::read(candidate.path.join("verdict.toml")).unwrap();
            let history = operate(request(&root.0, &candidate.path)).unwrap();
            let record = history.experiments.last().unwrap();
            assert_eq!(record.consecutive_without_evidence, number + 1);
            assert_eq!(record.decision_required, number == 2);
            assert_eq!(
                record.next_action,
                if number == 2 { Action::DecisionRequired } else { Action::LocalizeFailure }
            );
            assert_eq!(fs::read(candidate.path.join("verdict.toml")).unwrap(), before);
        }
        let history = verify_history(&root.0, "increment").unwrap();
        assert_eq!(history.len(), 3);
        assert!(history.last().unwrap().run_path.join("diagnostics/decision.toml").is_file());
        assert!(!root.0.join("out/accepted.toml").exists());
        let path = directory(&root.0, "increment").unwrap();
        let head =
            Head { schema_version: VERSION, sequence: 2, sha256: Some(digest(&path.join("0002.toml")).unwrap()) };
        pipeline_model::write(&path.join("head.toml"), &head).unwrap();
        assert_eq!(verify_history(&root.0, "increment").unwrap().len(), 3);
        pipeline_model::write(
            &path.join("head.toml"),
            &Head { schema_version: VERSION, sequence: 3, sha256: Some(digest(&path.join("0003.toml")).unwrap()) },
        )
        .unwrap();
        fs::remove_file(path.join("0003.toml")).unwrap();
        assert!(verify_history(&root.0, "increment").is_err());
    }
    #[test]
    fn actual_effect_novelty_survives_paths_timing_and_restart() {
        let root = project();
        let failed = candidate(&root.0, 0);
        for _ in 0..2 {
            operate(request(&root.0, &failed.path)).unwrap();
        }
        for (value, elapsed, novel, streak) in [(2, 1, true, 0), (2, 99, false, 1), (3, 100, true, 0)] {
            let mut request = request(&root.0, &failed.path);
            request.evidence.push(observation(&root.0, value, elapsed));
            let result = operate(request).unwrap();
            let record = result.experiments.last().unwrap();
            assert_eq!(record.new_discriminating_evidence, novel);
            assert_eq!(record.consecutive_without_evidence, streak);
            assert_eq!(verify_history(&root.0, "increment").unwrap().last().unwrap(), record);
        }
    }
    #[test]
    fn mutated_process_logs_artifacts_and_unexecuted_outputs_fail_closed() {
        let root = project();
        let descriptor = observation(&root.0, 2, 1);
        verify_observation(&descriptor).unwrap();
        let output = descriptor.run_path.join(&descriptor.evidence);
        let bytes = fs::read(&output).unwrap();
        fs::write(&output, "status = 'pass'").unwrap();
        assert!(verify_observation(&descriptor).is_err());
        fs::write(&output, bytes).unwrap();
        fs::write(descriptor.run_path.join("logs/probe.stdout.log"), "forged").unwrap();
        assert!(verify_observation(&descriptor).is_err());
        let descriptor = observation(&root.0, 2, 1);
        let other = descriptor.run_path.join("diagnostics/unexecuted.toml");
        fs::copy(descriptor.run_path.join(&descriptor.evidence), &other).unwrap();
        let mut manifest = origins::document(&descriptor.run_path.join("manifest.toml")).unwrap();
        manifest["identities"].as_table_mut().unwrap().insert("unexecuted".into(), crate :: report ! ({ "evidence" : "diagnostics/unexecuted.toml" , "sha256" : digest (& other) . unwrap () , }));
        crate::atomic_toml(&descriptor.run_path.join("manifest.toml"), &manifest).unwrap();
        let altered = Descriptor {
            evidence: "diagnostics/unexecuted.toml".into(),
            sha256: digest(&other).unwrap(),
            ..descriptor
        };
        assert!(verify_observation(&altered).unwrap_err().to_string().contains("output is not bound"));
    }
    #[test]
    fn lineage_safe_ids_and_legacy_history_cannot_reset_streak() {
        let root = project();
        assert!(verify_history(&root.0, "../escape").is_err());
        let candidate = candidate(&root.0, 0);
        let first = operate(request(&root.0, &candidate.path)).unwrap();
        let mut altered = request(&root.0, &candidate.path);
        altered.session_id = Some("other-session".into());
        assert!(operate(altered).is_err());
        let mut manifest = origins::document(&candidate.path.join("manifest.toml")).unwrap();
        manifest.as_table_mut().unwrap().insert("accepted_parent".into(), "other-parent".into());
        crate::atomic_toml(&candidate.path.join("manifest.toml"), &manifest).unwrap();
        assert!(verify_history(&root.0, "increment").is_err());
        assert_eq!(first.experiments.len(), 1);
        let legacy = directory(&root.0, "legacy").unwrap();
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("0001.toml"), "{}").unwrap();
        assert!(verify_history(&root.0, "legacy").is_err());
    }
    #[test]
    fn routing_requires_matched_controls_and_preserves_cold_failure() {
        let candidate = crate :: report ! ({ "status" : "divergence" , "complete" : true , "start_input_index" : 7 , "start_instruction" : 0 , "expected_instructions" : 4 , "scope" : "instruction-count" , "identities" : crate :: report ! ({ "input_sha256" : "input" , }) , "first_mismatch" : crate :: report ! ({ "instruction" : 3 , "reference" : crate :: report ! ({ "value" : 1 , }) , "subject" : crate :: report ! ({ "value" : 2 , }) , }) , });
        let mut passing = candidate.clone();
        passing["status"] = "pass".into();
        assert_eq!(route("divergence", "instruction-diagnosis", &candidate, &passing, &passing), Route::Increment);
        passing["start_input_index"] = 8.into();
        assert_eq!(
            route("divergence", "instruction-diagnosis", &candidate, &passing, &passing),
            Route::SharedPathsInputCacheHarness
        );
        assert_eq!(route("divergence", "generated-parity", &passing, &passing, &passing), Route::FailureNotReproduced);
        assert_eq!(
            route("divergence", "synchronization", &candidate, &passing, &passing),
            Route::IndependentHardwareInput
        );
        assert_eq!(route("uncovered", "changed-entry-coverage", &candidate, &passing, &passing), Route::Coverage);
        assert_eq!(route("error", "generated-parity", &candidate, &passing, &passing), Route::SetupBuildBudget);
    }
}
