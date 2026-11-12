//! The doors' tuning values: every number a Parameter entry of the spec lists.

use crate::phase::Wait;
use crate::station::Shake;

/// `SheIsNear`: the reach, in pixels.
const REACH: u16 = 48;
/// `Polling`: the poll, in frames.
const POLL_FRAMES: u16 = 2;
/// `Polling`: the poll, 2 frames.
const POLL: Wait = Wait::new(POLL_FRAMES).expect("the poll is 2 frames, not 0");
/// `Opening`, `Closing`: the hold of each figure, in frames.
const HOLD_FRAMES: u16 = 5;
/// `Opening`, `Closing`: the hold, 5 frames for each figure.
const HOLD: Wait = Wait::new(HOLD_FRAMES).expect("the hold is 5 frames, not 0");
/// `EscapeShakes`: the chance of a strong shake, `c` in 4,096.
const STRONG_BELOW: u16 = 128;
/// `EscapeShakes`: the small shake, 1 px for 2 frames.
const SMALL: Shake = Shake {
    strength: 1,
    length: 2,
};
/// `EscapeShakes`: the strong shake, 3 px for 4 frames.
const STRONG: Shake = Shake {
    strength: 3,
    length: 4,
};

/// The doors' tuning values. Its default holds the spec's values; another value is an adjusted
/// game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// `SheIsNear`: a door sees her when she was under this many pixels from it on both axes.
    pub reach: u16,
    /// `Polling`: the wait of a closed or open door between its looks for her, and of a door that
    /// has just started, opened or closed.
    pub poll: Wait,
    /// `Opening`, `Closing`: the wait of each figure of an opening or a closing.
    pub hold: Wait,
    /// `EscapeShakes`: a door's shake is strong when the random number's low 12 bits are under
    /// this, out of 4,096.
    pub strong_below: u16,
    /// `EscapeShakes`: the small shake.
    pub small: Shake,
    /// `EscapeShakes`: the strong shake.
    pub strong: Shake,
}

impl Default for Rules {
    /// The spec's values: reach 48 px, poll 2 frames, hold 5 frames, a strong shake below 128 in
    /// 4,096, the small shake 1 px for 2 frames and the strong 3 px for 4.
    fn default() -> Self {
        Self {
            reach: REACH,
            poll: POLL,
            hold: HOLD,
            strong_below: STRONG_BELOW,
            small: SMALL,
            strong: STRONG,
        }
    }
}
