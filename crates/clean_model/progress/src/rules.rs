//! The values the progress rules read: the reference profile holds the original game's.

use core::num::NonZeroU16;

use clean_types::{Beam, by_member};
use enum_map::{Enum, EnumMap};

use crate::inventory::{BeamTaken, Supply};

/// The values the rules of a run's progress read. `Rules::reference()` is the original game;
/// any other value is an adjusted game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// For each beam, the beam that taking it switches off, if any. Spec: TakingBeams.
    pub taking_beams: EnumMap<Beam, BeamTaken>,
    /// What each capacity is divided by, rounding down, to count towards the item percentage.
    /// Spec: ItemPercentage.
    pub percentage_divisors: EnumMap<Supply, PercentageDivisor>,
}

/// A row of the item percentage's divisors: a supply, and what its capacity is divided by,
/// rounding down, to count towards the percentage. Spec: ItemPercentage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PercentageDivisor {
    /// The supply whose capacity counts.
    pub supply: Supply,
    /// The capacity that counts one.
    pub divisor: NonZeroU16,
}

impl Rules {
    /// The original game's values, each table built from its rows.
    #[must_use]
    pub fn reference() -> Self {
        Self {
            taking_beams: by_member(REFERENCE_TAKING_BEAMS, |row| row.beam),
            percentage_divisors: by_member(REFERENCE_DIVISORS, |row| row.supply),
        }
    }
}

/// Each beam and the beam taking it switches off in the original game: spazer and plasma don't
/// fire together, so taking either switches the other off; the rest switch none off.
/// Spec: TakingBeams.
pub const REFERENCE_TAKING_BEAMS: <Beam as Enum>::Array<BeamTaken> = [
    BeamTaken {
        beam: Beam::Charge,
        switches_off: None,
    },
    BeamTaken {
        beam: Beam::Ice,
        switches_off: None,
    },
    BeamTaken {
        beam: Beam::Plasma,
        switches_off: Some(Beam::Spazer),
    },
    BeamTaken {
        beam: Beam::Spazer,
        switches_off: Some(Beam::Plasma),
    },
    BeamTaken {
        beam: Beam::Wave,
        switches_off: None,
    },
];

/// Each supply and what the original game divides its capacity by for the item percentage:
/// energy and reserve 100, missiles, super missiles and power bombs 5. Spec: ItemPercentage.
pub const REFERENCE_DIVISORS: <Supply as Enum>::Array<PercentageDivisor> = [
    PercentageDivisor {
        supply: Supply::Energy,
        divisor: BY_TANK,
    },
    PercentageDivisor {
        supply: Supply::Reserve,
        divisor: BY_TANK,
    },
    PercentageDivisor {
        supply: Supply::Missiles,
        divisor: BY_AMMO,
    },
    PercentageDivisor {
        supply: Supply::SuperMissiles,
        divisor: BY_AMMO,
    },
    PercentageDivisor {
        supply: Supply::PowerBombs,
        divisor: BY_AMMO,
    },
];

/// The energy or reserve capacity that counts one towards the item percentage.
const TANK_DIVISOR: u16 = 100;
/// The missile, super missile or power bomb capacity that counts one.
const AMMO_DIVISOR: u16 = 5;
/// `TANK_DIVISOR`, as a divisor.
const BY_TANK: NonZeroU16 = NonZeroU16::new(TANK_DIVISOR).expect("TANK_DIVISOR isn't 0");
/// `AMMO_DIVISOR`, as a divisor.
const BY_AMMO: NonZeroU16 = NonZeroU16::new(AMMO_DIVISOR).expect("AMMO_DIVISOR isn't 0");

impl Default for Rules {
    fn default() -> Self {
        Self::reference()
    }
}
