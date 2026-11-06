//! The BG graphics update when scrolling (bank `$80` log, `$80:A3DF`): a layer that has moved
//! into another block column or row builds that column or row's tilemap words for the NMI to
//! upload.
//!
//! The block positions (`$80:A4BB`) are the layers' positions and the BG scrolls in 16-pixel
//! blocks. Moving right builds the column 16 blocks on, the one coming into view; moving left
//! the one at the left edge. Moving down builds the row 15 blocks down, moving up the one
//! below the top. The column builder (`$80:A9DE`) reads 16 blocks down the level, the row
//! builder (`$80:AB78`) 17 along; BG2 builds from layer 2's level data only while its scroll
//! byte on that axis says BG2 shows it.

use data::level::Tile;
use data::tileset::{BlockTiles, TileTables};

use crate::state::room::{BlockPoint, RoomScoped};
use crate::state::video::{
    ColumnUpload, LayerUploads, LevelUploads, Registers, RowUpload, Scroll, TilemapEntry,
    TilemapPart, VramWord,
};

/// BG1's tilemap in VRAM, two 32x32 screens side by side.
pub const BG1_TILEMAP: VramWord = VramWord(0x5000);
/// A 32x32 tilemap screen's words.
const SCREEN_WORDS: u16 = 0x0400;
/// A block row's words in a tilemap screen: two rows of 32.
const BLOCK_ROW_WORDS: u16 = 0x0040;
/// The bytes a block adds to a column or row upload: two words.
const BLOCK_BYTES: u16 = 4;
/// The blocks a column builds.
const COLUMN_BLOCKS: u16 = 16;
/// The blocks a row builds.
const ROW_BLOCKS: u16 = 17;
/// The blocks the original's level buffer holds: decompressing a room's level data first
/// fills that many words with `$8000` (bank `$82` log, `$82:E7D3`).
const LEVEL_BUFFER_BLOCKS: usize = 0x3200;
/// The level buffer's bank wraps a block index at 64 KiB of words.
const BANK_BLOCKS: u16 = 0x7FFF;
/// The fill word's bytes, low then high.
const FILL: [u8; 2] = [0x00, 0x80];

/// A layer position in blocks, signed as the original's shift sign-extends it from bit 11.
const fn layer_block(position: u16) -> u16 {
    (position.cast_signed() >> 4).cast_unsigned()
}

/// A layer's top-left corner in blocks.
const fn layer_blocks(x: u16, y: u16) -> BlockPoint {
    BlockPoint {
        x: layer_block(x),
        y: layer_block(y),
    }
}

/// A BG scroll's top-left corner in blocks.
const fn bg_blocks(scroll: Scroll) -> BlockPoint {
    BlockPoint {
        x: scroll.x >> 4,
        y: scroll.y >> 4,
    }
}

/// Whether `now` is before `then`, as the sign of their difference says.
const fn before(now: u16, then: u16) -> bool {
    now.wrapping_sub(then).cast_signed() < 0
}

/// Which level data a builder reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Layer {
    /// Layer 1, for BG1.
    One,
    /// Layer 2, for BG2.
    Two,
}

/// What the builders read: the room's level data and the tile tables.
#[derive(Clone, Copy, Debug)]
struct Level<'a> {
    room: &'a RoomScoped,
    tables: &'a TileTables,
}

impl Level<'_> {
    /// The level data word at block `index` of `layer`, as the original's level buffer holds
    /// it, or `None` where the buffer holds what an earlier room left (docs/LEGACY.md).
    fn word(self, layer: Layer, index: u16) -> Option<u16> {
        let index = usize::from(index & BANK_BLOCKS);
        match layer {
            Layer::One => {
                if let Some(block) = self.room.blocks.get(index) {
                    return Some(block.word());
                }
                if index >= LEVEL_BUFFER_BLOCKS {
                    return None;
                }
                let past = index.checked_sub(self.room.blocks.len())?.wrapping_mul(2);
                let byte =
                    |at: usize, fill: u8| self.room.level_tail.get(at).copied().unwrap_or(fill);
                let [low, high] = FILL;
                Some(u16::from_le_bytes([
                    byte(past, low),
                    byte(past.wrapping_add(1), high),
                ]))
            }
            Layer::Two => self
                .room
                .layer2
                .as_ref()
                .and_then(|blocks| blocks.get(index))
                .map(|block| block.word()),
        }
    }

    /// The four tilemap words the block at `index` draws, with its flips: the entry an item's
    /// GFX load rewrote in the original's tile table, else the tileset's.
    ///
    /// A block word past the level data, or a tile number past what the tile tables fill,
    /// reads stale RAM in the original; the engine draws tile 0's words or zero words there
    /// (docs/LEGACY.md).
    fn drawn(self, layer: Layer, index: u16) -> Drawn {
        self.drawn_tile(Tile::from_word(self.word(layer, index).unwrap_or_default()))
    }

    /// The four tilemap words `tile` draws, with its flips.
    fn drawn_tile(self, tile: Tile) -> Drawn {
        let number = tile.number.0;
        let tiles = crate::plm::item_gfx::block(&self.room.item_gfx, number)
            .or_else(|| self.tables.block(self.room.area, self.room.tileset, number))
            .unwrap_or_default();
        Drawn::of(tiles, tile)
    }

    /// The level block index the original's multiply gives `block`: the row's low byte times
    /// the width's low byte, plus the column.
    fn index(self, block: BlockPoint) -> u16 {
        let [row, _] = block.y.to_le_bytes();
        let [width, _] = self.room.width.0.to_le_bytes();
        u16::from(row)
            .wrapping_mul(u16::from(width))
            .wrapping_add(block.x)
    }
}

/// The four tilemap words `tile` draws in `room` as the PLM draw copies them (`$84:8DAA`): top
/// left, top right, bottom left, bottom right, with the tile's flips.
pub(crate) fn tile_words(room: &RoomScoped, tables: &TileTables, tile: Tile) -> [u16; 4] {
    let Drawn {
        top_left,
        top_right,
        bottom_left,
        bottom_right,
    } = Level { room, tables }.drawn_tile(tile);
    [top_left, top_right, bottom_left, bottom_right].map(TilemapEntry::word)
}

/// A block's four tilemap words as it draws: its tile table entry with its flips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Drawn {
    top_left: TilemapEntry,
    top_right: TilemapEntry,
    bottom_left: TilemapEntry,
    bottom_right: TilemapEntry,
}

impl Drawn {
    /// A block flipped across mirrors its quarters and flips each word the same way.
    fn of(tiles: BlockTiles, tile: Tile) -> Self {
        let entry = |word: u16| {
            let entry = TilemapEntry::from_word(word);
            TilemapEntry {
                flip_x: entry.flip_x != tile.x_flip,
                flip_y: entry.flip_y != tile.y_flip,
                ..entry
            }
        };
        let [top_left, top_right, bottom_left, bottom_right] = [
            tiles.top_left,
            tiles.top_right,
            tiles.bottom_left,
            tiles.bottom_right,
        ]
        .map(entry);
        let unflipped = Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        };
        let across = if tile.x_flip {
            unflipped.mirrored_across()
        } else {
            unflipped
        };
        if tile.y_flip {
            across.mirrored_down()
        } else {
            across
        }
    }

    /// Left and right swapped.
    const fn mirrored_across(self) -> Self {
        Self {
            top_left: self.top_right,
            top_right: self.top_left,
            bottom_left: self.bottom_right,
            bottom_right: self.bottom_left,
        }
    }

    /// Top and bottom swapped.
    const fn mirrored_down(self) -> Self {
        Self {
            top_left: self.bottom_left,
            top_right: self.bottom_right,
            bottom_left: self.top_left,
            bottom_right: self.top_right,
        }
    }
}

/// Where a background's tilemap screens start and where in them a block lands.
#[derive(Clone, Copy, Debug)]
struct Tilemap {
    /// The background's first screen.
    base: VramWord,
}

impl Tilemap {
    /// The screen block column `column` is in, and the other one.
    const fn screens(self, column: u16) -> (VramWord, VramWord) {
        let left = self.base;
        let right = VramWord(self.base.0.wrapping_add(SCREEN_WORDS));
        if column & 0x10 == 0 {
            (left, right)
        } else {
            (right, left)
        }
    }

    /// The word `rows` block rows down and `column` block columns across a screen.
    const fn at(screen: VramWord, row: u16, column: u16) -> VramWord {
        VramWord(
            screen
                .0
                .wrapping_add((row & 0xF).wrapping_mul(BLOCK_ROW_WORDS))
                .wrapping_add((column & 0xF).wrapping_mul(2)),
        )
    }
}

/// Builds block column `block` of `layer` into `upload`, to land at the BG's block column
/// `at` (`$80:A9DE`).
// open: the original builds nothing while the mode 7 flag (`$0783`) is set, a room the slice's
// rooms don't load.
fn column(
    level: Level<'_>,
    layer: Layer,
    block: BlockPoint,
    at: BlockPoint,
    tilemap: Tilemap,
    upload: &mut ColumnUpload,
) {
    let wrapped = at.y.wrapping_mul(BLOCK_BYTES) & 0x3C;
    let (screen, _) = tilemap.screens(at.x);
    upload.first = TilemapPart {
        to: Tilemap::at(screen, at.y, at.x),
        bytes: COLUMN_BLOCKS
            .wrapping_mul(BLOCK_BYTES)
            .wrapping_sub(wrapped),
    };
    upload.wrapped = TilemapPart {
        to: Tilemap::at(screen, 0, at.x),
        bytes: wrapped,
    };
    let start = level.index(block);
    let words = upload
        .left
        .as_chunks_mut::<2>()
        .0
        .iter_mut()
        .zip(upload.right.as_chunks_mut::<2>().0.iter_mut());
    for (step, (left, right)) in (0..COLUMN_BLOCKS).zip(words) {
        let index = start.wrapping_add(step.wrapping_mul(level.room.width.0));
        let drawn = level.drawn(layer, index);
        *left = [drawn.top_left, drawn.bottom_left];
        *right = [drawn.top_right, drawn.bottom_right];
    }
    upload.requests = upload.requests.requested();
}

/// Builds block row `block` of `layer` into `upload`, to land at the BG's block row `at`
/// (`$80:AB78`).
fn row(
    level: Level<'_>,
    layer: Layer,
    block: BlockPoint,
    at: BlockPoint,
    tilemap: Tilemap,
    upload: &mut RowUpload,
) {
    let column = at.x & 0xF;
    let (screen, other) = tilemap.screens(at.x);
    upload.first = TilemapPart {
        to: Tilemap::at(screen, at.y, at.x),
        bytes: COLUMN_BLOCKS.wrapping_sub(column).wrapping_mul(BLOCK_BYTES),
    };
    upload.wrapped = TilemapPart {
        to: Tilemap::at(other, at.y, 0),
        bytes: column.wrapping_add(1).wrapping_mul(BLOCK_BYTES),
    };
    let start = level.index(block);
    let words = upload
        .top
        .as_chunks_mut::<2>()
        .0
        .iter_mut()
        .zip(upload.bottom.as_chunks_mut::<2>().0.iter_mut());
    for (step, (top, bottom)) in (0..ROW_BLOCKS).zip(words) {
        let drawn = level.drawn(layer, start.wrapping_add(step));
        *top = [drawn.top_left, drawn.top_right];
        *bottom = [drawn.bottom_left, drawn.bottom_right];
    }
    upload.requests = upload.requests.requested();
}

/// The blocks one layer and its background are at now, and where the layer was last built.
#[derive(Clone, Copy, Debug)]
struct Moved {
    layer: BlockPoint,
    bg: BlockPoint,
    drawn: BlockPoint,
}

/// Builds the column and the row each layer has moved into, and remembers where it is.
fn layer_uploads(
    level: Level<'_>,
    layer: Layer,
    moved: Moved,
    axes: (bool, bool),
    tilemap: Tilemap,
    uploads: &mut LayerUploads,
) -> BlockPoint {
    let Moved {
        layer: now,
        bg,
        drawn,
    } = moved;
    let (across, down) = axes;
    let mut built = drawn;
    if across && now.x != drawn.x {
        built.x = now.x;
        let ahead = if before(now.x, drawn.x) {
            0
        } else {
            COLUMN_BLOCKS
        };
        let block = BlockPoint {
            x: now.x.wrapping_add(ahead),
            y: now.y,
        };
        let at = BlockPoint {
            x: bg.x.wrapping_add(ahead),
            y: bg.y,
        };
        column(level, layer, block, at, tilemap, &mut uploads.column);
    }
    if down && now.y != drawn.y {
        built.y = now.y;
        let ahead = if before(now.y, drawn.y) {
            1
        } else {
            COLUMN_BLOCKS - 1
        };
        let block = BlockPoint {
            x: now.x,
            y: now.y.wrapping_add(ahead),
        };
        let at = BlockPoint {
            x: bg.x,
            y: bg.y.wrapping_add(ahead),
        };
        row(level, layer, block, at, tilemap, &mut uploads.row);
    }
    built
}

/// The BG graphics update (`$80:A3DF`): BG1's column, BG2's, BG1's row, then BG2's.
///
/// The original runs the four checks in that order; each builds into its own buffers, so
/// building a layer's column and row together gives the same uploads.
///
/// The NMI's scrolling DMAs (`$80:8CD8`, `$80:8DAC`) make the uploads and clear their requests.
pub fn update(
    room: &mut RoomScoped,
    mirrors: &Registers,
    tables: &TileTables,
    uploads: &mut LevelUploads,
) {
    let [bg1, bg2, ..] = mirrors.scroll;
    let camera = room.camera;
    let layer1 = Moved {
        layer: layer_blocks(camera.x.pixels, camera.y.pixels),
        bg: bg_blocks(bg1),
        drawn: camera.drawn.layer1,
    };
    let layer2 = Moved {
        layer: layer_blocks(camera.layer2.x, camera.layer2.y),
        bg: bg_blocks(bg2),
        drawn: camera.drawn.layer2,
    };
    let bg2_tilemap = Tilemap {
        base: VramWord(BG1_TILEMAP.0.wrapping_sub(room.tilemap_gap.0)),
    };
    let axes = (
        camera.layer2_scroll.x.shows_level(),
        camera.layer2_scroll.y.shows_level(),
    );
    let level = Level { room, tables };
    let drawn1 = layer_uploads(
        level,
        Layer::One,
        layer1,
        (true, true),
        Tilemap { base: BG1_TILEMAP },
        &mut uploads.bg1,
    );
    let drawn2 = layer_uploads(
        level,
        Layer::Two,
        layer2,
        axes,
        bg2_tilemap,
        &mut uploads.bg2,
    );
    room.camera.drawn.layer1 = drawn1;
    room.camera.drawn.layer2 = drawn2;
}

#[cfg(test)]
mod tests;
