//! The game itself: the `Session`, the `State` schema and its codec, the Tick, its strands,
//! the slot store and interpreter, every gameplay domain, the CPU side of sound, the cost
//! model, SRAM and the intent view.
//!
//! It touches no pixels, samples, files, devices or SNES addresses (ARCHITECTURE §4.1, K1).
#![deny(clippy::float_arithmetic)]

pub mod atmospheric;
pub mod bg_graphics;
pub mod collision;
pub mod commit;
pub mod enemy;
pub mod exact;
pub mod game_time;
pub mod intent;
pub mod interface_hud_bridge;
pub mod lines;
pub mod message_box;
pub mod model;
pub mod nmi;
pub mod plm;
pub mod pose;
pub mod projectile;
pub mod rng;
pub mod run_progress_bridge;
pub mod samus_appearance_bridge;
pub mod samus_drawing;
pub mod samus_movement_bridge;
pub mod samus_palette;
pub mod scrolling;
pub mod session;
pub mod state;
pub mod tick;
pub mod world_blocks_bridge;
pub mod world_camera_bridge;

#[cfg(test)]
mod tests;

pub use session::{
    AutoRepeat, Begun, Content, OpenError, Rules, Schedule, Session, SessionFault, SpeedCap,
    Switches, TickInput, TickOutput, TimingSource,
};
