#![cfg(not(doctest))]
//! The screens-boot model, built from its behaviour spec alone: the console's power-on, from a
//! black screen through the logo brightening, held and dimming, and black while the sound driver
//! loads and the game settles, until the title begins.
//!
//! Start from `PowerOnState::AT_POWER_ON` and call `step` at each frame end with the `Rules` and
//! what the frame end brought (`FrameEnd`). Its `Outcome` reports the screen and the count of
//! finished frames in the State it carries, or that the title begins.

mod catalogue;
mod fade;
mod frame;
mod power_on;
mod rules;

pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use frame::{DriverUpload, FrameEnd, FrameWork, Outcome};
pub use power_on::step;
pub use rules::{FadeStep, Rules};
pub use types::power_on::{
    FinishedFrames, HoldFrames, Level, PowerOnStage, PowerOnState, ScreenLight,
};
