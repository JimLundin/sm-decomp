//! Scripted push, fall and collision recovery share the owned movement state.
#![forbid(unsafe_code)]
use super::{effects::Host, speed};
use crate::game::{
    player_collision::operations as collision,
    player_transitions::{
        self as transitions,
        state::{
            BombJump, FrameGammaMode, InterruptionAction, MomentumAction, MovementMode,
            PoseRequestWord,
        },
    },
    session::Session,
};

pub(crate) fn begin_ceres_push(s: &mut Session) {
    s.player.movement_handler = MovementMode::Disabled;
    s.player.transitions.frame_gamma = FrameGammaMode::BeginCeresPush;
}

pub(crate) fn push_out_of_ceres(h: &mut impl Host) {
    h.session_mut().player.pose = if h.session().player.movement_and_facing as u8 == 4 {
        0x54
    } else {
        0x53
    };
    transitions::finish_pose_change(h);
    let camera_x = h.inputs().camera_x;
    let p = &mut h.session_mut().player;
    p.y = p.y.wrapping_sub(21).wrapping_add(p.contact.radius);
    p.scripted_movement_direction = if p.x.wrapping_sub(camera_x).wrapping_sub(128) & 0x8000 != 0 {
        1
    } else {
        2
    };
    p.contact.y_speed.whole = 5;
    p.contact.y_speed.fraction = 0;
    p.transitions.bomb_jump = BombJump::from_raw(0);
    p.transitions.frame_gamma = FrameGammaMode::CeresPush;
    p.transitions.clear_pose_requests();
}

pub(crate) fn clear_collided_motion(h: &mut impl Host) {
    if h.session().player.collision.flags != 0 {
        let p = &mut h.session_mut().player;
        p.contact.acceleration_mode = 0;
        p.collision.solid_actor_flags = 0;
        p.transitions.falling = 0;
        p.transitions.landing_motion_compatibility = 0;
        p.transitions.vertical_motion_compatibility = 0;
        p.contact.y_speed = Default::default();
        p.contact.y_direction = 0;
        p.transitions.vertical_auxiliary_compatibility = 0;
        p.contact.base_run = Default::default();
        transitions::align_bottom_with_previous_pose(h);
    }
}

fn accelerate_fall(s: &mut Session) {
    let c = &mut s.player.contact;
    if c.y_speed.whole.wrapping_sub(5) & 0x8000 != 0 {
        let next = speed::value(&c.y_speed).wrapping_add(speed::value(&c.y_acceleration));
        speed::store(&mut c.y_speed, next);
    }
}

fn move_until_collision(h: &mut impl Host, stop_on_floor: bool) {
    h.probe("samus.bomb.falling-x");
    let profile = speed::normal_speed_profile(h);
    let base = speed::base_speed(h, profile);
    let direction = if h.session().player.scripted_movement_direction == 1 {
        collision::HorizontalDirection::Left
    } else {
        collision::HorizontalDirection::Right
    };
    collision::horizontal(h, collision::HorizontalRequest { base, direction });
    let wall = h.session().player.collision.flags != 0;
    if !wall {
        accelerate_fall(h.session_mut());
        let amount = speed::value(&h.session().player.contact.y_speed) as i32;
        collision::displace(
            h,
            collision::DisplacementRequest::Vertical {
                direction: collision::VerticalDirection::Down,
                amount,
            },
        );
    }
    if wall || stop_on_floor && h.session().player.collision.flags != 0 {
        let p = &mut h.session_mut().player;
        p.movement_handler = MovementMode::Normal;
        p.transitions.frame_gamma = if stop_on_floor {
            FrameGammaMode::Disabled
        } else {
            FrameGammaMode::EscapeTimer
        };
        p.scripted_movement_direction = 0;
        clear_collided_motion(h);
        if stop_on_floor {
            let transitions = &mut h.session_mut().player.transitions;
            transitions.interrupted = PoseRequestWord::from_raw(65);
            transitions.interruption = InterruptionAction::from_raw(0);
        }
    }
}

pub(crate) fn advance_ceres_push(h: &mut impl Host) {
    if matches!(
        h.session().player.transitions.ordinary.to_raw(),
        0x4f | 0x50
    ) {
        let transitions = &mut h.session_mut().player.transitions;
        transitions.ordinary = PoseRequestWord::from_raw(0xffff);
        transitions.momentum = MomentumAction::from_raw(0);
    }
    [None, Some(()), Some(())][usize::from(h.session().player.scripted_movement_direction)]
        .expect("non-null function pointer");
    move_until_collision(h, false);
    h.session_mut().player.collision.pose_input = 0;
}

pub(crate) fn finish_forced_fall(h: &mut impl Host) {
    move_until_collision(h, true);
}

pub(crate) fn animate_fall(s: &mut Session) {
    accelerate_fall(s);
    if matches!(s.player.pose, 0x29 | 0x2a | 0x67 | 0x68)
        && s.player.contact.y_speed.whole.wrapping_sub(5) & 0x8000 == 0
    {
        s.player.animation_clock.remaining = 16;
        s.player.animation_clock.frame = 4;
    }
}
