//! Her colour sequences during ordinary dry movement: the visor and charge-glow sequences,
//! readied without touching the shared current and target palettes.

use enum_map::Enum;
use serde::{Deserialize, Serialize};
use types::{Action, Body, Item, LandedFrom, Set, Turn};

use crate::inputs::{Figure, Inputs};

/// Eligible visor palette ticks remaining, from 0 to 255.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisorDelay(pub u8);

/// Zero-based entry of the visor sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisorStep(pub usize);

/// Zero-based picture of the charge-glow sequence; this says nothing about glow activity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChargeGlowPosition(pub usize);

/// Spec: `OrdinaryDryColours`. Persistent sequence state, separate from shared colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColourSequences {
    /// Delay before the visor sequence advances.
    pub visor_delay: VisorDelay,
    /// Entry in the visor sequence.
    pub visor_step: VisorStep,
    /// Picture in the charge-glow sequence.
    pub charge_glow_position: ChargeGlowPosition,
}

/// Spec: `OrdinaryDryColours`. The sequences as the ordinary reset leaves them: one eligible
/// palette tick of visor delay, the visor's entry 3 (its fourth), the charge glow reset. Her
/// initialization leaves them so too.
pub const ORDINARY_READY: ColourSequences = ColourSequences {
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

/// What an admitted dry movement asks of the screw attack, among the items switched on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScrewAttackPremise {
    /// The movement is admitted whether the screw attack is on or off.
    Either,
    /// The movement is admitted only while the screw attack is switched off.
    SwitchedOff,
}

impl ScrewAttackPremise {
    /// Whether the premise holds, with the screw attack switched on or not.
    const fn holds(self, screw_attack_on: bool) -> bool {
        match self {
            Self::Either => true,
            Self::SwitchedOff => !screw_attack_on,
        }
    }
}

// Spec: `OrdinaryDryColours`, its Admission paragraph. These two items are the one place that
// decides which figures the ordinary reset admits; nothing else names an action or a body.

/// Only an upright figure is admitted: the spec refuses any action of a crouched or ball figure,
/// and the spring ball is a ball.
const ADMITTED_BODY: Body = Body::Upright;

/// The admitted dry movements of an upright figure, each with its screw-attack premise.
///
/// Every action not listed is refused: falling, crouching, morphing, unmorphing, rolling, taking
/// off, standing up, pressing against a wall, wall jumps, turns on the ground, in a fall or in a
/// moonwalk's jump, and landing from a spin or a wall jump. A spin is admitted only while the
/// screw attack is off. Admitting or refusing one more action is adding or removing its line.
const ADMITTED_DRY_MOVEMENTS: &[(Action, ScrewAttackPremise)] = &[
    // Running [R-ordinary-run-colours].
    (Action::Running, ScrewAttackPremise::Either),
    // Standing still [R-ordinary-still-colours].
    (Action::Still, ScrewAttackPremise::Either),
    // Moonwalking [R-ordinary-moonwalk-colours].
    (Action::Moonwalking, ScrewAttackPremise::Either),
    // A straight jump [R-ordinary-jump-colours].
    (Action::Jumping, ScrewAttackPremise::Either),
    // Turning in a straight jump.
    (Action::Turning(Turn::Jump), ScrewAttackPremise::Either),
    // Landing from a straight jump.
    (
        Action::Landing(LandedFrom::Jump),
        ScrewAttackPremise::Either,
    ),
    // A spin, only with the screw attack off.
    (Action::Spinning, ScrewAttackPremise::SwitchedOff),
];

/// Whether `figure`, the one she ends the frame in, is an admitted dry movement, with the screw
/// attack switched on or not.
fn admits_dry_movement(figure: Figure, screw_attack_on: bool) -> bool {
    figure.pose.body == ADMITTED_BODY
        && ADMITTED_DRY_MOVEMENTS.iter().any(|&(action, premise)| {
            action == figure.pose.action && premise.holds(screw_attack_on)
        })
}

/// Spec: `OrdinaryDryColours`. The palette opportunity's own observations, each its owner's.
///
/// Her figure and her items are the frame's inputs, not observations; where she is doesn't
/// enter. Where an owner cannot supply its fact, the opportunity is `PaletteOpportunity::Unrecognized`
/// instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaletteObservations {
    /// The suit selection made for this frame (equipment).
    pub suit: Suit,
    /// The room's colour treatment (room effects).
    pub room_colours: RoomColours,
    /// The liquid levels present (room effects and the current Environment).
    pub levels: Set<LiquidLevel>,
    /// The grapple beam's activity (Grapple).
    pub grapple: GrappleActivity,
    /// The colour effects active (their owners).
    pub effects: Set<ColourEffect>,
}

impl PaletteObservations {
    /// Whether these observations, with her figure as the frame ends and whether the screw
    /// attack is switched on, are ordinary dry movement.
    fn admit_ordinary_reset(&self, figure: Figure, screw_attack_on: bool) -> bool {
        admits_dry_movement(figure, screw_attack_on)
            && self.suit == Suit::Power
            && self.room_colours == RoomColours::Standard
            && self.levels.values().all(|present| !present)
            && self.grapple == GrappleActivity::Dormant
            && self.effects.values().all(|active| !active)
    }
}

/// The frame's palette opportunity, if it has one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteOpportunity {
    /// The frame has no palette opportunity, or only draws: the reset is not considered.
    Absent,
    /// The opportunity, with every owner's current fact.
    Observed(PaletteObservations),
    /// The opportunity, with a current fact its owner could not supply or that is not one of the
    /// recognized values: the reset is unsupported.
    Unrecognized,
}

/// What the frame's palette opportunity did with the ordinary reset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DryColourReset {
    /// The frame had no palette opportunity.
    NoOpportunity,
    /// The reset applied, even if the sequences were already as it leaves them.
    Applied,
    /// The opportunity is outside the ordinary dry-movement arm: the caller reports it
    /// unsupported, and the sequences are as they were.
    Unsupported,
}

/// Spec: `OrdinaryDryColours`. The frame's palette opportunity, after movement has selected her
/// figure and the suit has been selected, and before health maintenance and the frame's drawing
/// and palette upload. It reads her figure as the frame ends and the screw attack from the
/// frame's inputs.
///
/// An admitted opportunity sets the visor delay to 1, the visor to its entry 3 and the charge
/// glow to its reset position, whatever they were; any other leaves them as they were. Neither
/// way reads or writes a colour of the current or target palette, nor asks for a sound, a
/// graphic or a hardware request.
#[must_use]
pub fn ordinary_dry_colours(
    sequences: ColourSequences,
    opportunity: PaletteOpportunity,
    inputs: &Inputs,
) -> (ColourSequences, DryColourReset) {
    let screw_attack_on = inputs.items[Item::ScrewAttack];
    match opportunity {
        PaletteOpportunity::Absent => (sequences, DryColourReset::NoOpportunity),
        PaletteOpportunity::Observed(observed)
            if observed.admit_ordinary_reset(inputs.figure_after, screw_attack_on) =>
        {
            (ORDINARY_READY, DryColourReset::Applied)
        }
        PaletteOpportunity::Observed(_) | PaletteOpportunity::Unrecognized => {
            (sequences, DryColourReset::Unsupported)
        }
    }
}
