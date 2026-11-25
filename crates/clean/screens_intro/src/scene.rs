//! The scene as a whole, cleared as the intro begins (`IntroBegins`) and as it ends
//! (`IntroEnds`).

use types::intro::{ClickTurn, Glows, IntroState};

/// Everything of the scene cleared: the dissolve's count 0, the picture's height 0, no baby, no
/// page, no eyes, no caret, no glow, the click turn off.
pub fn clear_scene(state: &mut IntroState) {
    state.dissolve = 0;
    state.height = 0;
    state.baby = None;
    state.page = None;
    state.eyes = None;
    state.caret = None;
    state.glows = Glows::none();
    state.click = ClickTurn::Off;
}
