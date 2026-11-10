//! The catalogue: one behaviour per spec entry, with its kind, its switch and how it is checked.
//!
//! An entry's switch is `Parameter` where `Rules` holds its values, and `Fixed` otherwise. The
//! spec names no toggle. An entry whose values are an input, as `AimingHeight`'s heights are the
//! room's, or another entry's, as `Settling`'s ease is `ClosedScreens`' push, is `Fixed`: `Rules`
//! holds nothing of its own to adjust, so a caller changes it through that input or that entry.
//! `LeadingHer`'s setting is an input too, but the leads each setting gives are in `Rules`, so it
//! is a `Parameter`.

use clean_types::{Checked, Kind, Switch};

/// A spec entry's id, one variant per entry, named as the spec names it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BehaviourId {
    /// The view moves towards its aim as fast as she moved plus an extra amount.
    FollowingHer,
    /// Across, the view aims to keep her a set distance from its left edge.
    LeadingHer,
    /// Down, the view aims to keep her a set height below its top edge.
    AimingHeight,
    /// The camera remembers where it last saw her.
    Remembering,
    /// A creep left or up inside a pixel reads as a move right or down.
    CreepReadsRightward,
    /// A closed screen is never shown.
    ClosedScreens,
    /// A view in the room never shows past its edges; one left outside it comes back.
    RoomEdges,
    /// Where the view can go no lower, it stops on a short screen's top edge, or the margin lower.
    ShortScreens,
    /// On an axis she didn't move on, the view eases out of a closed screen.
    Settling,
    /// While time is stopped the view doesn't settle.
    TimeStopped,
    /// Going down, the view checks the screens from where its move took it.
    DownwardLooksAhead,
    /// The screen after a row's last column is the first of the row below.
    ScreensRunOn,
    /// Below the room's last row, the view reads the screens the room's loading left after its
    /// last screen.
    BelowTheRoom,
    /// A push's sum carries once more.
    PushCarries,
}

/// A catalogue entry.
pub struct Behaviour {
    /// Its spec entry.
    pub id: BehaviourId,
    /// What the player sees: the spec entry's summary.
    pub summary: &'static str,
    /// What kind of behaviour it is.
    pub kind: Kind,
    /// How it can be adjusted: `Parameter` where `Rules` holds its values, `Fixed` otherwise.
    pub switch: Switch,
    /// Whether vectors check it, or the original only.
    pub checked: Checked,
}

/// Every spec entry, in the spec's order.
pub const CATALOGUE: [Behaviour; 14] = [
    Behaviour {
        id: BehaviourId::FollowingHer,
        summary: "when she moves, the view moves towards its aim as fast as she moved plus an \
                  extra amount (a pixel in the reference profile), and never past the aim.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::LeadingHer,
        summary: "across, the view aims to keep her a set distance from its left edge, by the \
                  lead setting and the way she travels: she travels the way she faces, facing \
                  the screen as facing right, or the other way while she goes backwards.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::AimingHeight,
        summary: "down, the view aims to keep her a set height below its top edge, and lower \
                  while she rises, so it shows more above her.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Remembering,
        summary: "the camera remembers where it last saw her, which sets its next speed and \
                  whether she moved.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::CreepReadsRightward,
        summary: "a creep left or up inside a pixel reads as a move right or down of less than \
                  a pixel, so the view eases a pixel less that frame.",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ClosedScreens,
        summary: "a closed screen is never shown: a view that goes into one is pushed back to \
                  its edge, backing off by the push each frame against her move (2 px in the \
                  reference profile).",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RoomEdges,
        summary: "a view in the room never shows past its left, top or right edge, nor below its \
                  bottom; one another domain left outside it comes back, at once (at times to \
                  the far edge) or a few pixels a frame.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ShortScreens,
        summary: "where the view can go no lower, it stops with its top on a short screen's top \
                  edge, or the margin lower on any other screen (31 px in the reference \
                  profile), which shows that much more of that screen's bottom.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Settling,
        summary: "on an axis she didn't move on, the view, brought into the room, eases out of \
                  a closed screen it partly shows a few pixels a frame (3 in the reference \
                  profile), towards the open screen beside it.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TimeStopped,
        summary: "while time is stopped the view doesn't settle.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::DownwardLooksAhead,
        summary: "going down, the view checks the screens from where its move took it, before \
                  the aim stopped it, so a move that crosses into the next screen can back it \
                  up past the aim.",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ScreensRunOn,
        summary: "the screen after a row's last column is the first screen of the row below, \
                  so a view at the room's right edge can ease past it.",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BelowTheRoom,
        summary: "below the room's last row, the view reads the screens the room's loading left \
                  after the room's last screen.",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PushCarries,
        summary: "a push's sum carries once more.",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
];
