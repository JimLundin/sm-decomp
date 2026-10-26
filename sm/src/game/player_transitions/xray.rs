//! X-ray entry gates, turning and the original stand-up exit adjustment.
#![forbid(unsafe_code)]
use super::{
    effects::Host,
    state::{InputMode, MovementMode, PoseRequestWord},
};
use crate::game::{session::Session, visual_host::Record};

pub(crate) fn request(h: &mut impl Host) -> bool {
    const POSTURES: [u8; 28] = [
        1, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    ];
    let s = h.session();
    let pose = s.player.pose;
    if s.projectiles.controls.cooldown == 7
        && s.projectiles.bomb_count == 5
        && s.player.speed_divisor == 2
        || pose.wrapping_sub(0xa4) & 0x8000 == 0
            && (pose.wrapping_sub(0xa8) & 0x8000 != 0
                || pose.wrapping_sub(0xe0) & 0x8000 == 0 && pose.wrapping_sub(0xe8) & 0x8000 != 0)
        || h.inputs().transitions.game_state != 8
        || h.inputs().power_bomb_active != 0
        || s.player.contact.y_speed.whole != 0
        || s.player.contact.y_speed.fraction != 0
        || POSTURES[usize::from(s.projectiles.preparation.previous_movement_and_facing >> 8)] == 0
    {
        return false;
    }
    let posture = POSTURES[usize::from(s.player.movement_and_facing >> 8)];
    let pose = match posture {
        1 => 0xd5,
        2 => 0xd9,
        _ => return false,
    } + u16::from(s.player.movement_and_facing as u8 == 4);
    h.session_mut().player.transitions.interrupted = PoseRequestWord::from_raw(pose);
    true
}

/// Each scheduler retains its other flags; their original update order is fixed.
pub(crate) fn set_world_running(s: &mut Session, running: bool) {
    for flags in [
        &mut s.enemy_projectiles.flags,
        &mut s.mechanisms.flags,
        &mut s.visual.animated_tiles.flags,
        &mut s.visual.palette_effects.flags,
    ] {
        *flags = if running {
            *flags | 0x8000
        } else {
            *flags & 0x7fff
        };
    }
}

pub(crate) fn finish(h: &mut impl Host) {
    let p = &mut h.session_mut().player;
    p.pose = if p.movement_and_facing >> 8 == 5 {
        0x27
    } else {
        1
    } + u16::from(p.movement_and_facing as u8 == 4);
    super::finish_pose_change(h);
    let p = &mut h.session_mut().player;
    p.movement_handler = MovementMode::Normal;
    p.transitions.input_mode = InputMode::Normal;
    p.palette.xray_progress = u16::MAX;
    // Capture native content before rereading the pose changed by callbacks.
    let table = h.pose_table();
    let adjustment = u16::from(table.byte(usize::from(h.session().player.pose) * 8 + 6))
        .wrapping_sub(h.session().player.contact.radius);
    if adjustment & 0x8000 == 0 {
        let p = &mut h.session_mut().player;
        p.y = p.y.wrapping_sub(adjustment);
        p.previous_position.y.whole = p.y;
    }
    set_world_running(h.session_mut(), true);
}

pub(crate) fn turn(h: &mut impl Host, opposite_direction_held: bool) {
    let p = &mut h.session_mut().player;
    let left = p.movement_and_facing as u8 == 4;
    if p.movement_and_facing >> 8 == 14 {
        if p.animation_clock.frame != 2 || p.animation_clock.remaining != 1 {
            return;
        }
        let standing = p.pose == if left { 0x25 } else { 0x26 };
        p.pose = if standing { 0xd5 } else { 0xd9 } + u16::from(left);
    } else {
        if !opposite_direction_held {
            return;
        }
        p.transitions.xray_angle = 256u16.wrapping_sub(p.transitions.xray_angle);
        p.pose = if p.movement_and_facing >> 8 == 5 {
            0x43
        } else {
            0x25
        } + u16::from(left);
    }
    super::finish_pose_change(h);
}

#[derive(Clone, Copy)]
pub(crate) enum BlendPhase {
    Setup,
    Beam,
    Restore,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BeamPhase {
    Initial,
    Widening,
    Full,
}

/// Keep the original wide equality tests and signed wrapping carry/borrow tests.
pub(crate) fn steer(angle: &mut u16, width: u16, up: bool) {
    if (angle.wrapping_sub(128) & 0x8000 != 0) == up {
        let limit = if up { 0 } else { 128 };
        let gap = i32::from(*angle) - i32::from(width);
        if gap != limit
            && (((gap - limit) as i16) < 0 || {
                let borrow = i32::from(*angle == 0);
                *angle = angle.wrapping_sub(1);
                ((i32::from(*angle) - i32::from(width) - borrow - limit) as i16) < 0
            })
        {
            *angle = width.wrapping_add(limit as u16);
        }
    } else {
        let limit = if up { 256 } else { 128 };
        let sum = i32::from(width) + i32::from(*angle);
        if sum != limit
            && ((sum - limit) as i16 >= 0 || {
                let carried = width
                    .wrapping_add(*angle)
                    .wrapping_add(u16::from(*angle == u16::MAX));
                *angle = angle.wrapping_add(1);
                carried != limit as u16 && (carried as i16).wrapping_sub(limit as i16) >= 0
            })
        {
            *angle = (limit as u16).wrapping_sub(width);
        }
    }
}

pub(crate) fn widen(demo: &mut super::demo::DemoInput) {
    let speed = ((u32::from(demo.delay) << 16) | u32::from(demo.program)).wrapping_add(2048);
    demo.program = speed as u16;
    demo.delay = (speed >> 16) as u16;
    let width = ((u32::from(demo.held) << 16) | u32::from(demo.pressed)).wrapping_add(speed);
    demo.pressed = width as u16;
    demo.held = (width >> 16) as u16;
    if demo.held.wrapping_sub(11) & 0x8000 == 0 {
        demo.pressed = 0;
        demo.held = 10;
        demo.pre_action = demo.pre_action.wrapping_add(1);
    }
}
