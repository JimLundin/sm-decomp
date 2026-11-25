//! Her colour sequences during ordinary dry movement: the visor and charge-glow sequences,
//! readied while the shared current and target palettes stay with their owner. And the hurt
//! count, the hit colour pulse the ordinary reset reads.

use enum_map::Enum;
use serde::{Deserialize, Serialize};
use types::{Action, Body, Item, LandedFrom, Set, Turn};

use crate::inputs::{Figure, GrappleActivity, Inputs, Suit};

/// Spec: `HurtColours`. The hurt count: the frames of her hurt pulse so far.
///
/// It is 0 to `HurtCount::MOST`, with 0 no pulse. It is the hit colour pulse
/// `OrdinaryDryColours` reads, active while the count is above 0. It holds one of its values on
/// construction and on restore alike.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct HurtCount(u8);

impl TryFrom<u8> for HurtCount {
    type Error = &'static str;
    fn try_from(frames: u8) -> Result<Self, Self::Error> {
        Self::new(frames).ok_or("the hurt count must be from 0 to 59")
    }
}

impl From<HurtCount> for u8 {
    fn from(count: HurtCount) -> Self {
        count.frames()
    }
}

impl HurtCount {
    /// No pulse: the count 0.
    pub const NONE: Self = Self(0);
    /// The count a fall in her energy or a push's start sets: 1.
    pub const STRUCK: Self = Self(1);
    /// The count is at most 59: reaching 60, it is 0.
    pub const MOST: u8 = 59;

    /// The count of `frames`, if it is 0 to `MOST`.
    #[must_use]
    pub const fn new(frames: u8) -> Option<Self> {
        if frames <= Self::MOST {
            Some(Self(frames))
        } else {
            None
        }
    }

    /// Its frames.
    #[must_use]
    pub const fn frames(self) -> u8 {
        self.0
    }

    /// Whether her colours pulse: the count is above 0 (§11.2's hit colour pulse).
    #[must_use]
    pub const fn pulsing(self) -> bool {
        self.0 > Self::NONE.0
    }

    /// Spec: `HurtColours`. The count gone up by 1; reaching 60, it is 0 and the pulse is over.
    #[must_use]
    pub fn next(self) -> Self {
        Self::new(self.0.saturating_add(1)).unwrap_or(Self::NONE)
    }
}

/// The visor delay: eligible palette ticks before the visor sequence moves on, 0 to 255, each
/// value of its `u8` on construction and on restore.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisorDelay(u8);

impl VisorDelay {
    /// The delay of `ticks` eligible palette ticks.
    #[must_use]
    pub const fn new(ticks: u8) -> Self {
        Self(ticks)
    }

    /// Its eligible palette ticks.
    #[must_use]
    pub const fn ticks(self) -> u8 {
        self.0
    }
}

/// The visor sequence position: an entry of the visor's colour sequence, numbered from 0, held
/// as a `u8` on construction and on restore.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisorStep(u8);

impl VisorStep {
    /// The entry numbered `entry`, from 0.
    #[must_use]
    pub const fn new(entry: u8) -> Self {
        Self(entry)
    }

    /// Its number, from 0.
    #[must_use]
    pub const fn entry(self) -> u8 {
        self.0
    }
}

/// The charge-glow position: a picture of the charge glow's sequence, numbered from 0.
///
/// 0 is the reset position. It is held as a `u8` on construction and on restore. Whether a glow
/// shows is the current beam and effect facts'.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChargeGlowPosition(u8);

impl ChargeGlowPosition {
    /// The reset position, the sequence's first picture.
    pub const RESET: Self = Self(0);

    /// The picture numbered `picture`, from 0.
    #[must_use]
    pub const fn new(picture: u8) -> Self {
        Self(picture)
    }

    /// Its number, from 0.
    #[must_use]
    pub const fn picture(self) -> u8 {
        self.0
    }
}

/// Spec: `OrdinaryDryColours`. Her colour sequences: the visor delay, the visor sequence position
/// and the charge-glow position, kept beside the shared palettes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColourSequences {
    /// Eligible palette ticks before the visor sequence moves on.
    pub visor_delay: VisorDelay,
    /// The entry of the visor sequence.
    pub visor_step: VisorStep,
    /// The picture of the charge-glow sequence.
    pub charge_glow_position: ChargeGlowPosition,
}

/// Spec: `OrdinaryDryColours`. The sequences as the ordinary reset leaves them: one eligible
/// palette tick of visor delay, the visor's entry 3 (its fourth), the charge glow reset. Her
/// initialization leaves them so too.
pub const ORDINARY_READY: ColourSequences = ColourSequences {
    visor_delay: VisorDelay(1),
    visor_step: VisorStep(3),
    charge_glow_position: ChargeGlowPosition::RESET,
};

/// The room's colour treatment, from room effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomColours {
    /// Standard room treatment.
    Standard,
    /// Red background treatment.
    RedBackground,
    /// Orange background treatment.
    OrangeBackground,
}

/// A liquid level of the room, each present or absent on its own, from room effects and the
/// current Environment. Listed in no order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum LiquidLevel {
    /// The water level.
    Water,
    /// The lava level: the surface of lava or acid.
    Lava,
}

/// A current colour effect another owner supplies, each active or inactive on its own. Listed in
/// no order. The hit colour pulse is not one: it is her own hurt count.
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

// Spec: `OrdinaryDryColours`. These two items are the one place that decides which figures the
// ordinary reset admits.

/// The body of every admitted figure: upright.
const ADMITTED_BODY: Body = Body::Upright;

/// The admitted dry movements of an upright figure, each with its screw-attack premise: running,
/// standing still, moonwalking, a straight jump, turning in a take-off or a straight jump,
/// landing from a straight jump or a fall, and a spin with the screw attack switched off. Every
/// other action is refused.
const ADMITTED_DRY_MOVEMENTS: &[(Action, ScrewAttackPremise)] = &[
    (Action::Running, ScrewAttackPremise::Either),
    (Action::Still, ScrewAttackPremise::Either),
    (Action::Moonwalking, ScrewAttackPremise::Either),
    (Action::Jumping, ScrewAttackPremise::Either),
    (Action::Turning(Turn::Jump), ScrewAttackPremise::Either),
    (
        Action::Landing(LandedFrom::Jump),
        ScrewAttackPremise::Either,
    ),
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
/// The rule reads her figure and her items from the frame's inputs beside these.
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
    /// Whether these observations, with her figure as the frame ends, whether the screw attack is
    /// switched on and her hurt count, are ordinary dry movement: among them, no hit colour pulse.
    fn admit_ordinary_reset(&self, figure: Figure, screw_attack_on: bool, hurt: HurtCount) -> bool {
        admits_dry_movement(figure, screw_attack_on)
            && !hurt.pulsing()
            && self.suit == Suit::Power
            && self.room_colours == RoomColours::Standard
            && self.levels.values().all(|present| !present)
            && self.grapple == GrappleActivity::Dormant
            && self.effects.values().all(|active| !active)
    }
}

/// The frame's palette opportunity, with what its owners supply for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteOpportunity {
    /// The frame has no palette opportunity, or only draws.
    Absent,
    /// The opportunity, with every owner's current fact.
    Observed(PaletteObservations),
    /// The opportunity, with a current fact the rule can't read: an observation its owner
    /// couldn't supply or gave as an unknown value, or colours it can't represent.
    Unreadable,
}

/// Spec: `OrdinaryDryColours`. What the ordinary reset did at a palette opportunity whose facts
/// it read: both are ordinary results of the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DryColourReset {
    /// The reset applied: the sequences are as it leaves them, as they may already have been.
    Applied,
    /// A premise of the reset didn't hold: the sequences are as they were.
    Refused,
}

/// Spec: `OrdinaryDryColours`. A palette opportunity with a fact the rule can't read: the frame
/// is outside what the rule decides, the one failure of the rule, and the sequences are as they
/// were.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unrecognized;

/// What a frame's palette opportunity ends in: applied or refused, or unrecognized.
pub type DryColourOutcome = Result<DryColourReset, Unrecognized>;

/// Spec: `OrdinaryDryColours`. The frame's palette opportunity, after movement has selected her
/// figure and the suit has been selected, and before health maintenance and the frame's drawing
/// and palette upload. It reads her figure as the frame ends and the screw attack from the
/// frame's inputs, and `hurt`, her hurt count as the frame starts (her hurt colours come later,
/// after Health), as the hit colour pulse. It gives the sequences and the opportunity's outcome,
/// if the frame has one.
///
/// An admitted opportunity sets the visor delay to 1, the visor to its entry 3 and the charge
/// glow to its reset position, whatever they were; a refused or unrecognized one leaves them as
/// they were. The current and target palettes stay with their owner, as the frame's other
/// producers leave them, and the reset asks for no sound, graphic or hardware request.
#[must_use]
pub fn ordinary_dry_colours(
    sequences: ColourSequences,
    opportunity: PaletteOpportunity,
    inputs: &Inputs,
    hurt: HurtCount,
) -> (ColourSequences, Option<DryColourOutcome>) {
    let screw_attack_on = inputs.items[Item::ScrewAttack];
    match opportunity {
        PaletteOpportunity::Absent => (sequences, None),
        PaletteOpportunity::Observed(observed)
            if observed.admit_ordinary_reset(inputs.figure_after, screw_attack_on, hurt) =>
        {
            (ORDINARY_READY, Some(Ok(DryColourReset::Applied)))
        }
        PaletteOpportunity::Observed(_) => (sequences, Some(Ok(DryColourReset::Refused))),
        PaletteOpportunity::Unreadable => (sequences, Some(Err(Unrecognized))),
    }
}
