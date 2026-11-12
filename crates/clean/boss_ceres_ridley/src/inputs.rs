//! What a door reads as the frame starts, and the two rules that read only it: whether she is
//! near, and whether the door starts a shake.

use types::{Pair, Px};

use crate::rules::Rules;
use crate::station::{Shake, Stage};

/// `EscapeShakes`: a door reads the random number's low 12 bits, `random mod 4096`.
const LOW_12_BITS: u16 = 0x0FFF;

/// What a door reads as the frame starts, each from the part of the game that owns it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inputs {
    /// Where she was, in whole pixels, at the end of the last frame, before anything moves her
    /// this frame (samus-movement): the doors run before she moves.
    pub her: Pair<Px>,
    /// The station's stage (progress and the escape).
    pub stage: Stage,
    /// The frame's random number as it stands when the doors run.
    pub random: u16,
    /// How many frames the screen's current shake has left when this door runs, after any
    /// shake an earlier door started this frame.
    pub shake_frames_left: u16,
}

impl Inputs {
    /// `SheIsNear`: whether she was under the reach from the door's position on both axes.
    #[must_use]
    pub const fn she_is_near(&self, door: Pair<Px>, reach: u16) -> bool {
        gap(door.x, self.her.x) < reach && gap(door.y, self.her.y) < reach
    }

    /// `EscapeShakes`: the shake a door's run starts, whether it steps or not. During the escape,
    /// while the screen is still, it starts the strong shake when the random number's low 12
    /// bits are under the chance, and the small one otherwise. Calm, in the getaway, or while a
    /// shake runs, it starts none.
    #[must_use]
    pub const fn escape_shake(&self, rules: &Rules) -> Option<Shake> {
        if !matches!(self.stage, Stage::Escape) || self.shake_frames_left != 0 {
            return None;
        }
        if self.random & LOW_12_BITS < rules.strong_below {
            Some(rules.strong)
        } else {
            Some(rules.small)
        }
    }
}

/// `SheIsNear`: the gap along one axis, `(door − her) mod 2^16`, negated `mod 2^16` when its
/// bit 15 is set. A gap of `8000` negates to itself, so it is never near; her position left of
/// the room (`FFF0`) is 24 px from a door at `0008`.
const fn gap(door: Px, her: Px) -> u16 {
    door.0.wrapping_sub(her.0).cast_signed().unsigned_abs()
}
