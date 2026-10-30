//! Movement-specific pose normalization without legacy memory access.
#![forbid(unsafe_code)]
use super::{
    content::TransitionTable,
    effects::Host,
    jump, pose,
    state::{AnimationSkip, InputMode, MomentumAction, MovementMode},
};
use crate::{liquid_surface::LiquidAtHeight, session::Session, visual_host::Record};

fn previous_movement(h: &impl Host) -> u16 {
    h.session().player.history.previous.movement_and_facing >> 8
}
fn skip(h: &mut impl Host, value: u16) {
    h.session_mut().player.transitions.animation_skip = AnimationSkip::from_raw(value);
}
fn direction(h: &mut impl Host, previous: bool) -> u8 {
    let table = h.pose_table();
    let pose = if previous {
        h.session().player.history.previous.pose
    } else {
        h.session().player.pose
    };
    table.byte(usize::from(pose) * 8 + 3)
}
fn reload(h: &mut impl Host, #[cfg(sm_verify_dll)] publication: u16) {
    #[cfg(sm_verify_dll)]
    let observed_before = h.session().player.movement_and_facing;
    let table = h.pose_table();
    let packed = table.native_word(usize::from(h.session().player.pose) * 8);
    h.session_mut().player.movement_and_facing = packed;
    #[cfg(sm_verify_dll)]
    crate::transition_trace::publication(
        publication,
        0xa1e,
        observed_before,
        h.session().player.movement_and_facing,
    );
}
pub fn normalize(h: &mut impl Host) {
    let movement = usize::from(h.session().player.movement_and_facing >> 8);
    sm_data::table::check_index(movement, 28);
    let changed = match movement {
        0 => standing(h),
        1 => running(h),
        2 => normal_jump(h),
        3 => spin_jump(h),
        4 | 8 => ball(h, false),
        5 => crouching(h),
        6 => {
            acceleration(h);
            false
        }
        14 => ground_turn(h),
        15 => crouch_transition(h),
        16 => moonwalk(h),
        17..=19 => ball(h, true),
        20 => wall_jump(h),
        23 => air_turn(h, true),
        24 => air_turn(h, false),
        25 => {
            h.session_mut().player.movement_handler = MovementMode::Normal;
            false
        }
        27 => shinespark(h),
        _ => false,
    };
    #[cfg(sm_verify_dll)]
    crate::transition_trace::scalar(1, u16::from(changed));
    h.probe("transitions.phase.4");
    if changed {
        h.probe("transitions.phase.5");
        reload(
            h,
            #[cfg(sm_verify_dll)]
            2,
        );
        h.probe("transitions.phase.6");
        if h.session().player.movement_and_facing & 0xff00 == 0x0e00 {
            h.probe("transitions.phase.7");
            ground_turn(h);
            h.probe("transitions.phase.8");
            reload(
                h,
                #[cfg(sm_verify_dll)]
                3,
            );
            h.probe("transitions.phase.9");
        }
    }
    h.probe("transitions.phase.10");
}
fn standing(h: &mut impl Host) -> bool {
    h.probe("transitions.phase.20");
    if (direction(h, false) == 0 || direction(h, false) == 9)
        && (direction(h, true) == 0 || direction(h, true) == 9)
    {
        skip(h, 1);
    }
    false
}
fn running(h: &mut impl Host) -> bool {
    if previous_movement(h) == 1 {
        skip(h, 0x8000);
    }
    if h.inputs().transitions.turn_request == 0 {
        return false;
    }
    match h.session().player.pose {
        0x45 => h.session_mut().player.pose = 0x25,
        0x46 => h.session_mut().player.pose = 0x26,
        _ => return false,
    }
    true
}
fn acceleration(h: &mut impl Host) {
    let c = &mut h.session_mut().player.contact;
    c.acceleration_mode = if c.extra_run.whole != 0 || c.extra_run.fraction != 0 {
        2
    } else {
        0
    };
}
fn normal_jump(h: &mut impl Host) -> bool {
    h.probe("transitions.phase.21");
    let candidate = match h.session().player.pose {
        0x4d | 0x15 | 0x69 => Some(0xc7),
        0x4e | 0x16 | 0x6a => Some(0xc8),
        _ => None,
    };
    if let Some(selected) = candidate
        && h.session().player.shine_timer != 0
    {
        #[cfg(sm_verify_dll)]
        let observed_before = h.session().player.pose;
        h.session_mut().player.pose = selected;
        #[cfg(sm_verify_dll)]
        crate::transition_trace::publication(4, 0xa1c, observed_before, h.session().player.pose);
        jump::start_shinespark(h);
        if previous_movement(h) == 2 {
            #[cfg(sm_verify_dll)]
            let observed_y = h.session().player.y;
            h.session_mut().player.y = h.session().player.y.wrapping_sub(1);
            let updated_y = h.session().player.y;
            #[cfg(sm_verify_dll)]
            crate::transition_trace::publication(5, 0xafa, observed_y, h.session().player.y);
            #[cfg(sm_verify_dll)]
            let observed_previous_y = h.session().player.previous_position.y.whole;
            h.session_mut().player.previous_position.y.whole = updated_y;
            #[cfg(sm_verify_dll)]
            crate::transition_trace::publication(
                6,
                0xb14,
                observed_previous_y,
                h.session().player.previous_position.y.whole,
            );
        }
        return true;
    }
    acceleration(h);
    if matches!(h.session().player.pose, 0x15 | 0x16)
        && matches!(h.session().player.history.previous.pose, 0x55 | 0x56)
    {
        skip(h, 1);
    }
    if h.inputs().shoot_button & h.inputs().pressed != 0 {
        let aim = u16::from(direction(h, false)) | 0x8000;
        h.session_mut().projectiles.controls.pending_aim = aim;
    }
    false
}
fn add_extra(s: &mut Session) {
    let c = &mut s.player.contact;
    let base = (u32::from(c.base_run.whole) << 16) | u32::from(c.base_run.fraction);
    let extra = (u32::from(c.extra_run.whole) << 16) | u32::from(c.extra_run.fraction);
    let sum = base.wrapping_add(extra);
    c.base_run.fraction = sum as u16;
    c.base_run.whole = (sum >> 16) as u16;
}
fn clear_extra(s: &mut Session) {
    s.player.contact.extra_run.whole = 0;
    s.player.contact.extra_run.fraction = 0;
    s.player.contact.acceleration_mode = 1;
}
fn spin_jump(h: &mut impl Host) -> bool {
    if matches!(previous_movement(h), 3 | 20) {
        skip(h, 1);
        let old = h.session().player.history.previous.movement_and_facing & 0xf;
        let current = h.session().player.movement_and_facing;
        if (old == 8 && current == 0x0304) || (old == 4 && current == 0x0308) {
            add_extra(h.session_mut());
            crate::samus_palette::cancel_speed_boost(h);
            clear_extra(h.session_mut());
        }
    }
    let left = h.session().player.movement_and_facing as u8 == 4;
    if h.session().inventory.equipment.active & 0x20 == 0 {
        let top = pose::loaded_top(h);
        if pose::liquid_at_height(h, top) != LiquidAtHeight::None {
            return false;
        }
    }
    if h.session().inventory.equipment.active & 8 != 0 {
        h.session_mut().player.pose = if left { 0x82 } else { 0x81 };
        if h.session().player.transitions.animation_skip.to_raw() == 0 {
            h.sound(1, 6, 0x33);
        }
    } else if h.session().inventory.equipment.active & 0x200 != 0 {
        h.sound(1, 6, 0x3e);
        h.session_mut().player.pose = if left { 0x1c } else { 0x1b };
    } else if h.session().player.transitions.animation_skip.to_raw() == 0
        && h.inputs().cinematic == 0
    {
        h.sound(1, 6, 0x31);
    }
    false
}
fn crouching(h: &mut impl Host) -> bool {
    if matches!(h.session().player.pose, 0x85 | 0x86)
        && matches!(h.session().player.history.previous.pose, 0xf1 | 0xf2)
    {
        skip(h, 1);
    }
    false
}
fn crouch_transition(h: &mut impl Host) -> bool {
    let pose = h.session().player.pose;
    let selector = if pose.wrapping_sub(0xf1) & 0x8000 != 0 {
        if pose.wrapping_sub(0xdb) & 0x8000 != 0 {
            (2 * (i32::from(pose) - 0x35)) as u16
        } else {
            let index = (i32::from(pose) - 0xdb) as usize;
            sm_data::table::check_index(index, 4);
            return if index < 2 { morph(h) } else { false };
        }
    } else if pose.wrapping_sub(0xf7) & 0x8000 != 0 {
        0
    } else {
        h.session_mut().player.transitions.momentum = MomentumAction::from_raw(7);
        return false;
    };
    h.session_mut().player.transitions.momentum = MomentumAction::from_raw(7);
    let index = usize::from(selector >> 1);
    sm_data::table::check_index(index, 12);
    match index {
        0 | 1 => crouch(h),
        2 | 3 => morph(h),
        4 => unused_ball(h, false),
        5 => unused_ball(h, true),
        _ => false,
    }
}
fn crouch(h: &mut impl Host) -> bool {
    if (i32::from(h.session().player.contact.speed_boost & 0xff00) - 1024) & 0x8000 == 0 {
        let p = &mut h.session_mut().player;
        p.shine_timer = 180;
        p.shine_tick = 1;
        p.palette_frame = 0;
    }
    false
}
fn morph(h: &mut impl Host) -> bool {
    if h.session().inventory.equipment.active & 4 != 0 {
        if previous_movement(h) == 3 {
            h.session_mut().player.contact.acceleration_mode = 2;
        }
        h.session_mut().projectiles.controls.spread_charge = 0;
        false
    } else {
        h.session_mut().player.pose = h.session().player.history.previous.pose;
        true
    }
}
fn unused_ball(h: &mut impl Host, left: bool) -> bool {
    let airborne = matches!(previous_movement(h), 8 | 19);
    let spring = h.session().inventory.equipment.active & 2 != 0;
    h.session_mut().player.pose = match (airborne, spring, left) {
        (true, true, false) => 0x7d,
        (true, true, true) => 0x7e,
        (true, false, false) => 0x31,
        (true, false, true) => 0x32,
        (false, true, false) => 0x79,
        (false, true, true) => 0x7a,
        (false, false, false) => 0x1d,
        (false, false, true) => 0x41,
    };
    true
}
fn moonwalk(h: &mut impl Host) -> bool {
    h.probe("transitions.phase.23");
    if h.inputs().transitions.moonwalk_enabled != 0 {
        return false;
    }
    #[cfg(sm_verify_dll)]
    let observed_before = h.session().player.pose;
    h.session_mut().player.pose = if h.session().player.movement_and_facing as u8 == 4 {
        0x25
    } else {
        0x26
    };
    #[cfg(sm_verify_dll)]
    crate::transition_trace::publication(7, 0xa1c, observed_before, h.session().player.pose);
    true
}
fn ground_turn(h: &mut impl Host) -> bool {
    h.probe("transitions.phase.24");
    if !matches!(h.session().player.history.previous.pose, 0 | 0x9b) {
        let aim = direction(h, true);
        #[cfg(sm_verify_dll)]
        let observed_default_branch = previous_movement(h) != 16 && previous_movement(h) != 5;
        #[cfg(sm_verify_dll)]
        let observed_before = h.session().player.pose;
        let source = if previous_movement(h) == 16 {
            h.session_mut().projectiles.controls.pending_aim = u16::from(aim) | 0x100;
            if h.inputs().jump_button & h.inputs().held != 0 {
                TransitionTable::MoonwalkJumpTurn
            } else {
                TransitionTable::StandingTurn
            }
        } else if previous_movement(h) == 5 {
            TransitionTable::CrouchingTurn
        } else {
            TransitionTable::StandingTurn
        };
        let table = h.transition_table(source);
        h.session_mut().player.pose = u16::from(table.byte(usize::from(aim)));
        #[cfg(sm_verify_dll)]
        if observed_default_branch {
            crate::transition_trace::publication(
                8,
                0xa1c,
                observed_before,
                h.session().player.pose,
            );
        }
    }
    add_extra(h.session_mut());
    clear_extra(h.session_mut());
    true
}
fn air_turn(h: &mut impl Host, jumping: bool) -> bool {
    h.probe(if jumping {
        "samus.turn.jumping"
    } else {
        "samus.turn.falling"
    });
    let table = h.transition_table(if jumping {
        TransitionTable::JumpingTurn
    } else {
        TransitionTable::FallingTurn
    });
    let aim = direction(h, true);
    h.session_mut().player.pose = u16::from(table.byte(usize::from(aim)));
    add_extra(h.session_mut());
    clear_extra(h.session_mut());
    true
}
fn ball(h: &mut impl Host, spring: bool) -> bool {
    let previous = previous_movement(h);
    if (spring && matches!(previous, 17..=19)) || (!spring && matches!(previous, 4 | 8)) {
        skip(h, 0x8000);
    }
    let old = h.session().player.history.previous.movement_and_facing as u8;
    let current = h.session().player.movement_and_facing as u8;
    if (old == 8 && current == 4) || (old != 8 && current == 8) {
        add_extra(h.session_mut());
        crate::samus_palette::cancel_speed_boost(h);
        clear_extra(h.session_mut());
    }
    false
}
fn wall_jump(h: &mut impl Host) -> bool {
    let bottom = pose::loaded_bottom(h);
    if pose::liquid_at_height(h, bottom) != LiquidAtHeight::None {
        return false;
    }
    h.atmospheric_mut().slots[3].frame_and_kind = 1536;
    h.atmospheric_mut().slots[3].remaining = 3;
    h.atmospheric_mut().slots[3].y = bottom;
    h.atmospheric_mut().slots[3].x = if h.session().player.movement_and_facing as u8 != 8 {
        h.session().player.x.wrapping_add(6)
    } else {
        h.session().player.x.wrapping_sub(6)
    };
    h.atmospheric_created(3)
        .expect("atmospheric producer allocation");
    false
}
fn shinespark(h: &mut impl Host) -> bool {
    let pose = h.session().player.pose;
    if pose.wrapping_sub(0xcf) & 0x8000 != 0 {
        let modes = [
            MovementMode::ShinesparkHorizontal,
            MovementMode::ShinesparkHorizontal,
            MovementMode::ShinesparkVertical,
            MovementMode::ShinesparkVertical,
            MovementMode::ShinesparkDiagonal,
            MovementMode::ShinesparkDiagonal,
        ];
        let mode = modes[(i32::from(pose) - 0xc9) as usize];
        let s = h.session_mut();
        s.player.movement_handler = mode;
        s.player.transitions.input_mode = InputMode::Disabled;
        s.projectiles.echoes.index = 0;
        s.projectiles.echoes.x_speed[0] = 0;
        s.projectiles.echoes.x_speed[1] = 0;
        s.projectiles.echoes.x[0] = 0;
        s.projectiles.echoes.x[1] = 0;
        h.sound(3, 9, 0x0f);
    }
    false
}
