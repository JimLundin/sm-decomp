//! Dachora's palette rows: 32 bytes from bank $A7 to the slot's palette RAM.
#![forbid(unsafe_code)]
#![allow(non_snake_case)]
use crate::game::enemy_layouts::EnemyLayout8 as Dachora;
use crate::game::session_runtime::native_record;
use crate::game_state::Game;
use crate::ram_map::slot;
use crate::rtl::rom_table_word;

/// Palette RAM destinations, indexed by the slot's palette (high byte first).
const DESTINATIONS: u32 = 0xa7f55f;

pub(crate) fn Dachora_Func_1(j: u16, k: u16, game: &mut Game) {
    let palette = u16::swap_bytes(slot!(game, Dachora[i32::from(k)].base.palette_index));
    let destination = usize::try_from(
        rom_table_word!(game, DESTINATIONS, (palette >> 1) as isize) as i32 as isize
    )
    .expect("palette destination outside RAM");
    let source = native_record(0xa7, j, game);
    let bytes: [u8; 32] = core::array::from_fn(|i| source.byte(i));
    game.ram().copy_in(destination, &bytes);
}
