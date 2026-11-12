//! The tileset registry: our id and name for each tileset a loaded room's version uses
//! (docs/architecture/assets.md §1 point 1).
//!
//! The ids are ours: the importer (`import::world`) maps the original's tileset places to them,
//! and the registry holds no ROM bytes. They are dense (D15): a tileset's id is its place in
//! [`TILESETS`]. The names say where a tileset is seen, as labels for reading only.

use crate::id::TilesetId;

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
