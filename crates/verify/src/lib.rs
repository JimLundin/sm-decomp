//! Independent oracle observations, record services, and one indexed production replay.
//!
//! No engine crate depends on it (ARCHITECTURE §4.1, §4.2).

pub mod cache;
pub mod decode;
pub mod fold;
pub mod lock;
pub mod ppu;
pub mod replay;
pub mod report;
pub mod size;
pub mod source;
pub mod spec_units;

#[cfg(test)]
mod tests;

pub use decode::DecodeError;
