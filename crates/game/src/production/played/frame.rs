//! The game's counts of finished frames and its random number, frame to frame.
//!
//! Each frame of work begins by drawing the game's random number once (game-random's
//! `AdvanceRandom`), which the baby and the Ceres doors read. State carries the number; the
//! caller replaces it where the loop doesn't own every draw, as through a passage. Ahead of
//! room play the front end's screens carry the same draw and the sixteen-bit count as [`Counts`].

use super::passage::Passing;
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

/// The counts ahead of room play, from the title on: the count of finished frames in sixteen
/// bits and the random number, which the screens leave to the frame owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counts {
    /// The count of finished frames, which the power-on restarts as the game settles.
    pub finished_frames: u16,
    /// The game's random number as the last frame left it.
    pub random: types::RandomSequence,
}

impl Counts {
    /// The counts after a frame end at which `finished` frames of work finished, `drawn` of
    /// them having drawn the random number as they began. A frame of work drew as it began,
    /// but only a finished frame end shows it, so it is counted where the frame finishes.
    pub(super) fn after(self, finished: u16, drawn: u16) -> Self {
        let mut random = self.random;
        for _ in 0..drawn {
            game_random::advance_random(&mut random);
        }
        Self {
            finished_frames: self.finished_frames.wrapping_add(finished),
            random,
        }
    }
}

/// The frame count after a frame: a finished frame counts, and a frame the doors restart the
/// count on counts from 0 (`FinishedFrames`).
const fn counted(frame: FrameCount, finishing: Finishing) -> FrameCount {
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

#[cfg(test)]
mod tests {
    use super::counted;
    use interface_hud::FrameCount;
    use world_doors::Finishing;

    #[test]
    fn a_finished_frame_counts_and_a_restarting_one_counts_from_zero() {
        let last = FrameCount(0xff);
        assert_eq!(counted(last, Finishing::Overrun), last);
        assert_eq!(counted(last, Finishing::Finished), FrameCount(0));
        assert_eq!(
            counted(last, Finishing::Restarts(world_doors::SlideScreen::Sprites)),
            FrameCount(1)
        );
    }
}
