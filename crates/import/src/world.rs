//! The importer: the world's rooms from the ROM, the only code that knows where the original
//! keeps each registered room.
//!
//! It decodes each room's header, its states and its door list (`data::room`) and writes the
//! records of `data::assets::world`, each room under the id the registry gives it. Where the
//! original names other content by its place (a door's destination, the doorway a condition
//! asks about), the importer names it by our id, and refuses a place no id stands for.

use data::assets::world::rooms::{
    BLUE_BRINSTAR_ELEVATOR_ROOM, CERES_ELEVATOR_ROOM, CERES_RIDLEY_ROOM, CONSTRUCTION_ZONE,
    DEAD_SCIENTIST_ROOM, ESCAPE_ROOM_58, FALLING_TILE_ROOM, GREEN_HILL_ZONE, MAGNET_STAIRS_ROOM,
    MORPH_BALL_ROOM,
};
use data::assets::world::{self, Conditional, DoorwayId, Item, MapPlace, Room, Version, World};
use data::id::RoomId;
use data::rom::{Lorom, RomAddress};
use data::room::{self, BossBits, RoomError, RoomKey, RoomState};

/// The original's place for each registered room: its header's pointer in the room bank.
const PLACES: [(RoomId, u16); 10] = [
    (MORPH_BALL_ROOM, 0x9e9f),
    (CERES_ELEVATOR_ROOM, 0xdf45),
    (FALLING_TILE_ROOM, 0xdf8d),
    (MAGNET_STAIRS_ROOM, 0xdfd7),
    (DEAD_SCIENTIST_ROOM, 0xe021),
    (ESCAPE_ROOM_58, 0xe06b),
    (CERES_RIDLEY_ROOM, 0xe0b5),
    (GREEN_HILL_ZONE, 0x9e52),
    (CONSTRUCTION_ZONE, 0x9f11),
    (BLUE_BRINSTAR_ELEVATOR_ROOM, 0x97b5),
];

/// The original's place for each registered doorway: its door header. None is registered yet,
/// as no imported room has a version its entrance chooses.
const DOORWAYS: [(DoorwayId, RomAddress); 0] = [];

/// The rooms slice 0 loads.
pub const SLICE_ZERO: [RoomId; 1] = [MORPH_BALL_ROOM];

/// Why the world doesn't import.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldError {
    /// The registry has no room with this id.
    Unregistered(RoomId),
    /// The list names a room twice.
    Repeated(RoomId),
    /// A room doesn't decode.
    Room {
        /// The room.
        room: RoomId,
        /// Why.
        error: RoomError,
    },
    /// A door of the room leads to a room the registry has no id for.
    UnregisteredDestination {
        /// The room.
        room: RoomId,
        /// The door's place in the room's door list.
        door: usize,
    },
    /// A door of the room has its header in work RAM, so it leads wherever the RAM says.
    DoorHeaderInRam {
        /// The room.
        room: RoomId,
        /// The door's place in the room's door list.
        door: usize,
    },
    /// A version of the room is chosen by a doorway the registry has no id for.
    UnregisteredDoorway(RoomId),
    /// A version of the room is chosen by a boss bit that names no boss kind.
    UnknownBoss(RoomId),
}

/// Where the original keeps room `id`: its header, the key the decoders and the work RAM's
/// room pointer name it by (verification only, K1).
#[must_use]
pub fn key(id: RoomId) -> Option<RoomKey> {
    PLACES
        .iter()
        .find_map(|&(known, pointer)| (known == id).then(|| RoomKey::from_room_pointer(pointer)))
}

/// The registered room whose header is at `key`.
#[must_use]
pub fn id(key: RoomKey) -> Option<RoomId> {
    PLACES
        .iter()
        .find_map(|&(id, pointer)| (RoomKey::from_room_pointer(pointer) == key).then_some(id))
}

/// The rooms `ids`, in order, from `rom`.
///
/// # Errors
///
/// [`WorldError`] if an id isn't registered or repeats, or its room doesn't decode or names a
/// room, a doorway or a boss we have no id for.
pub fn import(rom: Lorom<'_>, ids: &[RoomId]) -> Result<World, WorldError> {
    let rooms = ids
        .iter()
        .map(|&id| {
            let key = key(id).ok_or(WorldError::Unregistered(id))?;
            let decoded =
                room::decode(rom, key).map_err(|error| WorldError::Room { room: id, error })?;
            from_decoded(id, decoded)
        })
        .collect::<Result<Vec<_>, _>>()?;
    World::new(rooms).map_err(WorldError::Repeated)
}

/// The record of the decoded room `decoded`, under `id`.
fn from_decoded(id: RoomId, decoded: room::Room) -> Result<Room, WorldError> {
    let doors = decoded
        .doors
        .doors
        .iter()
        .enumerate()
        .map(|(place, &door)| match door {
            room::Door::ToRoom(key) => {
                self::id(key)
                    .map(world::Door::To)
                    .ok_or(WorldError::UnregisteredDestination {
                        room: id,
                        door: place,
                    })
            }
            room::Door::Nowhere => Ok(world::Door::Nowhere),
            room::Door::HeaderInRam(_) => Err(WorldError::DoorHeaderInRam {
                room: id,
                door: place,
            }),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let conditional = decoded
        .conditional
        .into_iter()
        .map(|entry| {
            Ok(Conditional {
                condition: condition(id, entry.condition)?,
                version: version(entry.state),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Room {
        id,
        place: MapPlace {
            area: decoded.area,
            x: decoded.map_position.x,
            y: decoded.map_position.y,
        },
        size: decoded.size,
        scrollers: decoded.scrollers,
        doors,
        conditional,
        default: version(decoded.default),
    })
}

/// The record's condition for room `room`'s decoded `condition`.
fn condition(room: RoomId, condition: room::Condition) -> Result<world::Condition, WorldError> {
    let defeated = |bits: BossBits| {
        bits.kinds()
            .map(world::Condition::Defeated)
            .ok_or(WorldError::UnknownBoss(room))
    };
    match condition {
        room::Condition::Door(at) => DOORWAYS
            .iter()
            .find_map(|&(doorway, known)| (known == at).then_some(doorway))
            .map(world::Condition::EnteredBy)
            .ok_or(WorldError::UnregisteredDoorway(room)),
        room::Condition::MainAreaBossDead => defeated(BossBits::MAIN_BOSS),
        room::Condition::Event(event) => Ok(world::Condition::Happened(event)),
        room::Condition::BossDead(bits) => defeated(bits),
        room::Condition::MorphBall => Ok(world::Condition::Has(Item::MorphBall)),
        room::Condition::MorphBallAndMissiles => Ok(world::Condition::MorphBallAndMissiles),
        room::Condition::PowerBombs => Ok(world::Condition::PowerBombs),
        room::Condition::SpeedBooster => Ok(world::Condition::Has(Item::SpeedBooster)),
    }
}

/// The version a decoded state loads.
fn version(state: RoomState) -> Version {
    Version { plms: state.plms }
}

#[cfg(test)]
mod tests;
