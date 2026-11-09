//! The values the progress rules read: the reference profile holds the original game's.

use core::num::NonZeroU16;

use clean_types::Beam;

use crate::inventory::{PerBeam, PerSupply};

/// The values the rules of a run's progress read. `Rules::REFERENCE` is the original game; any
/// other value is an adjusted game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// For each beam, the beam that taking it switches off, if any. Spec: TakingBeams.
    pub beam_switches_off: PerBeam<Option<Beam>>,
    /// What each capacity is divided by, rounding down, to count towards the item percentage.
    /// Spec: ItemPercentage.
    pub percentage_divisors: PerSupply<NonZeroU16>,
}

impl Rules {
    /// The original game's values.
    pub const REFERENCE: Self = Self {
        // In `Beam`'s order: charge, ice, plasma, spazer, wave. Spazer and plasma don't fire
        // together: taking either switches the other off.
        beam_switches_off: PerBeam::new([None, None, Some(Beam::Spazer), Some(Beam::Plasma), None]),
        // In `Supply::ALL`'s order: energy, reserve, missiles, super missiles, power bombs.
        percentage_divisors: PerSupply::new([BY_TANK, BY_TANK, BY_AMMO, BY_AMMO, BY_AMMO]),
    };
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
        Self::REFERENCE
    }
}
