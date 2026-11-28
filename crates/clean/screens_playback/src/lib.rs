#![cfg(not(doctest))]
//! The screens-playback model, built from its behaviour spec alone.
//!
//! In the intro's flashback to Zebes, two scenes play Samus from recorded controls: the battle,
//! and the discovery of the baby, which walks her left until she reaches it and then watches.
//! Each turn the model gives Samus and her weapons the recorded pad sample with the one before
//! it, and hands each scene's start and finish on to them. Its State is
//! `types::playback::ScenePhase`; Samus's movement and the room keep their own.

mod catalogue;
mod content;
mod turn;

pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use content::{PadHold, ScenePad};
pub use turn::{
    CrossfadeCount, DemoPad, Handoff, SceneStart, TurnReadings, TurnReport, WholeX, play_turn,
};
