//! The importer: the world's rooms from the ROM, the only code that knows where the original
//! keeps each registered room.
//!
//! It decodes each room's header, its states and its door list (`data::room`) and writes the
//! records of `data::assets::world`, each room under the id the registry gives it.

use data::assets::world::rooms::{
    CERES_ELEVATOR_ROOM, CERES_RIDLEY_ROOM, DEAD_SCIENTIST_ROOM, ESCAPE_ROOM_58, FALLING_TILE_ROOM,
    MAGNET_STAIRS_ROOM, MORPH_BALL_ROOM,
};
use data::assets::world::{Conditional, MapPlace, Room, Version, World};
use data::id::RoomId;
use data::rom::Lorom;
use data::room::{self, RoomError, RoomKey, RoomState};

/// The original's place for each registered room: its header's pointer in the room bank.
const PLACES: [(RoomId, u16); 7] = [
    (MORPH_BALL_ROOM, 0x9e9f),
    (CERES_ELEVATOR_ROOM, 0xdf45),
    (FALLING_TILE_ROOM, 0xdf8d),
    (MAGNET_STAIRS_ROOM, 0xdfd7),
    (DEAD_SCIENTIST_ROOM, 0xe021),
    (ESCAPE_ROOM_58, 0xe06b),
    (CERES_RIDLEY_ROOM, 0xe0b5),
];

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
}

/// Where the original keeps room `id`: its header, the key the decoders and the work RAM's
/// room pointer name it by (verification only, K1).
#[must_use]
pub fn key(id: RoomId) -> Option<RoomKey> {
    PLACES
        .iter()
        .find_map(|&(known, pointer)| (known == id).then(|| RoomKey::from_room_pointer(pointer)))
}

/// The rooms `ids`, in order, from `rom`.
///
/// # Errors
///
/// [`WorldError`] if an id isn't registered or repeats, or its room doesn't decode.
pub fn import(rom: Lorom<'_>, ids: &[RoomId]) -> Result<World, WorldError> {
    let rooms = ids
        .iter()
        .map(|&id| {
            let key = key(id).ok_or(WorldError::Unregistered(id))?;
            let decoded =
                room::decode(rom, key).map_err(|error| WorldError::Room { room: id, error })?;
            Ok(from_decoded(id, decoded))
        })
        .collect::<Result<Vec<_>, _>>()?;
    World::new(rooms).map_err(WorldError::Repeated)
}

/// The record of the decoded room `decoded`, under `id`.
fn from_decoded(id: RoomId, decoded: room::Room) -> Room {
    Room {
        id,
        place: MapPlace {
            area: decoded.area,
            x: decoded.map_position.x,
            y: decoded.map_position.y,
        },
        size: decoded.size,
        scrollers: decoded.scrollers,
        doors: decoded.doors.doors,
        conditional: decoded
            .conditional
            .into_iter()
            .map(|entry| Conditional {
                condition: entry.condition,
                version: version(entry.state),
            })
            .collect(),
        default: version(decoded.default),
    }
}

/// The version a decoded state loads.
fn version(state: RoomState) -> Version {
    Version { plms: state.plms }
}

#[cfg(test)]
mod tests;
