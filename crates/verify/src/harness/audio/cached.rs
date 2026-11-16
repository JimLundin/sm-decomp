//! Ordinary rows currently lack independently decoded resident audio content, loaded transport
//! state and exact elapsed clock/port phases. They remain defects in scope, with no green gain.

use super::{Observation, native};
use crate::harness::{Domain, Field, Interval, Uncompared};
use audio_transport::{AudioInstant, Content, State, transport};
use oracle_record::FrameRecord;

/// Audio's ordinary-row decoder, separate from the admitted native paired fixture.
pub struct Audio;

/// The actual inputs required to advance an admitted interval.
pub struct Inputs {
    /// Resident content checked at the importer boundary.
    pub content: Content,
    /// Independently measured elapsed audio interval.
    pub elapsed: AudioInstant,
    /// Ordered physical presentations within the interval.
    pub presentations: Vec<transport::Input>,
}

/// A cached row cannot currently provide the audio model's required inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Missing {
    /// Loaded transport, resident content and clock/port phases have no cached decoder.
    NativeInputs,
}

impl std::fmt::Display for Missing {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NativeInputs => out.write_str(
                "audio resident content, loaded transport and exact clock/port phases undecoded",
            ),
        }
    }
}

impl Domain for Audio {
    const NAME: &'static str = "audio-transport";
    type State = State;
    type Inputs = Inputs;
    type World = ();
    type Fields = Vec<Observation>;
    type Skip = Missing;
    const FIELDS: &'static [Field<Self::Fields>] = &[Field {
        name: "admitted transport",
        same: |left, right| left == right,
        show: |fields| format!("{} admitted quantities", fields.len()),
    }];

    fn decode(&self, _: &FrameRecord, _: Interval) -> Result<(State, Inputs), Missing> {
        Err(Missing::NativeInputs)
    }

    fn world(&self, _: &FrameRecord, _: &FrameRecord) -> Result<(), Missing> {
        Err(Missing::NativeInputs)
    }

    fn step(&self, state: &mut State, inputs: &Inputs, (): &mut ()) {
        let _ = transport::advance(
            state,
            &inputs.content,
            inputs.elapsed.clone(),
            &inputs.presentations,
        );
    }

    fn fields(&self, state: &State) -> Vec<Observation> {
        native::project(state)
    }

    fn expected(&self, _: &FrameRecord) -> Result<Vec<Observation>, Missing> {
        Err(Missing::NativeInputs)
    }

    fn uncompared(&self, skip: &Missing) -> Uncompared {
        Uncompared::Defect(skip.to_string())
    }
}
