//! The values the progress rules read: the reference profile holds the original game's.

use core::num::Wrapping;

use clean_types::Beam;

use crate::count::Count;
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
    pub percentage_divisors: PerSupply<Count>,
}

impl Rules {
    /// The original game's values.
    pub const REFERENCE: Self = Self {
        // Spazer and plasma don't fire together: taking either switches the other off.
        beams_switched_off: &[
            BeamSwitchesOff {
                taken: Beam::Spazer,
                switched_off: Beam::Plasma,
            },
            BeamSwitchesOff {
                taken: Beam::Plasma,
                switched_off: Beam::Spazer,
            },
        ],
        // In `Supply::ALL`'s order: energy, reserve, missiles, super missiles, power bombs.
        percentage_divisors: PerSupply::new([
            Wrapping(100),
            Wrapping(100),
            Wrapping(5),
            Wrapping(5),
            Wrapping(5),
        ]),
    };
}

impl Default for Rules {
    fn default() -> Self {
        Self::REFERENCE
    }
}
