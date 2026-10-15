//! Preserved Ceres acceptance requirements. Readiness is deliberately separate
//! from a passing gate: these original jobs need adapters that are not yet
//! available to the confined supervisor, and historical evidence cannot replace
//! candidate or integrated-source verification.
use crate::assignment;
use crate::translation_migration::OriginalClaim;
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};
use tool_execution::bounded;
use translation_execution::{
    model::{digest_valid, sealed, sha256},
    retention::{self, Reference},
};

const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 64 * 1024 * 1024;
const MAX_REFERENCES: usize = 128;
const MAX_DIAGNOSTIC_BYTES: usize = 4096;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum LegacyKind {
    CompactController,
    SharedGameHost,
}

impl LegacyKind {
    fn id(self) -> &'static str {
        match self {
            Self::CompactController => "compact-controller",
            Self::SharedGameHost => "shared-game-host",
        }
    }

    fn revision(self) -> &'static str {
        match self {
            Self::CompactController => "48656a2-compact-v1",
            Self::SharedGameHost => "48656a2-host-v1",
        }
    }

    fn paths(self) -> BTreeSet<String> {
        match self {
            Self::CompactController => [
                "crates/pipeline/src/decomp_prefix.rs",
                "crates/pipeline/src/prefix_observer.lua.in",
                "crates/pipeline/src/prefix_compact_observer.lua.in",
                "crates/pipeline/tests/prefix.rs",
                "docs/DECOMP_PREFIX.md",
            ],
            Self::SharedGameHost => [
                "crates/port/src/game_host.rs",
                "crates/port/src/lib.rs",
                "crates/port/tests/game_host.rs",
                "tools/port/game-host-projectile.smf",
                "tools/port/game-host-room-scroll.smf",
            ],
        }
        .into_iter()
        .map(str::to_owned)
        .collect()
    }
}

/// Installed alongside the exact original migration claim. Extra sealed inputs
/// preserve reviewed historical requirements; none may supply an executable,
/// replacement command, smaller family selection, or accepting result.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LegacyContract {
    pub schema: String,
    pub kind: LegacyKind,
    pub original_claim: Reference,
    pub submission: Reference,
    pub integration_baseline: String,
    pub source_files: BTreeMap<String, String>,
    pub acceptance_inputs: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LegacyRequirement {
    pub id: String,
    /// A preserved required command, not permission to run it outside the
    /// confined supervisor. Non-command requirements have an empty argv.
    pub argv: Vec<String>,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreparedLegacy {
    pub schema: String,
    pub kind: LegacyKind,
    pub contract_sha256: String,
    pub source_commit: String,
    pub source_files: BTreeMap<String, String>,
    pub dependencies_sha256: String,
    pub verification_sha256: String,
    pub requirements: Vec<LegacyRequirement>,
    pub references: Vec<Reference>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LegacyReadiness {
    pub schema: String,
    pub contract_sha256: String,
    pub source_commit: String,
    pub ready: bool,
    pub accepted: bool,
    pub producers_started: u64,
    pub requirements: Vec<LegacyRequirement>,
    pub missing_prerequisites: Vec<String>,
    pub diagnostic: String,
    pub limitations: Vec<String>,
}

pub(crate) struct LegacyAdmissionData {
    pub scope: Reference,
    pub references: Vec<Reference>,
    pub limitations: Vec<String>,
}

#[derive(Deserialize)]
struct Submission {
    schema: String,
    status: String,
    accepted: bool,
    commit: String,
    baseline: String,
    #[serde(default)]
    evidence: Vec<assignment::Artifact>,
    #[serde(default)]
    source_sha256: BTreeMap<String, String>,
    #[serde(default)]
    verification_argv: Vec<String>,
    #[serde(default)]
    verification_timeout_seconds: u64,
    #[serde(default)]
    unit_tests_passed: usize,
    #[serde(default)]
    public_tests_passed: usize,
    #[serde(default)]
    tests_executed: usize,
    #[serde(default)]
    tests_passed: usize,
}

#[derive(Default)]
struct ReadSet {
    references: BTreeMap<PathBuf, Reference>,
    bytes: u64,
}

impl ReadSet {
    fn read(&mut self, reference: &Reference) -> Result<Vec<u8>> {
        ensure!(
            digest_valid(&reference.sha256) && reference.bytes <= MAX_FILE_BYTES,
            "invalid legacy reference seal or extent"
        );
        ensure!(
            self.references.len() < MAX_REFERENCES || self.references.contains_key(&reference.path),
            "legacy reference count exceeded"
        );
        self.bytes = self.bytes.checked_add(reference.bytes).context("legacy input extent overflow")?;
        ensure!(self.bytes <= MAX_TOTAL_BYTES, "legacy input aggregate byte budget exceeded");
        retention::authenticate_reference(reference, MAX_FILE_BYTES)?;
        let bytes = bounded::read_limit(&reference.path, MAX_FILE_BYTES)?;
        ensure!(
            bytes.len() as u64 == reference.bytes && sha256(&bytes) == reference.sha256,
            "legacy sealed input changed during reading"
        );
        if let Some(previous) = self.references.insert(reference.path.clone(), reference.clone()) {
            ensure!(previous == *reference, "legacy reference has conflicting seals");
        }
        Ok(bytes)
    }

    fn artifact(&mut self, artifact: &assignment::Artifact) -> Result<()> {
        let reference = retention::seal_file(&artifact.path, MAX_FILE_BYTES)?;
        ensure!(reference.sha256 == artifact.sha256, "legacy historical evidence changed");
        self.read(&reference)?;
        Ok(())
    }
}

fn exact_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn host_argv() -> Vec<String> {
    [
        "cargo",
        "test",
        "-p",
        "port",
        "--locked",
        "--offline",
        "--test",
        "game_host",
        "--test",
        "samus_projectile_frame",
        "--test",
        "room_scroll",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn requirement(id: &str, argv: &[&str], description: &str) -> LegacyRequirement {
    LegacyRequirement {
        id: id.into(),
        argv: argv.iter().map(|arg| (*arg).into()).collect(),
        description: description.into(),
    }
}

fn requirements(kind: LegacyKind) -> Vec<LegacyRequirement> {
    let mut checks = match kind {
        LegacyKind::CompactController => vec![
            requirement("controller-unit-tests", &["cargo", "test", "-p", "pipeline", "--locked", "--offline", "--bin", "sm-pipeline", "decomp_prefix::tests"], "Run the original eight compact/full controller unit controls on candidate and integrated source; reject zero or filtered-away tests."),
            requirement("controller-public-tests", &["cargo", "test", "-p", "pipeline", "--locked", "--offline", "--test", "prefix"], "Run both original public prefix controls on candidate and integrated source; preserve default full observer behavior."),
            requirement("compact-causal-diagnostic", &[], "Execute the original whole-experiment 60-second compact diagnostic through frame 2240 with independent changed-gameplay control, ordered polls, completed-state comparison and retained failures."),
            requirement("historical-capture-compatibility", &[], "Authenticate retained original full and compact captures with their exact historical readers and private inputs after cleanup; preserve nonaccepting diagnostic semantics."),
        ],
        LegacyKind::SharedGameHost => vec![
            LegacyRequirement { id: "shared-host-rust-tests".into(), argv: host_argv(), description: "Run the original 31 Rust controls within their 60-second bound on candidate and integrated source, including refusal before ownership and fatal failure after host effects.".into() },
            requirement("material-host-callbacks", &[], "Run both actual-C host matrices and authenticate all ten pairs with material PLM allocation, HDMA allocation and all 17 ordered VRAM flush callbacks."),
            requirement("all-affected-offline-families", &[], "Verify the complete affected Offline catalog on frozen merged source. Shared crates/port/src/lib.rs ownership leaves every family affected; the production preparation identifies 176 families. Preserve specialized callback and fixture-generation requirements."),
        ],
    };
    checks.push(requirement("retained-cleanup", &[], "Complete both cleanup passes, retire producer worktrees and authenticate retained evidence before original-claim completion."));
    checks
}

fn limitations() -> Vec<String> {
    vec![
        "This contract preserves original Ceres ownership and acceptance requirements; readiness is not a verification gate.".into(),
        "Historical submission evidence describes its original source and cannot establish candidate or merged-source verification.".into(),
        "No gameplay stage, Live replay, Ceres escape or original-ROM parity is established.".into(),
    ]
}

fn original_binding(contract: &LegacyContract, original: &OriginalClaim) -> Result<()> {
    ensure!(
        contract.schema == "translation-legacy-contract-v1" && exact_commit(&contract.integration_baseline),
        "unsupported legacy contract or inexact integration baseline"
    );
    ensure!(
        original.schema == "translation-original-claim-v1"
            && original.effort == "ceres-continuation"
            && original.job.kind == assignment::Kind::Implement
            && original.state == assignment::State::Submitted
            && original.job.id == contract.kind.id()
            && original.claim.id == original.job.id
            && original.job.revision == contract.kind.revision()
            && original.claim.revision == original.job.revision
            && original.claim.attempt == 1
            && original.claim.coordinator == "ceres-20261013"
            && exact_commit(&original.source_commit),
        "legacy contract differs from the original submitted Ceres lineage"
    );
    let paths = contract.kind.paths();
    ensure!(
        original.job.writable_paths.iter().map(|path| path.to_string_lossy().into_owned()).collect::<BTreeSet<_>>()
            == paths
            && original.job.writable_paths.len() == paths.len()
            && original.source_files.keys().cloned().collect::<BTreeSet<_>>() == paths
            && contract.source_files == original.source_files
            && contract.source_files.values().all(|hash| digest_valid(hash)),
        "legacy contract changed original writable source ownership"
    );
    let Some(assignment::WorkResult::Submitted { commit, evidence }) = &original.result else {
        bail!("legacy contract lacks the exact original submitted result")
    };
    ensure!(
        commit == &original.source_commit
            && evidence
                .iter()
                .any(|item| item.path == contract.submission.path && item.sha256 == contract.submission.sha256),
        "legacy contract submission is not the original result evidence"
    );
    ensure!(contract.acceptance_inputs.len() <= MAX_REFERENCES - 16, "legacy acceptance input count exceeded");
    Ok(())
}

fn prepare_inner(contract: &LegacyContract) -> Result<(PreparedLegacy, OriginalClaim)> {
    let mut inputs = ReadSet::default();
    let original: OriginalClaim = serde_json::from_slice(&inputs.read(&contract.original_claim)?)?;
    original_binding(contract, &original)?;
    let submission: Submission = toml::from_str(std::str::from_utf8(&inputs.read(&contract.submission)?)?)?;
    ensure!(submission.evidence.len() <= MAX_REFERENCES - 16, "legacy historical evidence count exceeded");
    let baseline = original.job.inputs.get("base-head").context("legacy original baseline input missing")?;
    ensure!(
        (7..=40).contains(&baseline.len())
            && baseline.bytes().all(|byte| byte.is_ascii_hexdigit())
            && submission.baseline.starts_with(baseline)
            && submission.commit == original.source_commit
            && !submission.accepted,
        "legacy submitted source, baseline or nonaccepting semantics changed"
    );
    match contract.kind {
        LegacyKind::CompactController => ensure!(
            submission.schema == "ceres-controller-handoff-v1"
                && submission.status == "submitted"
                && submission.unit_tests_passed == 8
                && submission.public_tests_passed == 2,
            "original controller evidence contract changed"
        ),
        LegacyKind::SharedGameHost => ensure!(
            submission.schema == "shared-game-host-implementation-v1"
                && submission.status == "submitted-and-merged"
                && submission.source_sha256 == contract.source_files
                && submission.verification_argv == host_argv()
                && submission.verification_timeout_seconds == 60
                && submission.tests_executed == 31
                && submission.tests_passed == 31,
            "original shared-host evidence contract changed"
        ),
    }
    for artifact in &submission.evidence {
        inputs.artifact(artifact)?;
    }
    ensure!(!submission.evidence.is_empty(), "legacy submission lacks retained historical evidence");
    if let Some(assignment::WorkResult::Submitted { evidence, .. }) = &original.result {
        for artifact in evidence {
            inputs.artifact(artifact)?;
        }
    }
    for artifact in &original.observations {
        inputs.artifact(artifact)?;
    }
    for (path, hash) in &contract.source_files {
        let reference = retention::seal_file(&original.source_checkout.join(path), MAX_FILE_BYTES)?;
        ensure!(&reference.sha256 == hash, "legacy original submitted source snapshot changed");
        inputs.read(&reference)?;
    }
    for reference in &contract.acceptance_inputs {
        inputs.read(reference)?;
    }
    let references = inputs.references.into_values().collect::<Vec<_>>();
    let requirements = requirements(contract.kind);
    let limitations = limitations();
    Ok((
        PreparedLegacy {
            schema: "translation-legacy-prepared-v1".into(),
            kind: contract.kind,
            contract_sha256: sealed(contract)?,
            source_commit: original.source_commit.clone(),
            source_files: contract.source_files.clone(),
            dependencies_sha256: sealed(&("translation-legacy-dependencies-v1", contract, &references))?,
            verification_sha256: sealed(&(
                "translation-legacy-verification-v1",
                contract.kind,
                &requirements,
                &limitations,
            ))?,
            requirements,
            references,
            limitations,
        },
        original,
    ))
}

/// Read-only preparation. This performs no process launch, filesystem writes,
/// claim changes, acceptance publication or source replacement.
pub(crate) fn prepare(contract: &LegacyContract) -> Result<PreparedLegacy> {
    decomp_source::context::limited(|| Ok(prepare_inner(contract)?.0))
}

pub(crate) fn readiness(prepared: &PreparedLegacy) -> LegacyReadiness {
    let mut missing = vec!["A confined Cargo adapter with sealed dependency/toolchain closure, hard output/time limits and nonzero original test-count authentication is not installed.".into()];
    match prepared.kind {
        LegacyKind::CompactController => {
            missing.push("The bounded compact causal experiment and historical capture compatibility do not yet have confined, source-bound producer/authenticator adapters.".into());
        }
        LegacyKind::SharedGameHost => {
            missing.push("The specialized material PLM/HDMA/VRAM callback matrices and complete affected Offline acceptance (176 families in this production preparation) do not yet have complete confined execution recipes.".into());
        }
    }
    let mut diagnostic = format!(
        "{} preserves its original submission but cannot run acceptance: {}",
        prepared.kind.id(),
        missing.join(" ")
    );
    if diagnostic.len() > MAX_DIAGNOSTIC_BYTES {
        let mut end = MAX_DIAGNOSTIC_BYTES;
        while !diagnostic.is_char_boundary(end) {
            end -= 1;
        }
        diagnostic.truncate(end);
    }
    LegacyReadiness {
        schema: "translation-legacy-readiness-v1".into(),
        contract_sha256: prepared.contract_sha256.clone(),
        source_commit: prepared.source_commit.clone(),
        ready: false,
        accepted: false,
        producers_started: 0,
        requirements: prepared.requirements.clone(),
        missing_prerequisites: missing,
        diagnostic,
        limitations: prepared.limitations.clone(),
    }
}

pub(crate) fn authenticate(
    contract: &LegacyContract,
    prepared: &PreparedLegacy,
    report: &LegacyReadiness,
) -> Result<()> {
    ensure!(prepare(contract)? == *prepared, "legacy prepared contract or immutable inputs changed");
    ensure!(readiness(prepared) == *report, "legacy readiness result changed or falsely claimed acceptance");
    Ok(())
}

pub(crate) fn validate_original(contract: &LegacyContract, original: &OriginalClaim) -> Result<LegacyAdmissionData> {
    decomp_source::context::limited(|| {
        let (prepared, frozen) = prepare_inner(contract)?;
        ensure!(frozen == *original, "legacy managed admission differs from its exact frozen original claim");
        Ok(LegacyAdmissionData {
            scope: contract.original_claim.clone(),
            references: prepared.references,
            limitations: prepared.limitations,
        })
    })
}

pub(crate) fn authenticate_completion(_contract: &LegacyContract) -> Result<()> {
    bail!("legacy readiness-only contracts cannot authenticate completion; original acceptance adapters remain missing")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::Path,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        contract: LegacyContract,
        original: OriginalClaim,
    }

    fn write(path: &Path, bytes: impl AsRef<[u8]>) -> Reference {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
        retention::seal_file(path, MAX_FILE_BYTES).unwrap()
    }

    impl Fixture {
        fn new(kind: LegacyKind) -> Self {
            let root = std::env::temp_dir().join(format!(
                "sm-legacy-readiness-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let source = root.join("source");
            let source_files = kind
                .paths()
                .into_iter()
                .map(|path| {
                    let reference = write(&source.join(&path), format!("frozen original {path}\n"));
                    (path, reference.sha256)
                })
                .collect::<BTreeMap<_, _>>();
            let historical =
                write(&root.join("historical.toml"), "accepted = false\nstatus = 'historical diagnostic'\n");
            let mut submission = serde_json::json!({
                "schema": if kind == LegacyKind::CompactController { "ceres-controller-handoff-v1" } else { "shared-game-host-implementation-v1" },
                "status": if kind == LegacyKind::CompactController { "submitted" } else { "submitted-and-merged" },
                "accepted": false, "commit": "c".repeat(40), "baseline": "48656a2",
                "evidence": [{ "path": historical.path, "sha256": historical.sha256 }],
            });
            match kind {
                LegacyKind::CompactController => {
                    submission["unit_tests_passed"] = 8.into();
                    submission["public_tests_passed"] = 2.into();
                }
                LegacyKind::SharedGameHost => {
                    submission["source_sha256"] = serde_json::to_value(&source_files).unwrap();
                    submission["verification_argv"] = serde_json::to_value(host_argv()).unwrap();
                    submission["verification_timeout_seconds"] = 60.into();
                    submission["tests_executed"] = 31.into();
                    submission["tests_passed"] = 31.into();
                }
            }
            let submission = write(&root.join("submission.toml"), toml::to_string(&submission).unwrap());
            let original = OriginalClaim {
                schema: "translation-original-claim-v1".into(),
                checkout: root.join("checkout"),
                effort: "ceres-continuation".into(),
                job: assignment::Job {
                    id: kind.id().into(),
                    revision: kind.revision().into(),
                    kind: assignment::Kind::Implement,
                    subject: "Preserved legacy work".into(),
                    writable_paths: kind.paths().iter().map(PathBuf::from).collect(),
                    blockers: Vec::new(),
                    inputs: BTreeMap::from([("base-head".into(), "48656a2".into())]),
                    instructions: "Preserve original acceptance".into(),
                    ticket: None,
                },
                claim: assignment::Claim {
                    id: kind.id().into(),
                    revision: kind.revision().into(),
                    attempt: 1,
                    coordinator: "ceres-20261013".into(),
                },
                state: assignment::State::Submitted,
                agent: Some("original-worker".into()),
                result: Some(assignment::WorkResult::Submitted {
                    commit: "c".repeat(40),
                    evidence: vec![assignment::Artifact {
                        path: submission.path.clone(),
                        sha256: submission.sha256.clone(),
                    }],
                }),
                observations: Vec::new(),
                source_checkout: source,
                source_commit: "c".repeat(40),
                source_files: source_files.clone(),
            };
            let original_claim = write(&root.join("original.json"), serde_json::to_vec(&original).unwrap());
            let contract = LegacyContract {
                schema: "translation-legacy-contract-v1".into(),
                kind,
                original_claim,
                submission,
                integration_baseline: "d".repeat(40),
                source_files,
                acceptance_inputs: Vec::new(),
            };
            Self { root, contract, original }
        }

        fn reseal_original(&mut self) {
            self.contract.original_claim =
                write(&self.contract.original_claim.path, serde_json::to_vec(&self.original).unwrap());
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut result = BTreeMap::new();
        for entry in fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                result.extend(tree(&path));
            } else {
                result.insert(path.clone(), fs::read(&path).unwrap());
            }
        }
        result
    }

    #[test]
    fn legacy_readiness_is_repeatable_nonaccepting_and_read_only() {
        for kind in [LegacyKind::CompactController, LegacyKind::SharedGameHost] {
            let fixture = Fixture::new(kind);
            let before = tree(&fixture.root);
            let prepared = prepare(&fixture.contract).unwrap();
            assert_eq!(prepared.source_files, fixture.original.source_files);
            assert!(digest_valid(&prepared.dependencies_sha256) && digest_valid(&prepared.verification_sha256));
            let report = readiness(&prepared);
            assert!(!report.ready && !report.accepted);
            assert_eq!(report.producers_started, 0);
            assert!(report.diagnostic.len() <= MAX_DIAGNOSTIC_BYTES && !report.missing_prerequisites.is_empty());
            authenticate(&fixture.contract, &prepared, &report).unwrap();
            let admitted = validate_original(&fixture.contract, &fixture.original).unwrap();
            assert_eq!(admitted.scope, fixture.contract.original_claim);
            assert_eq!(admitted.references, prepared.references);
            assert!(authenticate_completion(&fixture.contract).is_err());
            assert_eq!(tree(&fixture.root), before);
        }
    }

    #[test]
    fn host_contract_retains_complete_callback_and_catalog_requirements() {
        let fixture = Fixture::new(LegacyKind::SharedGameHost);
        let report = readiness(&prepare(&fixture.contract).unwrap());
        assert!(report.diagnostic.contains("176 families"));
        let callbacks = report.requirements.iter().find(|check| check.id == "material-host-callbacks").unwrap();
        assert!(callbacks.description.contains("ten pairs") && callbacks.description.contains("17 ordered VRAM"));
        assert!(report.requirements.iter().any(|check| check.id == "all-affected-offline-families"));
        assert_eq!(report.requirements[0].argv, host_argv());
    }

    #[test]
    fn readiness_and_prepared_fingerprint_tamper_cannot_claim_acceptance() {
        let fixture = Fixture::new(LegacyKind::CompactController);
        let prepared = prepare(&fixture.contract).unwrap();
        let mut forged = readiness(&prepared);
        forged.ready = true;
        forged.accepted = true;
        forged.missing_prerequisites.clear();
        assert!(authenticate(&fixture.contract, &prepared, &forged).is_err());
        let report = readiness(&prepared);
        let mut prepared = prepared;
        prepared.verification_sha256 = sha256("smaller requirements");
        assert!(authenticate(&fixture.contract, &prepared, &report).is_err());
    }

    #[test]
    fn original_lineage_ownership_and_sealed_evidence_cannot_be_replaced() {
        let mut fixture = Fixture::new(LegacyKind::CompactController);
        fixture.original.claim.attempt = 2;
        fixture.reseal_original();
        assert!(prepare(&fixture.contract).is_err());
        fixture.original.claim.attempt = 1;
        fixture.original.source_files.remove("docs/DECOMP_PREFIX.md");
        fixture.contract.source_files = fixture.original.source_files.clone();
        fixture.reseal_original();
        assert!(prepare(&fixture.contract).is_err());
        let fixture = Fixture::new(LegacyKind::SharedGameHost);
        fs::write(fixture.root.join("historical.toml"), "accepted = true\n").unwrap();
        assert!(prepare(&fixture.contract).is_err());
    }

    #[test]
    fn source_tamper_and_aliased_or_oversized_references_fail_closed() {
        let fixture = Fixture::new(LegacyKind::SharedGameHost);
        fs::write(fixture.original.source_checkout.join("crates/port/src/game_host.rs"), "different code").unwrap();
        assert!(prepare(&fixture.contract).is_err());
        let mut fixture = Fixture::new(LegacyKind::CompactController);
        fixture.contract.submission.bytes = MAX_FILE_BYTES + 1;
        assert!(prepare(&fixture.contract).is_err());
        let mut fixture = Fixture::new(LegacyKind::CompactController);
        let alias = fixture.root.join("alias.json");
        std::os::unix::fs::symlink(&fixture.contract.original_claim.path, &alias).unwrap();
        fixture.contract.original_claim.path = alias;
        assert!(prepare(&fixture.contract).is_err());
    }
}
