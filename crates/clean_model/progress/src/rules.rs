//! The values the progress rules read: the reference profile holds the original game's.

use clean_types::Beam;

use crate::count::Divisor;
use crate::supplies::PerSupply;

/// One row of the table of beams that don't fire together: taking `taken` switches
/// `switched_off` off. Spec: TakingBeams.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeamSwitchesOff {
    /// The beam she takes.
    pub taken: Beam,
    /// The beam that taking it switches off.
    pub switched_off: Beam,
}

/// The values the rules of a run's progress read. `Rules::REFERENCE` is the original game; any
/// other value is an adjusted game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// The beams that switch another off when taken; a beam with no row switches nothing off.
    /// Spec: TakingBeams.
    pub beams_switched_off: &'static [BeamSwitchesOff],
    /// What each capacity is divided by, rounding down, to count towards the item percentage.
    /// Spec: ItemPercentage.
    pub percentage_divisors: PerSupply<Divisor>,
}

impl Rules {
    /// The original game's values.
    pub const REFERENCE: Self = Self {
        beams_switched_off: SPAZER_AND_PLASMA,
        percentage_divisors: PerSupply {
            energy: Divisor::new(ENERGY_DIVISOR).expect("the energy divisor is not zero"),
            reserve: Divisor::new(RESERVE_DIVISOR).expect("the reserve divisor is not zero"),
            missiles: Divisor::new(MISSILES_DIVISOR).expect("the missile divisor is not zero"),
            super_missiles: Divisor::new(SUPER_MISSILES_DIVISOR)
                .expect("the super missile divisor is not zero"),
            power_bombs: Divisor::new(POWER_BOMBS_DIVISOR)
                .expect("the power bomb divisor is not zero"),
        },
    };

    /// The beam that taking `beam` switches off, if any.
    #[must_use]
    pub(crate) fn switched_off_by(&self, beam: Beam) -> Option<Beam> {
        self.beams_switched_off
            .iter()
            .find(|row| row.taken == beam)
            .map(|row| row.switched_off)
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self::REFERENCE
    }
}

/// Spec: TakingBeams. Spazer and plasma don't fire together: taking either switches the other off.
const SPAZER_AND_PLASMA: &[BeamSwitchesOff] = &[
    BeamSwitchesOff {
        taken: Beam::Spazer,
        switched_off: Beam::Plasma,
    },
    BeamSwitchesOff {
        taken: Beam::Plasma,
        switched_off: Beam::Spazer,
    },
];

/// Spec: ItemPercentage. Each 100 of energy capacity counts one.
const ENERGY_DIVISOR: u16 = 100;
/// Spec: ItemPercentage. Each 100 of reserve capacity counts one.
const RESERVE_DIVISOR: u16 = 100;
/// Spec: ItemPercentage. Each 5 of missile capacity counts one.
const MISSILES_DIVISOR: u16 = 5;
/// Spec: ItemPercentage. Each 5 of super missile capacity counts one.
const SUPER_MISSILES_DIVISOR: u16 = 5;
/// Spec: ItemPercentage. Each 5 of power bomb capacity counts one.
const POWER_BOMBS_DIVISOR: u16 = 5;
