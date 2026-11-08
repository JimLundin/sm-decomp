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
            energy: ENERGY_DIVISOR,
            reserve: RESERVE_DIVISOR,
            missiles: MISSILES_DIVISOR,
            super_missiles: SUPER_MISSILES_DIVISOR,
            power_bombs: POWER_BOMBS_DIVISOR,
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
const ENERGY_DIVISOR: Divisor = Divisor::new(100).expect("100 is not zero");
/// Spec: ItemPercentage. Each 100 of reserve capacity counts one.
const RESERVE_DIVISOR: Divisor = Divisor::new(100).expect("100 is not zero");
/// Spec: ItemPercentage. Each 5 of missile capacity counts one.
const MISSILES_DIVISOR: Divisor = Divisor::new(5).expect("5 is not zero");
/// Spec: ItemPercentage. Each 5 of super missile capacity counts one.
const SUPER_MISSILES_DIVISOR: Divisor = Divisor::new(5).expect("5 is not zero");
/// Spec: ItemPercentage. Each 5 of power bomb capacity counts one.
const POWER_BOMBS_DIVISOR: Divisor = Divisor::new(5).expect("5 is not zero");
