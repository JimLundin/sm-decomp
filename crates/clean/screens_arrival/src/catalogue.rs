//! The catalogue: one behaviour for each entry of the spec.

use enum_map::{Enum, EnumMap};
use types::{Checked, Kind, Switch};

/// A behaviour of the arrival, one for each spec entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// While the start is made ready, the screen stays as dark as it was.
    StartsInTheDark,
    /// She appears at the top of the shaft, facing the screen.
    ArrivesAtTheTop,
    /// The picture comes up one level every `rhythm + 1` frames.
    PictureBrightens,
    /// Once the picture is fully up, she plays.
    PlayBegins,
}

/// A catalogue entry: what the player sees, what kind of rule it is, how it can be adjusted and
/// how the spec checks it.
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

/// The catalogue entry of a behaviour.
#[must_use]
pub const fn behaviour(id: BehaviourId) -> Behaviour {
    match id {
        BehaviourId::StartsInTheDark => Behaviour {
            id: BehaviourId::StartsInTheDark,
            summary: "while the game makes the start ready, nothing of the arrival moves and \
                      the screen stays as dark as it was",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::ArrivesAtTheTop => Behaviour {
            id: BehaviourId::ArrivesAtTheTop,
            summary: "on the frame the preparation is done she appears at the top of the \
                      shaft, facing the screen, the room's colours go up and the mini-map \
                      shows; the fade in starts on the next frame",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::PictureBrightens => Behaviour {
            id: BehaviourId::PictureBrightens,
            summary: "each frame of the fade in counts down; when the count runs out the \
                      picture comes up one level and the count starts again",
            kind: Kind::Mechanic,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
        BehaviourId::PlayBegins => Behaviour {
            id: BehaviourId::PlayBegins,
            summary: "once the picture is fully up the fade in ends and she plays",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
    }
}

/// Every behaviour of the arrival, by its id.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    EnumMap::from_fn(behaviour)
}
