//! The game's content and the records shared across the engine: IDs and stable keys, the
//! ROM decoder and its tables, typed content, world records, profiles, the portable records
//! that `game` emits and `runtime` consumes, the address map and the shared digest.
//!
//! It holds no mutable state and no rules (ARCHITECTURE §4.1).

pub mod address_map;
pub mod assets;
pub mod blocks;
pub mod compression;
pub mod digest;
pub mod dma;
pub mod enemy;
pub mod hud;
pub mod id;
pub mod level;
pub mod map;
pub mod message;
pub mod movement;
pub mod palette;
pub mod plm;
pub mod pose;
pub mod rom;
pub mod room;
pub mod span;
pub mod spritemap;
pub mod video;
