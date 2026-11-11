//! The samus-looks model, built from its behaviour spec alone: her animation's timing.
//!
//! Her clips are data (`Clip`, a list of typed `Instruction`s), played a frame at a time. A
//! caller holds the `State` between frames, reads the end her clip reaches ahead of
//! samus-motion's step (`Playhead::end_ahead`), then runs `update` with the pose samus-motion
//! picked.

mod catalogue;
mod clip;
mod frame;
mod inputs;
mod moves;
mod playback;
mod rules;
mod run;
mod starting;
mod state;
mod update;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use clip::{Branch, Clip, Frames, Instruction, Pace, Paces, Step};
pub use inputs::{Content, Inputs, Liquid, Move, Pose, Report, Vertical};
pub use rules::Rules;
pub use state::{Boost, Laps, Playhead, RunDown, State};
pub use update::update;
