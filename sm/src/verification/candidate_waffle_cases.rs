//! Actual whole Waffle producer with synthetic sine/sprite content.
use super::{bytes, put};
use sm_game::actors::layouts::EnemyLayout72 as Layout;
fn call_nuclear_waffle_func(ea: u32, game: &mut Game) {
    assert_eq!(ea, 10917506);
    super::super::advance_fire_arc_orbit(game);
}
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use core::mem::offset_of;
pub(super) const CASES: u32 = 24;
fn field(game: &mut Game, enemy: usize, offset: usize, value: u16) {
    put(game, 0xf78 + enemy + offset, value);
}
pub(super) unsafe fn execute(id: u32, game: &mut Game, image: &mut [u8]) -> Option<usize> {
    assert!(id < CASES);
    let n = id as usize;
    let reverse = n / 6 % 2 != 0;
    let enemy = if n & 1 == 0 { 0 } else { 64 };
    put(game, w::CUR_ENEMY_INDEX, enemy as u16);
    let base = crate::rtl::lorom_offset(0xa0b143) as usize;
    for phase in 0..256 {
        bytes(image, base + phase, &[(phase * 37 + 11) as u8]);
    }
    let table = crate::rtl::lorom_offset(0xb4bda8) as usize;
    for selector in [44, 45, 46] {
        bytes(image, table + selector * 2, &0x9000u16.to_le_bytes());
    }
    bytes(
        image,
        crate::rtl::lorom_offset(0xb49000) as usize,
        &7u16.to_le_bytes(),
    );
    super::install(game, image);
    for slot in 0..32 {
        put(
            game,
            a::SPRITE_INSTR_LIST_PTRS + slot * 2,
            if n < 12 { 0 } else { 0x9100 },
        );
    }
    for (offset, value) in [
        (offset_of!(Layout, var_e), (n / 6 % 2) as u16),
        (
            offset_of!(Layout, var_d),
            if n & 1 == 0 { 0x00ff } else { 0xff80 },
        ),
        (offset_of!(Layout, var_20), 0xffff),
        (
            offset_of!(Layout, var_21),
            [0, 8, 16, 48, 80, 0xffff][n % 6],
        ),
        (offset_of!(Layout, var_23), if reverse { 8 } else { 80 }),
        (offset_of!(Layout, var_24), 1),
        (
            offset_of!(Layout, var_25),
            if n & 1 == 0 { 1 } else { 0xffff },
        ),
        (offset_of!(Layout, var_26), 0xfff0),
        (offset_of!(Layout, var_27), 0x7fff),
        (
            offset_of!(Layout, var_2b),
            if n & 1 == 0 { 4 } else { 0x10 },
        ),
        (offset_of!(Layout, var_2c), 8),
        (offset_of!(Layout, var_2e), if reverse { 80 } else { 8 }),
        (offset_of!(Layout, var_2f), if reverse { 48 } else { 16 }),
        (offset_of!(Layout, var_30), if reverse { 16 } else { 48 }),
        (
            offset_of!(Layout, var_33),
            if n & 1 == 0 { 0 } else { 0xffff },
        ),
        (offset_of!(Layout, var_34), 0xa00),
    ] {
        field(game, enemy, offset, value);
    }
    for child in [2, 4, 6, 8] {
        field(
            game,
            enemy + child,
            offset_of!(Layout, var_00),
            (child + 24) as u16,
        );
        field(
            game,
            enemy + child,
            offset_of!(Layout, var_08),
            if n & 1 == 0 { 0 } else { 0xffff },
        );
    }
    for child in [2, 4, 6] {
        field(
            game,
            enemy + child,
            offset_of!(Layout, var_10),
            child as u16,
        );
        field(
            game,
            enemy + child,
            offset_of!(Layout, var_18),
            if n & 1 == 0 { 0 } else { 0xffff },
        );
    }
    {
        call_nuclear_waffle_func(10917506, game)
    };
    None
}
