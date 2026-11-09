//! The values the progress rules read: the reference profile holds the original game's.

use core::num::NonZeroU16;

use clean_types::Beam;
use enum_map::EnumMap;

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
    /// The original game's values, each table's row stated by its member.
    #[must_use]
    pub fn reference() -> Self {
        Self {
            beam_switches_off: EnumMap::from_fn(reference_switches_off),
            percentage_divisors: EnumMap::from_fn(reference_divisor),
        }
    }
}

/// The beam taking `beam` switches off in the original game: spazer and plasma don't fire
/// together, so taking either switches the other off; the rest switch none off.
/// Spec: TakingBeams.
const fn reference_switches_off(beam: Beam) -> Option<Beam> {
    match beam {
        Beam::Spazer => Some(Beam::Plasma),
        Beam::Plasma => Some(Beam::Spazer),
        Beam::Charge | Beam::Ice | Beam::Wave => None,
    }
}

/// What the original game divides a capacity by for the item percentage: energy and reserve
/// 100, missiles, super missiles and power bombs 5. Spec: ItemPercentage.
const fn reference_divisor(supply: Supply) -> NonZeroU16 {
    match supply {
        Supply::Energy | Supply::Reserve => BY_TANK,
        Supply::Missiles | Supply::SuperMissiles | Supply::PowerBombs => BY_AMMO,
    }
}

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
