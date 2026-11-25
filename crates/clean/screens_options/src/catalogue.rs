//! The catalogue: one behaviour for each entry of the options menu's spec.

use enum_map::Enum;
use types::{Checked, Kind, Switch};

/// A spec entry, by its id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// Black while the menu loads.
    OptionsLoad,
    /// The menu fading in.
    OptionsBrighten,
    /// Up and Down round the options.
    OptionsCursor,
    /// B, or A and Start, on the options.
    OptionsChoice,
    /// The special settings.
    SpecialSettings,
    /// The screen dissolving out and in.
    Dissolve,
    /// The menu fading out as it hands on.
    OptionsLeave,
}

/// A behaviour of the catalogue: what the player sees, its kind, its switch and how it is checked.
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
        BehaviourId::OptionsLoad => Behaviour {
            id: BehaviourId::OptionsLoad,
            summary: "black while the menu loads over two finished frame ends; the cursor starts \
                      on start game",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::OptionsBrighten => Behaviour {
            id: BehaviourId::OptionsBrighten,
            summary: "the menu brightens one level per finished frame, up to full",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::OptionsCursor => Behaviour {
            id: BehaviourId::OptionsCursor,
            summary: "Up and Down move the cursor round the five items",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::OptionsChoice => Behaviour {
            id: BehaviourId::OptionsChoice,
            summary: "after the cursor's move, B goes back to file select, or A or Start acts on \
                      the item under the cursor",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::SpecialSettings => Behaviour {
            id: BehaviourId::SpecialSettings,
            summary: "Up and Down move round the three items; B or end goes back; A, Left, Right \
                      or Start switches the setting under the cursor",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::Dissolve => Behaviour {
            id: BehaviourId::Dissolve,
            summary: "the screen darkens one level per finished frame as its blocks grow, then \
                      brightens one level per finished frame as they shrink, into the cursor's \
                      screen",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::OptionsLeave => Behaviour {
            id: BehaviourId::OptionsLeave,
            summary: "leaving for the game or for file select, the menu darkens one level per \
                      finished frame",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
    }
}
