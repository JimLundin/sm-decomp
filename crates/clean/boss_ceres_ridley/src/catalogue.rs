//! The catalogue: one behaviour for each entry of the spec, with its kind, its switch and how it
//! is checked.

use enum_map::{Enum, EnumMap};
use types::{Checked, Kind, Switch};

/// A behaviour of the spec, by its entry id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// A door counts its wait off by one each run, and steps when it runs out.
    Waiting,
    /// A door sees her when she was within the reach on both axes.
    SheIsNear,
    /// A placed door shows itself, then hides and chooses to stand open or closed.
    RoomStart,
    /// A closed or open door looks for her every few frames.
    Polling,
    /// A closed door that sees her hisses and slides open.
    Opening,
    /// An open door that no longer sees her slides shut.
    Closing,
    /// Once Ridley has fled, the shaft's door is an unseen wall.
    ShaftWall,
    /// Which phases block her.
    Solid,
    /// Which phases are drawn.
    Shown,
    /// During the escape, the doors keep the station shaking.
    EscapeShakes,
    /// The doors' palette takes the calm or the escape colours.
    TheDoorsColours,
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

impl BehaviourId {
    /// This entry's behaviour, as the spec gives it.
    #[must_use]
    pub const fn behaviour(self) -> Behaviour {
        match self {
            Self::Waiting => Behaviour {
                id: Self::Waiting,
                summary: "a door counts its wait off by one each run, and steps when it runs out",
                kind: Kind::Mechanic,
                switch: Switch::Fixed,
                checked: Checked::ByVectors,
            },
            Self::SheIsNear => Behaviour {
                id: Self::SheIsNear,
                summary: "a door sees her when she was within a set distance of its position on \
                          both axes at the end of the last frame",
                kind: Kind::Mechanic,
                switch: Switch::Parameter,
                checked: Checked::ByVectors,
            },
            Self::RoomStart => Behaviour {
                id: Self::RoomStart,
                summary: "a door placed by a room's start is shown and solid with no figure for \
                          its first frame, then hidden and passable until it chooses, two frames \
                          later, to stand open or closed",
                kind: Kind::Mechanic,
                switch: Switch::Fixed,
                checked: Checked::ByVectors,
            },
            Self::Polling => Behaviour {
                id: Self::Polling,
                summary: "a door that is closed or open looks for her every few frames",
                kind: Kind::Mechanic,
                switch: Switch::Parameter,
                checked: Checked::ByVectors,
            },
            Self::Opening => Behaviour {
                id: Self::Opening,
                summary: "a closed door that sees her hisses and slides open over four figures, \
                          staying solid until it is open; then it is hidden and she can pass",
                kind: Kind::Mechanic,
                switch: Switch::Parameter,
                checked: Checked::ByVectors,
            },
            Self::Closing => Behaviour {
                id: Self::Closing,
                summary: "an open door that no longer sees her shows itself again and slides shut \
                          over four figures, without a sound and without looking for her until \
                          it is closed",
                kind: Kind::Mechanic,
                switch: Switch::Parameter,
                checked: Checked::ByVectors,
            },
            Self::ShaftWall => Behaviour {
                id: Self::ShaftWall,
                summary: "once Ridley has fled, the shaft's door is an unseen wall",
                kind: Kind::Mechanic,
                switch: Switch::Fixed,
                checked: Checked::ByVectors,
            },
            Self::Solid => Behaviour {
                id: Self::Solid,
                summary: "a door blocks in the phases that show it, and as the wall; hidden doors \
                          that are open or starting let her through",
                kind: Kind::Mechanic,
                switch: Switch::Fixed,
                checked: Checked::ByVectors,
            },
            Self::Shown => Behaviour {
                id: Self::Shown,
                summary: "a door is shown in the phases it is solid in, except the wall; it is \
                          drawn with its phase's figure",
                kind: Kind::Mechanic,
                switch: Switch::Fixed,
                checked: Checked::ByVectors,
            },
            Self::EscapeShakes => Behaviour {
                id: Self::EscapeShakes,
                summary: "during the escape, each door keeps the station shaking: whenever the \
                          screen is still, a door's run starts a small shake, and now and then a \
                          strong one",
                kind: Kind::Mechanic,
                switch: Switch::Parameter,
                checked: Checked::ByVectors,
            },
            Self::TheDoorsColours => Behaviour {
                id: Self::TheDoorsColours,
                summary: "when a room with doors starts, the doors' palette takes the doors' \
                          colours, another set during the escape",
                kind: Kind::Mechanic,
                switch: Switch::Fixed,
                checked: Checked::ByVectors,
            },
        }
    }
}

/// The catalogue: every entry's behaviour.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    EnumMap::from_fn(BehaviourId::behaviour)
}
