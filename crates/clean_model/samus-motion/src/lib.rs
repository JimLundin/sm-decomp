//! The samus-motion model, built from its behaviour spec alone: a character controller for
//! Samus, stepped one 60 Hz frame at a time, her motion in fixed-point subpixels.
//!
//! Every rule names the spec entry it comes from. `samus` holds her state and a frame's
//! outputs; `step` runs a frame, `motion` her horizontal motion, `collide` what her moves meet,
//! `fit` pose fit, `jump_held` jump held through, and `chart` with `situation` and `rows` the
//! pose chart; `rules` the tuning parameters and `catalogue` the spec's entries.
//!
//! A caller holds a `Samus`, every part of her state public, and steps her a `Frame` at a time
//! (`Samus::step`) in a `World` under `Rules`, reading the frame's `Outcome`. The pose chart
//! alone answers through `Moment::next_pose`.

pub mod catalogue;
mod chart;
mod collide;
mod fit;
mod jump_held;
mod motion;
mod rows;
pub mod rules;
mod samus;
mod situation;
mod step;
mod tick;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use chart::Moment;
pub use jump_held::{FrameBefore, JumpHeld};
pub use motion::{Carry, DashIs, Run};
pub use rows::Met;
pub use rules::{Rules, SpeedCap};
pub use samus::{Bounces, Drawn, Outcome, Samus, Seat, Spin, Standing, Vertical, WallTest};
pub use tick::Frame;
