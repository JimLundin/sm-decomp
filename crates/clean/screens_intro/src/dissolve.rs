//! The dissolves (`PicturesDissolve`, `DissolveEnds`, `BattleDissolves`): one set of colours dims
//! to black while another brightens from black, in 32 even steps, one every 4 frames. The
//! battle's two directions and the baby's two share this arithmetic; each direction supplies
//! only its colour sets, the order it moves them in and what happens as it ends.

use types::intro::{FadingColour, FadingColours, IntroStage, IntroState};
use types::{Colour, Palette};

use crate::content::{Dissolve, IntroContent};
use crate::report::{FlashbackRequest, IntroEvent, PictureLayout, Report};

/// `PageFiveBegins`, `PageOneIsMadeReady`, `BattleBegins`, `PageTwoBegins`: a dissolve runs 128
/// frames, its count from 127.
pub const DISSOLVE_FROM: u16 = 127;
/// `BattleDissolves`: the count the dissolve back to the text steps from as Samus and her shots
/// are hidden, the frame before it ends.
const FIGHT_HIDDEN_FROM: u16 = 1;
/// `PicturesDissolve`: the dissolve steps on every fourth count.
const BEAT: u16 = 4;
/// A channel's level is in 256ths of a whole level.
const LEVEL: u16 = 256;
/// `PicturesDissolve`: a brightening channel's level wraps at 8,192, 32 whole levels.
const BRIGHTEN_SPAN: u16 = 8192;
/// The colour shown takes a channel's whole levels modulo 32.
const SHOWN_LEVELS: u16 = 32;
/// A colour's step is its full level × 8, a 32nd of its full level in 256ths.
const STEP_PER_LEVEL: u16 = 8;

/// How a dissolve moves a colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Move {
    /// Each channel's level less its step, wrapping at 16 bits.
    Dim,
    /// Each channel's level plus its step, wrapping at 8,192.
    Brighten,
}

/// `PicturesDissolve`: the order each dissolve moves its sets in. To the scene, the dimming set
/// goes first; to the text, the brightening set. It tells only for a colour in both sets. The
/// spec gives no order for the battle's dissolves; they take the order of the dissolve that
/// leaves the text (into the fight) or returns to it (out of the fight).
const fn order(dissolve: Dissolve) -> [Move; 2] {
    match dissolve {
        Dissolve::ToScene | Dissolve::IntoFight => [Move::Dim, Move::Brighten],
        Dissolve::ToText | Dissolve::OutOfFight => [Move::Brighten, Move::Dim],
    }
}

/// A dissolve begins (`PageFiveBegins`, `BattleBegins`, `PageTwoBegins`): every colour at its
/// full level, its step its full level × 8; then the colours of `black` at 0; the colours shown
/// are all taken, and the count starts from 127.
pub fn dissolve_begins(
    state: &mut IntroState,
    content: &IntroContent,
    black: &[u8],
    report: &mut Report,
) {
    state.colours = FadingColours(content.full.0.map(at_full));
    for &index in black {
        if let Some(colour) = state.colours.colour_mut(index) {
            colour.levels = FadingColour::STILL_BLACK.levels;
        }
    }
    report.colours = Some(shown(&state.colours));
    state.dissolve = DISSOLVE_FROM;
}

/// `PicturesDissolve`, `BabyRises`: whether a dissolve's count is on the beat, a multiple of 4.
#[must_use]
pub const fn on_beat(count: u16) -> bool {
    count.wrapping_rem(BEAT) == 0
}

/// One frame of a dissolve (`PicturesDissolve`, `BattleDissolves`): the count goes down by 1,
/// wrapping past 0 to 65,535. On the beat, each set moves and the colours shown are all taken;
/// when the count was 0 the step is the last, its colours taken in the frame the dissolve ends
/// (`DissolveEnds`, `BattleDissolves`). Back out of the fight, the step from count 1 asks for
/// Samus and her shots to be hidden; the hide takes effect after room play.
pub fn dissolve_frame(
    state: &mut IntroState,
    dissolve: Dissolve,
    content: &IntroContent,
    report: &mut Report,
) {
    let count = state.dissolve;
    state.dissolve = count.wrapping_sub(1);
    if dissolve == Dissolve::OutOfFight && count == FIGHT_HIDDEN_FROM {
        report.events.push(IntroEvent::Flashback(
            FlashbackRequest::HideFightSamusAndShots,
        ));
    }
    if !on_beat(count) {
        return;
    }
    let sets = &content.sets[dissolve];
    for way in order(dissolve) {
        let members = match way {
            Move::Dim => &sets.dims,
            Move::Brighten => &sets.brightens,
        };
        for &index in members {
            if let Some(colour) = state.colours.colour_mut(index) {
                *colour = moved(*colour, way);
            }
        }
    }
    let mut shown = shown(&state.colours);
    if count == 0 {
        dissolve_ends(state, dissolve, content, &mut shown, report);
    }
    report.colours = Some(shown);
}

/// `PicturesDissolve`: a colour dimmed or brightened by its steps.
fn moved(colour: FadingColour, way: Move) -> FadingColour {
    let mut levels = colour.levels;
    for (level, step) in levels.iter_mut().zip(colour.steps) {
        *level = match way {
            Move::Dim => level.wrapping_sub(step),
            Move::Brighten => level.wrapping_add(step).wrapping_rem(BRIGHTEN_SPAN),
        };
    }
    FadingColour { levels, ..colour }
}

/// A dissolve ends, the frame's last step taken; the stage moves on and the count has wrapped
/// to 65,535. `DissolveEnds`: to the scene, the text is cleared; to the text, the caret's
/// colours are shown, their levels left. `BattleDissolves`: into the fight, the text is
/// cleared, the fight's picture is shown plain and its flashing lights begin; out of it, the
/// text's picture is shown plain and page 2 types on.
fn dissolve_ends(
    state: &mut IntroState,
    dissolve: Dissolve,
    content: &IntroContent,
    shown: &mut Palette,
    report: &mut Report,
) {
    match dissolve {
        Dissolve::IntoFight => {
            state.stage = IntroStage::Fight;
            report.events.extend([
                IntroEvent::TextCleared,
                IntroEvent::Layout(PictureLayout::PlainFight),
                IntroEvent::FlashingLightsBegin,
            ]);
        }
        Dissolve::OutOfFight => {
            state.stage = IntroStage::Text;
            report
                .events
                .push(IntroEvent::Layout(PictureLayout::PlainText));
        }
        Dissolve::ToScene => {
            state.stage = IntroStage::Scene;
            report.events.push(IntroEvent::TextCleared);
        }
        Dissolve::ToText => {
            state.stage = IntroStage::Text;
            for caret in &content.caret_colours {
                if let Some(colour) = shown.0.get_mut(usize::from(caret.index)) {
                    *colour = caret.colour;
                }
            }
        }
    }
}

/// The colours shown: each channel's whole levels, `level / 256` modulo 32.
fn shown(colours: &FadingColours) -> Palette {
    Palette(colours.0.map(|colour| {
        Colour::from_channels(
            colour
                .levels
                .map(|level| level.wrapping_div(LEVEL).wrapping_rem(SHOWN_LEVELS)),
        )
    }))
}

/// As a dissolve begins, a colour at its full level, its step its full level × 8.
fn at_full(full: Colour) -> FadingColour {
    let channels = full.channels();
    FadingColour {
        levels: channels.map(|channel| channel.wrapping_mul(LEVEL)),
        steps: channels.map(|channel| channel.wrapping_mul(STEP_PER_LEVEL)),
    }
}
