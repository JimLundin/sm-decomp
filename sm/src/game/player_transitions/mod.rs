//! Owned post-movement player rules and finite compatibility interfaces.
#![forbid(unsafe_code)]

mod actions;
mod animation;
mod atmosphere;
mod commands;
pub(crate) mod content;
pub(crate) mod demo;
pub(crate) mod effects;
pub(crate) mod equipment;
mod input;
mod jump;
mod landing;
pub(crate) mod mother_brain;
mod normalization;
mod pose;
mod requests;
mod resolver;
pub(crate) mod state;
pub(crate) mod suit;
pub(crate) mod xray;

#[cfg(sm_verify_dll)]
pub(crate) use actions::momentum as verify_momentum_action;
pub(crate) use animation::advance_animation;
pub(crate) use atmosphere::draw_atmospheric;
pub(crate) use commands::{cancel_grapple, command};
// Existing finite verification cases exercise the actual private core entries.
#[cfg(sm_verify_dll)]
pub(crate) fn verify_interpret(host: &mut impl effects::Host) {
    animation::interpret(host);
}
pub(crate) use input::{handle_input, request_fallback_pose};
pub(crate) use jump::{initialize_jump, start_shinespark};
#[cfg(sm_verify_dll)]
pub(crate) use normalization::normalize as verify_normalize;
pub(crate) use pose::{
    align_bottom_with_previous_pose, clamp_previous_position, finish_pose_change, liquid_at_height,
    loaded_bottom, loaded_top, record_pose_history, refresh_pose, refresh_pose_rules,
};

pub(crate) fn finish_movement(host: &mut impl effects::Host) {
    advance_animation(host, state::AnimationStep::Gameplay);
    requests::hit_and_bomb(host);
    requests::from_blocks(host);
    resolver::resolve_transitions(host);
}

/// Continue the post-movement rules into palette work on the same owned Session.
pub(crate) fn finish_movement_and_palette(host: &mut impl effects::Host) {
    finish_movement(host);
    crate::game::samus_palette::advance(host);
}

#[cfg(test)]
mod representation_tests;

#[cfg(sm_verify_dll)]
pub(crate) fn verify_hit_and_bomb(host: &mut impl effects::Host) {
    requests::hit_and_bomb(host);
}
