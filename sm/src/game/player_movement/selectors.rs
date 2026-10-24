//! The 28 normal movement selectors, sharing complete motion operations.
#![forbid(unsafe_code)]
use super::{content::Content, effects::Host, presentation, speed};
use crate::game::visual_host::Record;
use crate::game::{
    liquid_surface::LiquidAtHeight, player_collision::operations as collision,
    player_transitions as transitions, samus_palette,
};

fn clear_run(h: &mut impl Host) {
    let c = &mut h.session_mut().player.contact;
    c.extra_run.whole = 0;
    c.extra_run.fraction = 0;
    c.base_run.whole = 0;
    c.base_run.fraction = 0;
    c.acceleration_mode = 0;
}
fn cancel_and_clear_run(h: &mut impl Host) {
    samus_palette::cancel_speed_boost(h);
    clear_run(h);
}
fn clear_pose_input(h: &mut impl Host) {
    h.session_mut().player.collision.pose_input = 0;
}

pub(super) fn advance(h: &mut impl Host) {
    let movement = usize::from(h.session().player.movement_and_facing >> 8);
    // Keep the original checked 28-entry dispatch, including invalid selectors.
    let _ = [(); 28][movement];
    match movement {
        0 => standing(h),
        1 => running(h),
        2 | 25 => speed::jumping(h),
        3 => spin_jump(h),
        4 => grounded_ball(h, false),
        5 => {
            speed::no_base_x(h);
            speed::no_speed_y(h);
            clear_run(h);
        }
        6 => falling(h),
        7 | 11 | 12 | 26 | 27 => clear_pose_input(h),
        8 | 19 => falling_ball(h),
        9 | 13 => {}
        10 => {
            clear_pose_input(h);
            speed::no_speed_y(h);
        }
        14 => ground_turn(h),
        15 => crouch_transition(h),
        16 => {
            speed::handle_movement_x(h);
            speed::no_speed_y(h);
        }
        17 => grounded_ball(h, true),
        18 => {
            if h.session().player.transitions.bounce.to_raw() != 0 {
                speed::morphed_bouncing(h);
            } else {
                speed::jumping(h);
            }
        }
        20 => wall_jump(h),
        21 => {
            speed::no_base_x(h);
            speed::no_speed_y(h);
            cancel_and_clear_run(h);
        }
        22 => {
            if h.session().player.collision.pose_input != 5 {
                clear_pose_input(h);
            }
        }
        23 | 24 => air_turn(h),
        _ => unreachable!(),
    }
}
fn standing(h: &mut impl Host) {
    if !matches!(h.session().player.pose, 0 | 0x9b) {
        if matches!(h.session().player.pose, 1 | 2)
            && h.inputs().shoot_button & h.inputs().held != 0
        {
            h.session_mut().player.animation_clock.remaining = 16;
            h.session_mut().player.animation_clock.frame = 0;
        }
        speed::no_base_x(h);
        speed::no_speed_y(h);
        cancel_and_clear_run(h);
    } else {
        if h.session().room.entry.elevator_status != 0 {
            h.session_mut().player.contact.collision_direction = 2;
            collision::displace(
                h,
                collision::DisplacementRequest::TerrainOnly {
                    axis: collision::Axis::Y,
                    amount: 1 << 16,
                },
            );
        }
        clear_pose_input(h);
    }
}
fn running(h: &mut impl Host) {
    speed::handle_movement_x(h);
    speed::no_speed_y(h);
    if h.session().player.animation_clock.remaining == 1
        && [0u8, 0, 1, 0, 0, 0, 0, 1, 0, 0][usize::from(h.session().player.animation_clock.frame)]
            != 0
    {
        presentation::footsteps(h);
        if h.inputs().cinematic == 0
            && !h.boss_active()
            && h.session().player.shine_timer == 0
            && h.session().player.contact.speed_boost & 0x400 == 0
        {
            h.sound(3, 6, 6);
        }
    }
}
fn spin_jump(h: &mut impl Host) {
    let mut submerged = false;
    if h.session().player.palette.suit_index & 4 == 0 {
        let top = transitions::loaded_top(h);
        submerged = transitions::liquid_at_height(h, top) != LiquidAtHeight::None;
    }
    if !submerged {
        if h.session().inventory.equipment.active & 0x200 != 0
            && h.session().player.contact.y_direction == 2
        {
            let minimum = if h.session().player.transitions.liquid_physics != 0 {
                0x80i16
            } else {
                0x280i16
            };
            let middle = h.session().player.contact.y_speed.middle() as i16;
            if middle.wrapping_sub(minimum) >= 0 && middle.wrapping_sub(0x500) < 0 {
                let history = &mut h.session_mut().projectiles.preparation.aim_history;
                *history = (*history & 0xff00) | 1;
                if h.inputs().jump_button & h.inputs().pressed != 0 {
                    transitions::initialize_jump(h, transitions::state::JumpKind::Normal);
                }
            }
        }
        if matches!(h.session().player.pose, 0x81 | 0x82) {
            h.session_mut().player.contact.contact_attack = 3;
        } else if h.session().projectiles.controls.charge.wrapping_sub(60) & 0x8000 == 0 {
            h.session_mut().player.contact.contact_attack = 4;
        }
    } else if h.session().player.animation_clock.remaining == 1 {
        let frames = Content::spin_sound_frames(h);
        if frames.byte(usize::from(h.session().player.animation_clock.frame)) != 0 {
            h.sound(1, 6, 0x2f);
        }
    }
    speed::spin_jumping(h);
}
fn grounded_ball(h: &mut impl Host, spring: bool) {
    if h.session().player.contact.acceleration_mode == 0 {
        let pose = if h.session().player.movement_and_facing as u8 == 4 {
            if spring { 0x7a } else { 0x41 }
        } else if spring {
            0x79
        } else {
            0x1d
        };
        if h.session().player.pose == pose {
            speed::no_base_x(h);
            if !speed::check_and_move_y(h) {
                speed::no_speed_y(h);
                cancel_and_clear_run(h);
            }
            return;
        }
    }
    speed::handle_movement_x(h);
    if !speed::check_and_move_y(h) {
        speed::no_speed_y(h);
    }
}
fn falling(h: &mut impl Host) {
    speed::falling(h);
    if matches!(h.session().player.pose, 0x29 | 0x2a | 0x67 | 0x68)
        && h.session().player.contact.y_speed.whole.wrapping_sub(5) & 0x8000 == 0
        && h.session().player.animation_clock.frame.wrapping_sub(5) & 0x8000 != 0
    {
        h.session_mut().player.animation_clock.remaining = 8;
        h.session_mut().player.animation_clock.frame = 5;
    }
}
fn falling_ball(h: &mut impl Host) {
    if h.inputs().held & 0x300 == 0 && h.session().player.contact.acceleration_mode == 0 {
        cancel_and_clear_run(h);
    }
    if h.session().player.transitions.bounce.to_raw() != 0 {
        speed::morphed_bouncing(h);
    } else {
        speed::morphed_falling(h);
    }
}
fn crouch_transition(h: &mut impl Host) {
    let pose = h.session().player.pose;
    let apply = if pose.wrapping_sub(0xf1) & 0x8000 == 0 {
        true
    } else if pose.wrapping_sub(0xdb) & 0x8000 != 0 {
        let index = (i32::from(pose) - 53) as usize;
        let adjust = [
            false, false, false, false, true, true, false, false, false, false, true, true,
        ][index];
        if adjust {
            h.session_mut().player.collision.horizontal_slope_flags = 3;
            h.session_mut().player.crouch_transition_work = 0;
        }
        true
    } else {
        false
    };
    if apply {
        speed::no_base_x(h);
        if !speed::check_and_move_y(h) {
            speed::no_speed_y(h);
        }
    }
    if h.session().player.collision.pose_input == 1025 {
        h.session_mut().player.contact.y_speed.fraction = 0;
        h.session_mut().player.contact.y_speed.whole = 0;
        h.session_mut().player.contact.y_direction = 0;
        h.session_mut().player.transitions.bounce = transitions::state::LandingBounce::from_raw(0);
    }
    clear_pose_input(h);
}
fn ground_turn(h: &mut impl Host) {
    speed::handle_movement_x(h);
    speed::no_speed_y(h);
    clear_after_turn(h);
}
fn air_turn(h: &mut impl Host) {
    speed::handle_movement_x(h);
    if !speed::check_and_move_y(h) {
        speed::no_speed_y(h);
    }
    clear_after_turn(h);
}
fn clear_after_turn(h: &mut impl Host) {
    samus_palette::cancel_speed_boost(h);
    h.session_mut().player.contact.extra_run.whole = 0;
    h.session_mut().player.contact.extra_run.fraction = 0;
    clear_pose_input(h);
}
fn wall_jump(h: &mut impl Host) {
    if h.session().player.animation_clock.frame.wrapping_sub(23) & 0x8000 != 0 {
        if h.session().player.animation_clock.frame.wrapping_sub(3) & 0x8000 == 0
            && h.session().projectiles.controls.charge.wrapping_sub(60) & 0x8000 == 0
        {
            h.session_mut().player.contact.contact_attack = 4;
        }
    } else {
        h.session_mut().player.contact.contact_attack = 3;
    }
    speed::jumping(h);
}
