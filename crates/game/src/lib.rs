//! The game itself: the `Session`, the `State` schema and its codec, the Tick, its strands,
//! the slot store and interpreter, every gameplay domain, the CPU side of sound, the cost
//! model, SRAM and the intent view.
//!
//! It touches no pixels, samples, files, devices or SNES addresses (ARCHITECTURE §4.1, K1).
#![deny(clippy::float_arithmetic)]

pub mod game_time;
pub mod lines;
pub mod message_box;
pub mod nmi;
pub mod oam;
pub mod rng;
pub mod session;
pub mod state;
pub mod tick;

#[cfg(test)]
mod tests;

pub use session::{
    AutoRepeat, Begun, OpenError, Rules, Schedule, Session, SessionFault, TickInput, TickOutput,
    TimingSource,
};
