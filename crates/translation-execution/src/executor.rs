use crate::{journal::Journal, model::*};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

/// A trusted adapter could not establish whether its recorded operation has
/// finished. Resume must reconcile that intent instead of launching new work.
#[derive(Debug)]
pub struct ExecutionPending {
    pub reason: String,
}

impl std::fmt::Display for ExecutionPending {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(formatter)
    }
}
impl std::error::Error for ExecutionPending {}

/// Trusted execution seam. Each side-effecting callback must reconcile and
/// return its original result when called again with the same operation key.
/// Implementations own real authority, restricted execution and authenticators;
/// a hash-shaped outcome alone is not evidence authentication.
pub trait ExecutionAdapter {
    fn identity(&self) -> String;
    fn prepare(
        &mut self,
        context: &OperationContext,
        policy: &FrozenPolicy,
        source: Option<&SourceSnapshot>,
    ) -> Result<Preparation>;
    fn apply(
        &mut self,
        context: &OperationContext,
        policy: &FrozenPolicy,
        capsule: &RewriteCapsule,
        replacements: &SourceReplacements,
    ) -> Result<SourceSnapshot>;
    fn verify(
        &mut self,
        context: &OperationContext,
        policy: &FrozenPolicy,
        source: &SourceSnapshot,
        stage: VerificationStage,
    ) -> Result<GateOutcome>;
    fn integrate(
        &mut self,
        context: &OperationContext,
        policy: &FrozenPolicy,
        candidate: &SourceSnapshot,
    ) -> Result<SourceSnapshot>;
    fn finalize(
        &mut self,
        context: &OperationContext,
        policy: &FrozenPolicy,
        request: &FinalizationRequest,
    ) -> Result<Finalization>;
    fn publish(
        &mut self,
        context: &OperationContext,
        policy: &FrozenPolicy,
        completion: &Completion,
    ) -> Result<Publication>;
    /// Read-only retained-evidence authentication on an independent trust
    /// transition. This must never execute a producer or allocate new evidence.
    fn authenticate(
        &self,
        policy: &FrozenPolicy,
        request: &FinalizationRequest,
        finalization: &Finalization,
    ) -> Result<()>;
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Cursor {
    Prepare,
    Rewrite,
    Apply,
    CandidateGate(usize),
    Integrate,
    IntegrationGate(usize),
    Finalize,
    Publish,
    Terminal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Attempt {
    number: u32,
    cursor: Cursor,
    /// Stable side-effect intent retained until its result is committed.
    intent: Option<OperationContext>,
    prior_source: Option<SourceSnapshot>,
    preparation: Option<Preparation>,
    replacements: Option<SourceReplacements>,
    replacements_sha256: Option<String>,
    candidate: Option<SourceSnapshot>,
    integrated: Option<SourceSnapshot>,
    gates: Vec<GateOutcome>,
    disposition: Disposition,
    reason: String,
    finalization: Option<Finalization>,
    publication: Option<Publication>,
    interrupted_operation: Option<OperationContext>,
    prior_retained_bytes: u64,
}

impl Attempt {
    fn new(number: u32) -> Self {
        Self {
            number,
            cursor: Cursor::Prepare,
            intent: None,
            prior_source: None,
            preparation: None,
            replacements: None,
            replacements_sha256: None,
            candidate: None,
            integrated: None,
            gates: Vec::new(),
            disposition: Disposition::Complete,
            reason: String::new(),
            finalization: None,
            publication: None,
            interrupted_operation: None,
            prior_retained_bytes: 0,
        }
    }
    fn source(&self) -> Option<&SourceSnapshot> {
        self.integrated
            .as_ref()
            .or(self.candidate.as_ref())
            .or_else(|| self.preparation.as_ref().map(|prepared| &prepared.source))
            .or(self.prior_source.as_ref())
    }
}

/// Small caller interface over the complete deterministic lifecycle. Operator
/// repair/retry/cancel are never exposed to the rewrite worker transport.
pub struct Supervisor<A> {
    journal: Journal,
    policy: FrozenPolicy,
    adapter: A,
}

impl<A: ExecutionAdapter> Supervisor<A> {
    pub fn open(root: &Path, policy: FrozenPolicy, adapter: A) -> Result<Self> {
        policy.validate()?;
        ensure!(adapter.identity() == policy.adapter_sha256, "installed execution adapter differs from frozen policy");
        let journal = Journal::open(root, &policy)?;
        let _lock = journal.lock()?;
        if journal.load::<Attempt>()?.is_none() {
            journal.append(&Attempt::new(1))?;
        }
        drop(_lock);
        Ok(Self { journal, policy, adapter })
    }

    pub fn inspect(&self) -> Result<View> {
        let _lock = self.journal.lock()?;
        self.view(&self.load()?)
    }

    pub fn advance(&mut self) -> Result<View> {
        let _lock = self.journal.lock()?;
        let mut attempt = self.load()?;
        self.drive(&mut attempt)
    }

    /// The only worker-controlled input. An identical lost-response submission
    /// resumes the original evaluation even after its capsule has been consumed.
    pub fn submit(&mut self, replacements: SourceReplacements) -> Result<View> {
        let _lock = self.journal.lock()?;
        let mut attempt = self.load()?;
        let request_hash = sealed(&replacements)?;
        if let Some(previous) = &attempt.replacements_sha256 {
            ensure!(*previous == request_hash, "rewrite capsule has already received different replacements");
            return self.drive(&mut attempt);
        }
        ensure!(attempt.cursor == Cursor::Rewrite, "source submission requires the current rewrite capsule");
        let prepared = attempt.preparation.as_ref().context("rewrite preparation missing")?;
        let bytes = serde_json::to_vec(&replacements)?;
        ensure!(
            bytes.len() as u64 <= self.policy.rewrite_limits.max_response_bytes,
            "source response exceeds frozen byte budget"
        );
        let checked = crate::rewrite::decode(&prepared.capsule, &bytes, self.policy.rewrite_limits)?;
        ensure!(checked == replacements, "replacement decoding changed submission");
        ensure!(
            replacements.files.iter().any(|file| sha256(file.content.as_bytes()) != file.base_sha256),
            "unchanged source requires an explicit operator retry"
        );
        attempt.replacements = Some(replacements);
        attempt.replacements_sha256 = Some(request_hash);
        attempt.cursor = Cursor::Apply;
        self.record(&mut attempt)?;
        self.drive(&mut attempt)
    }

    /// Deliberately reevaluate the last candidate without asking for a rewrite.
    /// A fresh evaluation identity consumes one frozen attempt-budget slot.
    pub fn retry(&mut self, reason: &str) -> Result<View> {
        self.next_attempt(reason, false)
    }

    /// Deliberately request changed source, retaining the stopped candidate as
    /// the next capsule's starting point. Verification declarations stay frozen.
    pub fn repair(&mut self, reason: &str) -> Result<View> {
        self.next_attempt(reason, true)
    }

    fn next_attempt(&mut self, reason: &str, rewrite: bool) -> Result<View> {
        let _lock = self.journal.lock()?;
        ensure!(!reason.trim().is_empty() && reason.len() <= 4096, "operator action requires a bounded reason");
        let previous = self.load()?;
        self.authenticate_terminal(&previous)?;
        ensure!(
            previous.cursor == Cursor::Terminal && previous.disposition == Disposition::Failed,
            "retry or repair requires a finalized failed attempt"
        );
        ensure!(previous.number < self.policy.budgets.max_attempts, "frozen attempt budget exhausted");
        let mut attempt = Attempt::new(previous.number + 1);
        attempt.prior_retained_bytes = previous
            .prior_retained_bytes
            .checked_add(previous.finalization.as_ref().context("prior attempt finalization missing")?.retained_bytes)
            .context("retained evidence byte count overflow")?;
        ensure!(
            attempt.prior_retained_bytes < self.policy.budgets.max_evidence_bytes,
            "run retained-evidence budget exhausted"
        );
        attempt.reason = reason.to_owned();
        if rewrite {
            attempt.prior_source = previous.candidate.or_else(|| previous.preparation.map(|prepared| prepared.source));
        } else {
            attempt.candidate = Some(previous.candidate.context("explicit retry requires an existing candidate")?);
            attempt.cursor = Cursor::CandidateGate(0);
        }
        self.record(&mut attempt)?;
        self.drive(&mut attempt)
    }

    pub fn cancel(&mut self, reason: &str) -> Result<View> {
        let _lock = self.journal.lock()?;
        ensure!(!reason.trim().is_empty() && reason.len() <= 4096, "cancellation requires a bounded reason");
        let mut attempt = self.load()?;
        if attempt.cursor == Cursor::Terminal {
            return self.view(&attempt);
        }
        attempt.disposition = Disposition::Cancelled;
        attempt.reason = reason.to_owned();
        attempt.cursor = Cursor::Finalize;
        attempt.interrupted_operation = attempt.intent.take();
        self.record(&mut attempt)?;
        self.drive(&mut attempt)
    }

    fn load(&self) -> Result<Attempt> {
        self.journal.load()?.context("durable attempt missing")
    }
    fn record(&self, attempt: &mut Attempt) -> Result<()> {
        if self.journal.load::<Attempt>()?.as_ref() == Some(attempt) {
            return Ok(());
        }
        self.journal.append(attempt)
    }

    fn intent(&self, attempt: &mut Attempt, name: &str, phase: ExecutionPhase) -> Result<OperationContext> {
        let context = OperationContext {
            run_id: self.journal.run_id.clone(),
            attempt: attempt.number,
            policy_sha256: self.journal.policy_sha256.clone(),
            operation_key: sha256(format!("{}:{}:{name}", self.journal.run_id, attempt.number)),
            phase,
        };
        if let Some(saved) = &attempt.intent {
            ensure!(*saved == context, "durable operation intent differs from current lifecycle");
        } else {
            attempt.intent = Some(context.clone());
            self.journal.append(attempt)?;
        }
        Ok(context)
    }

    fn completed(&self, attempt: &mut Attempt, cursor: Cursor) -> Result<()> {
        attempt.intent = None;
        attempt.cursor = cursor;
        self.journal.append(attempt)
    }

    fn stop(&self, attempt: &mut Attempt, error: anyhow::Error) -> Result<()> {
        attempt.disposition = Disposition::Failed;
        attempt.reason =
            bounded(&format!("{error:#}"), self.policy.rewrite_limits.max_context_bytes.min(4096) as usize);
        self.completed(attempt, Cursor::Finalize)
    }

    fn pending(&self, attempt: &mut Attempt, error: &anyhow::Error) -> Result<bool> {
        if error.downcast_ref::<ExecutionPending>().is_some() {
            attempt.reason = bounded(&format!("{error:#}"), 4096);
            self.record(attempt)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn snapshot(&self, source: &SourceSnapshot) -> Result<()> {
        ensure!(
            source.clean
                && source.commit.len() == 40
                && source.commit.bytes().all(|b| b.is_ascii_hexdigit())
                && digest_valid(&source.sha256),
            "execution requires an exact clean frozen source"
        );
        ensure!(
            source.files_sha256.keys().eq(self.policy.approved_files.keys())
                && source.files_sha256.values().all(|hash| digest_valid(hash)),
            "source snapshot differs from approved file ownership"
        );
        Ok(())
    }

    fn check_seal(&self, context: &OperationContext, source: &str, seal: &OutcomeSeal) -> Result<()> {
        ensure!(
            seal.operation_key == context.operation_key
                && seal.policy_sha256 == context.policy_sha256
                && seal.source_sha256 == source
                && digest_valid(&seal.evidence_sha256),
            "execution outcome does not bind the frozen operation, policy and source"
        );
        Ok(())
    }

    fn completion(&self, attempt: &Attempt) -> Result<Completion> {
        Ok(Completion {
            run_id: self.journal.run_id.clone(),
            attempt: attempt.number,
            policy_sha256: self.journal.policy_sha256.clone(),
            candidate: attempt.candidate.clone().context("candidate missing")?,
            integrated: attempt.integrated.clone().context("integrated source missing")?,
            gates: attempt.gates.clone(),
            finalization: attempt.finalization.clone().context("completed cleanup missing")?,
        })
    }

    fn view(&self, attempt: &Attempt) -> Result<View> {
        self.authenticate_terminal(attempt)?;
        let state = match attempt.cursor {
            Cursor::Prepare => State::Preparing,
            Cursor::Rewrite => {
                State::Rewrite { capsule: attempt.preparation.as_ref().context("capsule missing")?.capsule.clone() }
            }
            Cursor::Apply => State::Applying,
            Cursor::CandidateGate(index) => {
                State::Verifying { phase: ExecutionPhase::Candidate, stage: self.policy.verification_stages[index] }
            }
            Cursor::Integrate => State::Integrating,
            Cursor::IntegrationGate(index) => {
                State::Verifying { phase: ExecutionPhase::Integration, stage: self.policy.verification_stages[index] }
            }
            Cursor::Finalize => State::Finalizing { disposition: attempt.disposition },
            Cursor::Publish => State::Publishing,
            Cursor::Terminal if attempt.disposition == Disposition::Complete => State::Complete {
                completion: Box::new(self.completion(attempt)?),
                publication: attempt.publication.clone().context("publication missing")?,
            },
            Cursor::Terminal => State::Stopped {
                disposition: attempt.disposition,
                reason: attempt.reason.clone(),
                finalization: attempt.finalization.clone(),
            },
        };
        Ok(View {
            run_id: self.journal.run_id.clone(),
            attempt: attempt.number,
            policy_sha256: self.journal.policy_sha256.clone(),
            state,
        })
    }

    fn authenticate_terminal(&self, attempt: &Attempt) -> Result<()> {
        if attempt.cursor == Cursor::Terminal {
            let finalization = attempt.finalization.as_ref().context("terminal result lacks finalization")?;
            let request = FinalizationRequest {
                source: attempt.source().cloned(),
                gates: attempt.gates.clone(),
                disposition: attempt.disposition,
                interrupted_operation: attempt.interrupted_operation.clone(),
                remaining_evidence_bytes: self
                    .policy
                    .budgets
                    .max_evidence_bytes
                    .checked_sub(attempt.prior_retained_bytes)
                    .context("retained evidence budget exhausted")?,
            };
            self.adapter.authenticate(&self.policy, &request, finalization)?;
        }
        Ok(())
    }

    fn drive(&mut self, attempt: &mut Attempt) -> Result<View> {
        ensure!(self.adapter.identity() == self.policy.adapter_sha256, "execution adapter changed");
        loop {
            match attempt.cursor.clone() {
                Cursor::Rewrite | Cursor::Terminal => return self.view(attempt),
                Cursor::Prepare => {
                    let context = self.intent(attempt, "prepare", ExecutionPhase::Preparation)?;
                    let result = self.adapter.prepare(&context, &self.policy, attempt.prior_source.as_ref()).and_then(
                        |prepared| {
                            self.snapshot(&prepared.source)?;
                            ensure!(
                                attempt.prior_source.is_some() || prepared.source.commit == self.policy.baseline_commit,
                                "prepared commit differs from exact approved baseline"
                            );
                            ensure!(
                                attempt.prior_source.is_some()
                                    || prepared.source.files_sha256 == self.policy.approved_files,
                                "prepared source changed from approved baseline"
                            );
                            let capsule = &prepared.capsule;
                            ensure!(
                                capsule.run_id == context.run_id
                                    && capsule.attempt == context.attempt
                                    && capsule.policy_sha256 == context.policy_sha256
                                    && capsule.source_sha256 == prepared.source.sha256,
                                "rewrite capsule differs from frozen attempt"
                            );
                            let files: BTreeMap<_, _> =
                                capsule.files.iter().map(|file| (file.path.clone(), file.sha256.clone())).collect();
                            ensure!(
                                files == prepared.source.files_sha256 && files.len() == capsule.files.len(),
                                "capsule differs from prepared source files"
                            );
                            crate::rewrite::encode(capsule, self.policy.rewrite_limits)?;
                            Ok(prepared)
                        },
                    );
                    match result {
                        Ok(prepared) => {
                            attempt.preparation = Some(prepared);
                            self.completed(attempt, Cursor::Rewrite)?;
                        }
                        Err(error) => {
                            if self.pending(attempt, &error)? {
                                return self.view(attempt);
                            }
                            self.stop(attempt, error)?;
                        }
                    }
                }
                Cursor::Apply => {
                    let context = self.intent(attempt, "apply", ExecutionPhase::Candidate)?;
                    let prepared = attempt.preparation.as_ref().context("preparation missing")?;
                    let replacements = attempt.replacements.as_ref().context("replacement intent missing")?;
                    let result = self.adapter.apply(&context, &self.policy, &prepared.capsule, replacements).and_then(
                        |source| {
                            self.snapshot(&source)?;
                            let mut expected = prepared.source.files_sha256.clone();
                            for file in &replacements.files {
                                expected.insert(file.path.clone(), sha256(file.content.as_bytes()));
                            }
                            ensure!(
                                source.files_sha256 == expected && source.sha256 != prepared.source.sha256,
                                "applied candidate differs from replacements or did not change source"
                            );
                            Ok(source)
                        },
                    );
                    match result {
                        Ok(source) => {
                            attempt.candidate = Some(source);
                            self.completed(attempt, Cursor::CandidateGate(0))?;
                        }
                        Err(error) => {
                            if self.pending(attempt, &error)? {
                                return self.view(attempt);
                            }
                            self.stop(attempt, error)?;
                        }
                    }
                }
                Cursor::CandidateGate(index) | Cursor::IntegrationGate(index) => {
                    let integration = matches!(attempt.cursor, Cursor::IntegrationGate(_));
                    let phase = if integration { ExecutionPhase::Integration } else { ExecutionPhase::Candidate };
                    let context = self.intent(
                        attempt,
                        &format!("{}-gate-{index}", if integration { "integration" } else { "candidate" }),
                        phase,
                    )?;
                    let source = if integration { attempt.integrated.as_ref() } else { attempt.candidate.as_ref() }
                        .context("gate frozen source missing")?;
                    let stage = *self.policy.verification_stages.get(index).context("gate outside frozen policy")?;
                    let result = self.adapter.verify(&context, &self.policy, source, stage).and_then(|gate| {
                        self.check_seal(&context, &source.sha256, &gate.seal)?;
                        ensure!(
                            gate.diagnostic.len() as u64 <= self.policy.rewrite_limits.max_context_bytes,
                            "gate diagnostic exceeds frozen byte budget"
                        );
                        let prior = attempt.gates.iter().try_fold(attempt.prior_retained_bytes, |bytes, item| {
                            bytes.checked_add(item.evidence_bytes).context("evidence byte count overflow")
                        })?;
                        ensure!(
                            prior
                                .checked_add(gate.evidence_bytes)
                                .is_some_and(|bytes| bytes <= self.policy.budgets.max_evidence_bytes),
                            "gate evidence exceeds frozen aggregate budget"
                        );
                        Ok(gate)
                    });
                    match result {
                        Ok(gate) => {
                            let passed = gate.passed;
                            let reason = gate.diagnostic.clone();
                            attempt.gates.push(gate);
                            if !passed {
                                attempt.disposition = Disposition::Failed;
                                attempt.reason = if reason.is_empty() {
                                    "prescribed verification gate failed".into()
                                } else {
                                    reason
                                };
                                self.completed(attempt, Cursor::Finalize)?;
                            } else if index + 1 < self.policy.verification_stages.len() {
                                self.completed(
                                    attempt,
                                    if integration {
                                        Cursor::IntegrationGate(index + 1)
                                    } else {
                                        Cursor::CandidateGate(index + 1)
                                    },
                                )?;
                            } else {
                                self.completed(
                                    attempt,
                                    if integration { Cursor::Finalize } else { Cursor::Integrate },
                                )?;
                            }
                        }
                        Err(error) => {
                            if self.pending(attempt, &error)? {
                                return self.view(attempt);
                            }
                            self.stop(attempt, error)?;
                        }
                    }
                }
                Cursor::Integrate => {
                    let context = self.intent(attempt, "integrate", ExecutionPhase::Integration)?;
                    let candidate = attempt.candidate.as_ref().context("submitted candidate missing")?;
                    let result = self.adapter.integrate(&context, &self.policy, candidate).and_then(|source| {
                        self.snapshot(&source)?;
                        ensure!(
                            source.files_sha256 == candidate.files_sha256,
                            "integration changed submitted owned source"
                        );
                        Ok(source)
                    });
                    match result {
                        Ok(source) => {
                            attempt.integrated = Some(source);
                            self.completed(attempt, Cursor::IntegrationGate(0))?;
                        }
                        Err(error) => {
                            if self.pending(attempt, &error)? {
                                return self.view(attempt);
                            }
                            self.stop(attempt, error)?;
                        }
                    }
                }
                Cursor::Finalize => {
                    let context = self.intent(
                        attempt,
                        &format!("finalize-{:?}", attempt.disposition),
                        ExecutionPhase::Finalization,
                    )?;
                    let request = FinalizationRequest {
                        source: attempt.source().cloned(),
                        gates: attempt.gates.clone(),
                        disposition: attempt.disposition,
                        interrupted_operation: attempt.interrupted_operation.clone(),
                        remaining_evidence_bytes: self
                            .policy
                            .budgets
                            .max_evidence_bytes
                            .checked_sub(attempt.prior_retained_bytes)
                            .context("retained evidence budget exhausted")?,
                    };
                    let source = request
                        .source
                        .as_ref()
                        .map(|source| source.sha256.as_str())
                        .unwrap_or(&self.journal.policy_sha256);
                    let result = self.adapter.finalize(&context, &self.policy, &request).and_then(|outcome| {
                        self.check_seal(&context, source, &outcome.seal)?;
                        ensure!(outcome.cleanup_complete && outcome.retained_authenticatable && outcome.worker_retired,
                            "finalization requires finished cleanup, retired worker and authenticatable retained evidence");
                        ensure!(outcome.retained_bytes <= request.remaining_evidence_bytes, "retained evidence exceeds frozen budget");
                        Ok(outcome)
                    });
                    match result {
                        Ok(outcome) => {
                            attempt.finalization = Some(outcome);
                            self.completed(
                                attempt,
                                if attempt.disposition == Disposition::Complete {
                                    Cursor::Publish
                                } else {
                                    Cursor::Terminal
                                },
                            )?;
                        }
                        Err(error) => {
                            // Cleanup is resumable, but failed gates never reenter
                            // evaluation. Retain the same cleanup operation key.
                            attempt.reason = bounded(&format!("finalization incomplete: {error:#}"), 4096);
                            self.record(attempt)?;
                            return self.view(attempt);
                        }
                    }
                }
                Cursor::Publish => {
                    let context = self.intent(attempt, "publish", ExecutionPhase::Publication)?;
                    let completion = self.completion(attempt)?;
                    match self.adapter.publish(&context, &self.policy, &completion).and_then(|publication| {
                        self.check_seal(&context, &completion.integrated.sha256, &publication.seal)?;
                        ensure!(
                            !publication.observation_ids.is_empty()
                                && publication.observation_ids.iter().all(|id| !id.is_empty()),
                            "publication requires achieved-stage observations"
                        );
                        Ok(publication)
                    }) {
                        Ok(publication) => {
                            attempt.publication = Some(publication);
                            self.completed(attempt, Cursor::Terminal)?;
                        }
                        Err(error) => {
                            attempt.reason = bounded(&format!("publication incomplete: {error:#}"), 4096);
                            self.record(attempt)?;
                            return self.view(attempt);
                        }
                    }
                }
            }
        }
    }
}

fn bounded(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}
