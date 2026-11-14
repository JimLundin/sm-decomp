//! The importer: the world's rooms from the ROM, the only code that knows where the original
//! keeps each registered room.
//!
//! It decodes each room's header, its states and its door list (`data::room`) and writes the
//! records of `data::assets::world`, each room under the id the registry gives it. Where the
//! original names other content by its place (a door's doorway and destination, the doorway a
//! condition asks about, a version's tileset), the importer names it by our id, and refuses a
//! place no id stands for.
//!
//! It is also the only code that knows where the original keeps what it imported: the
//! verifier asks it ([`key`], [`version_places`], [`tileset_index`]) to read the work RAM's
//! pointers (K1).

use std::collections::BTreeMap;

use crate::tileset::TilesetIndex;
use data::assets::tilesets::{
    BRINSTAR, CERES, CERES_ELEVATOR, CERES_ELEVATOR_ESCAPE, CERES_ESCAPE, CERES_RIDLEY,
    CERES_RIDLEY_ESCAPE,
};
use data::assets::world::doorways::{
    DEAD_SCIENTIST_TO_ESCAPE_58, DEAD_SCIENTIST_TO_MAGNET_STAIRS, ELEVATOR_TO_FALLING_TILE,
    ESCAPE_58_TO_DEAD_SCIENTIST, ESCAPE_58_TO_RIDLEY, FALLING_TILE_TO_ELEVATOR,
    FALLING_TILE_TO_MAGNET_STAIRS, MAGNET_STAIRS_TO_DEAD_SCIENTIST, MAGNET_STAIRS_TO_FALLING_TILE,
    MORPH_BALL_TO_CONSTRUCTION, MORPH_BALL_TO_ELEVATOR, MORPH_BALL_TO_GREEN_HILL,
    RIDLEY_TO_ESCAPE_58,
};
use data::assets::world::rooms::{
    BLUE_BRINSTAR_ELEVATOR_ROOM, CERES_ELEVATOR_ROOM, CERES_RIDLEY_ROOM, CONSTRUCTION_ZONE,
    DEAD_SCIENTIST_ROOM, ESCAPE_ROOM_58, FALLING_TILE_ROOM, GREEN_HILL_ZONE, MAGNET_STAIRS_ROOM,
    MORPH_BALL_ROOM,
};
use data::assets::world::{
    self, Arrival, Back, Conditional, Doorway, Heading, Inconsistent, Item, MapPlace, Pair, Room,
    ScreenCoordinate, Version, World,
};
use data::id::{DoorwayId, RoomId, TilesetId};
use data::plm::ScrollLists;
use data::rom::{AddressError, Lorom, RomAddress};
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

/// The bank of the door headers.
const DOOR_BANK: u8 = 0x83;

/// The original's place for each registered doorway: its door header's pointer in the door
/// bank, which a door list and a doorway's condition name.
const DOORWAYS: [(DoorwayId, u16); 13] = [
    (MORPH_BALL_TO_GREEN_HILL, 0x8e9e),
    (MORPH_BALL_TO_CONSTRUCTION, 0x8eaa),
    (MORPH_BALL_TO_ELEVATOR, 0x8eb6),
    (ELEVATOR_TO_FALLING_TILE, 0xab4c),
    (FALLING_TILE_TO_ELEVATOR, 0xab58),
    (FALLING_TILE_TO_MAGNET_STAIRS, 0xab64),
    (MAGNET_STAIRS_TO_FALLING_TILE, 0xab70),
    (MAGNET_STAIRS_TO_DEAD_SCIENTIST, 0xab7c),
    (DEAD_SCIENTIST_TO_MAGNET_STAIRS, 0xab88),
    (DEAD_SCIENTIST_TO_ESCAPE_58, 0xab94),
    (ESCAPE_58_TO_DEAD_SCIENTIST, 0xaba0),
    (ESCAPE_58_TO_RIDLEY, 0xabac),
    (RIDLEY_TO_ESCAPE_58, 0xabb8),
];

/// The original's place for each registered tileset: its index in the tileset table, the byte
/// a room state names it by.
const TILESETS: [(TilesetId, TilesetIndex); 7] = [
    (BRINSTAR, TilesetIndex(6)),
    (CERES, TilesetIndex(15)),
    (CERES_ESCAPE, TilesetIndex(16)),
    (CERES_ELEVATOR, TilesetIndex(17)),
    (CERES_ELEVATOR_ESCAPE, TilesetIndex(18)),
    (CERES_RIDLEY, TilesetIndex(19)),
    (CERES_RIDLEY_ESCAPE, TilesetIndex(20)),
];

/// The rooms the engine loads: slice 0's Morph Ball Room and Ceres's six.
pub const LOADED: [RoomId; 7] = [
    MORPH_BALL_ROOM,
    CERES_ELEVATOR_ROOM,
    FALLING_TILE_ROOM,
    MAGNET_STAIRS_ROOM,
    DEAD_SCIENTIST_ROOM,
    ESCAPE_ROOM_58,
    CERES_RIDLEY_ROOM,
];

/// Why the world doesn't import.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldError {
    /// The registry has no room with this id.
    Unregistered(RoomId),
    /// The rooms and their doorways don't make a world: an id repeats.
    Inconsistent(Inconsistent),
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
    /// A door of the room leads through, or a version of it is chosen by, a doorway the
    /// registry has no id for.
    UnregisteredDoorway(RoomId),
    /// A version of the room draws from a tileset the registry has no id for.
    UnregisteredTileset(RoomId),
    /// A version of the room is chosen by a boss bit that names no boss kind.
    UnknownBoss(RoomId),
    /// A doorway's arrival record isn't fully present in the image.
    Arrival {
        /// The doorway.
        doorway: DoorwayId,
        /// The failed read.
        error: AddressError,
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

/// The registered room whose header is at `key`.
#[must_use]
pub fn id(key: RoomKey) -> Option<RoomId> {
    PLACES
        .iter()
        .find_map(|&(id, pointer)| (RoomKey::from_room_pointer(pointer) == key).then_some(id))
}

/// The original's place for the registered tileset `id`: its index in the tileset table, which
/// the work RAM's tileset and the tile tables name it by (verification only, K1).
#[must_use]
pub fn tileset_index(id: TilesetId) -> Option<TilesetIndex> {
    TILESETS
        .iter()
        .find_map(|&(known, index)| (known == id).then_some(index))
}

/// The registered doorway whose door header is at `at`.
fn doorway(at: RomAddress) -> Option<DoorwayId> {
    DOORWAYS.iter().find_map(|&(doorway, pointer)| {
        (RomAddress::new(DOOR_BANK, pointer) == at).then_some(doorway)
    })
}

/// Where the original keeps what a version of a room loads that the work RAM names by its
/// place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionPlaces {
    /// Its state header, which the work RAM's room state pointer names.
    pub state: RomAddress,
    /// Its PLMs' scroll change lists, which their room arguments name.
    pub scroll_lists: ScrollLists,
}

/// Where the original keeps each version of room `id`, in the record's order: the conditional
/// versions, then the default (verification only, K1).
///
/// # Errors
///
/// [`WorldError`] if the room isn't registered or doesn't decode.
pub fn version_places(rom: Lorom<'_>, id: RoomId) -> Result<Vec<VersionPlaces>, WorldError> {
    let decoded = decode(rom, id)?;
    Ok(decoded
        .states()
        .map(|(_, state)| VersionPlaces {
            state: state.at,
            scroll_lists: state.scroll_lists.clone(),
        })
        .collect())
}

/// Registered room `id`, decoded.
fn decode(rom: Lorom<'_>, id: RoomId) -> Result<room::Room, WorldError> {
    let key = key(id).ok_or(WorldError::Unregistered(id))?;
    room::decode(rom, key).map_err(|error| WorldError::Room { room: id, error })
}

/// The rooms `ids`, in order, from `rom`, with the doorways their doors lead through.
///
/// # Errors
///
/// [`WorldError`] if an id isn't registered or repeats, or its room doesn't decode or names a
/// room, a doorway, a tileset or a boss we have no id for.
pub fn import(rom: Lorom<'_>, ids: &[RoomId]) -> Result<World, WorldError> {
    let mut doorways = BTreeMap::new();
    let rooms = ids
        .iter()
        .map(|&id| from_decoded(rom, id, decode(rom, id)?, &mut doorways))
        .collect::<Result<Vec<_>, _>>()?;
    World::new(rooms, doorways.into_values().collect()).map_err(WorldError::Inconsistent)
}

/// The door header of the door at `place` in `list`: the list holds a word a door.
fn door_header(rom: Lorom<'_>, list: &room::DoorList, place: usize) -> Option<RomAddress> {
    let at = list.at.advance(place.checked_mul(2)?)?;
    let word = rom.read(at, 2).ok()?;
    let (pair, _) = word.as_chunks::<2>();
    Some(RomAddress::new(
        DOOR_BANK,
        u16::from_le_bytes(*pair.first()?),
    ))
}

/// The passage's content in a door header: direction, destination screens and speed in
/// 1/256 pixels per frame. A negative speed leaves the passage's own speed in use.
fn arrival(rom: Lorom<'_>, at: RomAddress) -> Result<Arrival, AddressError> {
    let [
        _,
        _,
        _,
        direction,
        _,
        _,
        screen_x,
        screen_y,
        speed_low,
        speed_high,
    ] = *<&[u8; 10]>::try_from(rom.read(at, 10)?).map_err(|_| AddressError::PastEnd(at))?;
    // Only the low two bits choose the way; the closing cap is a separate owner's flag.
    let way = match (direction & 1 != 0, direction & 2 != 0) {
        (false, false) => Heading::Right,
        (true, false) => Heading::Left,
        (false, true) => Heading::Down,
        (true, true) => Heading::Up,
    };
    let speed = i16::from_le_bytes([speed_low, speed_high]);
    Ok(Arrival {
        way,
        view: Pair {
            x: ScreenCoordinate(screen_x),
            y: ScreenCoordinate(screen_y),
        },
        speed: u32::try_from(speed)
            .ok()
            .map(|speed| core::num::Wrapping(speed << 8)),
    })
}

/// The record of the decoded room `decoded`, under `id`, adding the doorways its doors lead
/// through to `doorways`.
fn from_decoded(
    rom: Lorom<'_>,
    id: RoomId,
    decoded: room::Room,
    doorways: &mut BTreeMap<DoorwayId, Doorway>,
) -> Result<Room, WorldError> {
    let doors = decoded
        .doors
        .doors
        .iter()
        .enumerate()
        .map(|(place, &door)| match door {
            room::Door::ToRoom(key) => {
                let destination = self::id(key).ok_or(WorldError::UnregisteredDestination {
                    room: id,
                    door: place,
                })?;
                let at = door_header(rom, &decoded.doors, place)
                    .ok_or(WorldError::UnregisteredDoorway(id))?;
                let doorway = doorway(at).ok_or(WorldError::UnregisteredDoorway(id))?;
                doorways.insert(
                    doorway,
                    Doorway {
                        id: doorway,
                        room: destination,
                        arrival: arrival(rom, at)
                            .map_err(|error| WorldError::Arrival { doorway, error })?,
                    },
                );
                Ok(world::Door::To(doorway))
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
                version: version(id, entry.state)?,
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
        default: version(id, decoded.default)?,
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
        room::Condition::Door(at) => doorway(at)
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

/// The version a decoded state of room `room` loads.
fn version(room: RoomId, state: RoomState) -> Result<Version, WorldError> {
    let index = TilesetIndex(state.tileset);
    let tileset = TILESETS
        .iter()
        .find_map(|&(id, known)| (known == index).then_some(id))
        .ok_or(WorldError::UnregisteredTileset(room))?;
    let back = if state.layers.layer2.is_some() {
        Back::Level
    } else {
        Back::Background
    };
    Ok(Version {
        tileset,
        back,
        plms: state.plms,
    })
}

#[cfg(test)]
mod tests;
