//! The game's counts of finished frames and its random number, frame to frame.
//!
//! Each frame of work begins by drawing the game's random number once (game-random's
//! `AdvanceRandom`), which the baby and the Ceres doors read. State carries the number; the
//! caller replaces it where the loop doesn't own every draw, as through a passage.

use super::Passing;
use interface_hud::FrameCount;
use world_doors::Finishing;

/// The counts of finished frames and the game's random number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    /// The game's count of finished frames, in eight bits, which the HUD reads as the frame
    /// begins.
    pub count: FrameCount,
    /// The count of finished frames in sixteen bits, which nothing restarts; the shaft's lights
    /// read it.
    pub finished_frames: u16,
    /// The game's random number as the last frame left it.
    pub random: types::RandomSequence,
}

impl State {
    /// The random number the frame's enemies read: drawn once as a frame of work begins.
    pub(super) fn drawn(self, passing: &Passing<'_>) -> types::RandomSequence {
        let mut random = passing.random.unwrap_or(self.random);
        if passing.work_begun {
            game_random::advance_random(&mut random);
        }
        random
    }

    /// The counts after a frame the doors report `finishing` of, with the number it drew.
    pub(super) const fn after(self, finishing: Finishing, random: types::RandomSequence) -> Self {
        Self {
            count: counted(self.count, finishing),
            finished_frames: finished(self.finished_frames, finishing),
            random,
        }
    }
}

/// The frame count after a frame: a finished frame counts, and a frame the doors restart the
/// count on counts from 0 (`FinishedFrames`).
pub(super) const fn counted(frame: FrameCount, finishing: Finishing) -> FrameCount {
    match finishing {
        Finishing::Overrun => frame,
        Finishing::Finished => FrameCount(frame.0.wrapping_add(1)),
        Finishing::Restarts(_) => FrameCount(1),
    }
}

/// The count of finished frames after a frame: every frame but one whose work overran counts.
const fn finished(count: u16, finishing: Finishing) -> u16 {
    match finishing {
        Finishing::Overrun => count,
        Finishing::Finished | Finishing::Restarts(_) => count.wrapping_add(1),
    }
}
