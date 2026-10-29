//! Palette copies from cartridge words, read through the native ROM image.
#![forbid(unsafe_code)]
#![allow(non_snake_case)]
use crate::game::session_runtime::native_rom_diagnostic;
use crate::game_state::Game;
use crate::ram_map::word;
use crate::rom::lorom_offset;

/// Copy native-order words from `bank:source`, reading each before its store.
/// A cursor below $8000 reports the invalid-window diagnostic and still reads;
/// like the original do-until loop, a zero count does not stop.
fn copy(
    mut k: u16,
    bank: u8,
    mut source: u16,
    count: u16,
    game: &mut Game,
    store: fn(&mut Game, u16, u16),
) {
    let mut n = i32::from(count);
    loop {
        let address = (u32::from(bank) << 16) | u32::from(source);
        if address & 0x8000 == 0 {
            native_rom_diagnostic(address, game);
        }
        let (at, image) = (lorom_offset(address) as usize, game.capture_native_rom());
        store(
            game,
            k >> 1,
            u16::from_ne_bytes([image.read(at), image.read(at + 1)]),
        );
        (k, source, n) = (k.wrapping_add(2), source.wrapping_add(2), n - 1);
        if n == 0 {
            break;
        }
    }
}
pub(crate) fn WriteColorsToPalette(k: u16, bank: u8, source: u16, count: u16, game: &mut Game) {
    copy(k, bank, source, count, game, |game, i, value| {
        word!(game.palette_buffer[i] = value)
    });
}
pub(crate) fn WriteColorsToTargetPalette(
    bank: u8,
    k: u16,
    source: u16,
    count: u16,
    game: &mut Game,
) {
    copy(k, bank, source, count, game, |game, i, value| {
        word!(game.target_palettes[i] = value)
    });
}
