//! Options sprites: allocation, script execution and drawing in their original order.
#![forbid(unsafe_code)]
use crate::game::{
    menu_tilemap::native_record, projectile_presentation_ram as sprites, sprite_drawing::Clip,
};
use crate::game_state::Game;
use crate::ram_map::word;

pub(crate) fn clear(game: &mut Game) {
    word_transfers!(game, descending, 8;
        le(eproj_y_pos, 12) <- value(0),
        le(eproj_y_vel, 24) <- value(0),
    );
}

pub(crate) fn spawn(parameter: u16, address: u16, game: &mut Game) -> u8 {
    word!(game.options_menu_init_param = parameter);
    let mut index = 14u16;
    while word!(game.cinematicspr_arr7[index >> 1]) != 0 {
        index = index.wrapping_sub(2);
        if index & 0x8000 != 0 {
            return 1;
        }
    }
    let record = native_record(0x820000 | u32::from(address), game);
    let slot = index >> 1;
    word! { game; optionsmenu_arr1[slot] = record.word(2);
    cinematicspr_arr7[slot] = record.word(4); cinematicspr_instr_ptr[slot] = 1; }
    word! { game, 0;
        cinematicbg_arr8[slot], optionsmenu_arr5[slot], cinematicspr_arr6[slot], optionsmenu_arr7[slot],
    }
    let entry = record.word(0);
    match entry {
        0xf296 | 0xf419 => {
            word! { game; optionsmenu_arr8[slot] = if entry == 0xf296 { 24 } else { 216 };
            cinematicbg_arr9[slot] = if entry == 0xf296 { 56 } else { 16 };
            optionsmenu_arr10[slot] = 3584; }
        }
        0xf34b | 0xf353 | 0xf35b | 0xf363 => {
            let x: u16 = match entry {
                0xf34b => 124,
                0xf353 => 132,
                _ => 128,
            };
            let at = 0x1aad + 2 * usize::from(slot);
            game.g_ram[at..at + 2].copy_from_slice(&x.to_ne_bytes());
            word! { game; cinematicbg_arr9[slot] = 16; optionsmenu_arr10[slot] = 3584; }
        }
        _ => {
            crate::game::animated_tiles::native::unreachable(game);
        }
    }
    0
}

pub(crate) fn advance_and_draw(game: &mut Game) {
    let mut index = 14i32;
    while index >= 0 {
        word!(game.optionsmenu_index = index as u16);
        if word!(game.cinematicspr_arr7[index >> 1]) != 0 {
            advance(index as u16, game);
            index = i32::from(word!(game.optionsmenu_index));
        }
        index -= 2;
    }
    for slot in (0..8).rev() {
        if word!(game.cinematicbg_arr8[slot]) != 0 {
            sprites::paletted(
                0x82,
                word!(game.cinematicbg_arr8[slot]),
                (
                    word!(game.optionsmenu_arr8[slot]),
                    word!(game.cinematicbg_arr9[slot]),
                ),
                word!(game.optionsmenu_arr10[slot]),
                Clip::Bounded,
                game,
            );
        }
    }
}

fn end_script(slot: u16, game: &mut Game) {
    word! { game; cinematicspr_instr_ptr[slot] = 1; cinematicspr_arr7[slot] = 0xf4b6; }
}

fn before_script(slot: u16, game: &mut Game) {
    match word!(game.optionsmenu_arr1[slot]) {
        0xf2a9 => {
            if word!(game.game_state) == 2 {
                let screen = word!(game.game_options_screen_index);
                let table = crate::rtl::rom_table_word!(game, 0x82f2ed, screen as isize);
                if table != 0 {
                    let address = table.wrapping_add(word!(game.menu_option_index).wrapping_mul(4));
                    let record = native_record(0x820000 | u32::from(address), game);
                    word! { game; optionsmenu_arr8[slot] = record.word(0);
                    cinematicbg_arr9[slot] = record.word(2); }
                } else {
                    word! { game; optionsmenu_arr8[slot] = 384; cinematicbg_arr9[slot] = 16; }
                }
            } else {
                end_script(slot, game);
            }
        }
        0xf376 => {
            if word!(game.game_state) != 2
                || word!(game.game_options_screen_index) == 6 && game.g_ram[0x51] == 0x80
            {
                end_script(slot, game);
            }
        }
        0xf3a0 => match word!(game.game_options_screen_index) {
            6 if game.g_ram[0x51] == 0x80 => end_script(slot, game),
            9 => {
                word!(game.cinematicbg_arr9[slot] -= 2);
            }
            10 => {
                word!(game.cinematicbg_arr9[slot] += 2);
            }
            _ => {}
        },
        0xf3e2 => {
            if word!(game.game_options_screen_index) == 6 && game.g_ram[0x51] == 0x80 {
                end_script(slot, game);
            }
        }
        0xf42c => {
            if word!(game.game_state) != 2 {
                end_script(slot, game);
            }
        }
        0x8c10 => {}
        _ => {
            crate::game::animated_tiles::native::unreachable(game);
        }
    }
}

fn instruction(opcode: u16, slot: u16, address: u16, game: &mut Game) -> u16 {
    match opcode {
        0x8c5a => {
            word! { game, 0; cinematicbg_arr8[slot], cinematicspr_arr7[slot] }
            0
        }
        0x8c64 => {
            word!(game.cinematicspr_arr7[slot] = address.wrapping_sub(2));
            0
        }
        0x8c6e | 0x8c93 => {
            let value = native_record(0x820000 | u32::from(address), game).word(0);
            if opcode == 0x8c6e {
                word!(game.optionsmenu_arr1[slot] = value);
            } else {
                word!(game.optionsmenu_arr5[slot] = value);
            }
            address.wrapping_add(2)
        }
        0x8c79 => {
            word!(game.optionsmenu_arr1[slot] = 0x8c81);
            address
        }
        0x8c82 => native_record(0x820000 | u32::from(address), game).word(0),
        0x8c89 => {
            let timer = word!(game.optionsmenu_arr5[slot]);
            word!(game.optionsmenu_arr5[slot] -= 1);
            if timer == 1 {
                address.wrapping_add(2)
            } else {
                native_record(0x820000 | u32::from(address), game).word(0)
            }
        }
        _ => crate::game::animated_tiles::native::unreachable(game),
    }
}

fn advance(index: u16, game: &mut Game) {
    before_script(index >> 1, game);
    let index = word!(game.optionsmenu_index);
    let slot = word!(game.optionsmenu_index) >> 1;
    let timer = word!(game.cinematicspr_instr_ptr[slot]);
    word!(game.cinematicspr_instr_ptr[slot] -= 1);
    if timer == 1 {
        let mut address = word!(game.cinematicspr_arr7[slot]);
        loop {
            let record = native_record(0x820000 | u32::from(address), game);
            if record.word(0) & 0x8000 == 0 {
                let slot = index >> 1;
                word! { game; cinematicspr_instr_ptr[slot] = record.word(0);
                cinematicbg_arr8[slot] = record.word(2);
                cinematicspr_arr7[slot] = address.wrapping_add(4); }
                return;
            }
            address = instruction(record.word(0), index >> 1, address.wrapping_add(2), game);
            if address == 0 {
                return;
            }
        }
    }
}
