//! The samus-looks model, built from its behaviour spec alone: her animation's timing.
//!
//! Her clips are data (`Clip`, in the spec's own cells), played by one player a frame at a
//! time. A caller holds the `State` between frames, reads the end her clip reaches ahead of
//! samus-motion's step (`Playhead::end_ahead`), then runs `step` with the pose samus-motion
//! picked.

mod catalogue;
mod clip;
mod frame;
mod inputs;
mod moves;
mod player;
mod rules;
mod run;
mod starting;
mod state;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use clip::{Branch, Clip, ClipStep, Frames, Pace, Paces, Step, UnwrittenStep};
pub use frame::step;
pub use inputs::{Content, Inputs, Liquid, Move, Pose, Report, Vertical};
pub use rules::Rules;
pub use state::{Boost, Laps, Playhead, State};
