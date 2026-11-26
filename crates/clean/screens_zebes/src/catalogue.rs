//! The catalogue: one behaviour per spec entry, with its kind, its switch and how it is
//! checked.

use types::{Checked, Kind, Switch};

/// A spec entry of the flight to Zebes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviourId {
    /// The mosaic reveal brightens.
    GunshipAppears,
    /// The planet and stars appear with the ship plane.
    PlanetAppears,
    /// The planet's caption runs.
    PlanetIsNamed,
    /// The ship drifts right, banks and recedes.
    ShipApproachesPlanet,
    /// The planet and stars hold still.
    PlanetHolds,
    /// The planet and stars slide away.
    PlanetAndStarsSlide,
    /// The scene blacks out and hands off.
    LandingSiteLoadBegins,
}

impl BehaviourId {
    /// Every entry, in the spec's order.
    pub const ALL: [Self; 7] = [
        Self::GunshipAppears,
        Self::PlanetAppears,
        Self::PlanetIsNamed,
        Self::ShipApproachesPlanet,
        Self::PlanetHolds,
        Self::PlanetAndStarsSlide,
        Self::LandingSiteLoadBegins,
    ];
}

/// A catalogue entry.
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

/// The catalogue entry of a spec entry.
#[must_use]
pub const fn behaviour(id: BehaviourId) -> Behaviour {
    match id {
        BehaviourId::GunshipAppears => Behaviour {
            id: BehaviourId::GunshipAppears,
            summary: "the scene begins dark with a coarse mosaic, then brightens while its blocks shrink",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::PlanetAppears => Behaviour {
            id: BehaviourId::PlanetAppears,
            summary: "when the mosaic reaches single pixels, the planet and stars appear with the ship plane",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::PlanetIsNamed => Behaviour {
            id: BehaviourId::PlanetIsNamed,
            summary: "the label waits in the dark, brightens, holds and dims before the flight resumes",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::ShipApproachesPlanet => Behaviour {
            id: BehaviourId::ShipApproachesPlanet,
            summary: "the ship drifts right, banks left and then recedes as it drifts left",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::PlanetHolds => Behaviour {
            id: BehaviourId::PlanetHolds,
            summary: "after the ship disappears, the planet and stars stay still before sliding away",
            kind: Kind::Mechanic,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
        BehaviourId::PlanetAndStarsSlide => Behaviour {
            id: BehaviourId::PlanetAndStarsSlide,
            summary: "the planet and stars accelerate upward until they disappear",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::LandingSiteLoadBegins => Behaviour {
            id: BehaviourId::LandingSiteLoadBegins,
            summary: "after the marked star field leaves, the scene blacks out and hands control to the Landing Site load; Samus's energy returns to its maximum",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
    }
}
