use super::{Game, decompress_level};
use crate::room_background_support::{ram_bytes, ram_span};

/// Synthetic regression for the actual decoder and room-array split.
/// Cases: overlapping large room, nonoverlap boundary, checked failure prefix.
#[forbid(unsafe_code)]
pub(crate) fn check_level_copy(case: u32) {
    use crate::rom::RomImage;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let size = match case {
        0 => 0x5a00usize,
        1 => 0x3c00,
        2 => 0x7000,
        _ => panic!("unknown room-copy case"),
    };
    let pattern =
        |i: usize, salt: u8| (i.wrapping_mul(73) as u8) ^ ((i >> 8) as u8).wrapping_mul(29) ^ salt;
    let mut game = Game::new();
    for i in 0..game.ram().len() {
        game.ram().write_byte(i, pattern(i, 0x5d));
    }
    let mut raw = (size as u16).to_le_bytes().to_vec();
    // The malformed case intentionally decompresses only its two-byte header.
    if case != 2 {
        raw.extend((0..size).map(|i| pattern(i, 0x11)));
        raw.extend((0..size / 2).map(|i| pattern(i, 0x47)));
        raw.extend((0..size).map(|i| pattern(i, 0xa3)));
    }
    let mut compressed = Vec::new();
    for chunk in raw.chunks(1024) {
        let length = chunk.len() - 1;
        compressed.extend_from_slice(&[0xe0 | (length >> 8) as u8, length as u8]);
        compressed.extend_from_slice(chunk);
    }
    compressed.push(0xff);
    compressed.resize(0x100000, 0);
    game.loaded_rom = Some(RomImage::from(compressed));
    game.ram().copy_in(0x7bd, &0x808000u32.to_ne_bytes());
    let mut expected = ram_bytes(&game);
    expected[0x10000..0x10000 + raw.len()].copy_from_slice(&raw);
    if case == 2 {
        // Source 1A802 reaches RAM's end after 57FE bytes; destination remains
        // in bounds. Forward copy retains exactly that successful prefix.
        let prefix = expected[0x1a802..].to_vec();
        expected[0x19602..0x19602 + prefix.len()].copy_from_slice(&prefix);
    } else {
        expected[0x19602..0x19602 + size].copy_from_slice(&raw[2 + size + size / 2..]);
        expected[0x16402..0x16402 + size / 2].copy_from_slice(&raw[2 + size..2 + size + size / 2]);
    }
    let source = u32::from_ne_bytes(ram_span::<4>(&game, 0x7bd)) & 0xffffff;
    let outcome = catch_unwind(AssertUnwindSafe(|| decompress_level(source, &mut game)));
    assert_eq!(
        outcome.is_err(),
        case == 2,
        "room copy case {case} panic status"
    );
    let first = (0..game.ram().len())
        .zip(&expected)
        .position(|(at, b)| game.ram().read_byte(at) != *b);
    assert!(
        first.is_none(),
        "room copy case {case}: first RAM mismatch {first:x?}"
    );
    assert!(!game.g_fail, "room copy case {case}: invalid ROM access");
}
