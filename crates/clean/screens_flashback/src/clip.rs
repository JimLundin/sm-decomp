//! How every clip of the encounter runs: a picture holds for its beats, then the next step
//! shows for its own full hold.

use types::flashback::Animation;

/// One beat of a clip: spend a beat of the current hold, or, when it expires, take the step
/// `next` gives with its full hold.
pub fn advanced<S: Copy>(clip: Animation<S>, next: impl FnOnce(S) -> Animation<S>) -> Animation<S> {
    clip.hold.counted_down().map_or_else(
        || next(clip.step),
        |hold| Animation {
            step: clip.step,
            hold,
        },
    )
}
