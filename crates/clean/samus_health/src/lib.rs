//! Samus's current energy, her protection after an eligible hit in Ceres (Ridley's tail, or
//! a steam puff in the escape), and the low-energy warning, played one whole frame at a
//! time by [`State::play_frame`]. Which frames are admitted, each hitter's touch,
//! eligibility and worth, loss production, pickups, drawing history and playback belong to
//! neighbouring owners, not to this model.

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

    /// The whole count.
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

    /// Spec: `TailHit`. Whole energy taken with a floor of zero; the fraction is kept.
    const fn less_whole(self, worth: WholeEnergy) -> Self {
        Self {
            whole: WholeEnergy(self.whole.0.saturating_sub(worth.0)),
            fraction: self.fraction,
        }
    }

    /// Spec: `PeriodicLoss`. Exact fractional borrowing, with an empty-energy clamp: a
    /// difference below zero leaves zero whole and zero fraction.
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
    /// Spec: `PeriodicLoss`. Every frame consumes the entire pending loss.
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

/// Spec: `LowEnergyWarning`. The frame's warning sound: a request to audio for that frame only.
///
/// Scheduling, delivery and audible playback are audio's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningSound {
    /// Leaves any existing warning request unchanged; does not request stopping playback.
    None,
    /// Entering low energy requests a warning.
    Start,
    /// Leaving low energy requests silence.
    Stop,
}

/// Spec: `TailHit`. The frame's impact, as the frame's hitter gives it.
///
/// The hitter decides whether it made an eligible hit and what that hit is worth
/// (Ridley's tail 15, a steam puff in the escape 0). Health never works the worth out
/// for itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Impact {
    /// No eligible hit this frame, including when a hitter skips contact.
    Absent,
    /// An eligible hit, worth this many whole energy.
    EligibleHit {
        /// Whole energy the hit takes if it lands.
        worth: WholeEnergy,
    },
}

/// Spec: `TailHit`, `ProtectionTime`. Remaining protection in active gameplay frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct ProtectionFrames(u8);

impl ProtectionFrames {
    /// No protection remains.
    pub const ZERO: Self = Self(0);
    /// Spec: `TailHit`. A hit that lands grants 96 frames, whatever it is worth.
    pub const HIT_GRANT: Self = Self(96);

    /// Constructs a remaining count in the admitted range 0..=96.
    #[must_use]
    pub const fn new(frames: u8) -> Option<Self> {
        if frames <= Self::HIT_GRANT.0 {
            Some(Self(frames))
        } else {
            None
        }
    }

    /// The remaining count.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Spec: `ProtectionTime`. One frame less, with a floor of zero.
    const fn spent_one(self) -> Self {
        Self(self.0.saturating_sub(1))
    }
}

impl TryFrom<u8> for ProtectionFrames {
    type Error = &'static str;

    fn try_from(frames: u8) -> Result<Self, Self::Error> {
        Self::new(frames).ok_or("protection exceeds 96 frames")
    }
}

impl From<ProtectionFrames> for u8 {
    fn from(frames: ProtectionFrames) -> Self {
        frames.get()
    }
}

impl Warning {
    /// Spec: `LowEnergyWarning`. Judges whole energy against the threshold: entering low
    /// energy turns the warning On and requests Start, leaving it turns it Off and requests
    /// Stop, and a warning already matching requests None. Fractions do not count.
    const fn judged(self, whole: WholeEnergy, rules: Rules) -> (Self, WarningSound) {
        let low = whole.0 <= rules.warning_threshold.0;
        match (self, low) {
            (Self::Off, true) => (Self::On, WarningSound::Start),
            (Self::On, false) => (Self::Off, WarningSound::Stop),
            (Self::Off, false) | (Self::On, true) => (self, WarningSound::None),
        }
    }
}

/// Spec: `PeriodicLoss`, `TailHit`, `ProtectionTime`, `LowEnergyWarning`.
///
/// Health's complete State, which a caller may set to any allowed value, read, save and
/// restore. Reading it changes nothing; only [`State::play_frame`] moves it on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Current energy, the one live energy. The HUD gauge shows its whole count.
    pub energy: EnergyAmount,
    /// Pending periodic loss, supplied by the hazards and actions that cause it.
    pub pending_loss: EnergyLoss,
    /// The warning last requested, whether or not audio played it.
    pub warning: Warning,
    /// Remaining protection, which each hitter's contact (Ridley's tail, the vents' touch)
    /// reads as the frame starts.
    pub protection: ProtectionFrames,
}

/// One played frame: Health's State as the frame leaves it, and the frame's outputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayedFrame {
    /// State as the frame leaves it, which the HUD gauge and drawing read.
    pub state: State,
    /// Spec: `LowEnergyWarning`. The frame's request to audio.
    pub warning_sound: WarningSound,
    /// Spec: `ProtectionTime`. Remaining protection after the frame's hit, before the frame
    /// spends one: what drawing reads for the flash after damage, that frame.
    pub protection_drawn: ProtectionFrames,
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
    /// Plays one admitted frame (time Running, neither suit; the caller refuses any other
    /// frame before calling), from this State as the frame starts and the frame's impact.
    /// Every rule applies every frame, in this order:
    ///
    /// 1. `TailHit`, judged on protection as the frame starts;
    /// 2. `PeriodicLoss`, on the energy the hit left;
    /// 3. `LowEnergyWarning`, on whole energy after both;
    /// 4. `ProtectionTime`: protection is drawn as the hit left it, then spends one frame.
    #[must_use]
    pub const fn play_frame(self, impact: Impact, rules: &Rules) -> PlayedFrame {
        let (energy, protection_drawn) = self.hit(impact);
        let energy = energy.reduced_by(self.pending_loss);
        let (warning, warning_sound) = self.warning.judged(energy.whole, *rules);
        PlayedFrame {
            state: Self {
                energy,
                pending_loss: EnergyLoss::ZERO,
                warning,
                protection: protection_drawn.spent_one(),
            },
            warning_sound,
            protection_drawn,
        }
    }

    /// Spec: `TailHit`. An eligible hit lands only when no protection remains as the frame
    /// starts: it takes its worth in whole energy (floor zero, fraction kept) and grants 96
    /// frames of protection, so a hit worth nothing still protects her. Otherwise energy and
    /// protection stay as they are. Gives the energy and the protection the frame goes on
    /// with.
    const fn hit(self, impact: Impact) -> (EnergyAmount, ProtectionFrames) {
        match impact {
            Impact::EligibleHit { worth } if self.protection.0 == 0 => {
                (self.energy.less_whole(worth), ProtectionFrames::HIT_GRANT)
            }
            Impact::EligibleHit { .. } | Impact::Absent => (self.energy, self.protection),
        }
    }
}
