//! Validated game content that the compiler keeps free of the emulator.
//!
//! Target package `sm_data` (docs/target-architecture). Records here are built
//! from bytes by an importer and checked when they are built; no ROM layout,
//! RAM or backend type crosses this boundary.
#![forbid(unsafe_code)]

pub mod ceres_elevator;
pub mod enemy_speeds;
pub mod sand_heap;
pub mod shooting_stars;
pub mod world;
