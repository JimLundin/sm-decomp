//! The values the progress rules read: the reference profile holds the original game's.

use core::num::NonZeroU16;

use enum_map::{EnumMap, enum_map};
use types::Beam;

use crate::inventory::Supply;

/// The values the rules of a run's progress read. `Rules::reference()` is the original game;
/// any other value is an adjusted game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    /// For each beam she takes, the beams that taking it switches off: `switches_off[taken]`.
    /// Only the row of the beam taken is read, never the rows that hold it. Spec: TakingBeams.
    pub switches_off: EnumMap<Beam, Vec<Beam>>,
    /// For each supply, the capacity that counts one towards the item percentage: what the
    /// capacity is divided by, rounding down. Spec: ItemPercentage.
    pub divisors: EnumMap<Supply, NonZeroU16>,
}

impl Rules {
    /// The original game's values.
    ///
    /// Spec: TakingBeams. Spazer and plasma don't fire together, so taking either switches the
    /// other off; taking any other beam switches none off.
    ///
    /// Spec: ItemPercentage. Energy and the reserve count one for each 100 of capacity;
    /// missiles, super missiles and power bombs one for each 5.
    #[must_use]
    pub fn reference() -> Self {
        Self {
            switches_off: enum_map! {
                Beam::Charge => Vec::new(),
                Beam::Ice => Vec::new(),
                Beam::Plasma => Vec::from([Beam::Spazer]),
                Beam::Spazer => Vec::from([Beam::Plasma]),
                Beam::Wave => Vec::new(),
            },
            divisors: enum_map! {
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
