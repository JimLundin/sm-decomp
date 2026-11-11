//! The samus-motion model, built from its behaviour spec alone: a character controller for
//! Samus, stepped one 60 Hz frame at a time, her motion in fixed-point subpixels.
//!
//! Every rule names the spec entry it comes from. `samus` holds her state and a frame's
//! outputs; `step` runs a frame in its order: `moves` her moves, `change` the pose change. `run`
//! is her horizontal motion and `vertical` her vertical motion, each a value with its own moves;
//! `collide` what her moves meet, `fit` pose fit, `jump_held` jump held through, and `chart`
//! with `situation` and `rows` the pose chart; `rules` the tuning parameters and `catalogue` the
//! spec's entries.
//!
//! A caller holds a `Samus`, every part of her state public (her `Run` with its `Dashing`,
//! `Vertical`, pose, `Spin`, `Seat` and `JumpHeld`), and steps her a `Frame` at a time
//! (`Samus::step`) in a `World` under `Rules`, reading the frame's `Outcome` (what it gives the
//! camera, the world and her animation, §1, §1.4 and `TouchedCells`, her sprite as the shared
//! `clean_types::Sprite`), or `UnknownPose` for a pose no situation names. The pose chart alone answers
//! through `Moment::next_pose`.

pub mod catalogue;
mod change;
mod chart;
mod collide;
mod fit;
mod jump_held;
mod moves;
mod rows;
pub mod rules;
mod run;
mod samus;
mod situation;
mod step;
mod tick;
mod vertical;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use chart::Moment;
pub use jump_held::{FrameBefore, JumpHeld};
pub use rows::Met;
pub use rules::{Bounces, Rules, SpeedCap, WithItem};
pub use run::{Carry, Dashing, Run};
pub use samus::{Outcome, Samus, Seat, Spin, StandingAnimation, WallTest};
pub use situation::UnknownPose;
pub use tick::Frame;
pub use vertical::{Phase, Vertical};
