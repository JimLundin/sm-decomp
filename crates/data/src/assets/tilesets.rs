//! The tilesets a loaded room's version draws with, and their registry
//! (docs/architecture/assets.md §1 point 1, §3.2).
//!
//! The ids are ours: the importers (`import::world`, `import::tileset`) map the original's
//! tileset places to them, and the registry holds no ROM bytes. They are dense (D15): a
//! tileset's id is its place in [`TILESETS`]. The names say where a tileset is seen, as labels
//! for reading only.
//!
//! A [`Tileset`] is a 4bpp sheet, its block looks and its palette rows. One that uses the
//! shared elements draws its first [`SHARED_BLOCKS`] blocks from them and its own looks after
//! them; one that doesn't draws its own from block 0.

use canonical::Canonical;

use crate::assets::art::{BlockLook, PaletteSet, TileSheet};
use crate::id::TilesetId;

/// The blocks the shared elements stand for in a tileset that uses them: its own looks start
/// after them.
pub const SHARED_BLOCKS: u16 = 0x100;

/// The common room elements, which every tileset that uses them draws its first blocks from.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct SharedElements {
    /// The looks of the first blocks, from block 0.
    pub looks: Vec<BlockLook>,
}

/// A tileset: the tiles, block looks and palette rows a room's level draws with.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Tileset {
    /// Its id.
    pub id: TilesetId,
    /// Its tiles.
    pub sheet: TileSheet,
    /// Its block looks, from its first block.
    pub looks: Vec<BlockLook>,
    /// Its palette rows.
    pub palettes: PaletteSet,
    /// Whether its first blocks are the shared elements'.
    pub uses_shared: bool,
}

/// The registered tilesets and the shared elements.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Tilesets {
    /// The shared elements.
    pub shared: SharedElements,
    /// The tilesets, in id order.
    pub sets: Vec<Tileset>,
}

impl Tilesets {
    /// Tileset `id`, if it is loaded.
    #[must_use]
    pub fn get(&self, id: TilesetId) -> Option<&Tileset> {
        self.sets.get(usize::from(id.0))
    }

    /// The look of block `number` in tileset `id`; `None` past what the tileset and the shared
    /// elements hold, where the original draws what an earlier room or the boot left
    /// (docs/LEGACY.md).
    #[must_use]
    pub fn block(&self, id: TilesetId, number: u16) -> Option<BlockLook> {
        let tileset = self.get(id)?;
        let (looks, index) = match number.checked_sub(SHARED_BLOCKS) {
            _ if !tileset.uses_shared => (&tileset.looks, number),
            Some(own) => (&tileset.looks, own),
            None => (&self.shared.looks, number),
        };
        looks.get(usize::from(index)).copied()
    }
}

/// Brinstar's, as the Morph Ball Room shows it.
pub const BRINSTAR: TilesetId = TilesetId(0);
/// Ceres's station, before Ridley flees.
pub const CERES: TilesetId = TilesetId(1);
/// Ceres's station during the escape.
pub const CERES_ESCAPE: TilesetId = TilesetId(2);
/// Ceres's elevator shaft, before Ridley flees.
pub const CERES_ELEVATOR: TilesetId = TilesetId(3);
/// Ceres's elevator shaft during the escape.
pub const CERES_ELEVATOR_ESCAPE: TilesetId = TilesetId(4);
/// Ceres Ridley's Room, before Ridley flees.
pub const CERES_RIDLEY: TilesetId = TilesetId(5);
/// Ceres Ridley's Room during the escape.
pub const CERES_RIDLEY_ESCAPE: TilesetId = TilesetId(6);

/// A registered tileset: its id and the name a reader knows it by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Our id.
    pub id: TilesetId,
    /// Its name.
    pub name: &'static str,
}

/// Every registered tileset, in id order.
pub const TILESETS: [Entry; 7] = [
    Entry {
        id: BRINSTAR,
        name: "Brinstar",
    },
    Entry {
        id: CERES,
        name: "Ceres",
    },
    Entry {
        id: CERES_ESCAPE,
        name: "Ceres, escape",
    },
    Entry {
        id: CERES_ELEVATOR,
        name: "Ceres elevator",
    },
    Entry {
        id: CERES_ELEVATOR_ESCAPE,
        name: "Ceres elevator, escape",
    },
    Entry {
        id: CERES_RIDLEY,
        name: "Ceres Ridley",
    },
    Entry {
        id: CERES_RIDLEY_ESCAPE,
        name: "Ceres Ridley, escape",
    },
];

/// The name of tileset `id`, if it is registered.
#[must_use]
pub fn name(id: TilesetId) -> Option<&'static str> {
    TILESETS.get(usize::from(id.0)).map(|entry| entry.name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::art::{Cell, TileRef};

    /// A look whose cells name the block and the looks it came from.
    fn look(source: u16, block: u16) -> BlockLook {
        let cell = Cell {
            tile: TileRef((source << 12) | block),
            ..Cell::default()
        };
        BlockLook::from_cells([cell; 4])
    }

    fn tilesets() -> Tilesets {
        let own = |id, uses_shared| Tileset {
            id,
            looks: (0..0x300).map(|block| look(2, block)).collect(),
            uses_shared,
            ..Tileset::default()
        };
        Tilesets {
            shared: SharedElements {
                looks: (0..SHARED_BLOCKS).map(|block| look(1, block)).collect(),
            },
            sets: vec![own(BRINSTAR, true), own(CERES, false)],
        }
    }

    #[test]
    fn a_tileset_with_the_shared_elements_draws_them_first_and_its_own_from_100h() {
        let tilesets = tilesets();
        assert_eq!(tilesets.block(BRINSTAR, 0xff), Some(look(1, 0xff)));
        assert_eq!(tilesets.block(BRINSTAR, 0x100), Some(look(2, 0)));
        assert_eq!(tilesets.block(BRINSTAR, 0x3ff), Some(look(2, 0x2ff)));
        assert_eq!(tilesets.block(CERES, 0x10), Some(look(2, 0x10)));
        assert_eq!(tilesets.block(CERES, 0x300), None);
        assert_eq!(tilesets.block(CERES_ESCAPE, 0), None);
    }

    #[test]
    fn each_tileset_has_its_place_as_its_id_and_a_name_of_its_own() {
        for (place, entry) in TILESETS.iter().enumerate() {
            assert_eq!(usize::from(entry.id.0), place);
            assert_eq!(name(entry.id), Some(entry.name));
            assert!(
                TILESETS
                    .iter()
                    .take(place)
                    .all(|other| other.name != entry.name)
            );
        }
        assert_eq!(name(TilesetId(7)), None);
    }
}
