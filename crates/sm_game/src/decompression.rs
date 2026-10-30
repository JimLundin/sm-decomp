//! Shared byte decoder for room, graphics and tilemap assets.
#![forbid(unsafe_code)]
use crate::ram::RawRam as _;

pub use crate::room_entry::prepared_tables::{Output, decode};

/// What decompression reads and writes: raw work RAM, the original's source
/// cursor and the loaded image's bytes.
pub trait DecompressionHost: crate::ram::WorkRamAccess {
    /// The original's 24-bit source cursor.
    fn decompress_source(&self) -> u32;
    fn set_decompress_source(&mut self, source: u32);
    /// The original "RomPtr - Invalid access" report for `address`.
    fn report_invalid_rom_address(&mut self, address: u32);
    /// The loaded image's byte at LoROM `address` plus `offset`.
    fn read_rom_byte(&self, address: u32, offset: usize) -> u8;
}

pub fn next_byte<H: DecompressionHost + ?Sized>(game: &mut H) -> u8 {
    let address = game.decompress_source();
    if address & 0x8000 == 0 {
        game.report_invalid_rom_address(address);
    }
    // RomPtr aliases both halves of a bank; retain the unmasked source cursor.
    let value = game.read_rom_byte(address | 0x8000, 0);
    game.set_decompress_source(address.wrapping_add(1));
    if address & 0xffff == 0xffff {
        game.set_decompress_source(game.decompress_source().wrapping_add(0x8000));
    }
    value
}

struct RamOutput<'a, H: ?Sized> {
    game: &'a mut H,
    base: usize,
    produced: usize,
    prepared: Option<core::slice::Iter<'a, u8>>,
}
impl<H: DecompressionHost + ?Sized> Output for RamOutput<'_, H> {
    fn next_byte(&mut self) -> u8 {
        match self.prepared.as_mut() {
            Some(stream) => *stream.next().expect("validated prepared room stream"),
            None => next_byte(self.game),
        }
    }
    fn read(&mut self, position: i32) -> u8 {
        self.game
            .ram()
            .read_byte(self.base.checked_add_signed(position as isize).unwrap())
    }
    fn write(&mut self, position: i32, value: u8, _back_reference: bool) {
        self.game.ram().write_byte(
            self.base.checked_add_signed(position as isize).unwrap(),
            value,
        );
        self.produced = self
            .produced
            .max(usize::try_from(position).expect("RAM output position") + 1);
    }
}

pub fn decompress_to_ram<H: DecompressionHost + ?Sized>(
    source: u32,
    destination: usize,
    game: &mut H,
) -> usize {
    game.set_decompress_source(source);
    let mut output = RamOutput {
        game,
        base: destination,
        produced: 0,
        prepared: None,
    };
    decode(0, &mut output);
    output.produced
}

pub fn decompress_prepared_table<H: DecompressionHost + ?Sized>(
    stream: &[u8],
    destination: usize,
    game: &mut H,
) -> usize {
    let mut output = RamOutput {
        game,
        base: destination,
        produced: 0,
        prepared: Some(stream.iter()),
    };
    decode(0, &mut output);
    output.produced
}
