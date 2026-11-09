//! `Jumping`, jump held through (§1.2): jump held through a landing, or through the end of a
//! turn or of a landing, counts as a press for the pad chart the next frame, so she takes off
//! again without pressing it.

use crate::chart::Moment;
use crate::rules::Rules;
use clean_types::Action::{Landing, Turning};
use clean_types::button::JUMP;
use core::num::Wrapping;

/// §1.2 landed or ended the frame before: what a frame leaves the next one, for that frame alone.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Before {
    /// She landed from a straight jump, a fall, a spin or a wall jump, or the animation of a turn
    /// or of a landing ended and handed over its pose.
    LandedOrEnded,
    /// Anything else.
    Nothing,
}

/// `Jumping` (§1.2): how long jump has stayed held since it was pressed, and what the frame
/// before left.
///
/// Only frames count it, so a caller starts one at `JumpHeld::NONE`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct JumpHeld {
    /// The frames jump has stayed held since its press, 0 to 65,535; from 65,535 it goes to 0.
    frames: Wrapping<u16>,
    /// What the frame before left.
    before: Before,
}

impl JumpHeld {
    /// Jump not held, and nothing the frame before.
    pub const NONE: Self = Self {
        frames: Wrapping(0),
        before: Before::Nothing,
    };

    /// `Jumping`: whether jump counts as just pressed for the pad chart this frame: the frame
    /// before, she landed or an animation ended, and the count is one that counts.
    #[must_use]
    pub(crate) fn presses(self, r: &Rules) -> bool {
        self.before == Before::LandedOrEnded && counts(self.frames, r)
    }

    /// `Jumping`: what she remembers at the end of the frame `moment` reads. A press the count
    /// made this frame (`Moment::jump_held_through`) puts it back to 0 first; then jump held and
    /// not just pressed counts one frame more, and jump just pressed or not held puts it at 0.
    /// The press this reads is the pad's own.
    #[must_use]
    pub(crate) fn then(self, moment: &Moment) -> Self {
        let f = moment.frame;
        let held = if moment.jump_held_through {
            Wrapping(0)
        } else {
            self.frames
        };
        let frames = if f.pad.any(JUMP) && !f.pressed.any(JUMP) {
            held + Wrapping(1)
        } else {
            Wrapping(0)
        };
        Self {
            frames,
            before: left_by(moment),
        }
    }
}

/// `Jumping`: the counts that make a press: 1 to 8, and, a quirk the original has, 32,777 to
/// 65,535 (jump held about 9 minutes or more), which act as one of 1 to 8 does.
fn counts(frames: Wrapping<u16>, r: &Rules) -> bool {
    r.held_through.contains(&frames.0) || r.held_long.contains(&frames.0)
}

/// §1.2: what a frame leaves the next. She landed: `WhatHerMovesMet` chose a `Landing`, whatever
/// pose fit then made of the change. Or the animation of a turn or of a landing ended and handed
/// over its pose (`AnimationEnds`).
fn left_by(moment: &Moment) -> Before {
    let landed = moment
        .met()
        .is_some_and(|c| matches!(c.pose.action, Landing(_)));
    let ended =
        moment.frame.animation_ends && matches!(moment.pose.action, Turning(_) | Landing(_));
    if landed || ended {
        Before::LandedOrEnded
    } else {
        Before::Nothing
    }
}
