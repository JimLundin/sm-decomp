//! The catalogue: one behaviour for each entry of the spec, with its kind, its switch and how it
//! is checked.

use enum_map::{Enum, EnumMap};
use types::{Checked, Kind, Switch, by_member};

/// A spec entry, by its id. Listed in the spec's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// Touching a door to a room starts the passage on the next frame.
    TheTouch,
    /// The counted frames of work of each phase, and the slide's frame every frame.
    PassageTime,
    /// Which frames are finished frames, and the HUD's frame count restarting.
    FinishedFrames,
    /// The room falls silent before the screen darkens.
    Quieting,
    /// The colours go dark but for the HUD's, and the new room's come up.
    TheFades,
    /// A fade's even steps, each channel on its own.
    Fading,
    /// The arrival taken, and the view crept onto a screen's edge.
    Squaring,
    /// The frames of work before the view sets off, the next room built on the second.
    Building,
    /// The view put one screen back from its arrival.
    SettingOff,
    /// The view's slide into the next room.
    Sliding,
    /// She is carried along with the slide.
    Carrying,
    /// She is placed, the room fetched, and she lands as the slide ends.
    Arriving,
    /// Where she is placed, settles and steps clear.
    WhereSheLands,
    /// The frames of work after the slide, and the next room's track.
    Landing,
    /// Nothing of her but her position changes.
    SheKeepsHerMotion,
    /// What runs on each frame of work.
    WhatRuns,
}

/// A catalogue entry: a behaviour of the spec.
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

/// The catalogue's rows, one for each spec entry, in the spec's order.
pub const CATALOGUE: <BehaviourId as Enum>::Array<Behaviour> = [
    Behaviour {
        id: BehaviourId::TheTouch,
        summary: "touching a door to a room starts the passage on the next frame",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PassageTime,
        summary: "the passage takes a counted number of frames of work in each phase, a frame of \
                  work ends when the platform says its cost is spent, and the slide runs one step \
                  every frame whatever the work",
        kind: Kind::Timing,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::FinishedFrames,
        summary: "from the slide's own first step until she steps clear, every frame is a \
                  finished frame, whatever the work, and the HUD's frame count starts again at \
                  each",
        kind: Kind::Timing,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Quieting,
        summary: "the room falls silent before the screen darkens",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TheFades,
        summary: "the room's colours go to black but for the HUD's, and the new room's come up \
                  the same way; play goes on when they are up",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Fading,
        summary: "each colour of a fade moves to its goal in even steps, each channel on its own",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Squaring,
        summary: "in the dark, the doorway's arrival is taken, then the view creeps to the \
                  nearest edge of a screen across the way she leaves",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Building,
        summary: "the view sets off on the third frame of work after the squaring, and the next \
                  room is built on the second",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SettingOff,
        summary: "the view is put one screen back from its arrival, along the way she leaves, \
                  and makes the slide's first step",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Sliding,
        summary: "the view slides across into the next room by a fixed step every frame, \
                  whatever the game's work, and stops on its arrival",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Carrying,
        summary: "she is carried along with the slide at the doorway's speed, or the passage's \
                  own",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Arriving,
        summary: "she is placed in the next room as the slide begins; the room's tiles and then \
                  its contents are fetched; she lands on the frame the slide ends",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WhereSheLands,
        summary: "she is moved into the next room's place, keeping where she was on the screen; \
                  after the slide her x settles on the far edge of its 8 px column, the way she \
                  went; then she is stepped away from a screen's second column or second-last \
                  column, and from a screen's second row, into the middle of the next",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Landing,
        summary: "she settles across on the first frame of work after the slide and the animated \
                  tiles run once on the second; the next room's track starts on the first frame \
                  of work after those two that finds the music ready; she steps clear on the \
                  frame of work after it starts, and the fade in begins on the next",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SheKeepsHerMotion,
        summary: "nothing of her but her position changes across the passage",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::OriginalOnly,
    },
    Behaviour {
        id: BehaviourId::WhatRuns,
        summary: "the enemies go on while the room is lit; nothing else of play runs",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
];

/// The catalogue keyed by each entry's id.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    by_member(CATALOGUE, |behaviour| behaviour.id)
}
