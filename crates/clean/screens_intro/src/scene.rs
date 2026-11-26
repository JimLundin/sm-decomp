//! The scene as a whole, cleared as the intro begins (`IntroBegins`) and as it ends
//! (`IntroEnds`), and the cue that turns a scene back toward the text (`PageTwoBegins`,
//! `DiscoveryReturns`, `PageFourBegins`).

use types::intro::{ClickTurn, Glows, IntroStage, IntroState, Pan};

use crate::report::{IntroEvent, Report, TextTiles};

/// The height the scene's picture settles at, px: where page 1's picture stands
/// (`PageOneIsMadeReady`), and where the baby's rise stops it (`BabyRises`).
pub const SETTLED_PICTURE_HEIGHT: i16 = 8;

/// Everything of the scene cleared: the dissolve's count 0, the picture's scroll (0, 0), no
/// baby, no page, no eyes, no caret, no glow, the click turn off.
pub fn clear_scene(state: &mut IntroState) {
    state.dissolve = 0;
    state.pan = Pan::AT_REST;
    state.height = 0;
    state.baby = None;
    state.page = None;
    state.eyes = None;
    state.caret = None;
    state.glows = Glows::none();
    state.click = ClickTurn::Off;
}

/// A scene's return to the text is cued while the scene `shows`: the stage becomes `pending`, its
/// page beginning at the next frame's stage step, and the text's tiles pause on this frame
/// (`PageTwoBegins`, `DiscoveryReturns`, `PageFourBegins`). In any other stage the cue is not
/// read.
pub fn return_cued(
    state: &mut IntroState,
    shows: IntroStage,
    pending: IntroStage,
    report: &mut Report,
) {
    if state.stage == shows {
        state.stage = pending;
        report
            .events
            .push(IntroEvent::TextUpdates(TextTiles::Paused));
    }
}
