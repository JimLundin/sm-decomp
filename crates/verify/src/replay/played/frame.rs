//! The counts of finished frames and the game's random number: their seed and compared fields.

use super::word;
use crate::replay::compare::Observation;
use crate::replay::hud;
use crate::replay::result::Outcome;
use crate::source;
use game::production::played;

/// The counts and the number as the seed holds them.
pub(super) fn seed(wram: &[u8]) -> Result<played::frame::State, Outcome> {
    Ok(played::frame::State {
        count: hud::frame_count(wram)?,
        finished_frames: word(wram, source::NMI_FRAME_COUNTER_WORD)?,
        random: random(wram)?,
    })
}

/// The counts as the record holds them.
pub(super) fn expected(wram: &[u8]) -> Result<[Observation; 2], Outcome> {
    Ok(observations(
        hud::frame_count(wram)?,
        word(wram, source::NMI_FRAME_COUNTER_WORD)?,
    ))
}

/// The counts as the played State holds them.
pub(super) fn actual(frame: played::frame::State) -> [Observation; 2] {
    observations(frame.count, frame.finished_frames)
}

/// The game's count of finished frames, which the HUD reads as the next frame begins, and the
/// count in sixteen bits, which nothing restarts.
fn observations(count: interface_hud::FrameCount, finished_frames: u16) -> [Observation; 2] {
    [
        Observation::quantity("frame-count", i64::from(count.0)),
        Observation::quantity("finished-frames", i64::from(finished_frames)),
    ]
}

/// The game's random number as the record holds it.
pub(super) fn random(wram: &[u8]) -> Result<types::RandomSequence, Outcome> {
    Ok(types::RandomSequence {
        number: types::RandomNumber(word(wram, source::RANDOM_NUMBER)?),
    })
}

pub(super) fn random_observation(random: types::RandomSequence) -> Observation {
    Observation::quantity("random", i64::from(random.number.0))
}
