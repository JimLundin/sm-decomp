//! One indexed acceptance runner over production-owned state and independent observations.

mod admission;
mod appearance;
mod audio;
mod audio_projection;
pub mod baseline;
mod camera;
mod ceres;
mod compare;
mod connected;
mod early_map;
mod health_tail;
mod hud;
pub mod index;
mod movement;
mod movement_touchdown;
mod native_search;
pub mod played;
mod progress;
mod readers;
mod recorded;
pub mod result;
mod runner;
mod sender_offer;
mod shaft;
mod shot_drawing;
mod touched_cells;
mod weapons;

use compare::{CompareFault, Observation};
use index::{Case, Start};
use result::{CaseResult, Counts, Outcome};
use std::path::{Path, PathBuf};

/// Local and CI supply the same paths; selected cases read only their required pinned data.
#[derive(Clone, Debug, Default)]
pub struct Sources {
    /// Pinned record cache directory.
    pub cache: PathBuf,
    /// Original input bundle directory.
    pub bundle: PathBuf,
    /// Player ROM path, checked by its importer.
    pub rom: PathBuf,
    /// Hash-pinned compact native fixture directory.
    pub native: PathBuf,
    /// Oracle lock path used to validate selected records.
    pub lock: PathBuf,
}

/// Admit and replay one case through production with carried selected state.
#[must_use]
pub fn run(case: &Case, sources: &Sources) -> CaseResult {
    let mut result = CaseResult {
        id: case.id.clone(),
        kind: case.kind,
        row: case.row.clone(),
        start_cut: match case.start {
            Start::CacheCut { frame } | Start::NativeRecord { frame, .. } => Some(frame),
            Start::NativeOpportunity { .. }
            | Start::NativeSeed { .. }
            | Start::NativeSenderOffer { .. }
            | Start::ConstructedSenderOffer { .. }
            | Start::PowerOn => None,
        },
        end_cut: case.end.cut_inclusive,
        counts: Counts::default(),
        outcome: Outcome::Pass,
        observation_sha256: String::new(),
        limits: case.limits.clone(),
    };
    let outcome = case
        .validate()
        .map_err(|_| data_failure(None, "case execution recipe"))
        .and_then(|()| admission::admit(case, sources))
        .and_then(|admitted| runner::run(case, admitted, &mut result));
    if let Err(outcome) = outcome {
        result.outcome = outcome;
    }
    if result.passed() && result.counts.cuts == 0 {
        result.outcome = data_failure(None, "empty comparison schedule");
    }
    result
}

fn data_failure(cut: Option<u64>, source: &str) -> Outcome {
    Outcome::DataFailure {
        cut,
        source: source.to_owned(),
    }
}

fn production_failure(cut: u64, fault: game::production::ProductionFault) -> Outcome {
    match fault {
        game::production::ProductionFault::Connected(fault) => Outcome::Unsupported {
            cut: Some(cut),
            dependency: format!("connected ordinary {fault:?}"),
        },
        game::production::ProductionFault::Unsupported(dependency) => Outcome::Unsupported {
            cut: Some(cut),
            dependency: format!("{dependency:?}"),
        },
        game::production::ProductionFault::WrongScope => {
            data_failure(Some(cut), "production input scope")
        }
        game::production::ProductionFault::Movement(_) => Outcome::Unsupported {
            cut: Some(cut),
            dependency: "movement pose capability".to_owned(),
        },
        game::production::ProductionFault::Weapons(_) => Outcome::Unsupported {
            cut: Some(cut),
            dependency: "ordinary weapons update capability".to_owned(),
        },
        game::production::ProductionFault::Appearance(_) => Outcome::Unsupported {
            cut: Some(cut),
            dependency: "appearance clip capability".to_owned(),
        },
        game::production::ProductionFault::Blocks(_) => {
            data_failure(Some(cut), "block input room bounds")
        }
    }
}

fn compared(cut: u64, expected: &[Observation], actual: &[Observation]) -> Result<(), Outcome> {
    compare::compare(expected, actual).map_err(|fault| match fault {
        CompareFault::Difference(field) => Outcome::Difference { cut, field },
        CompareFault::MissingActual(field) => {
            data_failure(Some(cut), &format!("missing actual {field}"))
        }
        CompareFault::DuplicateField(field) => {
            data_failure(Some(cut), &format!("duplicate field {field}"))
        }
        CompareFault::EmptyExpected => data_failure(Some(cut), "empty expected observation"),
    })
}

fn append_digest(bytes: &mut Vec<u8>, cut: u64, expected: &[Observation]) {
    bytes.extend_from_slice(&cut.to_le_bytes());
    for observed in expected {
        bytes.extend_from_slice(
            &u64::try_from(observed.field.len())
                .unwrap_or(u64::MAX)
                .to_le_bytes(),
        );
        bytes.extend_from_slice(observed.field.as_bytes());
        bytes.extend_from_slice(
            &u64::try_from(observed.bytes.len())
                .unwrap_or(u64::MAX)
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&observed.bytes);
    }
}

/// Read a complete strict case index.
///
/// # Errors
/// Missing, malformed and empty indices are rejected.
pub fn read_index(path: &Path) -> Result<index::IndexSnapshot, index::IndexFault> {
    index::IndexSnapshot::read(path)
}

#[cfg(test)]
mod tests {
    use super::{Sources, index::Index, result::Outcome, run};
    use std::io::Write;
    #[test]
    fn missing_native_data_fails_before_any_cut_is_credited() {
        let index = Index::parse(include_str!("../../../scenarios/replay/index.toml")).unwrap();
        let case = index
            .cases
            .iter()
            .find(|case| case.id.0 == "audio/native-control")
            .unwrap();
        let result = run(case, &Sources::default());
        assert!(matches!(result.outcome, Outcome::DataFailure { .. }));
        assert_eq!(result.counts.cuts, 0);
    }

    #[test]
    fn directly_constructed_invalid_recipe_cannot_reach_admission() {
        let index = Index::parse(include_str!("../../../scenarios/replay/index.toml")).unwrap();
        let mut case = index.cases.first().unwrap().clone();
        case.expectations.fields.clear();
        case.expectations.outputs.clear();
        let result = run(&case, &Sources::default());
        assert_eq!(
            result.outcome,
            Outcome::DataFailure {
                cut: None,
                source: "case execution recipe".to_owned()
            }
        );
        assert_eq!(result.counts, super::Counts::default());
    }
    #[test]
    #[ignore = "private pinned native capture"]
    fn native_control_production_receipt() {
        let index = Index::parse(include_str!("../../../scenarios/replay/index.toml")).unwrap();
        let case = index
            .cases
            .iter()
            .find(|case| case.id.0 == "audio/native-control")
            .unwrap();
        let native = std::env::var_os("SM_AUDIO_NATIVE")
            .map(std::path::PathBuf::from)
            .unwrap();
        let result = run(
            case,
            &Sources {
                native,
                rom: std::env::var_os("SM_ROM").unwrap().into(),
                ..Sources::default()
            },
        );
        let receipt = toml::to_string(&result).unwrap();
        std::io::stdout()
            .lock()
            .write_all(receipt.as_bytes())
            .unwrap();
        assert_eq!(result.outcome, Outcome::Pass);
        assert_eq!(result.counts.cuts, 184);
        assert_eq!(result.counts.fields, 7157);
        // Twelve replies were published by an effect turn still under way at their cut.
        assert_eq!(result.counts.replies, 724);
        assert_eq!(result.counts.outputs, 1276);
        assert_eq!(result.counts.restorations, 184);
    }
    #[test]
    #[ignore = "private pinned indexed captures"]
    fn indexed_recorded_case_receipt() {
        let index = Index::parse(include_str!("../../../scenarios/replay/index.toml")).unwrap();
        let id = std::env::var("SM_REPLAY_CASE").unwrap();
        let case = index.cases.iter().find(|case| case.id.0 == id).unwrap();
        let path = |name| {
            std::env::var_os(name)
                .map(std::path::PathBuf::from)
                .unwrap_or_default()
        };
        let sources = Sources {
            cache: path("SM_CACHE"),
            bundle: path("SM_BUNDLE"),
            rom: path("SM_ROM"),
            native: path("SM_AUDIO_NATIVE"),
            lock: std::path::PathBuf::from("scenarios/locks/lsnes.toml"),
        };
        let result = run(case, &sources);
        std::io::stdout()
            .lock()
            .write_all(toml::to_string(&result).unwrap().as_bytes())
            .unwrap();
        assert_eq!(result.outcome, Outcome::Pass);
    }
}
