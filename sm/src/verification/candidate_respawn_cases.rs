//! Defined-old drop respawn guard and population restoration; synthetic definitions.
use super::{bytes, put, slot};
fn respawn_from_drop(k: u16, game: &mut Game) {
    super::super::with_execution(game, |ctx| super::super::drops::respawn(k, ctx));
}
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use crate::rom::RomImage;
use crate::types::EnemyDef;
use core::mem::offset_of;
pub(super) const CASES: u32 = 8;
pub(super) unsafe fn execute(id: u32, game: &mut Game, image: &RomImage) -> Option<usize> {
    assert!(id < CASES);
    let marker = [0u16, 1, 0x7fff, 0xffff, 0x8000, 0x8040, 0x8001, 0xfffe][id as usize];
    let projectile = if id & 1 == 0 { 0 } else { 34 };
    slot(game, a::EPROJ_KILLED_ENEMY_INDEX, projectile, marker);
    put(game, w::ROOM_ENEMY_POPULATION_PTR, 0x8000);
    let index = marker & 0x7fff;
    let population =
        crate::rtl::lorom_offset(0xa10000 | u32::from(0x8000u16.wrapping_add(index >> 2))) as usize;
    for (n, value) in [
        0x9000u16, 0x1234, 0xfff0, 0x8000, 0x400, 0x35, 0x6789, 0xabcd,
    ]
    .into_iter()
    .enumerate()
    {
        bytes(image, population + n * 2, &value.to_le_bytes());
    }
    let definition = crate::rtl::lorom_offset(0xa09000) as usize;
    for (offset, value) in [
        (offset_of!(EnemyDef, x_radius), 13u16),
        (offset_of!(EnemyDef, y_radius), 17),
        (offset_of!(EnemyDef, health), 1234),
        (offset_of!(EnemyDef, bank), 0x07a2),
        (offset_of!(EnemyDef, ai_init), 0x804c), // Actual CallEnemyAi no-op identity A2804C.
    ] {
        bytes(image, definition + offset, &value.to_le_bytes());
    }
    bytes(image, definition + offset_of!(EnemyDef, layer), &[3]);
    {
        respawn_from_drop(projectile, game)
    };
    None
}
