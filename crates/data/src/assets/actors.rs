//! The colours of the sprites other than hers (docs/architecture/assets.md §3.4).
//!
//! Each species' palette, the palettes every room's sprites share, the Ceres doors' two
//! colour sets and the Ceres shaft beacon's; with the species registry.
//!
//! The ids are ours: `import::actors` maps the original's species places to them, and the
//! registry holds no ROM bytes. They are dense (D15): a species' id is its place in
//! [`SPECIES`]. The names are labels for reading only.

use crate::assets::art::{Colour, Palette, ROW_COLOURS};
use crate::id::SpeciesId;

/// The Morph Ball Room's eye.
pub const MORPH_BALL_EYE: SpeciesId = SpeciesId(0);
/// The elevator platform.
pub const ELEVATOR: SpeciesId = SpeciesId(1);
/// A Blue Brinstar face block.
pub const BLUE_BRINSTAR_FACE_BLOCK: SpeciesId = SpeciesId(2);
/// The super sidehopper, in the Morph Ball Room once the planet is awake.
pub const SUPER_SIDEHOPPER: SpeciesId = SpeciesId(3);
/// A door of the Ceres station.
pub const CERES_DOOR: SpeciesId = SpeciesId(4);
/// Ridley, in Ceres.
pub const CERES_RIDLEY: SpeciesId = SpeciesId(5);
/// The sidehopper, in Green Hill Zone.
pub const SIDEHOPPER: SpeciesId = SpeciesId(6);
/// The geega, in Green Hill Zone.
pub const GEEGA: SpeciesId = SpeciesId(7);
/// The zoomer, in Construction Zone once the planet is awake.
pub const ZOOMER: SpeciesId = SpeciesId(8);
/// The roach, in Construction Zone before the planet is awake.
pub const ROACH: SpeciesId = SpeciesId(9);

/// Every registered species with its name, in id order: the slice 0 species first, in the
/// order `data::enemy`'s species map gives them ids, so the two agree; then those the
/// registered rooms' versions load.
pub const SPECIES: [(SpeciesId, &str); 10] = [
    (MORPH_BALL_EYE, "Morph Ball Eye"),
    (ELEVATOR, "Elevator"),
    (BLUE_BRINSTAR_FACE_BLOCK, "Blue Brinstar Face Block"),
    (SUPER_SIDEHOPPER, "Super Sidehopper"),
    (CERES_DOOR, "Ceres Door"),
    (CERES_RIDLEY, "Ceres Ridley"),
    (SIDEHOPPER, "Sidehopper"),
    (GEEGA, "Geega"),
    (ZOOMER, "Zoomer"),
    (ROACH, "Roach"),
];

/// One of the eight sprite palette rows, 0 to 7: a sprite's palette field picks one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpriteRow(u8);

impl SpriteRow {
    /// The sprite palette rows.
    pub const ROWS: u8 = 8;
    /// The white palette enemies flash with.
    pub const FLASH: Self = Self(0);
    /// Her suit's palette.
    pub const SUIT: Self = Self(4);
    /// The common sprites' palette.
    pub const COMMON: Self = Self(5);
    /// Her shots' palette.
    pub const BEAM: Self = Self(6);
    /// The Ceres doors' palette, as a room places them; Ridley's in his room.
    pub const CERES_DOORS: Self = Self(7);
    /// The own palette of the door of Ridley's room, while he is in it.
    pub const RIDLEY_DOOR: Self = Self(2);
    /// The palette of the baby Ridley holds.
    pub const BABY: Self = Self(3);
    /// Ridley's second palette in Ceres, which his body's fade sets with his own.
    pub const RIDLEY_SECOND: Self = Self(1);

    /// Row `row`, if there is one.
    #[must_use]
    pub const fn new(row: u8) -> Option<Self> {
        if row < Self::ROWS {
            Some(Self(row))
        } else {
            None
        }
    }

    /// Its place among the eight.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// A species a room's version loads, and the sprite palette row it draws with there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnemyGraphics {
    /// The species.
    pub species: SpeciesId,
    /// Its row in this version.
    pub row: SpriteRow,
}

/// A species' colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Species {
    /// Our id.
    pub id: SpeciesId,
    /// The palette it draws with, which its room's loading sets in its row; none for a
    /// species that is drawn with the palette of what carries it, such as the elevator.
    pub palette: Option<Palette>,
}

/// A Ceres door colour set: colours 1 to 15 of the doors' palette; colour 0 is transparent.
pub type DoorShades = [Colour; ROW_COLOURS - 1];

/// The two colour sets the Ceres doors' palette takes when a room places them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CeresDoorColours {
    /// Before Ridley's getaway.
    pub calm: DoorShades,
    /// From the getaway on.
    pub escape: DoorShades,
}

/// A set of the beacon's colours, in the order the screen holds them.
pub type BeaconShades = [Colour; BeaconColours::SHADES];

/// The Ceres shaft's beacon: the colour set of each step of its cycle, and where on the screen
/// they go. The shaft's overlay, an actor of the doors' species, steps it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BeaconColours {
    /// Each step's colour set, by its place in the cycle.
    pub sets: [BeaconShades; Self::STEPS],
}

impl BeaconColours {
    /// The steps of the beacon's cycle.
    pub const STEPS: usize = 8;
    /// The colours a set holds.
    pub const SHADES: usize = 6;
    /// The screen colour a set's first colour goes to: background row 2's colour 9.
    pub const FIRST: usize = 41;
}

/// A colour set of the baby Ridley holds: colours 1 to 15 of its palette.
pub type BabyShades = [Colour; ROW_COLOURS - 1];

/// A colour set of Ceres Ridley's eye: colours 12 to 14 of his palette.
pub type EyeShades = [Colour; 3];

/// A colour set of Ceres Ridley's body: colours 1 to 11 of his two palettes.
pub type BodyShades = [Colour; 11];

/// The colours of Ridley's room in Ceres as he is placed, the baby's colour sets, and the sets
/// his eye and his body fade in through as he wakes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CeresRidleyColours {
    /// The goal colours of the door's own palette, row [`SpriteRow::RIDLEY_DOOR`].
    pub door: Palette,
    /// The goal colours of the baby's palette, row [`SpriteRow::BABY`].
    pub baby: Palette,
    /// The baby's colours at rest.
    pub rest: BabyShades,
    /// The glow's first colour set.
    pub glow1: BabyShades,
    /// Its second.
    pub glow2: BabyShades,
    /// Its third.
    pub glow3: BabyShades,
    /// His eye's steps, in the order it shows them, each the eye's colour set it names.
    pub eye_steps: Vec<EyeShades>,
    /// His body's colour sets, in the order it shows them.
    pub body_sets: Vec<BodyShades>,
    /// Getaway backdrop assets.
    pub getaway: Getaway,
}

/// Positions and palette spans used when Ridley stops coasting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Getaway {
    /// Left wall centre in room pixels.
    pub left_wall: types::Pair<types::Px>,
    /// Right wall centre in room pixels.
    pub right_wall: types::Pair<types::Px>,
    /// Colours 1 through 15 of the backdrop body.
    pub body_colours: [Colour; 15],
    /// Colours 1 through 8 of both wall palettes.
    pub wall_colours: [Colour; 8],
}

impl Default for Getaway {
    fn default() -> Self {
        let unset = types::Pair {
            x: types::Px::default(),
            y: types::Px::default(),
        };
        Self {
            left_wall: unset,
            right_wall: unset,
            body_colours: [Colour::default(); 15],
            wall_colours: [Colour::default(); 8],
        }
    }
}

/// The colours of the sprites other than hers, and the worth of Ceres Ridley's tail's hit.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Actors {
    /// Every registered species, in id order.
    pub species: Vec<Species>,
    /// The palette the common sprites (drops, explosions, smoke) share.
    pub common: Palette,
    /// The white palette an enemy flashes with when hit.
    pub flash: Palette,
    /// The Ceres doors' colour sets.
    pub ceres_doors: CeresDoorColours,
    /// The colour sets the Ceres shaft's beacon steps through.
    pub beacon: BeaconColours,
    /// Ridley's room's colours in Ceres and the baby's colour sets.
    pub ceres_ridley: CeresRidleyColours,
    /// The whole energy Ceres Ridley's tail's hit takes from her: his tail's own, set apart from
    /// the amount his body's touch takes.
    pub ceres_tail_worth: u16,
}

impl Actors {
    /// Species `id`, if it is registered.
    #[must_use]
    pub fn species(&self, id: SpeciesId) -> Option<&Species> {
        self.species.get(usize::from(id.0))
    }
}
