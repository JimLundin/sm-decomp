//! Maridia Puffer pursuit, recoil and native split-coordinate drift.
//! Actor words, including the late health scratch word, remain authoritative.
#![forbid(unsafe_code)]
use crate::game::actors::{api, combat};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rtl::{abs16, sign_extend8};
use sm_game::actors::access::ActorKey as Key;
use sm_game::actors::layouts::EnemyLayout72Late as Puffer;
use sm_game::enemy_projectiles::host::Effects;

const RECOIL_ANGLES: [u16; 10] = [0xc0, 0xe0, 0, 0x20, 0x40, 0x40, 0x60, 0x80, 0xa0, 0xc0];
const SPEED_LIMITS: [u16; 8] = [0x3ff, 0x4ff, 0x5ff, 0x6ff, 0x7ff, 0x8ff, 0x9ff, 0xaff];
const RATES: [u16; 26] = [
    3, 1, 4, 1, 5, 2, 6, 2, 7, 2, 8, 3, 9, 3, 0xa, 4, 0xb, 4, 0xc, 5, 0xd, 5, 0xe, 6, 0xf, 6,
];

pub(crate) fn initialize(game: &mut Game) {
    let E = word!(game.CUR_ENEMY_INDEX) as i32;
    slot!(game, Puffer[E].base; instruction_timer = 1, timer = 0, current_instruction = 0xd841);
    let rate = 4u16.wrapping_mul(slot!(game, Puffer[E].parameter_1)) as usize >> 1;
    let delay = RATES[rate];
    slot!(game, Puffer[E]; var_05 = delay, var_f = delay, var_06 = RATES[rate + 1], var_e = 16, var_a = 0xd92b,
        var_00 = SPEED_LIMITS[slot!(game, Puffer[E].parameter_2) as usize]);
}
fn aim(game: &mut Game) -> (i32, u16) {
    let key = Key::from_legacy(word!(game.CUR_ENEMY_INDEX));
    let angle = (api::angle_to_player(&Runtime::new(game), key) as u8).wrapping_sub(64) as u16;
    let E = word!(game.CUR_ENEMY_INDEX) as i32;
    slot!(game, Puffer[E].var_01 = angle);
    (E, angle)
}
fn drift(game: &mut Game) {
    for horizontal in [true, false] {
        let E = word!(game.CUR_ENEMY_INDEX) as i32;
        let component = sm_game::gameplay_tables::enemy_sine(
            game,
            (slot!(game, Puffer[E].var_02) as u8).wrapping_add(if horizontal { 64 } else { 0 })
                as isize,
        );
        let mut amount = u32::from(abs16(component) >> 8) * u32::from(slot!(game, Puffer[E].var_d));
        if component & 0x8000 != 0 {
            amount = amount.wrapping_neg();
        }
        if horizontal {
            slot!(game, Puffer[E].base(x_pos, x_subpos) += amount);
        } else {
            slot!(game, Puffer[E].base(y_pos, y_subpos) += amount);
        }
    }
}
fn change_speed(accelerating: bool, game: &mut Game) {
    let E = word!(game.CUR_ENEMY_INDEX) as i32;
    let previous = slot!(game, Puffer[E].var_f);
    slot!(game, Puffer[E].var_f, |old| old.wrapping_sub(1));
    if previous == 1 {
        let delay = if accelerating {
            slot!(game, Puffer[E].var_05)
        } else {
            slot!(game, Puffer[E].var_06)
        };
        slot!(game, Puffer[E].var_f = delay);
        let direction = if accelerating { 1 } else { u16::MAX };
        slot!(game, Puffer[E].var_b, |old| old.wrapping_add(
            slot!(game, Puffer[E].var_c).wrapping_mul(direction)
        ));
        slot!(game, Puffer[E].var_d, |old| old.wrapping_add(
            slot!(game, Puffer[E].var_b).wrapping_mul(direction)
        ));
    }
}

pub(crate) fn advance(game: &mut Game) {
    let E = word!(game.CUR_ENEMY_INDEX) as i32;
    let cooldown = slot!(game, Puffer[E].var_07).wrapping_sub(1);
    slot!(game, Puffer[E].var_07 = cooldown);
    if cooldown == 0 {
        slot!(game, Puffer[E]; var_07 = 1, var_03 = 0);
    }
    let state = slot!(game, Puffer[E].var_a);
    let E = word!(game.CUR_ENEMY_INDEX) as i32;
    match state {
        0xd92b => {
            let previous = slot!(game, Puffer[E].var_e);
            slot!(game, Puffer[E].var_e, |old| old.wrapping_sub(1));
            if previous == 1 {
                slot!(game, Puffer[E]; var_e = 16, var_a = 0xd940);
            }
        }
        0xd940 => {
            let (E, angle) = aim(game);
            slot!(game, Puffer[E]; var_02 = angle, var_a = 0xd963, var_c = 24);
        }
        0xd963 => {
            if (slot!(game, Puffer[E].var_d) as i16)
                .wrapping_sub(slot!(game, Puffer[E].var_00) as i16)
                < 0
            {
                change_speed(true, game);
            }
            drift(game);
            let (E, angle) = aim(game);
            let turn = abs16(sign_extend8(
                angle.wrapping_sub(slot!(game, Puffer[E].var_02)),
            ));
            if turn as i32 - 48 & 0x8000 == 0 {
                slot!(game, Puffer[E]; var_a = 0xd97c, var_c = 24);
            }
        }
        0xd97c => {
            if (slot!(game, Puffer[E].var_d) as i16) > 0
                && (slot!(game, Puffer[E].var_b) as i16) > 0
            {
                drift(game);
                change_speed(false, game);
            } else {
                slot!(game, Puffer[E]; var_a = 0xd92b, var_b = 0, var_c = 0, var_d = 0);
            }
        }
        _ => Runtime::new(game).unreachable(),
    }
}

pub(crate) fn shot(game: &mut Game) {
    let E = word!(game.CUR_ENEMY_INDEX) as i32;
    slot!(game, Puffer[E].var_40 = slot!(game, Puffer[E].base.health));
    combat::shot_reaction(&mut Runtime::new(game), api::ShotHandling::Finalize);
    if slot!(game, Puffer[E].base.health) == slot!(game, Puffer[E].var_40)
        && slot!(game, Puffer[E].var_03) == 0
    {
        slot!(game, Puffer[E].base; instruction_timer = 1, timer = 0, current_instruction = 0xd855);
        let direction = word!(game.PROJECTILE_DIR[word!(game.COLLISION_DETECTION_INDEX)]) & 0xf;
        slot!(game, Puffer[E].var_02 = RECOIL_ANGLES[direction as usize]);
        slot!(game, Puffer[E]; var_b = 256, var_d = 1536, var_a = 0xd97c, var_07 = 48, var_03 = 1);
    }
}
