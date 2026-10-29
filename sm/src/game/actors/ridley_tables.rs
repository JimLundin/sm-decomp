//! Ridley's table reads from bank $A6: tail-speed picks and RAM word fans.
#![forbid(unsafe_code)]
#![allow(non_snake_case)]
use crate::game::enemy_layouts::EnemyLayout104 as Ridley;
use crate::game::session_runtime::native_record;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rtl::rom_table_word;

/// Pick a random speed row: two row pointers by `random & 3`, the scroll pair
/// by Ridley's step, then var_C from the first row and a signed var_B from the
/// second, each through its own native fetch.
pub(crate) fn Ridley_Func_29(game: &mut Game) {
    let pick = (word!(game.random_number) & 3) as isize;
    let r18 = rom_table_word!(game, 0xa6b965, pick);
    let r20 = rom_table_word!(game, 0xa6b96d, pick);
    let offset = (2 * (i32::from(slot!(game, Ridley[0].var_12)) + 2)) as u16;
    let step = (offset >> 1) as isize;
    word!(game.tilemap_stuff[7] = rom_table_word!(game, 0xa6b94d, step));
    word!(game.tilemap_stuff[6] = rom_table_word!(game, 0xa6b959, step));
    slot!(
        game,
        Ridley[0].var_C = native_record(0xa6, r20, game).word(usize::from(offset))
    );
    let t = native_record(0xa6, r18, game).word(usize::from(offset));
    slot!(
        game,
        Ridley[0].var_B = (if slot!(game, Ridley[0].var_B) & 0x8000 != 0 {
            -i32::from(t)
        } else {
            i32::from(t)
        }) as u16
    );
}

/// Copy seven native words from bank $A6 at `source` to RAM, 20 bytes apart.
pub(crate) fn Ridley_Func_93(destination: u16, source: u16, game: &mut Game) {
    let words = native_record(0xa6, source, game);
    for i in 0..7 {
        let bytes = words.word(2 * i).to_ne_bytes();
        let at = usize::from(destination) + 20 * i;
        game.ram().write_byte(at, bytes[0]);
        game.ram().write_byte(at + 1, bytes[1]);
    }
}
