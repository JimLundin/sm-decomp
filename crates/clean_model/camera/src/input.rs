//! What the camera reads each frame (§1.3): her motion's outputs, the room and the frame.

use clean_types::{Facing, Fixed, Position, Px};

/// The kind of a screen of the room (§1.3, Screen kinds). Listed alphabetically; the order means
/// nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScreenKind {
    /// The view may not show it; a view that would is pushed back to its edge (`ClosedScreens`).
    Closed,
    /// The view may show all of it.
    Open,
    /// Like `Open`, but where the view can go no lower it stops with its top on the screen's top
    /// edge (`ShortScreens`).
    Short,
}

/// `AimingHeight`: the room's two distances in px from the view's top edge down to her.
#[derive(Clone, Copy, Debug)]
pub struct Heights {
    /// `hn`, while she isn't rising.
    pub level: Px,
    /// `hr`, while she rises.
    pub rising: Px,
}

/// The room as the camera reads it (§1.3), as its loading supplies it (§5).
#[derive(Clone, Debug)]
pub struct Room {
    /// Its screens, one row of screens a `Vec`, top row first, each left to right: so many
    /// across (`w`, the first row's length) and down (`h`).
    pub screens: Vec<Vec<ScreenKind>>,
    /// `BelowTheRoom`: the rows the room's loading left below its last row, top first; a row may
    /// be shorter than `w`.
    pub below: Vec<Vec<ScreenKind>>,
    /// `AimingHeight`: the room's two aim heights.
    pub heights: Heights,
}

/// `LeadingHer`: the lead setting, `Ahead` unless a boss sets another (§5). Listed
/// alphabetically; the order means nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum LeadSetting {
    /// More of the way she faces.
    #[default]
    Ahead,
    /// Her near the view's left edge either way.
    KeepLeft,
    /// Her near the view's right edge either way.
    KeepRight,
    /// A short lead.
    Near,
}

/// What the camera reads of her this frame, her motion's outputs (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Samus {
    /// Her position, after everything that moved her this frame.
    pub position: Position,
    /// `Remembering`: her motion's y the camera follows her from: her start y, or where pose fit
    /// or a change of shape placed her.
    pub camera_from: Fixed,
    /// `LeadingHer`: her facing.
    pub facing: Facing,
    /// `LeadingHer`: she is going backwards: she skids, moonwalks or takes a knockback.
    pub backwards: bool,
    /// `AimingHeight`: she is rising, from the rise's start until it ends, a rise at 0 and the
    /// frame that spends the rise included.
    pub rising: bool,
}

/// One frame's inputs (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Frame<'a> {
    /// What the camera reads of her.
    pub samus: Samus,
    /// The room she is in.
    pub room: &'a Room,
    /// `LeadingHer`: the lead setting.
    pub lead: LeadSetting,
    /// `TimeStopped`: whether time is stopped.
    pub time_stopped: bool,
}
