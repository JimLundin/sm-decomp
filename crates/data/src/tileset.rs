//! The tilesets' tile tables: the four 8x8 tilemap words each 16x16 block draws.
//!
//! Bank $82 log, load state header `$82:DEF2`: the room state's tileset byte indexes the
//! tileset pointers (`$8F:E7A7`, a bank $8F word each) to a tileset's three long pointers
//! (tile table, tiles, palette; `$07C0`, `$07C3`, `$07C6`). Loading the level data
//! (`$82:E7D3`) decompresses the tile table into `$7E:A000`: outside Ceres, the common room
//! elements' table (`$B9:A09D`) there first and the tileset's after it at `$7E:A800`, so the
//! tileset's blocks start at block 100h. A block's table entry is four words, top left, top
//! right, bottom left, bottom right; the BG graphics updates (bank $80 log, `$80:A3DF` and
//! the column and row builders it calls) read them by the block's ten-bit tile number.

use canonical::Canonical;

use crate::compression::{DecompressError, decompress};
use crate::rom::{AddressError, Lorom, RomAddress};
use crate::room::Area;

/// The tileset pointers: a bank $8F word per tileset.
pub const TILESET_POINTERS: RomAddress = RomAddress::new(0x8f, 0xe7a7);
/// The common room elements' compressed tile table.
pub const CRE_TILE_TABLE: RomAddress = RomAddress::new(0xb9, 0xa09d);
/// The tilesets the pointer table holds.
pub const TILESETS: usize = 29;
/// The blocks the CRE's table fills before the tileset's (`$7E:A800 - $7E:A000`, eight bytes
/// a block).
pub const CRE_BLOCKS: u16 = 0x100;
/// The blocks a tile number reaches: ten bits.
pub const BLOCKS: usize = 0x400;

/// A tileset's index, the room state's tileset byte.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TilesetId(pub u8);

/// The four tilemap words a 16x16 block draws, unflipped, as the PPU reads them.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BlockTiles {
    /// The top-left 8x8 tile.
    pub top_left: u16,
    /// The top-right tile.
    pub top_right: u16,
    /// The bottom-left tile.
    pub bottom_left: u16,
    /// The bottom-right tile.
    pub bottom_right: u16,
}

impl BlockTiles {
    /// The entry of eight table bytes.
    const fn from_bytes(bytes: [u8; 8]) -> Self {
        let [[tl0, tl1], [tr0, tr1], [bl0, bl1], [br0, br1]] = [
            [bytes[0], bytes[1]],
            [bytes[2], bytes[3]],
            [bytes[4], bytes[5]],
            [bytes[6], bytes[7]],
        ];
        Self {
            top_left: u16::from_le_bytes([tl0, tl1]),
            top_right: u16::from_le_bytes([tr0, tr1]),
            bottom_left: u16::from_le_bytes([bl0, bl1]),
            bottom_right: u16::from_le_bytes([br0, br1]),
        }
    }
}

/// A decompressed tile table: an entry per block from its first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TileTable(pub Vec<BlockTiles>);

impl TileTable {
    /// The table of decompressed bytes. A trailing part entry has no block to describe.
    fn from_bytes(bytes: &[u8]) -> Self {
        Self(
            bytes
                .as_chunks::<8>()
                .0
                .iter()
                .map(|&entry| BlockTiles::from_bytes(entry))
                .collect(),
        )
    }
}

/// Why the tile tables don't load.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TilesetError {
    /// A pointer or a table is outside the image.
    Address(AddressError),
    /// A table doesn't decompress.
    Decompress(RomAddress, DecompressError),
}

/// The CRE's table and every tileset's, from the ROM.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TileTables {
    /// The common room elements'.
    pub cre: TileTable,
    /// One per tileset, by index.
    pub tilesets: Vec<TileTable>,
}

impl TileTables {
    /// Every table from the ROM.
    ///
    /// # Errors
    ///
    /// [`TilesetError`] when a pointer or a table is outside the image or a table doesn't
    /// decompress.
    pub fn load(rom: Lorom<'_>) -> Result<Self, TilesetError> {
        let cre = table(rom, CRE_TILE_TABLE)?;
        let tilesets = (0..TILESETS)
            .map(|index| {
                let near = read_array::<2>(rom, TILESET_POINTERS, index.wrapping_mul(2))?;
                let record = RomAddress::new(TILESET_POINTERS.bank, u16::from_le_bytes(near));
                let long = read_array::<3>(rom, record, 0)?;
                table(rom, RomAddress::from_long(long))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { cre, tilesets })
    }

    /// What block `number` draws in a room of `area` with tileset `tileset`, as the
    /// original's `$7E:A000` holds it after the room loads; `None` past what the tables
    /// fill, where the RAM holds what an earlier room or the boot left (docs/LEGACY.md).
    #[must_use]
    pub fn block(&self, area: Area, tileset: TilesetId, number: u16) -> Option<BlockTiles> {
        let own = self.tilesets.get(usize::from(tileset.0))?;
        let (table, index) = if area == Area::Ceres {
            (own, number)
        } else if let Some(index) = number.checked_sub(CRE_BLOCKS) {
            (own, index)
        } else {
            (&self.cre, number)
        };
        table.0.get(usize::from(index)).copied()
    }
}

/// The decompressed table at `at`.
fn table(rom: Lorom<'_>, at: RomAddress) -> Result<TileTable, TilesetError> {
    let source = rom.from(at).map_err(TilesetError::Address)?;
    let decompressed = decompress(source).map_err(|error| TilesetError::Decompress(at, error))?;
    Ok(TileTable::from_bytes(&decompressed.bytes))
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
