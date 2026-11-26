//! The catalogue: one behaviour for each entry of the spec, with its kind, its switch and how
//! the spec checks it.

use types::{Checked, Kind, Switch};

/// A spec entry, by its id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviourId {
    /// Samus stops on the elevator and the escape clock holds.
    ElevatorStopsSamus,
    /// The shaft keeps running through the wait and the fade.
    PlatformWait,
    /// The blackout finishes the save and hands the scene to the explosion.
    DepartureHandoff,
}

impl BehaviourId {
    /// Every entry, in the spec's order.
    pub const ALL: &[Self] = &[
        Self::ElevatorStopsSamus,
        Self::PlatformWait,
        Self::DepartureHandoff,
    ];
}

/// A behaviour of the catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Its spec entry.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// What kind of behaviour it is.
    pub kind: Kind,
    /// How it can be adjusted.
    pub switch: Switch,
    /// How the spec checks it.
    pub checked: Checked,
}

/// The catalogue's behaviour for a spec entry.
#[must_use]
pub const fn behaviour(id: BehaviourId) -> Behaviour {
    match id {
        BehaviourId::ElevatorStopsSamus => Behaviour {
            id: BehaviourId::ElevatorStopsSamus,
            summary: "Samus stops on the elevator and the escape clock holds.",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::PlatformWait => Behaviour {
            id: BehaviourId::PlatformWait,
            summary: "The shaft keeps running through the wait and the fade.",
            kind: Kind::Timing,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::DepartureHandoff => Behaviour {
            id: BehaviourId::DepartureHandoff,
            summary: "The blackout finishes the save and hands the scene to the explosion.",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
    }
}
