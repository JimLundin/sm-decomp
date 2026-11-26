//! The importer: the colours of the sprites other than hers from the ROM, and the species a
//! room's version loads, the only code that knows where the original keeps them.
//!
//! A species' header names its palette; the graphics set a room state names lists the species
//! it loads, each with the sprite palette row it draws with there. The common sprites' and
//! the flashing enemies' palettes sit at fixed places, and the Ceres doors' two colour sets
//! beside their species' code. A room state's enemy population places the Ceres doors.

use data::assets::actors::{
    Actors, BLUE_BRINSTAR_FACE_BLOCK, BeaconColours, CERES_DOOR, CERES_RIDLEY, CeresDoorColours,
    CeresRidleyColours, DoorShades, ELEVATOR, EnemyGraphics, EnemyTilePatch, EyeShades, GEEGA,
    Getaway, MORPH_BALL_EYE, ROACH, SIDEHOPPER, SUPER_SIDEHOPPER, Species, SpriteRow, ZOOMER,
};
use data::assets::art::{Colour, Palette, ROW_COLOURS};
use data::assets::world::{
    CeresDoorKind, CeresDoorPlacement, CeresVentFacing, CeresVentPlacement, Pair, Side,
};
use data::enemy::{EnemyError, SpeciesKey};
use data::id::SpeciesId;
use data::rom::{AddressError, Lorom, RomAddress};
use std::num::Wrapping;

/// The original's place for each registered species, in id order: its header's pointer in the
/// species bank, which a graphics set names it by.
const PLACES: [(SpeciesId, u16); 10] = [
    (MORPH_BALL_EYE, 0xe6bf),
    (ELEVATOR, 0xd73f),
    (BLUE_BRINSTAR_FACE_BLOCK, 0xea7f),
    (SUPER_SIDEHOPPER, 0xd9bf),
    (CERES_DOOR, CERES_DOOR_PLACE),
    (CERES_RIDLEY, CERES_RIDLEY_PLACE),
    (SIDEHOPPER, 0xd93f),
    (GEEGA, 0xf253),
    (ZOOMER, 0xdcff),
    (ROACH, 0xd87f),
];

/// The Ceres doors' header, which their placements name them by.
const CERES_DOOR_PLACE: u16 = 0xe23f;
/// Ceres Ridley's header, which his placement names him by.
const CERES_RIDLEY_PLACE: u16 = 0xe13f;
/// The Ceres steam vent's header, which its placements name it by.
const CERES_STEAM_PLACE: u16 = 0xe1ff;
/// The species headers' bank.
const SPECIES_BANK: u8 = 0xa0;
/// Where a header holds its palette's pointer, and its bank.
const PALETTE_POINTER: usize = 2;
const PALETTE_BANK: usize = 0x0c;
/// The word that ends a graphics set in place of a species.
const END_OF_SET: u16 = 0xffff;
/// The bytes of a graphics set's entry: the species' pointer and its palette word.
const ENTRY_BYTES: usize = 4;
/// The common sprites' palette and the flashing enemies' white palette.
const COMMON: RomAddress = RomAddress::new(0x9a, 0x81a0);
const FLASH: RomAddress = RomAddress::new(0x9a, 0xfc00);
/// The Ceres shaft beacon's colour sets.
const BEACON: RomAddress = RomAddress::new(0xa6, 0xf871);
/// Ridley's room's two palette rows in Ceres, the door's own and the baby's.
const RIDLEY_ROOM: RomAddress = RomAddress::new(0xa6, 0xe16f);
/// The baby's colour sets: at rest, then the glow's three.
const BABY_SETS: [RomAddress; 4] = [
    RomAddress::new(0xa6, 0xe1f1),
    RomAddress::new(0xa6, 0xe20f),
    RomAddress::new(0xa6, 0xe22d),
    RomAddress::new(0xa6, 0xe24b),
];
/// Ridley's eye's steps, a byte each naming one of the eye's colour sets, up to the byte that
/// ends them; the eye's colour sets follow.
const EYE_STEPS: RomAddress = RomAddress::new(0xa6, 0xe269);
const END_OF_STEPS: u8 = 0xff;
/// The eye's colour sets, each of 3 colours.
const EYE_SETS: RomAddress = RomAddress::new(0xa6, 0xe2aa);
/// The body's colour sets, each of 11 colours, in the order it shows them.
const BODY_SETS: RomAddress = RomAddress::new(0xa6, 0xe30a);
/// Backdrop palettes and the enemy placements for the getaway's two walls.
const GETAWAY_BODY: RomAddress = RomAddress::new(0xa6, 0xa9e3);
const GETAWAY_WALLS: RomAddress = RomAddress::new(0xa6, 0xaa01);
const GETAWAY_LEFT: RomAddress = RomAddress::new(0xa6, 0xaa2f);
const GETAWAY_RIGHT: RomAddress = RomAddress::new(0xa6, 0xaa3f);
/// Ceres Ridley's set-up's load of his tail's hit worth (`$A6:A216`), and the store after it
/// that holds it where his tail's hit reads it (`$7E:7838`): his species header's contact damage
/// is his body's, not his tail's.
const TAIL_WORTH_LOAD: RomAddress = RomAddress::new(0xa6, 0xa216);
const LOAD: u8 = 0xa9;
const TAIL_WORTH_STORE: [u8; 4] = [0x8f, 0x38, 0x78, 0x7e];
/// How many colour sets the eye and the body each have.
const FADE_SETS: usize = 16;
/// Where a bank's ROM half starts.
const ROM_HALF: u16 = 0x8000;
/// The bytes of a palette row: a word a colour.
const ROW_BYTES: usize = 2 * ROW_COLOURS;

/// Why the actors' colours or a graphics set don't import.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorsError {
    /// A table is outside the image.
    Address(AddressError),
    /// A graphics set names a species the registry has no id for.
    Unregistered(RomAddress),
    /// A graphics set gives a species a palette row past the eight.
    Row(RomAddress),
    /// A species header could not be read.
    Species(EnemyError),
    /// Ridley's eye has no steps, its steps have no end byte before its sets, or a step names a
    /// colour set past them.
    EyeSteps,
    /// The code that sets Ridley's tail's hit worth is not the load and store expected there, or
    /// the worth it loads is more than the most whole energy, 32767.
    TailWorth,
}

impl From<AddressError> for ActorsError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// Every registered species' palette, the shared sprite palettes, the Ceres doors' colours and
/// Ceres Ridley's, and his tail's hit worth.
///
/// # Errors
///
/// [`ActorsError::Address`] when a table is outside the image, [`ActorsError::EyeSteps`] when
/// Ridley's eye's steps don't each name one of its sets, [`ActorsError::TailWorth`] when his
/// tail's hit worth isn't set by the code expected or is more than 32767.
pub fn import(rom: Lorom<'_>) -> Result<Actors, ActorsError> {
    let species = PLACES
        .iter()
        .map(|&(id, pointer)| {
            let header = RomAddress::new(SPECIES_BANK, pointer);
            let [bank, _] = word(rom, header, PALETTE_BANK)?.to_le_bytes();
            let pointer = word(rom, header, PALETTE_POINTER)?;
            // A pointer outside a bank's ROM half names no palette of the species' own.
            let palette = (pointer >= ROM_HALF)
                .then(|| palette(rom, RomAddress::new(bank, pointer)))
                .transpose()?;
            Ok(Species { id, palette })
        })
        .collect::<Result<_, AddressError>>()?;
    let doors = data::palette::CeresDoorColours::load(rom)?;
    Ok(Actors {
        species,
        common: palette(rom, COMMON)?,
        flash: palette(rom, FLASH)?,
        ceres_doors: CeresDoorColours {
            calm: shades(doors.calm),
            escape: shades(doors.escape),
        },
        beacon: beacon(rom)?,
        ceres_ridley: ceres_ridley_colours(rom)?,
        ceres_tail_worth: ceres_tail_worth(rom)?,
    })
}

/// The whole energy Ceres Ridley's tail's hit is worth: the operand of his set-up's load, read
/// only where that load and the store after it are the ones expected, and only up to the most
/// whole energy, [`MOST_WORTH`].
fn ceres_tail_worth(rom: Lorom<'_>) -> Result<u16, ActorsError> {
    let code = rom.read(TAIL_WORTH_LOAD, 7)?;
    match code {
        [LOAD, low, high, store @ ..] if *store == TAIL_WORTH_STORE => {
            Some(u16::from_le_bytes([*low, *high]))
                .filter(|&worth| worth <= MOST_WORTH)
                .ok_or(ActorsError::TailWorth)
        }
        _ => Err(ActorsError::TailWorth),
    }
}

/// The most whole energy a hit can be worth: Health's whole energy runs 0..=32767.
const MOST_WORTH: u16 = 0x7fff;

/// The shaft beacon's colour sets: its table holds a row of eight colours for each step, whose
/// first six are the step's set.
fn beacon(rom: Lorom<'_>) -> Result<BeaconColours, AddressError> {
    const ROW: usize = 8;
    let bytes = rom.read(BEACON, 2 * ROW * BeaconColours::STEPS)?;
    let mut sets = [[Colour::default(); BeaconColours::SHADES]; BeaconColours::STEPS];
    for (set, row) in sets.iter_mut().zip(bytes.as_chunks::<2>().0.chunks(ROW)) {
        for (colour, &word) in set.iter_mut().zip(row) {
            *colour = Colour(u16::from_le_bytes(word));
        }
    }
    Ok(BeaconColours { sets })
}

/// The Ceres door actors the enemy population at `at`, a room state's, places, in its order.
///
/// A door's first parameter names what it is: a door facing right or left, the shaft's door,
/// or an actor a later piece of the station models.
///
/// # Errors
///
/// [`CeresActorsError::Population`] when the population runs outside the image or its bank,
/// and [`CeresActorsError::Kind`] when a door's first parameter names none of these.
pub fn ceres_doors(
    rom: Lorom<'_>,
    at: RomAddress,
) -> Result<Vec<CeresDoorPlacement>, CeresActorsError> {
    let door = SpeciesKey(CERES_DOOR_PLACE);
    data::enemy::population(rom, at)
        .map_err(CeresActorsError::Population)?
        .placements
        .iter()
        .enumerate()
        .filter(|(_, placed)| placed.species == door)
        .map(|(slot, placed)| {
            Ok(CeresDoorPlacement {
                slot,
                kind: ceres_door_kind(placed.parameter_1)
                    .ok_or(CeresActorsError::Kind(placed.parameter_1))?,
                position: Pair {
                    x: Wrapping(placed.x),
                    y: Wrapping(placed.y),
                },
            })
        })
        .collect()
}

/// Whether the enemy population at `at`, a room state's, places Ceres Ridley.
///
/// # Errors
///
/// [`CeresActorsError::Population`] when the population runs outside the image or its bank.
pub fn ceres_ridley(rom: Lorom<'_>, at: RomAddress) -> Result<bool, CeresActorsError> {
    let ridley = SpeciesKey(CERES_RIDLEY_PLACE);
    Ok(data::enemy::population(rom, at)
        .map_err(CeresActorsError::Population)?
        .placements
        .iter()
        .any(|placed| placed.species == ridley))
}

/// The steam vents the room places, in enemy slot order.
///
/// # Errors
/// The population fails to decode or names an unknown vent direction.
pub fn ceres_vents(
    rom: Lorom<'_>,
    at: RomAddress,
) -> Result<Vec<CeresVentPlacement>, CeresActorsError> {
    data::enemy::population(rom, at)
        .map_err(CeresActorsError::Population)?
        .placements
        .iter()
        .enumerate()
        .filter(|(_, placed)| placed.species == SpeciesKey(CERES_STEAM_PLACE))
        .map(|(slot, placed)| {
            let facing = match placed.parameter_1 {
                0 => CeresVentFacing::Up,
                1 => CeresVentFacing::Left,
                2 => CeresVentFacing::Down,
                3 => CeresVentFacing::Right,
                4 => CeresVentFacing::ShaftLeft,
                5 => CeresVentFacing::ShaftRight,
                other => return Err(CeresActorsError::Kind(other)),
            };
            Ok(CeresVentPlacement {
                slot,
                facing,
                position: Pair {
                    x: Wrapping(placed.x),
                    y: Wrapping(placed.y),
                },
            })
        })
        .collect()
}

/// What a Ceres door actor's first parameter names it: 2 is the shaft's overlay, 3 the door of
/// Ridley's room, and 5 and 6 the getaway's walls at his room's left and right.
const fn ceres_door_kind(parameter: u16) -> Option<CeresDoorKind> {
    match parameter {
        0 => Some(CeresDoorKind::Door(Side::Right)),
        1 => Some(CeresDoorKind::Door(Side::Left)),
        2 => Some(CeresDoorKind::ShaftOverlay),
        3 => Some(CeresDoorKind::RidleyDoor),
        4 => Some(CeresDoorKind::ShaftDoor),
        5 => Some(CeresDoorKind::GetawayWall(Side::Left)),
        6 => Some(CeresDoorKind::GetawayWall(Side::Right)),
        _ => None,
    }
}

/// Why a population's Ceres actors don't import.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CeresActorsError {
    /// The population doesn't decode.
    Population(EnemyError),
    /// An unknown first parameter for a station actor.
    Kind(u16),
}

/// The graphics set at `at`, a room state's: each species it loads, in order, with its row.
///
/// # Errors
///
/// [`ActorsError`] when the set runs outside the image, or names a species we have no id for
/// or a row past the eight.
pub fn graphics_set(rom: Lorom<'_>, at: RomAddress) -> Result<Vec<EnemyGraphics>, ActorsError> {
    let mut set = Vec::new();
    let mut place = at;
    loop {
        let species = word(rom, place, 0)?;
        if species == END_OF_SET {
            return Ok(set);
        }
        let named = RomAddress::new(SPECIES_BANK, species);
        let id = PLACES
            .iter()
            .find_map(|&(id, pointer)| (pointer == species).then_some(id))
            .ok_or(ActorsError::Unregistered(named))?;
        let [row, _] = word(rom, place, 2)?.to_le_bytes();
        set.push(EnemyGraphics {
            species: id,
            row: SpriteRow::new(row).ok_or(ActorsError::Row(named))?,
        });
        place = place
            .advance(ENTRY_BYTES)
            .ok_or(AddressError::PastEnd(place))?;
    }
}

/// Import the common tiles and the writes named by this room's starting enemy set.
///
/// # Errors
/// A set entry or tile source outside the image is refused.
pub fn starting_enemy_tiles(
    rom: Lorom<'_>,
    at: RomAddress,
) -> Result<Vec<EnemyTilePatch>, ActorsError> {
    let common = rom.read(RomAddress::new(0x9a, 0xea00), 0x200)?;
    let mut patches = vec![EnemyTilePatch {
        offset: 0,
        bytes: common.to_vec(),
    }];
    let mut next = 0x800_u16;
    let mut place = at;
    loop {
        let key = word(rom, place, 0)?;
        if key == END_OF_SET {
            return Ok(patches);
        }
        let species = data::enemy::species(rom, SpeciesKey(key)).map_err(ActorsError::Species)?;
        let row = word(rom, place, 2)?;
        let size = usize::from(species.tile_bytes & 0x7fff);
        let offset = if species.tile_bytes & 0x8000 == 0 {
            next
        } else {
            // The upper bits in the palette word select a reserved object tile slot.
            (row & 0x3000) >> 3
        };
        patches.push(EnemyTilePatch {
            offset: usize::from(offset),
            bytes: rom.read(species.tiles, size)?.to_vec(),
        });
        next = next.wrapping_add(species.tile_bytes);
        place = place
            .advance(ENTRY_BYTES)
            .ok_or(AddressError::PastEnd(place))?;
    }
}

/// Ridley's room's colours in Ceres, the baby's four colour sets, each of colours 1 to 15, and
/// the sets his eye and his body fade in through, the eye's by its steps.
fn ceres_ridley_colours(rom: Lorom<'_>) -> Result<CeresRidleyColours, ActorsError> {
    let [rest, glow1, glow2, glow3] = BABY_SETS;
    let eye_sets: Vec<EyeShades> = colour_sets(rom, EYE_SETS)?;
    // The steps and their end byte sit between the steps' start and the eye's sets.
    let space = usize::from(EYE_SETS.offset.saturating_sub(EYE_STEPS.offset));
    let bytes = rom.read(EYE_STEPS, space)?;
    let end = bytes
        .iter()
        .position(|&step| step == END_OF_STEPS)
        .ok_or(ActorsError::EyeSteps)?;
    let eye_steps = bytes
        .iter()
        .take(end)
        .map(|&step| eye_sets.get(usize::from(step)).copied())
        .collect::<Option<Vec<_>>>()
        .filter(|steps| !steps.is_empty())
        .ok_or(ActorsError::EyeSteps)?;
    let wall_position = |at| -> Result<_, AddressError> {
        Ok(Pair {
            x: Wrapping(word(rom, at, 2)?),
            y: Wrapping(word(rom, at, 4)?),
        })
    };
    Ok(CeresRidleyColours {
        getaway: Getaway {
            left_wall: wall_position(GETAWAY_LEFT)?,
            right_wall: wall_position(GETAWAY_RIGHT)?,
            body_colours: colour_set(rom, GETAWAY_BODY)?,
            wall_colours: colour_set(rom, GETAWAY_WALLS)?,
        },
        eye_steps,
        body_sets: colour_sets(rom, BODY_SETS)?,
        door: palette(rom, RIDLEY_ROOM)?,
        baby: palette(
            rom,
            RIDLEY_ROOM
                .advance(ROW_BYTES)
                .ok_or(AddressError::PastEnd(RIDLEY_ROOM))?,
        )?,
        rest: colour_set(rom, rest)?,
        glow1: colour_set(rom, glow1)?,
        glow2: colour_set(rom, glow2)?,
        glow3: colour_set(rom, glow3)?,
    })
}

/// The colour set at `at`: `N` little-endian colour words.
fn colour_set<const N: usize>(rom: Lorom<'_>, at: RomAddress) -> Result<[Colour; N], AddressError> {
    let bytes = rom.read(at, N.saturating_mul(2))?;
    let mut shades = [Colour::default(); N];
    for (shade, &word) in shades.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *shade = Colour(u16::from_le_bytes(word));
    }
    Ok(shades)
}

/// The fade's [`FADE_SETS`] colour sets of `N` colours each, one after another from `at`.
fn colour_sets<const N: usize>(
    rom: Lorom<'_>,
    at: RomAddress,
) -> Result<Vec<[Colour; N]>, AddressError> {
    (0..FADE_SETS)
        .map(|set| {
            let place = at.advance(N.saturating_mul(2).saturating_mul(set));
            colour_set(rom, place.ok_or(AddressError::PastEnd(at))?)
        })
        .collect()
}

/// The palette row at `at`: 16 little-endian colour words.
pub(crate) fn palette(rom: Lorom<'_>, at: RomAddress) -> Result<Palette, AddressError> {
    let bytes: &[u8; ROW_BYTES] = rom
        .read(at, ROW_BYTES)?
        .try_into()
        .map_err(|_| AddressError::PastEnd(at))?;
    Ok(palette_of(bytes))
}

/// The colours of a row's 32 bytes.
fn palette_of(bytes: &[u8; ROW_BYTES]) -> Palette {
    let mut colours = [Colour::default(); ROW_COLOURS];
    for (colour, &word) in colours.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *colour = Colour(u16::from_le_bytes(word));
    }
    Palette(colours)
}

/// A door colour set's words as colours.
fn shades(words: data::palette::DoorShades) -> DoorShades {
    words.map(Colour)
}

/// The little-endian word `offset` bytes past `at`.
fn word(rom: Lorom<'_>, at: RomAddress, offset: usize) -> Result<u16, AddressError> {
    let place = at.advance(offset).ok_or(AddressError::PastEnd(at))?;
    let bytes: &[u8; 2] = rom
        .read(place, 2)?
        .try_into()
        .map_err(|_| AddressError::PastEnd(place))?;
    Ok(u16::from_le_bytes(*bytes))
}

#[cfg(test)]
mod tests;

// ROM sources for the data above, kept out of the public docs (the original's bank and RAM
// addresses): bank $A0 log, process enemy set `$A0:8D64`: a set's entry at `$B4:[$07D1]` is
// the species' header pointer (bank $A0) and a word whose low byte is the sprite palette
// row; the species' palette is 20h bytes at the header's `+2` pointer in its `+Ch` bank, set
// at target colours `$7E:C300 + 20h * row`. Bank $82 log, `$82:E139`: the common sprites'
// palette `$9A:81A0` to target sprite palette 5, the flashing enemies' `$9A:FC00` to sprite
// palette 0. The Ceres doors' sets are `data::palette`'s. Bank $A6 log, the Ceres door
// overlay's beacon `$A6:F850`: the table `$A6:F871`, 8 rows of 8 colours (the step
// times 10h bytes), the first 6 of a row written to the
// palette buffer `$7E:C052` (colours 29h to 2Eh).
