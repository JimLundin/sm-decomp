//! Owned display commitment and immutable, fully settled Scene inputs.
//!
//! Construction supplies complete typed starting fields. Committed Scenes retain
//! their media lease and line history; presentation observes them without running
//! gameplay, refresh, transfer or interrupt work.

pub mod atmosphere;
pub mod atmosphere_protocol;
pub mod checkpoint_codec;
pub mod commit;
pub mod component;
pub mod delivery;
pub use crate::display::error;
pub use crate::display::hdma;
pub mod hdma_sources;
pub mod irq;
pub mod objects;
pub mod owned_hdma;
pub use sm_data::scene as records;
pub mod owned_samus;
pub mod resident_media;
pub mod samus;
pub mod samus_body;
pub mod samus_position_tiles;
pub mod samus_tile_upload;
pub mod settle;
pub mod start;
pub mod state;

pub use commit::ResidentDisplayDescription;

pub mod samus_content;
pub mod samus_native_content;
