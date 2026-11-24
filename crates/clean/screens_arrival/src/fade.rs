//! Spec: `PictureBrightens`, `PlayBegins`. The fade in, from her arrival up to play.

use types::arrival::{ArrivalStage, ArrivalState, FadeFrames};
use types::brightness::Brightness;

use crate::rules::Rules;

/// A frame of the fade in: the fade takes its step, and then play begins if that step left the
/// picture fully up. (The rest of the frame plays as in any room, before the step: that is the
/// other domains'.)
pub fn fade_in_frame(state: &mut ArrivalState, rules: Rules) {
    picture_brightens(state, rules);
    play_begins(state);
}

/// Spec: `PictureBrightens`. Each frame the count goes down by one; on a frame it has run out it
/// starts again from the rhythm and the picture comes up one level, shown. So each level holds
/// `rhythm + 1` frames.
///
/// At level 15 a step restarts the count but leaves the brightness as it is: a blanked screen
/// is only ever shown by a step that brightens it.
fn picture_brightens(state: &mut ArrivalState, rules: Rules) {
    if let Some(count) = state.count.counted_down() {
        state.count = count;
        return;
    }
    state.count = rules.rhythm;
    if let Some(level) = state.brightness.level.brighter() {
        state.brightness = Brightness::shown(level);
    }
}

/// Spec: `PlayBegins`. Once the picture is shown fully up she plays, and the count is cleared.
///
/// Only a shown picture counts: a screen blanked at level 15 never comes up, so its fade in
/// never ends (no start leaves it that way).
fn play_begins(state: &mut ArrivalState) {
    if state.brightness == Brightness::FULLY_UP {
        state.stage = ArrivalStage::Playing;
        state.count = FadeFrames::NONE;
    }
}
