//! The opening (`IntroBegins`, `OpeningUnfolds`): the screen stays dark while the first tune
//! loads, then slowly brightens on the opening's words; they hold while the first tune plays and
//! the second follows, then the picture slowly darkens and page 1 is made ready.

use types::intro::{Brightness, FadeFrames, Hold, IntroStage, IntroState, StageWait};

use crate::fade::{FadeWay, Fading, slow_fade};
use crate::music::{
    MusicRequest, REQUEST_DELAY, TrackReply, Tune, tune_asked_for, tune_asked_to_follow,
};
use crate::report::{Equipment, IntroEvent, Report};
use crate::scene::clear_scene;

/// `IntroBegins`: the scene cleared, the screen black and blanked, the wait 0 and the fade's
/// count 3; the music stopped and the first tune asked to load; the equipment lent for the
/// played scenes.
pub fn intro_begins(state: &mut IntroState, report: &mut Report) {
    state.stage = IntroStage::OpeningLoads;
    clear_scene(state);
    state.wait = StageWait::ZERO;
    state.brightness = Brightness::BLANKED;
    state.fade = FadeFrames::AS_THE_INTRO_BEGINS;
    report.music.extend(tune_asked_for(Tune::OpeningFirst));
    report
        .events
        .push(IntroEvent::EquipmentSet(Equipment::LENT));
}

/// `OpeningUnfolds`: once no track waits, the first tune has loaded and the words start to
/// brighten.
pub const fn opening_loads(state: &mut IntroState, track: TrackReply) {
    if matches!(track, TrackReply::NoneWaits) {
        state.stage = IntroStage::OpeningBrightens;
        state.fade = FadeFrames::SLOW_FADE_PACE;
    }
}

/// `OpeningUnfolds`: once the words are at full, they hold before the first tune plays.
pub fn opening_brightens(state: &mut IntroState) {
    if slow_fade(state, FadeWay::Brightens) == Fading::Done {
        state.stage = IntroStage::OpeningPause;
        state.wait = StageWait::BEFORE_FIRST_TUNE;
    }
}

/// `OpeningUnfolds`: once the pause runs out, the loaded tune is asked to play, at the usual
/// delay, and the words hold while it plays.
pub fn opening_pause(state: &mut IntroState, report: &mut Report) {
    if holds(state) == Hold::RanOut {
        state.stage = IntroStage::FirstTune;
        state.wait = StageWait::FIRST_TUNE;
        report.music.push(MusicRequest::Play {
            after: REQUEST_DELAY,
        });
    }
}

/// `OpeningUnfolds`: once the first tune's hold runs out, the second tune is asked for.
pub fn first_tune(state: &mut IntroState, report: &mut Report) {
    if holds(state) == Hold::RanOut {
        state.stage = IntroStage::SecondTuneLoads;
        report
            .music
            .extend(tune_asked_to_follow(Tune::OpeningSecond));
    }
}

/// `OpeningUnfolds`: once no track waits, the second tune has loaded and the words hold while it
/// plays.
pub const fn second_tune_loads(state: &mut IntroState, track: TrackReply) {
    if matches!(track, TrackReply::NoneWaits) {
        state.stage = IntroStage::SecondTune;
        state.wait = StageWait::SECOND_TUNE;
    }
}

/// `OpeningUnfolds`: once the second tune's hold runs out, the words start to darken.
pub const fn second_tune(state: &mut IntroState) {
    if matches!(holds(state), Hold::RanOut) {
        state.stage = IntroStage::OpeningDarkens;
        state.fade = FadeFrames::SLOW_FADE_PACE;
    }
}

/// `OpeningUnfolds`: once the picture is dark, page 1 is made ready.
pub fn opening_darkens(state: &mut IntroState) {
    if slow_fade(state, FadeWay::Darkens) == Fading::Done {
        state.stage = IntroStage::PageOneMadeReady;
    }
}

/// `OpeningUnfolds`: one frame of the stage's hold, its wait counted down.
const fn holds(state: &mut IntroState) -> Hold {
    let (wait, hold) = state.wait.counted_down();
    state.wait = wait;
    hold
}
