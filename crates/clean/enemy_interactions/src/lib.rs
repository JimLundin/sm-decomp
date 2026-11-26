//! Ceres-door beam contact and shaft actor and effect pictures.
//!
//! The producer must establish this supported contract: ordinary left/right or
//! shaft Ceres doors, one contact figure, and the five current ordinary-power
//! shot slots after movement. Other actors, multiple figures and other weapons
//! are unsupported capabilities to report at the producer's edge, not misses.
//! Door contact retains no state. The caller applies consumption immediately,
//! retaining the power body for later actors this frame. Scheduling, protection
//! countdown, door responses and shot movement belong to callers.
//!
//! For shaft pictures, advance participating actor clocks and schedule their
//! visits, then accept bursts before advancing effects. Draw using the scrolled
//! view and refresh participating actors after drawing. A turn that only
//! publishes a pending picture performs none of these operations.

use enum_map::{Enum, EnumMap};

pub mod actor;
pub mod effect;
pub mod picture;

/// Spec: `ContactAllowed`. Protection at the start of this opportunity, in frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtectionFrames(pub u16);

/// Spec: `ContactAllowed`. Owned-shot count, over the full unsigned word domain.
/// This is a nonzero gate, not a bound on which stable slots are searched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShotCount(pub u16);

/// Spec: `BoxOverlap`. Whole-pixel centre coordinates, wrapping as unsigned words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Centre {
    /// Horizontal whole-pixel coordinate.
    pub x: u16,
    /// Vertical whole-pixel coordinate.
    pub y: u16,
}

/// Spec: `BoxOverlap`. Unsigned word radii, including zero and large radii.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Radii {
    /// Horizontal radius.
    pub x: u16,
    /// Vertical radius.
    pub y: u16,
}

/// Spec: `BoxOverlap`. A single contact figure, with no fractional coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContactBox {
    /// Whole-pixel centre.
    pub centre: Centre,
    /// Whole-pixel radii.
    pub radii: Radii,
}

/// Spec: `ContactAllowed`. Drawing visibility alone does not determine contact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorContact {
    /// The door is passable.
    Passable,
    /// The door has no contact figure.
    WithoutFigure,
    /// A tangible door, even when hidden from drawing.
    Box(ContactBox),
}

/// Spec: `ContactAllowed`. Independent current facts, not an opaque permission.
#[expect(
    clippy::struct_excessive_bools,
    reason = "The spec declares five independent boolean facts"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContactConditions {
    /// The frame processes actor updates.
    pub actor_updates_enabled: bool,
    /// Contact is not globally suppressed.
    pub interactions_enabled: bool,
    /// Samus is stopped in time.
    pub samus_time_stopped: bool,
    /// Enemies are stopped in time.
    pub enemies_stopped_in_time: bool,
    /// Membership in the actual processing receipt, not solid membership.
    pub processing_membership: bool,
    /// Protection before this opportunity's countdown.
    pub protection_remaining: ProtectionFrames,
}

impl ContactConditions {
    /// Spec: `ContactAllowed`. Return the figure only when all contact gates allow it.
    #[must_use]
    pub const fn allowed_box(self, door: DoorContact, count: ShotCount) -> Option<ContactBox> {
        if !self.actor_updates_enabled
            || !self.interactions_enabled
            || self.samus_time_stopped
            || self.enemies_stopped_in_time
            || !self.processing_membership
            || self.protection_remaining.0 != 0
            || count.0 == 0
        {
            return None;
        }
        match door {
            DoorContact::Box(figure) => Some(figure),
            DoorContact::Passable | DoorContact::WithoutFigure => None,
        }
    }
}

/// Spec: `FirstPowerShot`. Five stable slots in ascending contact priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum ShotSlot {
    /// First slot.
    Slot0,
    /// Second slot.
    Slot1,
    /// Third slot.
    Slot2,
    /// Fourth slot.
    Slot3,
    /// Fifth slot.
    Slot4,
}

/// Spec: `FirstPowerShot`. Stable ascending search order.
pub const SHOT_SLOTS: [ShotSlot; 5] = [
    ShotSlot::Slot0,
    ShotSlot::Slot1,
    ShotSlot::Slot2,
    ShotSlot::Slot3,
    ShotSlot::Slot4,
];

/// Spec: `FirstPowerShot`. Only ordinary uncharged power bodies are supported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotBody {
    /// An unowned slot.
    Free,
    /// An impact appearance, not an eligible power body.
    Impact,
    /// A current ordinary power body, retained even after consumption this frame.
    Power {
        /// Current geometry after shot movement and appearance update.
        figure: ContactBox,
        /// Consumption marker; deliberately does not affect contact eligibility.
        consumed: bool,
    },
}

/// Spec: `FirstPowerShot`. Exactly five stable current shot bodies.
pub type Shots = EnumMap<ShotSlot, ShotBody>;

/// Spec: `BoxOverlap`. The signed-word midpoint must negate with word wrapping too.
const SIGN_BIT: u16 = 0x8000;

/// Spec: `BoxOverlap`. Strict, wrapped radius subtraction, not a radius sum.
const fn axis_overlap(
    shot_centre: u16,
    door_centre: u16,
    shot_radius: u16,
    door_radius: u16,
) -> bool {
    let delta = shot_centre.wrapping_sub(door_centre);
    let distance = if delta & SIGN_BIT != 0 {
        delta.wrapping_neg()
    } else {
        delta
    };
    distance < shot_radius || distance.wrapping_sub(shot_radius) < door_radius
}

impl ContactBox {
    /// Spec: `BoxOverlap`. Both axes must cross the edge using exact word arithmetic.
    /// `self` is the shot figure and `door` is the door figure.
    #[must_use]
    pub const fn overlaps_door(self, door: Self) -> bool {
        axis_overlap(self.centre.x, door.centre.x, self.radii.x, door.radii.x)
            && axis_overlap(self.centre.y, door.centre.y, self.radii.y, door.radii.y)
    }
}

/// Spec: `ContactAllowed`, `FirstPowerShot`. Report only the first consumed slot.
/// Inputs are unchanged; an already-consumed Power remains eligible this frame.
#[must_use]
pub fn consumption(
    conditions: ContactConditions,
    door: DoorContact,
    count: ShotCount,
    shots: &Shots,
) -> Option<ShotSlot> {
    let door_box = conditions.allowed_box(door, count)?;
    SHOT_SLOTS.into_iter().find(|slot| match shots[*slot] {
        ShotBody::Power {
            figure,
            consumed: _,
        } => figure.overlaps_door(door_box),
        ShotBody::Free | ShotBody::Impact => false,
    })
}

/// The specified behaviours, all fixed and without parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// Independent permission gates.
    ContactAllowed,
    /// Wrapped strict overlap arithmetic.
    BoxOverlap,
    /// Stable slot selection and same-frame retention.
    FirstPowerShot,
    /// Schedule a draw visit for a participating shaft actor.
    ShaftActorDraw,
    /// Advance, spend and refresh the actor shake clock.
    ActorShake,
    /// Allocate stationary bursts in the highest available slots.
    EffectSlots,
    /// Advance and cull single-play stationary effects.
    EffectClip,
    /// Project visible figure origins for the exact renderer.
    PictureWindow,
    /// Order shaft figures around the normal player picture.
    ShaftPictureOrder,
}

/// Classification of a specified rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A deliberate rule.
    Mechanic,
    /// Exact nonstandard arithmetic.
    Quirk,
}

/// All rules in this piece are fixed; no alternatives are specified.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Switch {
    /// Not adjustable.
    Fixed,
}

/// One catalogue record, including how the specification checks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Specification entry.
    pub id: BehaviourId,
    /// Player-visible summary.
    pub summary: &'static str,
    /// Rule classification.
    pub kind: Kind,
    /// Supported adjustment.
    pub switch: Switch,
    /// Specification's verification methods.
    pub checked: &'static str,
}

/// Exhaustive catalogue: one record per entry, with no unspecified tuning values.
#[must_use]
pub const fn behaviour(id: BehaviourId) -> Behaviour {
    match id {
        BehaviourId::ContactAllowed => Behaviour {
            id: BehaviourId::ContactAllowed,
            summary: "A passable door or paused interaction cannot spend a beam",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: "source-backed rules, constructed vectors and recorded boundary/harness checks",
        },
        BehaviourId::BoxOverlap => Behaviour {
            id: BehaviourId::BoxOverlap,
            summary: "A beam must cross the box edge, not merely touch it",
            kind: Kind::Quirk,
            switch: Switch::Fixed,
            checked: "source-backed rules, constructed vectors and recorded boundary/harness checks",
        },
        BehaviourId::FirstPowerShot => Behaviour {
            id: BehaviourId::FirstPowerShot,
            summary: "Each door spends the first overlapping power shot without a door response",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: "source-backed rules, constructed vectors and recorded boundary/harness checks",
        },
        BehaviourId::ShaftActorDraw => {
            shaft_behaviour(id, "The overlay can leave view while it keeps rumbling")
        }
        BehaviourId::ActorShake => {
            shaft_behaviour(id, "The overlay and vents move left and right while shaken")
        }
        BehaviourId::EffectSlots => shaft_behaviour(id, "Each burst takes the highest empty slot"),
        BehaviourId::EffectClip => shaft_behaviour(
            id,
            "A stationary burst plays once and disappears when done or offscreen",
        ),
        BehaviourId::PictureWindow => shaft_behaviour(
            id,
            "Puffs and bursts near the edge can still show sprite pieces",
        ),
        BehaviourId::ShaftPictureOrder => shaft_behaviour(
            id,
            "Effects precede the overlay; vents follow Samus in room order",
        ),
    }
}

const fn shaft_behaviour(id: BehaviourId, summary: &'static str) -> Behaviour {
    Behaviour {
        id,
        summary,
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: "oracle and constructed vectors",
    }
}
