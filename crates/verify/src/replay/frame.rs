//! Budget-only production advancement and complete ordered frame observations.

use super::{compare::Observation, data_failure, production_failure, result::Outcome};
use data::timing::MasterTicks;
use game::production::{
    AdvanceInput, Production, ProductionFault, ProductionOutputs, frame, timing,
};
use game::state::owned::OwnedState;

mod callbacks;
pub(super) mod random;
use crate::lock::Sha256;
pub(super) use callbacks::{Datum, ProjectionFault, decode_callbacks, decode_endpoint, project};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Independent frozen output and endpoint files, inaccessible to the input reader.
pub(super) struct Expected {
    pub returns: BTreeMap<u64, Return>,
    pub(super) random: Option<random::Source>,
}

/// Outputs and terminal owners from one independently captured return.
pub(super) struct Return {
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
        let mut expected = self
            .returns
            .get(&cut)
            .ok_or_else(|| data_failure(Some(cut), "frame expected return"))?
            .read(cut, compare_health)?;
        if let Some(source) = &self.random {
            let observed = word("frame.random", source.read(cut)?.result);
            expected.fields.push(observed.clone());
            expected.semantics.push(observed);
            expected.semantic_count = expected.semantics.len();
        }
        Ok(expected)
    }

    #[cfg(test)]
    pub fn observed_random(&self, cut: u64) -> Result<Option<random::Draw>, Outcome> {
        self.random
            .as_ref()
            .map(|source| source.read(cut))
            .transpose()
    }
}

impl Return {
    fn read(&self, cut: u64, compare_health: bool) -> Result<super::readers::ExpectedCut, Outcome> {
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

/// A frame stop's typed production fault and ordered events before the stop.
#[derive(Default, PartialEq, Eq)]
pub(super) struct HeldAdvance {
    fault: Option<ProductionFault>,
    events: Vec<timing::Event>,
}

impl HeldAdvance {
    pub(super) fn retain_events(&mut self, events: Vec<timing::Event>) {
        self.events = events;
    }
}

impl Inputs {
    pub fn advance(
        &self,
        production: &mut Production,
        cut: u64,
        held: &mut HeldAdvance,
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
                .map_err(|fault| {
                    held.fault = Some(fault);
                    held.events = std::mem::take(&mut events);
                    production_failure(cut, fault)
                })?;
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

/// Receiver's pending interval and cumulative callback count, saved at each checkpoint.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Actual {
    pub events: Vec<game::production::timing::Event>,
    delivered: usize,
}

impl Actual {
    pub const fn publication_count(&self) -> usize {
        self.delivered
    }
    pub fn accept(
        &mut self,
        production: &Production,
        outputs: &frame::Outputs,
        cut: u64,
    ) -> Result<Vec<Observation>, Outcome> {
        let OwnedState::Frame(state) = production.state() else {
            return Err(data_failure(Some(cut), "frame owned scope"));
        };
        if outputs.progress != frame::Progress::Completed {
            return Err(data_failure(Some(cut), "complete frame output projection"));
        }
        let callbacks = self.finish_interval(&outputs.events, cut)?;
        Ok(observations(
            state,
            canonical::to_bytes(&callbacks),
            Some(cut),
        ))
    }

    fn finish_interval(
        &mut self,
        events: &[game::production::timing::Event],
        cut: u64,
    ) -> Result<Vec<Datum>, Outcome> {
        let mut callbacks =
            project(&self.events).map_err(|fault| projection_failure(Some(cut), fault))?;
        callbacks.extend(project(events).map_err(|fault| projection_failure(Some(cut), fault))?);
        self.delivered = self
            .delivered
            .checked_add(callbacks.len())
            .ok_or_else(|| data_failure(Some(cut), "frame delivered callback count"))?;
        self.events.clear();
        Ok(callbacks)
    }

    pub fn initial(&self, production: &Production) -> Result<Vec<Observation>, Outcome> {
        let OwnedState::Frame(state) = production.state() else {
            return Err(data_failure(None, "frame initial owned scope"));
        };
        let callbacks = project(&self.events).map_err(|fault| projection_failure(None, fault))?;
        Ok(observations(state, canonical::to_bytes(&callbacks), None))
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
    callbacks: Vec<u8>,
    return_cut: Option<u64>,
) -> Vec<Observation> {
    let mut fields = vec![
        Observation {
            field: "frame.outputs".to_owned(),
            bytes: callbacks,
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
    if let Some(number) = state.current_random() {
        fields.push(word("frame.random", number));
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::production::timing::{Event, EventKind};

    #[test]
    fn verified_return_retires_only_its_projected_callback_interval() {
        let start = Event {
            at: MasterTicks(19),
            kind: EventKind::ControllerStart,
        };
        let end = Event {
            at: MasterTicks(21),
            kind: EventKind::ControllerEnd,
        };
        let next = Event {
            at: MasterTicks(31),
            kind: EventKind::ControllerStart,
        };
        let mut actual = Actual::default();
        actual.events.push(start.clone());
        assert_eq!(
            actual
                .finish_interval(std::slice::from_ref(&end), 1)
                .unwrap(),
            project(&[start, end]).unwrap()
        );
        assert_eq!(actual.publication_count(), 2);

        let next = std::slice::from_ref(&next);
        let mut checkpoint = actual.clone();
        for receiver in [&mut actual, &mut checkpoint] {
            assert_eq!(
                receiver.finish_interval(next, 2).unwrap(),
                project(next).unwrap()
            );
            assert_eq!(receiver.publication_count(), 3);
        }
    }
}
