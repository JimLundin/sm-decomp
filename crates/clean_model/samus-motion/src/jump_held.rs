//! `Jumping`, jump held through (§1.2): jump held through a landing, or through the end of a
//! turn or of a landing, counts as a press for the pad chart the next frame, so she takes off
//! again without pressing it.

use crate::rows::Met;
use crate::rules::Rules;
use crate::tick::Frame;
use clean_types::Action::{Landing, Turning};
use clean_types::button::JUMP;
use clean_types::{Buttons, Pose};
use core::num::Wrapping;

/// §1.2 landed or ended the frame before: what a frame leaves the next one, for that frame alone.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FrameBefore {
    /// She landed from a straight jump, a fall, a spin or a wall jump, or the animation of a turn
    /// or of a landing ended and handed over its pose.
    LandedOrEnded,
    /// She neither landed nor had such an animation end hand over its pose.
    Other,
}

/// `Jumping` (§1.2): how long jump has stayed held since it was pressed, and what the frame
/// before left.
///
/// State a caller gives and reads back: any count and either frame before, in any pose. Each
/// frame starts from them and leaves them for the next.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct JumpHeld {
    /// The frames jump has stayed held since its press, 0 to 65,535; from 65,535 it goes to 0.
    pub frames: Wrapping<u16>,
    /// What the frame before left.
    pub before: FrameBefore,
}

impl JumpHeld {
    /// Jump not held, and nothing the frame before.
    pub const NONE: Self = Self {
        frames: Wrapping(0),
        before: FrameBefore::Other,
    };

    /// `Jumping`: whether jump counts as just pressed for the pad chart this frame: the frame
    /// before, she landed or an animation ended, and the count is one that counts. It counts so
    /// with jump let go on this frame too: only the frame's end sees it let go.
    fn presses(self, r: &Rules) -> bool {
        self.before == FrameBefore::LandedOrEnded && counts(self.frames, r)
    }

    /// `PadChart`, `Jumping`: the buttons the pad chart reads as just pressed this frame: the
    /// pad's (`pressed`), and jump where jump held through counts as a press.
    pub(crate) fn pressed(self, pressed: Buttons, r: &Rules) -> Buttons {
        if self.presses(r) {
            Buttons(pressed.0 | JUMP.0)
        } else {
            pressed
        }
    }

    /// `Jumping`: what she remembers at the end of frame `f`, with what it left behind for the
    /// next (`before`). A
    /// press the count made this frame puts it back to 0 first; then jump held and not just
    /// pressed counts one frame more, and jump just pressed or not held puts it at 0. The press
    /// this reads is the pad's own.
    pub(crate) fn then(self, r: &Rules, f: &Frame, before: FrameBefore) -> Self {
        let held = if self.presses(r) {
            Wrapping(0)
        } else {
            self.frames
        };
        let frames = if f.pad.any(JUMP) && !f.pressed.any(JUMP) {
            held + Wrapping(1)
        } else {
            Wrapping(0)
        };
        Self { frames, before }
    }
}

/// `Jumping`: the counts of frames jump has stayed held that make a press: 1 to 8, and, a quirk
/// the original has, 32,777 to 65,535 frames (jump held about 9 minutes or more), which act as
/// one of 1 to 8 does.
fn counts(frames: Wrapping<u16>, r: &Rules) -> bool {
    r.held_through.contains(&frames.0) || r.held_long.contains(&frames.0)
}

impl FrameBefore {
    /// §1.2: what a frame leaves behind for the next, from what her moves met, whether her
    /// animation's end handed over the pose she ends in (`handed_over`), the pose she started it
    /// in (`was`) and the one she ends it in (`now`). She landed: her moves met a floor and she
    /// ends in `Landing`, the pose `WhatHerMovesMet` chose and pose fit took; a landing pose fit
    /// refuses or turns into the crouch is none (`PoseFitAfterMoves`). Or the animation of a
    /// turn or of a landing ended and handed over its pose (`AnimationEnds`); an end that gave
    /// way handed over nothing.
    pub(crate) fn left_behind(met: Option<Met>, handed_over: bool, was: Pose, now: Pose) -> Self {
        let landed = met == Some(Met::Floor) && matches!(now.action, Landing(_));
        let ended = handed_over && matches!(was.action, Turning(_) | Landing(_));
        if landed || ended {
            Self::LandedOrEnded
        } else {
            Self::Other
        }
    }
}
