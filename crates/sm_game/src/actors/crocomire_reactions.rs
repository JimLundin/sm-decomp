//! Crocomire's complete reaction kernel is shared by both admitted actor owners.
//! Only immutable compatibility resource selection remains here.
#![forbid(unsafe_code)]
pub use super::crocomire_reactions_owned::*;

/// Selected imported tables are resources, never a gameplay callback on Game.
pub trait Host {
    fn crocomire_table_word(&self, address: u32, index: isize) -> u16;
    fn crocomire_melt_tilemap(&self, table: u32) -> Option<[u16; sm_data::crocomire::MELT_TILES]>;
}
