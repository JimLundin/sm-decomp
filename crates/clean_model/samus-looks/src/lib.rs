//! The samus-looks model, built from its behaviour spec alone: her animation's timing.
//!
//! Each frame a caller runs two things, in this order:
//!
//! 1. Before samus-motion picks her pose, `Playhead::end_ahead`, on the playhead it kept and the
//!    clip of her pose, which says whether her clip ends this frame.
//! 2. After it, `update`, with the `State` it kept, the frame's `Inputs` (the pose samus-motion
//!    picked among them), her clips and the run's paces (`Content`) and the tuning (`Rules`). It
//!    gives back the `State` to keep for the next frame, and a `Report`: the end, whether her
//!    spin has turned far enough to kick off a wall, and the sound asked for.
//!
//! The `State` is all the model remembers: her step, the time left, the boost stage and the laps
//! left. A caller may set each to any value the spec allows, and gives it back as `update` left
//! it. Her clips are data: each `Clip` is a list of steps, each one shown for a `Hold` or a
//! `Control`.

mod catalogue;
mod clip;
mod inputs;
mod moves;
mod playback;
mod rules;
mod run;
mod starting;
mod state;
mod update;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use clip::{Branch, Clip, Control, Frames, Hold, Instruction, Pace, Paces, Slowdown, Step};
pub use inputs::{Content, Inputs, Liquid, Move, Pose, Report};
pub use rules::Rules;
pub use state::{Boost, Laps, Playhead, RunDown, State};
pub use update::update;
