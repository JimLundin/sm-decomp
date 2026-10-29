//! Pose request production over the live player owner.
#![forbid(unsafe_code)]
use super::{
    content::{Content, TransitionTable},
    effects::{BounceFault, Host},
    state::*,
};
use crate::{session::Session, visual_host::Record};

pub fn request_pose(session: &mut Session, channel: RequestChannel, word: PoseRequestWord) {
    let t = &mut session.player.transitions;
    match channel {
        RequestChannel::Ordinary => t.ordinary = word,
        RequestChannel::Interrupted => t.interrupted = word,
        RequestChannel::Transitional => t.transitional = word,
    }
}
fn interrupted(h: &mut impl Host, pose: u16) {
    h.session_mut().player.transitions.interrupted = PoseRequestWord::from_raw(pose);
}
fn ordinary(h: &mut impl Host, pose: u16) {
    h.session_mut().player.transitions.ordinary = PoseRequestWord::from_raw(pose);
}
fn facing_pose(h: &impl Host, left: u16, right: u16) -> u16 {
    if h.session().player.movement_and_facing as u8 == 4 {
        left
    } else {
        right
    }
}
#[derive(Clone, Copy)]
enum Hit {
    Stand,
    Ball,
    Falling,
    Unused,
    Grapple,
    Turning,
    Shinespark,
}
const HIT: [Hit; 28] = {
    use Hit::*;
    [
        Stand, Stand, Stand, Stand, Ball, Stand, Falling, Unused, Ball, Ball, Grapple, Turning,
        Turning, Stand, Turning, Turning, Stand, Ball, Ball, Ball, Stand, Stand, Grapple, Turning,
        Turning, Turning, Turning, Shinespark,
    ]
};
fn hit(h: &mut impl Host) -> bool {
    let target = HIT[usize::from(h.session().player.movement_and_facing >> 8)];
    match target {
        Hit::Stand => {
            let pose = facing_pose(h, 84, 83);
            interrupted(h, pose);
            true
        }
        Hit::Ball => {
            let pose = h.session().player.pose;
            interrupted(h, pose);
            true
        }
        Hit::Unused => {
            let pose = facing_pose(h, 52, 51);
            interrupted(h, pose);
            true
        }
        Hit::Falling
            if h.inputs().transitions.frame_gamma != FrameGammaMode::CappedFallAnimation =>
        {
            let pose = facing_pose(h, 84, 83);
            interrupted(h, pose);
            true
        }
        Hit::Grapple => {
            h.session_mut().player.transitions.interruption = InterruptionAction::from_raw(0);
            false
        }
        Hit::Turning | Hit::Falling | Hit::Shinespark => {
            if matches!(target, Hit::Shinespark) && matches!(h.session().player.pose, 0xe8 | 0xe9) {
                h.session_mut().player.animation_clock.remaining = 17;
                h.session_mut().player.animation_clock.frame = 26;
            }
            h.session_mut().player.transitions.interruption = InterruptionAction::from_raw(0);
            h.session_mut().player.transitions.knockback = 0;
            false
        }
    }
}
fn bomb(h: &mut impl Host) {
    if h.session().player.transitions.bomb_jump.to_raw() >> 8 != 0 {
        return;
    }
    let movement = usize::from(h.session().player.movement_and_facing >> 8);
    let _ = [(); 28][movement];
    let selected = match movement {
        0 | 5 if h.inputs().frozen != 0 => {
            h.session_mut().player.transitions.bomb_jump = BombJump::from_raw(0);
            false
        }
        0 | 1 | 5 | 6 | 11..=13 | 16 | 20..=22 => {
            let pose = facing_pose(h, 0x52, 0x51);
            interrupted(h, pose);
            true
        }
        4 | 7..=10 | 17..=19 => {
            let pose = h.session().player.pose;
            interrupted(h, pose);
            true
        }
        2 | 3 | 14 | 15 | 23..=27 => {
            if movement < 26 {
                h.session_mut().player.transitions.input_mode = InputMode::Normal;
            }
            h.session_mut().player.transitions.bomb_jump = BombJump::from_raw(0);
            false
        }
        _ => unreachable!(),
    };
    if selected {
        h.session_mut().player.transitions.interruption = InterruptionAction::from_raw(3);
    }
}
pub fn hit_and_bomb(h: &mut impl Host) {
    if h.session().player.knockback_timer != 0 {
        if h.inputs().transitions.debug_invincibility.wrapping_sub(7) & 0x8000 != 0 {
            if h.inputs().frozen == 0 && h.session().player.transitions.knockback == 0 && hit(h) {
                h.session_mut().player.transitions.interruption = InterruptionAction::from_raw(1);
            }
        } else {
            h.session_mut().player.invincibility = 0;
            h.session_mut().player.knockback_timer = 0;
        }
    } else if h.session().player.transitions.knockback != 0 {
        if h.session().player.movement_and_facing >> 8 == 10 {
            if h.session().projectiles.controls.charge.wrapping_sub(16) & 0x8000 == 0 {
                h.sound(1, 6, 0x41);
            }
            let pose = facing_pose(h, 0x2a, 0x29);
            h.session_mut().player.transitions.transitional = PoseRequestWord::from_raw(pose);
        } else {
            if h.session().player.transitions.hurt.to_raw() == 3 {
                h.session_mut().player.transitions.hurt = HurtAction::from_raw(8);
                return;
            }
            let pose = h.session().player.pose;
            h.session_mut().player.transitions.transitional = PoseRequestWord::from_raw(pose);
        }
        h.session_mut().player.transitions.hurt = HurtAction::from_raw(1);
    } else if h.session().player.transitions.bomb_jump.to_raw() != 0 {
        bomb(h);
    }
}

fn landing_pose(h: &mut impl Host) {
    match h.session().player.history.previous.movement_and_facing >> 8 {
        3 | 20 => {
            let pose = facing_pose(h, 0xa7, 0xa6);
            ordinary(h, pose);
        }
        _ => {
            let metadata = h.pose_table();
            let mut direction = metadata.byte(usize::from(h.session().player.pose) * 8 + 3);
            if direction == 255 {
                let pose = facing_pose(h, 0xa5, 0xa4);
                ordinary(h, pose);
                return;
            }
            if matches!(direction, 2 | 7) {
                if h.inputs().shoot_button & h.inputs().held != 0 {
                    let pose = facing_pose(h, 0xe7, 0xe6);
                    ordinary(h, pose);
                    return;
                }
                let metadata = h.pose_table();
                direction = metadata.byte(usize::from(h.session().player.pose) * 8 + 3);
            }
            let table = Content::transition_table(h, TransitionTable::BlockLandPoses);
            ordinary(h, table.native_word(usize::from(direction) * 2));
        }
    }
}
fn ball_landing(h: &mut impl Host, spring: bool) {
    if spring && h.inputs().jump_button & h.inputs().held != 0 {
        let pose = h.session().player.pose;
        ordinary(h, pose);
        return;
    }
    let value = h.session().player.transitions.bounce.to_raw();
    let tested = if spring {
        u16::from(value as u8)
    } else {
        value
    };
    if tested != 0 {
        let doubled = tested.wrapping_mul(2);
        if doubled == 2 {
            let pose = h.session().player.pose;
            ordinary(h, pose);
            return;
        }
        if doubled != 4 {
            h.invalid_bounce(
                if spring {
                    BounceFault::BlockSpring
                } else {
                    BounceFault::BlockMorph
                },
                value,
            );
        }
    } else if h.session().player.contact.y_speed.whole.wrapping_sub(3) & 0x8000 == 0 {
        let pose = h.session().player.pose;
        ordinary(h, pose);
        return;
    }
    let pose = if spring {
        facing_pose(h, 0x7a, 121)
    } else {
        facing_pose(h, 0x41, 0x1d)
    };
    ordinary(h, pose);
}
pub fn from_blocks(h: &mut impl Host) {
    let request = h.session().player.collision.pose_input;
    if request == 0 {
        return;
    }
    let action = [0, 1, 2, 3, 3, 5][usize::from(request as u8)];
    match action {
        0 => (),
        1 => {
            let sub = h.session().player.collision.pose_input >> 8;
            if sub == 4 {
                return;
            }
            match [0, 1, 2, 3, 4, 5][usize::from(sub)] {
                0 => landing_pose(h),
                1 => ball_landing(h, false),
                2 => {
                    let pose = facing_pose(h, 66, 32);
                    ordinary(h, pose);
                }
                3 => ball_landing(h, true),
                4 => (),
                _ => {
                    let pose = h.session().player.pose;
                    ordinary(h, pose);
                }
            }
            h.session_mut().player.transitions.momentum = MomentumAction::from_raw(5);
        }
        2 => {
            let high = h.session().player.collision.pose_input >> 8;
            if high != 4 {
                let index = high.wrapping_mul(4) >> 1;
                let left = h.session().player.movement_and_facing as u8 == 4;
                let table = Content::transition_table(h, TransitionTable::BlockFallPoses);
                ordinary(
                    h,
                    table.native_word((usize::from(index) + usize::from(left)) * 2),
                );
                h.session_mut().player.transitions.momentum = MomentumAction::from_raw(5);
            }
        }
        3 => {
            let pose = h.session().player.pose;
            ordinary(h, pose);
            h.session_mut().player.transitions.momentum = MomentumAction::from_raw(5);
        }
        _ => {
            let pose = facing_pose(h, 0x84, 0x83);
            ordinary(h, pose);
            h.session_mut().player.transitions.momentum = MomentumAction::from_raw(5);
        }
    }
}
