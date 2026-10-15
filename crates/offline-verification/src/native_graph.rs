//! Bounded advisory projection of the existing Native wave/operation contract.
//! No projection is read for execution, reuse, ownership, or deadline decisions.
use super::{NativeIntent, NativeReceipt, PreparedNative};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};
use tool_execution::metrics;
use translation_execution::model::sealed;

const FILE_BYTES: u64 = 32 * 1024;
/// The current file and its atomic replacement both fit before publication.
pub(super) const RESERVED_BYTES: u64 = 2 * FILE_BYTES;

pub(super) fn validate_allowance(bytes: u64) -> Result<()> {
    ensure!(bytes == 0 || bytes == RESERVED_BYTES, "native graph metadata reservation changed");
    Ok(())
}

pub(super) fn check_budget(intent: &NativeIntent, receipt: &NativeReceipt) -> Result<()> {
    validate_allowance(intent.graph_metadata_bytes)?;
    let total = receipt
        .evidence_bytes
        .checked_add(intent.graph_metadata_bytes)
        .context("native evidence and graph metadata extent overflow")?;
    ensure!(
        total <= receipt.reserved_bytes && total <= receipt.max_output_bytes,
        "native evidence and graph metadata exceed reserved output capacity"
    );
    Ok(())
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Phase {
    Prepared,
    Building,
    Fixtures,
    Publishing,
    Complete,
    Rejected,
    OwnershipPending,
}

pub(super) struct Progress<'a> {
    prepared: &'a PreparedNative,
    intent: &'a NativeIntent,
    output: &'a Path,
    admitted_waves: AtomicUsize,
    settled_waves: AtomicUsize,
    compared_cases: AtomicUsize,
}

impl<'a> Progress<'a> {
    pub(super) fn new(prepared: &'a PreparedNative, intent: &'a NativeIntent, output: &'a Path) -> Self {
        Self {
            prepared,
            intent,
            output,
            admitted_waves: AtomicUsize::new(0),
            settled_waves: AtomicUsize::new(0),
            compared_cases: AtomicUsize::new(0),
        }
    }

    pub(super) fn admit_wave(&self) {
        self.admitted_waves.fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn compared(&self) {
        self.compared_cases.fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn settle_wave(&self) {
        let settled = self.settled_waves.fetch_add(1, Ordering::Relaxed) + 1;
        // Large single-case chains must not pay a sync/write for every case.
        // An interrupted projection may lag; operations remain authoritative.
        if settled.is_multiple_of(64) {
            self.phase(Phase::Fixtures, None);
        }
    }
    pub(super) fn phase(&self, phase: Phase, receipt: Option<&NativeReceipt>) {
        self.write(phase, receipt, false);
    }

    pub(super) fn replay(&self, receipt: &NativeReceipt, compared_cases: usize) {
        self.compared_cases.store(compared_cases, Ordering::Relaxed);
        self.write(if receipt.passed { Phase::Complete } else { Phase::Rejected }, Some(receipt), true);
    }

    fn write(&self, phase: Phase, receipt: Option<&NativeReceipt>, replay: bool) {
        // Existing intents did not reserve this sidecar. Preserve their exact
        // retained receipt/reservation semantics and write no new file.
        if self.intent.graph_metadata_bytes == 0 {
            return;
        }
        let result = (|| -> Result<()> {
            validate_allowance(self.intent.graph_metadata_bytes)?;
            #[derive(Serialize)]
            struct Projection<'a> {
                schema: &'static str,
                authority: &'static str,
                scheduling: &'static str,
                prepared_sha256: &'a str,
                candidate_sha256: &'a str,
                manifest_sha256: &'a str,
                operation_context_sha256: String,
                deadline_unix_ms: u64,
                fixture_jobs: usize,
                declared_cases: usize,
                required_producers: usize,
                phase: Phase,
                compared_cases: usize,
                admitted_waves: Option<usize>,
                settled_waves: Option<usize>,
                recovered_from_authenticated_receipt: bool,
                operation_index: Option<&'a translation_execution::retention::Reference>,
                reserved_advisory_bytes: u64,
            }
            let declared = self.prepared.contract.coverage.values().map(Vec::len).sum::<usize>();
            let projection = Projection {
                schema: "offline-native-work-graph-v1",
                authority: "advisory; original intent, operations and authenticated receipt own recovery",
                scheduling: "canonical dependency waves; atomic existing group reservations",
                prepared_sha256: &self.intent.prepared_sha256,
                candidate_sha256: &self.intent.candidate_sha256,
                manifest_sha256: &self.prepared.contract.manifest.sha256,
                operation_context_sha256: sealed(&self.intent.context)?,
                deadline_unix_ms: self.intent.deadline_unix_ms,
                fixture_jobs: self.intent.fixture_jobs,
                declared_cases: declared,
                required_producers: 2
                    + declared * 2
                    + usize::from(crate::recipe::abi(self.prepared.contract.recipe.as_ref())),
                phase,
                compared_cases: self.compared_cases.load(Ordering::Relaxed),
                admitted_waves: (!replay).then(|| self.admitted_waves.load(Ordering::Relaxed)),
                settled_waves: (!replay).then(|| self.settled_waves.load(Ordering::Relaxed)),
                recovered_from_authenticated_receipt: replay,
                operation_index: receipt.map(|receipt| &receipt.operation_index),
                reserved_advisory_bytes: self.intent.graph_metadata_bytes,
            };
            ensure!(
                toml::to_string(&projection)?.len() as u64 <= FILE_BYTES,
                "native graph projection extent exceeded"
            );
            crate::atomic_toml(&self.output.join("work-graph.toml"), &projection)
        })();
        if let Err(error) = result
            && let Some(context) = metrics::active()
        {
            // A projection write cannot turn a completed producer into new work.
            context.configuration(
                "native-graph-projection",
                format!("unavailable: {}", error.to_string().chars().take(512).collect::<String>()),
            );
        }
    }
}
