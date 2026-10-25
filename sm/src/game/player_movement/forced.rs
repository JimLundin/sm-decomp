//! Scripted push, fall and collision recovery share the owned movement state.
#![forbid(unsafe_code)]
use super::{effects::Host, speed};
use crate::game::{
    player_collision::operations as collision,
    player_transitions::{
        self as transitions,
        state::{
            BombJump, DrawMode, FrameGammaMode, InputMode, InterruptionAction, JumpKind,
            LandingBounce, MomentumAction, MovementMode, PoseRequestWord,
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

pub(crate) fn begin_bomb_jump(h: &mut impl Host) {
    transitions::initialize_jump(h, JumpKind::Bomb);
    let p = &mut h.session_mut().player;
    p.movement_handler = MovementMode::BombJump;
    p.collision.pose_input = 0;
}

pub(crate) fn finish_bomb_jump(s: &mut Session) {
    s.player.movement_handler = MovementMode::Normal;
    if s.player.transitions.input_mode != InputMode::Demo {
        s.player.transitions.input_mode = InputMode::Normal;
    }
    s.player.transitions.bomb_jump = BombJump::from_raw(0);
}

pub(crate) fn advance_bomb_jump<H: Host>(h: &mut H, invalid: fn(&mut H)) {
    let jump = h.session().player.transitions.bomb_jump.to_raw();
    if jump == 0 {
        finish_bomb_jump(h.session_mut());
        return;
    }
    match [0, 1, 2, 1][usize::from(jump as u8)] {
        0 => {
            invalid(h);
            return;
        }
        1 => {
            h.probe("samus.bomb.horizontal");
            speed::bomb_rising_x(h);
        }
        _ => {}
    }
    let p = &mut h.session_mut().player;
    if p.contact.y_direction == 1 {
        if p.contact.y_speed.whole & 0x8000 != 0 {
            p.contact.y_speed = Default::default();
            p.contact.y_direction = 2;
            if p.transitions.bomb_jump.to_raw() as u8 != 2 {
                p.contact.acceleration_mode = 2;
            }
        } else if p.contact.y_speed.whole == 0 && p.transitions.input_mode != InputMode::Demo {
            p.transitions.input_mode = InputMode::Normal;
        }
    }
    if h.session().player.contact.y_direction != 2 {
        speed::move_y_with_speed(h);
        if h.session().player.collision.flags == 0 {
            return;
        }
    }
    finish_bomb_jump(h.session_mut());
}

pub(crate) fn consume_crystal_flash(s: &mut Session, frame: u16) {
    let index = usize::from(s.player.transitions.crystal_flash_ammo);
    // Check the selector even on frames that do not consume ammunition.
    let ammo = &mut s.inventory.ammunition[index];
    if frame & 7 == 0 {
        ammo.current = ammo.current.wrapping_sub(1);
        s.inventory
            .restore(crate::game::item_pickups::Resource::Health, 50);
        s.shared_sequence.stage = s.shared_sequence.stage.wrapping_sub(1);
        if (s.shared_sequence.stage as i16) <= 0 {
            if index < 2 {
                s.shared_sequence.stage = 10;
                s.player.transitions.crystal_flash_ammo += 1;
            } else {
                s.player.movement_handler = MovementMode::CrystalFlashFinish;
                s.player.transitions.draw_handler = DrawMode::Normal;
                s.player.animation_clock.remaining = 3;
                s.player.animation_clock.frame = 12;
            }
        }
    }
    s.player.invincibility = 0;
    s.player.knockback_timer = 0;
}

pub(crate) fn grab_by_draygon(direction: u16, h: &mut impl Host) {
    h.session_mut().player.pose = if direction & 1 != 0 { 0xec } else { 0xba };
    transitions::refresh_pose(h);
    let s = h.session_mut();
    s.player.transitions.frame_gamma = FrameGammaMode::DraygonGrabbed;
    s.player.movement_handler = MovementMode::Disabled;
    s.shared_sequence = Default::default();
    s.player.palette.suit_red_green = 0;
    transitions::record_pose_history(s);
    s.player.transitions.clear_pose_requests();
}

pub(crate) fn release_from_draygon(h: &mut impl Host) {
    h.session_mut().player.pose = transitions::equipment::facing(h.session(), 1, 2);
    transitions::refresh_pose(h);
    let s = h.session_mut();
    s.player.movement_handler = MovementMode::Normal;
    s.player.transitions.frame_gamma = FrameGammaMode::Disabled;
    transitions::record_pose_history(s);
    let p = &mut s.player;
    p.contact.base_run = Default::default();
    p.transitions.clear_pose_requests();
    p.contact.y_speed = Default::default();
    p.contact.y_direction = 0;
    p.transitions.bounce = LandingBounce::from_raw(0);
    p.contact.acceleration_mode = 0;
    p.grapple_interaction_flags |= 2;
}

pub(crate) fn struggle_from_draygon(h: &mut impl Host) {
    let p = &mut h.session_mut().player;
    if p.grapple == 0xc77e {
        p.transitions.ordinary = PoseRequestWord::from_raw(0xffff);
        p.transitions.momentum = MomentumAction::from_raw(0);
    }
    let direction = h.inputs().pressed & 0xf00;
    let sequence = &mut h.session_mut().shared_sequence;
    if direction != 0 && direction != sequence.value {
        sequence.value = direction;
        sequence.stage = sequence.stage.wrapping_add(1);
        if (sequence.stage.wrapping_sub(60) as i16) >= 0 {
            release_from_draygon(h);
        }
    }
}

pub(crate) fn crystal_flash_allowed(
    game_state: u16,
    held: u16,
    shoot: u16,
    stationary: bool,
    health: u16,
    reserve: u16,
    ammunition: [u16; 3],
) -> bool {
    ((game_state.wrapping_sub(40) as i16) >= 0 || held == shoot | 0x430)
        && stationary
        && (health.wrapping_sub(51) as i16) < 0
        && reserve == 0
        && ammunition
            .iter()
            .all(|ammo| (ammo.wrapping_sub(10) as i16) >= 0)
}

pub(crate) fn initialize_crystal_flash(h: &mut impl Host) -> bool {
    transitions::refresh_pose(h);
    if h.session().player.movement_and_facing >> 8 != 27 {
        return false;
    }
    let s = h.session_mut();
    transitions::record_pose_history(s);
    let p = &mut s.player;
    p.movement_handler = MovementMode::CrystalFlashStart;
    if p.transitions.input_mode != InputMode::Demo {
        p.transitions.input_mode = InputMode::Disabled;
    }
    p.transitions.shinespark_start_stop_timer = 9;
    p.transitions.crystal_flash_ammo = 0;
    s.shared_sequence.stage = 10;
    s.shared_sequence.value = 0;
    p.palette.suit_red_green = 0;
    p.shine_tick = 7;
    p.palette_frame = 0;
    p.shine_timer = 1;
    p.palette.crystal_flash_delay = 1;
    p.invincibility = 0;
    p.knockback_timer = 0;
    p.transitions.knockback = 0;
    true
}

/// Knockback shares its horizontal step and collision settlement across directions.
pub(crate) fn knockback_motion<H: Host>(h: &mut H, horizontal: bool, vertical: fn(&mut H)) {
    if horizontal {
        speed::bomb_rising_x(h);
    }
    vertical(h);
    clear_collided_motion(h);
}
