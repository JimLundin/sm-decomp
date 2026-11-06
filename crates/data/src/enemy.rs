//! Enemy populations (bank `$A1`) and the species headers they name (bank `$A0`), decoded
//! into typed records (bank `$A0` log: "enemy headers", "load enemies").
//!
//! A population is a list of 16-byte placements ended by `$FFFF` and the room's kill quota
//! byte. A placement names its species by the 16-bit pointer to the species' 64-byte header
//! in bank `$A0`; that pointer is the species' stable key, and [`SpeciesMap`] gives each
//! loaded species its dense [`SpeciesId`].

pub mod program;

use crate::id::SpeciesId;
use crate::rom::{AddressError, Lorom, RomAddress};

/// Species headers.
const SPECIES_BANK: u8 = 0xa0;
/// A species header's size.
const HEADER_BYTES: usize = 0x40;
/// A placement's size.
const PLACEMENT_BYTES: usize = 0x10;
/// The word that ends a population in place of a species.
const END_OF_POPULATION: u16 = 0xffff;

/// A species' stable key: the pointer to its header in bank `$A0`, as an enemy slot's
/// species word holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpeciesKey(pub u16);

impl SpeciesKey {
    /// The Morph Ball Room's eye (`$A0:E6BF`), placed as two parts.
    pub const MORPH_BALL_EYE: Self = Self(0xe6bf);
    /// The elevator platform (`$A0:D73F`).
    pub const ELEVATOR: Self = Self(0xd73f);
    /// The Blue Brinstar face block (`$A0:EA7F`).
    pub const BLUE_BRINSTAR_FACE_BLOCK: Self = Self(0xea7f);

    /// Where the header is.
    #[must_use]
    pub const fn address(self) -> RomAddress {
        RomAddress::new(SPECIES_BANK, self.0)
    }
}

/// Which concrete AI a species runs (D14): one variant per species the engine knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Behaviour {
    /// The Morph Ball Room's eye.
    MorphBallEye,
    /// The elevator platform.
    Elevator,
    /// A Blue Brinstar face block.
    FaceBlock,
}

/// The species slice 0 loads, in load order, with the AI each runs.
pub const SLICE_ZERO_SPECIES: [(SpeciesKey, Behaviour); 3] = [
    (SpeciesKey::MORPH_BALL_EYE, Behaviour::MorphBallEye),
    (SpeciesKey::ELEVATOR, Behaviour::Elevator),
    (SpeciesKey::BLUE_BRINSTAR_FACE_BLOCK, Behaviour::FaceBlock),
];

/// The dense IDs of the loaded species: each species' ID is its place in the load list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpeciesMap {
    species: Vec<(SpeciesKey, Behaviour)>,
}

impl SpeciesMap {
    /// The map of `species`, in order, or `None` if a key repeats or there are too many.
    #[must_use]
    pub fn new(species: Vec<(SpeciesKey, Behaviour)>) -> Option<Self> {
        let distinct = species.iter().enumerate().all(|(index, (key, _))| {
            !species
                .iter()
                .take(index)
                .any(|(earlier, _)| earlier == key)
        });
        (distinct && u16::try_from(species.len()).is_ok()).then_some(Self { species })
    }

    /// The map of the species slice 0 loads.
    #[must_use]
    pub fn slice_zero() -> Self {
        Self {
            species: SLICE_ZERO_SPECIES.to_vec(),
        }
    }

    /// The ID and behaviour of the species with `key`.
    #[must_use]
    pub fn get(&self, key: SpeciesKey) -> Option<(SpeciesId, Behaviour)> {
        self.species
            .iter()
            .position(|&(known, _)| known == key)
            .and_then(|index| {
                let (_, behaviour) = *self.species.get(index)?;
                Some((SpeciesId(u16::try_from(index).ok()?), behaviour))
            })
    }

    /// Each species' key, ID and behaviour, in ID order.
    pub fn iter(&self) -> impl Iterator<Item = (SpeciesKey, SpeciesId, Behaviour)> + '_ {
        self.species
            .iter()
            .zip(0_u16..)
            .map(|(&(key, behaviour), id)| (key, SpeciesId(id), behaviour))
    }
}

/// A routine in the species' AI bank, named by its 16-bit entry point. It's a content
/// reference: the engine runs the species' AI by [`Behaviour`], not by address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Routine(pub RomAddress);

/// The routines a species header names, each in the species' AI bank.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Routines {
    /// Run once at spawn.
    pub init: Routine,
    /// Run each frame.
    pub main: Routine,
    /// Run while grappled.
    pub grapple: Routine,
    /// Run while hurt.
    pub hurt: Routine,
    /// Run while frozen.
    pub frozen: Routine,
    /// Run while time is frozen (the X-ray scope).
    pub time_frozen: Routine,
    /// The power bomb reaction.
    pub power_bomb: Routine,
    /// Samus touched it.
    pub touch: Routine,
    /// A projectile hit it.
    pub shot: Routine,
}

/// A species header (bank `$A0` log: "enemy headers", 64 bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Species {
    /// Its key.
    pub key: SpeciesKey,
    /// Bytes of tile data to upload.
    pub tile_bytes: u16,
    /// Its palette, in its AI bank.
    pub palette: RomAddress,
    /// Health at spawn.
    pub health: u16,
    /// Contact damage to Samus.
    pub damage: u16,
    /// Half its width.
    pub radius_x: u16,
    /// Half its height.
    pub radius_y: u16,
    /// The bank its AI, instruction lists and palette are in.
    pub ai_bank: u8,
    /// Frames it flashes when hurt.
    pub hurt_flash: u8,
    /// The sound it plays when hurt.
    pub cry: u16,
    /// Its boss value, zero for an ordinary enemy.
    pub boss: u16,
    /// Its parts: the slots it fills.
    pub parts: u16,
    /// Its AI routines.
    pub routines: Routines,
    /// Its death animation.
    pub death_animation: u16,
    /// Its tile data.
    pub tiles: RomAddress,
    /// The layer it draws on.
    pub layer: u8,
    /// Its item drop chances, in bank `$B4`.
    pub drops: u16,
    /// Its vulnerabilities, in bank `$B4`.
    pub vulnerabilities: u16,
    /// Its debug name, in bank `$B4`.
    pub name: u16,
}

/// One enemy a population places (bank `$A1`, 16 bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    /// Its species.
    pub species: SpeciesKey,
    /// Its centre's X in the room.
    pub x: u16,
    /// Its centre's Y in the room.
    pub y: u16,
    /// The initialisation parameter its init AI reads (often an instruction list).
    pub init: u16,
    /// Its properties word, as the slot's properties start.
    pub properties: u16,
    /// Its extra properties word.
    pub extra_properties: u16,
    /// The first species-defined parameter.
    pub parameter_1: u16,
    /// The second species-defined parameter.
    pub parameter_2: u16,
}

/// A room state's enemy population.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Population {
    /// Where it is.
    pub at: RomAddress,
    /// The placements, in slot order.
    pub placements: Vec<Placement>,
    /// Enemies Samus must kill before the room counts as cleared.
    pub kill_quota: u8,
}

/// Why an enemy record didn't decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyError {
    /// A read outside the image.
    Address(AddressError),
    /// A record runs past its bank's end.
    BankEnd(RomAddress),
}

impl From<AddressError> for EnemyError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// The `N` bytes at `at`.
fn take<const N: usize>(rom: Lorom<'_>, at: RomAddress) -> Result<[u8; N], EnemyError> {
    <[u8; N]>::try_from(rom.read(at, N)?).map_err(|_| EnemyError::BankEnd(at))
}

/// The little-endian words of `bytes`.
fn words<const N: usize, const W: usize>(bytes: [u8; N]) -> [u16; W] {
    std::array::from_fn(|index| {
        let low = bytes
            .get(index.wrapping_mul(2))
            .copied()
            .unwrap_or_default();
        let high = bytes
            .get(index.wrapping_mul(2).wrapping_add(1))
            .copied()
            .unwrap_or_default();
        u16::from_le_bytes([low, high])
    })
}

/// Decode the population at `at`.
///
/// # Errors
///
/// [`EnemyError`] if a placement or the terminator is outside the image or its bank.
pub fn population(rom: Lorom<'_>, at: RomAddress) -> Result<Population, EnemyError> {
    let mut placements = Vec::new();
    let mut cursor = at;
    loop {
        let [species] = words::<2, 1>(take(rom, cursor)?);
        if species == END_OF_POPULATION {
            let quota_at = cursor.advance(2).ok_or(EnemyError::BankEnd(cursor))?;
            let [kill_quota] = take(rom, quota_at)?;
            return Ok(Population {
                at,
                placements,
                kill_quota,
            });
        }
        let [
            _,
            x,
            y,
            init,
            properties,
            extra_properties,
            parameter_1,
            parameter_2,
        ] = words::<PLACEMENT_BYTES, 8>(take(rom, cursor)?);
        placements.push(Placement {
            species: SpeciesKey(species),
            x,
            y,
            init,
            properties,
            extra_properties,
            parameter_1,
            parameter_2,
        });
        cursor = cursor
            .advance(PLACEMENT_BYTES)
            .ok_or(EnemyError::BankEnd(cursor))?;
    }
}

/// Decode the species header with `key`.
///
/// # Errors
///
/// [`EnemyError`] if the header is outside the image or its bank.
pub fn species(rom: Lorom<'_>, key: SpeciesKey) -> Result<Species, EnemyError> {
    let bytes: [u8; HEADER_BYTES] = take(rom, key.address())?;
    let word: [u16; 32] = words(bytes);
    let at = |index: usize| word.get(index).copied().unwrap_or_default();
    let byte = |offset: usize| bytes.get(offset).copied().unwrap_or_default();
    let ai_bank = byte(0x0c);
    let routine = |index: usize| Routine(RomAddress::new(ai_bank, at(index)));
    Ok(Species {
        key,
        tile_bytes: at(0x00),
        palette: RomAddress::new(ai_bank, at(0x01)),
        health: at(0x02),
        damage: at(0x03),
        radius_x: at(0x04),
        radius_y: at(0x05),
        ai_bank,
        hurt_flash: byte(0x0d),
        cry: at(0x07),
        boss: at(0x08),
        parts: at(0x0a),
        routines: Routines {
            init: routine(0x09),
            main: routine(0x0c),
            grapple: routine(0x0d),
            hurt: routine(0x0e),
            frozen: routine(0x0f),
            time_frozen: routine(0x10),
            power_bomb: routine(0x14),
            touch: routine(0x18),
            shot: routine(0x19),
        },
        death_animation: at(0x11),
        tiles: RomAddress::from_long([byte(0x36), byte(0x37), byte(0x38)]),
        layer: byte(0x39),
        drops: at(0x1d),
        vulnerabilities: at(0x1e),
        name: at(0x1f),
    })
}

/// The loaded species, in [`SpeciesMap`] order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeciesTable {
    map: SpeciesMap,
    species: Vec<Species>,
}

impl SpeciesTable {
    /// Decode every species in `map`.
    ///
    /// # Errors
    ///
    /// [`EnemyError`] if a header doesn't decode.
    pub fn load(rom: Lorom<'_>, map: SpeciesMap) -> Result<Self, EnemyError> {
        let species = map
            .iter()
            .map(|(key, _, _)| species(rom, key))
            .collect::<Result<_, _>>()?;
        Ok(Self { map, species })
    }

    /// The map from keys to IDs.
    #[must_use]
    pub const fn map(&self) -> &SpeciesMap {
        &self.map
    }

    /// The species with `id`.
    #[must_use]
    pub fn get(&self, id: SpeciesId) -> Option<&Species> {
        self.species.get(usize::from(id.0))
    }
}

#[cfg(test)]
mod tests;
