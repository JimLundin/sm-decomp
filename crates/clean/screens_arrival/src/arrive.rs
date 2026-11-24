//! Spec: `ArrivesAtTheTop`. Her arrival at the top of the station's shaft.

use core::num::Wrapping;

use types::arrival::{ArrivalStage, ArrivalState};
use types::{Colour, Facing, Pair, Palette, Px};

use crate::report::Arrival;
use crate::rules::Rules;
use crate::start::Start;

/// Her height as she arrives: the top of the room.
const ARRIVAL_HEIGHT: Px = Wrapping(0);
/// The first of her colours on the screen, 192; her suit's sixteen run to 207.
const HER_COLOURS_FROM: usize = 192;
/// The colour the arrival turns black.
const TURNED_BLACK: usize = 223;

/// Spec: `ArrivesAtTheTop`. She appears at the start's x at the top of the room, facing the
/// screen, and the fade in starts its count; the brightness is left as it is, so the fade in
/// begins on the next frame.
pub fn arrives_at_the_top(state: &mut ArrivalState, start: &Start, rules: Rules) -> Arrival {
    state.stage = ArrivalStage::FadingIn;
    state.count = rules.rhythm;
    Arrival {
        place: Pair {
            x: start.x,
            y: ARRIVAL_HEIGHT,
        },
        facing: Facing::Screen,
        colours: screen_colours(start),
    }
}

/// Spec: `ArrivesAtTheTop`. The screen's colours at the arrival: every colour takes its goal,
/// then her colours take her suit's and colour 223 turns black.
///
/// The goal itself is left alone, so her colours show her suit's even where the goal holds
/// others, and colour 223 is black while its goal keeps the room's colour: a later fade heads
/// for the goal, not for what the arrival put up.
fn screen_colours(start: &Start) -> Palette {
    let mut colours = start.goal.clone();
    for (colour, &suit) in colours
        .0
        .iter_mut()
        .skip(HER_COLOURS_FROM)
        .zip(&start.suit.0)
    {
        *colour = suit;
    }
    if let Some(colour) = colours.0.get_mut(TURNED_BLACK) {
        *colour = Colour::BLACK;
    }
    colours
}
