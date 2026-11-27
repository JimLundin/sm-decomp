//! Reviewed replay floors distinguish exact behaviour from explicitly held capability boundaries.

use super::index::{Case, CaseId, IndexSnapshot, Start, valid_hash};
use super::result::{CaseResult, Counts, Outcome};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};
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
    pub fn validate_index(&self, snapshot: &IndexSnapshot) -> Result<(), BaselineFault> {
        self.validated(snapshot).map(|_| ())
    }

    /// Bind a reviewed baseline to the exact index it covers before checking results.
    ///
    /// # Errors
    /// Missing, extra, duplicate, malformed or incompatible floor rows are rejected.
    pub fn validated<'a>(
        &'a self,
        snapshot: &'a IndexSnapshot,
    ) -> Result<ValidatedBaseline<'a>, BaselineFault> {
        if self.format != 1 || self.index_sha256 != snapshot.sha256() {
            return Err(BaselineFault::IndexPin);
        }
        let index = snapshot.index();
        let cases: BTreeMap<_, _> = index.cases.iter().map(|case| (&case.id, case)).collect();
        let mut floors = BTreeMap::new();
        for floor in &self.cases {
            let Entry::Vacant(entry) = floors.entry(&floor.id) else {
                return Err(BaselineFault::DuplicateCase(floor.id.clone()));
            };
            let case = cases
                .get(&floor.id)
                .ok_or_else(|| BaselineFault::MissingCase(floor.id.clone()))?;
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
            if !valid {
                return Err(BaselineFault::InvalidClaim(floor.id.clone()));
            }
            entry.insert((*case, floor));
        }
        for case in &index.cases {
            if !floors.contains_key(&case.id) {
                return Err(BaselineFault::MissingCase(case.id.clone()));
            }
        }
        if floors.is_empty() {
            return Err(BaselineFault::Empty);
        }
        Ok(ValidatedBaseline { floors })
    }

    /// Hold exact outcomes, counts and observation digests for the complete index.
    ///
    /// # Errors
    /// Any missing receipt or changed proof atom fails the check.
    pub fn check(
        &self,
        snapshot: &IndexSnapshot,
        results: &[CaseResult],
    ) -> Result<Summary, BaselineFault> {
        let ids: Vec<_> = snapshot
            .index()
            .cases
            .iter()
            .map(|case| case.id.clone())
            .collect();
        self.check_selected(snapshot, &ids, results)
    }

    /// Check a nonempty explicit shard against the same complete reviewed baseline.
    ///
    /// # Errors
    /// Empty, duplicate, missing, extra or changed receipts are rejected.
    pub fn check_selected(
        &self,
        snapshot: &IndexSnapshot,
        selected: &[CaseId],
        results: &[CaseResult],
    ) -> Result<Summary, BaselineFault> {
        self.validated(snapshot)?.check_selected(selected, results)
    }
}

/// A reviewed floor for every case in one validated index. The borrowed declarations
/// remain fixed between admission and checking the measured results.
pub struct ValidatedBaseline<'a> {
    floors: BTreeMap<&'a CaseId, (&'a Case, &'a Floor)>,
}

impl ValidatedBaseline<'_> {
    /// Check a nonempty explicit selection against the reviewed floors.
    ///
    /// # Errors
    /// Empty, duplicate, missing, extra or changed receipts are rejected.
    pub fn check_selected(
        &self,
        selected: &[CaseId],
        results: &[CaseResult],
    ) -> Result<Summary, BaselineFault> {
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
            let (case, floor) = self
                .floors
                .get(&result.id)
                .ok_or_else(|| BaselineFault::MissingCase(result.id.clone()))?;
            let start_cut = match case.start {
                Start::CacheCut { frame }
                | Start::NativeRecord { frame, .. }
                | Start::PlayedContinuation { frame, .. } => Some(frame),
                Start::NativeOpportunity { .. }
                | Start::NativeSeed { .. }
                | Start::NativeSenderOffer { .. }
                | Start::ConstructedSenderOffer { .. } => None,
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

#[cfg(test)]
mod tests {
    use super::{Baseline, BaselineFault, Claim, Floor};
    use crate::replay::{
        index::IndexSnapshot,
        result::{CaseResult, Counts, Outcome},
    };

    fn fixture() -> (IndexSnapshot, Baseline, CaseResult) {
        let text = include_str!("../../../../scenarios/replay/index.toml");
        let second_case = text.match_indices("[[case]]").nth(1).unwrap().0;
        let snapshot = IndexSnapshot::parse(text.get(..second_case).unwrap()).unwrap();
        let case = snapshot.index().cases.first().unwrap();
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
            index_sha256: snapshot.sha256().to_owned(),
            cases: vec![Floor {
                id: case.id.clone(),
                claim: Claim::Behaviour,
                expected: Outcome::Pass,
                counts,
                observation_sha256: hash,
            }],
        };
        (snapshot, baseline, result)
    }

    #[test]
    fn loaded_declarations_keep_their_identity_after_file_replacement() {
        let text_a = r#"
format = 2
engine_api = "production-v1"
observation_schema = "semantic-v1"
lock = "invented-lock.toml"

[[case]]
id = "blocks/invented"
kind = "scoped"
row = "invented-row"
start = { source = "cache-cut", frame = 0 }
end = { cut_inclusive = 1 }
rules = []
profile = "reference"
responsibilities = ["blocks"]
content = "player-rom-import"
inputs = "unavailable"
expectations = { provider = "pinned-record-cache", fields = [], outputs = [] }
checkpoints = false
limits = ["invented scope A"]
"#;
        let text_b = text_a.replace("invented scope A", "invented scope B");
        let dir =
            std::env::temp_dir().join(format!("replay-index-snapshot-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("index.toml");
        std::fs::write(&path, text_a).unwrap();
        let a = crate::replay::read_index(&path).unwrap();
        std::fs::write(&path, &text_b).unwrap();
        let b = crate::replay::read_index(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir(&dir).unwrap();

        assert_eq!(
            a.index().cases.first().unwrap().limits,
            ["invented scope A"]
        );
        assert_eq!(
            b.index().cases.first().unwrap().limits,
            ["invented scope B"]
        );
        assert_eq!(
            a.sha256(),
            crate::lock::Sha256::of(text_a.as_bytes()).to_string()
        );
        assert_eq!(
            b.sha256(),
            crate::lock::Sha256::of(text_b.as_bytes()).to_string()
        );
        assert_ne!(a.sha256(), b.sha256());

        let mut baseline = Baseline {
            format: 1,
            index_sha256: a.sha256().to_owned(),
            cases: vec![Floor {
                id: a.index().cases.first().unwrap().id.clone(),
                claim: Claim::Boundary,
                expected: Outcome::Unsupported {
                    cut: None,
                    dependency: "invented producer".to_owned(),
                },
                counts: Counts::default(),
                observation_sha256: String::new(),
            }],
        };
        assert_eq!(baseline.validate_index(&a), Ok(()));
        baseline.index_sha256 = b.sha256().to_owned();
        assert_eq!(baseline.validate_index(&b), Ok(()));
        assert_eq!(baseline.validate_index(&a), Err(BaselineFault::IndexPin));
    }

    #[test]
    fn a_validated_pair_requires_each_selected_case_exactly_once() {
        let (snapshot, mut baseline, result) = fixture();
        let selected = std::slice::from_ref(&result.id);
        let validated = baseline.validated(&snapshot).unwrap();
        assert_eq!(
            validated.check_selected(selected, &[]),
            Err(BaselineFault::MissingCase(result.id.clone()))
        );
        assert_eq!(
            validated.check_selected(selected, &[result.clone(), result.clone()]),
            Err(BaselineFault::DuplicateCase(result.id.clone()))
        );

        let floor = baseline.cases.first().unwrap().clone();
        baseline.cases.push(floor);
        assert!(matches!(
            baseline.validated(&snapshot),
            Err(BaselineFault::DuplicateCase(id)) if id == result.id
        ));
        baseline.cases.clear();
        assert!(matches!(
            baseline.validated(&snapshot),
            Err(BaselineFault::MissingCase(id)) if id == result.id
        ));
    }

    #[test]
    fn exact_receipt_metadata_and_observation_atoms_are_required() {
        let (snapshot, baseline, result) = fixture();
        assert_eq!(
            baseline
                .check(&snapshot, std::slice::from_ref(&result))
                .unwrap()
                .passed_cases,
            1
        );
        let mut forged = result.clone();
        forged.kind = crate::replay::index::CaseKind::Integrated;
        assert!(matches!(
            baseline.check(&snapshot, &[forged]),
            Err(BaselineFault::Metadata(_))
        ));
        let mut forged = result.clone();
        forged.row.push_str("-other");
        assert!(matches!(
            baseline.check(&snapshot, &[forged]),
            Err(BaselineFault::Metadata(_))
        ));
        let validated = baseline.validated(&snapshot).unwrap();
        let selected = std::slice::from_ref(&result.id);
        let mut forged = result.clone();
        forged.counts.outputs = 3;
        assert_eq!(
            validated.check_selected(selected, &[forged]),
            Err(BaselineFault::Counts(result.id.clone()))
        );
        let mut forged = result.clone();
        forged.outcome = Outcome::CheckpointFailure { cut: 156 };
        assert_eq!(
            validated.check_selected(selected, &[forged]),
            Err(BaselineFault::Result(result.id.clone()))
        );
        let mut forged = result.clone();
        forged.observation_sha256 = "f".repeat(64);
        assert_eq!(
            validated.check_selected(selected, &[forged]),
            Err(BaselineFault::Digest(result.id))
        );
    }

    #[test]
    fn boundary_credit_requires_an_unsupported_result() {
        let (snapshot, baseline, _) = fixture();
        let mut boundary = baseline;
        boundary.cases[0].claim = Claim::Boundary;
        boundary.cases[0].expected = Outcome::Unsupported {
            cut: None,
            dependency: "producer".to_owned(),
        };
        boundary.cases[0].counts = Counts::default();
        boundary.cases[0].observation_sha256.clear();
        assert!(boundary.validate_index(&snapshot).is_ok());
        boundary.cases[0].expected = Outcome::Pass;
        assert!(matches!(
            boundary.validate_index(&snapshot),
            Err(BaselineFault::InvalidClaim(_))
        ));
    }
}
