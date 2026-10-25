//! File-menu tilemaps: native word stores, live palette reads and ROM text controls.
use crate::game_state::Game;
use crate::ram_map::checked_word as word;

/// Requires a live, exclusively accessed SNES backend.
pub(crate) unsafe fn prepare_graphics(game: &mut Game) {
    for (register, value) in [
        (0x6b, 0),
        (0x52, 3),
        (0x58, 81),
        (0x59, 88),
        (0x5a, 92),
        (0x5d, 0),
        (0x5e, 4),
        (0x69, 19),
    ] {
        game.g_ram[register] = value;
    }
    for (source, destination, bytes) in [
        (0x8e8000, 0, 0x5600),
        (0xb68000, 0x3000, 0x2000),
        (0xb6c000, 0x6000, 0x2000),
        (0x8ed600, 0x4000, 0x600),
    ] {
        unsafe {
            crate::game::sm_80::GraphicsAsset::VramWords(source, destination, bytes).load(game)
        };
    }
}

pub(crate) fn load_palette(game: &mut Game) {
    word_transfers!(game, (4), 128;
        le(palette_buffer, 0) <- rom(0x8ee400, 0),
        le(palette_buffer, 1) <- rom(0x8ee400, 1),
    );
}

pub(crate) fn upload(size: u16, source: u32, screen: usize, bank_word: bool, game: &mut Game) {
    let tail = word!(game.vram_write_queue_tail);
    let entry = 0xd0 + usize::from(tail);
    game.g_ram[entry..entry + 2].copy_from_slice(&size.to_ne_bytes());
    game.g_ram[entry + 2..entry + 4].copy_from_slice(&(source as u16).to_ne_bytes());
    let bank = (source >> 16) as u8;
    if bank_word {
        game.g_ram[entry + 4..entry + 6].copy_from_slice(&u16::from(bank).to_ne_bytes());
    } else {
        game.g_ram[entry + 4] = bank;
    }
    let destination = u16::from(game.g_ram[0x58 + screen] & 0xfc) << 8;
    game.g_ram[entry + 5..entry + 7].copy_from_slice(&destination.to_ne_bytes());
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
}

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
