//! The game itself: the `Session`, the `State` schema and its codec, the Tick, its strands,
//! the slot store and interpreter, every gameplay domain, the CPU side of sound, the cost
//! model, SRAM and the intent view.
//!
//! It touches no pixels, samples, files, devices or SNES addresses (ARCHITECTURE §4.1, K1).
#![deny(clippy::float_arithmetic)]

pub mod animation;
pub mod collision;
pub mod commit;
pub mod enemy;
pub mod game_time;
pub mod hud;
pub mod lines;
pub mod message_box;
pub mod movement;
pub mod nmi;
pub mod oam;
pub mod pose;
pub mod rng;
pub mod scrolling;
pub mod session;
pub mod state;
pub mod tick;

#[cfg(test)]
mod tests;

pub use session::{
    AutoRepeat, Begun, Content, OpenError, Rules, Schedule, Session, SessionFault, TickInput,
    TickOutput, TimingSource,
};
