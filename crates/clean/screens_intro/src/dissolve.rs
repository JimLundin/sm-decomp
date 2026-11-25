//! The dissolves (`PicturesDissolve`, `DissolveEnds`): one set of colours dims to black while
//! another brightens from black, in 32 even steps, one every 4 frames.

use types::intro::{FadingColour, FadingColours, IntroStage, IntroState};
use types::{Colour, Palette};

use crate::content::{Dissolve, IntroContent};
use crate::report::{IntroEvent, Report};

/// `PicturesDissolve`: the dissolve steps on every fourth count.
const BEAT: u16 = 4;
/// A channel's level is in 256ths of a whole level.
const LEVEL: u16 = 256;
/// `PicturesDissolve`: a brightening channel's level wraps at 8,192, 32 whole levels.
const BRIGHTEN_SPAN: u16 = 8192;
/// The colour shown takes a channel's whole levels modulo 32.
const SHOWN_LEVELS: u16 = 32;
/// `PageFiveBegins`: a colour's step is its full level × 8, a 32nd of its full level in 256ths.
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
/// goes first; to the text, the brightening set. It tells only for a colour in both sets.
const fn order(dissolve: Dissolve) -> [Move; 2] {
    match dissolve {
        Dissolve::ToScene => [Move::Dim, Move::Brighten],
        Dissolve::ToText => [Move::Brighten, Move::Dim],
    }
}

/// `PicturesDissolve`, `BabyRises`: whether a dissolve's count is on the beat, a multiple of 4.
#[must_use]
pub const fn on_beat(count: u16) -> bool {
    count.wrapping_rem(BEAT) == 0
}

/// One frame of a dissolve (`PicturesDissolve`): the count goes down by 1, wrapping past 0 to
/// 65,535. On the beat, each set moves and the colours shown are all taken; when the count was 0
/// the dissolve ends (`DissolveEnds`).
pub fn dissolve_frame(
    state: &mut IntroState,
    dissolve: Dissolve,
    content: &IntroContent,
    report: &mut Report,
) {
    let count = state.dissolve;
    state.dissolve = count.wrapping_sub(1);
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

/// `DissolveEnds`: to the scene, the text is cleared; to the text, the caret's colours are shown,
/// their levels left. The stage moves on; the count has wrapped to 65,535.
fn dissolve_ends(
    state: &mut IntroState,
    dissolve: Dissolve,
    content: &IntroContent,
    shown: &mut Palette,
    report: &mut Report,
) {
    match dissolve {
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
#[must_use]
pub fn shown(colours: &FadingColours) -> Palette {
    Palette(colours.0.map(|colour| {
        Colour::from_channels(
            colour
                .levels
                .map(|level| level.wrapping_div(LEVEL).wrapping_rem(SHOWN_LEVELS)),
        )
    }))
}

/// `PageFiveBegins`: a colour at its full level, its step its full level × 8.
#[must_use]
pub fn at_full(full: Colour) -> FadingColour {
    let channels = full.channels();
    FadingColour {
        levels: channels.map(|channel| channel.wrapping_mul(LEVEL)),
        steps: channels.map(|channel| channel.wrapping_mul(STEP_PER_LEVEL)),
    }
}
