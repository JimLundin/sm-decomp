//! Reviewed replay floors distinguish exact behaviour from explicitly held capability boundaries.

use super::index::{Case, CaseId, ExpectedProvider, Index, Responsibility, Start, valid_hash};
use super::result::{CaseResult, Counts, Outcome};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

/// The kind of claim held by a committed baseline row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Claim {
    /// The case must complete every declared comparison exactly.
    Behaviour,
    /// The case must reach the exact named missing capability; it remains unsupported.
    Boundary,
}

/// One reviewed expected result and its measured counters.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Floor {
    /// Committed case identity.
    pub id: CaseId,
    /// Behaviour proof or explicitly held capability boundary.
    pub claim: Claim,
    /// Exact result identity; boundary expectations are never Pass.
    pub expected: Outcome,
    /// Exact measured counts for the declared comparison scope.
    pub counts: Counts,
    /// Exact compared-observation digest, empty only when no comparison completed.
    pub observation_sha256: String,
}

/// The reviewed baseline for one complete exact index.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    /// Baseline format version.
    pub format: u16,
    /// SHA-256 of the exact index bytes.
    pub index_sha256: String,
    /// Exactly one floor for every indexed case.
    #[serde(rename = "case")]
    pub cases: Vec<Floor>,
}

/// Safe baseline failure identities, without expected values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BaselineFault {
    /// Baseline file is unavailable.
    Missing,
    /// Strict TOML decoding failed.
    Syntax,
    /// Baseline format or exact index pin differs.
    IndexPin,
    /// A case is absent from the baseline or measured selection.
    MissingCase(CaseId),
    /// A case appears more than once.
    DuplicateCase(CaseId),
    /// Case claim and expected result disagree.
    InvalidClaim(CaseId),
    /// Receipt scope, source, horizon or limits differ from the exact indexed case.
    Metadata(CaseId),
    /// Measured case result differs from its exact expected result.
    Result(CaseId),
    /// Measured counters differ from the committed counts.
    Counts(CaseId),
    /// Compared observation digest differs.
    Digest(CaseId),
    /// A required selection is empty.
    Empty,
}

/// A boundary held is counted separately from a completed behaviour proof.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Summary {
    /// Exact completed behaviour cases.
    pub passed_cases: usize,
    /// Exact expected missing-capability boundaries.
    pub held_boundary_cases: usize,
    /// Measured unsupported cases, including held boundaries.
    pub unsupported_cases: usize,
}

impl Baseline {
    /// Read strict reviewed baseline metadata.
    ///
    /// # Errors
    /// Missing or malformed baseline files are rejected.
    pub fn read(path: &Path) -> Result<Self, BaselineFault> {
        let text = std::fs::read_to_string(path).map_err(|_| BaselineFault::Missing)?;
        toml::from_str(&text).map_err(|_| BaselineFault::Syntax)
    }

    /// Validate complete case coverage and the exact index identity.
    ///
    /// # Errors
    /// Missing, extra, duplicate, malformed or incompatible floor rows are rejected.
    pub fn validate_index(&self, index: &Index, hash: &str) -> Result<(), BaselineFault> {
        index.validate().map_err(|_| BaselineFault::IndexPin)?;
        if self.format != 1 || self.index_sha256 != hash || !valid_hash(hash) {
            return Err(BaselineFault::IndexPin);
        }
        let mut ids = BTreeSet::new();
        for floor in &self.cases {
            if !ids.insert(&floor.id) {
                return Err(BaselineFault::DuplicateCase(floor.id.clone()));
            }
            let case = index
                .cases
                .iter()
                .find(|case| case.id == floor.id)
                .ok_or_else(|| BaselineFault::MissingCase(floor.id.clone()))?;
            case.validate()
                .map_err(|_| BaselineFault::InvalidClaim(floor.id.clone()))?;
            let valid = match (&floor.claim, &floor.expected) {
                (Claim::Behaviour, Outcome::Pass) => {
                    floor.counts.cuts > 0 && valid_hash(&floor.observation_sha256)
                }
                (Claim::Boundary, Outcome::Unsupported { .. }) => {
                    if floor.counts.cuts == 0 {
                        floor.observation_sha256.is_empty()
                    } else {
                        valid_hash(&floor.observation_sha256)
                    }
                }
                _ => false,
            };
            if !valid || !valid_counts(case, floor) {
                return Err(BaselineFault::InvalidClaim(floor.id.clone()));
            }
        }
        for case in &index.cases {
            if !ids.contains(&case.id) {
                return Err(BaselineFault::MissingCase(case.id.clone()));
            }
        }
        if ids.is_empty() {
            return Err(BaselineFault::Empty);
        }
        Ok(())
    }

    /// Hold exact outcomes, counts and observation digests for the complete index.
    ///
    /// # Errors
    /// Any missing receipt or changed proof atom fails the check.
    pub fn check(
        &self,
        index: &Index,
        hash: &str,
        results: &[CaseResult],
    ) -> Result<Summary, BaselineFault> {
        let ids: Vec<_> = index.cases.iter().map(|case| case.id.clone()).collect();
        self.check_selected(index, hash, &ids, results)
    }

    /// Check a nonempty explicit shard against the same complete reviewed baseline.
    ///
    /// # Errors
    /// Empty, duplicate, missing, extra or changed receipts are rejected.
    pub fn check_selected(
        &self,
        index: &Index,
        hash: &str,
        selected: &[CaseId],
        results: &[CaseResult],
    ) -> Result<Summary, BaselineFault> {
        self.validate_index(index, hash)?;
        if selected.is_empty() {
            return Err(BaselineFault::Empty);
        }
        let mut expected_ids = BTreeSet::new();
        for id in selected {
            if !expected_ids.insert(id) {
                return Err(BaselineFault::DuplicateCase(id.clone()));
            }
        }
        let mut ids = BTreeSet::new();
        let mut summary = Summary::default();
        for result in results {
            if !ids.insert(&result.id) {
                return Err(BaselineFault::DuplicateCase(result.id.clone()));
            }
            if !expected_ids.contains(&result.id) {
                return Err(BaselineFault::MissingCase(result.id.clone()));
            }
            let floor = self
                .cases
                .iter()
                .find(|floor| floor.id == result.id)
                .ok_or_else(|| BaselineFault::MissingCase(result.id.clone()))?;
            let case = index
                .cases
                .iter()
                .find(|case| case.id == result.id)
                .ok_or_else(|| BaselineFault::MissingCase(result.id.clone()))?;
            let start_cut = match case.start {
                Start::CacheCut { frame } | Start::NativeCheckpoint { frame, .. } => Some(frame),
                Start::NativeOpportunity { .. } | Start::NativeSeed { .. } | Start::PowerOn => None,
            };
            if result.kind != case.kind
                || result.row != case.row
                || result.start_cut != start_cut
                || result.end_cut != case.end.cut_inclusive
                || result.limits != case.limits
            {
                return Err(BaselineFault::Metadata(result.id.clone()));
            }
            if result.outcome != floor.expected {
                return Err(BaselineFault::Result(result.id.clone()));
            }
            if result.counts != floor.counts {
                return Err(BaselineFault::Counts(result.id.clone()));
            }
            if result.observation_sha256 != floor.observation_sha256 {
                return Err(BaselineFault::Digest(result.id.clone()));
            }
            match floor.claim {
                Claim::Behaviour => summary.passed_cases = summary.passed_cases.saturating_add(1),
                Claim::Boundary => {
                    summary.held_boundary_cases = summary.held_boundary_cases.saturating_add(1);
                }
            }
            if matches!(result.outcome, Outcome::Unsupported { .. }) {
                summary.unsupported_cases = summary.unsupported_cases.saturating_add(1);
            }
        }
        for id in expected_ids {
            if !ids.contains(id) {
                return Err(BaselineFault::MissingCase(id.clone()));
            }
        }
        Ok(summary)
    }
}

fn valid_counts(case: &Case, floor: &Floor) -> bool {
    let Some(first) = case.first_cut() else {
        return false;
    };
    let Some(horizon) = case
        .end
        .cut_inclusive
        .checked_sub(first)
        .and_then(|span| span.checked_add(1))
        .and_then(|count| usize::try_from(count).ok())
    else {
        return false;
    };
    let counts = &floor.counts;
    if counts.cuts > horizon
        || counts.null_cuts > counts.cuts
        || counts.null_fields > counts.fields
        || counts.publications > counts.outputs
        || counts.pending_restorations != counts.pending_boundaries
    {
        return false;
    }
    match floor.expected {
        Outcome::Pass => {
            if counts.cuts != horizon
                || counts.restorations != if case.checkpoints { horizon } else { 0 }
            {
                return false;
            }
        }
        Outcome::Unsupported { cut: None, .. } => {
            if *counts != Counts::default() {
                return false;
            }
        }
        Outcome::Unsupported { cut: Some(cut), .. } => {
            if cut < first
                || cut > case.end.cut_inclusive
                || usize::try_from(cut.saturating_sub(first)).ok() != Some(counts.cuts)
                || counts.restorations != 0
                || counts.pending_boundaries != 0
            {
                return false;
            }
        }
        Outcome::Difference { .. }
        | Outcome::DataFailure { .. }
        | Outcome::CheckpointFailure { .. } => return false,
    }
    if case.expectations.provider == ExpectedProvider::NativeFrameCallbacks {
        // One declared stream contains every observed publication at this return.
        return case.expectations.fields.len().checked_mul(counts.cuts) == Some(counts.fields)
            && counts.outputs == counts.publications
            && (counts.outputs == 0) == (counts.cuts == 0)
            && counts.replies == 0
            && counts.pending_boundaries == 0;
    }
    if case.responsibilities != [Responsibility::Audio] {
        let semantic_count = case.expectations.fields.len().checked_mul(counts.cuts);
        let output_cuts = case.expectations.output_cut.map_or(counts.cuts, |at| {
            usize::from(
                at.checked_sub(first)
                    .and_then(|offset| usize::try_from(offset).ok())
                    .is_some_and(|offset| offset < counts.cuts),
            )
        });
        if semantic_count != Some(counts.fields)
            || case.expectations.outputs.len().checked_mul(output_cuts) != Some(counts.outputs)
            || counts.publications != 0
            || counts.replies != 0
            || counts.pending_boundaries != 0
        {
            return false;
        }
    }
    if case.responsibilities == [Responsibility::Audio]
        && (counts.cuts.checked_mul(4) != Some(counts.replies)
            || counts.outputs != counts.publications
            || counts.pending_boundaries > counts.publications
            || (!case.checkpoints && counts.pending_boundaries != 0))
    {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{Baseline, BaselineFault, Claim, Floor};
    use crate::replay::{
        index::Index,
        result::{CaseResult, Counts, Outcome},
    };

    fn fixture() -> (Index, Baseline, CaseResult, String) {
        let mut index =
            Index::parse(include_str!("../../../../scenarios/replay/index.toml")).unwrap();
        index.cases.truncate(1);
        let case = index.cases.first().unwrap();
        let hash = "0".repeat(64);
        let counts = Counts {
            cuts: 1,
            outputs: 4,
            restorations: 1,
            ..Counts::default()
        };
        let result = CaseResult {
            id: case.id.clone(),
            kind: case.kind,
            row: case.row.clone(),
            start_cut: Some(155),
            end_cut: 156,
            counts: counts.clone(),
            outcome: Outcome::Pass,
            observation_sha256: hash.clone(),
            limits: case.limits.clone(),
        };
        let baseline = Baseline {
            format: 1,
            index_sha256: hash.clone(),
            cases: vec![Floor {
                id: case.id.clone(),
                claim: Claim::Behaviour,
                expected: Outcome::Pass,
                counts,
                observation_sha256: hash.clone(),
            }],
        };
        (index, baseline, result, hash)
    }

    #[test]
    fn exact_receipt_metadata_and_observation_atoms_are_required() {
        let (index, baseline, result, hash) = fixture();
        assert_eq!(
            baseline
                .check(&index, &hash, std::slice::from_ref(&result))
                .unwrap()
                .passed_cases,
            1
        );
        let mut forged = result.clone();
        forged.kind = crate::replay::index::CaseKind::Integrated;
        assert!(matches!(
            baseline.check(&index, &hash, &[forged]),
            Err(BaselineFault::Metadata(_))
        ));
        let mut forged = result.clone();
        forged.row.push_str("-other");
        assert!(matches!(
            baseline.check(&index, &hash, &[forged]),
            Err(BaselineFault::Metadata(_))
        ));
        let mut forged = result;
        forged.counts.outputs = 3;
        assert!(matches!(
            baseline.check(&index, &hash, &[forged]),
            Err(BaselineFault::Counts(_))
        ));
    }

    #[test]
    fn invalid_horizon_null_counts_restores_and_boundary_credit_are_rejected() {
        let (index, baseline, _, hash) = fixture();
        for counts in [
            Counts {
                cuts: 2,
                ..baseline.cases[0].counts.clone()
            },
            Counts {
                null_cuts: 2,
                ..baseline.cases[0].counts.clone()
            },
            Counts {
                restorations: 0,
                ..baseline.cases[0].counts.clone()
            },
        ] {
            let mut invalid = baseline.clone();
            invalid.cases[0].counts = counts;
            assert!(matches!(
                invalid.validate_index(&index, &hash),
                Err(BaselineFault::InvalidClaim(_))
            ));
        }
        let mut boundary = baseline;
        boundary.cases[0].claim = Claim::Boundary;
        boundary.cases[0].expected = Outcome::Unsupported {
            cut: None,
            dependency: "producer".to_owned(),
        };
        boundary.cases[0].counts = Counts::default();
        boundary.cases[0].observation_sha256.clear();
        assert!(boundary.validate_index(&index, &hash).is_ok());
        boundary.cases[0].expected = Outcome::Pass;
        assert!(matches!(
            boundary.validate_index(&index, &hash),
            Err(BaselineFault::InvalidClaim(_))
        ));
    }
}
