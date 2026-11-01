//! Owned post-movement player rules and finite compatibility interfaces.

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
/// Reset Samus for a new game, a load or a demo.
pub fn initialize(h: &mut impl effects::SamusLease) {
    use crate::{ram::words as ram, samus_palette, visual_state::PaletteBuffer};
    use state::*;
    let debug_invincibility = h.word(ram::DEBUG_INVINCIBILITY);
    // This compatibility workspace is only part of Session, including byte aliases.
    for address in (0xa02..=0xe0b).rev() {
        h.set_byte(address, 0);
    }
    if h.word(ram::GAME_STATE) != 40 {
        let ceres = h.word(ram::LOADING_GAME_STATE) == 34;
        let t = &mut h.session_mut().player.transitions;
        t.frame_alpha = FrameAlphaMode::Empty;
        t.frame_beta = if ceres {
            FrameBetaMode::Minimap
        } else {
            FrameBetaMode::Fanfare
        };
        t.draw_handler = DrawMode::Normal;
        t.momentum = MomentumAction::from_raw(if ceres { u16::MAX } else { 0 });
        t.interruption = InterruptionAction::from_raw(0);
        t.hurt = HurtAction::from_raw(0);
        if ceres {
            samus_palette::load_suit(h, PaletteBuffer::Current);
        }
        h.session_mut().player.transitions.input_mode = InputMode::Normal;
        if !ceres {
            h.write_slice_le(ram::DEBUG_INVINCIBILITY, debug_invincibility);
        }
    }
    let escape = h.session().room.area == 6;
    let transitions = &mut h.session_mut().player.transitions;
    transitions.invalidate_pose_requests();
    transitions.frame_gamma = if escape {
        FrameGammaMode::EscapeTimer
    } else {
        FrameGammaMode::Disabled
    };
    h.session_mut().player.movement_handler = MovementMode::Normal;
    h.write_slice_le(ram::UNUSED_WORD_7E0A5E, (-2764i16) as u16);
    h.write_slice_le(ram::SAMUS_PREV_HEALTH_FOR_FLASH, 50);
    h.session_mut().presentation.palette_mut().visor_timer = 1537;
    for shot in &mut h.session_mut().projectiles.shots {
        shot.motion = 0xb169;
    }
    let state = h.session_mut();
    let p = &mut state.player;
    p.grapple = 0xc4f0;
    p.collision.horizontal_slope_flags = 3;
    state.presentation.palette_mut().hurt_flash = 0;
    p.special_palette_flags = 0;
    let one = crate::geometry::FixedMotion {
        fraction: 0,
        whole: 1,
    };
    p.camera_distance.fill(one);
    xray::clear_window(h);
    h.session_mut().player.contact.y_acceleration.fraction = 0x1c00;
    h.session_mut().player.contact.y_acceleration.whole = 0;
    h.session_mut().room.liquid.surface.whole = u16::MAX;
    h.session_mut().room.liquid.lava_acid.whole = u16::MAX;
    h.update_beam_graphics();
    crate::sequence_state::Access::set_cinematic_function(h, 0);
    let state = h.session_mut();
    let p = &mut state.player;
    p.pose = 0;
    p.movement_and_facing = 0;
    p.history = Default::default();
    h.write_slice_le(ram::ENEMY_INDEX_TO_SHAKE, u16::MAX);
    h.session_mut().projectiles.controls.selected_weapon = 0;
    h.session_mut().projectiles.controls.automatic_weapon = 0;
    let state = h.session_mut();
    let p = &mut state.player;
    p.invincibility = 0;
    p.knockback_timer = 0;
    state.presentation.palette_mut().hurt_flash = 0;
    h.write_slice_le(ram::DEBUG_INVINCIBILITY, 0);
    if h.word(ram::GAME_STATE) == 40 {
        demo::load_demo(h);
    }
    let health = h.session().inventory.health.current;
    h.write_slice_le(ram::SAMUS_PREV_HEALTH_FOR_FLASH, health);
}
