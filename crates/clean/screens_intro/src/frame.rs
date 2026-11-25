//! One frame of the intro, in the order its parts read each other's results: first the stage's
//! step (the opening's, page 1's, the flashback's, a dissolve's, a page's turn or the fade's),
//! then, after room play, the hide of Samus and her shots and the encounter's signal, then the
//! baby, the caret's script, the page's script, the eyes'
//! script and the glows. So a script placed by the stage's step runs its first step in the same
//! frame, while one placed by a script's signal (the caret's blinking script) runs from the next;
//! the baby's signal for page 5 takes effect at the next frame's stage step; and the encounter's
//! return makes the stage `PageTwoPending` in the frame it comes, page 2 beginning at the next
//! frame's stage step and its dissolve counting down from the frame after.
//!
//! A finished frame comes in two parts, with the flashback's room play between them, each run
//! exactly once and in this order:
//! 1. `begin_frame` runs the stage's step and hands back a `FrameAwaitingReply`. From it the
//!    caller reads the flashback's requests (`FrameAwaitingReply::requests`): the encounter's
//!    start (`BattleBegins`), applied before room play, and the hiding of Samus and her shots
//!    (`BattleDissolves`), the visibility edge, applied after room play and before drawing.
//! 2. The caller plays the room. Samus and her shots are still shown in the intro's state, so
//!    they take their last room step before the hide takes effect on the picture.
//! 3. `FrameAwaitingReply::finish` consumes the encounter's reply and completes the frame: the
//!    hide takes effect, the reply is read where `PageTwoBegins` reads it, and the scripts run.
//!
//! `step` is the same two parts run back to back, for a caller that already has the reply.

use core::num::Wrapping;

use types::intro::{Baby, Caret, IntroStage, IntroState, ScriptPlace};
use types::{Pair, Px};

use crate::baby::{BABY_X, baby_leaves, baby_rises};
use crate::content::{Dissolve, IntroContent};
use crate::dissolve::dissolve_frame;
use crate::fade::{intro_ends, picture_darkens};
use crate::flashback::{
    FightReply, battle_begins, fight_signals, hide_takes_effect, page_two_begins,
};
use crate::music::TrackReply;
use crate::opening::{
    first_tune, intro_begins, opening_brightens, opening_darkens, opening_loads, opening_pause,
    second_tune, second_tune_loads,
};
use crate::page_one::{page_one_brightens, page_one_is_made_ready, page_one_loads};
use crate::pages::{
    letter_is_typed, letters_glow, page_five_begins, press_turns_the_page, take_signal,
};
use crate::report::{FlashbackRequest, IntroEvent, Report, ShownPicture};
use crate::rules::Rules;
use crate::script::{PictureName, Ran, Shows, Step, run, showing};

/// What the intro reads each frame besides its content and rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameInputs {
    /// Whether any button was pressed anew this frame (the movie's pad).
    pub pressed: bool,
    /// Audio's reply: whether a tune asked for still waits to load and start.
    pub track: TrackReply,
    /// The encounter's signal, received after room play: whether the fight is ready to return
    /// to the text.
    pub fight: FightReply,
}

/// One whole frame of the intro, for a caller that already has the encounter's reply: the same
/// two parts as `begin_frame` and `FrameAwaitingReply::finish`, run back to back.
///
/// A press while page 2 waits (`PageTwoWaits`) starts the baby's discovery, a part not yet
/// specced (`PageTwoEnds`): the caller refuses it, and the model reads it as no press.
pub fn step(
    state: &mut IntroState,
    inputs: FrameInputs,
    content: &IntroContent,
    rules: Rules,
) -> Report {
    begin_frame(state, inputs.pressed, inputs.track, content, rules).finish(inputs.fight)
}

/// Expose the scene requests before room play.
pub fn begin_frame<'frame>(
    state: &'frame mut IntroState,
    pressed: bool,
    track: TrackReply,
    content: &'frame IntroContent,
    rules: Rules,
) -> FrameAwaitingReply<'frame> {
    let mut report = Report::empty();
    stage_step(state, pressed, track, content, rules, &mut report);
    FrameAwaitingReply {
        state,
        content,
        rules,
        report,
    }
}

/// A frame whose stage has stepped, waiting for the encounter's reply from room play before its
/// scripts run. It holds the intro's state until it is finished.
#[must_use = "a frame ends only when it is finished with the encounter's reply"]
#[derive(Debug)]
pub struct FrameAwaitingReply<'frame> {
    /// The intro's state, its stage stepped.
    state: &'frame mut IntroState,
    /// The scene's content.
    content: &'frame IntroContent,
    /// The rules.
    rules: Rules,
    /// What the frame has reported so far.
    report: Report,
}

impl FrameAwaitingReply<'_> {
    /// The intro's state after the stage's step, as room play sees it: a hide asked for this
    /// frame has not yet taken effect.
    #[must_use]
    pub const fn state(&self) -> &IntroState {
        self.state
    }

    /// The flashback's requests the stage's step made, in order: the encounter's start, for
    /// production to apply before room play, and the hiding of Samus and her shots, to apply
    /// after room play and before drawing.
    pub fn requests(&self) -> impl Iterator<Item = FlashbackRequest> {
        self.report.events.iter().filter_map(|event| {
            if let IntroEvent::Flashback(request) = event {
                Some(*request)
            } else {
                None
            }
        })
    }

    /// The second part of the frame, once room play has run: the hide asked for this frame
    /// takes effect, the encounter's reply is read (`PageTwoBegins`), then the baby, the caret's
    /// script, the page's script, the eyes' script and the glows. The frame's whole report.
    #[must_use]
    pub fn finish(self, fight: FightReply) -> Report {
        let Self {
            state,
            content,
            rules,
            mut report,
        } = self;
        hide_takes_effect(state, &report);
        fight_signals(state, fight, &mut report);
        baby_frame(state, content, rules);
        caret_frame(state, content, rules);
        page_frame(state, content, rules, &mut report);
        eyes_frame(state, content, rules);
        letters_glow(state, &mut report);
        show_pictures(state, content, &mut report);
        report
    }
}

/// The stage's step: the opening's, page 1's, the flashback's, a dissolve's frame, a page's turn,
/// or the fade's. A waiting stage moves on here only on a new press (`PageOneWaits`,
/// `WaitingForPress`); the fight waits for the encounter's signal, taken after room play; the
/// stages left out hold while the scripts run.
fn stage_step(
    state: &mut IntroState,
    pressed: bool,
    track: TrackReply,
    content: &IntroContent,
    rules: Rules,
    report: &mut Report,
) {
    match state.stage {
        IntroStage::Begins => intro_begins(state, report),
        IntroStage::OpeningLoads => opening_loads(state, track),
        IntroStage::OpeningBrightens => opening_brightens(state),
        IntroStage::OpeningPause => opening_pause(state, report),
        IntroStage::FirstTune => first_tune(state, report),
        IntroStage::SecondTuneLoads => second_tune_loads(state, track),
        IntroStage::SecondTune => second_tune(state),
        IntroStage::OpeningDarkens => opening_darkens(state),
        IntroStage::PageOneMadeReady => page_one_is_made_ready(state, content, report),
        IntroStage::PageOneLoads => page_one_loads(state, track),
        IntroStage::PageOneBrightens => page_one_brightens(state),
        IntroStage::PageOneWaits => {
            if pressed {
                battle_begins(state, content, report);
            }
        }
        IntroStage::IntoFight => dissolve_frame(state, Dissolve::IntoFight, content, report),
        IntroStage::PageTwoPending => page_two_begins(state, content, report),
        IntroStage::OutOfFight => dissolve_frame(state, Dissolve::OutOfFight, content, report),
        IntroStage::ToScene => dissolve_frame(state, Dissolve::ToScene, content, report),
        IntroStage::ToText => dissolve_frame(state, Dissolve::ToText, content, report),
        IntroStage::ToPageFive => page_five_begins(state, content, report),
        IntroStage::WaitingForPress => {
            if pressed {
                press_turns_the_page(state, content, report);
            }
        }
        IntroStage::FadingOut => {
            picture_darkens(state, rules);
            intro_ends(state, report);
        }
        IntroStage::Fight
        | IntroStage::PageTwoWaits
        | IntroStage::Scene
        | IntroStage::Text
        | IntroStage::Done => {}
    }
}

/// The signals a script handed the scene, taken in order.
fn take_signals(state: &mut IntroState, ran: &Ran, rules: Rules) {
    for &signal in &ran.signals {
        take_signal(state, signal, rules);
    }
}

/// The baby: it leaves or rises, then its script runs.
fn baby_frame(state: &mut IntroState, content: &IntroContent, rules: Rules) {
    baby_leaves(state);
    baby_rises(state);
    let Some(baby) = state.baby else {
        return;
    };
    let ran = run(baby.progress, &content.baby);
    state.baby = ran.progress.map(|progress| Baby { progress, ..baby });
    take_signals(state, &ran, rules);
}

/// The caret's script runs (`CaretBlinks`).
fn caret_frame(state: &mut IntroState, content: &IntroContent, rules: Rules) {
    let Some(caret) = state.caret else {
        return;
    };
    let ran = run(caret.script.progress, &content.caret[caret.script.script]);
    state.caret = ran.progress.map(|progress| Caret {
        script: ScriptPlace {
            progress,
            ..caret.script
        },
        ..caret
    });
    take_signals(state, &ran, rules);
}

/// The page's script runs: its signals, and the letter its new step types (`LetterIsTyped`).
fn page_frame(state: &mut IntroState, content: &IntroContent, rules: Rules, report: &mut Report) {
    let Some(page) = state.page else {
        return;
    };
    let script = &content.pages[page.script];
    let ran = run(page.progress, script);
    state.page = ran
        .progress
        .map(|progress| ScriptPlace { progress, ..page });
    take_signals(state, &ran, rules);
    let Some(began) = ran.began else {
        return;
    };
    if let Some(Step::Show {
        shows: Shows::Text(letter),
        ..
    }) = script.get(began)
    {
        letter_is_typed(state, *letter, script.get(began.wrapping_add(1)), report);
    }
}

/// Samus's eyes' script runs.
fn eyes_frame(state: &mut IntroState, content: &IntroContent, rules: Rules) {
    let Some(eyes) = state.eyes else {
        return;
    };
    let ran = run(eyes.progress, &content.eyes[eyes.script]);
    state.eyes = ran
        .progress
        .map(|progress| ScriptPlace { progress, ..eyes });
    take_signals(state, &ran, rules);
}

/// The pictures the eyes', the baby's and the caret's scripts show as the frame ends; the caret
/// shows whenever its script's step shows its picture (`CaretBlinks`).
fn show_pictures(state: &IntroState, content: &IntroContent, report: &mut Report) {
    report.eyes = state
        .eyes
        .and_then(|eyes| image(showing(eyes.progress, &content.eyes[eyes.script])))
        .cloned();
    report.baby = state.baby.and_then(|baby| {
        let place = Pair {
            x: Wrapping(BABY_X),
            y: baby.y,
        };
        placed(showing(baby.progress, &content.baby), place)
    });
    report.caret = state.caret.and_then(|caret| {
        let script = &content.caret[caret.script.script];
        placed(showing(caret.script.progress, script), caret.place)
    });
}

/// The picture a step shows, if it shows one.
const fn image(shows: &Shows) -> Option<&PictureName> {
    match shows {
        Shows::Image(picture) => Some(picture),
        Shows::Nothing | Shows::Text(_) => None,
    }
}

/// The picture a step shows at a place, if it shows one.
fn placed(shows: &Shows, place: Pair<Px>) -> Option<ShownPicture> {
    image(shows).map(|picture| ShownPicture {
        picture: picture.clone(),
        place,
    })
}
