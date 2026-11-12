//! The world in our own format (docs/architecture/assets.md §3.1).
//!
//! The rooms by our [`RoomId`], each with its place on the map, its size, its scrollers, its
//! doors and its versions, the conditional ones in the order they are checked, then the
//! default.
//!
//! `import::world` builds it from the ROM; the engine reads only this. It holds what the
//! engine reads of a room today: a version's PLMs, and the room's doors. A record names other
//! content by our ids, never by where the original keeps it.

pub mod rooms;

pub use types::{BossKind, Event, Item, Set};

use crate::id::RoomId;
use crate::plm::PlmPopulation;
use crate::room::{Area, Screens, Scrollers};

/// A doorway: one way into a room, by our id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DoorwayId(pub u16);

/// Where a door of a room's door list leads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Door {
    /// To this room: a door block of this door lets her through, and her passage there starts.
    To(RoomId),
    /// Nowhere: an elevator's doorway, whose door block is solid to her.
    Nowhere,
}

/// What chooses a conditional version.
///
/// It says what progress's own condition (`run_progress::Condition`) says, in the same terms,
/// so the two can become one type when the records can name progress's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    /// She came in through this doorway.
    EnteredBy(DoorwayId),
    /// The event has happened.
    Happened(Event),
    /// A boss of any of these kinds is defeated in the room's own area.
    Defeated(Set<BossKind>),
    /// The item is hers.
    Has(Item),
    /// The Morph Ball is hers and her missile capacity isn't 0.
    MorphBallAndMissiles,
    /// Her power bomb capacity isn't 0.
    PowerBombs,
}

/// Where a room sits on its area's map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapPlace {
    /// The area.
    pub area: Area,
    /// The room's left column, in map cells.
    pub x: u8,
    /// The room's top row, in map cells.
    pub y: u8,
}

/// One version of a room: what loads when its condition chooses it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    /// The PLMs it places, and the scroll changes they make.
    pub plms: PlmPopulation,
}

/// A version a condition chooses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conditional {
    /// The condition.
    pub condition: Condition,
    /// The version it chooses.
    pub version: Version,
}

/// A room.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    /// Our id.
    pub id: RoomId,
    /// Where it sits on the map.
    pub place: MapPlace,
    /// Its size.
    pub size: Screens,
    /// Its vertical scrollers.
    pub scrollers: Scrollers,
    /// Its doors, by the place its door blocks name.
    pub doors: Vec<Door>,
    /// The conditional versions, in the order they are checked.
    pub conditional: Vec<Conditional>,
    /// The version that loads when no condition holds.
    pub default: Version,
}

impl Room {
    /// The version at `place` in the room's list, the default after the conditional ones.
    #[must_use]
    pub fn version(&self, place: usize) -> Option<&Version> {
        self.conditional
            .get(place)
            .map(|entry| &entry.version)
            .or_else(|| (place == self.conditional.len()).then_some(&self.default))
    }
}

/// The rooms, each once.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct World {
    rooms: Vec<Room>,
}

impl World {
    /// The world of `rooms`.
    ///
    /// # Errors
    ///
    /// The id of the first room an earlier one shares its id with.
    pub fn new(rooms: Vec<Room>) -> Result<Self, RoomId> {
        let repeated = rooms.iter().enumerate().find_map(|(place, room)| {
            rooms
                .iter()
                .take(place)
                .any(|earlier| earlier.id == room.id)
                .then_some(room.id)
        });
        repeated.map_or(Ok(Self { rooms }), Err)
    }

    /// Room `id`, if the world holds it.
    #[must_use]
    pub fn room(&self, id: RoomId) -> Option<&Room> {
        self.rooms.iter().find(|room| room.id == id)
    }

    /// Each room, in the order it was imported.
    pub fn rooms(&self) -> impl Iterator<Item = &Room> {
        self.rooms.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn room(id: RoomId) -> Room {
        let version = || Version {
            plms: PlmPopulation {
                entries: Vec::new(),
                scroll_changes: Vec::new(),
            },
        };
        Room {
            id,
            place: MapPlace {
                area: Area::Ceres,
                x: 1,
                y: 2,
            },
            size: Screens {
                width: 1,
                height: 1,
            },
            scrollers: Scrollers {
                up: 0x70,
                down: 0xa0,
            },
            doors: vec![Door::Nowhere, Door::To(rooms::FALLING_TILE_ROOM)],
            conditional: vec![Conditional {
                condition: Condition::PowerBombs,
                version: version(),
            }],
            default: version(),
        }
    }

    #[test]
    fn a_room_lists_its_conditional_versions_then_its_default() {
        let room = room(rooms::CERES_RIDLEY_ROOM);
        let is = |place: usize, version: &Version| {
            room.version(place)
                .is_some_and(|found| core::ptr::eq(found, version))
        };
        assert!(is(0, &room.conditional[0].version));
        assert!(is(1, &room.default));
        assert_eq!(room.version(2), None);
    }

    #[test]
    fn a_world_holds_each_room_once_by_its_id() {
        let ceres = [rooms::CERES_ELEVATOR_ROOM, rooms::FALLING_TILE_ROOM];
        let world = World::new(ceres.map(room).to_vec()).unwrap();
        assert_eq!(
            world.room(rooms::FALLING_TILE_ROOM).map(|room| room.id),
            Some(rooms::FALLING_TILE_ROOM)
        );
        assert_eq!(world.room(rooms::MORPH_BALL_ROOM), None);
        assert_eq!(
            World::new(vec![
                room(rooms::MORPH_BALL_ROOM),
                room(rooms::MORPH_BALL_ROOM)
            ]),
            Err(rooms::MORPH_BALL_ROOM)
        );
    }
}
