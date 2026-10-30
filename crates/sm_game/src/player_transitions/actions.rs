//! Transition actions share collision, jump and palette algorithms.
use super::{
    effects::{BounceFault, GrappleTransition, Host},
    jump, landing, pose,
    state::*,
};
use {
    crate::player_collision::operations::{self as collision, DisplacementRequest},
    crate::{
        session::{FixedMotion, Session},
        visual_host::Record,
    },
};

fn clear_vertical(s: &mut Session) {
    s.player.contact.y_speed = FixedMotion::default();
}
fn clear_base(s: &mut Session) {
    s.player.contact.base_run = FixedMotion::default();
}
fn clear_extra(s: &mut Session) {
    s.player.contact.extra_run = FixedMotion::default();
}
fn began_falling(s: &mut Session) {
    if s.player.history.previous.movement_and_facing >> 8 != 6
        && s.player.movement_and_facing >> 8 == 6
    {
        clear_vertical(s);
        s.player.contact.y_direction = 2;
    }
}
fn cancel_and_fall(h: &mut impl Host) {
    crate::samus_palette::cancel_speed_boost(h);
    clear_extra(h.session_mut());
    began_falling(h.session_mut());
}
fn finish_landing(s: &mut Session) {
    s.player.transitions.falling = 0;
    s.player.transitions.landing_motion_compatibility = 0;
    s.player.transitions.vertical_motion_compatibility = 0;
    clear_vertical(s);
    s.player.contact.y_direction = 0;
    s.player.transitions.vertical_auxiliary_compatibility = 0;
    s.player.transitions.bounce = LandingBounce::from_raw(0);
}
fn bounce(h: &mut impl Host, spring: bool) -> bool {
    if spring && h.inputs().jump_button & h.inputs().held != 0 {
        h.session_mut().player.transitions.bounce = LandingBounce::from_raw(0);
        jump::initialize_jump(h, JumpKind::Normal);
        return true;
    }
    let original = h.session().player.transitions.bounce.to_raw();
    let value = if spring {
        u16::from(original as u8)
    } else {
        original
    };
    if value != 0 {
        let doubled = value.wrapping_mul(2);
        if doubled == 2 {
            h.session_mut().player.transitions.bounce = LandingBounce::from_raw(if spring {
                0x602
            } else {
                original.wrapping_add(1)
            });
            h.session_mut().player.contact.y_direction = 1;
            h.session_mut().player.contact.y_speed.fraction = 0;
            h.session_mut().player.contact.y_speed.whole = 0;
            return true;
        }
        if doubled != 4 {
            h.invalid_bounce(
                if spring {
                    BounceFault::LandingSpring
                } else {
                    BounceFault::LandingMorph
                },
                original,
            );
        }
    } else if h.session().player.contact.y_speed.whole.wrapping_sub(3) & 0x8000 == 0 {
        h.session_mut().player.transitions.bounce = LandingBounce::from_raw(if spring {
            0x601
        } else {
            original.wrapping_add(1)
        });
        h.session_mut().player.contact.y_direction = 1;
        h.session_mut().player.contact.y_speed.fraction = 0;
        h.session_mut().player.contact.y_speed.whole = 1;
        return true;
    }
    h.session_mut().player.transitions.bounce = LandingBounce::from_raw(0);
    h.session_mut().player.contact.y_direction = 0;
    clear_vertical(h.session_mut());
    false
}
fn landing_action(h: &mut impl Host) {
    landing::handle_landing(h);
    let index = h.session().player.collision.pose_input >> 8;
    if index == 4 {
        finish_landing(h.session_mut());
        return;
    }
    let keep_motion = match [0, 1, 2, 3, 4, 5][usize::from(index)] {
        0 => {
            if h.session().player.transitions.input_mode.to_raw() != InputMode::Demo.to_raw() {
                h.session_mut().player.transitions.input_mode = InputMode::AutoJump;
            }
            false
        }
        1 => bounce(h, false),
        2 => {
            h.session_mut().player.transitions.bounce = LandingBounce::from_raw(0);
            h.session_mut().player.collision.horizontal_slope_flags = 3;
            false
        }
        3 => bounce(h, true),
        4 => false,
        _ => {
            h.session_mut().player.transitions.bounce = LandingBounce::from_raw(0);
            false
        }
    };
    if !keep_motion {
        h.session_mut().player.transitions.landing_compatibility = 0;
        h.session_mut().player.contact.acceleration_mode = 0;
        clear_base(h.session_mut());
        finish_landing(h.session_mut());
    }
}
fn collision_action(h: &mut impl Host) {
    match [0, 1, 2, 3, 4, 5, 6][usize::from(h.session().player.collision.pose_input as u8)] {
        0 | 3 => (),
        1 => landing_action(h),
        2 => {
            if h.session().player.contact.y_direction != 1 {
                h.session_mut().player.transitions.bounce = LandingBounce::from_raw(0);
                clear_vertical(h.session_mut());
                h.session_mut().player.transitions.falling = 1;
                h.session_mut().player.contact.y_direction = 2;
            }
            h.session_mut().player.transitions.landing_compatibility = 0;
        }
        4 => {
            clear_vertical(h.session_mut());
            h.session_mut()
                .player
                .transitions
                .landing_motion_compatibility = 0;
            h.session_mut().player.contact.y_direction = 2;
        }
        5 => {
            h.session_mut().player.contact.acceleration_mode = 0;
            h.session_mut().player.collision.solid_actor_flags = 0;
            h.session_mut().player.transitions.falling = 0;
            h.session_mut()
                .player
                .transitions
                .landing_motion_compatibility = 0;
            clear_base(h.session_mut());
            h.session_mut().player.transitions.landing_compatibility = 0;
            h.sound(3, 6, 5);
        }
        _ => {
            if h.session().player.collision.solid_actor_flags != 0
                && h.session().player.history.previous.movement_and_facing >> 8 == 9
            {
                let right = h.session().player.movement_and_facing as u8 != 4;
                h.session_mut().player.collision.horizontal_slope_flags = u16::from(right);
                h.session_mut().player.transitions.landing_compatibility = 0;
            }
        }
    }
}
fn clearance_adjustment(h: &mut impl Host) {
    let current = h.session().player.pose;
    let mut amount = if current.wrapping_sub(0xdb) & 0x8000 != 0 {
        const OFFSETS: [u16; 12] = [5, 5, 9, 9, 0, 0, 0, 0, 0, 0, 0, 0];
        i32::from(OFFSETS[(i32::from(current) - 53) as usize]) << 16
    } else if current.wrapping_sub(0xf1) & 0x8000 == 0 && current.wrapping_sub(0xf7) & 0x8000 != 0 {
        5 << 16
    } else {
        return;
    };
    if amount != 0 {
        let metadata = h.pose_table();
        let radius = metadata.byte(usize::from(h.session().player.pose) * 8 + 6);
        h.session_mut().player.contact.radius = u16::from(radius);
        amount =
            collision::displace(h, DisplacementRequest::PoseClearanceProbe { amount }).displacement;
    }
    h.session_mut().player.y = h.session().player.y.wrapping_add((amount >> 16) as u16);
    h.session_mut().player.previous_position.y.whole = h.session().player.y;
    if h.session().player.transitions.bounce.to_raw() != 0 {
        h.session_mut().player.transitions.bounce = LandingBounce::from_raw(0);
        clear_vertical(h.session_mut());
        h.session_mut().player.contact.y_direction = 0;
    }
}
pub fn momentum(h: &mut impl Host, action: MomentumAction) {
    match [0, 1, 2, 3, 4, 5, 6, 7, 8][usize::from(action.to_raw())] {
        0 | 3 | 4 => (),
        1 => {
            if h.session().player.contact.base_run.whole != 0
                || h.session().player.contact.base_run.fraction != 0
            {
                let base = &h.session().player.contact.base_run;
                let extra = &h.session().player.contact.extra_run;
                let sum = ((u32::from(base.whole) << 16) | u32::from(base.fraction))
                    .wrapping_add((u32::from(extra.whole) << 16) | u32::from(extra.fraction));
                h.session_mut().player.contact.base_run.fraction = sum as u16;
                h.session_mut().player.contact.base_run.whole = (sum >> 16) as u16;
                h.session_mut().player.contact.acceleration_mode = 2;
                cancel_and_fall(h);
            } else {
                h.session_mut().player.contact.acceleration_mode = 0;
                crate::samus_palette::cancel_speed_boost(h);
            }
        }
        2 => {
            h.session_mut().player.contact.acceleration_mode = 0;
            crate::samus_palette::cancel_speed_boost(h);
        }
        5 => collision_action(h),
        6 => {
            h.session_mut().player.contact.acceleration_mode = 0;
            clear_base(h.session_mut());
            cancel_and_fall(h);
        }
        7 => clearance_adjustment(h),
        _ => cancel_and_fall(h),
    }
}
fn knockback(h: &mut impl Host) {
    const TARGETS: [u8; 28] = [
        0, 0, 0, 0, 4, 0, 6, 7, 4, 4, 10, 11, 11, 0, 11, 11, 0, 4, 4, 4, 0, 0, 10, 11, 11, 11, 11,
        11,
    ];
    let target = TARGETS[usize::from(h.session().player.history.previous.movement_and_facing >> 8)];
    match target {
        0 | 6
            if target == 0
                || h.inputs().transitions.frame_gamma != FrameGammaMode::CappedFallAnimation =>
        {
            let key = if h.session().player.movement_and_facing as u8 == 4 {
                0x200
            } else {
                0x100
            };
            let held = h.inputs().held & key != 0;
            let direction = if h.session().player.knockback_x_direction != 0 {
                if held { 5 } else { 2 }
            } else if held {
                4
            } else {
                1
            };
            h.session_mut().player.transitions.knockback = direction;
            h.session_mut().player.movement_handler = MovementMode::Knockback;
        }
        4 | 7 => {
            let direction = if h.session().player.movement_and_facing as u8 == 4 {
                1
            } else {
                2
            };
            h.session_mut().player.transitions.knockback = direction;
            h.session_mut().player.movement_handler = MovementMode::Knockback;
        }
        _ => (),
    }
    jump::initialize_jump(h, JumpKind::Knockback);
    h.session_mut().player.transitions.bomb_jump = BombJump::from_raw(0);
    h.session_mut().player.contact.contact_attack = 0;
    h.session_mut().presentation.palette_mut().hurt_flash = 1;
}
fn clamp(h: &mut impl Host) {
    let delta = (h.session().player.x as i16)
        .wrapping_sub(h.session().player.previous_position.x.whole as i16);
    pose::clamp_previous_position(h.session_mut(), delta);
}
fn after_grapple(h: &mut impl Host) {
    clamp(h);
    crate::samus_palette::cancel_speed_boost(h);
    clear_base(h.session_mut());
    clear_vertical(h.session_mut());
    clear_extra(h.session_mut());
}
pub fn interruption(h: &mut impl Host, action: InterruptionAction) {
    match [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10][usize::from(action.to_raw())] {
        0 | 6 | 7 => (),
        1 => knockback(h),
        2 => {
            h.session_mut().player.transitions.knockback = 0;
            h.session_mut().player.movement_handler = MovementMode::Normal;
            clear_vertical(h.session_mut());
            h.session_mut().player.contact.y_direction = 0;
            pose::align_bottom_with_previous_pose(h);
        }
        3 => {
            let value = u16::from(h.session().player.transitions.bomb_jump.to_raw() as u8) | 0x800;
            h.session_mut().player.transitions.bomb_jump = BombJump::from_raw(value);
            h.session_mut().player.movement_handler = MovementMode::BombJumpStart;
            if h.session().player.transitions.input_mode.to_raw() != InputMode::Demo.to_raw() {
                h.session_mut().player.transitions.input_mode = InputMode::Disabled;
            }
        }
        4 => jump::initialize_jump(h, JumpKind::Normal),
        5 => {
            if matches!(h.session().player.movement_and_facing >> 8, 0 | 1 | 5 | 21) {
                let angle = if h.session().player.movement_and_facing as u8 == 4 {
                    192
                } else {
                    64
                };
                h.session_mut().player.transitions.xray_angle = angle;
                h.session_mut().player.animation_clock.frame = 2;
                h.session_mut().player.animation_clock.remaining = 63;
                h.session_mut().player.movement_handler = MovementMode::Xray;
                h.session_mut().player.transitions.input_mode = InputMode::Xray;
                h.session_mut().player.shine_tick = 8;
                h.session_mut().presentation.palette_mut().timer = 1;
                h.session_mut().player.palette_frame = 0;
                h.session_mut().player.shine_timer = 0;
                h.session_mut().projectiles.controls.charge = 0;
                h.session_mut().projectiles.controls.flare_frames = [0; 3];
                h.session_mut().projectiles.controls.flare_timers = [0; 3];
                h.sound(1, 6, 9);
            }
        }
        8 => {
            h.session_mut().player.y = h.session().player.y.wrapping_sub(5);
            h.session_mut().player.previous_position.y.whole = h.session().player.y;
            h.session_mut().player.transitions.frame_alpha = FrameAlphaMode::Empty;
        }
        9 => {
            h.grapple_transition(GrappleTransition::SwingPose);
            after_grapple(h);
        }
        _ => {
            h.grapple_transition(GrappleTransition::AnchoredPose);
            after_grapple(h);
        }
    }
}
fn falling_hurt(h: &mut impl Host) {
    h.session_mut().player.transitions.knockback = 0;
    h.session_mut().player.movement_handler = MovementMode::Normal;
    h.session_mut().player.transitions.bounce = LandingBounce::from_raw(0);
    clear_vertical(h.session_mut());
    h.session_mut().player.transitions.falling = 1;
    h.session_mut().player.contact.y_direction = 2;
    pose::align_bottom_with_previous_pose(h);
    if h.session().player.transitions.input_mode.to_raw() != InputMode::Demo.to_raw() {
        h.session_mut().player.transitions.input_mode = InputMode::Normal;
    }
}
fn extend_animation(h: &mut impl Host) {
    h.session_mut().player.transitions.bounce = LandingBounce::from_raw(0);
    let remaining = h
        .session()
        .player
        .animation_clock
        .remaining
        .wrapping_add(h.session().player.transitions.animation_buffer);
    h.session_mut().player.animation_clock.remaining = remaining;
}
pub fn hurt(h: &mut impl Host, action: HurtAction) {
    match [0, 1, 2, 3, 4, 5, 6, 7, 8][usize::from(action.to_raw())] {
        0 => (),
        1 => falling_hurt(h),
        2 => {
            pose::align_bottom_with_previous_pose(h);
            h.session_mut().player.movement_handler = MovementMode::Normal;
            let mode = if h.inputs().transitions.game_state == 42 {
                InputMode::Demo
            } else {
                InputMode::Normal
            };
            h.session_mut().player.transitions.input_mode = mode;
        }
        3 => extend_animation(h),
        4 => {
            h.session_mut().player.x = h.inputs().camera_x.wrapping_add(128);
            h.session_mut().player.previous_position.x.whole =
                h.inputs().camera_x.wrapping_add(128);
            h.session_mut().player.y = h.inputs().camera_y.wrapping_add(128);
            h.session_mut().player.previous_position.y.whole =
                h.inputs().camera_y.wrapping_add(128);
        }
        5 => {
            h.session_mut().player.y = h.session().player.y.wrapping_add(5);
            h.session_mut().player.previous_position.y.whole = h.session().player.y;
        }
        6 => jump::handle_transition(h),
        7 => clamp(h),
        _ => {
            falling_hurt(h);
            extend_animation(h);
        }
    }
}
