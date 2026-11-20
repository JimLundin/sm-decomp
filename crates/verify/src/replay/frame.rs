//! Budget-only production advancement and complete ordered frame observations.

use super::{compare::Observation, data_failure, production_failure, result::Outcome};
use data::timing::MasterTicks;
use game::production::{AdvanceInput, Production, ProductionOutputs, frame};
use game::state::owned::OwnedState;

mod callbacks;
use crate::lock::Sha256;
pub(super) use callbacks::{Datum, ProjectionFault, decode_callbacks, decode_endpoint, project};
use std::path::{Path, PathBuf};

/// Independent frozen output and endpoint files, inaccessible to the input reader.
pub(super) struct Expected {
    pub callbacks: PathBuf,
    pub callbacks_sha256: Sha256,
    pub endpoint: PathBuf,
    pub endpoint_sha256: Sha256,
    pub final_owners: Option<(PathBuf, Sha256)>,
}

impl Expected {
    /// The common runner calls this only after production advances to actual completion.
    pub fn read(
        &self,
        cut: u64,
        compare_health: bool,
    ) -> Result<super::readers::ExpectedCut, Outcome> {
        if compare_health && self.final_owners.is_none() {
            return Err(data_failure(Some(cut), "missing final Health observations"));
        }
        let bytes = read_expected(&self.callbacks, self.callbacks_sha256, cut)?;
        let callbacks = decode_callbacks(&bytes)
            .map_err(|_| data_failure(Some(cut), "frame expected callbacks"))?;
        let bytes = read_expected(&self.endpoint, self.endpoint_sha256, cut)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| data_failure(Some(cut), "frame endpoint encoding"))?;
        let time = decode_endpoint(text)
            .map_err(|_| data_failure(Some(cut), "frame observed endpoint"))?;
        let time = Observation {
            field: "frame.time".to_owned(),
            bytes: canonical::to_bytes(&time),
        };
        let mut semantics = vec![time];
        if let Some((path, pin)) = &self.final_owners {
            let bytes = read_expected(path, *pin, cut)?;
            let text = std::str::from_utf8(&bytes)
                .map_err(|_| data_failure(Some(cut), "final frame owner endpoint encoding"))?;
            let final_owners: FinalOwners = toml::from_str(text)
                .map_err(|_| data_failure(Some(cut), "final frame owner endpoint"))?;
            let play_time = super::frame_history_inputs::play_time([
                final_owners.play_hours,
                final_owners.play_minutes,
                final_owners.play_seconds,
                final_owners.play_frames,
            ])?;
            semantics.extend(play_time_fields(play_time));
            semantics.extend(shake_fields(
                final_owners.shake_timer,
                final_owners.shake_phase,
            ));
            if compare_health != final_owners.health.is_some() {
                return Err(data_failure(Some(cut), "declared final Health fields"));
            }
            if let Some(health) = final_owners.health {
                semantics.extend(health.fields());
            }
        }
        let mut fields = vec![Observation {
            field: "frame.outputs".to_owned(),
            bytes: canonical::to_bytes(&callbacks),
        }];
        fields.extend(semantics.iter().cloned());
        Ok(super::readers::ExpectedCut {
            fields,
            semantic_count: semantics.len(),
            semantics,
            // Exact stream comparison proves the complete callback cardinality;
            // Actual::publication_count counts it once, without a duplicate here.
            output_count: 0,
            replies: 0,
        })
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FinalOwners {
    play_hours: u16,
    play_minutes: u16,
    play_seconds: u16,
    play_frames: u16,
    shake_timer: u16,
    shake_phase: u16,
    health: Option<FinalHealth>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FinalHealth {
    energy_whole: u16,
    energy_fraction: u16,
    pending_whole: u16,
    pending_fraction: u16,
    warning: u16,
}

impl FinalHealth {
    fn fields(&self) -> [Observation; 5] {
        [
            word("frame.health.energy.whole", self.energy_whole),
            word("frame.health.energy.fraction", self.energy_fraction),
            word("frame.health.pending_loss.whole", self.pending_whole),
            word("frame.health.pending_loss.fraction", self.pending_fraction),
            word("frame.health.warning", self.warning),
        ]
    }
}

impl From<&samus_health::State> for FinalHealth {
    fn from(state: &samus_health::State) -> Self {
        Self {
            energy_whole: state.energy.whole.get(),
            energy_fraction: state.energy.fraction,
            pending_whole: state.pending_loss.whole.get(),
            pending_fraction: state.pending_loss.fraction,
            warning: match state.warning {
                samus_health::Warning::Off => 0,
                samus_health::Warning::On => 1,
            },
        }
    }
}

fn play_time_fields(time: types::PlayTime) -> [Observation; 4] {
    [
        word("frame.play_time.hours", u16::from(time.hours.get())),
        word("frame.play_time.minutes", u16::from(time.minutes.get())),
        word("frame.play_time.seconds", u16::from(time.seconds.get())),
        word("frame.play_time.frames", u16::from(time.frames.get())),
    ]
}

fn shake_fields(timer: u16, phase: u16) -> [Observation; 2] {
    [
        word("frame.room_shake.timer", timer),
        word("frame.room_shake.phase", phase),
    ]
}

fn word(field: &str, value: u16) -> Observation {
    Observation {
        field: field.to_owned(),
        bytes: canonical::to_bytes(&value),
    }
}

fn read_expected(path: &Path, pin: Sha256, cut: u64) -> Result<Vec<u8>, Outcome> {
    let bytes =
        std::fs::read(path).map_err(|_| data_failure(Some(cut), "frame expected member"))?;
    if Sha256::of(&bytes) != pin {
        return Err(data_failure(Some(cut), "frame expected member pin"));
    }
    Ok(bytes)
}

/// Host pause granularity, independent of expected completion time or event counts.
pub(super) struct Inputs {
    pub budget: MasterTicks,
}

impl Inputs {
    pub fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        if self.budget.0 == 0 {
            return Err(data_failure(Some(cut), "positive frame host budget"));
        }
        let mut events = Vec::new();
        loop {
            let outputs = production
                .advance(AdvanceInput::Frame {
                    budget: self.budget,
                })
                .map_err(|fault| production_failure(cut, fault))?;
            let ProductionOutputs::Frame(outputs) = outputs else {
                return Err(data_failure(Some(cut), "frame output scope"));
            };
            events.extend(outputs.events);
            match outputs.progress {
                frame::Progress::Advanced => {}
                frame::Progress::Completed => {
                    return Ok(ProductionOutputs::Frame(frame::Outputs {
                        progress: frame::Progress::Completed,
                        events,
                    }));
                }
            }
        }
    }
}

/// Receiver's exact existing prefix; cloned together with each production checkpoint.
#[derive(Clone, Debug, Default)]
pub(super) struct Actual {
    pub events: Vec<game::production::timing::Event>,
}

impl Actual {
    pub fn publication_count(&self) -> usize {
        self.events
            .iter()
            .filter(|event| {
                !matches!(
                    event.kind,
                    game::production::timing::EventKind::Sample(_, _)
                        | game::production::timing::EventKind::ControllerCommitted
                )
            })
            .count()
    }
    pub fn accept(
        &mut self,
        production: &Production,
        outputs: frame::Outputs,
        cut: u64,
    ) -> Result<Vec<Observation>, Outcome> {
        let OwnedState::Frame(state) = production.state() else {
            return Err(data_failure(Some(cut), "frame owned scope"));
        };
        if outputs.progress != frame::Progress::Completed {
            return Err(data_failure(Some(cut), "complete frame output projection"));
        }
        let mut events = self.events.clone();
        events.extend(outputs.events);
        let fields = observations(state, &events, Some(cut))?;
        self.events = events;
        Ok(fields)
    }

    pub fn initial(&self, production: &Production) -> Result<Vec<Observation>, Outcome> {
        let OwnedState::Frame(state) = production.state() else {
            return Err(data_failure(None, "frame initial owned scope"));
        };
        observations(state, &self.events, None)
    }
}

fn projection_failure(cut: Option<u64>, fault: ProjectionFault) -> Outcome {
    match fault {
        ProjectionFault::Port => Outcome::Unsupported {
            cut,
            dependency: "frame callback output namespace".to_owned(),
        },
        ProjectionFault::Format | ProjectionFault::Class | ProjectionFault::HardwareIdentity => {
            data_failure(cut, "frame callback format")
        }
    }
}

fn observations(
    state: &frame::State,
    events: &[game::production::timing::Event],
    return_cut: Option<u64>,
) -> Result<Vec<Observation>, Outcome> {
    let mut fields = vec![
        Observation {
            field: "frame.outputs".to_owned(),
            bytes: canonical::to_bytes(
                &project(events).map_err(|fault| projection_failure(return_cut, fault))?,
            ),
        },
        Observation {
            field: "frame.time".to_owned(),
            bytes: canonical::to_bytes(&state.timing.hardware.now),
        },
    ];
    // The null model retains its complete admission values. A returned owner must
    // have completed before its fields can witness the real opportunity.
    if (return_cut.is_none() || state.play_time.is_complete())
        && let Some(time) = state.play_time.current()
    {
        fields.extend(play_time_fields(time));
    }
    if (return_cut.is_none() || state.room_quake.is_complete())
        && let Some(quake) = state.room_quake.current()
    {
        fields.extend(shake_fields(quake.remaining.0, quake.phase.0));
    }
    if (return_cut.is_none() || state.maintenance.energy().is_some())
        && let Some(health) = &state.health
    {
        fields.extend(FinalHealth::from(health.state()).fields());
    }
    Ok(fields)
}
