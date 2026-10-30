//! Crocomire's tables in bank $A4.
pub use crate::table::LengthError;
use crate::table::Rows;
pub use crate::table::import_words;

/// $A4:8B79: the charge forward / step back and moving claws spritemaps,
/// which keep BG2 scrolled with Crocomire.
pub const SCROLL_SPRITEMAP_ADDRESS: u32 = 0xa4_8b79;
pub const SCROLL_SPRITEMAPS: usize = 17;
pub type ScrollSpritemaps = Rows<u16, SCROLL_SPRITEMAPS>;

/// $A4:9156: X positions of the bridge crumbling enemy projectiles.
pub const BRIDGE_DEBRIS_ADDRESS: u32 = 0xa4_9156;
pub const BRIDGE_DEBRIS: usize = 11;
pub type BridgeDebris = Rows<u16, BRIDGE_DEBRIS>;

/// $A4:99CB: the spike wall's tile words, ended by $FFFF, and their VRAM
/// destinations at $A4:99D9.
pub const SPIKE_WALL_TILE_ADDRESS: u32 = 0xa4_99cb;
pub const SPIKE_WALL_DESTINATION_ADDRESS: u32 = 0xa4_99d9;
pub const SPIKE_WALL_COLUMNS: usize = 6;
/// The tile words with their end marker.
pub type SpikeWallTiles = Rows<u16, { SPIKE_WALL_COLUMNS + 1 }>;
pub type SpikeWallDestinations = Rows<u16, SPIKE_WALL_COLUMNS>;

/// $A4:9C79 and $A4:9E7B: the melting flesh and skeleton BG2 tilemaps, each
/// 256 tiles ended by $FFFF.
pub const FLESH_TILEMAP_ADDRESS: u32 = 0xa4_9c79;
pub const SKELETON_TILEMAP_ADDRESS: u32 = 0xa4_9e7b;
pub const MELT_TILES: usize = 256;
pub const MELT_TILEMAP_BYTES: usize = 2 * (MELT_TILES + 1);
const END: u16 = 0xffff;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeltTilemapError {
    Length(usize),
    /// The first $FFFF is not right after the 256th tile.
    End {
        at: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeltTilemap {
    tiles: Box<[u16; MELT_TILES]>,
}

impl MeltTilemap {
    pub fn import(bytes: &[u8]) -> Result<Self, MeltTilemapError> {
        if bytes.len() != MELT_TILEMAP_BYTES {
            return Err(MeltTilemapError::Length(bytes.len()));
        }
        let word = |at: usize| u16::from_le_bytes([bytes[2 * at], bytes[2 * at + 1]]);
        let end = (0..=MELT_TILES).find(|&at| word(at) == END);
        if end != Some(MELT_TILES) {
            return Err(MeltTilemapError::End {
                at: end.unwrap_or(MELT_TILES + 1),
            });
        }
        Ok(Self {
            tiles: Box::new(core::array::from_fn(word)),
        })
    }

    pub fn tiles(&self) -> &[u16; MELT_TILES] {
        &self.tiles
    }
}
