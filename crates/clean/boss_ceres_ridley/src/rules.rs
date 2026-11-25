//! The station's tuning values, the doors', the shaft's and Ridley's, his waking's and his tail's:
//! every number a Parameter entry of the spec lists.

use core::num::{NonZeroU8, NonZeroU16, Wrapping};

use types::Px;

use crate::pad_wait::PadWait;
use crate::phase::Wait;
use crate::ridley_wait::WakeWait;
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
/// `PadWaits`: the pad's wait, in frames.
const PAD_WAIT_FRAMES: u16 = 60;
/// `PadWaits`: the pad's wait, 60 frames.
const PAD_WAIT: PadWait = PadWait::new(PAD_WAIT_FRAMES).expect("the wait is 60, within 32,768");
/// `PadCarries`: the shaft's floor, 72 px down the room.
const FLOOR: Px = Wrapping(72);
/// `BeaconCycles`: the rhythm, in frames a step.
const RHYTHM_FRAMES: u16 = 8;
/// `BeaconCycles`: the rhythm, 8 frames a step.
const RHYTHM: NonZeroU16 = NonZeroU16::new(RHYTHM_FRAMES).expect("the rhythm is 8 frames, not 0");
/// `SlamShut`: the slam's hold of each figure, in frames.
const SLAM_FRAMES: u16 = 2;
/// `SlamShut`: the slam's hold, 2 frames for each figure.
const SLAM: Wait = Wait::new(SLAM_FRAMES).expect("the slam's hold is 2 frames, not 0");
/// `RidleyWaits`: Ridley's wait, in frames.
const WAKE_FRAMES: u16 = 512;
/// `RidleyWaits`: Ridley's wait, 512 frames.
const WAKE: WakeWait = WakeWait::new(WAKE_FRAMES).expect("the wait is 512, from 1 to 32,768");
/// `BodyFadesIn`: the pace, in runs a set.
const PACE_RUNS: u8 = 2;
/// `BodyFadesIn`: the pace, 2 runs a set.
const PACE: NonZeroU8 = NonZeroU8::new(PACE_RUNS).expect("the pace is 2 runs, not 0");
/// `TailHits`: the tail's reach, in pixels.
const TAIL_REACH: u8 = 14;

/// The station's tuning values. Its default holds the spec's values; another value is an
/// adjusted game.
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
    /// `PadWaits`: the wait, the runs the pad holds her still before it starts to lower her; a
    /// rule from 0 to 32,768 frames.
    pub pad_wait: PadWait,
    /// `PadCarries`, `CoverHides`: the shaft's floor, the height the pad sets her down at.
    pub floor: Px,
    /// `BeaconCycles`: the rhythm, the frames the beacon shows each step of its cycle; a rule
    /// from 1 to 65,535 frames.
    pub rhythm: NonZeroU16,
    /// `SlamShut`: the wait of each figure of the slam of Ridley's room's door.
    pub slam: Wait,
    /// `RidleyWaits`: how long Ridley waits in the dark once the passage into his room is over.
    pub wake: WakeWait,
    /// `BodyFadesIn`: the pace, the runs his body shows each of its colour sets for as it fades
    /// in; a rule from 1 to 255 runs.
    pub pace: NonZeroU8,
    /// `TailHits`: the tail's reach, how much closer than her half-size on each axis she can
    /// come to his tail's tip before it hits her; a rule from 0 to 255 px.
    pub tail_reach: u8,
}

impl Default for Rules {
    /// The spec's values: reach 48 px, poll 2 frames, hold 5 frames, a strong shake below 128 in
    /// 4,096, the small shake 1 px for 2 frames and the strong 3 px for 4; the pad's wait 60
    /// frames, the floor 72 px and the beacon's rhythm 8 frames a step; the slam's hold 2
    /// frames, Ridley's wait 512 frames, his body's pace 2 runs a set and his tail's reach 14 px.
    fn default() -> Self {
        Self {
            reach: REACH,
            poll: POLL,
            hold: HOLD,
            strong_below: STRONG_BELOW,
            small: SMALL,
            strong: STRONG,
            pad_wait: PAD_WAIT,
            floor: FLOOR,
            rhythm: RHYTHM,
            slam: SLAM,
            wake: WAKE,
            pace: PACE,
            tail_reach: TAIL_REACH,
        }
    }
}
