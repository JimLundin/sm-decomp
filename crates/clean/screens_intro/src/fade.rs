//! The brightness fades. The slow fades of the opening and page 1 (`SlowFade`) brighten or
//! darken a level every 2 frames; the fade out at the end (`PictureDarkens`, `IntroEnds`)
//! darkens a level every `d + 1` frames to black, and once it is blanked the intro is over.

use types::intro::{Blanking, Brightness, BrightnessLevel, FadeFrames, IntroStage, IntroState};

use crate::report::{Equipment, IntroEvent, Report};
use crate::rules::Rules;
use crate::scene::clear_scene;

/// Which way a slow fade goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FadeWay {
    /// Toward full, 15.
    Brightens,
    /// Toward black, blanked.
    Darkens,
}

/// Whether a slow fade goes on or is done.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fading {
    /// It goes on.
    Going,
    /// It has reached its end: the stage moves on.
    Done,
}

/// `SlowFade`: one frame of a slow fade. While its count is over 1 it counts down; at 1 or 0 the
/// picture moves a level its way and the count starts again from the pace, 2, until the level
/// reaches its end: 15 brightening, black and blanked darkening, where the count is 0 and the
/// fade is done.
pub fn slow_fade(state: &mut IntroState, way: FadeWay) -> Fading {
    if state.fade.frames() > 1 {
        state.fade = state.fade.fewer().unwrap_or(FadeFrames::ZERO);
        return Fading::Going;
    }
    let was = state.brightness;
    let (now, end) = match way {
        FadeWay::Brightens => (brightened(was), BrightnessLevel::FULL),
        FadeWay::Darkens => (darkened(was), BrightnessLevel::BLACK),
    };
    state.brightness = now;
    if now.level != end {
        state.fade = FadeFrames::SLOW_FADE_PACE;
        return Fading::Going;
    }
    state.fade = if way == FadeWay::Darkens && was.level == BrightnessLevel::BLACK {
        count_left_at_black(state.fade)
    } else {
        FadeFrames::ZERO
    };
    Fading::Done
}

/// `SlowFade` (C-darken-at-zero): darkening from level 0 leaves the brightness and the count one
/// less, in a byte: 0 from 1, 255 from 0.
fn count_left_at_black(count: FadeFrames) -> FadeFrames {
    count.fewer().unwrap_or(FadeFrames::SLOW_COUNT_BELOW_ZERO)
}

/// `SlowFade`: one level brighter, at most 15, and shown.
const fn brightened(brightness: Brightness) -> Brightness {
    Brightness {
        level: brightness.level.brighter(),
        blanking: Blanking::Shown,
    }
}

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

/// `PictureDarkens`, `SlowFade`: one level darker and shown; from level 1, black and blanked; at
/// level 0 as it was.
fn darkened(brightness: Brightness) -> Brightness {
    let darker = brightness.level.darker();
    if brightness.level == BrightnessLevel::BLACK {
        brightness
    } else if darker == BrightnessLevel::BLACK {
        Brightness::BLANKED
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
    if state.brightness != Brightness::BLANKED {
        return;
    }
    state.stage = IntroStage::Done;
    clear_scene(state);
    state.fade = FadeFrames::ZERO;
    report
        .events
        .push(IntroEvent::Equipment(Equipment::RETURNED));
}
