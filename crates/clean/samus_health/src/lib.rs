//! Current energy and warning requests for already qualified health opportunities.
//! Gameplay qualification, loss production, pickups, drawing history and playback
//! belong to neighboring owners, not to this model.

use serde::{Deserialize, Serialize};

mod catalogue;
pub use catalogue::{Behaviour, BehaviourId, behaviour};

/// Spec: `PeriodicLoss`, `LowEnergyWarning`. Largest admitted whole energy count.
pub const MAX_WHOLE_ENERGY: u16 = 32767;
/// Spec: `LowEnergyWarning`. Reference whole-energy warning threshold.
pub const REFERENCE_WARNING_THRESHOLD: WholeEnergy = WholeEnergy(30);

/// Spec: `PeriodicLoss`, `LowEnergyWarning`. A whole count in 0..=32767.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct WholeEnergy(u16);

impl WholeEnergy {
    /// Spec: `PeriodicLoss`. Zero whole energy.
    pub const ZERO: Self = Self(0);

    /// Constructs an admitted whole count, rejecting counts above the spec range.
    #[must_use]
    pub const fn new(value: u16) -> Option<Self> {
        if value <= MAX_WHOLE_ENERGY {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Reads the whole count without selecting a health opportunity.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for WholeEnergy {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value).ok_or("whole energy exceeds 32767")
    }
}

impl From<WholeEnergy> for u16 {
    fn from(value: WholeEnergy) -> Self {
        value.0
    }
}

/// Spec: `PeriodicLoss`. Current energy, including its fraction in 1/65536 units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnergyAmount {
    /// Whole current energy in the admitted range.
    pub whole: WholeEnergy,
    /// Fractional current energy in 0..=65535.
    pub fraction: u16,
}

impl EnergyAmount {
    /// Spec: `PeriodicLoss`. The clamp result has neither whole nor fractional energy.
    pub const ZERO: Self = Self {
        whole: WholeEnergy::ZERO,
        fraction: 0,
    };

    /// Spec: `PeriodicLoss`. Exact fractional borrowing, with an empty-energy clamp.
    const fn reduced_by(self, loss: EnergyLoss) -> Self {
        let borrow = self.fraction < loss.fraction;
        if self.whole.0 < loss.whole.0 || (self.whole.0 == loss.whole.0 && borrow) {
            return Self::ZERO;
        }
        Self {
            whole: WholeEnergy(
                self.whole
                    .0
                    .saturating_sub(loss.whole.0)
                    .saturating_sub(if borrow { 1 } else { 0 }),
            ),
            fraction: self.fraction.wrapping_sub(loss.fraction),
        }
    }
}

/// Spec: `PeriodicLoss`. Independently settable accumulated loss, not a hazard producer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnergyLoss {
    /// Whole pending loss in the admitted range.
    pub whole: WholeEnergy,
    /// Fractional pending loss in 0..=65535, measured in 1/65536 energy.
    pub fraction: u16,
}

impl EnergyLoss {
    /// Spec: `PeriodicLoss`. Every completed opportunity consumes the entire pending loss.
    pub const ZERO: Self = Self {
        whole: WholeEnergy::ZERO,
        fraction: 0,
    };
}

/// Spec: `LowEnergyWarning`. Remembers the requested warning, regardless of playback.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Warning {
    /// No warning is currently requested.
    Off,
    /// A warning is currently requested.
    On,
}

/// Spec: `LowEnergyWarning`. Audio request for this completed opportunity only.
///
/// Scheduling, delivery and audible playback are external responsibilities.
#[must_use = "handle the warning sound request; State has already been updated"]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningSound {
    /// Leaves any existing warning request unchanged; does not request stopping playback.
    None,
    /// Entering low energy requests a warning.
    Start,
    /// Leaving low energy requests silence.
    Stop,
}

/// Spec: `PeriodicLoss`, `LowEnergyWarning`. Complete independently writable owner State.
///
/// Serializing this value preserves both fractions, pending loss and warning state.
/// Reading `energy` never requires a health opportunity or a drawing-history value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// The sole current-energy authority.
    pub energy: EnergyAmount,
    /// Loss supplied explicitly by its external owner.
    pub pending_loss: EnergyLoss,
    /// The last requested warning state.
    pub warning: Warning,
}

/// Spec: `LowEnergyWarning`. Adjustable whole-energy threshold; fractions do not affect it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rules {
    /// Energy at or below this count requests a warning.
    pub warning_threshold: WholeEnergy,
}

impl Rules {
    /// Spec: `LowEnergyWarning`. The reference profile uses a threshold of 30.
    pub const REFERENCE: Self = Self {
        warning_threshold: REFERENCE_WARNING_THRESHOLD,
    };
}

impl State {
    /// Spec: `PeriodicLoss`, `LowEnergyWarning`. Completes an already qualified opportunity.
    ///
    /// Loss is applied and consumed before comparing whole energy to the threshold.
    /// State is mutated immediately, even if the returned sound request is ignored.
    /// `WarningSound::None` leaves any existing warning request unchanged; it is not
    /// a request to stop playback. Restoring `Warning::On` does not automatically
    /// resend Start: a warning still matching low energy returns None. Scheduling,
    /// delivery and playback remain external.
    /// Qualification for running time with neither damage-reducing suit is external;
    /// callers must reject unsupported contexts before calling this operation.
    pub const fn health_opportunity(&mut self, rules: &Rules) -> WarningSound {
        self.energy = self.energy.reduced_by(self.pending_loss);
        self.pending_loss = EnergyLoss::ZERO;
        let next_warning = if self.energy.whole.0 <= rules.warning_threshold.0 {
            Warning::On
        } else {
            Warning::Off
        };
        let sound = match (self.warning, next_warning) {
            (Warning::Off, Warning::On) => WarningSound::Start,
            (Warning::On, Warning::Off) => WarningSound::Stop,
            (Warning::Off, Warning::Off) | (Warning::On, Warning::On) => WarningSound::None,
        };
        self.warning = next_warning;
        sound
    }
}
