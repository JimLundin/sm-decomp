//! The area maps: each area's map tilemap and the cells its map station reveals, and the
//! bitmap layout the explored cells share with them.
//!
//! Bank $90 log, update mini-map `$90:A91B`, and its RAM map note on `$7E:07F7`: an area's
//! map is 64 by 32 cells, two pages of 32 by 32 side by side. A bitmap of it keeps one bit per
//! cell, four bytes per row and 80h bytes per page, the most significant bit leftmost; the
//! map tilemap keeps one tilemap word per cell, 20h words per row and 400h words per page.

use canonical::Canonical;

use crate::rom::{AddressError, Lorom, RomAddress};
use crate::room::Area;

/// The map tilemaps' long pointers, one per area (`$90:AA7C`: `$82:964A + [area] * 3`).
pub const MAP_TILEMAP_POINTERS: RomAddress = RomAddress::new(0x82, 0x964a);
/// The map station bitmaps' pointers into bank $82, one per area (`$90:A9C8`:
/// `$82:9717 + [area] * 2`).
pub const MAP_STATION_POINTERS: RomAddress = RomAddress::new(0x82, 0x9717);
/// The areas with a map: the debug area's pointers name no map.
pub const MAPPED_AREAS: [Area; 7] = [
    Area::Crateria,
    Area::Brinstar,
    Area::Norfair,
    Area::WreckedShip,
    Area::Maridia,
    Area::Tourian,
    Area::Ceres,
];
/// A map's columns: two pages of [`PAGE_COLUMNS`] side by side.
pub const MAP_COLUMNS: u8 = 64;
/// A page's columns.
pub const PAGE_COLUMNS: u8 = 32;
/// A map's rows, the top one padding that no room is on.
pub const MAP_ROWS: u8 = 32;
/// A map bitmap's bytes: eight cells each in [`LayoutIndex`] order, the first cell the most
/// significant bit.
pub const MAP_BIT_BYTES: usize = 0x100;
/// A map tilemap's words: one per cell in [`LayoutIndex`] order.
pub const MAP_TILEMAP_WORDS: usize = 0x800;

/// A half of an area's map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    /// Columns 0 to 31.
    Left,
    /// Columns 32 to 63.
    Right,
}

/// A cell of an area's map: column `x` of [`MAP_COLUMNS`] and row `y` of [`MAP_ROWS`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapCell {
    /// The column, from the left page's left edge.
    pub x: u8,
    /// The row, from the top padding row.
    pub y: u8,
}

impl MapCell {
    /// The cell's place in [`LayoutIndex`] order.
    #[must_use]
    pub fn layout(self) -> LayoutIndex {
        let page = if self.x < PAGE_COLUMNS {
            Page::Left
        } else {
            Page::Right
        };
        LayoutIndex::on_page(page, u16::from(self.y), self.x % PAGE_COLUMNS)
    }
}

/// A place in the order the original lays a map out, its bitmaps and its tilemap alike: the
/// left page's rows top to bottom, then the right page's, [`PAGE_COLUMNS`] cells a row.
///
/// The original steps through a map in this order rather than by column and row, so a step
/// past a row's end lands at the start of the row below, and a row past the left page's
/// bottom is the right page's top. A place before the map or past it holds no cell; the
/// mini-map reads such places at the map's edges (docs/LEGACY.md, the mini-map's edges).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutIndex(pub i32);

impl LayoutIndex {
    /// The places a row takes.
    pub const ROW: i32 = 32;
    /// The places a page takes.
    pub const PAGE: i32 = 0x400;
    /// The places the map takes.
    pub const MAP: i32 = 0x800;

    /// Column `column` of row `row` on `page`; a row past [`MAP_ROWS`] runs on in layout
    /// order.
    #[must_use]
    pub fn on_page(page: Page, row: u16, column: u8) -> Self {
        let page_start = match page {
            Page::Left => 0,
            Page::Right => Self::PAGE,
        };
        Self(
            i32::from(row)
                .wrapping_mul(Self::ROW)
                .wrapping_add(page_start)
                .wrapping_add(i32::from(column)),
        )
    }

    /// The place `places` on in layout order, or back for a negative count.
    #[must_use]
    pub const fn add(self, places: i32) -> Self {
        Self(self.0.wrapping_add(places))
    }

    /// The place `places` back in layout order.
    #[must_use]
    pub const fn sub(self, places: i32) -> Self {
        Self(self.0.wrapping_sub(places))
    }

    /// The place `rows` rows on, the same column.
    #[must_use]
    pub const fn rows(self, rows: i32) -> Self {
        self.add(rows.wrapping_mul(Self::ROW))
    }

    /// The place reduced into the first `places` places, as a fixed-width index of the
    /// original wraps.
    #[must_use]
    pub const fn wrapped(self, places: i32) -> Self {
        match self.0.checked_rem_euclid(places) {
            Some(index) => Self(index),
            None => self,
        }
    }

    /// Whether the place starts a row.
    #[must_use]
    pub const fn starts_row(self) -> bool {
        matches!(self.0.checked_rem_euclid(Self::ROW), Some(0))
    }

    /// The cell at this place, when it is on the map.
    #[must_use]
    pub fn cell(self) -> Option<MapCell> {
        if !(0..Self::MAP).contains(&self.0) {
            return None;
        }
        let page = u8::try_from(self.0 / Self::PAGE).ok()?;
        let row = u8::try_from(self.0 % Self::PAGE / Self::ROW).ok()?;
        let column = u8::try_from(self.0 % Self::ROW).ok()?;
        Some(MapCell {
            x: page.wrapping_mul(PAGE_COLUMNS).wrapping_add(column),
            y: row,
        })
    }

    fn index(self) -> Option<usize> {
        usize::try_from(self.0).ok()
    }
}

/// A set of a map's cells: the cells explored, or the cells a map station reveals.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapCells([u64; 32]);

impl MapCells {
    /// Whether `cell` is in the set.
    #[must_use]
    pub fn contains(&self, cell: MapCell) -> bool {
        self.0
            .get(usize::from(cell.y))
            .is_some_and(|row| row.checked_shr(u32::from(cell.x)).unwrap_or(0) & 1 != 0)
    }

    /// Whether the cell at `at` is in the set; a place off the map holds none.
    #[must_use]
    pub fn contains_at(&self, at: LayoutIndex) -> bool {
        at.cell().is_some_and(|cell| self.contains(cell))
    }

    /// Add `cell`; a cell off the map is left out.
    pub fn insert(&mut self, cell: MapCell) {
        if let (Some(row), Some(bit)) = (
            self.0.get_mut(usize::from(cell.y)),
            1_u64.checked_shl(u32::from(cell.x)),
        ) {
            *row |= bit;
        }
    }

    /// The set a bitmap in the original's layout holds.
    #[must_use]
    pub fn from_bitmap(bitmap: &[u8; MAP_BIT_BYTES]) -> Self {
        let mut cells = Self::default();
        for (cell, set) in layout_order().zip(bitmap.iter().flat_map(|&byte| bits(byte))) {
            if set {
                cells.insert(cell);
            }
        }
        cells
    }

    /// The set as a bitmap in the original's layout.
    #[must_use]
    pub fn to_bitmap(&self) -> [u8; MAP_BIT_BYTES] {
        let mut bitmap = [0; MAP_BIT_BYTES];
        let mut cells = layout_order();
        for byte in &mut bitmap {
            for cell in cells.by_ref().take(8) {
                *byte = (*byte << 1) | u8::from(self.contains(cell));
            }
        }
        bitmap
    }
}

/// Every cell of a map in [`LayoutIndex`] order.
fn layout_order() -> impl Iterator<Item = MapCell> {
    (0..LayoutIndex::MAP).filter_map(|index| LayoutIndex(index).cell())
}

/// A bitmap byte's eight cells, the most significant bit first.
fn bits(byte: u8) -> impl Iterator<Item = bool> {
    (0..8).rev().map(move |bit| (byte >> bit) & 1 != 0)
}

/// One area's map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AreaMap {
    /// The area.
    pub area: Area,
    /// The map tilemap's words in [`LayoutIndex`] order, as the PPU reads them.
    pub tiles: Vec<u16>,
    /// The cells the area's map station reveals.
    pub station: MapCells,
}

impl AreaMap {
    /// The tilemap word at `at`, when it is on the map.
    #[must_use]
    pub fn tile(&self, at: LayoutIndex) -> Option<u16> {
        self.tiles.get(at.index()?).copied()
    }
}

/// Every area's map, from the ROM.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AreaMaps(pub Vec<AreaMap>);

impl AreaMaps {
    /// The maps of [`MAPPED_AREAS`] from the ROM.
    ///
    /// # Errors
    ///
    /// [`AddressError`] when a pointer table, a tilemap or a bitmap is outside the image.
    pub fn load(rom: Lorom<'_>) -> Result<Self, AddressError> {
        let mut maps = Vec::with_capacity(MAPPED_AREAS.len());
        for (index, area) in (0_usize..).zip(MAPPED_AREAS) {
            let long = read_array::<3>(rom, MAP_TILEMAP_POINTERS, index.wrapping_mul(3))?;
            let tilemap = RomAddress::from_long(long);
            let words = rom.read(tilemap, MAP_TILEMAP_WORDS.wrapping_mul(2))?;
            let tiles = words
                .as_chunks::<2>()
                .0
                .iter()
                .map(|&word| u16::from_le_bytes(word))
                .collect();
            let near = read_array::<2>(rom, MAP_STATION_POINTERS, index.wrapping_mul(2))?;
            let bitmap = RomAddress::new(MAP_STATION_POINTERS.bank, u16::from_le_bytes(near));
            let bytes: &[u8; MAP_BIT_BYTES] = rom
                .read(bitmap, MAP_BIT_BYTES)?
                .first_chunk()
                .ok_or(AddressError::PastEnd(bitmap))?;
            maps.push(AreaMap {
                area,
                tiles,
                station: MapCells::from_bitmap(bytes),
            });
        }
        Ok(Self(maps))
    }

    /// `area`'s map, when it has one.
    #[must_use]
    pub fn of(&self, area: Area) -> Option<&AreaMap> {
        self.0.iter().find(|map| map.area == area)
    }
}

/// The `N` bytes `offset` bytes into the table at `table`.
fn read_array<const N: usize>(
    rom: Lorom<'_>,
    table: RomAddress,
    offset: usize,
) -> Result<[u8; N], AddressError> {
    let at = table.advance(offset).ok_or(AddressError::PastEnd(table))?;
    rom.read(at, N)?
        .first_chunk()
        .copied()
        .ok_or(AddressError::PastEnd(at))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom::Rom;

    /// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
    fn rom() -> Option<Rom> {
        let path = std::env::var_os("SM_ROM").map_or_else(
            || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
            std::path::PathBuf::from,
        );
        let image = std::fs::read(path).ok()?;
        Rom::new(image).ok()
    }

    #[test]
    fn layout_order_runs_rows_then_pages() {
        let cell = |x, y| Some(MapCell { x, y });
        assert_eq!(LayoutIndex(0).cell(), cell(0, 0));
        assert_eq!(LayoutIndex(31).add(1).cell(), cell(0, 1));
        assert_eq!(LayoutIndex(0).rows(32).cell(), cell(32, 0));
        assert_eq!(LayoutIndex(LayoutIndex::MAP - 1).cell(), cell(63, 31));
        assert_eq!(LayoutIndex(-1).cell(), None);
        assert_eq!(LayoutIndex(LayoutIndex::MAP).cell(), None);
        for index in 0..LayoutIndex::MAP {
            let at = LayoutIndex(index);
            assert_eq!(at.cell().map(MapCell::layout), Some(at));
        }
    }

    #[test]
    fn a_cell_set_round_trips_the_bitmap() {
        let bitmap: [u8; MAP_BIT_BYTES] =
            core::array::from_fn(|index| u8::try_from(index).unwrap().wrapping_mul(37));
        let cells = MapCells::from_bitmap(&bitmap);
        assert_eq!(cells.to_bitmap(), bitmap);
        // Byte 81h's most significant bit is the right page's top row, column 8.
        let mut one = [0; MAP_BIT_BYTES];
        one[0x81] = 0x80;
        let cells = MapCells::from_bitmap(&one);
        assert!(cells.contains(MapCell { x: 40, y: 0 }));
        assert!(cells.contains_at(LayoutIndex(0x408)));
        assert!(!cells.contains_at(LayoutIndex(-1)));
    }

    #[test]
    fn every_mapped_area_loads_from_the_rom() {
        let Some(cartridge) = rom() else {
            return;
        };
        let maps = AreaMaps::load(cartridge.lorom()).unwrap();
        assert_eq!(maps.0.len(), MAPPED_AREAS.len());
        for map in &maps.0 {
            assert_eq!(map.tiles.len(), MAP_TILEMAP_WORDS);
            // Every map's top row is padding: no room is on it, so no station reveals it.
            assert!((0..MAP_COLUMNS).all(|x| !map.station.contains(MapCell { x, y: 0 })));
        }
        assert!(maps.of(Area::Debug).is_none());
        // Brinstar's map station reveals cells; the map tiles are in bank $B5.
        let brinstar = maps.of(Area::Brinstar).unwrap();
        assert_ne!(brinstar.station, MapCells::default());
        assert!(brinstar.tiles.iter().any(|&word| word & 0x03ff != 0x001f));
    }
}
