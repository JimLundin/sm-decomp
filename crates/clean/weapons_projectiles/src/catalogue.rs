//! One entry for each named behaviour in the supplied specification.
use enum_map::{Enum, EnumMap, enum_map};
use types::{Checked, Kind, Switch};
/// Spec: ShotSpacing through BeamVisibility. Exhaustive catalogue identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// Waiting between shots.
    ShotSpacing,
    /// Held shoot repeats.
    HoldingShoot,
    /// Pose-supplied muzzle origin.
    CannonOrigin,
    /// Asymmetric inherited motion.
    InheritedMotion,
    /// Initial muzzle probe.
    MuzzleContact,
    /// Accelerating flight.
    FlyingBeam,
    /// Ordered whole-edge block contact.
    ShotContact,
    /// Pre-motion periodic trail.
    BeamTrail,
    /// Removal outside view distance.
    LeavingView,
    /// Actor consumption and removal.
    EndingShot,
    /// Clip hold and frame changes.
    BeamPictures,
    /// Phased, separately ordered picture producers.
    BeamVisibility,
}
/// Spec: ShotSpacing through BeamVisibility. Catalogue metadata and vector evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Matching spec entry identifier.
    pub id: BehaviourId,
    /// What the player observes.
    pub summary: &'static str,
    /// Rule classification.
    pub kind: Kind,
    /// Matching spec adjustability.
    pub switch: Switch,
    /// All entries are checked by supplied vectors.
    pub checked: Checked,
}
impl Behaviour {
    const fn new(id: BehaviourId, summary: &'static str, kind: Kind, switch: Switch) -> Self {
        Self {
            id,
            summary,
            kind,
            switch,
            checked: Checked::ByVectors,
        }
    }
}
/// Spec: ShotSpacing through BeamVisibility. One compiler-exhaustive row per entry.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    enum_map! {
        BehaviourId::ShotSpacing => Behaviour::new(BehaviourId::ShotSpacing, "Waiting between power-beam shots", Kind::Timing, Switch::Fixed),
        BehaviourId::HoldingShoot => Behaviour::new(BehaviourId::HoldingShoot, "Held shoot repeats when the cannon is available", Kind::Mechanic, Switch::Fixed),
        BehaviourId::CannonOrigin => Behaviour::new(BehaviourId::CannonOrigin, "The shot starts at the displayed pose's cannon", Kind::Mechanic, Switch::Fixed),
        BehaviourId::InheritedMotion => Behaviour::new(BehaviourId::InheritedMotion, "The shot inherits a peculiar part of preceding motion", Kind::Quirk, Switch::Fixed),
        BehaviourId::MuzzleContact => Behaviour::new(BehaviourId::MuzzleContact, "The muzzle can touch a block before flight", Kind::Quirk, Switch::Fixed),
        BehaviourId::FlyingBeam => Behaviour::new(BehaviourId::FlyingBeam, "The beam gets faster as it flies", Kind::Mechanic, Switch::Parameter),
        BehaviourId::ShotContact => Behaviour::new(BehaviourId::ShotContact, "A whole contacted edge must stop the shot to burst", Kind::Mechanic, Switch::Fixed),
        BehaviourId::BeamTrail => Behaviour::new(BehaviourId::BeamTrail, "A moving beam leaves a trail every fourth update", Kind::Timing, Switch::Parameter),
        BehaviourId::LeavingView => Behaviour::new(BehaviourId::LeavingView, "A flying shot or fresh impact disappears far from view", Kind::Mechanic, Switch::Fixed),
        BehaviourId::EndingShot => Behaviour::new(BehaviourId::EndingShot, "Actor contact spends a shot and completed clips disappear", Kind::Mechanic, Switch::Fixed),
        BehaviourId::BeamPictures => Behaviour::new(BehaviourId::BeamPictures, "Flight repeats a clip and a burst plays to its end", Kind::Timing, Switch::Fixed),
        BehaviourId::BeamVisibility => Behaviour::new(BehaviourId::BeamVisibility, "Beam slots alternate visibility while impacts stay visible", Kind::Mechanic, Switch::Fixed),
    }
}
