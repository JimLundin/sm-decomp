//! Options sprites: allocation, script execution and drawing in their original order.
#![forbid(unsafe_code)]
use super::menu_tilemap::native_record;
use super::prelude::*;
use crate::ram::word;

pub fn clear<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, descending, 8;
        le(options_object_spritemaps, 7) <- value(0),
        le(options_object_instr_lists, 7) <- value(0),
    );
}

pub fn spawn<H: Host + ?Sized>(parameter: u16, address: u16, game: &mut H) -> u8 {
    word!(game.options_object_init_param = parameter);
    let mut index = 14u16;
    while word!(game.options_object_instr_lists[index >> 1]) != 0 {
        index = index.wrapping_sub(2);
        if index & 0x8000 != 0 {
            return 1;
        }
    }
    let record = native_record(0x820000 | u32::from(address), game);
    let slot = index >> 1;
    word! { game; options_object_preinstrs[slot] = record.word(2);
    options_object_instr_lists[slot] = record.word(4); options_object_instr_timers[slot] = 1; }
    word! { game, 0;
        options_object_spritemaps[slot], options_object_timers[slot], options_object_unused_a[slot], options_object_unused_b[slot],
    }
    let entry = record.word(0);
    match entry {
        0xf296 | 0xf419 => {
            word! { game; options_object_x_pos[slot] = if entry == 0xf296 { 24 } else { 216 };
            options_object_y_pos[slot] = if entry == 0xf296 { 56 } else { 16 };
            options_object_palettes[slot] = 3584; }
        }
        0xf34b | 0xf353 | 0xf35b | 0xf363 => {
            let x: u16 = match entry {
                0xf34b => 124,
                0xf353 => 132,
                _ => 128,
            };
            let at = 0x1aad + 2 * usize::from(slot);
            game.ram().copy_in(at, &x.to_ne_bytes());
            word! { game; options_object_y_pos[slot] = 16; options_object_palettes[slot] = 3584; }
        }
        _ => {
            game.unreachable_native();
        }
    }
    0
}

pub fn advance_and_draw<H: Host + ?Sized>(game: &mut H) {
    let mut index = 14i32;
    while index >= 0 {
        word!(game.options_object_index = index as u16);
        if word!(game.options_object_instr_lists[index >> 1]) != 0 {
            advance(index as u16, game);
            index = i32::from(word!(game.options_object_index));
        }
        index -= 2;
    }
    for slot in (0..8).rev() {
        if word!(game.options_object_spritemaps[slot]) != 0 {
            crate::sprite_output::paletted(
                0x82,
                word!(game.options_object_spritemaps[slot]),
                (
                    word!(game.options_object_x_pos[slot]),
                    word!(game.options_object_y_pos[slot]),
                ),
                word!(game.options_object_palettes[slot]),
                crate::sprite_drawing::Clip::Bounded,
                game,
            );
        }
    }
}

fn end_script<H: Host + ?Sized>(slot: u16, game: &mut H) {
    word! { game; options_object_instr_timers[slot] = 1; options_object_instr_lists[slot] = 0xf4b6; }
}

fn before_script<H: Host + ?Sized>(slot: u16, game: &mut H) {
    match word!(game.options_object_preinstrs[slot]) {
        0xf2a9 => {
            if word!(game.game_state) == 2 {
                let screen = word!(game.game_options_screen_index);
                let table = crate::menu_lists::options_screen_objects(game, screen as isize);
                if table != 0 {
                    let address = table.wrapping_add(word!(game.menu_option_index).wrapping_mul(4));
                    let record = native_record(0x820000 | u32::from(address), game);
                    word! { game; options_object_x_pos[slot] = record.word(0);
                    options_object_y_pos[slot] = record.word(2); }
                } else {
                    word! { game; options_object_x_pos[slot] = 384; options_object_y_pos[slot] = 16; }
                }
            } else {
                end_script(slot, game);
            }
        }
        0xf376 => {
            if word!(game.game_state) != 2
                || word!(game.game_options_screen_index) == 6
                    && game.ram().read_byte(crate::ram::mirror::INIDISP) == 0x80
            {
                end_script(slot, game);
            }
        }
        0xf3a0 => match word!(game.game_options_screen_index) {
            6 if game.ram().read_byte(crate::ram::mirror::INIDISP) == 0x80 => {
                end_script(slot, game)
            }
            9 => {
                word!(game.options_object_y_pos[slot] -= 2);
            }
            10 => {
                word!(game.options_object_y_pos[slot] += 2);
            }
            _ => {}
        },
        0xf3e2 => {
            if word!(game.game_options_screen_index) == 6
                && game.ram().read_byte(crate::ram::mirror::INIDISP) == 0x80
            {
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
            game.unreachable_native();
        }
    }
}

fn instruction<H: Host + ?Sized>(opcode: u16, slot: u16, address: u16, game: &mut H) -> u16 {
    match opcode {
        0x8c5a => {
            word! { game, 0; options_object_spritemaps[slot], options_object_instr_lists[slot] }
            0
        }
        0x8c64 => {
            word!(game.options_object_instr_lists[slot] = address.wrapping_sub(2));
            0
        }
        0x8c6e | 0x8c93 => {
            let value = native_record(0x820000 | u32::from(address), game).word(0);
            if opcode == 0x8c6e {
                word!(game.options_object_preinstrs[slot] = value);
            } else {
                word!(game.options_object_timers[slot] = value);
            }
            address.wrapping_add(2)
        }
        0x8c79 => {
            word!(game.options_object_preinstrs[slot] = 0x8c81);
            address
        }
        0x8c82 => native_record(0x820000 | u32::from(address), game).word(0),
        0x8c89 => {
            let timer = word!(game.options_object_timers[slot]);
            word!(game.options_object_timers[slot] -= 1);
            if timer == 1 {
                address.wrapping_add(2)
            } else {
                native_record(0x820000 | u32::from(address), game).word(0)
            }
        }
        _ => game.unreachable_native(),
    }
}

fn advance<H: Host + ?Sized>(index: u16, game: &mut H) {
    before_script(index >> 1, game);
    let index = word!(game.options_object_index);
    let slot = word!(game.options_object_index) >> 1;
    let timer = word!(game.options_object_instr_timers[slot]);
    word!(game.options_object_instr_timers[slot] -= 1);
    if timer == 1 {
        let mut address = word!(game.options_object_instr_lists[slot]);
        loop {
            let record = native_record(0x820000 | u32::from(address), game);
            if record.word(0) & 0x8000 == 0 {
                let slot = index >> 1;
                word! { game; options_object_instr_timers[slot] = record.word(0);
                options_object_spritemaps[slot] = record.word(2);
                options_object_instr_lists[slot] = address.wrapping_add(4); }
                return;
            }
            address = instruction(record.word(0), index >> 1, address.wrapping_add(2), game);
            if address == 0 {
                return;
            }
        }
    }
}
