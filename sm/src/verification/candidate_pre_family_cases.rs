//! Additional defined-old pre families, following independently audited recipes.
//! Child of carrier. Deliberately excludes the old odd Definition/u32 dereferences.
use super::{put, slot};
fn CallEprojPreInstr(ea: u32, k: u16, game: &mut Game) {
    super::super::with_execution(game, |ctx| super::super::pre_actions::dispatch(ea, k, ctx));
}
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use crate::types::EnemyData;
use core::mem::offset_of;
pub(super) const CASES: u32 = 44;
pub(super) unsafe fn execute(id: u32, game: &mut Game) -> Option<usize> {
    assert!(id < CASES);
    let k = if id == 41 {
        4
    } else if id == 42 {
        5
    } else {
        0
    };
    for h in (0..36).step_by(2) {
        slot(game, a::eproj_id, h, 0);
    }
    for (field, value) in [
        (a::eproj_id, 1),
        (a::eproj_x_pos, 128),
        (a::eproj_y_pos, 128),
        (a::eproj_radius, 0x0404),
        (a::eproj_x_subpos, 0xff00),
        (a::eproj_y_subpos, 0xff00),
        (a::eproj_x_vel, 0x0080),
        (a::eproj_y_vel, 0x0080),
        (a::eproj_E, 0),
        (a::eproj_F, 0),
        (a::eproj_G, 0),
        (a::eproj_instr_list_ptr, 0x9000),
        (a::eproj_instr_timers, 7),
    ] {
        slot(game, field, k, value);
    }
    for (field, value) in [
        (w::eproj_index, k),
        (w::cur_enemy_index, 0),
        (w::samus_x_pos, 300),
        (w::samus_y_pos, 300),
        (w::samus_x_radius, 4),
        (w::samus_y_radius, 4),
        (w::layer1_x_pos, 0),
        (w::layer1_y_pos, 0),
        (w::power_bomb_explosion_status, 0),
        (w::power_bomb_explosion_radius, 0),
        (w::bomb_counter, 0),
        (w::samus_contact_damage_index, 0),
    ] {
        put(game, field, value);
    }
    for enemy in [0, 64] {
        put(
            game,
            0xf78 + enemy + offset_of!(EnemyData, x_pos),
            if enemy == 0 { 128 } else { 200 },
        );
        put(
            game,
            0xf78 + enemy + offset_of!(EnemyData, y_pos),
            if enemy == 0 { 100 } else { 180 },
        );
    }
    {
        match id {
            0..=3 => {
                put(game, w::samus_x_pos, 100);
                put(game, w::samus_y_pos, 90);
                put(game, w::samus_x_speed_divisor, if id == 2 { 0 } else { 1 });
                put(
                    game,
                    w::samus_contact_damage_index,
                    if id == 3 { 1 } else { 0 },
                );
                slot(game, a::eproj_E, k, if id == 2 { 1 } else { 2 });
                slot(game, a::eproj_F, k, 3);
                CallEprojPreInstr(8818122, k, game);
                if id == 0 {
                    CallEprojPreInstr(8818122, k, game);
                }
            }
            4..=7 => {
                if id & 1 != 0 {
                    put(game, 0xf78 + offset_of!(EnemyData, x_pos), 0xffff);
                }
                slot(game, a::eproj_x_vel, k, 1);
                slot(game, a::eproj_y_vel, k, 16);
                slot(game, a::eproj_y_pos, k, 0x1234);
                CallEprojPreInstr(if id < 6 { 8821317 } else { 8821466 }, k, game);
            }
            8..=11 => {
                slot(game, a::eproj_E, k, if id & 1 == 0 { 0 } else { 0x8000 });
                slot(game, a::eproj_x_pos, k, if id & 1 == 0 { 1 } else { 255 });
                CallEprojPreInstr(if id < 10 { 8822876 } else { 8822906 }, k, game);
            }
            12 => {
                put(game, w::samus_y_pos, 70);
                CallEprojPreInstr(8823592, k, game);
                CallEprojPreInstr(8823592, k, game);
                CallEprojPreInstr(8823652, k, game);
            }
            13..=14 => {
                slot(game, a::eproj_E, k, if id == 13 { 0 } else { 0x8000 });
                slot(game, a::eproj_instr_list_ptr, k, 0xfffe);
                CallEprojPreInstr(8826947, k, game);
            }
            15..=16 => {
                slot(game, a::eproj_E, k, 64);
                put(
                    game,
                    0xf78 + 64 + offset_of!(EnemyData, parameter_1),
                    if id == 15 { 0 } else { 0x8000 },
                );
                CallEprojPreInstr(8827405, k, game);
            }
            17..=18 => {
                slot(game, a::eproj_y_pos, k, if id == 17 { 255 } else { 128 });
                slot(game, a::eproj_y_vel, k, 0x0100);
                CallEprojPreInstr(8829437, k, game);
            }
            19 => {
                slot(game, a::eproj_E, k, 2);
                CallEprojPreInstr(8833101, k, game);
                slot(game, a::eproj_y_pos, k, 214);
                slot(game, a::eproj_y_vel, k, 0x0100);
                slot(game, a::eproj_y_subpos, k, 0);
                CallEprojPreInstr(8833158, k, game);
            }
            20 => {
                slot(game, a::eproj_x_pos, k, 100);
                slot(game, a::eproj_E, k, 77);
                put(game, w::nmi_frame_counter_word, 1);
                CallEprojPreInstr(8837055, k, game);
                put(game, w::nmi_frame_counter_word, 0);
                CallEprojPreInstr(8837055, k, game);
                CallEprojPreInstr(8837086, k, game);
            }
            21..=30 => {
                slot(game, a::eproj_E, k, 2 * (id - 21) as u16);
                slot(game, a::eproj_x_vel, k, 0x0100);
                slot(game, a::eproj_y_vel, k, 0x0200);
                CallEprojPreInstr(8837595, k, game);
            }
            31..=34 => {
                slot(
                    game,
                    a::eproj_E,
                    k,
                    [0xdf40, 0xdf6a, 0xe050, 0xe07a][(id - 31) as usize],
                );
                slot(game, a::eproj_x_vel, k, 0x0100);
                slot(game, a::eproj_y_vel, k, 0x0200);
                CallEprojPreInstr(if id < 33 { 8838969 } else { 8839241 }, k, game);
            }
            35..=36 => {
                slot(game, a::eproj_timers, k, 0x0fff);
                slot(
                    game,
                    a::eproj_y_vel,
                    k,
                    if id == 35 { 0x0101 } else { 0xff01 },
                );
                slot(game, a::eproj_y_pos, k, 100);
                slot(game, a::eproj_instr_list_ptr, k, 0xfffe);
                CallEprojPreInstr(8840709, k, game);
            }
            37..=40 => {
                slot(game, a::eproj_E, k, if id & 1 == 0 { 0x0120 } else { 0x20 });
                slot(game, a::eproj_F, k, if id & 1 == 0 { 0 } else { 1 });
                let pre = if id < 39 { 8822993 } else { 8823076 };
                CallEprojPreInstr(pre, k, game);
                CallEprojPreInstr(pre, k, game);
            }
            41..=42 => {
                slot(game, a::eproj_x_pos, k, 0x12ff);
                slot(game, a::eproj_y_pos, k, 100);
                slot(game, a::eproj_x_vel, k, 0x03ff);
                slot(game, a::eproj_y_vel, k, 0);
                slot(game, a::eproj_E, k, 0xfeff);
                slot(game, a::eproj_F, k, 0xffff);
                CallEprojPreInstr(8818965, k, game);
            }
            43 => {
                slot(game, a::eproj_E, k, 2);
                slot(game, a::eproj_F, k, 64);
                slot(game, a::eproj_G, k, 1);
                CallEprojPreInstr(8835274, k, game);
                CallEprojPreInstr(8835274, k, game);
            }
            _ => unreachable!(),
        }
    }
    None
}
