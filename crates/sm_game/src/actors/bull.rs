//! Bull, Maridia's gas bag, which waits, then speeds at Samus and slows when she slips
//! off its heading; a shot knocks it away (bank $A8:D821-$A8:DBC6). Was `maridia_puffer`.
#![forbid(unsafe_code)]
use crate::actors::host::{EnemyHost, Shot, abs16, sign_extend8};
use crate::actors::layouts::part_view;
use crate::gameplay_tables::enemy_sine;
use crate::session_slot;

part_view! {
    Bull: EnemyLayout72Late {
        words: [function, acceleration, acceleration_delta, speed, activation_timer, acceleration_timer, parameter_1,
            parameter_2],
        pad1: [u8; 26696],
        words: [max_speed, angle_to_samus, angle, shot_reaction_disabled, var_04, acceleration_interval,
            deceleration_interval, shot_reaction_timer, var_08, var_09, var_0a, var_0b, var_0c, var_0d, var_0e,
            var_0f, var_10, var_11, var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19, var_1a, var_1b,
            var_1c, var_1d, var_1e, var_1f],
        pad3: [u8; 4032],
        words: [previous_health, var_41, var_42, var_43, var_44, var_45, var_46, var_47, var_48, var_49, var_4a,
            var_4b, var_4c, var_4d, var_4e, var_4f, var_50, var_51, var_52, var_53, var_54, var_55, var_56, var_57,
            var_58, var_59, var_5a, var_5b, var_5c, var_5d, var_5e, var_5f],
    }
}

const MOVEMENT_DELAY: u16 = 0xd92b;
const TARGET_SAMUS: u16 = 0xd940;
const ACCELERATING: u16 = 0xd963;
const DECELERATING: u16 = 0xd97c;
/// The knockback angle by the shot's direction.
const RECOIL_ANGLES: [u16; 10] = [0xc0, 0xe0, 0, 0x20, 0x40, 0x40, 0x60, 0x80, 0xa0, 0xc0];
/// $A8:D885: the max speeds; $A8:D895: the acceleration then deceleration intervals.
const MAX_SPEEDS: [u16; 8] = [0x3ff, 0x4ff, 0x5ff, 0x6ff, 0x7ff, 0x8ff, 0x9ff, 0xaff];
const INTERVALS: [u16; 26] = [
    3, 1, 4, 1, 5, 2, 6, 2, 7, 2, 8, 3, 9, 3, 0xa, 4, 0xb, 4, 0xc, 5, 0xd, 5, 0xe, 6, 0xf, 6,
];

/// Initialisation AI - bull ($A8:D8C9).
pub fn initialize(game: &mut impl EnemyHost) {
    let bull = i32::from(game.current_enemy());
    session_slot!(game, Bull[bull].base; instruction_timer = 1, timer = 0, current_instruction = 0xd841);
    let index = 4u16.wrapping_mul(session_slot!(game, Bull[bull].parameter_1)) as usize >> 1;
    let max_speed = MAX_SPEEDS[session_slot!(game, Bull[bull].parameter_2) as usize];
    session_slot!(game, Bull[bull]; acceleration_interval = INTERVALS[index], acceleration_timer = INTERVALS[index],
        deceleration_interval = INTERVALS[index + 1], activation_timer = 16, function = MOVEMENT_DELAY,
        max_speed = max_speed);
}
/// The angle to Samus, from the positive X axis.
fn aim(game: &mut impl EnemyHost) -> (i32, u16) {
    let angle = (game.angle_to_player(game.current_enemy()) as u8).wrapping_sub(64) as u16;
    let bull = i32::from(game.current_enemy());
    session_slot!(game, Bull[bull].angle_to_samus = angle);
    (bull, angle)
}
/// Bull X and Y movement ($A8:D9DB, $A8:DA28): along its angle at its speed.
fn drift(game: &mut impl EnemyHost) {
    for horizontal in [true, false] {
        let bull = i32::from(game.current_enemy());
        let quarter = if horizontal { 64 } else { 0 };
        let angle = (session_slot!(game, Bull[bull].angle) as u8).wrapping_add(quarter);
        let component = enemy_sine(game, angle as isize);
        let speed = u32::from(session_slot!(game, Bull[bull].speed));
        let mut amount = u32::from(abs16(component) >> 8) * speed;
        if component & 0x8000 != 0 {
            amount = amount.wrapping_neg();
        }
        match horizontal {
            true => session_slot!(game, Bull[bull].base(x_pos, x_subpos) += amount),
            false => session_slot!(game, Bull[bull].base(y_pos, y_subpos) += amount),
        }
    }
}
/// Accelerate ($A8:DA71) or decelerate ($A8:DA92), each interval.
fn change_speed(game: &mut impl EnemyHost, accelerating: bool) {
    let bull = i32::from(game.current_enemy());
    let previous = session_slot!(game, Bull[bull].acceleration_timer);
    let timer = previous.wrapping_sub(1);
    session_slot!(game, Bull[bull].acceleration_timer = timer);
    if previous == 1 {
        let interval = match accelerating {
            true => session_slot!(game, Bull[bull].acceleration_interval),
            false => session_slot!(game, Bull[bull].deceleration_interval),
        };
        session_slot!(game, Bull[bull].acceleration_timer = interval);
        let direction = if accelerating { 1 } else { u16::MAX };
        let delta = session_slot!(game, Bull[bull].acceleration_delta).wrapping_mul(direction);
        session_slot!(game, Bull[bull].acceleration, |old| old.wrapping_add(delta));
        let acceleration = session_slot!(game, Bull[bull].acceleration).wrapping_mul(direction);
        session_slot!(game, Bull[bull].speed, |old| old.wrapping_add(acceleration));
    }
}
/// Main AI - bull ($A8:D90B): re-enable the shot reaction, and run its function.
pub fn main_ai(game: &mut impl EnemyHost) {
    let bull = i32::from(game.current_enemy());
    let timer = session_slot!(game, Bull[bull].shot_reaction_timer).wrapping_sub(1);
    session_slot!(game, Bull[bull].shot_reaction_timer = timer);
    if timer == 0 {
        session_slot!(game, Bull[bull]; shot_reaction_timer = 1, shot_reaction_disabled = 0);
    }
    match session_slot!(game, Bull[bull].function) {
        MOVEMENT_DELAY => {
            let previous = session_slot!(game, Bull[bull].activation_timer);
            session_slot!(game, Bull[bull].activation_timer = previous.wrapping_sub(1));
            if previous == 1 {
                session_slot!(game, Bull[bull]; activation_timer = 16, function = TARGET_SAMUS);
            }
        }
        TARGET_SAMUS => {
            let (bull, angle) = aim(game);
            session_slot!(game, Bull[bull]; angle = angle, function = ACCELERATING, acceleration_delta = 24);
        }
        ACCELERATING => {
            let speed = session_slot!(game, Bull[bull].speed) as i16;
            if speed.wrapping_sub(session_slot!(game, Bull[bull].max_speed) as i16) < 0 {
                change_speed(game, true);
            }
            drift(game);
            // Trigger braking if too far off target ($A8:D9AA).
            let (bull, angle) = aim(game);
            let off_target = angle.wrapping_sub(session_slot!(game, Bull[bull].angle));
            let turn = abs16(sign_extend8(off_target));
            if turn as i32 - 48 & 0x8000 == 0 {
                session_slot!(game, Bull[bull]; function = DECELERATING, acceleration_delta = 24);
            }
        }
        DECELERATING => {
            let moving = (session_slot!(game, Bull[bull].speed) as i16) > 0;
            if moving && (session_slot!(game, Bull[bull].acceleration) as i16) > 0 {
                drift(game);
                change_speed(game, false);
            } else {
                session_slot!(game, Bull[bull]; function = MOVEMENT_DELAY, acceleration = 0, acceleration_delta = 0,
                    speed = 0);
            }
        }
        _ => game.unknown_routine(),
    }
}
/// Enemy shot - bull: knocked away from the shot unless it hurt or was recently shot.
pub fn shot(game: &mut impl EnemyHost) {
    let bull = i32::from(game.current_enemy());
    let health = session_slot!(game, Bull[bull].base.health);
    session_slot!(game, Bull[bull].previous_health = health);
    game.shot_reaction(Shot::Finalize);
    let hurt = session_slot!(game, Bull[bull].base.health) != health;
    if !hurt && session_slot!(game, Bull[bull].shot_reaction_disabled) == 0 {
        session_slot!(game, Bull[bull].base; instruction_timer = 1, timer = 0, current_instruction = 0xd855);
        let direction = game.collided_projectile_direction() & 0xf;
        session_slot!(game, Bull[bull].angle = RECOIL_ANGLES[direction as usize]);
        session_slot!(game, Bull[bull]; acceleration = 256, speed = 1536, function = DECELERATING,
            shot_reaction_timer = 48, shot_reaction_disabled = 1);
    }
}
