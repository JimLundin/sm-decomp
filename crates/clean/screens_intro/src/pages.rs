//! The pages (`PageFiveBegins`, `LetterIsTyped`, `LetterGlows`, `CaretBlinks`, `PageEnds`,
//! `PressTurnsThePage`): each page is typed letter by letter, each letter glowing as it appears,
//! every other letter clicking and the caret running ahead; at its end the page waits for a
//! press, or holds while the picture fades out.

use core::num::Wrapping;

use types::intro::{
    Caret, CaretScript, ClickTurn, ColourSet, EyesScript, FadingColour, FadingColours, Glow,
    IntroStage, IntroState, PageScript, ScriptPlace, TextPlace,
};
use types::{Pair, Px};

use crate::content::IntroContent;
use crate::dissolve::{DISSOLVE_FROM, at_full, shown};
use crate::report::{IntroEvent, Recoloured, Report};
use crate::rules::Rules;
use crate::script::{Letter, Shows, Signal, Step};

/// A column or a row of the text layer, px.
const CELL: u16 = 8;
/// `LetterGlows`: the frames each colour set holds before the next.
const GLOW_HOLD: u16 = 5;
/// A space: typed and glowing like any letter, but never clicking.
const SPACE: char = ' ';

/// `PageFiveBegins`: the stage turns toward the text and page 5 begins. Every colour is set at
/// its full level, its step its full level × 8, then the text's colours at 0, and the colours
/// shown are all taken; the dissolve's count starts from 127.
pub fn page_five_begins(state: &mut IntroState, content: &IntroContent, report: &mut Report) {
    state.stage = IntroStage::ToText;
    begin_page(state, PageScript::Five, content);
    state.colours = FadingColours(content.full.0.map(at_full));
    for &index in &content.text {
        if let Some(colour) = state.colours.colour_mut(index) {
            colour.levels = FadingColour::STILL_BLACK.levels;
        }
    }
    report.colours = Some(shown(&state.colours));
    state.dissolve = DISSOLVE_FROM;
}

/// `PressTurnsThePage`: the text is cleared, page 6 begins, Samus's eyes start to stare from
/// their script's start and the music is asked to stop.
pub fn press_turns_the_page(state: &mut IntroState, content: &IntroContent, report: &mut Report) {
    state.stage = IntroStage::Text;
    report.events.push(IntroEvent::TextCleared);
    begin_page(state, PageScript::Six, content);
    state.eyes = Some(ScriptPlace::start(EyesScript::Staring));
    report.events.push(IntroEvent::MusicStops);
}

/// A page begins: its script placed (`place_page`), and the caret at its home on its steady
/// script (`caret_at_home`).
const fn begin_page(state: &mut IntroState, page: PageScript, content: &IntroContent) {
    place_page(state, page);
    caret_at_home(state, content);
}

/// A page's script placed at its start, the click turn off.
pub const fn place_page(state: &mut IntroState, page: PageScript) {
    state.page = Some(ScriptPlace::start(page));
    state.click = ClickTurn::Off;
}

/// The caret at its home, on its steady script from its start.
pub const fn caret_at_home(state: &mut IntroState, content: &IntroContent) {
    state.caret = Some(Caret {
        place: content.caret_home,
        script: ScriptPlace::start(CaretScript::Steady),
    });
}

/// A signal a script hands the scene. `PageFiveBegins`: the baby's page 5, taken while the scene
/// shows. `PageEnds`: wait for a press (page 5's, or page 1's before the fight), or finish and
/// start the fade from the rhythm `d`. `CaretBlinks`: the caret blinks while the page waits.
pub const fn take_signal(state: &mut IntroState, signal: Signal, rules: Rules) {
    match signal {
        Signal::PageFive => {
            if matches!(state.stage, IntroStage::Scene) {
                state.stage = IntroStage::ToPageFive;
            }
        }
        Signal::WaitForPress => {
            state.stage = IntroStage::WaitingForPress;
            caret_blinks(state);
        }
        Signal::WaitForPressThenFight => {
            state.stage = IntroStage::PageOneWaits;
            caret_blinks(state);
        }
        Signal::CaretBlinks => caret_blinks(state),
        Signal::Finish => {
            state.stage = IntroStage::FadingOut;
            state.fade = rules.rhythm;
        }
    }
}

/// `CaretBlinks`: the caret's blinking script placed at its start, its place kept.
const fn caret_blinks(state: &mut IntroState) {
    if let Some(caret) = &mut state.caret {
        caret.script = ScriptPlace::start(CaretScript::Blinking);
    }
}

/// `LetterIsTyped`: a page's letter appears as its step begins. The click turn turns, and the
/// letter clicks if it is now on and the letter is not a space; it glows from set 0 if fewer than
/// 8 glow (so a letter typed while 8 glow keeps set 0); the caret moves to where the next letter
/// goes (`caret_ahead`).
pub fn letter_is_typed(
    state: &mut IntroState,
    letter: Letter,
    next: Option<&Step>,
    report: &mut Report,
) {
    state.click = state.click.turned();
    if state.click == ClickTurn::On && letter.glyph != SPACE {
        report.events.push(IntroEvent::Click);
    }
    state.glows.place(Glow {
        at: letter.at,
        next: ColourSet::FIRST,
        left: 1,
    });
    if let Some(caret) = &mut state.caret {
        caret.place = caret_ahead(letter.at, next);
    }
    report.typed = Some(letter);
}

/// `LetterIsTyped`: where the caret goes once a letter is typed. Ahead of the page's next step's
/// letter at `(c, r)`, `(8 × c, 8 × r − 8)`; if the next step is not a letter, at the start of
/// the line below the typed one, `(8, 8 × row + 8)`.
fn caret_ahead(typed: TextPlace, next: Option<&Step>) -> Pair<Px> {
    let cell = Wrapping(CELL);
    if let Some(Step::Show {
        shows: Shows::Text(letter),
        ..
    }) = next
    {
        Pair {
            x: cell * Wrapping(letter.at.column),
            y: cell * Wrapping(letter.at.row) - cell,
        }
    } else {
        Pair {
            x: cell,
            y: cell * Wrapping(typed.row) + cell,
        }
    }
}

/// `LetterGlows`: each glowing letter counts down its frames; when its last runs out it is shown
/// in its next colour set, which holds 5 frames, until set 3, which it keeps as the glow ends.
pub fn letters_glow(state: &mut IntroState, report: &mut Report) {
    state.glows.update(|glow| {
        if glow.left > 1 {
            return Some(Glow {
                left: glow.left.wrapping_sub(1),
                ..glow
            });
        }
        report.recoloured.push(Recoloured {
            at: glow.at,
            set: glow.next,
        });
        glow.next.after().map(|next| Glow {
            next,
            left: GLOW_HOLD,
            ..glow
        })
    });
}
