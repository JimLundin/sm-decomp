//! Shared byte decoder for room, graphics and tilemap assets.
#![forbid(unsafe_code)]
use crate::game_state::Game;

pub(super) use super::room_entry::prepared_tables::{Output, decode};

pub(super) fn next_byte(game: &mut Game) -> u8 {
    let address = game.decompress_src;
    if address & 0x8000 == 0 {
        crate::rtl::report_invalid_rom_address(address, game);
    }
    // RomPtr aliases both halves of a bank; retain the unmasked source cursor.
    let value = game.read_rom_byte(address | 0x8000, 0);
    game.decompress_src = address.wrapping_add(1);
    if address & 0xffff == 0xffff {
        game.decompress_src = game.decompress_src.wrapping_add(0x8000);
    }
    value
}

struct RamOutput<'a> {
    game: &'a mut Game,
    base: usize,
    produced: usize,
    prepared: Option<core::slice::Iter<'a, u8>>,
}
impl Output for RamOutput<'_> {
    fn next_byte(&mut self) -> u8 {
        match self.prepared.as_mut() {
            Some(stream) => *stream.next().expect("validated prepared table stream"),
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

pub(crate) fn decompress_to_ram(source: u32, destination: usize, game: &mut Game) -> usize {
    game.decompress_src = source;
    let mut output = RamOutput {
        game,
        base: destination,
        produced: 0,
        prepared: None,
    };
    decode(0, &mut output);
    output.produced
}

pub(crate) fn decompress_prepared_table(
    stream: &[u8],
    destination: usize,
    game: &mut Game,
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
