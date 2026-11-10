//! Room headers and their states, decoded from ROM (D15).
//!
//! A room header is 11 bytes: index, area, map position, size in screens, the up and down
//! scrollers, the CRE bits and the door list. The state conditions follow: each is a
//! condition's code, its operands and a state header's pointer, and the list ends with the
//! default condition, whose state header follows it directly. The original checks the
//! conditions in order and loads the first state whose condition holds.
//!
//! Outside the engine a room is named by its header's ROM address, a [`RoomKey`]; inside by
//! the dense [`RoomId`] the [`RoomMap`] gives it.

use canonical::Canonical;
use clean_types::Event;

use crate::address_map::{AddressMap, Location, StatePart};
use crate::compression::{DecompressError, decompress};
use crate::id::RoomId;
use crate::level::{Level, LevelError, ScrollArea, scroll_areas};
use crate::plm::{PlmError, PlmPopulation, population};
use crate::rom::{AddressError, Lorom, RomAddress};

/// Room headers, state headers, door lists, scroll tables and PLM populations.
const ROOM_BANK: u8 = 0x8f;
/// Door headers.
const DOOR_BANK: u8 = 0x83;
/// FX lists.
const FX_BANK: u8 = 0x83;
/// Enemy populations.
const ENEMY_POPULATION_BANK: u8 = 0xa1;
/// Enemy graphics sets.
const ENEMY_SET_BANK: u8 = 0xb4;

/// A room's stable key: its header's ROM address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoomKey(pub RomAddress);

impl RoomKey {
    /// The Morph Ball Room.
    pub const MORPH_BALL_ROOM: Self = Self::from_room_pointer(0x9e9f);

    /// The room whose header the original's 16-bit room pointer names.
    #[must_use]
    pub const fn from_room_pointer(pointer: u16) -> Self {
        Self(RomAddress::new(ROOM_BANK, pointer))
    }

    /// The original's 16-bit room pointer for this room, if its header is in the room bank.
    #[must_use]
    pub fn room_pointer(self) -> Option<u16> {
        (self.0.bank == ROOM_BANK).then_some(self.0.offset)
    }
}

/// The rooms slice 0 loads.
pub const SLICE_ZERO_ROOMS: [RoomKey; 1] = [RoomKey::MORPH_BALL_ROOM];

/// The dense IDs of the loaded rooms: each room's ID is its place in the load list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomMap {
    keys: Vec<RoomKey>,
}

impl RoomMap {
    /// The map of `keys`, in order, or `None` if a key repeats or there are too many.
    #[must_use]
    pub fn new(keys: Vec<RoomKey>) -> Option<Self> {
        let distinct = keys
            .iter()
            .enumerate()
            .all(|(index, key)| !keys.iter().take(index).any(|earlier| earlier == key));
        (distinct && u16::try_from(keys.len()).is_ok()).then_some(Self { keys })
    }

    /// The map of the rooms slice 0 loads.
    #[must_use]
    pub fn slice_zero() -> Self {
        Self {
            keys: SLICE_ZERO_ROOMS.to_vec(),
        }
    }

    /// The ID of the room with `key`.
    #[must_use]
    pub fn id(&self, key: RoomKey) -> Option<RoomId> {
        self.keys
            .iter()
            .position(|&known| known == key)
            .and_then(|index| u16::try_from(index).ok())
            .map(RoomId)
    }

    /// The key of the room with `id`.
    #[must_use]
    pub fn key(&self, id: RoomId) -> Option<RoomKey> {
        self.keys.get(usize::from(id.0)).copied()
    }

    /// Each room's key and ID, in ID order.
    pub fn iter(&self) -> impl Iterator<Item = (RoomKey, RoomId)> + '_ {
        self.keys
            .iter()
            .zip(0..)
            .map(|(&key, id)| (key, RoomId(id)))
    }
}

/// The map area a room is in.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Area {
    /// Area 0.
    #[default]
    Crateria,
    /// Area 1.
    Brinstar,
    /// Area 2.
    Norfair,
    /// Area 3.
    WreckedShip,
    /// Area 4.
    Maridia,
    /// Area 5.
    Tourian,
    /// Area 6.
    Ceres,
    /// Area 7, the debug rooms.
    Debug,
}

impl TryFrom<u8> for Area {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, u8> {
        match value {
            0 => Ok(Self::Crateria),
            1 => Ok(Self::Brinstar),
            2 => Ok(Self::Norfair),
            3 => Ok(Self::WreckedShip),
            4 => Ok(Self::Maridia),
            5 => Ok(Self::Tourian),
            6 => Ok(Self::Ceres),
            7 => Ok(Self::Debug),
            other => Err(other),
        }
    }
}

impl From<Area> for u8 {
    fn from(area: Area) -> Self {
        match area {
            Area::Crateria => 0,
            Area::Brinstar => 1,
            Area::Norfair => 2,
            Area::WreckedShip => 3,
            Area::Maridia => 4,
            Area::Tourian => 5,
            Area::Ceres => 6,
            Area::Debug => 7,
        }
    }
}

/// A position on the area map, in map cells.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MapPosition {
    /// The room's left column.
    pub x: u8,
    /// The room's top row.
    pub y: u8,
}

/// A room's size in screens of 16 by 16 blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Screens {
    /// Screens across.
    pub width: u8,
    /// Screens down.
    pub height: u8,
}

impl Screens {
    /// Blocks across: 16 per screen.
    #[must_use]
    pub fn blocks_across(self) -> u16 {
        u16::from(self.width).wrapping_mul(16)
    }

    /// Blocks in the room: 256 per screen.
    #[must_use]
    pub fn blocks(self) -> usize {
        usize::from(self.width)
            .wrapping_mul(usize::from(self.height))
            .wrapping_mul(256)
    }
}

/// The header's vertical scrollers, as the original stores them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Scrollers {
    /// The up scroller.
    pub up: u8,
    /// The down scroller.
    pub down: u8,
}

/// The events by the numbers the original gives them.
///
/// A number is the event's bit in the event flags and the operand of the event state
/// condition. Events 3 to 5 count the zebetites destroyed; 1 and 16h to 3Fh have no meaning.
// The event flags are `$7E:D820` (`$80:81FA`), the condition `$8F:E612`, the zebetites' count
// `$A6:FCAA`.
pub const EVENTS: [(u8, Event); 18] = [
    (0x00, Event::PlanetAwake),
    (0x02, Event::MotherBrainTankBroken),
    (0x06, Event::PhantoonStatueGrey),
    (0x07, Event::RidleyStatueGrey),
    (0x08, Event::DraygonStatueGrey),
    (0x09, Event::KraidStatueGrey),
    (0x0a, Event::TourianOpen),
    (0x0b, Event::GlassTunnelBroken),
    (0x0c, Event::AcidReceded),
    (0x0d, Event::ShaktoolPath),
    (0x0e, Event::TimeBombSet),
    (0x0f, Event::AnimalsFreed),
    (0x10, Event::MetroidRoom1Empty),
    (0x11, Event::MetroidRoom2Empty),
    (0x12, Event::MetroidRoom3Empty),
    (0x13, Event::MetroidRoom4Empty),
    (0x14, Event::TourianRoomsUnset),
    (0x15, Event::LavaQuakeOutrun),
];

/// The event the original numbers `number`, if it names one.
#[must_use]
pub fn event(number: u8) -> Option<Event> {
    EVENTS
        .iter()
        .find_map(|&(known, event)| (known == number).then_some(event))
}

/// A set of the current area's boss bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BossBits(pub u8);

/// A state condition other than the default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Condition {
    /// The room was entered through this door.
    Door(RomAddress),
    /// The area's main boss is dead.
    MainAreaBossDead,
    /// The event is set.
    Event(Event),
    /// One of these boss bits is set for the area.
    BossDead(BossBits),
    /// The Morph Ball is collected.
    MorphBall,
    /// The Morph Ball and missiles are collected.
    MorphBallAndMissiles,
    /// Power bombs are collected.
    PowerBombs,
    /// The Speed Booster is collected.
    SpeedBooster,
}

/// A state's place in its room's list, the default last.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateIndex(pub u8);

/// The music a state plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Music {
    /// The music data set.
    pub data: u8,
    /// The track in it.
    pub track: u8,
}

/// The decompressed level data: the layer 1 size word, then layer 1, BTS and layer 2.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelData {
    /// Where the compressed stream is.
    pub at: RomAddress,
    /// The decompressed bytes.
    pub bytes: Vec<u8>,
}

impl LevelData {
    /// The leading word: layer 1's size in bytes.
    #[must_use]
    pub const fn layer1_bytes(&self) -> Option<u16> {
        let [low, high, ..] = self.bytes.as_slice() else {
            return None;
        };
        Some(u16::from_le_bytes([*low, *high]))
    }
}

/// A state header's records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomState {
    /// Where the state header is.
    pub at: RomAddress,
    /// The level data, decompressed at load.
    pub level: LevelData,
    /// The level data's blocks.
    pub layers: Level,
    /// The tileset's index.
    pub tileset: u8,
    /// The music.
    pub music: Music,
    /// The FX list.
    pub fx: RomAddress,
    /// The enemy population.
    pub enemy_population: RomAddress,
    /// The enemy graphics set.
    pub enemy_set: RomAddress,
    /// Layer 2's scroll rates, as the original stores them (`x`, `y`).
    pub layer2_scroll: [u8; 2],
    /// The scroll table.
    pub scroll: RomAddress,
    /// The scroll areas it gives, a screen each, row-major; a table's go on past the room's
    /// screens with the run after them (`scroll_areas`).
    pub scroll_areas: Vec<ScrollArea>,
    /// The special x-ray blocks.
    pub special_xray_blocks: RomAddress,
    /// The room's main code.
    pub main_code: RomAddress,
    /// The PLM population.
    pub plm_population: RomAddress,
    /// The PLMs it places.
    pub plms: PlmPopulation,
    /// The library background.
    pub library_background: RomAddress,
    /// The room's setup code.
    pub setup_code: RomAddress,
}

/// A state chosen by a condition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionalState {
    /// The condition.
    pub condition: Condition,
    /// The state it chooses.
    pub state: RoomState,
}

/// A room header and its states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    /// The room's stable key.
    pub key: RoomKey,
    /// The room's index in its area.
    pub index: u8,
    /// The area.
    pub area: Area,
    /// The top-left corner on the area map.
    pub map_position: MapPosition,
    /// The size.
    pub size: Screens,
    /// The vertical scrollers.
    pub scrollers: Scrollers,
    /// The CRE bits, as the original stores them.
    pub cre: u8,
    /// The door list.
    pub doors: RomAddress,
    /// The conditional states, in the order they're checked.
    pub conditional: Vec<ConditionalState>,
    /// The default state.
    pub default: RoomState,
}

impl Room {
    /// The state at `index`.
    #[must_use]
    pub fn state(&self, index: StateIndex) -> Option<&RoomState> {
        let index = usize::from(index.0);
        self.conditional
            .get(index)
            .map(|entry| &entry.state)
            .or_else(|| (index == self.conditional.len()).then_some(&self.default))
    }

    /// Each state with its index, the default last.
    pub fn states(&self) -> impl Iterator<Item = (StateIndex, &RoomState)> + '_ {
        self.conditional
            .iter()
            .map(|entry| &entry.state)
            .chain(core::iter::once(&self.default))
            .zip(0..)
            .map(|(state, index)| (StateIndex(index), state))
    }
}

/// Why a room doesn't decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomError {
    /// A read outside the image.
    Address(AddressError),
    /// A record runs past its bank's end.
    BankEnd(RomAddress),
    /// The header names an area that doesn't exist.
    Area {
        /// The room.
        room: RoomKey,
        /// The area byte.
        value: u8,
    },
    /// A state condition's code names no condition routine.
    Condition {
        /// Where the code is.
        at: RomAddress,
        /// The code.
        code: u16,
    },
    /// An event state condition names an event number with no meaning. The original would test
    /// the number's bit; no room of the release names one (every room a load station or a door
    /// reaches is checked), so the importer refuses an image that does rather than invent an
    /// event for it.
    Event {
        /// Where the condition is.
        at: RomAddress,
        /// The event number.
        number: u8,
    },
    /// A room has more states than a [`StateIndex`] counts.
    TooManyStates(RoomKey),
    /// A state's level data doesn't decompress.
    LevelData {
        /// Where the stream is.
        at: RomAddress,
        /// Why.
        error: DecompressError,
    },
    /// A state's level data doesn't decode into blocks.
    Level {
        /// Where the stream is.
        at: RomAddress,
        /// Why.
        error: LevelError,
    },
    /// A state's scroll table is outside the image.
    Scroll {
        /// The state header.
        state: RomAddress,
        /// Why.
        error: AddressError,
    },
    /// A state's PLM population doesn't decode.
    Plm {
        /// The state header.
        state: RomAddress,
        /// Why.
        error: PlmError,
    },
    /// The load list repeats a key or is too long.
    RoomList,
}

impl From<AddressError> for RoomError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// The loaded rooms, by ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rooms {
    map: RoomMap,
    rooms: Vec<Room>,
}

impl Rooms {
    /// Decode the rooms in `keys`, in order, and the address map of what they read.
    ///
    /// # Errors
    ///
    /// [`RoomError`] if a key repeats or a room doesn't decode.
    pub fn load(rom: Lorom<'_>, keys: &[RoomKey]) -> Result<(Self, AddressMap), RoomError> {
        let map = RoomMap::new(keys.to_vec()).ok_or(RoomError::RoomList)?;
        let mut addresses = AddressMap::default();
        let rooms = map
            .iter()
            .map(|(key, id)| {
                let decoded = decode(rom, key)?;
                record(&mut addresses, id, &decoded);
                Ok(decoded)
            })
            .collect::<Result<_, RoomError>>()?;
        Ok((Self { map, rooms }, addresses))
    }

    /// The map from keys to IDs.
    #[must_use]
    pub const fn map(&self) -> &RoomMap {
        &self.map
    }

    /// The room with `id`.
    #[must_use]
    pub fn get(&self, id: RoomId) -> Option<&Room> {
        self.rooms.get(usize::from(id.0))
    }
}

/// Add `room`'s locations to `addresses`.
fn record(addresses: &mut AddressMap, id: RoomId, room: &Room) {
    addresses.insert(Location::RoomHeader(id), room.key.0);
    addresses.insert(Location::DoorList(id), room.doors);
    for (index, state) in room.states() {
        for (part, address) in [
            (StatePart::Header, state.at),
            (StatePart::LevelData, state.level.at),
            (StatePart::Fx, state.fx),
            (StatePart::EnemyPopulation, state.enemy_population),
            (StatePart::EnemySet, state.enemy_set),
            (StatePart::Scroll, state.scroll),
            (StatePart::PlmPopulation, state.plm_population),
            (StatePart::LibraryBackground, state.library_background),
        ] {
            addresses.insert(Location::State(id, index, part), address);
        }
    }
}

/// A cursor over records in one bank.
struct Reader<'a> {
    rom: Lorom<'a>,
    at: RomAddress,
}

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], RoomError> {
        let bytes = self.rom.read(self.at, N)?;
        let array = <[u8; N]>::try_from(bytes).map_err(|_| RoomError::BankEnd(self.at))?;
        self.at = self.at.advance(N).ok_or(RoomError::BankEnd(self.at))?;
        Ok(array)
    }

    fn byte(&mut self) -> Result<u8, RoomError> {
        let [byte] = self.take()?;
        Ok(byte)
    }

    fn word(&mut self) -> Result<u16, RoomError> {
        self.take().map(u16::from_le_bytes)
    }

    /// A 16-bit pointer into `bank`.
    fn pointer(&mut self, bank: u8) -> Result<RomAddress, RoomError> {
        Ok(RomAddress::new(bank, self.word()?))
    }
}

/// Decode the room whose header is at `key`.
///
/// # Errors
///
/// [`RoomError`] if a record is outside the image, names an unknown area or condition, or
/// its level data doesn't decompress.
pub fn decode(rom: Lorom<'_>, key: RoomKey) -> Result<Room, RoomError> {
    let mut reader = Reader { rom, at: key.0 };
    let [index, area, x, y, width, height, up, down, cre] = reader.take()?;
    let size = Screens { width, height };
    let area = Area::try_from(area).map_err(|value| RoomError::Area { room: key, value })?;
    let doors = reader.pointer(ROOM_BANK)?;
    let mut conditional = Vec::new();
    while let Some(condition) = condition(&mut reader)? {
        let at = reader.pointer(ROOM_BANK)?;
        conditional.push(ConditionalState {
            condition,
            state: state(rom, at, size)?,
        });
    }
    if u8::try_from(conditional.len()).is_err() {
        return Err(RoomError::TooManyStates(key));
    }
    Ok(Room {
        key,
        index,
        area,
        map_position: MapPosition { x, y },
        size,
        scrollers: Scrollers { up, down },
        cre,
        doors,
        conditional,
        default: state(rom, reader.at, size)?,
    })
}

/// The level data at `at` of a room of `size`, decompressed and decoded, refused where a chain of
/// stand-ins loops once the room's start has placed those of `plms`.
fn level(
    rom: Lorom<'_>,
    at: RomAddress,
    size: Screens,
    plms: &PlmPopulation,
) -> Result<(LevelData, Level), RoomError> {
    let level = decompress(rom.from(at)?)
        .map(|decoded| LevelData {
            at,
            bytes: decoded.bytes,
        })
        .map_err(|error| RoomError::LevelData { at, error })?;
    let width = size.blocks_across();
    let layers = Level::decode(&level.bytes)
        .and_then(|layers| {
            layers.refuse_stand_in_loops(width, plms.placed_blocks(width))?;
            Ok(layers)
        })
        .map_err(|error| RoomError::Level { at, error })?;
    Ok((level, layers))
}

/// The next state condition, or `None` at the default condition that ends the list.
fn condition(reader: &mut Reader<'_>) -> Result<Option<Condition>, RoomError> {
    let at = reader.at;
    let condition = match reader.word()? {
        0xe5e6 => return Ok(None),
        0xe5eb => Condition::Door(reader.pointer(DOOR_BANK)?),
        0xe5ff => Condition::MainAreaBossDead,
        0xe612 => {
            let number = reader.byte()?;
            Condition::Event(event(number).ok_or(RoomError::Event { at, number })?)
        }
        0xe629 => Condition::BossDead(BossBits(reader.byte()?)),
        0xe640 => Condition::MorphBall,
        0xe652 => Condition::MorphBallAndMissiles,
        0xe669 => Condition::PowerBombs,
        0xe678 => Condition::SpeedBooster,
        code => return Err(RoomError::Condition { at, code }),
    };
    Ok(Some(condition))
}

/// The state header at `at` of a room of `size`, its level data decompressed and decoded.
fn state(rom: Lorom<'_>, at: RomAddress, size: Screens) -> Result<RoomState, RoomError> {
    let mut reader = Reader { rom, at };
    let [
        level_low,
        level_high,
        level_bank,
        tileset,
        music_data,
        music_track,
    ] = reader.take()?;
    let fx = reader.pointer(FX_BANK)?;
    let enemy_population = reader.pointer(ENEMY_POPULATION_BANK)?;
    let enemy_set = reader.pointer(ENEMY_SET_BANK)?;
    let layer2_scroll = reader.take()?;
    let scroll = reader.pointer(ROOM_BANK)?;
    let special_xray_blocks = reader.pointer(ROOM_BANK)?;
    let main_code = reader.pointer(ROOM_BANK)?;
    let plm_population = reader.pointer(ROOM_BANK)?;
    let library_background = reader.pointer(ROOM_BANK)?;
    let setup_code = reader.pointer(ROOM_BANK)?;
    let level_at = RomAddress::from_long([level_low, level_high, level_bank]);
    let plms =
        population(rom, plm_population).map_err(|error| RoomError::Plm { state: at, error })?;
    let (level, layers) = level(rom, level_at, size, &plms)?;
    let scroll_areas =
        scroll_areas(rom, scroll, size).map_err(|error| RoomError::Scroll { state: at, error })?;
    Ok(RoomState {
        at,
        level,
        layers,
        tileset,
        music: Music {
            data: music_data,
            track: music_track,
        },
        fx,
        enemy_population,
        enemy_set,
        layer2_scroll,
        scroll,
        scroll_areas,
        special_xray_blocks,
        main_code,
        plm_population,
        plms,
        library_background,
        setup_code,
    })
}

#[cfg(test)]
mod tests;

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// Room headers and their states, decoded from bank `$8F` (D15).
// conditions in order and loads the first state whose condition holds (`$8F:E5E6` to
// `$8F:E678` are the condition routines).
// The Morph Ball Room, `$8F:9E9F`.
// The original's 16-bit room pointer for this room, if its header is in bank `$8F`.
// `$E5EB` (unused): the room was entered through this door.
// `$E5FF`: the area's main boss is dead.
// `$E612`: the event is set.
// `$E629`: one of these boss bits is set for the area.
// `$E640` (unused): the Morph Ball is collected.
// `$E652`: the Morph Ball and missiles are collected.
// `$E669`: power bombs are collected.
// `$E678` (unused): the Speed Booster is collected.
