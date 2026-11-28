#![cfg(not(doctest))]
//! The screens-explosion model, built from its behaviour spec alone.
//!
//! The station explosion is the cinematic between the escape from Ceres and the flight to
//! Zebes. Once the scheduler completes its setup, the station's picture waits in the dark for
//! the music, drifts into view and brightens, and recedes while three bands of bursts cross it.
//! A flash then covers it, the gunship turns and recedes until it disappears, space holds, and
//! the scene dims to black and hands over to the flight to Zebes.
//!
//! The caller carries the scene's State (`types::explosion::ExplosionState`) from frame to
//! frame, and calls `step` once a scene frame with the scheduler's and the audio service's
//! readiness. `step` returns what the scene asks of the other owners; `sprites` reports the
//! picture's sprites.

mod bursts;
pub mod catalogue;
mod departure;
mod events;
mod frame;
mod opening;
mod owners;
mod picture;
mod plane;
mod rules;
mod sky;
mod station;

pub use bursts::BurstFrame;
pub use events::{MapRegion, MusicDelay, MusicRequest, SceneEvent};
pub use frame::step;
pub use owners::{MusicService, Scheduler};
pub use picture::{BurstPicture, SpritePicture, sprites};
pub use rules::Rules;
