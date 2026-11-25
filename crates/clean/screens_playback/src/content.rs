//! The scenes' pad content: three timelines of timed holds, and the sample each turn reads.

use core::num;

use types::playback::{PadSample, TimelineProgress};

/// A hold: the same held controls for a span of room actor turns, with the fresh presses of
/// its first turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadHold {
    /// How many room actor turns the hold lasts.
    pub turns: num::NonZeroU16,
    /// The sample of its first turn; its later turns hold the same controls with no fresh press.
    pub sample: PadSample,
}

/// The scenes' pad content: the holds of the battle, of the approach to the baby, and of the
/// watch by the baby, each in the order they play.
///
/// The importer supplies each timeline nonempty, and each discovery timeline ending on a hold
/// of Left alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePad {
    /// The battle's holds.
    pub battle: Vec<PadHold>,
    /// The approach's holds.
    pub approach: Vec<PadHold>,
    /// The watch's holds.
    pub watch: Vec<PadHold>,
}

/// Spec: `DemoPlaysPad`. The sample a timeline issues at this progress, with fresh presses only
/// on the first turn of its hold; none once the timeline has run out.
pub fn sample_at(timeline: &[PadHold], progress: TimelineProgress) -> Option<PadSample> {
    let mut first_turn = TimelineProgress::START;
    for hold in timeline {
        let after = TimelineProgress(first_turn.0.checked_add(u32::from(hold.turns.get()))?);
        if progress < after {
            let sample = if progress == first_turn {
                hold.sample
            } else {
                hold.sample.held_on()
            };
            return Some(sample);
        }
        first_turn = after;
    }
    None
}
