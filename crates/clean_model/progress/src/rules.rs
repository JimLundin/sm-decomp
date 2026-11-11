//! The values the progress rules read: the reference profile holds the original game's.

use core::num::NonZeroU16;

use clean_types::Beam;
use enum_map::{EnumMap, enum_map};

use crate::inventory::Supply;

/// The values the rules of a run's progress read. `Rules::reference()` is the original game;
/// any other value is an adjusted game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// For each beam, the beam that taking it switches off, if any. Spec: TakingBeams.
    pub switched_off_by: EnumMap<Beam, Option<Beam>>,
    /// For each supply, the capacity that counts one towards the item percentage: what the
    /// capacity is divided by, rounding down. Spec: ItemPercentage.
    pub percentage_divisors: EnumMap<Supply, NonZeroU16>,
}

impl Rules {
    /// The original game's values.
    ///
    /// Spec: TakingBeams. Spazer and plasma don't fire together, so taking either switches the
    /// other off; the other beams switch none off.
    ///
    /// Spec: ItemPercentage. Energy and the reserve count one for each 100 of capacity;
    /// missiles, super missiles and power bombs one for each 5.
    #[must_use]
    pub fn reference() -> Self {
        Self {
            switched_off_by: enum_map! {
                Beam::Charge => None,
                Beam::Ice => None,
                Beam::Plasma => Some(Beam::Spazer),
                Beam::Spazer => Some(Beam::Plasma),
                Beam::Wave => None,
            },
            percentage_divisors: enum_map! {
                Supply::Energy => BY_TANK,
                Supply::Reserve => BY_TANK,
                Supply::Missiles => BY_AMMO,
                Supply::SuperMissiles => BY_AMMO,
                Supply::PowerBombs => BY_AMMO,
            },
        }
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
