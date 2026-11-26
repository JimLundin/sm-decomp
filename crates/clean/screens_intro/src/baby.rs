//! The baby (`BabyCrossesThePicture`, `BabyRises`, `BabyLeaves`): in the delivery it is carried
//! across as the picture pans; in the examination it rises into view as the picture settles;
//! either way it is gone once the picture has dissolved back to the text.

use core::num::Wrapping;

use types::intro::{IntroStage, IntroState};

use crate::dissolve::on_beat;
use crate::scene::SETTLED_PICTURE_HEIGHT;

/// `BabyLeaves`: once the dissolve back to the text from the delivery or the examination
/// reaches its last count, 0, the baby is removed.
pub const fn baby_leaves(state: &mut IntroState) {
    let leaving = matches!(state.stage, IntroStage::ToText | IntroStage::OutOfDelivery);
    if leaving && state.dissolve == 0 {
        state.baby = None;
    }
}

/// `BabyCrossesThePicture`: as the delivery dissolves in, on each frame that leaves the
/// dissolve's count on the beat the picture pans a pixel less and the baby moves a pixel right,
/// until the pan is at rest.
pub fn baby_crosses_the_picture(state: &mut IntroState) {
    if state.stage != IntroStage::IntoDelivery || !on_beat(state.dissolve) {
        return;
    }
    let (Some(baby), Some(narrower)) = (&mut state.baby, state.pan.narrower()) else {
        return;
    };
    state.pan = narrower;
    baby.place.x += Wrapping(1);
}

/// `BabyRises`: as the picture dissolves to the scene, on each frame that leaves the dissolve's
/// count on the beat the picture moves down a pixel and the baby rises a pixel, until the
/// picture's height reaches 8.
pub fn baby_rises(state: &mut IntroState) {
    let rising = state.stage == IntroStage::ToScene
        && on_beat(state.dissolve)
        && state.height < SETTLED_PICTURE_HEIGHT;
    if !rising {
        return;
    }
    if let Some(baby) = &mut state.baby {
        state.height = state.height.wrapping_add(1);
        baby.place.y -= Wrapping(1);
    }
}
