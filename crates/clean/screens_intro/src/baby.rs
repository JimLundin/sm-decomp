//! The baby (`BabyRises`, `BabyLeaves`): it rises into view as the picture settles, and is gone
//! once the picture has dissolved back to the text.

use core::num::Wrapping;

use types::intro::{IntroStage, IntroState};

use crate::dissolve::on_beat;

/// `BabyRises`: the picture's height it stops at, px.
const SETTLED_HEIGHT: i16 = 8;
/// The baby's x on the screen, px.
pub const BABY_X: u16 = 112;

/// `BabyLeaves`: once the dissolve back to the text reaches its last count, 0, the baby is
/// removed.
pub fn baby_leaves(state: &mut IntroState) {
    if state.stage == IntroStage::ToText && state.dissolve == 0 {
        state.baby = None;
    }
}

/// `BabyRises`: as the picture dissolves to the scene, on each frame that leaves the dissolve's
/// count on the beat the picture moves down a pixel and the baby rises a pixel, until the
/// picture's height reaches 8.
pub fn baby_rises(state: &mut IntroState) {
    let rising = state.stage == IntroStage::ToScene
        && on_beat(state.dissolve)
        && state.height < SETTLED_HEIGHT;
    if !rising {
        return;
    }
    if let Some(baby) = &mut state.baby {
        state.height = state.height.wrapping_add(1);
        baby.y -= Wrapping(1);
    }
}
