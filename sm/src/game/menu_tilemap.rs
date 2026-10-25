//! File-menu tilemaps: native word stores, live palette reads and ROM text controls.
use crate::game_state::Game;
use crate::ram_map::checked_word as word;

fn paint(offset: usize, tile: u16, game: &mut Game) {
    let at = 0xf78 + core::mem::offset_of!(crate::types::EnemyData, palette_index);
    let palette = u16::from_ne_bytes([game.g_ram[at], game.g_ram[at + 1]]);
    let at = 0x3600 + offset;
    game.g_ram[at..at + 2].copy_from_slice(&(palette | tile).to_ne_bytes());
}

fn text_word(address: u16, game: &mut Game) -> u16 {
    let address = 0x810000 | u32::from(address);
    // The warning can replace the native image; capture after it on every word.
    unsafe { crate::rtl::RomPtr(address, game) };
    game.capture_native_rom()
        .table_word(crate::rtl::lorom_offset(address) as usize, 0)
}

pub(crate) fn load(mut destination: u16, mut source: u16, game: &mut Game) {
    let mut row = i32::from(destination);
    loop {
        match text_word(source, game) {
            0xfffe => {
                row += 64;
                destination = row as u16;
            }
            0xffff => break,
            tile => {
                paint(usize::from(destination), tile, game);
                destination = destination.wrapping_add(2);
            }
        }
        source = source.wrapping_add(2);
    }
}

fn clear_row(offset: u16, game: &mut Game) {
    let count = 32 - usize::from((offset & 0x3f) >> 1);
    word_transfers!(game, ascending, count; native(0x3600 + usize::from(offset)) <- value(15));
}

fn number(offset: usize, value: u16, game: &mut Game) {
    paint(offset + 2, value % 10 + 8288, game);
    paint(offset, value / 10 + 8288, game);
}

pub(crate) fn health(empty: u16, offset: u16, game: &mut Game) {
    if empty != 0 {
        clear_row(offset, game);
        load(offset.wrapping_add(64), 0xb4ac, game);
        return;
    }
    load(offset, 0xb496, game);
    let mut full = word!(game.samus_health) / 100;
    let remainder = word!(game.samus_health) % 100;
    let maximum = word!(game.samus_max_health) / 100;
    let mut position = offset.wrapping_add(72);
    let mut columns = 7;
    for _ in 0..maximum {
        let tile = if full != 0 {
            full -= 1;
            152
        } else {
            153
        };
        paint(usize::from(position), tile, game);
        position = position.wrapping_add(2);
        columns -= 1;
        if columns == 0 {
            position = position.wrapping_sub(78);
            columns = 8;
        }
    }
    // Digit offsets stay linear; only the tank cursor wraps to sixteen bits.
    number(usize::from(offset) + 66, remainder, game);
}

pub(crate) fn time(empty: u16, offset: u16, game: &mut Game) {
    if empty != 0 {
        return;
    }
    number(usize::from(offset), word!(game.game_time_hours), game);
    load(offset.wrapping_add(4), 0xb4a8, game);
    number(usize::from(offset) + 6, word!(game.game_time_minutes), game);
}
