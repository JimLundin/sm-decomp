//! Area follow (§2): where the view aims and how fast it moves there.
//!
//! `FollowingHer`, `LeadingHer`, `AimingHeight`, `CreepReadsRightward`; `Remembering` is the
//! step's order (the crate root).

use clean_types::{Facing, Fixed, Pair, Parts, Px, below, px};

use crate::input::{Frame, LeadSetting};
use crate::rules::Rules;

/// `FollowingHer`: which way the camera moves on an axis, towards its aim.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Going {
    /// Left or up.
    Back,
    /// Right or down.
    Forward,
}

/// `FollowingHer`: the camera's move on an axis, before the room limits it (§3).
#[derive(Clone, Copy, Debug)]
pub struct Move {
    /// Where it ends: where the move took it, or stopped on the aim.
    pub to: Fixed,
    /// `cm`: the pixel part where the move took it before the aim stopped it.
    pub reach: Px,
    /// Which way it moved.
    pub going: Going,
}

/// `LeadingHer` and `AimingHeight`: the point the view aims its top-left corner at.
pub fn aim(rules: &Rules, frame: &Frame) -> Pair<Px> {
    let samus = &frame.samus;
    let leads = match frame.lead {
        LeadSetting::Ahead => rules.ahead,
        LeadSetting::KeepLeft => rules.keep_left,
        LeadSetting::KeepRight => rules.keep_right,
        LeadSetting::Near => rules.near,
    };
    // LeadingHer: going backwards turns the lead round, to the way she travels.
    let way = if samus.backwards {
        samus.facing.back()
    } else {
        samus.facing
    };
    let lead = if way == Facing::Left {
        leads.left
    } else {
        leads.right
    };
    // AimingHeight: a rise at 0 and the frame that spends the rise aim as a rise.
    let height = if samus.rising {
        frame.room.heights.rising
    } else {
        frame.room.heights.level
    };
    Pair {
        x: samus.position.x.pixel() - lead,
        y: samus.position.y.pixel() - height,
    }
}

/// `FollowingHer`: the speed, how far she moved from where it last saw her, plus the extra pixel.
///
/// `CreepReadsRightward`: the way she moved is read on the pixel parts alone, so a creep left or
/// up inside a pixel reads as a move right or down of under a pixel.
pub fn speed(her: Fixed, seen: Fixed, extra: Fixed) -> Fixed {
    let moved = if below(her.pixel().0, seen.pixel().0) {
        seen - her
    } else {
        her - seen
    };
    moved + extra
}

/// `FollowingHer`: the camera moves towards its aim by the speed, and never past it; with the
/// aim's pixel part its own it stays as it is (`None`).
pub fn chase(camera: Fixed, aim: Px, speed: Fixed) -> Option<Move> {
    let c = camera.pixel();
    if aim == c {
        return None;
    }
    let forward = below(c.0, aim.0);
    let reach = if forward {
        camera + speed
    } else {
        camera - speed
    };
    let (first, last) = if forward {
        (aim, reach.pixel())
    } else {
        (reach.pixel(), aim)
    };
    Some(Move {
        to: if below(first.0, last.0) {
            px(aim)
        } else {
            reach
        },
        reach: reach.pixel(),
        going: if forward { Going::Forward } else { Going::Back },
    })
}
