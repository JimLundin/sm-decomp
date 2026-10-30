//! Scripted player control, pose and presentation transitions.
#![forbid(unsafe_code)]
use super::{
    effects::Host, finish_pose_change, record_pose_history, refresh_pose, refresh_pose_rules,
    state::*,
};
use {
    crate::game::player_projectiles,
    sm_game::{
        samus_palette, session::Session, visual_host::Effects as VisualEffects,
        visual_state::PaletteBuffer,
    },
};

fn handlers(s: &mut Session, alpha: u16, beta: FrameBetaMode) {
    s.player.transitions.frame_alpha = FrameAlphaMode::from_raw(alpha);
    s.player.transitions.frame_beta = beta;
}
fn stand(s: &mut Session) {
    s.player.pose = if s.player.movement_and_facing as u8 == 4 {
        2
    } else {
        1
    };
    s.player.transitions.animation_skip = AnimationSkip::from_raw(0);
}
fn frozen(s: &mut Session) {
    handlers(s, 0xe713, FrameBetaMode::Frozen);
    s.player.transitions.draw_handler = DrawMode::Frozen;
}
fn spin_sound(pose: u16) -> u16 {
    match pose {
        0x81 | 0x82 => 0x33,
        0x1b | 0x1c => 0x3e,
        _ => 0x31,
    }
}
fn movement_sound(h: &mut impl Host) {
    let p = &h.session().player;
    let cue = match p.movement_and_facing >> 8 {
        20 if p.animation_clock.frame.wrapping_sub(23) & 0x8000 == 0 => 0x33,
        20 if p.animation_clock.frame.wrapping_sub(13) & 0x8000 == 0 => 0x3e,
        20 => 0x31,
        3 => spin_sound(p.pose),
        _ => return,
    };
    VisualEffects::sound(h, 1, 9, cue);
}

/// Odd command results discard pending transitions only after normal completion.
pub(crate) fn command<H: Host + player_projectiles::host::Host>(
    h: &mut H,
    a: u16,
    finish_ceres: fn(&mut H),
) -> u16 {
    let code = match a & 31 {
        0 => {
            handlers(h.session_mut(), 0xe713, FrameBetaMode::Minimap);
            1
        }
        1 => {
            handlers(h.session_mut(), 0xe695, FrameBetaMode::Gameplay);
            1
        }
        2 => {
            stand(h.session_mut());
            refresh_pose(h);
            h.session_mut().player.transitions.elevator_fade_timer = 60;
            handlers(h.session_mut(), 0xe713, FrameBetaMode::Minimap);
            1
        }
        3 => {
            if h.session().player.grapple != 0xc4f0 {
                h.session_mut().player.grapple = 0xc8c5;
                0
            } else if matches!(h.session().player.movement_and_facing >> 8, 3 | 20) {
                stand(h.session_mut());
                finish_pose_change(h);
                1
            } else {
                0
            }
        }
        4 | 6 => {
            if a & 31 == 4 {
                h.session_mut().projectiles.controls.charge_palette = 0;
            } else {
                handlers(h.session_mut(), 0xe713, FrameBetaMode::EquipmentFrozen);
                if h.session().projectiles.controls.charge.wrapping_sub(15) & 0x8000 == 0 {
                    VisualEffects::sound(h, 1, 15, 2);
                }
            }
            player_projectiles::deselect_owned(h);
            0
        }
        5 | 0x18 => {
            h.session_mut().player.transitions.frame_gamma = if a & 31 == 5 {
                FrameGammaMode::RecoverFromDrain
            } else {
                FrameGammaMode::StruggleDuringDrain
            };
            h.session_mut().player.pose = 0x54;
            h.session_mut().player.transitions.animation_skip = AnimationSkip::from_raw(0);
            finish_pose_change(h);
            player_projectiles::deselect_owned(h);
            handlers(h.session_mut(), 0xe713, FrameBetaMode::Drained);
            1
        }
        7 => {
            super::equipment::face_forward(h);
            let p = &mut h.session_mut().player;
            p.transitions.frame_beta = FrameBetaMode::AnimationOnly;
            p.movement_handler = MovementMode::Normal;
            p.transitions.draw_handler = DrawMode::Elevator;
            p.transitions.input_mode = InputMode::Normal;
            p.transitions.bomb_jump = BombJump::from_raw(0);
            1
        }
        8 => {
            handlers(h.session_mut(), 0xe8cd, FrameBetaMode::Minimap);
            h.session_mut().player.pose = 0;
            h.session_mut().player.transitions.animation_skip = AnimationSkip::from_raw(0);
            refresh_pose(h);
            samus_palette::load_suit(h, PaletteBuffer::Current);
            let p = &mut h.session_mut().player;
            p.transitions.draw_handler = DrawMode::Normal;
            p.history.previous.pose = p.pose;
            p.history.last_different.pose = p.pose;
            p.history.previous.movement_and_facing = p.movement_and_facing;
            p.history.last_different.movement_and_facing = p.movement_and_facing;
            p.y = 0;
            finish_ceres(h);
            1
        }
        9 => {
            let equipment = h.session().inventory.equipment.active;
            let (palette, pose) = if equipment & 0x20 != 0 {
                (0xe1fc, 0x9b)
            } else if equipment & 1 != 0 {
                (0xe1f8, 0x9b)
            } else {
                (0xe1f4, 0)
            };
            h.spawn_palette(palette);
            h.session_mut().player.pose = pose;
            samus_palette::load_suit(h, PaletteBuffer::Current);
            refresh_pose_rules(h);
            h.session_mut().player.animation_clock.remaining = 3;
            h.session_mut().player.animation_clock.frame = 2;
            h.session_mut().shared_sequence.stage = 0;
            1
        }
        0xa => {
            h.session_mut().player.transitions.draw_handler = DrawMode::Disabled;
            0
        }
        0xb => {
            h.session_mut().player.transitions.draw_handler = DrawMode::Normal;
            handlers(h.session_mut(), 0xe695, FrameBetaMode::Gameplay);
            1
        }
        0xc => {
            super::equipment::refresh(h);
            if h.session().player.transitions.frame_beta == FrameBetaMode::EquipmentFrozen {
                handlers(h.session_mut(), 0xe695, FrameBetaMode::Gameplay);
            }
            1
        }
        0xd => u16::from(h.session().player.grapple != 0xc4f0) * 2,
        0xe => {
            handlers(h.session_mut(), 0xe695, FrameBetaMode::Escape);
            1
        }
        0xf => {
            h.session_mut().player.transitions.frame_gamma = FrameGammaMode::EscapeTimer;
            0
        }
        0x10 | 0x1b => {
            if h.session().player.transitions.frame_beta != FrameBetaMode::Drained {
                let (alpha, beta) = if a & 31 == 0x10 {
                    (0xe695, FrameBetaMode::Gameplay)
                } else {
                    (0xe713, FrameBetaMode::Minimap)
                };
                handlers(h.session_mut(), alpha, beta);
            }
            1
        }
        0x11 => {
            sm_game::palette_effects::execution::set_enabled(h.session_mut(), false);
            frozen(h.session_mut());
            1
        }
        0x12 => {
            h.session_mut().player.special_palette_flags = 1;
            0
        }
        0x13 => {
            h.session_mut().player.special_palette_flags = 0;
            samus_palette::load_suit(h, PaletteBuffer::Current);
            0
        }
        0x14 => {
            if h.session().inventory.health.current.wrapping_sub(31) & 0x8000 != 0 {
                VisualEffects::sound(h, 3, 6, 2);
            }
            if h.session().player.grapple != 0xc4f0 {
                VisualEffects::sound(h, 1, 6, 6);
            } else if h.session().player.movement_and_facing as u8 == 3 {
                // This command tests facing, unlike the movement-sound command.
                let cue = spin_sound(h.session().player.pose);
                VisualEffects::sound(h, 1, 6, cue);
            }
            0
        }
        0x15 => {
            record_pose_history(h.session_mut());
            player_projectiles::deselect_owned(h);
            frozen(h.session_mut());
            1
        }
        0x16 | 0x17 => {
            let enabled = a & 31 == 0x16;
            h.session_mut().player.special_palette_flags = if enabled { 0x8000 } else { 0 };
            h.session_mut().player.palette_frame = u16::from(enabled);
            h.session_mut().presentation.palette_mut().timer = u16::from(enabled);
            h.session_mut().projectiles.controls.charge_palette = 0;
            if !enabled {
                samus_palette::load_suit(h, PaletteBuffer::Current);
                h.session_mut().player.animation_clock.remaining = 1;
                h.session_mut().player.animation_clock.frame = 13;
            }
            0
        }
        0x19 => {
            h.session_mut().player.animation_clock.remaining = 1;
            h.session_mut().player.animation_clock.frame = 28;
            1
        }
        0x1a => {
            h.session_mut().player.transitions.frame_beta = FrameBetaMode::LowHealth;
            0
        }
        0x1c => {
            movement_sound(h);
            0
        }
        0x1d => {
            if matches!(h.session().player.movement_and_facing >> 8, 3 | 20) {
                VisualEffects::sound(h, 1, 15, 0x32);
            } else if h.inputs().shoot_button & h.inputs().held == 0
                && h.session().projectiles.controls.charge.wrapping_sub(16) & 0x8000 != 0
            {
                VisualEffects::sound(h, 1, 15, 2);
            }
            0
        }
        0x1e => {
            if h.inputs().transitions.game_state == 8 {
                if matches!(h.session().player.movement_and_facing >> 8, 3 | 20) {
                    movement_sound(h);
                } else if h.session().projectiles.controls.charge.wrapping_sub(16) & 0x8000 == 0 {
                    VisualEffects::sound(h, 1, 9, 0x41);
                }
            }
            0
        }
        0x1f => {
            cancel_grapple(h);
            0
        }
        _ => unreachable!(),
    };
    if code & 1 == 0 {
        return code >> 1;
    }
    h.session_mut().player.transitions.clear_pose_requests();
    0xffff
}

/// Cancel the active beam before equipment or scripted control takes over.
pub(crate) fn cancel_grapple(h: &mut impl Host) {
    if h.session().player.grapple == 0xc4f0 {
        return;
    }
    let p = &mut h.session_mut().player;
    p.grapple_extension_compatibility.x = 0;
    p.grapple_extension_compatibility.y = 0;
    p.grapple_direction = 0;
    p.grapple_direction_compatibility = 0;
    p.transitions.grapple_walljump_timer = 0;
    p.grapple_slow_scrolling = 0;
    p.grapple_delay = 0;
    p.grapple_flags = 0;
    let beams = h.session().inventory.beams.active;
    samus_palette::write_beam_palette(h, beams & 0xfff);
    h.session_mut().player.grapple = 0xc4f0;
    h.session_mut().player.transitions.draw_handler = DrawMode::Normal;
}
