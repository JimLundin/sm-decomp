//! The room registry: our id and name for each room (docs/architecture/assets.md §1 point 1).
//!
//! The ids are ours: the importer (`import::world`) maps the original's places to them, and the
//! registry holds no ROM bytes. They are dense (D15): a room's id is its place in [`ROOMS`]. The
//! registry may name rooms the engine doesn't load yet, such as the rooms a loaded room's doors
//! lead to; a world holds the rooms it was imported with. The names are sm-json-data's, as labels for reading only.

use crate::id::RoomId;

/// The Morph Ball Room, in Brinstar.
pub const MORPH_BALL_ROOM: RoomId = RoomId(0);
/// Ceres's first room, where the elevator comes down.
pub const CERES_ELEVATOR_ROOM: RoomId = RoomId(1);
/// Ceres's second room.
pub const FALLING_TILE_ROOM: RoomId = RoomId(2);
/// Ceres's third room.
pub const MAGNET_STAIRS_ROOM: RoomId = RoomId(3);
/// Ceres's fourth room.
pub const DEAD_SCIENTIST_ROOM: RoomId = RoomId(4);
/// Ceres's fifth room.
pub const ESCAPE_ROOM_58: RoomId = RoomId(5);
/// Ceres's last room, where Ridley holds the baby.
pub const CERES_RIDLEY_ROOM: RoomId = RoomId(6);
/// A Brinstar room a door of the Morph Ball Room leads to.
pub const GREEN_HILL_ZONE: RoomId = RoomId(7);
/// The other Brinstar room a door of the Morph Ball Room leads to.
pub const CONSTRUCTION_ZONE: RoomId = RoomId(8);
/// The Crateria room a door of the Morph Ball Room leads to.
pub const BLUE_BRINSTAR_ELEVATOR_ROOM: RoomId = RoomId(9);

/// A registered room: its id and the name a reader knows it by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Our id.
    pub id: RoomId,
    /// Its name.
    pub name: &'static str,
}

/// Every registered room, in id order.
pub const ROOMS: [Entry; 10] = [
    Entry {
        id: MORPH_BALL_ROOM,
        name: "Morph Ball Room",
    },
    Entry {
        id: CERES_ELEVATOR_ROOM,
        name: "Ceres Elevator Room",
    },
    Entry {
        id: FALLING_TILE_ROOM,
        name: "Falling Tile Room",
    },
    Entry {
        id: MAGNET_STAIRS_ROOM,
        name: "Magnet Stairs Room",
    },
    Entry {
        id: DEAD_SCIENTIST_ROOM,
        name: "Dead Scientist Room",
    },
    Entry {
        id: ESCAPE_ROOM_58,
        name: "58 Escape Room",
    },
    Entry {
        id: CERES_RIDLEY_ROOM,
        name: "Ceres Ridley's Room",
    },
    Entry {
        id: GREEN_HILL_ZONE,
        name: "Green Hill Zone",
    },
    Entry {
        id: CONSTRUCTION_ZONE,
        name: "Construction Zone",
    },
    Entry {
        id: BLUE_BRINSTAR_ELEVATOR_ROOM,
        name: "Blue Brinstar Elevator Room",
    },
];

/// The name of room `id`, if it is registered.
#[must_use]
pub fn name(id: RoomId) -> Option<&'static str> {
    ROOMS.get(usize::from(id.0)).map(|entry| entry.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_room_has_its_place_as_its_id_and_a_name_of_its_own() {
        for (place, entry) in ROOMS.iter().enumerate() {
            assert_eq!(usize::from(entry.id.0), place);
            assert_eq!(name(entry.id), Some(entry.name));
            assert!(
                ROOMS
                    .iter()
                    .take(place)
                    .all(|other| other.name != entry.name)
            );
        }
        assert_eq!(name(RoomId(10)), None);
    }
}
