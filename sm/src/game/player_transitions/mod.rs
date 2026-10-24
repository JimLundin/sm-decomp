//! Owned post-movement player rules and finite compatibility interfaces.
#![forbid(unsafe_code)]

mod actions;
mod animation;
mod atmosphere;
pub(crate) mod content;
pub(crate) mod effects;
mod jump;
mod landing;
mod normalization;
mod pose;
mod requests;
mod resolver;
pub(crate) mod state;

#[cfg(sm_verify_dll)]
pub(crate) use actions::momentum as verify_momentum_action;
pub(crate) use animation::advance_animation;
// Existing finite verification cases exercise the actual private core entries.
#[cfg(sm_verify_dll)]
pub(crate) fn verify_interpret(host: &mut impl effects::Host) {
    animation::interpret(host);
}
pub(crate) use jump::{initialize_jump, start_shinespark};
#[cfg(sm_verify_dll)]
pub(crate) use normalization::normalize as verify_normalize;
pub(crate) use pose::{
    align_bottom_with_previous_pose, clamp_previous_position, finish_pose_change,
    record_pose_history, refresh_pose, refresh_pose_rules, reset_pose_animation,
};
pub(crate) use requests::request_pose;

pub(crate) fn finish_movement(host: &mut impl effects::Host) {
    advance_animation(host, state::AnimationStep::Gameplay);
    requests::hit_and_bomb(host);
    requests::from_blocks(host);
    resolver::resolve_transitions(host);
}

#[cfg(test)]
mod representation_tests;

#[cfg(sm_verify_dll)]
pub(crate) fn verify_hit_and_bomb(host: &mut impl effects::Host) {
    requests::hit_and_bomb(host);
}
