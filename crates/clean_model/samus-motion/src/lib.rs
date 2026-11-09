//! The samus-motion model, built from its behaviour spec alone: a character controller for
//! Samus, stepped one 60 Hz frame at a time, her motion in fixed-point subpixels.
//!
//! Every rule names the spec entry it comes from. `samus` holds her state and a frame's
//! outputs; `step` runs a frame, `motion` her horizontal motion, `collide` what her moves meet,
//! `fit` pose fit, `jump_held` jump held through, and `chart` with `situation` and `rows` the
//! pose chart; `rules` the tuning parameters and `catalogue` the spec's entries.

pub mod catalogue;
pub mod chart;
mod collide;
mod fit;
mod jump_held;
mod motion;
pub mod rows;
pub mod rules;
mod samus;
pub mod situation;
mod step;
mod tick;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use chart::{Choice, Moment};
pub use jump_held::JumpHeld;
pub use motion::{Carry, DashIs, Run};
pub use rows::{Effect, Met};
pub use rules::{Rules, SpeedCap};
pub use samus::{Bounces, Drawn, Outcome, Samus, Seat, Spin, Standing, Vertical, WallTest};
pub use situation::classify;
pub use tick::Frame;
