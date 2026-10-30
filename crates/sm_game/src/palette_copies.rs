//! Palette copies from cartridge words, read through the native ROM image.
#![forbid(unsafe_code)]
use crate::cartridge::NativeDiagnostics;
use crate::cartridge::native_record;
use crate::ram::word;

/// Copy native-order words from `bank:source`, reading each before its store.
/// A cursor below $8000 reports the invalid-window diagnostic and still reads;
/// like the original do-until loop, a zero count does not stop.
fn copy<H: crate::ram::CompatibilityRam + NativeDiagnostics + ?Sized>(
    mut k: u16,
    bank: u8,
    mut source: u16,
    count: u16,
    game: &mut H,
    store: fn(&mut H, u16, u16),
) {
    let mut n = i32::from(count);
    loop {
        let value = native_record(bank, source, game).word(0);
        store(game, k >> 1, value);
        (k, source, n) = (k.wrapping_add(2), source.wrapping_add(2), n - 1);
        if n == 0 {
            break;
        }
    }
}
/// Was WriteColorsToPalette.
pub fn copy_rom_colors_to_palette(
    k: u16,
    bank: u8,
    source: u16,
    count: u16,
    game: &mut (impl crate::ram::CompatibilityRam + NativeDiagnostics + ?Sized),
) {
    copy(k, bank, source, count, game, |game, i, value| {
        word!(game.palette_buffer[i] = value)
    });
}
/// Was WriteColorsToTargetPalette.
pub fn copy_rom_colors_to_target_palette(
    bank: u8,
    k: u16,
    source: u16,
    count: u16,
    game: &mut (impl crate::ram::CompatibilityRam + NativeDiagnostics + ?Sized),
) {
    copy(k, bank, source, count, game, |game, i, value| {
        word!(game.target_palettes[i] = value)
    });
}
