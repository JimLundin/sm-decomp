//! Progress as the engine meets it: the progress model (`run_progress`).
//!
//! The engine keeps slice 0's part of what the game remembers of a run as the game keeps it:
//! what she has in `Samus::equipment` and the emptied item places in `State::collected_items`,
//! one bit per room argument (`docs/behaviours/run-progress.decode.md`, "State"). The model's
//! `Progress` is built from those, takes the report and is written back; the room's state is
//! the model's choice over the room's conditions as the importer decoded them.
//!
//! The engine holds no events, bosses, beams, doors, maps or stations yet: a report of them has
//! nowhere to be kept, so only `Taken` is made, by the item PLM's pickup. `Taken` concerns no
//! area, so the adapter names none.

use core::num::Wrapping;

use data::assets::world;
use data::id::DoorwayId;
use data::rom::RomAddress;
use data::room::{self, Area, BossBits, Room, StateIndex};
use enum_map::EnumMap;
use run_progress::{
    Condition, EnteredRoom, Inventory, Pickup, Progress, Remembered, Report, ReserveMode,
    RoomStates, Rules, Supplies, Supply, Version,
};
use types::{Area as Region, Entrance, Item as Piece, LocationId, Set};

use crate::state::samus::{self, Equipment, Gauge, Item, Items};
use crate::state::{COLLECTED_ITEM_WORDS, CollectedItems, State};

/// Every item the engine keeps.
const ITEMS: [Item; 11] = [
    Item::VariaSuit,
    Item::SpringBall,
    Item::MorphBall,
    Item::ScrewAttack,
    Item::GravitySuit,
    Item::HiJumpBoots,
    Item::SpaceJump,
    Item::Bombs,
    Item::SpeedBooster,
    Item::GrappleBeam,
    Item::XRayScope,
];

/// A room argument with bit 15 set names no place: its item is never remembered (`$84:887C`).
const NO_PLACE: u16 = 0x8000;
/// Bits a word of the emptied places holds.
const BITS_PER_WORD: u16 = 16;

/// The model's item for the engine's.
#[must_use]
pub const fn piece(item: Item) -> Piece {
    match item {
        Item::VariaSuit => Piece::VariaSuit,
        Item::SpringBall => Piece::SpringBall,
        Item::MorphBall => Piece::MorphBall,
        Item::ScrewAttack => Piece::ScrewAttack,
        Item::GravitySuit => Piece::GravitySuit,
        Item::HiJumpBoots => Piece::HiJumpBoots,
        Item::SpaceJump => Piece::SpaceJump,
        Item::Bombs => Piece::Bombs,
        Item::SpeedBooster => Piece::SpeedBooster,
        Item::GrappleBeam => Piece::GrappleBeam,
        Item::XRayScope => Piece::XRayScope,
    }
}

/// The model's items for a set of the engine's.
fn pieces(items: Items) -> Set<Piece> {
    let mut pieces = Set::default();
    for item in ITEMS.into_iter().filter(|&item| items.contains(item)) {
        pieces[piece(item)] = true;
    }
    pieces
}

/// The engine's set of the model's items.
fn items(pieces: Set<Piece>) -> Items {
    let mut items = Items::default();
    for item in ITEMS.into_iter().filter(|&item| pieces[piece(item)]) {
        items.insert(item);
    }
    items
}

/// The model's reserve mode for the engine's.
const fn mode(mode: samus::ReserveMode) -> ReserveMode {
    match mode {
        samus::ReserveMode::None => ReserveMode::NoTank,
        samus::ReserveMode::Automatic => ReserveMode::Automatic,
        samus::ReserveMode::Manual => ReserveMode::Manual,
    }
}

/// The engine's reserve mode for the model's.
const fn engine_mode(mode: ReserveMode) -> samus::ReserveMode {
    match mode {
        ReserveMode::NoTank => samus::ReserveMode::None,
        ReserveMode::Automatic => samus::ReserveMode::Automatic,
        ReserveMode::Manual => samus::ReserveMode::Manual,
    }
}

/// The engine's gauge of `supply`.
const fn gauge(equipment: &mut Equipment, supply: Supply) -> &mut Gauge {
    match supply {
        Supply::Energy => &mut equipment.energy,
        Supply::Reserve => &mut equipment.reserve,
        Supply::Missiles => &mut equipment.missiles,
        Supply::SuperMissiles => &mut equipment.super_missiles,
        Supply::PowerBombs => &mut equipment.power_bombs,
    }
}

/// The gauges' capacities or what she holds, by `part`.
fn supplies(mut equipment: Equipment, part: fn(Gauge) -> u16) -> Supplies {
    Supplies(EnumMap::from_fn(|supply| {
        Wrapping(part(*gauge(&mut equipment, supply)))
    }))
}

/// What she has, as the model keeps it. The engine has no beams yet: none are hers.
fn inventory(equipment: &Equipment) -> Inventory {
    Inventory {
        items: pieces(equipment.collected),
        items_switched_on: pieces(equipment.equipped),
        beams: Set::default(),
        beams_switched_on: Set::default(),
        capacity: supplies(*equipment, |gauge| gauge.max),
        held: supplies(*equipment, |gauge| gauge.current),
        reserve_mode: mode(equipment.reserve_mode),
    }
}

/// `inventory` written back to the engine's equipment.
fn store_inventory(inventory: &Inventory, equipment: &mut Equipment) {
    equipment.collected = items(inventory.items);
    equipment.equipped = items(inventory.items_switched_on);
    for ((supply, held), max) in inventory.held.0.iter().zip(inventory.capacity.0.values()) {
        *gauge(equipment, supply) = Gauge {
            current: held.0,
            max: max.0,
        };
    }
    equipment.reserve_mode = engine_mode(inventory.reserve_mode);
}

/// What her motion reads: the items switched on, never the ones merely hers.
#[must_use]
pub fn switched_on(equipment: &Equipment) -> Set<Piece> {
    pieces(equipment.equipped)
}

/// The place a room argument names: none with bit 15 set.
#[must_use]
pub const fn place(argument: u16) -> Option<LocationId> {
    if argument & NO_PLACE == 0 {
        Some(LocationId(argument))
    } else {
        None
    }
}

/// The word of the emptied places that keeps `place`, and its bit in the word.
fn bit_of(LocationId(place): LocationId) -> (usize, u16) {
    (
        usize::from(place / BITS_PER_WORD),
        1 << (place % BITS_PER_WORD),
    )
}

/// Whether the engine's bit for `place` is set; a place past its bits never is.
fn is_set(collected: &CollectedItems, place: LocationId) -> bool {
    let (word, bit) = bit_of(place);
    collected.0.get(word).is_some_and(|&word| word & bit != 0)
}

/// The emptied places the engine keeps, one bit per place.
fn emptied(collected: &CollectedItems) -> Remembered<LocationId> {
    let places = (0..).take(COLLECTED_ITEM_WORDS * usize::from(BITS_PER_WORD));
    Remembered(
        places
            .map(LocationId)
            .filter(|&place| is_set(collected, place))
            .collect(),
    )
}

/// `emptied` written back as the engine keeps it, one bit per place the model remembers. A place
/// past those bits isn't kept: the game's populations name places below 100h.
fn store_emptied(emptied: &Remembered<LocationId>, collected: &mut CollectedItems) {
    let mut words = [0; COLLECTED_ITEM_WORDS];
    for (word, bit) in emptied.0.iter().copied().map(bit_of) {
        if let Some(word) = words.get_mut(word) {
            *word |= bit;
        }
    }
    *collected = CollectedItems(words);
}

/// The run's progress as the engine keeps it: her equipment and the emptied places.
#[must_use]
pub fn progress(equipment: &Equipment, collected: &CollectedItems) -> Progress {
    Progress {
        inventory: inventory(equipment),
        emptied: emptied(collected),
        ..Progress::default()
    }
}

/// Whether the item PLM with room argument `argument` finds its place emptied (`$84:887C`).
#[must_use]
pub fn is_emptied(collected: &CollectedItems, argument: u16) -> bool {
    place(argument).is_some_and(|place| emptied(collected).contains(place))
}

/// The model's area for a room's, or none for the debug rooms, which no state condition reads.
#[must_use]
pub const fn region(area: Area) -> Option<Region> {
    Some(match area {
        Area::Crateria => Region::Crateria,
        Area::Brinstar => Region::Brinstar,
        Area::Norfair => Region::Norfair,
        Area::WreckedShip => Region::WreckedShip,
        Area::Maridia => Region::Maridia,
        Area::Tourian => Region::Tourian,
        Area::Ceres => Region::Ceres,
        Area::Debug => return None,
    })
}

/// She takes the item of the item PLM with room argument `argument`: the item is hers and
/// switched on, and its place is emptied (`$84:8899`, `$84:88F3`).
pub fn take(state: &mut State, item: Item, argument: u16) {
    let mut progress = progress(&state.samus.equipment, &state.collected_items);
    let report = Report::Taken {
        pickup: Pickup::Item(piece(item)),
        location: place(argument),
    };
    progress.apply(&Rules::reference(), &[report]);
    store_inventory(&progress.inventory, &mut state.samus.equipment);
    store_emptied(&progress.emptied, &mut state.collected_items);
}

/// The model's condition for the decoder's.
fn condition(condition: room::Condition) -> Option<Condition> {
    Some(match condition {
        room::Condition::Door(door) => Condition::EnteredBy(Entrance(door.offset)),
        room::Condition::MainAreaBossDead => Condition::Defeated(BossBits::MAIN_BOSS.kinds()?),
        room::Condition::Event(event) => Condition::Happened(event),
        room::Condition::BossDead(bits) => Condition::Defeated(bits.kinds()?),
        room::Condition::MorphBall => Condition::Has(Piece::MorphBall),
        room::Condition::MorphBallAndMissiles => Condition::MorphBallAndMissiles,
        room::Condition::PowerBombs => Condition::PowerBombs,
        room::Condition::SpeedBooster => Condition::Has(Piece::SpeedBooster),
    })
}

/// The model's condition for a record's: the same terms, a doorway its entrance. The two are
/// separate types (`data::assets::world::Condition` says why), so this match names every
/// variant: a new one is a compile error here, not a condition read as another.
const fn version_condition(condition: world::Condition) -> Condition {
    match condition {
        world::Condition::EnteredBy(doorway) => Condition::EnteredBy(entrance(doorway)),
        world::Condition::Happened(event) => Condition::Happened(event),
        world::Condition::Defeated(kinds) => Condition::Defeated(kinds),
        world::Condition::Has(item) => Condition::Has(item),
        world::Condition::MorphBallAndMissiles => Condition::MorphBallAndMissiles,
        world::Condition::PowerBombs => Condition::PowerBombs,
    }
}

/// The entrance the model knows a doorway by.
const fn entrance(doorway: DoorwayId) -> Entrance {
    Entrance(doorway.0)
}

/// The conditions of the room's conditional states, in its order, as `RoomStates` reads them;
/// none when a condition names a boss bit no kind has.
#[must_use]
pub fn conditions(room: &Room) -> Option<Vec<Condition>> {
    room.conditional
        .iter()
        .map(|entry| condition(entry.condition))
        .collect()
}

/// The place in a room's list of the version that loads: the default comes after the
/// `conditional` ones.
#[must_use]
pub const fn state_place(version: Version, conditional: usize) -> usize {
    match version {
        Version::Conditional(place) => place,
        Version::Default => conditional,
    }
}

/// The place in a room's list of the version that loads for `progress` in a room of `area`
/// with the `conditional` versions' conditions, entered by `entered_by` if any (`$8F:E5D2`);
/// none when the model can't read its area.
fn chosen_place(
    area: Area,
    conditional: &[Condition],
    progress: &Progress,
    entered_by: Option<Entrance>,
) -> Option<usize> {
    let entered = EnteredRoom {
        area: region(area)?,
        entered_by,
    };
    let version = RoomStates { conditional }.choose(progress, entered);
    Some(state_place(version, conditional.len()))
}

/// The state `room` loads in for `progress`, entered by the door at `door` if any; none when
/// the model can't read its conditions.
#[must_use]
pub fn room_state(
    room: &Room,
    progress: &Progress,
    door: Option<RomAddress>,
) -> Option<StateIndex> {
    let conditional = conditions(room)?;
    let entered_by = door.map(|door| Entrance(door.offset));
    let place = chosen_place(room.area, &conditional, progress, entered_by)?;
    u8::try_from(place).ok().map(StateIndex)
}

/// The place in `room`'s list of the version it loads in for `progress`, entered through
/// `doorway` if any; none when the model can't read its area.
#[must_use]
pub fn version_place(
    room: &world::Room,
    progress: &Progress,
    doorway: Option<DoorwayId>,
) -> Option<usize> {
    let conditional: Vec<_> = room
        .conditional
        .iter()
        .map(|entry| version_condition(entry.condition))
        .collect();
    chosen_place(
        room.place.area,
        &conditional,
        progress,
        doorway.map(entrance),
    )
}

/// The version `room` loads in for `progress`, entered through `doorway` if any; none when
/// the model can't read its area.
#[must_use]
pub fn room_version<'a>(
    room: &'a world::Room,
    progress: &Progress,
    doorway: Option<DoorwayId>,
) -> Option<&'a world::Version> {
    room.version(version_place(room, progress, doorway)?)
}

#[cfg(test)]
mod tests;
