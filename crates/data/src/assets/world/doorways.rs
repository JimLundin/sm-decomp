//! The doorway registry: our id and name for each doorway a loaded room's door leads through
//! (docs/architecture/assets.md §1 point 1).
//!
//! A doorway is one way into a room: leaving a room through a door is arriving through the
//! door's doorway (D15). The ids are ours: the importer (`import::world`) maps the original's
//! places to them, and the registry holds no ROM bytes. They are dense (D15): a doorway's id is
//! its place in [`DOORWAYS`]. The names say which door of which room leads through it, as labels
//! for reading only.

use crate::id::DoorwayId;

/// The Morph Ball Room's door to Green Hill Zone.
pub const MORPH_BALL_TO_GREEN_HILL: DoorwayId = DoorwayId(0);
/// The Morph Ball Room's door to Construction Zone.
pub const MORPH_BALL_TO_CONSTRUCTION: DoorwayId = DoorwayId(1);
/// The Morph Ball Room's door to the Blue Brinstar Elevator Room.
pub const MORPH_BALL_TO_ELEVATOR: DoorwayId = DoorwayId(2);
/// Ceres Elevator Room's door to Falling Tile Room.
pub const ELEVATOR_TO_FALLING_TILE: DoorwayId = DoorwayId(3);
/// Falling Tile Room's door to Ceres Elevator Room.
pub const FALLING_TILE_TO_ELEVATOR: DoorwayId = DoorwayId(4);
/// Falling Tile Room's door to Magnet Stairs Room.
pub const FALLING_TILE_TO_MAGNET_STAIRS: DoorwayId = DoorwayId(5);
/// Magnet Stairs Room's door to Falling Tile Room.
pub const MAGNET_STAIRS_TO_FALLING_TILE: DoorwayId = DoorwayId(6);
/// Magnet Stairs Room's door to Dead Scientist Room.
pub const MAGNET_STAIRS_TO_DEAD_SCIENTIST: DoorwayId = DoorwayId(7);
/// Dead Scientist Room's door to Magnet Stairs Room.
pub const DEAD_SCIENTIST_TO_MAGNET_STAIRS: DoorwayId = DoorwayId(8);
/// Dead Scientist Room's door to 58 Escape Room.
pub const DEAD_SCIENTIST_TO_ESCAPE_58: DoorwayId = DoorwayId(9);
/// 58 Escape Room's door to Dead Scientist Room.
pub const ESCAPE_58_TO_DEAD_SCIENTIST: DoorwayId = DoorwayId(10);
/// 58 Escape Room's door to Ceres Ridley's Room.
pub const ESCAPE_58_TO_RIDLEY: DoorwayId = DoorwayId(11);
/// Ceres Ridley's Room's door to 58 Escape Room.
pub const RIDLEY_TO_ESCAPE_58: DoorwayId = DoorwayId(12);

/// A registered doorway: its id and the name a reader knows it by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Our id.
    pub id: DoorwayId,
    /// Its name.
    pub name: &'static str,
}

/// Every registered doorway, in id order.
pub const DOORWAYS: [Entry; 13] = [
    Entry {
        id: MORPH_BALL_TO_GREEN_HILL,
        name: "Morph Ball Room to Green Hill Zone",
    },
    Entry {
        id: MORPH_BALL_TO_CONSTRUCTION,
        name: "Morph Ball Room to Construction Zone",
    },
    Entry {
        id: MORPH_BALL_TO_ELEVATOR,
        name: "Morph Ball Room to Blue Brinstar Elevator Room",
    },
    Entry {
        id: ELEVATOR_TO_FALLING_TILE,
        name: "Ceres Elevator Room to Falling Tile Room",
    },
    Entry {
        id: FALLING_TILE_TO_ELEVATOR,
        name: "Falling Tile Room to Ceres Elevator Room",
    },
    Entry {
        id: FALLING_TILE_TO_MAGNET_STAIRS,
        name: "Falling Tile Room to Magnet Stairs Room",
    },
    Entry {
        id: MAGNET_STAIRS_TO_FALLING_TILE,
        name: "Magnet Stairs Room to Falling Tile Room",
    },
    Entry {
        id: MAGNET_STAIRS_TO_DEAD_SCIENTIST,
        name: "Magnet Stairs Room to Dead Scientist Room",
    },
    Entry {
        id: DEAD_SCIENTIST_TO_MAGNET_STAIRS,
        name: "Dead Scientist Room to Magnet Stairs Room",
    },
    Entry {
        id: DEAD_SCIENTIST_TO_ESCAPE_58,
        name: "Dead Scientist Room to 58 Escape Room",
    },
    Entry {
        id: ESCAPE_58_TO_DEAD_SCIENTIST,
        name: "58 Escape Room to Dead Scientist Room",
    },
    Entry {
        id: ESCAPE_58_TO_RIDLEY,
        name: "58 Escape Room to Ceres Ridley's Room",
    },
    Entry {
        id: RIDLEY_TO_ESCAPE_58,
        name: "Ceres Ridley's Room to 58 Escape Room",
    },
];

/// The name of doorway `id`, if it is registered.
#[must_use]
pub fn name(id: DoorwayId) -> Option<&'static str> {
    DOORWAYS.get(usize::from(id.0)).map(|entry| entry.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_doorway_has_its_place_as_its_id_and_a_name_of_its_own() {
        for (place, entry) in DOORWAYS.iter().enumerate() {
            assert_eq!(usize::from(entry.id.0), place);
            assert_eq!(name(entry.id), Some(entry.name));
            assert!(
                DOORWAYS
                    .iter()
                    .take(place)
                    .all(|other| other.name != entry.name)
            );
        }
        assert_eq!(name(DoorwayId(13)), None);
    }
}
