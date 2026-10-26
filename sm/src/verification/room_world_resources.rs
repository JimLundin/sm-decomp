//! Synthetic resource backing for the authored fixture on actual hardware.
//! Install only after real SnesInit in an isolated child. No cartridge data emitted.
#![forbid(unsafe_code)]
use crate::{game_state::Game, rom::RomImage};

fn write(image: &RomImage, address: u32, bytes: &[u8]) {
    let base = crate::rtl::lorom_offset(address) as usize;
    for (offset, &byte) in bytes.iter().enumerate() {
        image.write(base + offset, byte);
    }
}
fn fill_commands(count: usize, value: u8, out: &mut Vec<u8>) {
    let mut left = count;
    while left != 0 {
        let length = left.min(1024);
        let encoded = length - 1;
        out.extend_from_slice(&[0xe4 | ((encoded >> 8) as u8), encoded as u8, value]);
        left -= length;
    }
}
fn level(tile: u16) -> Vec<u8> {
    // 6x6 screens:18432 level bytes,9216 BTS,18432 layer2 plus2-byte size.
    // First tile distinguishes actual selected/install paths after decompression.
    let [low, high] = tile.to_le_bytes();
    let mut bytes = vec![3, 0, 0x48, low, high]; // Four literal bytes.
    fill_commands(46078, 0, &mut bytes);
    bytes.push(0xff);
    bytes
}
pub(crate) fn install(game: &mut Game) {
    assert!(!game.g_snes.is_null(), "room.world.hardware-initialized");
    let loaded = game.loaded_rom.clone().expect("real SnesInit loaded ROM");
    let native = game.capture_native_rom();
    for image in [&loaded, &native] {
        for (address, tile) in [(0xc08000, 0x0101), (0xc08100, 0x0202), (0xc08200, 0x0303)] {
            write(image, address, &level(tile));
        }
        for (address, count) in [(0xc08300, 6144), (0xc08400, 4096), (0xc08500, 512)] {
            let mut bytes = Vec::new();
            fill_commands(count, 0, &mut bytes);
            bytes.push(0xff);
            write(image, address, &bytes);
        }
        // Both retained enemy-list consumers stop onFFFF; quota follows population.
        write(image, 0xa19000, &[0xff, 0xff, 0]);
        write(image, 0xb49000, &[0xff, 0xff]);
    }
}
