//! The catalogue: one behaviour per spec entry, with its kind, its switch and how it is checked.

use enum_map::{Enum, EnumMap};
use types::{Checked, Kind, Switch};

/// A spec entry's id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// The encounter advances once per scene work frame.
    EncounterBeats,
    /// Mother Brain and the ring schedule start as page 1 gives way to the room.
    EncounterStarts,
    /// Mother Brain cycles her breathing pictures.
    BrainBreathes,
    /// The first missile in precedence hits when its centre crosses the contact line.
    BrainEndures,
    /// The boss and the rings share the palette that flashes after a hit.
    BrainFlashes,
    /// Two pairs of rings form at set places, then drift.
    RingsAppear,
    /// The moving rings slide diagonally; one can hit Samus.
    RingsSweep,
    /// Bursts of two sizes start at their own times around the boss.
    BurstsPulse,
    /// The room background bobs while the boss explodes and the picture dissolves back.
    BackgroundBobs,
    /// The bursts run for a hold, then cue page 2; the boss leaves as the return ends.
    ReturnToPage,
    /// Ring and burst pictures overlap, with Mother Brain behind them.
    FightPictures,
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
        BehaviourId::EncounterBeats => Behaviour {
            id: BehaviourId::EncounterBeats,
            summary: "the encounter advances once per scene work frame, including a frame that spans multiple video cuts",
            kind: Kind::Timing,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::EncounterStarts => Behaviour {
            id: BehaviourId::EncounterStarts,
            summary: "Mother Brain and the ring schedule start as page 1 gives way to the room",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::BrainBreathes => Behaviour {
            id: BehaviourId::BrainBreathes,
            summary: "Mother Brain cycles three pictures while she is present",
            kind: Kind::Output,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::BrainEndures => Behaviour {
            id: BehaviourId::BrainEndures,
            summary: "the first missile in weapon precedence hits when its centre crosses the contact line",
            kind: Kind::Mechanic,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
        BehaviourId::BrainFlashes => Behaviour {
            id: BehaviourId::BrainFlashes,
            summary: "the boss and the rings share the palette that flashes after a missile hit",
            kind: Kind::Output,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::RingsAppear => Behaviour {
            id: BehaviourId::RingsAppear,
            summary: "two pairs of rings form at set places, then drift across the room",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::RingsSweep => Behaviour {
            id: BehaviourId::RingsSweep,
            summary: "the moving rings slide diagonally, with one ring able to hit Samus",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::BurstsPulse => Behaviour {
            id: BehaviourId::BurstsPulse,
            summary: "flashes of different sizes start at different times around the boss",
            kind: Kind::Output,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::BackgroundBobs => Behaviour {
            id: BehaviourId::BackgroundBobs,
            summary: "the room background moves up or down on alternate beats while the boss explodes and while the picture dissolves back to text",
            kind: Kind::Output,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
        BehaviourId::ReturnToPage => Behaviour {
            id: BehaviourId::ReturnToPage,
            summary: "the bursts run for a hold before they cue page 2; the boss stays visible during most of the return dissolve",
            kind: Kind::Mechanic,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
        BehaviourId::FightPictures => Behaviour {
            id: BehaviourId::FightPictures,
            summary: "ring and burst pictures can overlap, with Mother Brain behind them",
            kind: Kind::Output,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
    }
}

/// The whole catalogue, one entry per spec entry, in the spec's order.
#[must_use]
pub fn catalogue() -> Vec<Behaviour> {
    EnumMap::from_fn(behaviour).into_values().collect()
}
