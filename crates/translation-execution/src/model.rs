use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Component, Path},
};

pub fn sha256(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}

pub fn sealed<T: Serialize>(value: &T) -> Result<String> {
    Ok(sha256(serde_json::to_vec(value)?))
}

pub fn digest_valid(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

pub fn relative_file(value: &str) -> bool {
    !value.is_empty() && Path::new(value).components().all(|c| matches!(c, Component::Normal(_)))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClaimLineage {
    pub id: String,
    pub revision: String,
    pub attempt: u64,
    pub coordinator: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Budgets {
    pub workers: u32,
    pub max_attempts: u32,
    pub max_evidence_bytes: u64,
    pub gate_timeout_seconds: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RewriteLimits {
    pub max_context_bytes: u64,
    pub max_response_bytes: u64,
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationStage {
    Focused,
    Offline,
    Live,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FrozenPolicy {
    pub schema: String,
    pub version: u32,
    pub scope_id: String,
    pub claim: ClaimLineage,
    pub baseline_commit: String,
    pub pinned_c_sha256: String,
    pub dependencies_sha256: String,
    pub verification_sha256: String,
    pub adapter_sha256: String,
    pub approved_files: BTreeMap<String, String>,
    pub verification_stages: Vec<VerificationStage>,
    pub budgets: Budgets,
    pub rewrite_limits: RewriteLimits,
}

impl FrozenPolicy {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema == "translation-execution-policy-v1" && self.version > 0, "unsupported frozen policy");
        ensure!(
            !self.scope_id.is_empty()
                && !self.claim.id.is_empty()
                && !self.claim.revision.is_empty()
                && !self.claim.coordinator.is_empty()
                && self.claim.attempt > 0,
            "policy requires exact scope and claim lineage"
        );
        ensure!(
            self.baseline_commit.len() == 40 && self.baseline_commit.bytes().all(|b| b.is_ascii_hexdigit()),
            "policy baseline must be an exact Git commit"
        );
        ensure!(
            [&self.pinned_c_sha256, &self.dependencies_sha256, &self.verification_sha256, &self.adapter_sha256]
                .iter()
                .all(|value| digest_valid(value)),
            "policy identities require SHA-256 seals"
        );
        ensure!(
            !self.approved_files.is_empty()
                && self.approved_files.iter().all(|(path, hash)| relative_file(path) && digest_valid(hash)),
            "policy requires exact sealed writable files"
        );
        ensure!(!self.verification_stages.is_empty(), "policy requires prescribed verification stages");
        for (index, stage) in self.verification_stages.iter().enumerate() {
            ensure!(!self.verification_stages[..index].contains(stage), "duplicate prescribed verification stage");
        }
        ensure!(
            (1..=64).contains(&self.budgets.workers)
                && self.budgets.max_attempts > 0
                && self.budgets.max_evidence_bytes > 0
                && self.budgets.gate_timeout_seconds > 0,
            "policy budgets must be positive and workers bounded"
        );
        let limits = self.rewrite_limits;
        ensure!(
            limits.max_context_bytes > 0
                && limits.max_response_bytes > 0
                && limits.max_file_bytes > 0
                && limits.max_total_bytes >= limits.max_file_bytes,
            "invalid rewrite limits"
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    pub path: String,
    pub sha256: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CapsuleBinding {
    pub run_id: String,
    pub attempt: u32,
    pub policy_sha256: String,
    pub source_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RewriteCapsule {
    pub id: String,
    pub run_id: String,
    pub attempt: u32,
    pub policy_sha256: String,
    pub source_sha256: String,
    pub files: Vec<SourceFile>,
    pub context: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Replacement {
    pub path: String,
    pub base_sha256: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceReplacements {
    pub capsule_id: String,
    pub files: Vec<Replacement>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceApplied {
    pub files_sha256: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceSnapshot {
    pub commit: String,
    pub sha256: String,
    pub files_sha256: BTreeMap<String, String>,
    pub clean: bool,
}

/// An existing submitted revision admitted by the trusted installation. Its
/// admission seal identifies the original claim/source evidence, not a model
/// response. The adapter authenticates both fields on every history replay.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImportedSubmission {
    pub source: SourceSnapshot,
    pub admission_sha256: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionPhase {
    Preparation,
    Candidate,
    Integration,
    Finalization,
    Publication,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OperationContext {
    pub run_id: String,
    pub attempt: u32,
    pub policy_sha256: String,
    pub operation_key: String,
    pub phase: ExecutionPhase,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Preparation {
    pub source: SourceSnapshot,
    pub capsule: RewriteCapsule,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OutcomeSeal {
    pub operation_key: String,
    pub policy_sha256: String,
    pub source_sha256: String,
    pub evidence_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GateOutcome {
    pub seal: OutcomeSeal,
    pub passed: bool,
    pub diagnostic: String,
    pub evidence_bytes: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Disposition {
    Complete,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FinalizationRequest {
    pub source: Option<SourceSnapshot>,
    pub gates: Vec<GateOutcome>,
    pub disposition: Disposition,
    /// Cancellation must reconcile and stop this exact operation's writers
    /// before releasing any selected source or evidence resources.
    pub interrupted_operation: Option<OperationContext>,
    pub remaining_evidence_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Finalization {
    pub seal: OutcomeSeal,
    pub cleanup_complete: bool,
    pub retained_authenticatable: bool,
    pub worker_retired: bool,
    pub retained_bytes: u64,
    pub reclaimed_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Completion {
    pub run_id: String,
    pub attempt: u32,
    pub policy_sha256: String,
    pub candidate: SourceSnapshot,
    pub integrated: SourceSnapshot,
    pub gates: Vec<GateOutcome>,
    pub finalization: Finalization,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    pub seal: OutcomeSeal,
    pub observation_ids: Vec<String>,
    pub retained_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
pub enum State {
    Preparing,
    Rewrite { capsule: RewriteCapsule },
    Applying,
    Verifying { phase: ExecutionPhase, stage: VerificationStage },
    Integrating,
    Finalizing { disposition: Disposition },
    Publishing,
    Complete { completion: Box<Completion>, publication: Publication },
    Stopped { disposition: Disposition, reason: String, finalization: Option<Finalization> },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct View {
    pub run_id: String,
    pub attempt: u32,
    pub policy_sha256: String,
    pub state: State,
}
