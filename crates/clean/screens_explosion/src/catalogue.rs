//! The catalogue: one behaviour for each entry of the spec.

use enum_map::Enum;
use types::{Checked, Kind, Switch};

/// A spec entry, by its id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// The black screen changes to a prepared picture of the station in space.
    StationPictureArrives,
    /// The station stays dark until the audio service is ready.
    StationAwaitsMusic,
    /// The station drifts into view and brightens.
    StationBrightens,
    /// Asteroid fields drift in front of the purple background.
    SpaceDrifts,
    /// Three bands of bursts cross the station before the final explosion.
    StationBursts,
    /// After the station has receded, a flash covers it and the ship appears.
    StationBreaks,
    /// The gunship turns and recedes until it disappears.
    ShipDeparts,
    /// After the ship disappears, space holds before the final fade.
    SpaceHolds,
    /// Space dims to black and gives way to the flight to Zebes.
    SceneBlacksOut,
}

/// A catalogue entry: what the player sees, what kind of behaviour it is, how it may be
/// adjusted and how the spec checks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Its spec entry.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// What kind of behaviour it is.
    pub kind: Kind,
    /// How it may be adjusted.
    pub switch: Switch,
    /// How the spec checks it.
    pub checked: Checked,
}

/// The catalogue entry of a spec entry.
#[must_use]
pub const fn behaviour(id: BehaviourId) -> Behaviour {
    match id {
        BehaviourId::StationPictureArrives => Behaviour {
            id: BehaviourId::StationPictureArrives,
            summary: "the black screen changes to a prepared picture of the station in space",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::StationAwaitsMusic => Behaviour {
            id: BehaviourId::StationAwaitsMusic,
            summary: "the station stays dark until the audio service is ready",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::StationBrightens => Behaviour {
            id: BehaviourId::StationBrightens,
            summary: "the station drifts into view and brightens",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::SpaceDrifts => Behaviour {
            id: BehaviourId::SpaceDrifts,
            summary: "asteroid fields drift in front of the purple background",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::StationBursts => Behaviour {
            id: BehaviourId::StationBursts,
            summary: "three bands of bursts cross the station before the final explosion",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::StationBreaks => Behaviour {
            id: BehaviourId::StationBreaks,
            summary: "after the station has receded, a flash covers it and the ship appears",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::ShipDeparts => Behaviour {
            id: BehaviourId::ShipDeparts,
            summary: "the gunship turns and recedes until it disappears",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::SpaceHolds => Behaviour {
            id: BehaviourId::SpaceHolds,
            summary: "after the ship disappears, space holds before the final fade",
            kind: Kind::Mechanic,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
        BehaviourId::SceneBlacksOut => Behaviour {
            id: BehaviourId::SceneBlacksOut,
            summary: "space dims to black and gives way to the flight to Zebes",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
    }
}
