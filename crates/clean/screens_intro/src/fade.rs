//! The fade out and the end (`PictureDarkens`, `IntroEnds`): the picture darkens a level at a
//! time to black, and once it is blanked the intro is over.

use types::intro::{
    Blanking, Brightness, BrightnessLevel, ClickTurn, FadeFrames, Glows, IntroStage, IntroState,
};

use crate::report::{IntroEvent, Report};
use crate::rules::Rules;

/// Black and blanked: where the fade ends.
const BLANKED: Brightness = Brightness {
    level: BrightnessLevel::BLACK,
    blanking: Blanking::Blanked,
};

/// `PictureDarkens`: while the fade's count is over 0 it counts down; at 0 it starts again from
/// the rhythm `d` and the picture darkens a level, so each level holds `d + 1` frames.
pub fn picture_darkens(state: &mut IntroState, rules: Rules) {
    if let Some(fewer) = state.fade.fewer() {
        state.fade = fewer;
    } else {
        state.fade = rules.rhythm;
        state.brightness = darkened(state.brightness);
    }
}

/// `PictureDarkens`: one level darker and shown; from level 1, black and blanked; at level 0 as
/// it was.
fn darkened(brightness: Brightness) -> Brightness {
    let darker = brightness.level.darker();
    if brightness.level == BrightnessLevel::BLACK {
        brightness
    } else if darker == BrightnessLevel::BLACK {
        BLANKED
    } else {
        Brightness {
            level: darker,
            blanking: Blanking::Shown,
        }
    }
}

/// `IntroEnds`: once the picture is black and blanked the intro is done; everything of the scene
/// is cleared and the equipment lent for the played scenes is returned. Any other brightness
/// keeps it fading.
pub fn intro_ends(state: &mut IntroState, report: &mut Report) {
    if state.brightness != BLANKED {
        return;
    }
    state.stage = IntroStage::Done;
    state.dissolve = 0;
    state.height = 0;
    state.baby = None;
    state.page = None;
    state.eyes = None;
    state.caret = None;
    state.glows = Glows::none();
    state.click = ClickTurn::Off;
    state.fade = FadeFrames::ZERO;
    report.events.push(IntroEvent::EquipmentReturned);
}
