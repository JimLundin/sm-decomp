//! The catalogue: one behaviour per spec entry, with its kind, its switch and how it is checked.

use clean_types::{Checked, Kind, Switch};

/// A spec entry's id, one variant per entry, as the spec names it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BehaviourId {
    /// §2.
    FollowingHer,
    /// §2.
    LeadingHer,
    /// §2.
    AimingHeight,
    /// §2.
    Remembering,
    /// §2.
    CreepReadsRightward,
    /// §3.
    ClosedScreens,
    /// §3.
    RoomEdges,
    /// §3.
    ShortScreens,
    /// §3.
    Settling,
    /// §3.
    TimeStopped,
    /// §3.
    DownwardLooksAhead,
    /// §3.
    ScreensRunOn,
    /// §3.
    BelowTheRoom,
    /// §3.
    PushCarries,
}

/// A catalogue entry.
pub struct Behaviour {
    /// Its spec entry.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// What kind of behaviour it is.
    pub kind: Kind,
    /// How it can be adjusted.
    pub switch: Switch,
    /// Whether vectors check it, or the original only.
    pub checked: Checked,
}

/// Every spec entry, in the spec's order.
pub const CATALOGUE: [Behaviour; 14] = [
    Behaviour {
        id: BehaviourId::FollowingHer,
        summary: "when she moves, the view moves towards its aim as fast as she moved plus a \
                  pixel, and never past the aim",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::LeadingHer,
        summary: "across, the view aims to show more of the way she faces; going backwards, \
                  more of the way she goes",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::AimingHeight,
        summary: "down, the view aims to keep her a set height below its top edge, and lower \
                  while she rises, so it shows more above her",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Remembering,
        summary: "the camera remembers where it last saw her, which sets its next speed and \
                  whether she moved",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::CreepReadsRightward,
        summary: "a creep left or up inside a pixel reads as a move right or down of less than \
                  a pixel, so the view eases 2 px that frame, not 3",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ClosedScreens,
        summary: "a closed screen is never shown: a view that goes into one is pushed back to \
                  its edge, backing off 2 px a frame against her move",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RoomEdges,
        summary: "the view never shows past the room's left, top or right edge, nor below its \
                  bottom",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ShortScreens,
        summary: "where the view can go no lower, it stops with its top on a short screen's top \
                  edge, or 31 px lower on any other screen",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Settling,
        summary: "on an axis she didn't move on, the view, brought into the room, eases out of \
                  a closed screen it partly shows at 3 px a frame",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TimeStopped,
        summary: "while time is stopped the view doesn't settle",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::DownwardLooksAhead,
        summary: "going down, the view checks the screens from where its move took it, before \
                  the aim stopped it, so it can back up past the aim",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ScreensRunOn,
        summary: "the screen after a row's last column is the first screen of the row below, \
                  so a view at the room's right edge can ease past it",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BelowTheRoom,
        summary: "below the room's last row, the view reads the screens the room's loading left \
                  there, from earlier rooms",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PushCarries,
        summary: "a push's sum carries once more",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::OriginalOnly,
    },
];
