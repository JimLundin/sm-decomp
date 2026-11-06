//! The item GFX load (bank $84 log, `$84:8764`), the first instruction of an item's list.
//!
//! It takes the next of the four item GFX slots, queues the item's tiles to that slot's VRAM
//! and rewrites that slot's two blocks' tile table entries with its tiles and their palettes.
//!
//! Slot `n` holds tiles `3E0h + 8n` on, and blocks `8Eh + 2n` and `8Fh + 2n` draw them: four
//! tiles each, top left, top right, bottom left, bottom right (the tables `$84:87CD`,
//! `$84:87D5`, `$84:87DD`).

use data::plm::ItemPlm;
use data::tileset::BlockTiles;

use crate::state::room::{ITEM_GFX_SLOTS, ItemGfx, ItemGfxSlot};

/// The first block the item GFX slots rewrite, slot 0's first.
const FIRST_BLOCK: u16 = 0x8e;
/// The blocks each slot rewrites.
const SLOT_BLOCKS: u16 = 2;
/// Slot 0's first tile.
const FIRST_TILE: u16 = 0x3e0;
/// The tiles each slot holds: its blocks' four each.
const SLOT_TILES: usize = 8;
/// The tile numbers from one slot's first to the next's.
const SLOT_STRIDE: u16 = 8;
/// How far up a tilemap word a tile's palette byte goes: the load shifts it into the high
/// byte, then two more (`$84:87B6`), dropping what passes bit 15.
const PALETTE_SHIFT: u32 = 10;

/// The palette byte of each of an item's eight tiles: its load's arguments past its tiles'
/// pointer.
const fn palettes(item: ItemPlm) -> [u8; SLOT_TILES] {
    match item {
        // `$84:E3EF`: every tile palette 0.
        ItemPlm::MorphBall => [0; SLOT_TILES],
    }
}

/// Load `item`'s GFX into the next slot, which the next load after it takes the slot after
/// (`$84:8764`, wrapping after the fourth), and return the slot.
// open: the VRAM transfer of the item's tiles it queues (100h bytes to the slot's VRAM) needs
// the item tiles' bank in the content's graphics; slice 0's rows start with the Morph Ball's
// already loaded.
pub fn load(gfx: &mut ItemGfx, item: ItemPlm) -> ItemGfxSlot {
    let slot = gfx.next;
    if let Some(loaded) = gfx.loaded.get_mut(usize::from(slot.0)) {
        *loaded = Some(item);
    }
    let after = usize::from(slot.0).wrapping_add(1) % ITEM_GFX_SLOTS;
    gfx.next = ItemGfxSlot(u8::try_from(after).unwrap_or_default());
    slot
}

/// The tile table entries `item`'s load into `slot` writes: its first block's, then its
/// second's.
#[must_use]
pub fn entries(slot: ItemGfxSlot, item: ItemPlm) -> [BlockTiles; 2] {
    let first = FIRST_TILE.wrapping_add(u16::from(slot.0).wrapping_mul(SLOT_STRIDE));
    let mut words = [0; SLOT_TILES];
    for ((word, palette), tile) in words.iter_mut().zip(palettes(item)).zip(first..) {
        *word = (u16::from(palette) << PALETTE_SHIFT).wrapping_add(tile);
    }
    let block = |[top_left, top_right, bottom_left, bottom_right]: [u16; 4]| BlockTiles {
        top_left,
        top_right,
        bottom_left,
        bottom_right,
    };
    let [first_block, second_block] = [0, 4].map(|start| {
        let mut quarter = [0; 4];
        for (word, &written) in quarter.iter_mut().zip(words.iter().skip(start)) {
            *word = written;
        }
        block(quarter)
    });
    [first_block, second_block]
}

/// What block `number` draws if a loaded slot rewrote it; `None` for a block the loads don't
/// reach or a slot nothing has loaded since the room loaded, which the tile table draws.
#[must_use]
pub fn block(gfx: &ItemGfx, number: u16) -> Option<BlockTiles> {
    let past = number.checked_sub(FIRST_BLOCK)?;
    let slot = past / SLOT_BLOCKS;
    let item = (*gfx.loaded.get(usize::from(slot))?)?;
    let [first, second] = entries(ItemGfxSlot(u8::try_from(slot).ok()?), item);
    Some(if past % SLOT_BLOCKS == 0 {
        first
    } else {
        second
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_take_the_slots_in_turn_and_wrap_after_the_fourth() {
        let mut gfx = ItemGfx::default();
        let slots: Vec<u8> = (0..5)
            .map(|_| load(&mut gfx, ItemPlm::MorphBall).0)
            .collect();
        assert_eq!(slots, [0, 1, 2, 3, 0]);
        assert_eq!(gfx.next, ItemGfxSlot(1));
        assert_eq!(gfx.loaded, [Some(ItemPlm::MorphBall); ITEM_GFX_SLOTS]);
    }

    #[test]
    fn the_morph_balls_slot_0_entries_are_tiles_3e0_on_in_palette_0() {
        let [first, second] = entries(ItemGfxSlot(0), ItemPlm::MorphBall);
        assert_eq!(
            [
                first.top_left,
                first.top_right,
                first.bottom_left,
                first.bottom_right
            ],
            [0x3e0, 0x3e1, 0x3e2, 0x3e3]
        );
        assert_eq!(
            [
                second.top_left,
                second.top_right,
                second.bottom_left,
                second.bottom_right
            ],
            [0x3e4, 0x3e5, 0x3e6, 0x3e7]
        );
    }

    #[test]
    fn slot_3_holds_tiles_3f8_on() {
        let [first, _] = entries(ItemGfxSlot(3), ItemPlm::MorphBall);
        assert_eq!(first.top_left, 0x3f8);
    }

    #[test]
    fn only_loaded_slots_rewrite_their_blocks() {
        let mut gfx = ItemGfx::default();
        assert_eq!(block(&gfx, 0x8e), None);
        load(&mut gfx, ItemPlm::MorphBall);
        assert_eq!(block(&gfx, 0x8d), None);
        assert_eq!(block(&gfx, 0x8e).map(|tiles| tiles.top_left), Some(0x3e0));
        assert_eq!(block(&gfx, 0x8f).map(|tiles| tiles.top_left), Some(0x3e4));
        // Slot 1's blocks keep the tile table's until a second item loads.
        assert_eq!(block(&gfx, 0x90), None);
        assert_eq!(block(&gfx, 0x96), None);
    }
}
