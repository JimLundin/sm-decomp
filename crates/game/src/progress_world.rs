//! Progress as the engine meets it: the progress model (`clean_model_progress`).
//!
//! The engine keeps slice 0's part of what the game remembers of a run as the game keeps it:
//! what she has in `Samus::equipment` and the emptied item places in `State::collected_items`,
//! one bit per room argument (`docs/behaviours/progress.decode.md`, "State"). The model's
//! `Progress` is built from those, takes the report and is written back; the room's state is
//! the model's choice over the room's conditions as the importer decoded them.
//!
//! The engine holds no events, bosses, beams, doors, maps or stations yet: a report of them has
//! nowhere to be kept, so only `Taken` is made, by the item PLM's pickup.

use std::collections::BTreeSet;

use clean_model_progress::{
    Collection, Condition, Count, Inventory, PerSupply, Pickup, Progress, Report, ReserveMode,
    RoomEntry, RoomStates, Rules,
};
use clean_types::{Area as Region, BossKind, Entrance, Item as Piece, LocationId};
use data::rom::RomAddress;
use data::room::{self, Area, BossBits, Room, StateIndex};

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

/// The boss kinds by their bit in an area's boss byte (`$80:81A6`'s list).
const BOSS_KINDS: [(u8, BossKind); 3] = [
    (1, BossKind::MainBoss),
    (2, BossKind::MiniBoss),
    (4, BossKind::Torizo),
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
fn pieces(items: Items) -> BTreeSet<Piece> {
    ITEMS
        .into_iter()
        .filter(|&item| items.contains(item))
        .map(piece)
        .collect()
}

/// The engine's set of the model's items.
fn items(pieces: &BTreeSet<Piece>) -> Items {
    let mut items = Items::default();
    for item in ITEMS
        .into_iter()
        .filter(|&item| pieces.contains(&piece(item)))
    {
        items.insert(item);
    }
    items
}

/// The model's reserve mode for the engine's.
const fn mode(mode: samus::ReserveMode) -> ReserveMode {
    match mode {
        samus::ReserveMode::None => ReserveMode::None,
        samus::ReserveMode::Automatic => ReserveMode::Automatic,
        samus::ReserveMode::Manual => ReserveMode::Manual,
    }
}

/// The engine's reserve mode for the model's.
const fn engine_mode(mode: ReserveMode) -> samus::ReserveMode {
    match mode {
        ReserveMode::None => samus::ReserveMode::None,
        ReserveMode::Automatic => samus::ReserveMode::Automatic,
        ReserveMode::Manual => samus::ReserveMode::Manual,
    }
}

/// The gauges' capacities or what she holds, by `part`.
fn supplies(equipment: &Equipment, part: fn(Gauge) -> u16) -> PerSupply<Count> {
    let count = |gauge| Count::new(part(gauge));
    PerSupply {
        energy: count(equipment.energy),
        reserve: count(equipment.reserve),
        missiles: count(equipment.missiles),
        super_missiles: count(equipment.super_missiles),
        power_bombs: count(equipment.power_bombs),
    }
}

/// What she has, as the model keeps it. The engine has no beams yet: none are hers.
fn inventory(equipment: &Equipment) -> Inventory {
    Inventory {
        items: Collection {
            hers: pieces(equipment.collected),
            switched_on: pieces(equipment.equipped),
        },
        beams: Collection::new(),
        capacity: supplies(equipment, |gauge| gauge.max),
        held: supplies(equipment, |gauge| gauge.current),
        reserve_mode: mode(equipment.reserve_mode),
    }
}

/// `inventory` written back to the engine's equipment.
fn store_inventory(inventory: &Inventory, equipment: &mut Equipment) {
    let gauge = |current: Count, max: Count| Gauge {
        current: current.get(),
        max: max.get(),
    };
    let (held, capacity) = (&inventory.held, &inventory.capacity);
    *equipment = Equipment {
        collected: items(&inventory.items.hers),
        equipped: items(&inventory.items.switched_on),
        energy: gauge(held.energy, capacity.energy),
        reserve: gauge(held.reserve, capacity.reserve),
        missiles: gauge(held.missiles, capacity.missiles),
        super_missiles: gauge(held.super_missiles, capacity.super_missiles),
        power_bombs: gauge(held.power_bombs, capacity.power_bombs),
        reserve_mode: engine_mode(inventory.reserve_mode),
    };
}

/// What she can do and how she looks, as her motion reads it: the model's answer from the items
/// switched on.
#[must_use]
pub fn equipment(equipment: &Equipment) -> clean_types::Equipment {
    inventory(equipment).equipment()
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

/// The emptied places the engine keeps, one bit per place.
fn emptied(collected: &CollectedItems) -> BTreeSet<LocationId> {
    collected
        .0
        .iter()
        .zip((0..).step_by(usize::from(BITS_PER_WORD)))
        .flat_map(|(&word, first): (&u16, u16)| {
            (0..BITS_PER_WORD)
                .filter(move |bit| (word >> bit) & 1 != 0)
                .map(move |bit| LocationId(first.wrapping_add(bit)))
        })
        .collect()
}

/// `emptied` written back as the engine keeps it. A place past the engine's bits isn't kept:
/// the game's populations name places below 100h.
fn store_emptied(emptied: &BTreeSet<LocationId>, collected: &mut CollectedItems) {
    let mut words = [0; COLLECTED_ITEM_WORDS];
    for &LocationId(place) in emptied {
        if let Some(word) = words.get_mut(usize::from(place / BITS_PER_WORD)) {
            *word |= 1 << (place % BITS_PER_WORD);
        }
    }
    *collected = CollectedItems(words);
}

/// The run's progress as the engine keeps it: her equipment and the emptied places.
#[must_use]
pub fn progress(equipment: &Equipment, collected: &CollectedItems) -> Progress {
    let mut progress = Progress::new_file();
    progress.inventory = inventory(equipment);
    progress.world.emptied = emptied(collected);
    progress
}

/// Whether the item PLM with room argument `argument` finds its place emptied (`$84:887C`).
#[must_use]
pub fn is_emptied(collected: &CollectedItems, argument: u16) -> bool {
    let world = clean_model_progress::WorldMemory {
        emptied: emptied(collected),
        ..clean_model_progress::WorldMemory::default()
    };
    world.is_emptied(place(argument))
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

/// She takes the item of the item PLM with room argument `argument`, in the frame's `area`:
/// the item is hers and switched on, and its place is emptied (`$84:8899`, `$84:88F3`).
pub fn take(state: &mut State, item: Item, argument: u16) {
    let mut progress = progress(&state.samus.equipment, &state.collected_items);
    let report = Report::Taken {
        pickup: Pickup::Item(piece(item)),
        location: place(argument),
    };
    // Taking an item reads no area (only a boss's, a map's and a station's reports do), so the
    // debug rooms, which the model has no area for, take theirs as area 0's.
    let area = region(state.room.area).unwrap_or(Region::Crateria);
    progress.apply(&Rules::REFERENCE, area, report);
    store_inventory(&progress.inventory, &mut state.samus.equipment);
    store_emptied(&progress.world.emptied, &mut state.collected_items);
}

/// The boss kinds of an area's boss byte, or none when it holds a bit no kind has.
fn kinds(bits: BossBits) -> Option<BTreeSet<BossKind>> {
    let known = BOSS_KINDS.iter().fold(0, |all, &(bit, _)| all | bit);
    (bits.0 & !known == 0).then(|| {
        BOSS_KINDS
            .iter()
            .filter(|&&(bit, _)| bits.0 & bit != 0)
            .map(|&(_, kind)| kind)
            .collect()
    })
}

/// The model's condition for the importer's.
fn condition(condition: room::Condition) -> Option<Condition> {
    Some(match condition {
        room::Condition::Door(door) => Condition::EnteredBy(Entrance(door.offset)),
        room::Condition::MainAreaBossDead => {
            Condition::Defeated(BTreeSet::from([BossKind::MainBoss]))
        }
        room::Condition::Event(event) => Condition::Happened(event),
        room::Condition::BossDead(bits) => Condition::Defeated(kinds(bits)?),
        room::Condition::MorphBall => Condition::Has(Piece::MorphBall),
        room::Condition::MorphBallAndMissiles => Condition::MorphBallAndMissiles,
        room::Condition::PowerBombs => Condition::PowerBombs,
        room::Condition::SpeedBooster => Condition::Has(Piece::SpeedBooster),
    })
}

/// The room's states as the model reads them, the default last; none when a condition names a
/// boss bit no kind has.
#[must_use]
pub fn room_states(room: &Room) -> Option<RoomStates> {
    let conditions = room
        .conditional
        .iter()
        .map(|entry| condition(entry.condition))
        .chain(core::iter::once(Some(Condition::Always)))
        .collect::<Option<Vec<_>>>()?;
    RoomStates::new(conditions)
}

/// The state `room` loads in for `progress`, entered by the door at `door` if any (`$8F:E5D2`);
/// none when the model can't read its conditions.
#[must_use]
pub fn room_state(
    room: &Room,
    progress: &Progress,
    door: Option<RomAddress>,
) -> Option<StateIndex> {
    let entry = RoomEntry {
        area: region(room.area)?,
        entered_by: door.map(|door| Entrance(door.offset)),
    };
    let index = room_states(room)?.choose(progress, entry);
    u8::try_from(index).ok().map(StateIndex)
}

#[cfg(test)]
mod tests;
