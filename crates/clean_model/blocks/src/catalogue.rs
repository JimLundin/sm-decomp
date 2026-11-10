//! The catalogue: one behaviour per spec entry, with its kind, its switch and how it is checked.

use clean_types::{Checked, Kind, Switch};

/// A spec entry's id, by its name in the spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviourId {
    /// §3 `Places`.
    Places,
    /// §3 `TheScriptsRun`.
    TheScriptsRun,
    /// §3 `PatternCells`.
    PatternCells,
    /// §3 `Regrowing`.
    Regrowing,
    /// §3 `RoomStart`.
    RoomStart,
    /// §4 `ScrollCells`.
    ScrollCells,
    /// §5 `CrumbleBlocks`.
    CrumbleBlocks,
    /// §5 `SpeedBlocks`.
    SpeedBlocks,
    /// §5 `BombBlocks`.
    BombBlocks,
    /// §5 `ShotBlocks`.
    ShotBlocks,
    /// §5 `SuperMissileBlocks`.
    SuperMissileBlocks,
    /// §5 `PowerBombBlocks`.
    PowerBombBlocks,
    /// §5 `BombsReveal`.
    BombsReveal,
    /// §5 `AnswersToHerMotion`.
    AnswersToHerMotion,
    /// §5 `BlocksMarkedNothing`.
    BlocksMarkedNothing,
    /// §5 `FullPlacesLetHerThrough`.
    FullPlacesLetHerThrough,
    /// §6 `WhatTheScreenRedraws`.
    WhatTheScreenRedraws,
    /// §6 `Sounds`.
    Sounds,
}

/// A catalogue entry.
pub struct Behaviour {
    /// The spec entry.
    pub id: BehaviourId,
    /// What the player sees: the entry's summary.
    pub summary: &'static str,
    /// What kind of behaviour it is.
    pub kind: Kind,
    /// How it can be adjusted: `Parameter` for an entry whose rule is a row of the `Rules` (§2,
    /// "The rules' rows"), `Fixed` for every other.
    pub switch: Switch,
    /// Whether vectors check it, or the original only.
    pub checked: Checked,
}

/// Every spec entry, in the spec's order.
pub const CATALOGUE: &[Behaviour] = &[
    Behaviour {
        id: BehaviourId::Places,
        summary: "at most 40 reactions run at once; each one started takes the highest free place.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TheScriptsRun,
        summary: "every frame, after her weapons and her moves, each running reaction counts down \
                  its wait; when the wait runs out its next steps run; a paused frame runs none of \
                  it.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PatternCells,
        summary: "a show changes each of its pattern's cells to what the pattern says, keeping its \
                  mark.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Regrowing,
        summary: "a block that grows back takes its kind's whole look, not the look it had, so a \
                  hidden block shows itself once it has grown back.",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RoomStart,
        summary: "when a room starts, its placements take places in the room's order: the scroll \
                  cells keep theirs, the stand-ins leave theirs at once.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ScrollCells,
        summary: "touching a scroll cell changes screens the camera may show, in the same frame.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::CrumbleBlocks,
        summary: "a crumble block falls away a few frames after she lands on it.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SpeedBlocks,
        summary: "a speed block shatters as she dashes or shinesparks into it, and stops her \
                  otherwise.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BombBlocks,
        summary: "a bomb block breaks under a bomb's or a power bomb's blast, or as she dashes, \
                  shinesparks or screw attacks into it.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ShotBlocks,
        summary: "a shot block breaks under any of her weapons and under the grapple.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SuperMissileBlocks,
        summary: "a super missile breaks a super missile block; a bomb shows it for what it is.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PowerBombBlocks,
        summary: "a power bomb's blast breaks a power bomb block; a bomb shows it for what it is.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BombsReveal,
        summary: "a bomb's blast shows a hidden crumble or speed block for what it is.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::AnswersToHerMotion,
        summary: "how each reacting cell meets her motion, touch by touch.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BlocksMarkedNothing,
        summary: "a cell marked `Nothing` reacts to nothing, but what would ask a block of its \
                  family to react still takes a place until the run, and a touch of one passes \
                  through; a cell marked `Inert` takes none; a cell marked `Other` is its part's, \
                  but for the place its mark may hold when she is inside it, and for a blast on \
                  one met as crumble blocks.",
        kind: Kind::Quirk,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::FullPlacesLetHerThrough,
        summary: "with every place taken, a reacting block lets her through when she touches it.",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WhatTheScreenRedraws,
        summary: "the cells a step writes, in order, for the screen.",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Sounds,
        summary: "the sounds a script asks for, in order, each with its queue limit.",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
];
