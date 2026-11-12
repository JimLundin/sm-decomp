//! The importer: the registered tilesets and the shared elements from the ROM, the only code
//! that knows where the original keeps them and how it packs them.
//!
//! The room state's tileset byte indexes the tileset pointers to a tileset's three tables: its
//! tile table, its tiles and its palette, each compressed. Decompressed, the tile table holds
//! four tilemap words per block, the tiles are 4bpp planar, 32 bytes a tile, and the palette is
//! a colour word per colour. Outside Ceres the common room elements' tile table fills the
//! first blocks and the tileset's follows it; in Ceres the tileset's starts at block 0. The
//! importer writes `data::assets::tilesets`' records, and the round trip back to the bytes
//! is `game::exact`'s, which the verifier checks against [`raw`] (K1).

use data::assets::art::{
    BlockLook, Cell, Colour, Palette, PaletteRow, PaletteSet, ROW_COLOURS, TILE_SIDE, Tile,
    TileRef, TileSheet,
};
use data::assets::tilesets::{SharedElements, TILESETS, Tileset, Tilesets};
use data::assets::world::World;
use data::compression::{DecompressError, decompress};
use data::id::TilesetId;
use data::rom::{AddressError, Lorom, RomAddress};
use data::room::Area;

use crate::world::tileset_index;

/// The tileset pointers: a word per tileset.
const TILESET_POINTERS: RomAddress = RomAddress::new(0x8f, 0xe7a7);
/// The common room elements' compressed tile table.
const CRE_TILE_TABLE: RomAddress = RomAddress::new(0xb9, 0xa09d);
/// The bytes of a tile table entry: four words.
const ENTRY_BYTES: usize = 8;
/// A tilemap word's fields: the tile number's bits, the palette row's shift and width, and
/// the priority and flip bits.
const TILE_BITS: u16 = 0x03ff;
const PALETTE_SHIFT: u16 = 10;
const PALETTE_BITS: u16 = 0x07;
const PRIORITY_BIT: u16 = 0x2000;
const FLIP_X_BIT: u16 = 0x4000;
const FLIP_Y_BIT: u16 = 0x8000;
/// The bytes of a 4bpp tile: two bytes a row for the first two bit planes, then two a row for
/// the last two.
const TILE_BYTES: usize = 32;
/// The bytes of the last two bit planes' rows into a tile.
const HIGH_PLANES: usize = 16;
/// The bytes of a palette row: a word a colour.
const ROW_BYTES: usize = 32;

/// A tileset's place in the original's tileset table: the room state's tileset byte.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TilesetIndex(pub u8);

/// Why the tilesets don't import.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TilesetError {
    /// A pointer or a table is outside the image.
    Address(AddressError),
    /// A table doesn't decompress.
    Decompress(RomAddress, DecompressError),
    /// The registry has no place for the tileset.
    Unregistered(TilesetId),
    /// No room of the world draws with the tileset, so whether it uses the shared elements
    /// isn't known.
    Unused(TilesetId),
    /// Rooms in Ceres and rooms outside it draw with the tileset, so it uses the shared
    /// elements in some and not in others.
    SharedInSome(TilesetId),
    /// The tileset's tiles or palette end partway through a tile or a palette row.
    Partial(TilesetId),
}

/// A tileset's three tables, decompressed, as the original uploads them: the bytes the records
/// re-encode to (verification only, K1).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Raw {
    /// The tile table: four tilemap words a block.
    pub tile_table: Vec<u8>,
    /// The tiles, 4bpp planar.
    pub tiles: Vec<u8>,
    /// The palette: a colour word a colour.
    pub palette: Vec<u8>,
}

/// The common room elements' tile table, decompressed (verification only, K1).
///
/// # Errors
///
/// [`TilesetError`] when the table is outside the image or doesn't decompress.
pub fn raw_shared(rom: Lorom<'_>) -> Result<Vec<u8>, TilesetError> {
    decompressed(rom, CRE_TILE_TABLE)
}

/// Tileset `id`'s three tables, decompressed (verification only, K1).
///
/// # Errors
///
/// [`TilesetError`] when the tileset isn't registered, or a pointer or a table is outside the
/// image or doesn't decompress.
pub fn raw(rom: Lorom<'_>, id: TilesetId) -> Result<Raw, TilesetError> {
    let index = tileset_index(id).ok_or(TilesetError::Unregistered(id))?;
    let near = read_array::<2>(rom, TILESET_POINTERS, usize::from(index.0).wrapping_mul(2))?;
    let record = RomAddress::new(TILESET_POINTERS.bank, u16::from_le_bytes(near));
    let [tile_table, tiles, palette] = [0, 3, 6].map(|offset| {
        read_array::<3>(rom, record, offset)
            .and_then(|long| decompressed(rom, RomAddress::from_long(long)))
    });
    Ok(Raw {
        tile_table: tile_table?,
        tiles: tiles?,
        palette: palette?,
    })
}

/// Every registered tileset and the shared elements, from the ROM; whether a tileset uses
/// the shared elements comes from where `world`'s rooms draw with it.
///
/// # Errors
///
/// [`TilesetError`] when a table doesn't import, or no room or rooms both in and outside
/// Ceres draw with a tileset.
pub fn import(rom: Lorom<'_>, world: &World) -> Result<Tilesets, TilesetError> {
    let shared = SharedElements {
        looks: looks(&raw_shared(rom)?),
    };
    let sets = TILESETS
        .iter()
        .map(|entry| {
            let raw = raw(rom, entry.id)?;
            Ok(Tileset {
                id: entry.id,
                sheet: sheet(&raw.tiles).ok_or(TilesetError::Partial(entry.id))?,
                looks: looks(&raw.tile_table),
                palettes: palettes(&raw.palette).ok_or(TilesetError::Partial(entry.id))?,
                uses_shared: uses_shared(world, entry.id)?,
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(Tilesets { shared, sets })
}

/// Whether tileset `id` draws the shared elements first: the original loads them for every
/// room outside Ceres.
fn uses_shared(world: &World, id: TilesetId) -> Result<bool, TilesetError> {
    let mut areas = world.rooms().filter_map(|room| {
        let drawn = (0..=room.conditional.len())
            .filter_map(|place| room.version(place))
            .any(|version| version.tileset == id);
        drawn.then_some(room.place.area != Area::Ceres)
    });
    let first = areas.next().ok_or(TilesetError::Unused(id))?;
    if areas.all(|outside| outside == first) {
        Ok(first)
    } else {
        Err(TilesetError::SharedInSome(id))
    }
}

/// The block looks of a tile table, four words an entry. A trailing part entry describes no
/// block.
fn looks(table: &[u8]) -> Vec<BlockLook> {
    table
        .as_chunks::<ENTRY_BYTES>()
        .0
        .iter()
        .map(|entry| {
            let (words, _) = entry.as_chunks::<2>();
            let mut cells = [Cell::default(); 4];
            for (cell, &word) in cells.iter_mut().zip(words) {
                *cell = cell_of(u16::from_le_bytes(word));
            }
            BlockLook::from_cells(cells)
        })
        .collect()
}

/// The cell a tilemap word places: ten bits of tile, three of palette row, then priority and
/// the two flips.
const fn cell_of(word: u16) -> Cell {
    let [palette, _] = ((word >> PALETTE_SHIFT) & PALETTE_BITS).to_le_bytes();
    Cell {
        tile: TileRef(word & TILE_BITS),
        palette: PaletteRow(palette),
        priority: word & PRIORITY_BIT != 0,
        flip_x: word & FLIP_X_BIT != 0,
        flip_y: word & FLIP_Y_BIT != 0,
    }
}

/// The tiles of 4bpp planar bytes, or `None` if they end partway through a tile.
fn sheet(bytes: &[u8]) -> Option<TileSheet> {
    let (tiles, rest) = bytes.as_chunks::<TILE_BYTES>();
    rest.is_empty()
        .then(|| TileSheet(tiles.iter().map(tile_of).collect()))
}

/// The pixels of a 4bpp planar tile: row `y`'s planes 0 and 1 are bytes `2y` and `2y + 1`,
/// planes 2 and 3 the same 16 bytes on; pixel `x` is bit `7 - x` of each.
fn tile_of(bytes: &[u8; TILE_BYTES]) -> Tile {
    let (low, high) = bytes.split_at(HIGH_PLANES);
    let mut tile = Tile::default();
    for ((row, low), high) in tile
        .0
        .iter_mut()
        .zip(low.as_chunks::<2>().0)
        .zip(high.as_chunks::<2>().0)
    {
        let planes = [low[0], low[1], high[0], high[1]];
        for (x, pixel) in row.iter_mut().enumerate() {
            let shift = TILE_SIDE.wrapping_sub(1).wrapping_sub(x);
            *pixel = planes.iter().enumerate().fold(0, |index, (plane, &byte)| {
                index | (((byte >> shift) & 1) << plane)
            });
        }
    }
    tile
}

/// The palette rows of colour words, or `None` if they end partway through a row.
fn palettes(bytes: &[u8]) -> Option<PaletteSet> {
    let (rows, rest) = bytes.as_chunks::<ROW_BYTES>();
    rest.is_empty().then(|| {
        PaletteSet(
            rows.iter()
                .map(|row| {
                    let mut colours = [Colour::default(); ROW_COLOURS];
                    for (colour, &word) in colours.iter_mut().zip(row.as_chunks::<2>().0) {
                        *colour = Colour(u16::from_le_bytes(word));
                    }
                    Palette(colours)
                })
                .collect(),
        )
    })
}

/// The decompressed table at `at`.
fn decompressed(rom: Lorom<'_>, at: RomAddress) -> Result<Vec<u8>, TilesetError> {
    let source = rom.from(at).map_err(TilesetError::Address)?;
    let table = decompress(source).map_err(|error| TilesetError::Decompress(at, error))?;
    Ok(table.bytes)
}

/// The `N` bytes `offset` bytes into the table at `table`.
fn read_array<const N: usize>(
    rom: Lorom<'_>,
    table: RomAddress,
    offset: usize,
) -> Result<[u8; N], TilesetError> {
    let past = TilesetError::Address(AddressError::PastEnd(table));
    let at = table.advance(offset).ok_or(past)?;
    rom.read(at, N)
        .map_err(TilesetError::Address)?
        .first_chunk()
        .copied()
        .ok_or(past)
}

#[cfg(test)]
mod tests;

// ROM sources for the data above, kept out of the public docs (the original's bank and RAM
// addresses): bank $82 log, load state header `$82:DEF2`: the room state's tileset byte
// indexes the tileset pointers (`$8F:E7A7`, a bank $8F word each) to a tileset's three long
// pointers (tile table, tiles, palette; `$07C0`, `$07C3`, `$07C6`). Loading the level data
// (`$82:E7D3`) decompresses the tile table into `$7E:A000`: outside Ceres, the common room
// elements' table (`$B9:A09D`) there first and the tileset's after it at `$7E:A800`. A block's
// entry is four words, top left, top right, bottom left, bottom right; the BG graphics updates
// (bank $80 log, `$80:A3DF`) read them by the block's ten-bit tile number.
