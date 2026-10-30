//! Owned post-movement player rules and finite compatibility interfaces.
#![forbid(unsafe_code)]

pub mod actions;
pub mod animation;
pub mod atmosphere;
pub mod commands;
pub mod content;
pub mod demo;
pub mod effects;
pub mod equipment;
pub mod input;
pub mod jump;
pub mod landing;
pub mod mother_brain;
pub mod normalization;
pub mod pose;
pub mod requests;
pub mod resolver;
pub mod state;
pub mod suit;
pub mod xray;

#[cfg(sm_verify_dll)]
pub use actions::momentum as verify_momentum_action;
pub use animation::advance_animation;
pub use atmosphere::draw_atmospheric;
pub use commands::{cancel_grapple, command};
// Existing finite verification cases exercise the actual private core entries.
#[cfg(sm_verify_dll)]
pub fn verify_interpret(host: &mut impl effects::Host) {
    animation::interpret(host);
}
pub use input::{finish_input, handle_input, request_fallback_pose};
pub use jump::{initialize_jump, start_shinespark};
#[cfg(sm_verify_dll)]
pub use normalization::normalize as verify_normalize;
pub use pose::{
    align_bottom_with_previous_pose, clamp_previous_position, finish_pose_change, liquid_at_height,
    loaded_bottom, loaded_top, record_pose_history, refresh_pose, refresh_pose_rules,
};

pub fn finish_movement(host: &mut impl effects::Host) {
    advance_animation(host, state::AnimationStep::Gameplay);
    requests::hit_and_bomb(host);
    requests::from_blocks(host);
    resolver::resolve_transitions(host);
}

/// Continue the post-movement rules into palette work on the same owned Session.
pub fn finish_movement_and_palette(host: &mut impl effects::Host) {
    finish_movement(host);
    crate::samus_palette::advance(host);
}

#[cfg(sm_verify_dll)]
pub fn verify_hit_and_bomb(host: &mut impl effects::Host) {
    requests::hit_and_bomb(host);
}
