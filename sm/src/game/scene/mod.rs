//! Owned display commitment and immutable, fully settled Scene inputs.
//!
//! Construction supplies complete typed starting fields. Committed Scenes retain
//! their media lease and line history; presentation observes them without running
//! gameplay, refresh, transfer or interrupt work.
#![forbid(unsafe_code)]

pub(crate) mod atmosphere;
pub(crate) mod commit;
pub mod component;
pub(crate) mod delivery;
pub(crate) mod hdma;
pub mod hdma_sources;
pub(crate) mod irq;
pub(crate) mod objects;
pub(crate) mod owned_hdma;
pub mod prepared;
pub mod records;
pub(crate) mod settle;
pub mod start;
pub mod state;

pub use commit::ResidentDisplayDescription;
