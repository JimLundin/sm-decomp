//! Page 1 (`PageOneIsMadeReady`, `PageOneBegins`): made ready in the dark, it starts to type
//! once the pages' tune has loaded, while the picture slowly brightens beside Samus's blinking
//! eyes. Its end (`PageEnds`) is a page's signal.

use types::intro::{EyesScript, FadeFrames, IntroStage, IntroState, PageScript, ScriptPlace};

use crate::baby::SETTLED_HEIGHT;
use crate::content::IntroContent;
use crate::dissolve::DISSOLVE_FROM;
use crate::fade::{FadeWay, Fading, slow_fade};
use crate::music::{TrackReply, Tune, tune_asked_to_follow};
use crate::pages::{caret_at_home, place_page};
use crate::report::{IntroEvent, Report};

/// `PageOneIsMadeReady`: in the dark, the text cleared and no page; the dissolve's count 127,
/// which the next part's dissolve starts from; the picture standing at 8; Samus's eyes blinking
/// from their script's start; the caret at its home; the music stopped and the pages' tune asked
/// to load and to play 14 frames after.
pub fn page_one_is_made_ready(state: &mut IntroState, content: &IntroContent, report: &mut Report) {
    state.stage = IntroStage::PageOneLoads;
    report.events.push(IntroEvent::TextCleared);
    state.page = None;
    state.dissolve = DISSOLVE_FROM;
    state.height = SETTLED_HEIGHT;
    state.eyes = Some(ScriptPlace::start(EyesScript::Blinking));
    caret_at_home(state, content);
    report.music.extend(tune_asked_to_follow(Tune::Pages));
}

/// `PageOneBegins`: once no track waits, the pages' tune has loaded: page 1 is placed and the
/// picture starts to brighten. While a track waits the stage holds and the scripts run on.
pub const fn page_one_loads(state: &mut IntroState, track: TrackReply) {
    if matches!(track, TrackReply::NoneWaits) {
        state.stage = IntroStage::PageOneBrightens;
        state.fade = FadeFrames::SLOW_FADE_PACE;
        place_page(state, PageScript::One);
    }
}

/// `PageOneBegins`: once the picture is at full, the text goes on typing.
pub fn page_one_brightens(state: &mut IntroState) {
    if slow_fade(state, FadeWay::Brightens) == Fading::Done {
        state.stage = IntroStage::Text;
    }
}
