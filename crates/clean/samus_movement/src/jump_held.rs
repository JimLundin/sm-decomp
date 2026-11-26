//! `Jumping`, jump held through (§1.2): jump held through a landing, or through the end of a
//! turn or of a landing, counts as a press for the pad chart the next frame, so she takes off
//! again without pressing it. Every draw that updates jump history counts it on against the
//! previous such draw's jump: one while her motion is paused does so by itself
//! (`DrawnJumpHistory`), and an active frame with the rest of her frame.

use crate::rows::Met;
use crate::rules::Rules;
use crate::tick::Frame;
use core::num::Wrapping;
use types::button::JUMP;
use types::{Action, Buttons, Pose};

/// §1.2 landed or ended the frame before: what a frame leaves the next one, for that frame alone.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
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

    /// `Jumping`: what she remembers at the end of frame `f`, whose draw updates jump history,
    /// with what it left behind for the next (`before`). A press the count made this frame puts
    /// it back to 0 first; then the draw counts on from the `previous` history-writing draw's
    /// jump, as `counted_on` says. Only the count reads `previous`: the pad chart and her
    /// controls read the frame's own buttons.
    pub(crate) fn then(
        self,
        r: &Rules,
        f: &Frame,
        previous: JumpButton,
        before: FrameBefore,
    ) -> Self {
        let held = if self.presses(r) {
            Wrapping(0)
        } else {
            self.frames
        };
        Self {
            frames: counted_on(held, previous, JumpButton::of(f.pad)),
            before,
        }
    }
}

/// `Jumping`: a draw that updates jump history counts on from `frames`: one more where jump is
/// held `now` and was on the `previous` such draw (65,535 goes to 0), else 0.
fn counted_on(frames: Wrapping<u16>, previous: JumpButton, now: JumpButton) -> Wrapping<u16> {
    if previous == JumpButton::Held && now == JumpButton::Held {
        frames + Wrapping(1)
    } else {
        Wrapping(0)
    }
}

/// `Jumping`: jump as a draw that updates jump history samples it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum JumpButton {
    /// Jump is held.
    Held,
    /// Jump is up.
    Up,
}

impl JumpButton {
    /// Jump as `pad` holds it.
    const fn of(pad: Buttons) -> Self {
        if pad.any(JUMP) { Self::Held } else { Self::Up }
    }

    /// `Jumping`: the previous history-writing draw's jump, as a frame `f` that follows it with
    /// no passage between tells it: held where jump is held and not just pressed, so it was held
    /// the frame before. Up otherwise: jump up now puts the count at 0 whatever came before.
    pub(crate) const fn previous_draw(f: &Frame) -> Self {
        if f.pressed.any(JUMP) {
            Self::Up
        } else {
            Self::of(f.pad)
        }
    }
}

/// `Jumping`: jump held through, with the jump of the previous draw that updated it.
///
/// A caller keeps it across such draws while her motion and pose choice are paused (a door
/// passage). State a caller gives and reads back. Between draws it holds as it is; the frame before
/// stays for the next frame she is active.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct DrawnJumpHistory {
    /// The count and what the last active frame left.
    pub jump_held: JumpHeld,
    /// Jump as the previous draw that updated jump history sampled it.
    pub last_draw: JumpButton,
}

impl DrawnJumpHistory {
    /// `Jumping`: a draw that updates jump history, sampling `jump`, without the rest of her
    /// frame. Jump held now and on the previous such draw counts one more; jump up on either
    /// puts the count at 0. The controller's own press plays no part, and the frame before is
    /// kept as it is. Then this draw's jump becomes the previous draw's.
    pub fn draw(&mut self, jump: JumpButton) {
        self.jump_held.frames = counted_on(self.jump_held.frames, self.last_draw, jump);
        self.last_draw = jump;
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
    /// animation's end selected a pose (`handed_over`), the pose she started it in (`was`)
    /// and the one she ends it in (`now`). She landed: her moves met a floor and she
    /// ends in `Landing`, the pose `WhatHerMovesMet` chose and pose fit took; a landing pose fit
    /// refuses or turns into the crouch is none (`PoseFitAfterMoves`). Or the animation of a
    /// turn or of a landing selected a pose (`AnimationEnds`), independently of later fitting:
    /// the same selected end supplies this history. An end that gave way selected nothing.
    pub(crate) fn left_behind(met: Option<Met>, handed_over: bool, was: Pose, now: Pose) -> Self {
        let landed = met == Some(Met::Floor) && matches!(now.action, Action::Landing(_));
        let ended = handed_over && matches!(was.action, Action::Turning(_) | Action::Landing(_));
        if landed || ended {
            Self::LandedOrEnded
        } else {
            Self::Other
        }
    }
}
