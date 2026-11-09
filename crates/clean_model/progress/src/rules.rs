//! The values the progress rules read: the reference profile holds the original game's.

use core::num::NonZeroU16;

use clean_types::{Beam, by_member};
use enum_map::{Enum, EnumMap};

use crate::inventory::Supply;

/// The values the rules of a run's progress read. `Rules::reference()` is the original game;
/// any other value is an adjusted game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// For each beam, the beam that taking it switches off, if any. Spec: TakingBeams.
    pub beam_switches_off: EnumMap<Beam, Option<Beam>>,
    /// What each capacity is divided by, rounding down, to count towards the item percentage.
    /// Spec: ItemPercentage.
    pub percentage_divisors: EnumMap<Supply, NonZeroU16>,
}

impl Rules {
    /// The original game's values, each table built from its rows.
    #[must_use]
    pub fn reference() -> Self {
        Self {
            beam_switches_off: values(REFERENCE_SWITCHES_OFF),
            percentage_divisors: values(REFERENCE_DIVISORS),
        }
    }
}

/// A table of the values its rows give, each row a member and its value.
fn values<K, V, const N: usize>(rows: [(K, V); N]) -> EnumMap<K, V>
where
    K: Copy + Enum<Array<(K, V)> = [(K, V); N]>,
{
    by_member(rows, |&(member, _)| member).map(|_, (_, value)| value)
}

/// Each beam and the beam taking it switches off in the original game: spazer and plasma don't
/// fire together, so taking either switches the other off; the rest switch none off.
/// Spec: TakingBeams.
pub const REFERENCE_SWITCHES_OFF: <Beam as Enum>::Array<(Beam, Option<Beam>)> = [
    (Beam::Charge, None),
    (Beam::Ice, None),
    (Beam::Plasma, Some(Beam::Spazer)),
    (Beam::Spazer, Some(Beam::Plasma)),
    (Beam::Wave, None),
];

/// Each supply and what the original game divides its capacity by for the item percentage:
/// energy and reserve 100, missiles, super missiles and power bombs 5. Spec: ItemPercentage.
pub const REFERENCE_DIVISORS: <Supply as Enum>::Array<(Supply, NonZeroU16)> = [
    (Supply::Energy, BY_TANK),
    (Supply::Reserve, BY_TANK),
    (Supply::Missiles, BY_AMMO),
    (Supply::SuperMissiles, BY_AMMO),
    (Supply::PowerBombs, BY_AMMO),
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
