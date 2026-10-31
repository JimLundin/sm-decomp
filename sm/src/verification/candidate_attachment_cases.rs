//! Defined-old attachment observations; child of the core verification carrier.
//! Does not exercise Waffle or the intervening Botwoon enemy pre-instruction.
use super::{put, slot};
use sm_game::actors::layouts::EnemyLayout104 as Layout;
mod sm_a5 {
    pub(super) use super::super::super::update_spore_stem as SporeSpawn_Func_5;
}
mod sm_a8 {
    use crate::game_state::Game;
    pub(super) fn yapping_maw_func_12(game: &mut Game) {
        super::super::super::update_maw_segment(0, game);
    }
    pub(super) fn yapping_maw_func_13(game: &mut Game) {
        super::super::super::update_maw_segment(1, game);
    }
    pub(super) fn yapping_maw_func_14(game: &mut Game) {
        super::super::super::update_maw_segment(2, game);
    }
    pub(super) fn yapping_maw_func_15(game: &mut Game) {
        super::super::super::update_maw_segment(3, game);
    }
    pub(super) use super::super::super::update_maw_palette as YappingMaw_Func_18;
}
mod sm_b3 {
    pub(super) use super::super::super::{
        advance_botwoon_history as Botwoon_Func_24, follow_botwoon_history as Botwoon_Func_23,
        orient_botwoon_body as Botwoon_Func_25, record_botwoon_head as Botwoon_Func_22,
    };
}
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use core::mem::offset_of;

pub(super) const CASES: u32 = 48;
fn field(game: &mut Game, enemy: u16, offset: usize, value: u16) {
    put(game, 0xf78 + usize::from(enemy) + offset, value);
}
pub(super) unsafe fn execute(id: u32, game: &mut Game) -> Option<usize> {
    assert!(id < CASES);
    let variant = id as usize % 16;
    if id < 16 {
        let x = [0, 1, 0x7fff, 0x8000][variant % 4];
        let y = [0xffff, 0x8000, 1, 0][variant / 4];
        field(game, 0, offset_of!(Layout, base.x_pos), x);
        field(game, 0, offset_of!(Layout, base.y_pos), y);
        field(game, 0, offset_of!(Layout, var_04), 0xfff0);
        field(game, 0, offset_of!(Layout, var_05), 0x7ff0);
        // Spore intentionally ignores the current-enemy selector.
        put(game, w::CUR_ENEMY_INDEX, 64);
        {
            sm_a5::SporeSpawn_Func_5(game)
        };
    } else if id < 32 {
        let enemy = if variant & 1 == 0 { 0 } else { 64 };
        put(game, w::CUR_ENEMY_INDEX, enemy);
        for (offset, value) in [
            (offset_of!(Layout, var_0c), 0xfff0),
            (offset_of!(Layout, var_0d), 0x7fff),
            (offset_of!(Layout, var_00), 0x30),
            (offset_of!(Layout, var_01), 0x8001),
            (offset_of!(Layout, var_02), 0xffff),
            (offset_of!(Layout, var_03), 1),
            (offset_of!(Layout, var_04), 0x8000),
            (offset_of!(Layout, var_05), 0xffff),
            (offset_of!(Layout, var_45), 0xa55a),
            (
                offset_of!(Layout, base.frozen_timer),
                [0, 1, 2, 89, 90, 91, 0x8000, 0xffff][variant / 2],
            ),
        ] {
            field(game, enemy, offset, value);
        }
        let handles = if variant & 2 == 0 {
            [0, 1, 32, 34]
        } else {
            [34, 34, 0xffff, 0]
        };
        for (offset, handle) in [
            offset_of!(Layout, var_40),
            offset_of!(Layout, var_41),
            offset_of!(Layout, var_42),
            offset_of!(Layout, var_43),
        ]
        .into_iter()
        .zip(handles)
        {
            field(game, enemy, offset, handle);
            slot(game, a::EPROJ_GFX_IDX, handle, 0xf135);
        }
        {
            // Actual caller order, including collisions between repeated handles.
            sm_a8::yapping_maw_func_15(game);
            sm_a8::yapping_maw_func_14(game);
            sm_a8::yapping_maw_func_13(game);
            sm_a8::yapping_maw_func_12(game);
            for handle in handles {
                sm_a8::YappingMaw_Func_18(handle, game);
            }
        }
    } else {
        let enemy = if variant & 1 == 0 { 0 } else { 64 };
        let cursor: u16 = [0, 4, 0x3fc, 0x3fe][variant % 4];
        let spacing: u16 = [4, 0, 0x3fe, 0x400][variant / 4];
        put(game, w::CUR_ENEMY_INDEX, enemy);
        for at in (0..0x404).step_by(2) {
            put(
                game,
                0x9000 + at,
                (at as u16).wrapping_mul(37).wrapping_add(0xff00),
            );
        }
        for (offset, value) in [
            (offset_of!(Layout, base.x_pos), 0xfff0),
            (offset_of!(Layout, base.y_pos), 0x8010),
            (offset_of!(Layout, var_b), cursor),
            (offset_of!(Layout, var_c), spacing),
            (offset_of!(Layout, var_33), variant as u16 & 1),
            (
                offset_of!(Layout, var_36),
                if variant & 4 == 0 {
                    cursor.wrapping_sub(spacing.wrapping_mul(13)) & 0x3ff
                } else {
                    0xffff
                },
            ),
        ] {
            field(game, enemy, offset, value);
        }
        for n in (0..=24u16).step_by(2) {
            field(game, n, offset_of!(Layout, var_00), n);
            field(
                game,
                n,
                offset_of!(Layout, var_10),
                ((n / 2 + variant as u16) % 3) as u16,
            );
        }
        {
            sm_b3::Botwoon_Func_22(game);
            sm_b3::Botwoon_Func_23(game);
            sm_b3::Botwoon_Func_25(game);
            sm_b3::Botwoon_Func_24(game);
        }
    }
    None
}
