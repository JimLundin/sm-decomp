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
/// A map bitmap's bytes: two pages of 32 rows of four bytes.
pub const MAP_BIT_BYTES: usize = 0x100;
/// A map tilemap's words: two pages of 32 rows of 32 cells.
pub const MAP_TILEMAP_WORDS: usize = 0x800;

/// A byte of a map bitmap, by its index as the original's X register holds it.
///
/// The original computes indices one row or one byte outside the bitmap at its edges, so an
/// index is any word, and [`MapBits::byte`] answers only for the bitmap's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapByte(pub u16);

impl MapByte {
    /// The index `bytes` further on, wrapping as the original's 16-bit index does.
    #[must_use]
    pub const fn add(self, bytes: u16) -> Self {
        Self(self.0.wrapping_add(bytes))
    }

    /// The index `bytes` before, wrapping as the original's 16-bit index does.
    #[must_use]
    pub const fn sub(self, bytes: u16) -> Self {
        Self(self.0.wrapping_sub(bytes))
    }

    fn index(self) -> usize {
        usize::from(self.0)
    }
}

/// A map bitmap: one bit per cell of an area's map, in the original's layout.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapBits(pub [u8; MAP_BIT_BYTES]);

impl Default for MapBits {
    fn default() -> Self {
        Self([0; MAP_BIT_BYTES])
    }
}

impl MapBits {
    /// The byte at `at`, when it is inside the bitmap.
    #[must_use]
    pub fn byte(&self, at: MapByte) -> Option<u8> {
        self.0.get(at.index()).copied()
    }

    /// Set the bits of `mask` in the byte at `at`; an index outside the bitmap is left alone.
    pub fn mark(&mut self, at: MapByte, mask: u8) {
        if let Some(byte) = self.0.get_mut(at.index()) {
            *byte |= mask;
        }
    }
}

/// One area's map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AreaMap {
    /// The area.
    pub area: Area,
    /// The map tilemap's words, as the PPU reads them.
    pub tiles: Vec<u16>,
    /// The cells the area's map station reveals.
    pub station: MapBits,
}

impl AreaMap {
    /// The tilemap word at `index`, when it is inside the map.
    #[must_use]
    pub fn tile(&self, index: usize) -> Option<u16> {
        self.tiles.get(index).copied()
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
                station: MapBits(*bytes),
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
    fn a_bitmap_answers_only_inside_it() {
        let mut bits = MapBits::default();
        bits.mark(MapByte(0xff), 0x80);
        bits.mark(MapByte(0x100), 0x80);
        assert_eq!(bits.byte(MapByte(0xff)), Some(0x80));
        assert_eq!(bits.byte(MapByte(0x100)), None);
        assert_eq!(bits.byte(MapByte(0).sub(1)), None);
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
            assert!(map.station.0.iter().take(4).all(|&byte| byte == 0));
        }
        assert!(maps.of(Area::Debug).is_none());
        // Brinstar's map station reveals cells; the map tiles are in bank $B5.
        let brinstar = maps.of(Area::Brinstar).unwrap();
        assert!(brinstar.station.0.iter().any(|&byte| byte != 0));
        assert!(brinstar.tiles.iter().any(|&word| word & 0x03ff != 0x001f));
    }
}
