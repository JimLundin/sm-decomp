//! The catalogue: one behaviour for each entry of the file select spec.

use enum_map::{Enum, EnumMap};
use types::{Checked, Kind, Switch};

/// A spec entry, by its id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// Black while file select loads; the fourth finished frame end unblanks at level 0.
    MenuLoads,
    /// The cursor starts on the slot chosen last, or slot A; copy and clear offered with a file.
    CursorStarts,
    /// The menu brightens a level every menu pace of finished frames, up to full.
    MenuBrightens,
    /// A or Start acts on the item under the cursor.
    ItemPressed,
    /// Up and Down move the cursor round the menu; B goes back to the title.
    CursorMoves,
    /// A chosen file's helmet turns; A or Start, or the turn's end, starts the fade-out.
    HelmetTurns,
    /// Leaving, the menu dims a level every menu pace of finished frames; at dark, options.
    MenuDims,
}

/// A catalogue entry: what the player sees, and how the spec classes, adjusts and checks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Its spec entry.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// Its kind.
    pub kind: Kind,
    /// How it may be adjusted.
    pub switch: Switch,
    /// How the spec checks it.
    pub checked: Checked,
}

/// The behaviour of each spec entry, as its entry gives it.
#[must_use]
pub const fn behaviour(id: BehaviourId) -> Behaviour {
    match id {
        BehaviourId::MenuLoads => Behaviour {
            id: BehaviourId::MenuLoads,
            summary: "black while file select loads over five finished frame ends; the fourth \
                      unblanks the screen at level 0",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::CursorStarts => Behaviour {
            id: BehaviourId::CursorStarts,
            summary: "the cursor starts on the slot chosen last, or on slot A; copy and clear \
                      are offered when a slot holds a file",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::MenuBrightens => Behaviour {
            id: BehaviourId::MenuBrightens,
            summary: "the menu brightens one level every menu pace of finished frames, up to \
                      full",
            kind: Kind::Mechanic,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
        BehaviourId::ItemPressed => Behaviour {
            id: BehaviourId::ItemPressed,
            summary: "A or Start acts on the item under the cursor",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::CursorMoves => Behaviour {
            id: BehaviourId::CursorMoves,
            summary: "Up and Down move the cursor round the menu, skipping copy and clear when \
                      they are not offered; B goes back to the title",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::OriginalOnly,
        },
        BehaviourId::HelmetTurns => Behaviour {
            id: BehaviourId::HelmetTurns,
            summary: "once a file is chosen its helmet turns for a fixed number of frames of \
                      work; A or Start, or the turn's end, starts the fade-out",
            kind: Kind::Mechanic,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
        BehaviourId::MenuDims => Behaviour {
            id: BehaviourId::MenuDims,
            summary: "leaving, the menu dims one level every menu pace of finished frames; at \
                      dark the options menu begins",
            kind: Kind::Mechanic,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
    }
}

/// The whole catalogue, an entry for each spec entry.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    EnumMap::from_fn(behaviour)
}
