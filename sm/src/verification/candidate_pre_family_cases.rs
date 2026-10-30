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
        slot(game, a::EPROJ_ID, h, 0);
    }
    for (field, value) in [
        (a::EPROJ_ID, 1),
        (a::EPROJ_X_POS, 128),
        (a::EPROJ_Y_POS, 128),
        (a::EPROJ_RADIUS, 0x0404),
        (a::EPROJ_X_SUBPOS, 0xff00),
        (a::EPROJ_Y_SUBPOS, 0xff00),
        (a::EPROJ_X_VEL, 0x0080),
        (a::EPROJ_Y_VEL, 0x0080),
        (a::EPROJ_E, 0),
        (a::EPROJ_F, 0),
        (a::EPROJ_G, 0),
        (a::EPROJ_INSTR_LIST_PTR, 0x9000),
        (a::EPROJ_INSTR_TIMERS, 7),
    ] {
        slot(game, field, k, value);
    }
    for (field, value) in [
        (w::EPROJ_INDEX, k),
        (w::CUR_ENEMY_INDEX, 0),
        (w::SAMUS_X_POS, 300),
        (w::SAMUS_Y_POS, 300),
        (w::SAMUS_X_RADIUS, 4),
        (w::SAMUS_Y_RADIUS, 4),
        (w::LAYER1_X_POS, 0),
        (w::LAYER1_Y_POS, 0),
        (w::POWER_BOMB_EXPLOSION_STATUS, 0),
        (w::POWER_BOMB_EXPLOSION_RADIUS, 0),
        (w::BOMB_COUNTER, 0),
        (w::SAMUS_CONTACT_DAMAGE_INDEX, 0),
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
                put(game, w::SAMUS_X_POS, 100);
                put(game, w::SAMUS_Y_POS, 90);
                put(game, w::SAMUS_X_SPEED_DIVISOR, if id == 2 { 0 } else { 1 });
                put(
                    game,
                    w::SAMUS_CONTACT_DAMAGE_INDEX,
                    if id == 3 { 1 } else { 0 },
                );
                slot(game, a::EPROJ_E, k, if id == 2 { 1 } else { 2 });
                slot(game, a::EPROJ_F, k, 3);
                CallEprojPreInstr(8818122, k, game);
                if id == 0 {
                    CallEprojPreInstr(8818122, k, game);
                }
            }
            4..=7 => {
                if id & 1 != 0 {
                    put(game, 0xf78 + offset_of!(EnemyData, x_pos), 0xffff);
                }
                slot(game, a::EPROJ_X_VEL, k, 1);
                slot(game, a::EPROJ_Y_VEL, k, 16);
                slot(game, a::EPROJ_Y_POS, k, 0x1234);
                CallEprojPreInstr(if id < 6 { 8821317 } else { 8821466 }, k, game);
            }
            8..=11 => {
                slot(game, a::EPROJ_E, k, if id & 1 == 0 { 0 } else { 0x8000 });
                slot(game, a::EPROJ_X_POS, k, if id & 1 == 0 { 1 } else { 255 });
                CallEprojPreInstr(if id < 10 { 8822876 } else { 8822906 }, k, game);
            }
            12 => {
                put(game, w::SAMUS_Y_POS, 70);
                CallEprojPreInstr(8823592, k, game);
                CallEprojPreInstr(8823592, k, game);
                CallEprojPreInstr(8823652, k, game);
            }
            13..=14 => {
                slot(game, a::EPROJ_E, k, if id == 13 { 0 } else { 0x8000 });
                slot(game, a::EPROJ_INSTR_LIST_PTR, k, 0xfffe);
                CallEprojPreInstr(8826947, k, game);
            }
            15..=16 => {
                slot(game, a::EPROJ_E, k, 64);
                put(
                    game,
                    0xf78 + 64 + offset_of!(EnemyData, parameter_1),
                    if id == 15 { 0 } else { 0x8000 },
                );
                CallEprojPreInstr(8827405, k, game);
            }
            17..=18 => {
                slot(game, a::EPROJ_Y_POS, k, if id == 17 { 255 } else { 128 });
                slot(game, a::EPROJ_Y_VEL, k, 0x0100);
                CallEprojPreInstr(8829437, k, game);
            }
            19 => {
                slot(game, a::EPROJ_E, k, 2);
                CallEprojPreInstr(8833101, k, game);
                slot(game, a::EPROJ_Y_POS, k, 214);
                slot(game, a::EPROJ_Y_VEL, k, 0x0100);
                slot(game, a::EPROJ_Y_SUBPOS, k, 0);
                CallEprojPreInstr(8833158, k, game);
            }
            20 => {
                slot(game, a::EPROJ_X_POS, k, 100);
                slot(game, a::EPROJ_E, k, 77);
                put(game, w::NMI_FRAME_COUNTER_WORD, 1);
                CallEprojPreInstr(8837055, k, game);
                put(game, w::NMI_FRAME_COUNTER_WORD, 0);
                CallEprojPreInstr(8837055, k, game);
                CallEprojPreInstr(8837086, k, game);
            }
            21..=30 => {
                slot(game, a::EPROJ_E, k, 2 * (id - 21) as u16);
                slot(game, a::EPROJ_X_VEL, k, 0x0100);
                slot(game, a::EPROJ_Y_VEL, k, 0x0200);
                CallEprojPreInstr(8837595, k, game);
            }
            31..=34 => {
                slot(
                    game,
                    a::EPROJ_E,
                    k,
                    [0xdf40, 0xdf6a, 0xe050, 0xe07a][(id - 31) as usize],
                );
                slot(game, a::EPROJ_X_VEL, k, 0x0100);
                slot(game, a::EPROJ_Y_VEL, k, 0x0200);
                CallEprojPreInstr(if id < 33 { 8838969 } else { 8839241 }, k, game);
            }
            35..=36 => {
                slot(game, a::EPROJ_TIMERS, k, 0x0fff);
                slot(
                    game,
                    a::EPROJ_Y_VEL,
                    k,
                    if id == 35 { 0x0101 } else { 0xff01 },
                );
                slot(game, a::EPROJ_Y_POS, k, 100);
                slot(game, a::EPROJ_INSTR_LIST_PTR, k, 0xfffe);
                CallEprojPreInstr(8840709, k, game);
            }
            37..=40 => {
                slot(game, a::EPROJ_E, k, if id & 1 == 0 { 0x0120 } else { 0x20 });
                slot(game, a::EPROJ_F, k, if id & 1 == 0 { 0 } else { 1 });
                let pre = if id < 39 { 8822993 } else { 8823076 };
                CallEprojPreInstr(pre, k, game);
                CallEprojPreInstr(pre, k, game);
            }
            41..=42 => {
                slot(game, a::EPROJ_X_POS, k, 0x12ff);
                slot(game, a::EPROJ_Y_POS, k, 100);
                slot(game, a::EPROJ_X_VEL, k, 0x03ff);
                slot(game, a::EPROJ_Y_VEL, k, 0);
                slot(game, a::EPROJ_E, k, 0xfeff);
                slot(game, a::EPROJ_F, k, 0xffff);
                CallEprojPreInstr(8818965, k, game);
            }
            43 => {
                slot(game, a::EPROJ_E, k, 2);
                slot(game, a::EPROJ_F, k, 64);
                slot(game, a::EPROJ_G, k, 1);
                CallEprojPreInstr(8835274, k, game);
                CallEprojPreInstr(8835274, k, game);
            }
            _ => unreachable!(),
        }
    }
    None
}
