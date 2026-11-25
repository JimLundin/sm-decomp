//! One frame of the intro, in the order its parts read each other's results: first the stage's
//! step, then the baby, the caret's script, the page's script, the eyes' script and the glows.
//! So a script placed by the stage's step runs its first step in the same frame, while one placed
//! by a script's signal (the caret's blinking script) runs from the next; and the baby's signal
//! for page 5 takes effect at the next frame's stage step.

use core::num::Wrapping;

use types::intro::{Baby, Caret, IntroStage, IntroState, ScriptPlace};
use types::{Pair, Px};

use crate::baby::{BABY_X, baby_leaves, baby_rises};
use crate::content::{Dissolve, IntroContent};
use crate::dissolve::dissolve_frame;
use crate::fade::{intro_ends, picture_darkens};
use crate::pages::{
    letter_is_typed, letters_glow, page_five_begins, press_turns_the_page, take_signal,
};
use crate::report::{Report, ShownPicture};
use crate::rules::Rules;
use crate::script::{PictureName, Ran, Shows, Step, run, showing};

/// One frame of the intro: `pressed` is whether any button was pressed anew this frame.
pub fn step(state: &mut IntroState, pressed: bool, content: &IntroContent, rules: Rules) -> Report {
    let mut report = Report::empty();
    stage_step(state, pressed, content, rules, &mut report);
    baby_frame(state, content, rules);
    caret_frame(state, content, rules);
    page_frame(state, content, rules, &mut report);
    eyes_frame(state, content, rules);
    letters_glow(state, &mut report);
    show_pictures(state, content, &mut report);
    report
}

/// The stage's step: a dissolve's frame, a page's turn, or the fade's.
fn stage_step(
    state: &mut IntroState,
    pressed: bool,
    content: &IntroContent,
    rules: Rules,
    report: &mut Report,
) {
    match state.stage {
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
        IntroStage::Scene | IntroStage::Text | IntroStage::Done => {}
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
