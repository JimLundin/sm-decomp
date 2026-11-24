//! Her arrival at Ceres as a new game begins: the screen stays dark while the start is made
//! ready, she appears at the top of the station's shaft facing the screen, the picture comes up
//! from black over a second, and then she plays.
//!
//! The State the caller carries is `types::arrival::ArrivalState`; `step` runs one frame of it.

mod arrive;
pub mod catalogue;
mod fade;
mod report;
mod rules;
mod start;
#[cfg(test)]
mod vectors;

pub use report::{Arrival, Report};
pub use rules::Rules;
pub use start::{Preparation, SUIT_COLOURS, Start, SuitColours};

use types::arrival::{ArrivalStage, ArrivalState};

/// One frame of the arrival, as the game finishes it: the State moves on and the frame's
/// outputs are reported.
///
/// - Spec: `StartsInTheDark`. While the start is being made ready nothing of the arrival moves.
/// - Spec: `ArrivesAtTheTop`. On the frame it is ready she arrives; the fade starts next frame.
/// - Spec: `PictureBrightens`, then `PlayBegins`, on each frame of the fade in.
/// - Spec: `PlayBegins`. Once she plays, the arrival keeps its State as it is.
pub fn step(state: &mut ArrivalState, preparation: Preparation<'_>, rules: Rules) -> Report {
    let arrival = match state.stage {
        ArrivalStage::Preparing => match preparation {
            Preparation::Underway => None,
            Preparation::Ready(start) => Some(arrive::arrives_at_the_top(state, start, rules)),
        },
        ArrivalStage::FadingIn => {
            fade::fade_in_frame(state, rules);
            None
        }
        ArrivalStage::Playing => None,
    };
    Report {
        arrival,
        brightness: state.brightness,
        play_begins: state.stage == ArrivalStage::Playing,
    }
}
