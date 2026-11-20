//! Ordinary dry-run sequence preparation, independent of the shared palette banks.

use enum_map::Enum;
use serde::{Deserialize, Serialize};
use types::{Action, Position, Set};

use crate::inputs::Figure;

/// Eligible visor palette ticks remaining, from 0 to 255.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisorDelay(pub u8);

/// Zero-based entry of the visor sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisorStep(pub usize);

/// Zero-based picture of the charge-glow sequence; this says nothing about glow activity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChargeGlowPosition(pub usize);

/// Spec: `OrdinaryRunColours`. Persistent sequence state, separate from shared colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColourSequences {
    /// Delay before the visor sequence advances.
    pub visor_delay: VisorDelay,
    /// Entry in the visor sequence.
    pub visor_step: VisorStep,
    /// Picture in the charge-glow sequence.
    pub charge_glow_position: ChargeGlowPosition,
}

const ORDINARY_READY: ColourSequences = ColourSequences {
    visor_delay: VisorDelay(1),
    visor_step: VisorStep(3),
    charge_glow_position: ChargeGlowPosition(0),
};

/// Actual current equipment-dependent suit selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Suit {
    /// Neither Gravity nor Varia is enabled.
    Power,
    /// Varia is enabled without Gravity.
    Varia,
    /// Gravity is enabled, regardless of Varia.
    Gravity,
}

/// Room colour treatment supplied by room effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomColours {
    /// Standard room treatment.
    Standard,
    /// Red background treatment.
    RedBackground,
    /// Orange background treatment.
    OrangeBackground,
}

/// Current activity supplied by Grapple, independent of camera scrolling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrappleActivity {
    /// Grapple is dormant.
    Dormant,
    /// Grapple is busy.
    Busy,
}

/// Independently present liquid levels; presence does not mean Samus is submerged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum LiquidLevel {
    /// A water level.
    Water,
    /// A magma level.
    Magma,
}

/// Current colour effects, independently supplied by their owners.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum ColourEffect {
    /// Lingering glow after firing.
    StoredGlow,
    /// Weapon charging.
    WeaponCharging,
    /// Appearance override.
    OverrideColours,
    /// Special colouring.
    SpecialColours,
    /// Speed glow.
    SpeedGlow,
    /// Hit colour pulse.
    HitColourPulse,
}

/// Spec: `OrdinaryRunColours`. Complete known facts from the actual palette opportunity.
///
/// Movement supplies a valid selected figure and its position; equipment supplies the current
/// suit answer, not a prediction. Do not construct these facts when any owner answer is unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunColourFacts {
    /// Current figure after selection.
    pub figure: Figure,
    /// Current position after movement and selection.
    pub position: Position,
    /// Current suit selection.
    pub suit: Suit,
    /// Actual room colour treatment.
    pub room_colours: RoomColours,
    /// The levels currently present.
    pub levels: Set<LiquidLevel>,
    /// Actual activity of the grapple beam.
    pub grapple: GrappleActivity,
    /// The effects currently active.
    pub effects: Set<ColourEffect>,
}

impl RunColourFacts {
    fn admits_ordinary_reset(&self) -> bool {
        self.figure.pose.action == Action::Running
            && self.suit == Suit::Power
            && self.room_colours == RoomColours::Standard
            && self.levels.values().all(|present| !present)
            && self.grapple == GrappleActivity::Dormant
            && self.effects.values().all(|active| !active)
    }
}

/// The actual palette opportunity is outside the supported ordinary dry-run arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnsupportedRunColours;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum RunColourProgress {
    Unobserved,
    Pending(RunColourFacts),
    Completed,
}

/// Spec: `RetainedRunColours`. Work for one actual palette opportunity.
///
/// Save it alongside the sequence state, the authoritative palettes and the rest of the caller's
/// look and input/delivery progress. It never takes ownership of the movement or palette inputs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunColourWork(RunColourProgress);

impl RunColourWork {
    /// Start this only for a new actual palette opportunity, never a redraw or a retry.
    pub const UNOBSERVED: Self = Self(RunColourProgress::Unobserved);

    /// Spec: `OrdinaryRunColours`. After selection and before health maintenance, observe the
    /// current facts. Pass `None` when a required fact or valid colour bank could not be supplied.
    /// A refusal leaves progress unchanged. `Ok(false)` means this work was already observed.
    /// Neither sequence state nor colours are changed here.
    ///
    /// # Errors
    /// Returns `UnsupportedRunColours` for unobserved work outside the ordinary arm.
    pub fn observe(
        &mut self,
        facts: Option<RunColourFacts>,
    ) -> Result<bool, UnsupportedRunColours> {
        if *self != Self::UNOBSERVED {
            return Ok(false);
        }
        let facts = facts
            .filter(RunColourFacts::admits_ordinary_reset)
            .ok_or(UnsupportedRunColours)?;
        self.0 = RunColourProgress::Pending(facts);
        Ok(true)
    }

    /// Inspect the retained actual facts while the reset is pending, without advancing it.
    #[must_use]
    pub const fn pending(&self) -> Option<&RunColourFacts> {
        match &self.0 {
            RunColourProgress::Pending(facts) => Some(facts),
            RunColourProgress::Unobserved | RunColourProgress::Completed => None,
        }
    }

    /// Spec: `RetainedRunColours`. Apply an admitted reset once, without consulting later
    /// facts. Returns whether a completion took place, even if the sequences were already ready.
    /// This changes only sequence state: no palette access, sound, graphic or hardware request.
    pub const fn complete(&mut self, sequences: &mut ColourSequences) -> bool {
        if !matches!(self.0, RunColourProgress::Pending(_)) {
            return false;
        }
        *sequences = ORDINARY_READY;
        self.0 = RunColourProgress::Completed;
        true
    }
}
