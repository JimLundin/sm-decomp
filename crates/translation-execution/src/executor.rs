use crate::{journal::Journal, model::*};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

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
    /// Authenticate a journaled gate against its original frozen source and
    /// prescribed stage before any recovery can launch another producer.
    /// Historical checkouts may be retired; the adapter follows their retained
    /// source and evidence rather than reevaluating or treating tamper as a miss.
    fn authenticate_gate(
        &self,
        policy: &FrozenPolicy,
        source: &SourceSnapshot,
        stage: VerificationStage,
        outcome: &GateOutcome,
    ) -> Result<()>;
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
        journal: &FinalizationJournal<'_>,
    ) -> Result<Finalization>;
    fn publish(
        &mut self,
        context: &OperationContext,
        policy: &FrozenPolicy,
        completion: &Completion,
        journal: &PublicationJournal<'_>,
    ) -> Result<Publication>;
    /// Read-only retained-evidence authentication on an independent trust
    /// transition. This must never execute a producer or allocate new evidence.
    fn authenticate(
        &self,
        policy: &FrozenPolicy,
        request: &FinalizationRequest,
        finalization: &Finalization,
    ) -> Result<()>;
    /// Authenticate the accepted publication and its projections independently
    /// of the journal. No verification producer may run at this transition.
    fn authenticate_publication(
        &self,
        policy: &FrozenPolicy,
        completion: &Completion,
        publication: &Publication,
    ) -> Result<()>;
    /// Restore missing projections only from an authenticated accepted record.
    /// An inspection remains read-only; an explicit resume can repair a lost
    /// projection without reevaluating source or allocating another capture.
    fn resume_publication(
        &mut self,
        policy: &FrozenPolicy,
        completion: &Completion,
        publication: &Publication,
    ) -> Result<()> {
        self.authenticate_publication(policy, completion, publication)
    }
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

fn operation_context(journal: &Journal, attempt: &Attempt) -> Result<Option<OperationContext>> {
    let (name, phase) = match attempt.cursor {
        Cursor::Prepare => ("prepare".into(), ExecutionPhase::Preparation),
        Cursor::Rewrite => ("rewrite".into(), ExecutionPhase::Candidate),
        Cursor::Apply => ("apply".into(), ExecutionPhase::Candidate),
        Cursor::CandidateGate(index) => (format!("candidate-gate-{index}"), ExecutionPhase::Candidate),
        Cursor::Integrate => ("integrate".into(), ExecutionPhase::Integration),
        Cursor::IntegrationGate(index) => (format!("integration-gate-{index}"), ExecutionPhase::Integration),
        Cursor::Finalize => (format!("finalize-{:?}", attempt.disposition), ExecutionPhase::Finalization),
        Cursor::Publish => ("publish".into(), ExecutionPhase::Publication),
        Cursor::Terminal => return Ok(None),
    };
    Ok(Some(OperationContext {
        run_id: journal.run_id.clone(),
        attempt: attempt.number,
        policy_sha256: journal.policy_sha256.clone(),
        operation_key: sha256(format!("{}:{}:{name}", journal.run_id, attempt.number)),
        phase,
    }))
}

fn checked_snapshot(policy: &FrozenPolicy, source: &SourceSnapshot) -> Result<()> {
    ensure!(
        source.clean
            && source.commit.len() == 40
            && source.commit.bytes().all(|b| b.is_ascii_hexdigit())
            && digest_valid(&source.sha256),
        "execution requires an exact clean frozen source"
    );
    ensure!(
        source.files_sha256.keys().eq(policy.approved_files.keys())
            && source.files_sha256.values().all(|hash| digest_valid(hash)),
        "source snapshot differs from approved file ownership"
    );
    Ok(())
}

fn checked_seal(context: &OperationContext, source: &str, seal: &OutcomeSeal) -> Result<()> {
    ensure!(
        seal.operation_key == context.operation_key
            && seal.policy_sha256 == context.policy_sha256
            && seal.source_sha256 == source
            && digest_valid(&seal.evidence_sha256),
        "execution outcome does not bind the frozen operation, policy and source"
    );
    Ok(())
}

fn finalization_request(policy: &FrozenPolicy, attempt: &Attempt) -> Result<FinalizationRequest> {
    Ok(FinalizationRequest {
        source: attempt.source().cloned(),
        gates: attempt.gates.clone(),
        disposition: attempt.disposition,
        interrupted_operation: attempt.interrupted_operation.clone(),
        remaining_evidence_bytes: policy
            .budgets
            .max_evidence_bytes
            .checked_sub(attempt.prior_retained_bytes)
            .context("retained evidence budget exhausted")?,
    })
}

/// Snapshot fields are immutable except for the one result/intent admitted by
/// this transition. A predecessor hash alone cannot authenticate an editable
/// last event; replaying the lifecycle also prevents a terminal cursor from
/// being changed into permission to run an unexecuted stage.
fn checked_transition(
    journal: &Journal,
    policy: &FrozenPolicy,
    previous: Option<&Attempt>,
    next: &Attempt,
) -> Result<()> {
    ensure!((1..=policy.budgets.max_attempts).contains(&next.number), "journal attempt exceeds frozen budget");
    ensure!(
        next.reason.len() as u64 <= policy.rewrite_limits.max_context_bytes.max(4096),
        "journal diagnostic exceeds frozen bound"
    );
    ensure!(next.gates.len() <= policy.verification_stages.len() * 2, "journal contains extra gates");
    if let Cursor::CandidateGate(index) | Cursor::IntegrationGate(index) = next.cursor {
        ensure!(
            index < policy.verification_stages.len() && next.disposition == Disposition::Complete,
            "stopped attempt cannot authorize another gate"
        );
    }
    if let Some(intent) = &next.intent {
        ensure!(
            Some(intent.clone()) == operation_context(journal, next)?,
            "journal operation intent differs from lifecycle"
        );
    }
    let Some(previous) = previous else {
        ensure!(*next == Attempt::new(1), "journal does not start with an untouched first attempt");
        return Ok(());
    };
    if next.number != previous.number {
        ensure!(
            next.number == previous.number.checked_add(1).context("attempt number overflow")?
                && previous.cursor == Cursor::Terminal
                && previous.disposition == Disposition::Failed,
            "new evaluation requires a finalized failed predecessor"
        );
        ensure!(!next.reason.trim().is_empty() && next.reason.len() <= 4096, "new evaluation lacks an operator reason");
        let mut expected = Attempt::new(next.number);
        expected.reason = next.reason.clone();
        expected.prior_retained_bytes = previous
            .prior_retained_bytes
            .checked_add(previous.finalization.as_ref().context("prior finalization missing")?.retained_bytes)
            .context("historical retained byte overflow")?;
        ensure!(
            expected.prior_retained_bytes < policy.budgets.max_evidence_bytes,
            "new evaluation exhausted retained budget"
        );
        match next.cursor {
            Cursor::Prepare => {
                expected.prior_source = previous
                    .candidate
                    .clone()
                    .or_else(|| previous.preparation.as_ref().map(|prepared| prepared.source.clone()))
            }
            Cursor::CandidateGate(0) => {
                expected.candidate = Some(previous.candidate.clone().context("retry requires a candidate")?);
                expected.cursor = Cursor::CandidateGate(0);
            }
            _ => anyhow::bail!("new evaluation skipped its approved starting state"),
        }
        ensure!(expected == *next, "new evaluation changed original lineage or source");
        return Ok(());
    }
    if previous == next {
        return Ok(());
    }
    let mut expected = previous.clone();
    if next.cursor == previous.cursor && next.intent == previous.intent {
        ensure!(previous.intent.is_some(), "idle journal state changed without an operation");
        ensure!(
            previous.cursor != Cursor::Finalize
                || previous.disposition == Disposition::Complete
                || previous.reason == next.reason,
            "finalization changed the original stopped diagnosis"
        );
        ensure!(
            previous.cursor != Cursor::Publish || previous.reason == next.reason,
            "publication changed its original reserved intent"
        );
        expected.reason = next.reason.clone();
        ensure!(expected == *next && next.reason.len() <= 4096, "pending operation changed accepted state");
        return Ok(());
    }
    if previous.intent.is_none() && next.intent.is_some() {
        expected.intent = operation_context(journal, previous)?;
        ensure!(expected == *next, "recording an intent changed accepted state");
        return Ok(());
    }
    if next.cursor == Cursor::Finalize && next.disposition == Disposition::Cancelled {
        ensure!(
            !matches!(previous.cursor, Cursor::Finalize | Cursor::Publish | Cursor::Terminal)
                && !next.reason.trim().is_empty()
                && next.reason.len() <= 4096,
            "cancellation changed an already selected finalization or omitted its reason"
        );
        expected.cursor = Cursor::Finalize;
        expected.disposition = Disposition::Cancelled;
        expected.reason = next.reason.clone();
        expected.interrupted_operation = expected.intent.take();
        ensure!(expected == *next, "cancellation changed accepted source or evidence");
        return Ok(());
    }
    if next.cursor == Cursor::Finalize && next.disposition == Disposition::Failed && next.gates == previous.gates {
        ensure!(
            matches!(
                previous.cursor,
                Cursor::Prepare
                    | Cursor::Rewrite
                    | Cursor::Apply
                    | Cursor::CandidateGate(_)
                    | Cursor::Integrate
                    | Cursor::IntegrationGate(_)
            ),
            "failure cannot reopen finalization or a terminal result"
        );
        ensure!(
            previous.intent.is_some() || previous.cursor == Cursor::Rewrite,
            "failure lacks its original operation"
        );
        expected.cursor = Cursor::Finalize;
        expected.disposition = Disposition::Failed;
        expected.reason = next.reason.clone();
        if previous.cursor == Cursor::Rewrite {
            expected.interrupted_operation = previous.intent.clone();
        }
        expected.intent = None;
        ensure!(expected == *next, "operation failure changed accepted source or evidence");
        return Ok(());
    }
    if previous.cursor == Cursor::Rewrite && next.cursor == Cursor::Apply {
        let prepared = previous.preparation.as_ref().context("rewrite preparation missing")?;
        let replacements = next.replacements.as_ref().context("replacement result missing")?;
        let bytes = serde_json::to_vec(replacements)?;
        ensure!(
            crate::rewrite::decode(&prepared.capsule, &bytes, policy.rewrite_limits)? == *replacements
                && replacements.files.iter().any(|file| sha256(file.content.as_bytes()) != file.base_sha256),
            "journal rewrite result differs from its source capsule"
        );
        expected.replacements = Some(replacements.clone());
        expected.replacements_sha256 = Some(sealed(replacements)?);
        expected.intent = None;
        expected.cursor = Cursor::Apply;
        ensure!(expected == *next, "rewrite result changed accepted state");
        return Ok(());
    }
    let context = previous.intent.as_ref().context("journal completion lacks its original intent")?;
    expected.intent = None;
    match previous.cursor {
        Cursor::Prepare => {
            let prepared = next.preparation.as_ref().context("preparation result missing")?;
            checked_snapshot(policy, &prepared.source)?;
            ensure!(
                previous.prior_source.as_ref().is_none_or(|source| *source == prepared.source),
                "repair preparation changed stopped source"
            );
            ensure!(
                previous.prior_source.is_some()
                    || (prepared.source.commit == policy.baseline_commit
                        && prepared.source.files_sha256 == policy.approved_files),
                "preparation changed approved baseline"
            );
            let capsule = &prepared.capsule;
            ensure!(
                capsule.run_id == journal.run_id
                    && capsule.attempt == next.number
                    && capsule.policy_sha256 == journal.policy_sha256
                    && capsule.source_sha256 == prepared.source.sha256,
                "preparation capsule identity differs"
            );
            ensure!(
                capsule.files.iter().map(|file| (file.path.clone(), file.sha256.clone())).collect::<BTreeMap<_, _>>()
                    == prepared.source.files_sha256,
                "capsule differs from prepared files"
            );
            crate::rewrite::encode(capsule, policy.rewrite_limits)?;
            expected.preparation = Some(prepared.clone());
            expected.cursor = Cursor::Rewrite;
        }
        Cursor::Apply => {
            let candidate = next.candidate.as_ref().context("candidate result missing")?;
            checked_snapshot(policy, candidate)?;
            let prepared = previous.preparation.as_ref().context("candidate preparation missing")?;
            let mut files = prepared.source.files_sha256.clone();
            for replacement in &previous.replacements.as_ref().context("candidate replacements missing")?.files {
                files.insert(replacement.path.clone(), sha256(replacement.content.as_bytes()));
            }
            ensure!(
                candidate.files_sha256 == files && candidate.sha256 != prepared.source.sha256,
                "candidate differs from accepted replacements"
            );
            expected.candidate = Some(candidate.clone());
            expected.cursor = Cursor::CandidateGate(0);
        }
        Cursor::CandidateGate(index) | Cursor::IntegrationGate(index) => {
            let integration = matches!(previous.cursor, Cursor::IntegrationGate(_));
            let source = if integration { previous.integrated.as_ref() } else { previous.candidate.as_ref() }
                .context("journal gate source missing")?;
            let gate = next.gates.last().context("journal gate result missing")?;
            checked_seal(context, &source.sha256, &gate.seal)?;
            ensure!(
                gate.diagnostic.len() as u64 <= policy.rewrite_limits.max_context_bytes,
                "journal gate diagnostic exceeds frozen extent"
            );
            let bytes = previous.gates.iter().try_fold(previous.prior_retained_bytes, |bytes, gate| {
                bytes.checked_add(gate.evidence_bytes).context("historical gate byte overflow")
            })?;
            ensure!(
                bytes.checked_add(gate.evidence_bytes).is_some_and(|bytes| bytes <= policy.budgets.max_evidence_bytes),
                "journal gate exceeds aggregate evidence budget"
            );
            expected.gates.push(gate.clone());
            expected.cursor = if !gate.passed {
                expected.disposition = Disposition::Failed;
                expected.reason = if gate.diagnostic.is_empty() {
                    "prescribed verification gate failed".into()
                } else {
                    gate.diagnostic.clone()
                };
                Cursor::Finalize
            } else if index + 1 < policy.verification_stages.len() {
                if integration { Cursor::IntegrationGate(index + 1) } else { Cursor::CandidateGate(index + 1) }
            } else if integration {
                Cursor::Finalize
            } else {
                Cursor::Integrate
            };
        }
        Cursor::Integrate => {
            let integrated = next.integrated.as_ref().context("integration result missing")?;
            checked_snapshot(policy, integrated)?;
            ensure!(
                integrated.files_sha256
                    == previous.candidate.as_ref().context("integration candidate missing")?.files_sha256,
                "integration changed submitted owned source"
            );
            expected.integrated = Some(integrated.clone());
            expected.cursor = Cursor::IntegrationGate(0);
        }
        Cursor::Finalize => {
            let finalization = next.finalization.as_ref().context("finalization result missing")?;
            let request = finalization_request(policy, previous)?;
            checked_seal(
                context,
                request.source.as_ref().map(|source| source.sha256.as_str()).unwrap_or(&journal.policy_sha256),
                &finalization.seal,
            )?;
            ensure!(
                finalization.cleanup_complete
                    && finalization.retained_authenticatable
                    && finalization.worker_retired
                    && finalization.retained_bytes <= request.remaining_evidence_bytes,
                "journal finalization is incomplete"
            );
            expected.finalization = Some(finalization.clone());
            expected.cursor =
                if previous.disposition == Disposition::Complete { Cursor::Publish } else { Cursor::Terminal };
        }
        Cursor::Publish => {
            let publication = next.publication.as_ref().context("publication result missing")?;
            checked_seal(
                context,
                &previous.integrated.as_ref().context("publication source missing")?.sha256,
                &publication.seal,
            )?;
            ensure!(
                !publication.observation_ids.is_empty() && publication.observation_ids.iter().all(|id| !id.is_empty()),
                "journal publication has no achieved observations"
            );
            ensure!(
                previous
                    .prior_retained_bytes
                    .checked_add(
                        previous.finalization.as_ref().context("publication finalization missing")?.retained_bytes
                    )
                    .and_then(|bytes| bytes.checked_add(publication.retained_bytes))
                    .is_some_and(|bytes| bytes <= policy.budgets.max_evidence_bytes),
                "journal publication exceeds retained budget"
            );
            expected.publication = Some(publication.clone());
            expected.cursor = Cursor::Terminal;
        }
        Cursor::Rewrite | Cursor::Terminal => anyhow::bail!("journal reopened an idle or terminal result"),
    }
    ensure!(expected == *next, "journal completion changed unrelated accepted state");
    Ok(())
}

fn replay_history(
    journal: &Journal,
    policy: &FrozenPolicy,
    mut accepted: impl FnMut(&Attempt) -> Result<()>,
) -> Result<Option<Attempt>> {
    let mut previous = None;
    journal.visit_states::<Attempt>(|sequence, next| {
        checked_transition(journal, policy, previous.as_ref(), &next)
            .with_context(|| format!("invalid lifecycle at journal event {sequence}"))?;
        accepted(&next)?;
        previous = Some(next);
        Ok(())
    })?;
    Ok(previous)
}

fn authenticated_history<A: ExecutionAdapter>(
    journal: &Journal,
    policy: &FrozenPolicy,
    adapter: &A,
) -> Result<Option<Attempt>> {
    let mut gates = BTreeSet::new();
    let mut finalizations = BTreeSet::new();
    replay_history(journal, policy, |attempt| {
        for (index, gate) in attempt.gates.iter().enumerate() {
            if gates.insert(gate.seal.operation_key.clone()) {
                let count = policy.verification_stages.len();
                let source = if index < count { attempt.candidate.as_ref() } else { attempt.integrated.as_ref() }
                    .context("historical gate source missing")?;
                adapter.authenticate_gate(policy, source, policy.verification_stages[index % count], gate)?;
            }
        }
        if let Some(finalization) = &attempt.finalization
            && finalizations.insert(finalization.seal.operation_key.clone())
        {
            adapter.authenticate(policy, &finalization_request(policy, attempt)?, finalization)?;
        }
        Ok(())
    })
}

/// Small caller interface over the complete deterministic lifecycle. Operator
/// repair/retry/cancel are never exposed to the rewrite worker transport.
pub struct Supervisor<A> {
    journal: Journal,
    policy: FrozenPolicy,
    adapter: A,
}

/// Read-only finalization accounting under the current lifecycle lease. It
/// projects the exact journal result before its retained byte count is sealed.
pub struct FinalizationJournal<'a> {
    journal: &'a Journal,
    policy: &'a FrozenPolicy,
    attempt: &'a Attempt,
}

impl FinalizationJournal<'_> {
    pub fn current_bytes(&self) -> Result<u64> {
        self.journal.retained_bytes()
    }

    /// Exact serialized upper bound for the next result event. Hashes and
    /// booleans have their prescribed widths; both result byte counters use
    /// their maximum decimal widths. This reserves a stopped terminal suffix
    /// before retaining its immutable closure. It grants no gate or result.
    pub fn max_additional_bytes(&self) -> Result<u64> {
        let context = self.attempt.intent.as_ref().context("finalization accounting lacks its original intent")?;
        ensure!(
            self.attempt.cursor == Cursor::Finalize
                && operation_context(self.journal, self.attempt)?.as_ref() == Some(context),
            "finalization accounting differs from its original operation"
        );
        let source = self.attempt.source().map(|source| source.sha256.as_str()).unwrap_or(&context.policy_sha256);
        let placeholder = Finalization {
            seal: OutcomeSeal {
                operation_key: context.operation_key.clone(),
                policy_sha256: context.policy_sha256.clone(),
                source_sha256: source.into(),
                evidence_sha256: "0".repeat(64),
            },
            cleanup_complete: true,
            retained_authenticatable: true,
            worker_retired: true,
            retained_bytes: u64::MAX,
            reclaimed_bytes: u64::MAX,
        };
        self.journal.projected_append_bytes(&finalization_successor(self.attempt, &placeholder))
    }

    /// Bytes of the prescribed next result event: Terminal for a stopped
    /// attempt, or Publish for a successfully finalized attempt.
    pub fn additional_bytes(&self, finalization: &Finalization) -> Result<u64> {
        let result = finalized_attempt(self.journal, self.policy, self.attempt, finalization)?;
        self.journal.projected_append_bytes(&result)
    }

    pub fn retained_bytes(&self, finalization: &Finalization) -> Result<u64> {
        self.current_bytes()?
            .checked_add(self.additional_bytes(finalization)?)
            .context("finalization journal byte overflow")
    }
}

fn finalized_attempt(
    journal: &Journal,
    policy: &FrozenPolicy,
    attempt: &Attempt,
    finalization: &Finalization,
) -> Result<Attempt> {
    ensure!(attempt.cursor == Cursor::Finalize, "finalization accounting requires the pending finalization");
    let result = finalization_successor(attempt, finalization);
    checked_transition(journal, policy, Some(attempt), &result)?;
    Ok(result)
}

fn finalization_successor(attempt: &Attempt, finalization: &Finalization) -> Attempt {
    let mut result = attempt.clone();
    result.finalization = Some(finalization.clone());
    result.intent = None;
    result.cursor = if result.disposition == Disposition::Complete { Cursor::Publish } else { Cursor::Terminal };
    result
}

/// Authenticate the exact stopped finalization tail without acquiring a second
/// journal lease or calling an adapter. The caller holds the lifecycle lease or
/// is reading retained immutable state. A result sealed immediately before a
/// lost response may still be pending at the journal head; its exact prescribed
/// next event is projected from that authenticated predecessor. Later attempts
/// retain their own exact historical terminal event and never use today's size.
pub fn authenticate_finalization_tail(
    root: &Path,
    policy: &FrozenPolicy,
    context: &OperationContext,
    request: &FinalizationRequest,
    finalization: &Finalization,
) -> Result<u64> {
    policy.validate()?;
    ensure!(request.disposition != Disposition::Complete, "successful finalization tail belongs to publication");
    let journal = Journal::existing(root, policy)?;
    ensure!(
        context.run_id == journal.run_id && context.policy_sha256 == journal.policy_sha256,
        "finalization tail belongs to another journal"
    );
    let mut previous: Option<Attempt> = None;
    let mut pending: Option<(Attempt, u64, String)> = None;
    let mut found = None;
    journal.visit_events::<Attempt>(|sequence, next, bytes| {
        checked_transition(&journal, policy, previous.as_ref(), &next)
            .with_context(|| format!("invalid lifecycle at journal event {sequence}"))?;
        if let Some((predecessor, predecessor_sequence, predecessor_sha256)) = pending.take() {
            let expected = finalized_attempt(&journal, policy, &predecessor, finalization)?;
            ensure!(next == expected, "retained stopped finalization differs from its exact result");
            let expected_bytes = journal.event_bytes(
                predecessor_sequence.checked_add(1).context("journal sequence overflow")?,
                &predecessor_sha256,
                &expected,
            )?;
            ensure!(bytes == expected_bytes, "retained stopped finalization event encoding changed");
            ensure!(found.is_none(), "stopped finalization result occurred more than once");
            found = Some(expected_bytes.len() as u64);
        }
        if next.number == context.attempt && next.cursor == Cursor::Finalize && next.intent.as_ref() == Some(context) {
            ensure!(finalization_request(policy, &next)? == *request, "finalization tail request changed");
            ensure!(found.is_none(), "journal reopened an accepted finalization");
            pending = Some((next.clone(), sequence, sha256(bytes)));
        }
        previous = Some(next);
        Ok(())
    })?;
    if let Some((predecessor, sequence, predecessor_sha256)) = pending {
        let expected = finalized_attempt(&journal, policy, &predecessor, finalization)?;
        let bytes = journal.event_bytes(
            sequence.checked_add(1).context("journal sequence overflow")?,
            &predecessor_sha256,
            &expected,
        )?;
        ensure!(found.is_none(), "finalization tail has conflicting retained and pending results");
        found = Some(bytes.len() as u64);
    }
    found.context("exact stopped finalization predecessor is missing")
}

/// Trusted, read-only publication accounting under the existing controller
/// lease. It exposes byte counts rather than a journal path or writable state.
/// The prospective terminal event has exactly the serialization append uses.
pub struct PublicationJournal<'a> {
    journal: &'a Journal,
    policy: &'a FrozenPolicy,
    attempt: &'a Attempt,
}

impl PublicationJournal<'_> {
    /// Bytes already present at the publication intent, before its terminal
    /// result is appended. No new event or file is allocated by this query.
    pub fn current_bytes(&self) -> Result<u64> {
        self.journal.retained_bytes()
    }

    /// Additional bytes for the exact terminal append carrying this result.
    /// The adapter subtracts any frozen prefix it already counts separately
    /// from current_bytes + additional_bytes to obtain its publication delta.
    pub fn additional_bytes(&self, publication: &Publication) -> Result<u64> {
        let terminal = published_attempt(self.journal, self.policy, self.attempt, publication)?;
        self.journal.projected_append_bytes(&terminal)
    }

    /// Complete logical journal extent after the accepted terminal append.
    pub fn retained_bytes(&self, publication: &Publication) -> Result<u64> {
        self.current_bytes()?
            .checked_add(self.additional_bytes(publication)?)
            .context("publication journal byte overflow")
    }
}

fn attempt_completion(journal: &Journal, attempt: &Attempt) -> Result<Completion> {
    Ok(Completion {
        run_id: journal.run_id.clone(),
        attempt: attempt.number,
        policy_sha256: journal.policy_sha256.clone(),
        candidate: attempt.candidate.clone().context("candidate missing")?,
        integrated: attempt.integrated.clone().context("integrated source missing")?,
        gates: attempt.gates.clone(),
        finalization: attempt.finalization.clone().context("completed cleanup missing")?,
    })
}

fn published_attempt(
    journal: &Journal,
    policy: &FrozenPolicy,
    attempt: &Attempt,
    publication: &Publication,
) -> Result<Attempt> {
    ensure!(
        attempt.cursor == Cursor::Publish && attempt.disposition == Disposition::Complete,
        "publication accounting requires the current successful publication intent"
    );
    let mut terminal = attempt.clone();
    terminal.publication = Some(publication.clone());
    terminal.intent = None;
    terminal.cursor = Cursor::Terminal;
    checked_transition(journal, policy, Some(attempt), &terminal)?;
    Ok(terminal)
}

/// Authenticate the exact complete publication extent without acquiring a
/// second journal lease or calling an adapter. The caller holds the lifecycle
/// lease or reads retained immutable state. A missing terminal result is allowed
/// only after the exact current publication intent and matching completion. Its
/// prescribed event is projected using the real append serializer. An existing
/// terminal event must match that exact encoding and end the complete history.
pub fn authenticate_publication_extent(
    root: &Path,
    policy: &FrozenPolicy,
    context: &OperationContext,
    completion: &Completion,
    publication: &Publication,
) -> Result<u64> {
    policy.validate()?;
    let journal = Journal::existing(root, policy)?;
    ensure!(
        context.run_id == journal.run_id
            && context.policy_sha256 == journal.policy_sha256
            && context.phase == ExecutionPhase::Publication
            && completion.run_id == journal.run_id
            && completion.policy_sha256 == journal.policy_sha256
            && completion.attempt == context.attempt,
        "publication extent belongs to another operation or journal"
    );
    let mut previous: Option<Attempt> = None;
    let mut pending: Option<(Attempt, u64, String)> = None;
    let mut terminal = false;
    journal.visit_events::<Attempt>(|sequence, next, bytes| {
        ensure!(!terminal, "accepted publication no longer ends the current journal");
        checked_transition(&journal, policy, previous.as_ref(), &next)
            .with_context(|| format!("invalid lifecycle at journal event {sequence}"))?;
        if let Some((predecessor, predecessor_sequence, predecessor_sha256)) = pending.take() {
            let expected = published_attempt(&journal, policy, &predecessor, publication)?;
            ensure!(next == expected, "retained publication differs from its exact accepted result");
            let expected_bytes = journal.event_bytes(
                predecessor_sequence.checked_add(1).context("journal sequence overflow")?,
                &predecessor_sha256,
                &expected,
            )?;
            ensure!(bytes == expected_bytes, "retained publication terminal event encoding changed");
            terminal = true;
        }
        if next.number == context.attempt && next.cursor == Cursor::Publish && next.intent.as_ref() == Some(context) {
            ensure!(
                attempt_completion(&journal, &next)? == *completion,
                "publication completion changed from its intent"
            );
            pending = Some((next.clone(), sequence, sha256(bytes)));
        }
        previous = Some(next);
        Ok(())
    })?;
    let current_bytes = journal.retained_bytes()?;
    if let Some((predecessor, _, _)) = pending {
        let expected = published_attempt(&journal, policy, &predecessor, publication)?;
        return current_bytes
            .checked_add(journal.projected_append_bytes(&expected)?)
            .context("publication journal byte overflow");
    }
    ensure!(terminal, "exact current publication predecessor is missing");
    Ok(current_bytes)
}

/// Bind a retained accepted result to the original complete controller history.
/// This inspection creates no state and never calls an execution adapter.
pub fn authenticate_completion_history(
    root: &Path,
    policy: &FrozenPolicy,
    completion: &Completion,
    publication: &Publication,
) -> Result<()> {
    policy.validate()?;
    let (journal, _lock) = Journal::read_only(root, policy)?;
    let attempt = replay_history(&journal, policy, |_| Ok(()))?.context("retained attempt history is missing")?;
    ensure!(
        attempt.cursor == Cursor::Terminal
            && attempt.disposition == Disposition::Complete
            && attempt_completion(&journal, &attempt)? == *completion
            && attempt.publication.as_ref() == Some(publication),
        "retained completion differs from accepted journal history"
    );
    Ok(())
}

/// Authenticate every journaled gate and finalized attempt through the
/// installed adapter, including retained failures preceding a later success.
/// This opens only an existing shared lease and never executes a producer.
pub fn authenticate_finalized_history<A: ExecutionAdapter>(
    root: &Path,
    policy: &FrozenPolicy,
    adapter: &A,
) -> Result<()> {
    policy.validate()?;
    ensure!(adapter.identity() == policy.adapter_sha256, "retained execution adapter differs from frozen policy");
    let (journal, _lock) = Journal::read_only(root, policy)?;
    ensure!(authenticated_history(&journal, policy, adapter)?.is_some(), "retained attempt history is missing");
    Ok(())
}

impl<A: ExecutionAdapter> Supervisor<A> {
    pub fn open(root: &Path, policy: FrozenPolicy, adapter: A) -> Result<Self> {
        policy.validate()?;
        ensure!(adapter.identity() == policy.adapter_sha256, "installed execution adapter differs from frozen policy");
        let journal = Journal::open(root, &policy)?;
        let _lock = journal.lock()?;
        if authenticated_history(&journal, &policy, &adapter)?.is_none() {
            journal.append(&Attempt::new(1))?;
        }
        drop(_lock);
        Ok(Self { journal, policy, adapter })
    }

    pub fn inspect(&self) -> Result<View> {
        let _lock = self.journal.lock()?;
        self.view(&self.load()?)
    }

    /// Inspect an existing attempt without initializing or repairing state.
    pub fn inspect_retained(root: &Path, policy: FrozenPolicy, adapter: A) -> Result<View> {
        policy.validate()?;
        ensure!(adapter.identity() == policy.adapter_sha256, "installed execution adapter differs from frozen policy");
        let (journal, _lock) = Journal::read_only(root, &policy)?;
        let supervisor = Self { journal, policy, adapter };
        supervisor.view(&supervisor.load()?)
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
        self.accept_replacements(&mut attempt, replacements)?;
        self.drive(&mut attempt)
    }

    /// Run the installed rewrite exchange while holding the same lifecycle
    /// lease as preparation, cancellation and source application. The callback
    /// is trusted transport; its response still crosses the source-only seam.
    pub fn run_with_rewrite<F>(&mut self, rewrite: F) -> Result<View>
    where
        F: FnOnce(&mut A, &OperationContext, &RewriteCapsule) -> Result<SourceReplacements>,
    {
        let _lock = self.journal.lock()?;
        let mut attempt = self.load()?;
        let view = self.drive(&mut attempt)?;
        if attempt.cursor != Cursor::Rewrite {
            return Ok(view);
        }
        let context = self.intent(&mut attempt, "rewrite", ExecutionPhase::Candidate)?;
        let capsule = attempt.preparation.as_ref().context("rewrite preparation missing")?.capsule.clone();
        let result = rewrite(&mut self.adapter, &context, &capsule)
            .and_then(|replacements| self.accept_replacements(&mut attempt, replacements));
        if let Err(error) = result {
            if self.pending(&mut attempt, &error)? {
                return self.view(&attempt);
            }
            attempt.interrupted_operation = attempt.intent.take();
            self.stop(&mut attempt, error)?;
        }
        self.drive(&mut attempt)
    }

    fn accept_replacements(&self, attempt: &mut Attempt, replacements: SourceReplacements) -> Result<()> {
        let request_hash = sealed(&replacements)?;
        if let Some(previous) = &attempt.replacements_sha256 {
            ensure!(*previous == request_hash, "rewrite capsule has already received different replacements");
            return Ok(());
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
        attempt.intent = None;
        attempt.cursor = Cursor::Apply;
        self.record(attempt)
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

    /// Cancellation before finalization stops the current operation. Once its
    /// disposition is selected, reconcile that finalization/publication instead
    /// of changing the identity of a partially completed retention operation.
    pub fn cancel(&mut self, reason: &str) -> Result<View> {
        let _lock = self.journal.lock()?;
        ensure!(!reason.trim().is_empty() && reason.len() <= 4096, "cancellation requires a bounded reason");
        let mut attempt = self.load()?;
        if attempt.cursor == Cursor::Terminal {
            return self.view(&attempt);
        }
        if matches!(attempt.cursor, Cursor::Finalize | Cursor::Publish) {
            return self.drive(&mut attempt);
        }
        attempt.disposition = Disposition::Cancelled;
        attempt.reason = reason.to_owned();
        attempt.cursor = Cursor::Finalize;
        attempt.interrupted_operation = attempt.intent.take();
        self.record(&mut attempt)?;
        self.drive(&mut attempt)
    }

    /// The trusted invocation adapter reports a refused or failed rewrite here.
    /// This closes the current evaluation through the same bounded finalization
    /// path as a failed gate. It is not part of the worker response protocol.
    pub fn fail_rewrite(&mut self, reason: &str) -> Result<View> {
        let _lock = self.journal.lock()?;
        ensure!(!reason.trim().is_empty() && reason.len() <= 4096, "rewrite failure requires a bounded reason");
        let mut attempt = self.load()?;
        if attempt.cursor == Cursor::Terminal {
            return self.view(&attempt);
        }
        ensure!(attempt.cursor == Cursor::Rewrite, "rewrite failure requires the pending rewrite exchange");
        attempt.disposition = Disposition::Failed;
        attempt.reason = reason.to_owned();
        attempt.cursor = Cursor::Finalize;
        attempt.interrupted_operation = attempt.intent.take();
        self.record(&mut attempt)?;
        self.drive(&mut attempt)
    }

    fn load(&self) -> Result<Attempt> {
        authenticated_history(&self.journal, &self.policy, &self.adapter)?.context("durable attempt missing")
    }
    fn record(&self, attempt: &mut Attempt) -> Result<()> {
        if self.journal.load::<Attempt>()?.as_ref() == Some(attempt) {
            return Ok(());
        }
        self.append(attempt)
    }

    fn append(&self, attempt: &Attempt) -> Result<()> {
        let previous: Option<Attempt> = self.journal.load()?;
        checked_transition(&self.journal, &self.policy, previous.as_ref(), attempt)?;
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
            self.append(attempt)?;
        }
        Ok(context)
    }

    fn completed(&self, attempt: &mut Attempt, cursor: Cursor) -> Result<()> {
        attempt.intent = None;
        attempt.cursor = cursor;
        self.append(attempt)
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
        checked_snapshot(&self.policy, source)
    }

    fn check_seal(&self, context: &OperationContext, source: &str, seal: &OutcomeSeal) -> Result<()> {
        checked_seal(context, source, seal)
    }

    fn completion(&self, attempt: &Attempt) -> Result<Completion> {
        attempt_completion(&self.journal, attempt)
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
            let request = finalization_request(&self.policy, attempt)?;
            self.adapter.authenticate(&self.policy, &request, finalization)?;
            if attempt.disposition == Disposition::Complete {
                self.adapter.authenticate_publication(
                    &self.policy,
                    &self.completion(attempt)?,
                    attempt.publication.as_ref().context("terminal publication missing")?,
                )?;
            }
        }
        Ok(())
    }

    fn drive(&mut self, attempt: &mut Attempt) -> Result<View> {
        ensure!(self.adapter.identity() == self.policy.adapter_sha256, "execution adapter changed");
        if attempt.cursor == Cursor::Terminal && attempt.disposition == Disposition::Complete {
            self.adapter.resume_publication(
                &self.policy,
                &self.completion(attempt)?,
                attempt.publication.as_ref().context("terminal publication missing")?,
            )?;
        }
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
                        self.adapter.authenticate_gate(&self.policy, source, stage, &gate)?;
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
                    let journal = FinalizationJournal { journal: &self.journal, policy: &self.policy, attempt };
                    let result = self.adapter.finalize(&context, &self.policy, &request, &journal).and_then(|outcome| {
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
                            // evaluation. Retain the same cleanup operation key
                            // and the original failed/cancelled diagnosis.
                            if attempt.disposition == Disposition::Complete {
                                attempt.reason = bounded(&format!("finalization incomplete: {error:#}"), 4096);
                            }
                            self.record(attempt)?;
                            return self.view(attempt);
                        }
                    }
                }
                Cursor::Publish => {
                    let context = self.intent(attempt, "publish", ExecutionPhase::Publication)?;
                    let completion = self.completion(attempt)?;
                    let journal = PublicationJournal { journal: &self.journal, policy: &self.policy, attempt };
                    match self.adapter.publish(&context, &self.policy, &completion, &journal).and_then(|publication| {
                        self.check_seal(&context, &completion.integrated.sha256, &publication.seal)?;
                        ensure!(
                            !publication.observation_ids.is_empty()
                                && publication.observation_ids.iter().all(|id| !id.is_empty()),
                            "publication requires achieved-stage observations"
                        );
                        ensure!(
                            attempt
                                .prior_retained_bytes
                                .checked_add(completion.finalization.retained_bytes)
                                .and_then(|bytes| bytes.checked_add(publication.retained_bytes))
                                .is_some_and(|bytes| bytes <= self.policy.budgets.max_evidence_bytes),
                            "publication exceeds frozen retained evidence budget"
                        );
                        Ok(publication)
                    }) {
                        Ok(publication) => {
                            attempt.publication = Some(publication);
                            self.completed(attempt, Cursor::Terminal)?;
                        }
                        Err(_) => {
                            // The adapter retains its bounded diagnostic and
                            // writer ownership. Keep the accepted intent stable:
                            // a lost response cannot change the terminal bytes
                            // reserved by a sealed publication result.
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
