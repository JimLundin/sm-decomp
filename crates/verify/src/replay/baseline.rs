//! Reviewed replay floors distinguish exact behaviour from explicitly held capability boundaries.

use super::index::{
    Case, CaseId, ExpectedProvider, IndexSnapshot, InputProvider, Responsibility, Start, valid_hash,
};
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
    pub fn validate_index(&self, snapshot: &IndexSnapshot) -> Result<(), BaselineFault> {
        if self.format != 1 || self.index_sha256 != snapshot.sha256() {
            return Err(BaselineFault::IndexPin);
        }
        let index = snapshot.index();
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
        self.validate_index(snapshot)?;
        let index = snapshot.index();
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
                Start::CacheCut { frame }
                | Start::NativeRecord { frame, .. }
                | Start::PlayedContinuation { frame, .. } => Some(frame),
                Start::NativeOpportunity { .. }
                | Start::NativeSeed { .. }
                | Start::NativeSenderOffer { .. }
                | Start::ConstructedSenderOffer { .. }
                | Start::PowerOn => None,
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

fn expanded_counts(case: &Case, counts: &Counts) -> Option<bool> {
    if case.inputs == InputProvider::PlayedShaftSound {
        return Some(
            counts.cuts.checked_mul(case.expectations.fields.len()) == Some(counts.fields)
                && counts.cuts.checked_mul(case.expectations.outputs.len()) == Some(counts.outputs)
                && counts.pending_boundaries == 1
                && counts.publications == 0
                && counts.replies == 0,
        );
    }
    if case.inputs == InputProvider::ScopedIntroDeliveryInputs {
        return Some(
            counts.cuts == 2497
                && counts.fields == 77233
                && counts.outputs == 6532
                && counts.publications == 0
                && counts.replies == 0
                && counts.pending_boundaries == 1,
        );
    }
    if case.inputs == InputProvider::ScopedIntroInputs {
        return Some(super::played::intro::indexed::valid_count_shape(counts));
    }
    if case.inputs == InputProvider::Departure {
        return Some(super::departure::valid_counts(counts));
    }
    if case.inputs == InputProvider::StationExplosion {
        let base = counts.cuts.checked_mul(4)?;
        let settled_fields = counts.fields.checked_sub(base)?;
        let settled = settled_fields / 5;
        let extra = counts
            .outputs
            .checked_sub(counts.cuts.checked_add(settled.checked_mul(2)?)?)?;
        return Some(
            settled_fields.is_multiple_of(5)
                && settled <= counts.cuts
                && matches!(extra, 0 | 3 | 5)
                && counts.publications == 0
                && counts.replies == 0
                && counts.pending_boundaries == 0,
        );
    }
    if case.inputs == InputProvider::Blackout {
        return Some(
            counts.cuts.checked_mul(case.expectations.fields.len()) == Some(counts.fields)
                && counts.cuts.checked_mul(case.expectations.outputs.len()) == Some(counts.outputs)
                && counts.pending_boundaries == 1
                && counts.publications == 0
                && counts.replies == 0,
        );
    }
    if case.inputs == InputProvider::SaveFileBoundaries {
        return Some(
            counts.fields == counts.cuts
                && counts.outputs == 0
                && counts.publications == 0
                && counts.replies == 0
                && counts.pending_boundaries == 2,
        );
    }
    if case.inputs == InputProvider::ZebesFlightInputs {
        return Some(super::zebes::valid_counts(case, counts));
    }
    if case.expectations.provider == ExpectedProvider::NativeEarlyMap {
        return Some(
            super::early_map::field_count(case, counts.cuts) == Some(counts.fields)
                && counts.outputs == 0
                && counts.publications == 0
                && counts.replies == 0
                && counts.pending_boundaries == 0,
        );
    }
    if case.expectations.provider == ExpectedProvider::NativeHealthTailObservations {
        use super::health_tail::{OUTPUTS_PER_CUT, field_count};
        return Some(
            field_count(counts.cuts) == Some(counts.fields)
                && counts.cuts.checked_mul(OUTPUTS_PER_CUT) == Some(counts.outputs)
                && counts.publications == 0
                && counts.replies == 0
                && counts.pending_boundaries == 0,
        );
    }
    None
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
    let restores_per_cut = usize::from(case.checkpoints);
    if counts.cuts > horizon
        || counts.null_cuts > counts.cuts
        || counts.null_fields > counts.fields
        || counts.publications > counts.outputs
        || counts.restorations > counts.cuts.saturating_mul(restores_per_cut)
        || counts.pending_restorations != counts.pending_boundaries
    {
        return false;
    }
    if case.inputs == InputProvider::PlayedShaftDust {
        return floor.expected == Outcome::Pass
            && super::played::shaft_dust::valid_counts(counts, horizon);
    }
    if case.inputs == InputProvider::ScopedCeresFightPicture {
        // This producer advances the entire horizon, compares two complete pictures and
        // restores the first for a single continuation through the second.
        return floor.expected == Outcome::Pass
            && *counts
                == Counts {
                    cuts: 2,
                    fields: 2,
                    outputs: 4,
                    restorations: 1,
                    ..Counts::default()
                };
    }
    if case.checkpoints
        && case
            .expectations
            .outputs
            .contains(&super::index::ObservationId::CeresRibsUploads)
    {
        // One continuation forks the carried first reveal with publication held.
        return floor.expected == Outcome::Pass
            && counts.cuts == horizon
            && counts.restorations == 1
            && counts.pending_boundaries == 1
            && scoped_counts(case, counts, first);
    }
    match floor.expected {
        Outcome::Pass => {
            if counts.cuts != horizon
                || counts.restorations != horizon.saturating_mul(restores_per_cut)
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
                || counts.pending_boundaries != 0
            {
                return false;
            }
        }
        Outcome::Difference { .. }
        | Outcome::DataFailure { .. }
        | Outcome::CheckpointFailure { .. } => return false,
    }
    if case.inputs == super::index::InputProvider::CeresShaftActors {
        return super::played::indexed::shaft::valid_counts(counts);
    }
    if let Some(valid) = expanded_counts(case, counts) {
        return valid;
    }
    if case.responsibilities == [Responsibility::Audio] {
        // Each cut compares at most four replies, and every cut its three request lines.
        return counts
            .cuts
            .checked_mul(4)
            .is_some_and(|most| counts.replies <= most)
            && counts
                .cuts
                .checked_mul(3)
                .and_then(|lines| lines.checked_add(counts.replies))
                == Some(counts.outputs)
            && counts.publications == 0
            && counts.pending_boundaries == 0;
    }
    scoped_counts(case, counts, first)
}

fn scoped_counts(case: &Case, counts: &Counts, first: u64) -> bool {
    use super::index::{InputProvider, ObservationId};
    const WIDTHS: &[(ObservationId, usize)] = &[
        (
            ObservationId::IntroRoomGrid,
            super::intro_rooms::GRID_FIELDS,
        ),
        (
            ObservationId::IntroRoomActor,
            super::intro_rooms::ACTOR_FIELDS,
        ),
        (
            ObservationId::CeresTailState,
            4 + 9 * boss_ceres_ridley::TAIL_SEGMENTS,
        ),
        (
            ObservationId::CeresTailPlaces,
            boss_ceres_ridley::TAIL_SEGMENTS,
        ),
        (ObservationId::CeresTailKnockback, 2),
        (ObservationId::CeresRetreatBody, 23),
        (ObservationId::CeresBackdropWalls, 2),
        (ObservationId::CeresBackdropColours, 6),
        (ObservationId::CeresTailHealth, 6),
    ];
    if case.inputs == InputProvider::ShaftPictures {
        // All slots and the overlay clock are compared on every cut. The number of
        // mounted clocks and compared OAM spans varies with the native room.
        return counts.fields >= counts.cuts.saturating_mul(19)
            && counts.outputs >= counts.cuts.min(2)
            && counts.publications == 1
            && counts.pending_boundaries == 1
            && counts.replies == 0;
    }
    if case.inputs == super::index::InputProvider::ScopedCeresWarning {
        return counts.cuts.checked_mul(3) == Some(counts.fields)
            && counts.outputs >= counts.cuts.saturating_mul(10)
            && counts.publications == 0
            && counts.replies == 0
            && counts.pending_boundaries == 0;
    }
    if case.inputs == super::index::InputProvider::ScopedCeresFlyby {
        // The first successor finishes the retreat. The last restores the room and has
        // no active zoom or two body palette observations.
        return counts.cuts == 113
            && counts.fields == 678
            && counts.outputs == 4987
            && counts.publications == 0
            && counts.replies == 0
            && counts.pending_boundaries == 0;
    }
    let width = |field: &ObservationId| {
        WIDTHS
            .iter()
            .find_map(|(id, width)| (*id == *field).then_some(*width))
            .unwrap_or(1)
    };
    let fields = case.expectations.fields.iter().map(width).sum::<usize>();
    let outputs = case.expectations.outputs.iter().map(width).sum::<usize>();
    let output_cuts = case.expectations.output_cut.map_or(counts.cuts, |at| {
        usize::from(
            at.checked_sub(first)
                .and_then(|offset| usize::try_from(offset).ok())
                .is_some_and(|offset| offset < counts.cuts),
        )
    });
    let has_ribs = case
        .expectations
        .outputs
        .contains(&ObservationId::CeresRibsUploads);
    let valid_publications = if has_ribs {
        counts.publications <= counts.cuts
    } else {
        counts.publications == 0
    };
    fields.checked_mul(counts.cuts) == Some(counts.fields)
        && outputs.checked_mul(output_cuts) == Some(counts.outputs)
        && valid_publications
        && counts.replies == 0
        && (has_ribs && case.checkpoints || counts.pending_boundaries == 0)
}

#[cfg(test)]
mod tests {
    use super::{Baseline, BaselineFault, Claim, Floor, valid_counts};
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
format = 1
engine_api = "production-v1"
observation_schema = "semantic-v1"
lock = "invented-lock.toml"

[[case]]
id = "movie/invented"
kind = "movie"
row = "invented-row"
start = { source = "power-on" }
end = { cut_inclusive = 1 }
rules = ["samus-movement/TravelledDistances"]
profile = "reference"
responsibilities = ["movement", "audio"]
content = "player-rom-import"
inputs = "movie"
expectations = { provider = "pinned-record-cache", fields = ["movement.travel.Left", "movement.travel.Right", "movement.travel.Up", "movement.travel.Down"], outputs = [] }
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
        let mut forged = result;
        forged.counts.outputs = 3;
        assert!(matches!(
            baseline.check(&snapshot, &[forged]),
            Err(BaselineFault::Counts(_))
        ));
    }

    #[test]
    fn a_known_boundary_allows_restorations_only_for_checkpointed_cuts() {
        let (snapshot, baseline, _) = fixture();
        let mut case = snapshot.index().cases.first().unwrap().clone();
        case.end.cut_inclusive = 157;
        let mut boundary = baseline.cases.into_iter().next().unwrap();
        boundary.claim = Claim::Boundary;
        boundary.expected = Outcome::Unsupported {
            cut: Some(157),
            dependency: "producer".to_owned(),
        };

        assert!(valid_counts(&case, &boundary));
        boundary.counts.restorations = 2;
        assert!(!valid_counts(&case, &boundary));
        boundary.counts.restorations = 1;
        case.checkpoints = false;
        assert!(!valid_counts(&case, &boundary));
        boundary.counts.restorations = 0;
        assert!(valid_counts(&case, &boundary));
    }

    #[test]
    fn invalid_horizon_null_counts_restores_and_boundary_credit_are_rejected() {
        let (snapshot, baseline, _) = fixture();
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
                invalid.validate_index(&snapshot),
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
        assert!(boundary.validate_index(&snapshot).is_ok());
        boundary.cases[0].expected = Outcome::Pass;
        assert!(matches!(
            boundary.validate_index(&snapshot),
            Err(BaselineFault::InvalidClaim(_))
        ));
    }
}
