//! Pause menu equipment screen tables in bank $82.
//!
//! The screen lists four categories: tanks, beams, suits/misc and boots.
pub use crate::table::LengthError;
use crate::table::Rows;
pub use crate::table::import_words;

/// $82:C02C: four parallel per-category tables, ending at $82:C04C.
pub const CATEGORY_ADDRESS: u32 = 0x82_c02c;
pub const CATEGORIES: usize = 4;
pub const CATEGORY_BYTES: usize = 4 * 2 * CATEGORIES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Category {
    /// Bank $82 list of each entry's tilemap offset.
    pub tilemap_offsets: u16,
    /// Bank $82 list of each entry's equipment bit; zero for tanks.
    pub bitmasks: u16,
    /// RAM address of the equipped bits; zero for tanks.
    pub equipped_address: u16,
    /// Bank $82 list of each entry's enabled tilemap; zero for tanks.
    pub tilemaps: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Categories {
    categories: [Category; CATEGORIES],
}

impl Categories {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        if bytes.len() != CATEGORY_BYTES {
            return Err(LengthError(bytes.len()));
        }
        let column = |table: usize, category: usize| {
            let at = 2 * (CATEGORIES * table + category);
            u16::from_le_bytes([bytes[at], bytes[at + 1]])
        };
        Ok(Self {
            categories: core::array::from_fn(|category| Category {
                tilemap_offsets: column(0, category),
                bitmasks: column(1, category),
                equipped_address: column(2, category),
                tilemaps: column(3, category),
            }),
        })
    }

    pub fn category(&self, index: usize) -> Option<Category> {
        self.categories.get(index).copied()
    }
}

/// $82:C04C: charge, ice, wave, spazer and plasma beam bits.
pub const BEAM_BITMASK_ADDRESS: u32 = 0x82_c04c;
pub const BEAMS: usize = 5;
/// $82:C056: suit and misc item bits.
pub const ITEM_BITMASK_ADDRESS: u32 = 0x82_c056;
pub const ITEMS: usize = 6;
/// $82:C062: boot item bits.
pub const BOOT_BITMASK_ADDRESS: u32 = 0x82_c062;
pub const BOOTS: usize = 3;

pub type BeamBitmasks = Rows<u16, BEAMS>;
pub type ItemBitmasks = Rows<u16, ITEMS>;
pub type BootBitmasks = Rows<u16, BOOTS>;

/// $82:C06C: the beams' tilemap offsets, in bitmask order.
pub const BEAM_TILEMAP_OFFSET_ADDRESS: u32 = 0x82_c06c;
pub type BeamTilemapOffsets = Rows<u16, BEAMS>;
