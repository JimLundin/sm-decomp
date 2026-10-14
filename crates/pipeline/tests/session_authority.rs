//! Real child processes exercise session generations without mutating production state.
#[allow(dead_code)]
#[path = "../src/assignment.rs"]
mod assignment;
#[allow(dead_code)]
#[path = "../src/bounded_inputs.rs"]
pub(crate) mod bounded_inputs;
#[allow(dead_code)]
#[path = "../src/family_model.rs"]
mod family_model;
mod inputs {
    pub(crate) use crate::bounded_inputs as bounded;
}
#[allow(dead_code)]
#[path = "../src/translation_migration_guard.rs"]
mod translation_migration_guard;
#[allow(dead_code)]
#[path = "../src/translation_production_registry.rs"]
mod translation_production_registry;
use assignment::{
    authority::{self, RoleKind, RoleRegistration, SessionAuthority},
    *,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

fn digest(path: &Path) -> anyhow::Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
const INTEGRATOR: &str = "01a13490-0000-7000-8000-000000000001";
const DISPATCHER: &str = "01a13490-0000-7000-8000-000000000002";
const REPLACEMENT: &str = "01a13490-0000-7000-8000-000000000003";
const CHILD: &str = "01a13490-0000-7000-8000-000000000004";
static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Deserialize)]
#[serde(tag = "route", rename_all = "kebab-case")]
enum Action {
    Authority { request: authority::Request },
    Queue { request: Request },
}
#[derive(Serialize, Deserialize)]
struct Response {
    value: Option<toml::Value>,
    error: Option<String>,
}

#[test]
#[ignore]
fn session_process() {
    let Some(path) = std::env::var_os("SM_AUTHORITY_TEST_ACTION") else {
        return;
    };
    let action: Action = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    let result = match action {
        Action::Authority { request } => {
            authority::operate(request).and_then(|receipt| Ok(toml::Value::try_from(receipt)?))
        }
        Action::Queue { request } => operate(request).and_then(|receipt| Ok(toml::Value::try_from(receipt)?)),
    };
    let response = match result {
        Ok(value) => Response { value: Some(value), error: None },
        Err(error) => Response { value: None, error: Some(format!("{error:#}")) },
    };
    fs::write(std::env::var_os("SM_AUTHORITY_TEST_RESPONSE").unwrap(), toml::to_string(&response).unwrap()).unwrap();
}

struct Fixture {
    root: PathBuf,
    serial: AtomicU64,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "sm-session-authority-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        assert!(Command::new("git").args(["init", "-q"]).arg(&root).status().unwrap().success());
        assert!(
            Command::new("git")
                .args([
                    "-C",
                    root.to_str().unwrap(),
                    "-c",
                    "user.name=Test",
                    "-c",
                    "user.email=test@example.invalid",
                    "commit",
                    "--allow-empty",
                    "-qm",
                    "initial"
                ])
                .status()
                .unwrap()
                .success()
        );
        Self { root, serial: AtomicU64::new(0) }
    }
    fn artifact(&self, name: &str, value: &str) -> Artifact {
        let path = self.root.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, value).unwrap();
        Artifact { sha256: digest(&path).unwrap(), path }
    }
    fn seal<T: Serialize>(&self, name: &str, value: &T) -> Artifact {
        self.artifact(name, &toml::to_string(value).unwrap())
    }
    fn inspect(&self, session: &str) -> Artifact {
        let evidence = self.artifact(
            &format!("observations/{session}-raw.txt"),
            &format!("real process test session {session} observed running"),
        );
        self.seal(
            &format!("observations/{session}.toml"),
            &authority::SessionObservation {
                schema: "decomp-session-observation-v1".into(),
                session_id: session.into(),
                status: "running".into(),
                observed_at: "2026-10-13T09:00:00Z".into(),
                evidence: vec![evidence],
            },
        )
    }
    fn run(&self, session: &str, action: Action) -> Result<toml::Value, String> {
        let n = self.serial.fetch_add(1, Ordering::Relaxed);
        let action_path = self.root.join(format!("requests/action-{n}.toml"));
        let response_path = self.root.join(format!("requests/response-{n}.toml"));
        fs::create_dir_all(action_path.parent().unwrap()).unwrap();
        fs::write(&action_path, toml::to_string(&action).unwrap()).unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "session_process", "--ignored"])
            .env("CODEX_THREAD_ID", session)
            .env("SM_AUTHORITY_TEST_ACTION", action_path)
            .env("SM_AUTHORITY_TEST_RESPONSE", &response_path)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let response: Response = toml::from_str(&fs::read_to_string(response_path).unwrap()).unwrap();
        response.error.map_or_else(|| Ok(response.value.unwrap()), Err)
    }
    fn credential(session: &str, generation: u64) -> Option<SessionAuthority> {
        Some(SessionAuthority { session_id: session.into(), generation })
    }
    fn authority(&self, operation: &str, session: &str, token: &str, generation: u64) -> authority::Request {
        authority::Request {
            checkout: self.root.clone(),
            operation: operation.into(),
            coordinator: token.into(),
            authority: Self::credential(session, generation),
            ..Default::default()
        }
    }
    fn queue(&self, operation: &str, session: &str, token: &str, generation: u64, effort: &str) -> Request {
        Request {
            checkout: self.root.clone(),
            operation: operation.into(),
            effort: effort.into(),
            coordinator: token.into(),
            authority: Self::credential(session, generation),
            capacity: 3,
            available_slots: 1,
            ..Default::default()
        }
    }
    fn prepare(&self, bridge: Artifact) -> authority::Preparation {
        let preparation = authority::Preparation {
            canonical_effort: "decomp-parallel".into(),
            capacity: 3,
            admitted_efforts: Vec::new(),
            drill_efforts: vec!["drill-a".into(), "drill-b".into()],
            bridge,
            integrator: RoleRegistration {
                token: "integrator".into(),
                kind: RoleKind::Integrator,
                session_id: INTEGRATOR.into(),
                observation: self.inspect(INTEGRATOR),
            },
        };
        let mut request = self.authority("prepare", INTEGRATOR, "integrator", 1);
        request.preparation = Some(preparation.clone());
        self.run(INTEGRATOR, Action::Authority { request }).unwrap();
        preparation
    }
    fn empty_bridge(&self) -> Artifact {
        self.artifact("bridge.toml", "schema = 'decomp-legacy-owner-bridge-v1'\nholdings = []\n")
    }
    fn register(&self) {
        let mut request = self.authority("register", INTEGRATOR, "integrator", 1);
        request.registration = Some(RoleRegistration {
            token: "dispatcher".into(),
            kind: RoleKind::Dispatcher,
            session_id: DISPATCHER.into(),
            observation: self.inspect(DISPATCHER),
        });
        self.run(INTEGRATOR, Action::Authority { request }).unwrap();
    }
    fn next(&self, session: &str, token: &str, effort: &str, candidate: Job) -> Receipt {
        let mut request = self.queue("next", session, token, 1, effort);
        request.jobs = vec![candidate];
        self.run(session, Action::Queue { request }).unwrap().try_into().unwrap()
    }
    fn record(
        &self,
        session: &str,
        token: &str,
        generation: u64,
        effort: &str,
        update: Update,
    ) -> Result<Receipt, String> {
        let mut request = self.queue("record", session, token, generation, effort);
        request.update = Some(update);
        self.run(session, Action::Queue { request })
            .and_then(|value| value.try_into().map_err(|error| format!("{error}")))
    }
    fn child(&self, effort: &str, claim: &Claim, agent: Option<&str>, status: &str) -> authority::ChildObservation {
        let mut value = toml::map::Map::new();
        value.insert("schema".into(), "decomp-child-observation-v1".into());
        value.insert("effort".into(), effort.into());
        value.insert("claim".into(), toml::Value::try_from(claim).unwrap());
        if let Some(agent) = agent {
            value.insert("agent".into(), agent.into());
        }
        value.insert("session_id".into(), agent.unwrap_or(INTEGRATOR).into());
        value.insert("status".into(), status.into());
        value.insert("observed_at".into(), "2026-10-13T09:00:00Z".into());
        let evidence = self.artifact(
            &format!("observations/child-{}-{}-raw.txt", claim.id, status),
            "checked actual child test-process state",
        );
        value.insert("evidence".into(), toml::Value::try_from(vec![evidence]).unwrap());
        let observation = self.seal(&format!("observations/child-{}-{}.toml", claim.id, status), &value);
        authority::ChildObservation {
            effort: effort.into(),
            claim: claim.clone(),
            agent: agent.map(str::to_owned),
            status: status.into(),
            observation,
        }
    }
    fn transfer(
        &self,
        token: &str,
        old: &str,
        children: Vec<authority::ChildObservation>,
        status: &str,
    ) -> authority::Request {
        let inspection =
            self.artifact("observations/transfer-raw.txt", "process test inspected old role and held children");
        let replacement = self.inspect(REPLACEMENT);
        let proof = self.seal(
            &format!("observations/transfer-{status}.toml"),
            &authority::TransferObservation {
                schema: "decomp-session-transfer-v1".into(),
                token: token.into(),
                session_id: old.into(),
                generation: 1,
                status: status.into(),
                observed_at: "2026-10-13T09:00:00Z".into(),
                evidence: vec![inspection],
                replacement,
                children,
                processes: Vec::new(),
            },
        );
        let mut request = self.authority("transfer", REPLACEMENT, token, 2);
        request.transfer =
            Some(authority::RoleTransfer { token: token.into(), session_id: REPLACEMENT.into(), observation: proof });
        request
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn job(id: &str, path: &str) -> Job {
    Job {
        id: id.into(),
        revision: "revision-1".into(),
        kind: Kind::Implement,
        subject: id.into(),
        writable_paths: vec![path.into()],
        blockers: Vec::new(),
        inputs: BTreeMap::new(),
        instructions: "Maintain this complete test subsystem.".into(),
        ticket: None,
    }
}

#[test]
fn prepared_policy_requires_real_sessions_and_blocks_production_and_arbitrary_efforts() {
    let f = Fixture::new();
    let preparation = f.prepare(f.empty_bridge());
    f.register();
    let mut retry = f.authority("prepare", INTEGRATOR, "integrator", 1);
    retry.preparation = Some(preparation);
    f.run(INTEGRATOR, Action::Authority { request: retry }).unwrap();
    for effort in ["decomp-parallel", "unmanaged-bypass"] {
        let request = f.queue("next", DISPATCHER, "dispatcher", 1, effort);
        assert!(f.run(DISPATCHER, Action::Queue { request }).unwrap_err().contains("not admitted"));
        assert!(!f.root.join(format!(".scratch/{effort}/ledger.toml")).exists());
    }
    let mut copy = f.queue("next", INTEGRATOR, "integrator", 1, "drill-a");
    copy.jobs = vec![job("copy", "src/copy.rs")];
    assert!(f.run(DISPATCHER, Action::Queue { request: copy }).unwrap_err().contains("calling real session"));
    let mut no_proof = f.queue("next", DISPATCHER, "dispatcher", 1, "drill-a");
    no_proof.authority = None;
    assert!(f.run(DISPATCHER, Action::Queue { request: no_proof }).unwrap_err().contains("requires session authority"));
}

#[test]
fn managed_capacity_is_shared_between_admitted_efforts() {
    let f = Fixture::new();
    f.prepare(f.empty_bridge());
    f.register();
    assert!(f.next(INTEGRATOR, "integrator", "drill-a", job("a", "src/a.rs")).assignment.is_some());
    assert!(f.next(DISPATCHER, "dispatcher", "drill-b", job("b", "src/b.rs")).assignment.is_some());
    assert!(f.next(INTEGRATOR, "integrator", "drill-a", job("c", "src/c.rs")).assignment.is_some());
    let waiting = f.next(DISPATCHER, "dispatcher", "drill-b", job("d", "src/d.rs"));
    assert!(waiting.assignment.is_none());
    let request = f.queue("status", DISPATCHER, "dispatcher", 1, "drill-b");
    let status: Receipt = f.run(DISPATCHER, Action::Queue { request }).unwrap().try_into().unwrap();
    assert_eq!(status.jobs.iter().filter(|entry| matches!(entry.state, State::Reserved | State::Running)).count(), 1);
}

#[test]
fn explicit_integrator_keeps_first_submitter_out_and_fences_old_integrator_retries() {
    let f = Fixture::new();
    f.prepare(f.empty_bridge());
    f.register();
    let claim = f
        .next(DISPATCHER, "dispatcher", "drill-a", job("implementation", "src/implementation.rs"))
        .assignment
        .unwrap()
        .claim;
    f.record(DISPATCHER, "dispatcher", 1, "drill-a", Update::Started { claim: claim.clone(), agent: CHILD.into() })
        .unwrap();
    let evidence = f.artifact("submission.toml", "sealed = true\n");
    f.record(
        DISPATCHER,
        "dispatcher",
        1,
        "drill-a",
        Update::Finished {
            claim: claim.clone(),
            result: WorkResult::Submitted { commit: "a".repeat(40), evidence: vec![evidence] },
        },
    )
    .unwrap();
    let request = f.queue("next", DISPATCHER, "dispatcher", 1, "drill-a");
    let waiting: Receipt = f.run(DISPATCHER, Action::Queue { request }).unwrap().try_into().unwrap();
    assert!(waiting.assignment.is_none());
    let request = f.queue("next", INTEGRATOR, "integrator", 1, "drill-a");
    let integration: Receipt = f.run(INTEGRATOR, Action::Queue { request }).unwrap().try_into().unwrap();
    let integration = integration.assignment.unwrap();
    assert_eq!(integration.job.kind, Kind::Integrate);
    let started = Update::Started { claim: integration.claim.clone(), agent: INTEGRATOR.into() };
    f.record(INTEGRATOR, "integrator", 1, "drill-a", started.clone()).unwrap();
    let child = f.child("drill-a", &integration.claim, Some(INTEGRATOR), "interrupted");
    let transfer = f.transfer("integrator", INTEGRATOR, vec![child], "interrupted");
    f.run(REPLACEMENT, Action::Authority { request: transfer.clone() }).unwrap();
    f.run(REPLACEMENT, Action::Authority { request: transfer }).unwrap();
    assert!(
        f.record(INTEGRATOR, "integrator", 1, "drill-a", started.clone())
            .unwrap_err()
            .contains("stale session authority")
    );
    f.record(REPLACEMENT, "integrator", 2, "drill-a", started).unwrap();
    let request = f.queue("status", REPLACEMENT, "integrator", 2, "drill-a");
    let status: Receipt = f.run(REPLACEMENT, Action::Queue { request }).unwrap().try_into().unwrap();
    assert_eq!(status.jobs.iter().find(|entry| entry.job.id == "implementation").unwrap().state, State::Submitted);
}

#[test]
fn stopped_parent_transfer_preserves_live_child_claim_and_rejects_omissions() {
    let f = Fixture::new();
    f.prepare(f.empty_bridge());
    f.register();
    let claim =
        f.next(DISPATCHER, "dispatcher", "drill-b", job("live-child", "src/child.rs")).assignment.unwrap().claim;
    let started = Update::Started { claim: claim.clone(), agent: CHILD.into() };
    f.record(DISPATCHER, "dispatcher", 1, "drill-b", started.clone()).unwrap();
    f.record(DISPATCHER, "dispatcher", 1, "drill-b", started.clone()).unwrap();
    let missing = f.transfer("dispatcher", DISPATCHER, Vec::new(), "interrupted");
    assert!(f.run(REPLACEMENT, Action::Authority { request: missing }).unwrap_err().contains("omits held child"));
    let running =
        f.transfer("dispatcher", DISPATCHER, vec![f.child("drill-b", &claim, Some(CHILD), "running")], "running");
    assert!(f.run(REPLACEMENT, Action::Authority { request: running }).unwrap_err().contains("stopped real owner"));
    let transfer =
        f.transfer("dispatcher", DISPATCHER, vec![f.child("drill-b", &claim, Some(CHILD), "running")], "interrupted");
    f.run(REPLACEMENT, Action::Authority { request: transfer }).unwrap();
    assert!(
        f.record(DISPATCHER, "dispatcher", 1, "drill-b", started.clone())
            .unwrap_err()
            .contains("stale session authority")
    );
    f.record(REPLACEMENT, "dispatcher", 2, "drill-b", started).unwrap();
    let request = f.queue("status", REPLACEMENT, "dispatcher", 2, "drill-b");
    let status: Receipt = f.run(REPLACEMENT, Action::Queue { request }).unwrap().try_into().unwrap();
    let entry = status.jobs.iter().find(|entry| entry.job.id == "live-child").unwrap();
    assert_eq!(entry.claim.as_ref(), Some(&claim));
    assert_eq!(entry.agent.as_deref(), Some(CHILD));
    assert_eq!(entry.state, State::Running);
    let mut duplicate = f.queue("next", REPLACEMENT, "dispatcher", 2, "drill-a");
    duplicate.jobs = vec![job("duplicate", "src/child.rs")];
    let waiting: Receipt = f.run(REPLACEMENT, Action::Queue { request: duplicate }).unwrap().try_into().unwrap();
    assert!(waiting.assignment.is_none());
    assert!(waiting.deferred["duplicate"].contains("writable ownership"));
}

#[test]
fn bridge_hash_source_verification_and_markdown_fences_are_enforced() {
    let bad = Fixture::new();
    let source = bad.artifact("frozen.rs", "retained source");
    let bridge = bad.artifact("bridge.toml", &format!("schema='decomp-legacy-owner-bridge-v1'\n[[holdings]]\nid='legacy'\neffort='legacy-effort'\nkind='markdown-hold'\nstate='claimed'\nretained=true\nticket='.scratch/legacy-effort/issues/01.md'\nwritable_paths=['src/held.rs']\n[[holdings.source]]\npath='{}'\nsha256='{}'\n", source.path.display(), source.sha256));
    fs::write(source.path, "changed source").unwrap();
    let preparation = authority::Preparation {
        canonical_effort: "decomp-parallel".into(),
        capacity: 3,
        admitted_efforts: vec![],
        drill_efforts: vec!["drill-a".into()],
        bridge,
        integrator: RoleRegistration {
            token: "integrator".into(),
            kind: RoleKind::Integrator,
            session_id: INTEGRATOR.into(),
            observation: bad.inspect(INTEGRATOR),
        },
    };
    let mut request = bad.authority("prepare", INTEGRATOR, "integrator", 1);
    request.preparation = Some(preparation);
    assert!(bad.run(INTEGRATOR, Action::Authority { request }).unwrap_err().contains("artifact identity differs"));

    let f = Fixture::new();
    let source = f.artifact("frozen.rs", "retained source");
    let bridge = f.artifact("bridge.toml", &format!("schema='decomp-legacy-owner-bridge-v1'\n[[holdings]]\nid='legacy'\neffort='legacy-effort'\nkind='markdown-hold'\nstate='claimed'\nretained=true\nticket='.scratch/legacy-effort/issues/01.md'\nwritable_paths=['src/held.rs']\n[[holdings.source]]\npath='{}'\nsha256='{}'\n", source.path.display(), source.sha256));
    f.prepare(bridge.clone());
    let waiting = f.next(INTEGRATOR, "integrator", "drill-a", job("overlap", "src/held.rs"));
    assert!(waiting.assignment.is_none());
    assert!(waiting.deferred["overlap"].contains("retained legacy legacy"));
    let ticket = f.root.join(".scratch/legacy-effort/issues/01.md");
    fs::create_dir_all(ticket.parent().unwrap()).unwrap();
    fs::write(ticket, "# Stale\nStatus: open\nBlocked by: None\nTriage: ready-for-agent\n").unwrap();
    let mut duplicate = job("duplicate-ticket", "src/free.rs");
    duplicate.ticket = Some(".scratch/legacy-effort/issues/01.md".into());
    let waiting = f.next(INTEGRATOR, "integrator", "drill-a", duplicate);
    assert!(waiting.assignment.is_none());
    assert!(waiting.deferred["duplicate-ticket"].contains("ticket ownership"));
    fs::write(bridge.path, "schema='decomp-legacy-owner-bridge-v1'\nholdings=[]\n").unwrap();
    let request = f.queue("next", INTEGRATOR, "integrator", 1, "drill-a");
    assert!(f.run(INTEGRATOR, Action::Queue { request }).unwrap_err().contains("artifact identity differs"));
}

#[test]
fn activation_checks_drill_source_controls_and_keeps_arbitrary_efforts_fenced() {
    let f = Fixture::new();
    f.prepare(f.empty_bridge());
    f.register();
    let receipt: authority::Receipt = f
        .run(INTEGRATOR, Action::Authority { request: f.authority("status", INTEGRATOR, "integrator", 1) })
        .unwrap()
        .try_into()
        .unwrap();
    let head = String::from_utf8(
        Command::new("git").args(["-C", f.root.to_str().unwrap(), "rev-parse", "HEAD"]).output().unwrap().stdout,
    )
    .unwrap()
    .trim()
    .to_owned();
    let controls = [
        "independent-sessions",
        "writable-overlap",
        "duplicate-ticket",
        "crash-restart",
        "surviving-child",
        "stopped-transfer",
        "stale-authority",
        "same-attempt-retry",
        "shared-capacity",
        "cleanup",
        "integration-success",
    ]
    .map(|name| (name.into(), true))
    .into_iter()
    .collect();
    let evidence = f.artifact("drill-result.txt", "all isolated controls passed");
    let mut drill = authority::DrillObservation {
        schema: "decomp-concurrency-drill-v1".into(),
        checkout: f.root.clone(),
        source_checkout: f.root.clone(),
        isolated_checkout: f.root.with_file_name("isolated-authority-drill"),
        bridge: receipt.bridge.unwrap(),
        head,
        source_fingerprint: source_fingerprint(&f.root).unwrap(),
        controls,
        evidence: vec![evidence],
        execution: None,
    };
    let mut request = f.authority("activate", INTEGRATOR, "integrator", 1);
    drill.controls.insert("crash-restart".into(), false);
    request.drill = Some(f.seal("failed-drill.toml", &drill));
    assert!(
        f.run(INTEGRATOR, Action::Authority { request: request.clone() }).unwrap_err().contains("control did not pass")
    );
    drill.controls.insert("crash-restart".into(), true);
    let source = f.root.join("crates/pipeline/src/changed.rs");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "changed after drill").unwrap();
    request.drill = Some(f.seal("stale-drill.toml", &drill));
    assert!(
        f.run(INTEGRATOR, Action::Authority { request: request.clone() })
            .unwrap_err()
            .contains("source baseline changed")
    );
    fs::remove_file(source).unwrap();
    request.drill = Some(f.seal("passed-drill.toml", &drill));
    assert!(
        f.run(INTEGRATOR, Action::Authority { request: request.clone() })
            .unwrap_err()
            .contains("isolated repository is missing")
    );
    let isolated = Fixture::new();
    drill.isolated_checkout = isolated.root.clone();
    request.drill = Some(f.seal("synthetic-drill.toml", &drill));
    assert!(
        f.run(INTEGRATOR, Action::Authority { request: request.clone() })
            .unwrap_err()
            .contains("boolean/text summaries are insufficient")
    );
    let policy_path = f.root.join(".scratch/decomp-parallel/authority.toml");
    let before = fs::read(&policy_path).unwrap();
    request.operation = "validate-activation".into();
    assert!(
        f.run(INTEGRATOR, Action::Authority { request })
            .unwrap_err()
            .contains("boolean/text summaries are insufficient")
    );
    assert_eq!(fs::read(policy_path).unwrap(), before);
    let status: authority::Receipt = f
        .run(INTEGRATOR, Action::Authority { request: f.authority("status", INTEGRATOR, "integrator", 1) })
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(status.status, "prepared");
    let request = f.queue("next", DISPATCHER, "dispatcher", 1, "new-effort-bypass");
    assert!(f.run(DISPATCHER, Action::Queue { request }).unwrap_err().contains("not admitted"));
}

#[test]
fn submitted_only_legacy_integrator_requires_checked_stop_without_releasing_submission() {
    let f = Fixture::new();
    let mut next = f.queue("next", INTEGRATOR, "integrator", 1, "legacy-effort");
    next.authority = None;
    next.jobs = vec![job("submitted-legacy", "src/legacy.rs")];
    let receipt: Receipt = f.run(INTEGRATOR, Action::Queue { request: next }).unwrap().try_into().unwrap();
    let claim = receipt.assignment.unwrap().claim;
    let mut update = f.queue("record", INTEGRATOR, "integrator", 1, "legacy-effort");
    update.authority = None;
    update.update = Some(Update::Started { claim: claim.clone(), agent: CHILD.into() });
    f.run(INTEGRATOR, Action::Queue { request: update.clone() }).unwrap();
    let commit = String::from_utf8(
        Command::new("git").args(["rev-parse", "HEAD"]).current_dir(&f.root).output().unwrap().stdout,
    )
    .unwrap()
    .trim()
    .to_owned();
    update.update = Some(Update::Finished {
        claim: claim.clone(),
        result: WorkResult::Submitted {
            commit,
            evidence: vec![f.artifact("submitted-legacy-evidence.txt", "retained legacy submission evidence")],
        },
    });
    f.run(INTEGRATOR, Action::Queue { request: update }).unwrap();
    let ledger_path = f.root.join(".scratch/legacy-effort/ledger.toml");
    let before = fs::read(&ledger_path).unwrap();
    let bridge = f.seal(
        "submitted-bridge.toml",
        &BTreeMap::from([
            ("schema", toml::Value::String("decomp-legacy-owner-bridge-v1".into())),
            (
                "holdings",
                toml::Value::Array(vec![
                    toml::Value::try_from(BTreeMap::from([
                        ("id", toml::Value::String("submitted-legacy".into())),
                        ("effort", toml::Value::String("legacy-effort".into())),
                        ("kind", toml::Value::String("durable-claim".into())),
                        ("state", toml::Value::String("submitted".into())),
                        ("retained", toml::Value::Boolean(true)),
                        ("owner_thread", toml::Value::String(CHILD.into())),
                        ("parent_thread", toml::Value::String(INTEGRATOR.into())),
                        ("claim", toml::Value::try_from(&claim).unwrap()),
                        ("writable_paths", toml::Value::try_from(vec!["src/legacy.rs"]).unwrap()),
                    ]))
                    .unwrap(),
                ]),
            ),
        ]),
    );
    let mut preparation = authority::Preparation {
        canonical_effort: "decomp-parallel".into(),
        capacity: 3,
        admitted_efforts: vec!["legacy-effort".into()],
        drill_efforts: vec!["drill-a".into()],
        bridge,
        integrator: RoleRegistration {
            token: "integrator".into(),
            kind: RoleKind::Integrator,
            session_id: REPLACEMENT.into(),
            observation: f.inspect(REPLACEMENT),
        },
    };
    let mut request = f.authority("prepare", REPLACEMENT, "integrator", 1);
    request.preparation = Some(preparation.clone());
    assert!(
        f.run(REPLACEMENT, Action::Authority { request: request.clone() })
            .unwrap_err()
            .contains("checked stopped-owner transfer")
    );
    let transfer = f.transfer(
        "integrator",
        INTEGRATOR,
        vec![f.child("legacy-effort", &claim, Some(CHILD), "completed")],
        "interrupted",
    );
    let mut proof: authority::TransferObservation =
        toml::from_str(&fs::read_to_string(transfer.transfer.unwrap().observation.path).unwrap()).unwrap();
    proof.generation = 0;
    let transfer = f.seal("submitted-generation-zero-transfer.toml", &proof);
    preparation.integrator.observation = f.seal(
        "submitted-registration.toml",
        &authority::SessionObservation {
            schema: "decomp-session-observation-v1".into(),
            session_id: REPLACEMENT.into(),
            status: "running".into(),
            observed_at: "2026-10-13T09:00:00Z".into(),
            evidence: vec![transfer],
        },
    );
    request.preparation = Some(preparation);
    f.run(REPLACEMENT, Action::Authority { request }).unwrap();
    assert_eq!(fs::read(ledger_path).unwrap(), before);
}

struct TicketControl {
    fixture: Fixture,
    frozen: authority::drill_evidence::Frozen,
    reservation: Artifact,
    holder: Claim,
    candidate: Job,
}
impl TicketControl {
    fn new() -> Self {
        let fixture = Fixture::new();
        let engine = fixture.artifact("frozen-engine", "validator fixture; no actual Codex drill execution");
        let frozen = authority::drill_evidence::Frozen {
            schema: "decomp-drill-frozen-input-v1".into(),
            engine_binary: engine.clone(),
            engine_source_checkout: fixture.root.clone(),
            engine_source_commit: "0".repeat(40),
            engine_source_fingerprint: "fixture".into(),
            engine_build: engine.clone(),
            isolated_fixture_checkout: fixture.root.clone(),
            isolated_fixture_baseline: "0".repeat(40),
            harness_source: engine,
            created_at: "fixture".into(),
        };
        let mut held = job("held-original", "docs/held.md");
        held.ticket = Some(".scratch/drill/issues/shared.md".into());
        let holder =
            Claim { id: held.id.clone(), revision: held.revision.clone(), attempt: 7, coordinator: "holder".into() };
        let candidate =
            Job { id: "competing".into(), writable_paths: vec!["docs/competing.md".into()], ..held.clone() };
        let mut control = Self {
            fixture,
            frozen,
            reservation: Artifact { path: PathBuf::new(), sha256: String::new() },
            holder,
            candidate,
        };
        control.reservation = control.receipt(
            "reservation",
            DISPATCHER,
            "holder",
            vec![held.clone()],
            toml::Value::try_from(BTreeMap::from([(
                "assignment",
                toml::Value::try_from(BTreeMap::from([
                    ("claim", toml::Value::try_from(&control.holder).unwrap()),
                    ("job", toml::Value::try_from(held).unwrap()),
                ]))
                .unwrap(),
            )]))
            .unwrap(),
        );
        control
    }
    fn receipt(&self, name: &str, session: &str, coordinator: &str, jobs: Vec<Job>, output: toml::Value) -> Artifact {
        let scopes: Vec<_> = jobs
            .iter()
            .enumerate()
            .map(|(index, job)| {
                self.fixture.seal(
                    &format!("{name}-scope-{index}.toml"),
                    &BTreeMap::from([
                        ("schema", toml::Value::String("decomp-infrastructure-scope-v1".into())),
                        ("job", toml::Value::try_from(job).unwrap()),
                    ]),
                )
            })
            .collect();
        let request = self.fixture.seal(
            &format!("{name}-request.toml"),
            &BTreeMap::from([
                ("operation", toml::Value::String("infrastructure-next".into())),
                (
                    "queue",
                    toml::Value::try_from(self.fixture.queue("next", session, coordinator, 1, "drill-a")).unwrap(),
                ),
                (
                    "admission",
                    toml::Value::try_from(BTreeMap::from([
                        ("domain", toml::Value::String("infrastructure".into())),
                        ("scopes", toml::Value::try_from(scopes).unwrap()),
                    ]))
                    .unwrap(),
                ),
            ]),
        );
        let process = authority::drill_evidence::Process {
            pid: 100,
            boot_id: INTEGRATOR.into(),
            start_ticks: 1,
            kernel_state: Some("R".into()),
            argv: vec![self.frozen.engine_binary.path.display().to_string(), "decomp-session".into()],
            cwd: self.fixture.root.clone(),
            observed_at: "1".into(),
            status: "running".into(),
        };
        self.fixture.seal(
            &format!("{name}-execution.toml"),
            &authority::drill_evidence::CommandReceipt {
                schema: "decomp-drill-command-v1".into(),
                session_id: session.into(),
                pid: process.pid,
                process,
                route: "decomp-session".into(),
                request,
                stdout: self.fixture.seal(&format!("{name}-stdout.toml"), &output),
                stderr: self.fixture.artifact(&format!("{name}-stderr.txt"), ""),
                exit_code: 0,
                observed_at: "2".into(),
            },
        )
    }
    fn competitor(&self, name: &str, holder: Option<&Claim>, state: &str, stale: bool) -> Artifact {
        let jobs: Vec<_> = holder
            .into_iter()
            .map(|claim| {
                toml::Value::try_from(BTreeMap::from([
                    ("claim", toml::Value::try_from(claim).unwrap()),
                    ("state", toml::Value::String(state.into())),
                    ("stale", toml::Value::Boolean(stale)),
                ]))
                .unwrap()
            })
            .collect();
        self.receipt(
            name,
            REPLACEMENT,
            "competitor",
            vec![self.candidate.clone()],
            toml::Value::try_from(BTreeMap::from([
                (
                    "deferred",
                    toml::Value::try_from(BTreeMap::from([(self.candidate.id.clone(), "ticket is claimed")])).unwrap(),
                ),
                ("jobs", toml::Value::Array(jobs)),
            ]))
            .unwrap(),
        )
    }
    fn check(&self, receipts: Vec<Artifact>) -> bool {
        authority::drill_evidence::validate_duplicate_ticket_receipts(
            &receipts,
            &self.frozen,
            &self.fixture.root,
            &BTreeSet::from([DISPATCHER.into(), REPLACEMENT.into()]),
        )
        .unwrap()
    }
}

#[test]
fn claimed_ticket_control_requires_exact_retained_owner_and_disjoint_competitor() {
    let mut control = TicketControl::new();
    let compete = control.competitor("valid", Some(&control.holder), "running", false);
    assert!(control.check(vec![control.reservation.clone(), compete.clone()]));
    assert!(!control.check(vec![compete.clone(), control.reservation.clone()]));
    assert!(!control.check(vec![compete]));
    let unrelated = control.competitor("missing", None, "running", false);
    assert!(!control.check(vec![control.reservation.clone(), unrelated]));
    let altered = Claim { attempt: control.holder.attempt + 1, ..control.holder.clone() };
    let stale_attempt = control.competitor("different-attempt", Some(&altered), "running", false);
    assert!(!control.check(vec![control.reservation.clone(), stale_attempt]));
    let stale = control.competitor("stale", Some(&control.holder), "running", true);
    assert!(!control.check(vec![control.reservation.clone(), stale]));
    let complete = control.competitor("completed", Some(&control.holder), "complete", false);
    assert!(!control.check(vec![control.reservation.clone(), complete]));
    control.candidate.ticket = Some(".scratch/drill/issues/unrelated.md".into());
    let ticket = control.competitor("unrelated-ticket", Some(&control.holder), "running", false);
    assert!(!control.check(vec![control.reservation.clone(), ticket]));
    control.candidate.ticket = Some(".scratch/drill/issues/shared.md".into());
    control.candidate.writable_paths = vec!["docs/held.md".into()];
    let overlap = control.competitor("overlapping", Some(&control.holder), "running", false);
    assert!(!control.check(vec![control.reservation.clone(), overlap]));
    control.candidate.writable_paths = vec!["docs/competing.md".into()];
    control.candidate.id = control.holder.id.clone();
    let same = control.competitor("same-job", Some(&control.holder), "running", false);
    assert!(!control.check(vec![control.reservation.clone(), same]));
    control.candidate.id = "competing".into();
    control.candidate.ticket = Some(".scratch/../outside.md".into());
    let escaped = control.competitor("escaped-ticket", Some(&control.holder), "running", false);
    assert!(!control.check(vec![control.reservation.clone(), escaped]));
}

#[test]
fn managed_owner_stop_requires_exact_child_thread_and_preserves_old_attempt_history() {
    let f = Fixture::new();
    f.prepare(f.empty_bridge());
    f.register();
    let claim =
        f.next(DISPATCHER, "dispatcher", "drill-a", job("stopped-child", "src/stopped.rs")).assignment.unwrap().claim;
    f.record(DISPATCHER, "dispatcher", 1, "drill-a", Update::Started { claim: claim.clone(), agent: CHILD.into() })
        .unwrap();
    let evidence = f.artifact("observations/stop-raw.txt", "observed actual child ended");
    let mut proof = toml::map::Map::new();
    proof.insert("agent".into(), CHILD.into());
    proof.insert("coordinator".into(), "dispatcher".into());
    proof.insert("status".into(), "interrupted".into());
    proof.insert("claim".into(), toml::Value::try_from(&claim).unwrap());
    proof.insert("session_id".into(), REPLACEMENT.into());
    proof.insert("observed_at".into(), "2026-10-13T09:00:00Z".into());
    proof.insert("evidence".into(), toml::Value::try_from(vec![evidence]).unwrap());
    let wrong = f.seal("wrong-stopped.toml", &proof);
    assert!(
        f.record(
            DISPATCHER,
            "dispatcher",
            1,
            "drill-a",
            Update::OwnerStopped { claim: claim.clone(), observation: wrong }
        )
        .unwrap_err()
        .contains("another real child")
    );
    proof.insert("session_id".into(), CHILD.into());
    let stopped = Update::OwnerStopped { claim: claim.clone(), observation: f.seal("stopped.toml", &proof) };
    f.record(DISPATCHER, "dispatcher", 1, "drill-a", stopped.clone()).unwrap();
    f.record(DISPATCHER, "dispatcher", 1, "drill-a", stopped.clone()).unwrap();
    let next = f.next(DISPATCHER, "dispatcher", "drill-a", job("stopped-child", "src/stopped.rs")).assignment.unwrap();
    assert_eq!(next.claim.attempt, claim.attempt + 1);
    assert!(f.record(DISPATCHER, "dispatcher", 1, "drill-a", stopped).unwrap_err().contains("stale assignment"));
    let status: Receipt = f
        .run(DISPATCHER, Action::Queue { request: f.queue("status", DISPATCHER, "dispatcher", 1, "drill-a") })
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(status.jobs[0].history[0].claim.as_ref(), Some(&claim));
}

#[test]
fn checked_maintenance_exception_is_exact_and_retains_original_legacy_fence() {
    let f = Fixture::new();
    let inspection = f.artifact(
        "observations/maintenance-raw.txt",
        "checked stopped old source owner and authorized exact maintenance",
    );
    let recommendation = f.artifact("recommendation.toml", "[[adoptions]]\nid='maintenance'\neffort='drill-a'\noriginal_hold_ids=['legacy']\nwritable_paths=['src/held.rs']\n");
    let approval = f.artifact("approval.toml", &format!("schema='decomp-preparation-maintenance-approval-v1'\nstatus='approved'\nlegacy_claims_released=false\ngameplay_launch_authorized=false\n[recommendation]\npath='{}'\nsha256='{}'\n[observation]\npath='{}'\nsha256='{}'\n[[workers]]\nadoption='maintenance'\nsession_id='{}'\nwritable_paths=['src/held.rs']\n", recommendation.path.display(), recommendation.sha256, inspection.path.display(), inspection.sha256, DISPATCHER));
    let bridge = f.artifact("bridge.toml", &format!("schema='decomp-legacy-owner-bridge-v1'\n[maintenance_approval]\npath='{}'\nsha256='{}'\n[[holdings]]\nid='legacy'\neffort='legacy-effort'\nkind='markdown-hold'\nstate='claimed'\nretained=true\nticket='.scratch/legacy-effort/issues/01.md'\nwritable_paths=['src/held.rs']\n[[maintenance_adoptions]]\nid='maintenance'\nholding_ids=['legacy']\neffort='drill-a'\nsession_id='{}'\nwritable_paths=['src/held.rs']\nobservation={{path='{}',sha256='{}'}}\n", approval.path.display(), approval.sha256, DISPATCHER, approval.path.display(), approval.sha256));
    f.prepare(bridge);
    f.register();
    let wrong_session = f.next(INTEGRATOR, "integrator", "drill-b", job("wrong-session", "src/held.rs"));
    assert!(wrong_session.assignment.is_none());
    let expanded = f.next(DISPATCHER, "dispatcher", "drill-a", job("expanded", "src"));
    assert!(expanded.assignment.is_none());
    let allowed = f.next(DISPATCHER, "dispatcher", "drill-a", job("allowed", "src/held.rs"));
    assert!(allowed.assignment.is_some());
    let other = f.next(INTEGRATOR, "integrator", "drill-b", job("new-overlap", "src/held.rs"));
    assert!(other.assignment.is_none());
    assert!(other.deferred["new-overlap"].contains("writable ownership"));
    let contents = fs::read_to_string(f.root.join("bridge.toml")).unwrap();
    assert!(contents.contains("retained=true"));
}

#[test]
fn initial_legacy_integrator_requires_checked_stop_and_retains_exact_original_tuple() {
    let f = Fixture::new();
    let mut next = f.queue("next", INTEGRATOR, "integrator", 1, "legacy-effort");
    next.authority = None;
    next.jobs = vec![job("legacy-work", "src/legacy.rs")];
    let reserved: Receipt = f.run(INTEGRATOR, Action::Queue { request: next }).unwrap().try_into().unwrap();
    let claim = reserved.assignment.unwrap().claim;
    let mut started = f.queue("record", INTEGRATOR, "integrator", 1, "legacy-effort");
    started.authority = None;
    started.update = Some(Update::Started { claim: claim.clone(), agent: INTEGRATOR.into() });
    f.run(INTEGRATOR, Action::Queue { request: started }).unwrap();
    let ledger = Artifact {
        path: f.root.join(".scratch/legacy-effort/ledger.toml"),
        sha256: digest(&f.root.join(".scratch/legacy-effort/ledger.toml")).unwrap(),
    };
    let mut bridge = toml::map::Map::new();
    bridge.insert("schema".into(), "decomp-legacy-owner-bridge-v1".into());
    bridge.insert("ledger".into(), toml::Value::try_from(ledger).unwrap());
    let mut holding = toml::map::Map::new();
    for (key, value) in [
        ("id", "legacy-work"),
        ("effort", "legacy-effort"),
        ("kind", "durable-claim"),
        ("state", "running"),
        ("owner_thread", INTEGRATOR),
    ] {
        holding.insert(key.into(), value.into());
    }
    holding.insert("retained".into(), true.into());
    holding.insert("claim".into(), toml::Value::try_from(&claim).unwrap());
    holding.insert("writable_paths".into(), toml::Value::try_from(vec!["src/legacy.rs"]).unwrap());
    bridge.insert("holdings".into(), toml::Value::Array(vec![toml::Value::Table(holding)]));
    let bridge = f.seal("legacy-bridge.toml", &bridge);
    let replacement = f.inspect(REPLACEMENT);
    let mut preparation = authority::Preparation {
        canonical_effort: "decomp-parallel".into(),
        capacity: 3,
        admitted_efforts: vec!["legacy-effort".into()],
        drill_efforts: vec!["drill-a".into()],
        bridge,
        integrator: RoleRegistration {
            token: "integrator".into(),
            kind: RoleKind::Integrator,
            session_id: REPLACEMENT.into(),
            observation: replacement.clone(),
        },
    };
    let mut request = f.authority("prepare", REPLACEMENT, "integrator", 1);
    request.preparation = Some(preparation.clone());
    assert!(
        f.run(REPLACEMENT, Action::Authority { request: request.clone() })
            .unwrap_err()
            .contains("checked stopped-owner transfer")
    );
    let transfer = f.transfer(
        "integrator",
        INTEGRATOR,
        vec![f.child("legacy-effort", &claim, Some(INTEGRATOR), "interrupted")],
        "interrupted",
    );
    let mut proof: authority::TransferObservation =
        toml::from_str(&fs::read_to_string(transfer.transfer.unwrap().observation.path).unwrap()).unwrap();
    proof.generation = 0;
    let transfer = f.seal("initial-transfer.toml", &proof);
    let registration = f.seal(
        "initial-registration.toml",
        &authority::SessionObservation {
            schema: "decomp-session-observation-v1".into(),
            session_id: REPLACEMENT.into(),
            status: "running".into(),
            observed_at: "2026-10-13T09:00:00Z".into(),
            evidence: vec![transfer],
        },
    );
    preparation.integrator.observation = registration;
    request.preparation = Some(preparation);
    f.run(REPLACEMENT, Action::Authority { request }).unwrap();
    let status: Receipt = f
        .run(REPLACEMENT, Action::Queue { request: f.queue("status", REPLACEMENT, "integrator", 1, "legacy-effort") })
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(status.jobs[0].claim.as_ref(), Some(&claim));
    assert_eq!(status.jobs[0].state, State::Running);
    let mut expanded = f.queue("next", REPLACEMENT, "integrator", 1, "legacy-effort");
    let mut candidate = job("legacy-work", "src/legacy.rs");
    candidate.revision = "revision-2".into();
    candidate.writable_paths.push("src/new.rs".into());
    expanded.jobs = vec![candidate];
    let receipt: Receipt = f.run(REPLACEMENT, Action::Queue { request: expanded }).unwrap().try_into().unwrap();
    assert_eq!(receipt.jobs[0].job.writable_paths, vec![PathBuf::from("src/legacy.rs")]);
    assert!(receipt.jobs[0].pending_job.is_some());
}
