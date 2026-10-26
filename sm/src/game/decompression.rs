//! Shared byte decoder for room, graphics and tilemap assets.
#![forbid(unsafe_code)]
use crate::game_state::Game;

// Output implementations retain their byte/write protocol. VRAM also needs
// the back-reference flag to preserve its address-register writes.
pub(super) trait Output {
    fn next_byte(&mut self) -> u8;
    fn read(&mut self, position: i32) -> u8;
    fn write(&mut self, position: i32, value: u8, back_reference: bool);
}

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
}
impl Output for RamOutput<'_> {
    fn next_byte(&mut self) -> u8 {
        next_byte(self.game)
    }
    fn read(&mut self, position: i32) -> u8 {
        self.game.g_ram[self.base.checked_add_signed(position as isize).unwrap()]
    }
    fn write(&mut self, position: i32, value: u8, _back_reference: bool) {
        self.game.g_ram[self.base.checked_add_signed(position as isize).unwrap()] = value;
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
    };
    decode(0, &mut output);
    output.produced
}

pub(super) fn decode(start: i32, output: &mut impl Output) {
    let mut destination = start;
    loop {
        let header = output.next_byte();
        if header == 0xff {
            return;
        }
        let (command, length) = if header & 0xe0 == 0xe0 {
            (
                (header << 3) & 0xe0,
                (i32::from(header & 3) << 8 | i32::from(output.next_byte())) + 1,
            )
        } else {
            (header & 0xe0, i32::from(header & 0x1f) + 1)
        };
        // Read each command's operands before its first output byte. Absolute
        // references are relative to the initial destination, even in VRAM.
        let (mut source, first, second) = match command {
            0x20 | 0x60 => (0, output.next_byte(), 0),
            0x40 => (0, output.next_byte(), output.next_byte()),
            0x80 | 0xa0 => {
                let low = i32::from(output.next_byte());
                (low + i32::from(output.next_byte()) * 256 + start, 0, 0)
            }
            0xc0 | 0xe0 => (destination - i32::from(output.next_byte()), 0, 0),
            _ => (0, 0, 0),
        };
        for index in 0..length {
            let value = match command {
                0x20 => first,
                0x40 => {
                    if index & 1 == 0 {
                        first
                    } else {
                        second
                    }
                }
                0x60 => first.wrapping_add(index as u8),
                0x80 | 0xa0 | 0xc0 | 0xe0 => {
                    let value = output.read(source) ^ if command & 0x20 != 0 { 0xff } else { 0 };
                    source += 1;
                    value
                }
                _ => output.next_byte(),
            };
            // Read and write each reference byte separately: overlapping copies
            // propagate earlier writes instead of copying a frozen source range.
            output.write(destination, value, command & 0x80 != 0);
            destination += 1;
        }
    }
}
