#![cfg(not(doctest))]
//! The screens-blackout model, built from its behaviour spec alone.
//!
//! Samus reaches the elevator at the foot of the rotating shaft during the Ceres escape. The
//! scene stops her and freezes the escape clock, waits on the platform while the shaft keeps
//! running, dims the light to forced blank, saves the game and hands the scene to the station
//! explosion. The scheduler drives it one work opportunity at a time (`Opportunity`); each
//! answers with the requests the scene makes of the other parts of the game (`Request`), in
//! order. The State it carries is `types::ceres_blackout::BlackoutState`.

pub mod catalogue;
mod scene;

pub use scene::{
    MUSIC_STOP_DELAY, Opportunity, Reach, Request, RoomPass, SILENCED_CHANNELS, SoundChannel,
    WorkIntervals, advance,
};
