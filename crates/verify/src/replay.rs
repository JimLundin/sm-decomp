//! One indexed acceptance runner over production-owned state and independent observations.

mod admission;
mod appearance;
mod audio;
mod audio_admission;
mod audio_projection;
pub mod baseline;
mod camera;
mod ceres;
mod compare;
mod connected;
mod frame;
pub(crate) mod frame_appearance_inputs;
pub(crate) mod frame_appearance_owner;
pub(super) mod frame_camera_inputs;
pub(crate) mod frame_contact_owner;
pub(crate) mod frame_countdown_owner;
pub(crate) mod frame_countdown_scene;
pub(crate) mod frame_drawing_owners;
pub(crate) mod frame_hdma_content;
pub(crate) mod frame_health_inputs;
mod frame_history_inputs;
pub(crate) mod frame_hud_camera_owner;
pub(crate) mod frame_hud_controls_inputs;
pub(crate) mod frame_hud_controls_owner;
pub(crate) mod frame_hud_inputs;
pub(crate) mod frame_inside_contract;
pub(crate) mod frame_liquid_inputs;
pub(crate) mod frame_motion_reset_contract;
pub(crate) mod frame_movement_down_owner;
pub(crate) mod frame_movement_ground_owner;
pub(crate) mod frame_movement_owner;
pub(crate) mod frame_movement_pair_owner;
pub(crate) mod frame_movement_support_owner;
mod frame_origin;
mod frame_palette;
pub(crate) mod frame_player_inputs;
pub(crate) mod frame_player_preparation_receipts;
pub(crate) mod frame_post_down_owner;
pub(crate) mod frame_sender_contract;
pub(crate) mod frame_sender_inputs;
pub(crate) mod frame_suffix_owner;
pub mod frame_tiles;
mod frame_timing;
mod full_movement;
mod hud;
pub mod index;
mod movement;
mod native_search;
mod opportunity;
mod pending;
mod progress;
mod readers;
mod recorded;
pub mod result;
mod runner;
mod weapons;

use compare::{CompareFault, Observation};
use game::production::{AdvanceInput, Production, ProductionOutputs};
use game::state::owned::OwnedState;
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
            Start::CacheCut { frame } | Start::NativeCheckpoint { frame, .. } => Some(frame),
            Start::NativeOpportunity { .. } | Start::NativeSeed { .. } | Start::PowerOn => None,
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
        game::production::ProductionFault::FrameHardware(fault) => {
            use game::production::timing::Fault;
            match fault {
                Fault::MissingContent
                | Fault::UnsupportedDevice
                | Fault::NestedTransfer
                | Fault::InterruptBoundary => Outcome::Unsupported {
                    cut: Some(cut),
                    dependency: "frame hardware mechanism".to_owned(),
                },
                Fault::PadChangeInPast | Fault::PadChangesUnordered => {
                    data_failure(Some(cut), "frame physical pad input order")
                }
                Fault::InvalidState
                | Fault::InvalidProfile
                | Fault::InvalidCharge
                | Fault::TimeOverflow
                | Fault::ContentOverlap
                | Fault::MathBusy
                | Fault::WorkBusy => data_failure(Some(cut), "frame hardware continuation"),
            }
        }
        game::production::ProductionFault::FramePublications(_) => {
            data_failure(Some(cut), "frame publication order/cursor")
        }
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
        game::production::ProductionFault::TimeReversed => {
            data_failure(Some(cut), "audio clock order")
        }
        game::production::ProductionFault::PresentationOutsideInterval => {
            data_failure(Some(cut), "audio presentation interval")
        }
        game::production::ProductionFault::PresentationsUnordered => {
            data_failure(Some(cut), "audio presentation order")
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

/// This reader owns only physical presentations and the declared measured stop schedule.
struct AudioInputReader<'a> {
    tape: &'a audio::InputTape,
}
impl AudioInputReader<'_> {
    fn advance(
        &self,
        production: &mut Production,
        cut: u64,
        stop: &audio_transport::AudioInstant,
    ) -> Result<ProductionOutputs, Outcome> {
        let OwnedState::Audio(state) = production.state() else {
            return Err(data_failure(Some(cut), "audio state scope"));
        };
        let presentations: Vec<_> = self
            .tape
            .presentations
            .iter()
            .filter(|input| input.at > state.0.now && input.at <= *stop)
            .cloned()
            .collect();
        production
            .advance(AdvanceInput::Audio {
                stop: stop.clone(),
                presentations: &presentations,
            })
            .map_err(|fault| production_failure(cut, fault))
    }
}

/// Expected quantities can be read only after advancement, by the comparison side.
struct AudioExpectedReader<'a> {
    tape: &'a audio::ExpectedTape,
}
impl AudioExpectedReader<'_> {
    fn at(&self, cut: u64) -> Result<&audio::Cut, Outcome> {
        self.tape
            .cuts
            .iter()
            .find(|expected| u64::try_from(expected.index).ok() == Some(cut))
            .ok_or_else(|| data_failure(Some(cut), "missing native expected cut"))
    }
    fn observations(&self, cut: u64) -> Result<Vec<Observation>, Outcome> {
        let expected = self.at(cut)?;
        let mut fields: Vec<_> = expected
            .fields
            .iter()
            .map(|field| Observation::quantity(field.field.clone(), field.value))
            .collect();
        fields.push(Observation::quantity(
            "audio.pending_music",
            i64::from(expected.pending_music),
        ));
        for (slot, byte) in expected.replies.iter().enumerate() {
            fields.push(Observation::quantity(
                format!("audio.reply{slot}"),
                i64::from(*byte),
            ));
        }
        let offset = self
            .tape
            .cuts
            .iter()
            .take_while(|capture| u64::try_from(capture.index).ok() != Some(cut))
            .last()
            .map_or(0, |capture| capture.publications);
        let publications = self
            .tape
            .publications
            .get(offset..expected.publications)
            .ok_or_else(|| data_failure(Some(cut), "native expected publication horizon"))?;
        fields.extend(publication_observations_range(
            publications,
            expected.publications,
            offset,
        )?);
        Ok(fields)
    }
}

fn publication_observations_range(
    publications: &[audio::Publication],
    total: usize,
    offset: usize,
) -> Result<Vec<Observation>, Outcome> {
    #[derive(serde::Serialize)]
    struct Instant<'a> {
        at: &'a audio_transport::AudioInstant,
    }
    let mut fields = vec![Observation::quantity(
        "audio.publication_count",
        i64::try_from(total).map_err(|_| data_failure(None, "publication count range"))?,
    )];
    for (index, publication) in publications.iter().enumerate() {
        let index = offset
            .checked_add(index)
            .ok_or_else(|| data_failure(None, "publication coordinate range"))?;
        fields.push(Observation::quantity(
            format!("audio.publication{index}.line"),
            i64::try_from(audio::reply_slot(publication.line)).unwrap_or(i64::MAX),
        ));
        fields.push(Observation::quantity(
            format!("audio.publication{index}.byte"),
            i64::from(publication.byte),
        ));
        let instant = toml::to_string(&Instant {
            at: &publication.at,
        })
        .map_err(|_| data_failure(None, "publication instant encoding"))?;
        fields.push(Observation {
            field: format!("audio.publication{index}.instant"),
            bytes: instant.into_bytes(),
        });
    }
    Ok(fields)
}

fn add_publications(
    tape: &audio::InputTape,
    outputs: ProductionOutputs,
    publications: &mut Vec<audio::Publication>,
    replies: &mut [u8; 4],
    cut: u64,
) -> Result<(), Outcome> {
    let ProductionOutputs::Audio(outputs) = outputs else {
        return Err(data_failure(Some(cut), "audio output scope"));
    };
    for output in outputs {
        if let audio_transport::transport::Output::Publication {
            at, line, reply, ..
        } = output
        {
            let byte = audio::encode(&tape.aliases, line, reply)
                .map_err(|_| data_failure(Some(cut), "physical reply alias"))?;
            *replies
                .get_mut(audio::reply_slot(line))
                .ok_or_else(|| data_failure(Some(cut), "physical reply line"))? = byte;
            publications.push(audio::Publication { line, byte, at });
        }
    }
    Ok(())
}

fn actual_audio(
    production: &Production,
    publications: &[audio::Publication],
    replies: [u8; 4],
) -> Result<Vec<Observation>, Outcome> {
    actual_audio_from(production, publications, replies, 0)
}

fn actual_audio_from(
    production: &Production,
    publications: &[audio::Publication],
    replies: [u8; 4],
    offset: usize,
) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::Audio(state) = production.state() else {
        return Err(data_failure(None, "audio state scope"));
    };
    let mut fields: Vec<_> = audio_projection::project(&state.0)
        .into_iter()
        .map(|field| Observation::quantity(field.field, field.value))
        .collect();
    let pending = matches!(
        state.0.phase,
        audio_transport::AudioPhase::Work {
            result: audio_transport::CompletionResult::Music(_),
            ..
        }
    );
    fields.push(Observation::quantity(
        "audio.pending_music",
        i64::from(pending),
    ));
    for (slot, byte) in replies.iter().enumerate() {
        fields.push(Observation::quantity(
            format!("audio.reply{slot}"),
            i64::from(*byte),
        ));
    }
    let newly_published = publications
        .get(offset..)
        .ok_or_else(|| data_failure(None, "actual publication coordinate"))?;
    fields.extend(publication_observations_range(
        newly_published,
        publications.len(),
        offset,
    )?);
    Ok(fields)
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
        assert_eq!(result.counts.publications, 764);
        assert_eq!(result.counts.replies, 736);
        assert_eq!(result.counts.restorations, 184);
        assert_eq!(result.counts.pending_boundaries, 573);
        assert_eq!(result.counts.pending_restorations, 573);
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
