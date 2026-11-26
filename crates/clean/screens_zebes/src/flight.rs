//! One F4 step: the scheduler's cut, the stage's rule, and the picture published.

use types::zebes::{SubtitleChoice, ZebesScene, ZebesStage, ZebesState};

use crate::report::{self, FrameReport, Requests};
use crate::rules::Rules;
use crate::{approach, caption, handoff, hold, reveal, slide};

/// What the scheduler (T1) permits on this cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameAdvance {
    /// One finished frame: the scene advances.
    Finished,
    /// A borrowed lag cut: the scene holds its last published picture and sends nothing.
    BorrowedLag,
}

/// One F4 step: the cut's rule, then the picture published with the cut's requests.
///
/// On a finished frame the rule of the stage the frame begins in runs; on a borrowed lag cut
/// nothing changes. Button presses never reach the scene.
#[must_use]
pub fn step(
    state: &mut ZebesState,
    scene: &ZebesScene,
    rules: Rules,
    subtitles: SubtitleChoice,
    advance: FrameAdvance,
) -> FrameReport {
    let requests = match advance {
        FrameAdvance::Finished => finish_frame(state, scene, rules, subtitles),
        FrameAdvance::BorrowedLag => Requests::NONE,
    };
    FrameReport {
        picture: report::publish(state, scene),
        requests,
    }
}

/// The rule of the stage the frame begins in.
///
/// The setup frame begins the reveal. Each reveal frame then carries how long the light still
/// holds its level before rising, and the mosaic clock.
fn finish_frame(
    state: &mut ZebesState,
    scene: &ZebesScene,
    rules: Rules,
    subtitles: SubtitleChoice,
) -> Requests {
    match state.stage {
        ZebesStage::Pending => return reveal::begin_reveal(state, scene),
        ZebesStage::Revealing { fade, beat } => reveal::reveal(state, scene, fade, beat),
        ZebesStage::Caption { age } => return caption::name_planet(state, scene, subtitles, age),
        ZebesStage::Rightward => approach::rightward(state),
        ZebesStage::Banking => approach::banking(state),
        ZebesStage::Leftward => approach::leftward(state, rules),
        ZebesStage::Still { hold } => state.stage = hold::hold(hold),
        ZebesStage::Sliding => slide::slide(state, scene),
        ZebesStage::Leaving => return handoff::leave(state),
        ZebesStage::Complete => {}
    }
    Requests::NONE
}
