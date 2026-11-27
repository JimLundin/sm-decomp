//! Replay receipts contain identities and counts, never original observed values.

use super::index::{CaseId, CaseKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case", deny_unknown_fields)]
/// Structured first result of a required replay.
pub enum Outcome {
    /// Every declared cut and observation matched.
    Pass,
    /// First exact differing cut and field.
    Difference {
        /// Original cut identity at the first difference.
        cut: u64,
        /// Stable field identity, with no observed values logged.
        field: String,
    },
    /// A required genuine production dependency is absent.
    Unsupported {
        /// Last admitted boundary, if known.
        cut: Option<u64>,
        /// Missing genuine production dependency.
        dependency: String,
    },
    /// Independent data or observations are missing or malformed.
    DataFailure {
        /// Original cut identity, if known.
        cut: Option<u64>,
        /// Failed independent source identity.
        source: String,
    },
    /// A complete saved continuation did not match.
    CheckpointFailure {
        /// Original saved cut whose continuation failed.
        cut: u64,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Observed proof counts; unavailable cases earn no passing credit.
pub struct Counts {
    /// Complete matching declared cuts.
    pub cuts: usize,
    /// Compared semantic field identities or quantities.
    pub fields: usize,
    /// Compared output identities or occurrences.
    pub outputs: usize,
    /// Matching ordered physical publications.
    pub publications: usize,
    /// Compared physical replies.
    pub replies: usize,
    /// Cuts also matched by holding the preceding State and inputs without issuing new outputs.
    pub null_cuts: usize,
    /// Semantic fields also matched by that held step.
    pub null_fields: usize,
    /// Completed checkpoints proven through completion or a held boundary.
    pub restorations: usize,
    /// Effect or live-offer boundaries proven by split advancement.
    pub pending_boundaries: usize,
    /// Pending effect or live-offer saves proven without early publication.
    pub pending_restorations: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// One safe replay receipt with explicit scope and limits.
pub struct CaseResult {
    /// Committed witness identity.
    pub id: CaseId,
    /// Dependency claim.
    pub kind: CaseKind,
    /// Original oracle row identity.
    pub row: String,
    /// Admitted original cut, absent for a pre-row seed.
    pub start_cut: Option<u64>,
    /// Declared original comparison horizon.
    pub end_cut: u64,
    /// Observed proof counters.
    pub counts: Counts,
    /// Structured first result.
    pub outcome: Outcome,
    /// Digest of the exact compared observation sequence.
    pub observation_sha256: String,
    /// Explicit unproved dependencies and omitted outputs.
    pub limits: Vec<String>,
}

impl CaseResult {
    /// Whether every declared comparison completed exactly.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.outcome == Outcome::Pass
    }
}
