//! What the camera reads each frame (Spec: §1.3): her motion's outputs, the room, the lead
//! setting and whether time is stopped.

use clean_types::{Facing, Fixed, Position};

use crate::room::Room;

/// The lead setting: which lead the view keeps from her (Spec: LeadingHer). It is `Ahead` unless
/// a boss sets another. Listed alphabetically; the order means nothing. The leads each gives are
/// in `Rules`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum LeadSetting {
    /// The view shows more of the way she travels: in the reference profile she is 96 px from the
    /// view's edge behind her.
    #[default]
    Ahead,
    /// She is kept near the view's left edge, whichever way she travels: 32 px from it in the
    /// reference profile.
    KeepLeft,
    /// She is kept near the view's right edge, whichever way she travels: 224 px from its left
    /// edge in the reference profile.
    KeepRight,
    /// In the reference profile she is 64 px from the view's left edge when she travels right, so
    /// it shows more ahead of her than `Ahead` does, and 80 px when she travels left, so it shows
    /// less.
    Near,
}

/// What the camera reads of her this frame, her motion's outputs (Spec: §1.3).
#[derive(Clone, Copy, Debug)]
pub struct Samus {
    /// Her position, after everything that moved her this frame.
    pub position: Position,
    /// The y her motion says the camera follows her from: her start y, or where pose fit or a
    /// change of shape placed her (Spec: Remembering).
    pub camera_from: Fixed,
    /// Which way she faces (Spec: LeadingHer).
    pub facing: Facing,
    /// Whether she is going backwards: she skids, moonwalks or takes a knockback (Spec:
    /// LeadingHer).
    pub backwards: bool,
    /// Whether she is rising: from the rise's start until it ends, a rise at 0 and the frame that
    /// spends the rise included (Spec: AimingHeight).
    pub rising: bool,
}

/// One frame's inputs (Spec: §1.3).
#[derive(Clone, Copy, Debug)]
pub struct Frame<'a> {
    /// What the camera reads of her.
    pub samus: Samus,
    /// The room she is in.
    pub room: &'a Room,
    /// The lead setting (Spec: LeadingHer).
    pub lead: LeadSetting,
    /// Whether time is stopped, which keeps the view from settling (Spec: TimeStopped).
    pub time_stopped: bool,
}
