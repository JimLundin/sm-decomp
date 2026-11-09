//! Progress as a harness domain: the progress model (`clean_model_progress`) against the oracle.
//!
//! It is checked frame by frame on every quantity the spec's State holds
//! (`docs/behaviours/progress.md`), by the decode notes' harness note
//! (`docs/behaviours/progress.decode.md`, "The harness"). Progress is a store: other domains
//! report, and nothing of the same frame reads it back, so it is checked on its own. Its State is
//! decoded from record N; the room load from record N+1, which other domains write; and the
//! model's output is compared with record N+1.
//!
//! The reports should come from the record's watched instruction hits (the entry points of the
//! decode notes' Inputs table). The cached rows were recorded without those watches, so the
//! harness stands in for the reporting domains from what record N+1 shows changed: a place newly
//! emptied is the pickup of the item PLM record N holds with that room argument (its pickup
//! instruction read from the ROM), a door newly open the door PLM's report, and an event, a
//! zebetite, a boss, a map or a station newly marked the report that marks it, naming the area
//! she is in (record N+1's), or for leaving the area the one she leaves (record N's). Those
//! answers copy record N+1, so every field they move is unverified, never matched (the harness's
//! neutral stand-in makes none). A room load is the door's domain's, from the room pointer: its
//! state is the model's choice over the room's conditions, and the items its state places (the
//! PLM domain's `ItemsPlaced`) come from the ROM, so neither copies an answer.
//!
//! The null model is progress left as record N has it, and the room loaded in its default state.

use core::cell::RefCell;
use core::fmt;
use core::num::Wrapping;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use clean_model_progress::{
    Beams, BehaviourId, BossKinds, CATALOGUE, Condition, Count, Events, Items, MapStatus, PerArea,
    PerSupply, Pickup, Progress, Remembered, Report, ReserveMode, RoomEntry, RoomStates, Rules,
    Supply,
};
use clean_types::{Area, Beam, BossKind, DoorId, Entrance, Item, LocationId, StationId};
use data::rom::{Lorom, RomAddress};
use data::room::{RoomKey, decode as decode_room};
use game::progress_world;
use oracle_record::FrameRecord;

use super::{Domain, Entry, Field, FieldSet, Interval, Mask, Uncompared};
use crate::decode::DecodeError;
use crate::domain_field;
use crate::source::{self, WramBlock, WramWord};

/// A work RAM word the harness reads that `source` doesn't name.
const fn wram(name: &'static str, offset: u16) -> WramWord {
    WramWord { name, offset }
}

/// A run of work RAM bytes the harness reads that `source` doesn't name.
const fn block(name: &'static str, offset: u32, len: u16) -> WramBlock {
    WramBlock { name, offset, len }
}

/// Bank $90 log: the beams switched on and hers, in the beam bits.
const EQUIPPED_BEAMS: WramWord = wram("EQUIPPED_BEAMS", 0x09A6);
const COLLECTED_BEAMS: WramWord = wram("COLLECTED_BEAMS", 0x09A8);
/// Bank $82 log: the door the room was entered by, a pointer into bank $83.
const DOOR_POINTER: WramWord = wram("DOOR_POINTER", 0x078D);
/// Bank $82 log, `$82:DED0`: the room's door list, which loading the room header sets just before
/// its state is chosen (`$82:DED3`). The room pointer changes earlier, when the door transition
/// loads the door (`$82:DE1D`), so a new door list is the frame the room loads.
const DOOR_LIST: WramWord = wram("DOOR_LIST_POINTER", 0x07B5);
/// Bank $80 log, `$80:81FA`: the event bits, bit n of byte n / 8.
const EVENTS: WramBlock = block("EVENT_BITS", 0xD820, 8);
/// Bank $80 log, `$80:81A6`: a byte of boss bits per area.
const BOSSES: WramBlock = block("BOSS_BITS", 0xD828, 7);
/// Bank $84 log, `$84:8A91`: the doors open for good, bit n of byte n / 8.
const DOORS: WramBlock = block("OPENED_DOOR_BITS", 0xD8B0, 0x40);
/// Bank $84 log, `$84:8D12`: a word per area, its low byte the stations saved at.
const STATIONS: WramBlock = block("USED_SAVE_STATIONS", 0xD8F8, 14);
/// Bank $84 log, `$84:8C8F`: a byte per area, the map known.
const MAPS: WramBlock = block("MAP_STATION_BYTES", 0xD908, 7);
/// Bank $84 log, `$84:EE6D`: the items the rooms' item PLMs placed.
const ITEMS_PLACED: WramWord = wram("NUM_ITEMS_PLACED", 0xD91A);

/// The game's areas in its order (`079F`).
const AREAS: [Area; 7] = [
    Area::Crateria,
    Area::Brinstar,
    Area::Norfair,
    Area::WreckedShip,
    Area::Maridia,
    Area::Tourian,
    Area::Ceres,
];
/// The item bits (`09A2`, `09A4`).
const ITEMS: [(u16, Item); 11] = [
    (0x0001, Item::VariaSuit),
    (0x0002, Item::SpringBall),
    (0x0004, Item::MorphBall),
    (0x0008, Item::ScrewAttack),
    (0x0020, Item::GravitySuit),
    (0x0100, Item::HiJumpBoots),
    (0x0200, Item::SpaceJump),
    (0x1000, Item::Bombs),
    (0x2000, Item::SpeedBooster),
    (0x4000, Item::GrappleBeam),
    (0x8000, Item::XRayScope),
];
/// The beam bits (`09A6`, `09A8`).
const BEAMS: [(u16, Beam); 5] = [
    (0x0001, Beam::Wave),
    (0x0002, Beam::Ice),
    (0x0004, Beam::Spazer),
    (0x0008, Beam::Plasma),
    (0x1000, Beam::Charge),
];
/// The boss bits of an area's byte (`$80:81A6`'s list).
const BOSS_KINDS: [(u16, BossKind); 3] = [
    (1, BossKind::MainBoss),
    (2, BossKind::MiniBoss),
    (4, BossKind::Torizo),
];
/// Each supply's capacity and what she holds of it (`09C2` to `09D8`).
const SUPPLIES: [(Supply, WramWord, WramWord); 5] = [
    (
        Supply::Energy,
        source::SAMUS_MAX_HEALTH,
        source::SAMUS_HEALTH,
    ),
    (
        Supply::Reserve,
        source::SAMUS_MAX_RESERVE_HEALTH,
        source::SAMUS_RESERVE_HEALTH,
    ),
    (
        Supply::Missiles,
        source::SAMUS_MAX_MISSILES,
        source::SAMUS_MISSILES,
    ),
    (
        Supply::SuperMissiles,
        source::SAMUS_MAX_SUPER_MISSILES,
        source::SAMUS_SUPER_MISSILES,
    ),
    (
        Supply::PowerBombs,
        source::SAMUS_MAX_POWER_BOMBS,
        source::SAMUS_POWER_BOMBS,
    ),
];
/// The events whose bits count the zebetites destroyed, lowest first (`$A6:FCAA`).
const ZEBETITE_BITS: [u16; 3] = [3, 4, 5];
/// The map byte a station writes (`$84:8C8F`); any other but 0 is the arrival's.
const DOWNLOADED: u8 = 0xFF;

/// The item PLMs' IDs (`$84:EED7` to `$84:EFCF`).
const ITEM_PLMS: core::ops::RangeInclusive<u16> = 0xEED7..=0xEFCF;
/// The PLM slots.
const PLM_SLOTS: u16 = 40;
/// An item PLM's list: "set the place emptied" (`$84:8899`), then the fanfare (`$84:8BDD` and
/// its byte), then the pickup instruction and its word.
const BEFORE_PICKUP: [u8; 4] = [0x99, 0x88, 0xDD, 0x8B];
const PICKUP_AFTER: usize = 5;
/// How far into an item PLM's list the harness looks for its pickup.
const LIST_REACH: usize = 0x100;
/// The PLM bank.
const PLM_BANK: u8 = 0x84;

/// The game states of the pause screens, where the equipment screen switches items, beams and
/// the reserve mode (`$82:8CEF` to `$82:A5B7`, game states Ch to 12h).
const PAUSE: core::ops::RangeInclusive<u16> = 0x0C..=0x12;
/// Mother Brain's species, live while her drain can give the hyper beam (`$91:E5F0`).
const MOTHER_BRAIN: [u16; 2] = [0xEC3F, 0xEC7F];
/// The enemy slots.
const ENEMY_SLOTS: u16 = 32;

/// The declared fields by their places in [`ProgressDomain`]'s `FIELDS`.
const ITEMS_ON: FieldSet = FieldSet::single(1);
const BEAMS_ON: FieldSet = FieldSet::single(3);
const HELD: FieldSet = FieldSet::single(5);
const RESERVE_MODE: FieldSet = FieldSet::single(6);
const ROOM_STATE: FieldSet = FieldSet::single(15);
/// The declared fields only the catalogue's entries name, by their places in
/// [`ProgressDomain`]'s `FIELDS`.
mod field {
    use super::FieldSet;

    pub const ITEMS: FieldSet = FieldSet::single(0);
    pub const BEAMS: FieldSet = FieldSet::single(2);
    pub const CAPACITY: FieldSet = FieldSet::single(4);
    pub const EVENTS: FieldSet = FieldSet::single(7);
    pub const ZEBETITES: FieldSet = FieldSet::single(8);
    pub const DEFEATED: FieldSet = FieldSet::single(9);
    pub const EMPTIED: FieldSet = FieldSet::single(10);
    pub const OPENED: FieldSet = FieldSet::single(11);
    pub const MAPS: FieldSet = FieldSet::single(12);
    pub const STATIONS: FieldSet = FieldSet::single(13);
    pub const ITEMS_PLACED: FieldSet = FieldSet::single(14);
}

/// The declared fields a catalogue entry's output reaches. The item percentage is the ending's,
/// which no declared field holds, so it is credited nothing.
const fn shown_by(id: BehaviourId) -> FieldSet {
    use BehaviourId as B;
    match id {
        B::EventsHappen => field::EVENTS,
        B::ZebetitesCounted => field::ZEBETITES,
        B::BossesDefeated => field::DEFEATED,
        B::EmptiedLocations => field::EMPTIED,
        B::DoorsStayOpen => field::OPENED,
        B::MapsByArea | B::MapOnArrival => field::MAPS,
        B::StationsSavedAt => field::STATIONS,
        B::ItemsPlacedCount => field::ITEMS_PLACED,
        B::TakingItems => field::ITEMS.union(ITEMS_ON),
        B::TakingBeams => field::BEAMS.union(BEAMS_ON),
        B::Tanks => field::CAPACITY.union(HELD).union(RESERVE_MODE),
        B::RoomStates => ROOM_STATE,
        B::ItemPercentage => FieldSet::EMPTY,
    }
}

/// Why a frame isn't compared: every one is a harness defect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Skip {
    /// A source couldn't be read, or holds a value the spec names nothing for.
    Unreadable(DecodeError),
    /// A room's header or states don't decode from the ROM.
    Room(u16, String),
    /// The stand-in found no report for what record N+1 changed.
    NoReport(String),
}

impl fmt::Display for Skip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(error) => write!(f, "unreadable: {error:?}"),
            Self::Room(room, error) => write!(f, "room {room:#06X}: {error}"),
            Self::NoReport(what) => write!(f, "no stand-in report: {what}"),
        }
    }
}

impl From<DecodeError> for Skip {
    fn from(error: DecodeError) -> Self {
        Self::Unreadable(error)
    }
}

fn word(record: &FrameRecord, source: WramWord) -> Result<u16, Skip> {
    Ok(source.read(&record.wram)?)
}

fn bytes(record: &FrameRecord, source: WramBlock) -> Result<&[u8], Skip> {
    Ok(source.read(&record.wram)?)
}

/// A value the spec names nothing for.
const fn unnamed(source: &'static str, value: u16) -> Skip {
    Skip::Unreadable(DecodeError::Unrepresentable { source, value })
}

/// The members of `word` by `table`'s bits; a bit no member has is unnamed.
fn members<T: Copy>(word: u16, table: &[(u16, T)], source: &'static str) -> Result<Vec<T>, Skip> {
    let known = table.iter().fold(0, |all, &(bit, _)| all | bit);
    if word & !known != 0 {
        return Err(unnamed(source, word & !known));
    }
    Ok(table
        .iter()
        .filter(|&&(bit, _)| word & bit != 0)
        .map(|&(_, member)| member)
        .collect())
}

/// The one member of `members`, if it has exactly one.
const fn single<T: Copy>(members: &[T]) -> Option<T> {
    match *members {
        [member] => Some(member),
        _ => None,
    }
}

/// The numbers whose bits are set in `bytes`: bit n of byte n / 8.
fn numbers(bytes: &[u8]) -> impl Iterator<Item = u16> + '_ {
    bytes
        .iter()
        .zip((0_u16..).step_by(8))
        .flat_map(|(&byte, first)| {
            (0..8_u16)
                .filter(move |bit| (byte >> bit) & 1 != 0)
                .map(move |bit| first.wrapping_add(bit))
        })
}

/// The numbers whose bits `next` sets in `source` and `from` doesn't.
fn newly(from: &FrameRecord, next: &FrameRecord, source: WramBlock) -> Result<Vec<u16>, Skip> {
    let was: BTreeSet<u16> = numbers(bytes(from, source)?).collect();
    Ok(numbers(bytes(next, source)?)
        .filter(|number| !was.contains(number))
        .collect())
}

/// The event of event bit `number`, or none for a zebetite's bit or a bit no event has.
fn event(number: u16) -> Option<clean_types::Event> {
    if ZEBETITE_BITS.contains(&number) {
        return None;
    }
    u8::try_from(number).ok().and_then(data::room::event)
}

/// The frame's area (`079F`).
fn area(record: &FrameRecord) -> Result<Area, Skip> {
    let index = word(record, source::AREA_INDEX)?;
    AREAS
        .get(usize::from(index))
        .copied()
        .ok_or_else(|| unnamed(source::AREA_INDEX.name, index))
}

/// A count word.
fn count(record: &FrameRecord, source: WramWord) -> Result<Count, Skip> {
    Ok(Wrapping(word(record, source)?))
}

/// The progress `record` holds (the decode notes' State).
fn progress(record: &FrameRecord) -> Result<Progress, Skip> {
    let mut progress = Progress::new_file();
    let items = |source: WramWord| -> Result<Items, Skip> {
        Ok(Items::of(&members(
            word(record, source)?,
            &ITEMS,
            source.name,
        )?))
    };
    let beams = |source: WramWord| -> Result<Beams, Skip> {
        Ok(Beams::of(&members(
            word(record, source)?,
            &BEAMS,
            source.name,
        )?))
    };
    let inventory = &mut progress.inventory;
    inventory.items = items(source::COLLECTED_ITEMS)?;
    inventory.equipped = items(source::EQUIPPED_ITEMS)?;
    inventory.beams = beams(COLLECTED_BEAMS)?;
    inventory.beams_equipped = beams(EQUIPPED_BEAMS)?;
    for (supply, capacity, held) in SUPPLIES {
        inventory.capacity[supply] = count(record, capacity)?;
        inventory.held[supply] = count(record, held)?;
    }
    inventory.reserve_mode = match word(record, source::RESERVE_HEALTH_MODE)? {
        0 => ReserveMode::None,
        1 => ReserveMode::Automatic,
        2 => ReserveMode::Manual,
        mode => return Err(unnamed(source::RESERVE_HEALTH_MODE.name, mode)),
    };
    for number in numbers(bytes(record, EVENTS)?) {
        if let Some(place) = ZEBETITE_BITS.iter().position(|&bit| bit == number) {
            progress.zebetites += Wrapping(1 << place);
        } else {
            let event = event(number).ok_or_else(|| unnamed(EVENTS.name, number))?;
            progress.events = progress.events.with(event);
        }
    }
    let per_area = |source: WramBlock, stride: usize| -> Result<Vec<(Area, u8)>, Skip> {
        let bytes = bytes(record, source)?;
        Ok(AREAS
            .iter()
            .zip(bytes.iter().step_by(stride))
            .map(|(&area, &byte)| (area, byte))
            .collect())
    };
    for (area, bits) in per_area(BOSSES, 1)? {
        let kinds = members(u16::from(bits), &BOSS_KINDS, BOSSES.name)?;
        progress.defeated[area] = BossKinds::of(&kinds);
    }
    for (area, byte) in per_area(MAPS, 1)? {
        progress.maps[area] = match byte {
            0 => MapStatus::None,
            DOWNLOADED => MapStatus::Downloaded,
            _ => MapStatus::OnArrival,
        };
    }
    // The high bytes are the debug elevators', which no door reaches.
    for (area, bits) in per_area(STATIONS, 2)? {
        progress.stations[area] = numbers(&[bits]).map(StationId).collect();
    }
    progress.emptied =
        Remembered::of(numbers(bytes(record, source::ITEM_BIT_ARRAY)?).map(LocationId));
    progress.opened = Remembered::of(numbers(bytes(record, DOORS)?).map(DoorId));
    progress.placements = count(record, ITEMS_PLACED)?;
    Ok(progress)
}

/// A room as the harness reads it from the ROM.
pub struct RoomStatesAt {
    /// The room's own area.
    area: Area,
    /// Its states' conditions as the model reads them, the default's last.
    conditions: Vec<Condition>,
    /// Each state's header pointer, in the list's order, the default last.
    headers: Vec<u16>,
    /// The item PLMs each state's population places.
    items: Vec<u16>,
}

/// A room's standing state: the one a record holds and the one the model chooses, by their
/// places in the room's list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Standing {
    /// The state the record holds.
    pub held: usize,
    /// The state the model chooses.
    pub chosen: Option<usize>,
    /// The room's states, the default last.
    pub states: usize,
}

/// A room load in the frame: the room loaded and the door she came in by.
pub struct Load {
    /// The room.
    room: Rc<RoomStatesAt>,
    /// The door entry she came in by (`078D`).
    door: Entrance,
}

/// What the frame gives progress besides its State: the stand-in's reports and the room load.
#[derive(Clone)]
pub struct Upstream {
    /// The reports the stand-in decoded from record N+1's changes, copied.
    reports: Vec<Report>,
    /// The room loaded in the frame, if one is.
    load: Option<Rc<Load>>,
    /// The items the room's state places, when its PLMs spawn in the frame.
    placed: Option<Count>,
}

/// The model's State and the room state it chose in the frame.
#[derive(Clone)]
pub struct Held {
    /// The progress.
    progress: Progress,
    /// The room's state, by its place in the list: its default as decoded, the model's choice
    /// when a room loads in the frame.
    room_state: Option<usize>,
}

/// The declared fields: every quantity of the spec's State, and the room state at a load.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declared {
    items: Items,
    items_on: Items,
    beams: Beams,
    beams_on: Beams,
    capacity: PerSupply<Count>,
    held: PerSupply<Count>,
    reserve_mode: ReserveMode,
    events: Events,
    zebetites: Count,
    defeated: PerArea<BossKinds>,
    emptied: Remembered<LocationId>,
    opened: Remembered<DoorId>,
    maps: PerArea<MapStatus>,
    stations: PerArea<BTreeSet<StationId>>,
    placements: Count,
    room_state: Option<usize>,
}

impl Declared {
    fn of(progress: &Progress, room_state: Option<usize>) -> Self {
        let inventory = &progress.inventory;
        Self {
            items: inventory.items,
            items_on: inventory.equipped,
            beams: inventory.beams,
            beams_on: inventory.beams_equipped,
            capacity: inventory.capacity,
            held: inventory.held,
            reserve_mode: inventory.reserve_mode,
            events: progress.events,
            zebetites: progress.zebetites,
            defeated: progress.defeated,
            emptied: progress.emptied.clone(),
            opened: progress.opened.clone(),
            maps: progress.maps,
            stations: progress.stations.clone(),
            placements: progress.placements,
            room_state,
        }
    }
}

/// The progress domain over the ROM, which holds the rooms' states and the item PLMs' lists.
pub struct ProgressDomain {
    image: Vec<u8>,
    rooms: RefCell<BTreeMap<u16, Rc<RoomStatesAt>>>,
}

impl RoomStatesAt {
    /// The default state's place in the list: the last.
    const fn default_state(&self) -> usize {
        self.headers.len().saturating_sub(1)
    }

    /// The state the model chooses for `progress`, entered by `door`; none if the conditions
    /// don't end in the default's (`progress_world::conditions` always ends them so).
    fn choose(&self, progress: &Progress, door: Entrance) -> Option<usize> {
        let entry = RoomEntry {
            area: self.area,
            entered_by: Some(door),
        };
        RoomStates::new(&self.conditions).map(|states| states.choose(progress, entry))
    }
}

impl ProgressDomain {
    /// The domain over the ROM `image`.
    #[must_use]
    pub const fn new(image: Vec<u8>) -> Self {
        Self {
            image,
            rooms: RefCell::new(BTreeMap::new()),
        }
    }

    /// The room whose header is at `$8F:pointer`, read once.
    fn room(&self, pointer: u16) -> Result<Rc<RoomStatesAt>, Skip> {
        if let Some(room) = self.rooms.borrow().get(&pointer) {
            return Ok(Rc::clone(room));
        }
        let failed = |error: String| Skip::Room(pointer, error);
        let decoded = decode_room(Lorom::new(&self.image), RoomKey::from_room_pointer(pointer))
            .map_err(|error| failed(format!("{error:?}")))?;
        let conditions = progress_world::conditions(&decoded)
            .ok_or_else(|| failed("a condition the model can't read".to_owned()))?;
        let area = progress_world::region(decoded.area)
            .ok_or_else(|| failed("the debug area".to_owned()))?;
        let (headers, items) = decoded
            .states()
            .map(|(_, state)| {
                let placed = state
                    .plms
                    .entries
                    .iter()
                    .filter(|entry| ITEM_PLMS.contains(&entry.kind.id()))
                    .count();
                (state.at.offset, u16::try_from(placed).unwrap_or(u16::MAX))
            })
            .unzip();
        let room = Rc::new(RoomStatesAt {
            area,
            conditions,
            headers,
            items,
        });
        self.rooms.borrow_mut().insert(pointer, Rc::clone(&room));
        Ok(room)
    }

    /// The room `record` is in.
    fn room_of(&self, record: &FrameRecord) -> Result<Rc<RoomStatesAt>, Skip> {
        self.room(word(record, source::ROOM_PTR)?)
    }

    /// The room state `record`'s room holds and the one the model chooses for it over
    /// `record`'s progress, entered by `record`'s door: a row's first record, where the load
    /// that chose it lies before the row.
    ///
    /// # Errors
    ///
    /// The decode's failure, or a state the room doesn't list.
    pub fn standing_state(&self, record: &FrameRecord) -> Result<Standing, Skip> {
        let room = self.room_of(record)?;
        let state = word(record, source::ROOMDEFROOMSTATE_PTR)?;
        let held = room
            .headers
            .iter()
            .position(|&header| header == state)
            .ok_or_else(|| Skip::NoReport(format!("state {state:#06X} isn't one of its room's")))?;
        Ok(Standing {
            held,
            chosen: room.choose(&progress(record)?, Entrance(word(record, DOOR_POINTER)?)),
            states: room.headers.len(),
        })
    }

    /// The items placed in the frame from `from` to `next`, the PLM domain's report: when the
    /// room's PLMs spawn (an item PLM `next` holds that `from` doesn't), the item PLMs of the
    /// population of the state the room loaded in (record N's).
    fn placed(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Option<Count>, Skip> {
        let before = item_plms(from)?;
        if item_plms(next)?.is_subset(&before) {
            return Ok(None);
        }
        let state = word(from, source::ROOMDEFROOMSTATE_PTR)?;
        let room = self.room_of(from)?;
        room.headers
            .iter()
            .position(|&header| header == state)
            .and_then(|place| room.items.get(place))
            .map(|&placed| Some(Wrapping(placed)))
            .ok_or_else(|| Skip::NoReport(format!("state {state:#06X} isn't one of its room's")))
    }

    /// What the item PLM `plm` gives: its list's pickup instruction and its word.
    fn pickup(&self, plm: u16) -> Result<Pickup, Skip> {
        let rom = Lorom::new(&self.image);
        let none = || Skip::NoReport(format!("item PLM {plm:#06X} has no pickup the spec names"));
        let read = |offset: u16, count: usize| rom.read(RomAddress::new(PLM_BANK, offset), count);
        let [_, _, list_low, list_high] = read(plm, 4)
            .ok()
            .and_then(|header| <[u8; 4]>::try_from(header).ok())
            .ok_or_else(none)?;
        let list = u16::from_le_bytes([list_low, list_high]);
        let bytes = read(list, LIST_REACH).map_err(|_| none())?;
        let at = bytes
            .windows(BEFORE_PICKUP.len())
            .position(|window| window == BEFORE_PICKUP)
            .ok_or_else(none)?
            .wrapping_add(PICKUP_AFTER);
        let [code_low, code_high, argument_low, argument_high] = bytes
            .get(at..at.wrapping_add(4))
            .and_then(|slice| <[u8; 4]>::try_from(slice).ok())
            .ok_or_else(none)?;
        let argument = u16::from_le_bytes([argument_low, argument_high]);
        let tank = |supply| Pickup::Tank {
            supply,
            amount: Wrapping(argument),
        };
        Ok(match u16::from_le_bytes([code_low, code_high]) {
            0x88B0 => {
                Pickup::Beam(single(&members(argument, &BEAMS, "beam pickup")?).ok_or_else(none)?)
            }
            0x88F3 | 0x891A | 0x8941 => {
                Pickup::Item(single(&members(argument, &ITEMS, "item pickup")?).ok_or_else(none)?)
            }
            0x8968 => tank(Supply::Energy),
            0x8986 => tank(Supply::Reserve),
            0x89A9 => tank(Supply::Missiles),
            0x89D2 => tank(Supply::SuperMissiles),
            0x89FB => tank(Supply::PowerBombs),
            _ => return Err(none()),
        })
    }

    /// The reports the stand-in decodes for the frame from `from` to `next`: what changed, by
    /// the domain that would have reported it (the decode notes' Inputs).
    fn stand_in(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Vec<Report>, Skip> {
        let (was, now) = (progress(from)?, progress(next)?);
        // Every report of an area names the one she is in, but leaving it names the one she
        // leaves, which record N is in.
        let (left, here) = (area(from)?, area(next)?);
        let mut reports: Vec<Report> = newly(from, next, EVENTS)?
            .into_iter()
            .filter_map(event)
            .map(Report::Happened)
            .collect();
        match (now.zebetites - was.zebetites).0 {
            0 => {}
            1 => reports.push(Report::ZebetiteDestroyed),
            _ => {
                return Err(Skip::NoReport(
                    "the zebetites' count moved by other than one".into(),
                ));
            }
        }
        for area in AREAS {
            let defeated = BOSS_KINDS
                .iter()
                .filter(|&&(_, kind)| {
                    now.defeated[area].contains(kind) && !was.defeated[area].contains(kind)
                })
                .map(|&(_, kind)| Report::Defeated { kind, area });
            let saved = now.stations[area]
                .difference(&was.stations[area])
                .map(|&station| Report::Saved { area, station });
            let map = match (was.maps[area], now.maps[area]) {
                (map, now) if map == now => None,
                (MapStatus::None, MapStatus::Downloaded) => Some(Report::MapStation(area)),
                (MapStatus::None, MapStatus::OnArrival) => Some(Report::MapOnArrival(area)),
                (MapStatus::OnArrival, MapStatus::Downloaded) => Some(Report::LeftArea(area)),
                (map, now) => {
                    return Err(Skip::NoReport(format!(
                        "{area:?}'s map went from {map:?} to {now:?}"
                    )));
                }
            };
            let found: Vec<Report> = defeated.chain(saved).chain(map).collect();
            let reporter = |report: &Report| {
                if matches!(report, Report::LeftArea(_)) {
                    left
                } else {
                    here
                }
            };
            if found.iter().any(|report| reporter(report) != area) {
                return Err(Skip::NoReport(format!(
                    "{area:?} changed outside the area she is in or leaves"
                )));
            }
            reports.extend(found);
        }
        reports.extend(
            newly(from, next, DOORS)?
                .into_iter()
                .map(|door| Report::DoorOpened(Some(DoorId(door)))),
        );
        let plms = item_plms(from)?;
        for place in newly(from, next, source::ITEM_BIT_ARRAY)? {
            let plm = plms
                .iter()
                .find(|&&(_, argument)| argument == place)
                .map(|&(plm, _)| plm)
                .ok_or_else(|| {
                    Skip::NoReport(format!("place {place} emptied, no item PLM holds it"))
                })?;
            reports.push(Report::Taken {
                pickup: self.pickup(plm)?,
                location: Some(LocationId(place)),
            });
        }
        Ok(reports)
    }
}

/// The item PLMs in `record`'s slots, by ID and room argument.
fn item_plms(record: &FrameRecord) -> Result<BTreeSet<(u16, u16)>, Skip> {
    let mut plms = BTreeSet::new();
    for slot in 0..PLM_SLOTS {
        let plm = word(record, source::PLM_HEADER_PTR.word(slot)?)?;
        if ITEM_PLMS.contains(&plm) {
            plms.insert((plm, word(record, source::PLM_ROOM_ARGUMENTS.word(slot)?)?));
        }
    }
    Ok(plms)
}

/// Whether a room loads in the frame from `from` to `next`: its header's door list is new.
fn loads(from: &FrameRecord, next: &FrameRecord) -> Result<bool, Skip> {
    Ok(word(from, DOOR_LIST)? != word(next, DOOR_LIST)?)
}

/// Whether the main loop is in a pause screen in `record`.
fn paused(record: &FrameRecord) -> Result<bool, Skip> {
    Ok(PAUSE.contains(&word(record, source::GAME_STATE)?))
}

/// Whether Mother Brain is live in `record`.
fn mother_brain(record: &FrameRecord) -> Result<bool, Skip> {
    for slot in 0..ENEMY_SLOTS {
        let species = word(
            record,
            source::ENEMY_DATA.word(source::ENEMY_SPECIES, slot)?,
        )?;
        if MOTHER_BRAIN.contains(&species) {
            return Ok(true);
        }
    }
    Ok(false)
}

impl Domain for ProgressDomain {
    const NAME: &'static str = "progress";
    type State = Held;
    type Inputs = ();
    type World = Upstream;
    type Fields = Declared;
    type Skip = Skip;
    const FIELDS: &'static [Field<Declared>] = &[
        domain_field!("items", items),
        domain_field!("items switched on", items_on),
        domain_field!("beams", beams),
        domain_field!("beams switched on", beams_on),
        domain_field!("capacities", capacity),
        domain_field!("what she holds", held),
        domain_field!("reserve mode", reserve_mode),
        domain_field!("events", events),
        domain_field!("zebetites", zebetites),
        domain_field!("bosses defeated", defeated),
        domain_field!("emptied places", emptied),
        domain_field!("open doors", opened),
        domain_field!("maps", maps),
        domain_field!("stations saved at", stations),
        domain_field!("items placed", placements),
        domain_field!("room state", room_state),
    ];

    fn behaviours(&self) -> Vec<Entry> {
        CATALOGUE
            .iter()
            .map(|entry| Entry {
                name: entry.name.to_owned(),
                fields: shown_by(entry.id),
            })
            .collect()
    }

    fn decode(&self, from: &FrameRecord, _interval: Interval) -> Result<(Held, ()), Skip> {
        // The room pointer is the new room's from the door's load on, well before its state is
        // chosen, so record N's room is the one a load in the frame loads.
        let held = Held {
            progress: progress(from)?,
            room_state: self.room_of(from).ok().map(|room| room.default_state()),
        };
        Ok((held, ()))
    }

    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Upstream, Skip> {
        let load = if loads(from, next)? {
            Some(Rc::new(Load {
                room: self.room_of(next)?,
                door: Entrance(word(next, DOOR_POINTER)?),
            }))
        } else {
            None
        };
        Ok(Upstream {
            reports: self.stand_in(from, next)?,
            load,
            placed: self.placed(from, next)?,
        })
    }

    fn masks(&self, from: &FrameRecord, next: &FrameRecord, (): &()) -> Result<Vec<Mask>, Skip> {
        // Record N+1's quantities are read here only to find the frames another domain's
        // writer moved them: on those the field isn't the model's, on every other frame it is
        // compared, so a report that wrongly moves it is caught.
        let (before, after) = (progress(from)?, progress(next)?);
        let (was, now) = (&before.inventory, &after.inventory);
        let mut masks = Vec::new();
        let mut mask = |reason, fields| masks.push(Mask { reason, fields });
        let tank = self.stand_in(from, next)?.iter().any(|report| {
            matches!(
                report,
                Report::Taken {
                    pickup: Pickup::Tank { .. },
                    ..
                }
            )
        });
        if !tank && now.held != was.held {
            mask(
                "another domain: the energy and weapons domains spent or refilled what she holds, with no tank",
                HELD,
            );
        }
        if paused(from)? || paused(next)? {
            let switched = [
                (ITEMS_ON, now.equipped != was.equipped),
                (BEAMS_ON, now.beams_equipped != was.beams_equipped),
                (RESERVE_MODE, now.reserve_mode != was.reserve_mode),
            ]
            .into_iter()
            .filter(|&(_, changed)| changed)
            .fold(FieldSet::EMPTY, |set, (field, _)| set.union(field));
            if switched != FieldSet::EMPTY {
                mask(
                    "another domain: the equipment screen switched items or beams or set the reserve mode",
                    switched,
                );
            }
        }
        if mother_brain(from)? || mother_brain(next)? {
            mask(
                "another domain: Mother Brain's drain switches on the hyper beam ($91:E5F0)",
                BEAMS_ON,
            );
        }
        if !loads(from, next)? {
            mask("no room loads in the frame: no state is chosen", ROOM_STATE);
        }
        Ok(masks)
    }

    fn neutral(&self, upstream: &Upstream) -> Option<Upstream> {
        (!upstream.reports.is_empty()).then(|| Upstream {
            reports: Vec::new(),
            ..upstream.clone()
        })
    }

    fn step(&self, held: &mut Held, (): &(), upstream: &mut Upstream) {
        let rules = Rules::REFERENCE;
        if let Some(load) = &upstream.load {
            held.room_state = load.room.choose(&held.progress, load.door);
        }
        let placed = upstream.placed.map(Report::ItemsPlaced);
        for &report in placed.iter().chain(&upstream.reports) {
            held.progress.apply(&rules, report);
        }
    }

    fn exercised(&self, _held: &Held, (): &(), upstream: &Upstream) -> Vec<usize> {
        use BehaviourId as B;
        let mut entries = Vec::new();
        for report in &upstream.reports {
            match report {
                Report::Happened(_) => entries.push(B::EventsHappen),
                Report::ZebetiteDestroyed => entries.push(B::ZebetitesCounted),
                Report::Defeated { .. } => entries.push(B::BossesDefeated),
                Report::Taken { pickup, location } => {
                    entries.push(match pickup {
                        Pickup::Item(_) => B::TakingItems,
                        Pickup::Beam(_) => B::TakingBeams,
                        Pickup::Tank { .. } => B::Tanks,
                    });
                    if location.is_some() {
                        entries.push(B::EmptiedLocations);
                    }
                }
                Report::DoorOpened(_) => entries.push(B::DoorsStayOpen),
                Report::MapStation(_) => entries.push(B::MapsByArea),
                Report::MapOnArrival(_) | Report::LeftArea(_) => entries.push(B::MapOnArrival),
                Report::Saved { .. } => entries.push(B::StationsSavedAt),
                Report::ItemsPlaced(_) => entries.push(B::ItemsPlacedCount),
            }
        }
        if upstream.load.is_some() {
            entries.push(B::RoomStates);
        }
        if upstream.placed.is_some_and(|placed| placed.0 != 0) {
            entries.push(B::ItemsPlacedCount);
        }
        let mut places: Vec<usize> = entries
            .into_iter()
            .filter_map(|id| CATALOGUE.iter().position(|entry| entry.id == id))
            .collect();
        places.sort_unstable();
        places.dedup();
        places
    }

    fn fields(&self, held: &Held) -> Declared {
        Declared::of(&held.progress, held.room_state)
    }

    fn expected(&self, next: &FrameRecord) -> Result<Declared, Skip> {
        // A room that doesn't decode leaves no state to compare; on a load it is the world's
        // defect.
        let state = word(next, source::ROOMDEFROOMSTATE_PTR)?;
        let chosen = self
            .room_of(next)
            .ok()
            .and_then(|room| room.headers.iter().position(|&header| header == state));
        Ok(Declared::of(&progress(next)?, chosen))
    }

    fn uncompared(&self, skip: &Skip) -> Uncompared {
        Uncompared::Defect(skip.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::field::{
        BEAMS, CAPACITY, DEFEATED, EMPTIED, EVENTS, ITEMS, ITEMS_PLACED, MAPS, OPENED, STATIONS,
        ZEBETITES,
    };
    use super::{
        BEAMS_ON, Domain, FieldSet, HELD, ITEMS_ON, ProgressDomain, RESERVE_MODE, ROOM_STATE,
    };

    #[test]
    fn the_field_sets_name_the_declared_fields_at_their_places() {
        let named = [
            (ITEMS, "items"),
            (ITEMS_ON, "items switched on"),
            (BEAMS, "beams"),
            (BEAMS_ON, "beams switched on"),
            (CAPACITY, "capacities"),
            (HELD, "what she holds"),
            (RESERVE_MODE, "reserve mode"),
            (EVENTS, "events"),
            (ZEBETITES, "zebetites"),
            (DEFEATED, "bosses defeated"),
            (EMPTIED, "emptied places"),
            (OPENED, "open doors"),
            (MAPS, "maps"),
            (STATIONS, "stations saved at"),
            (ITEMS_PLACED, "items placed"),
            (ROOM_STATE, "room state"),
        ];
        for (set, name) in named {
            let place = ProgressDomain::FIELDS
                .iter()
                .position(|field| field.name == name);
            assert_eq!(place.map(FieldSet::single), Some(set), "{name}");
        }
    }
}
