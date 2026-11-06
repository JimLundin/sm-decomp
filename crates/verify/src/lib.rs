//! The comparison between the engine and the oracle: the oracle record to State decoder, one
//! projection per stream, facts, the stream digests, and free-run and lockstep comparison.
//!
//! No engine crate depends on it (ARCHITECTURE §4.1, §4.2).

pub mod cache;
pub mod coverage;
pub mod decode;
pub mod fold;
pub mod lock;
pub mod source;
pub mod strand;

#[cfg(test)]
mod tests;

pub use decode::{DecodeError, RoomHeader, RoomMap, Rules, decode};
