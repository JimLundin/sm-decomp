//! Independent oracle observations, record services, and one indexed production replay.
//!
//! No engine crate depends on it (ARCHITECTURE §4.1, §4.2).

pub mod cache;
pub mod decode;
pub mod lock;
mod ppu;
pub mod replay;
pub mod report;
pub mod source;

#[cfg(test)]
mod tests;

pub use decode::DecodeError;
