//! VRAM, CGRAM and OAM with the PPU's port semantics ($2101-$2104,
//! $2115-$2119, $2121-$2122 and the reads $2138-$213B).
//!
//! `Ports` holds the address and latch state; the memory itself is behind
//! `VideoMemory`, so the same port logic drives owned memory or a checked view.
//! VRAM addresses are words, CGRAM entries are BGR555 words and OAM is the
//! 256-word low table plus the 32-byte high table.

pub trait VideoMemory {
    fn vram(&self, index: u16) -> u16;
    fn set_vram(&mut self, index: u16, value: u16);
    fn cgram(&self, index: u8) -> u16;
    fn set_cgram(&mut self, index: u8, value: u16);
    fn oam(&self, index: u8) -> u16;
    fn set_oam(&mut self, index: u8, value: u16);
    fn high_oam(&self, index: u8) -> u8;
    fn set_high_oam(&mut self, index: u8, value: u8);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedMemory {
    pub vram: Box<[u16; 0x8000]>,
    pub cgram: [u16; 256],
    pub oam: [u16; 256],
    pub high_oam: [u8; 32],
}

impl Default for OwnedMemory {
    fn default() -> Self {
        Self {
            vram: Box::new([0; 0x8000]),
            cgram: [0; 256],
            oam: [0; 256],
            high_oam: [0; 32],
        }
    }
}

impl VideoMemory for OwnedMemory {
    fn vram(&self, index: u16) -> u16 {
        self.vram[usize::from(index & 0x7fff)]
    }
    fn set_vram(&mut self, index: u16, value: u16) {
        self.vram[usize::from(index & 0x7fff)] = value;
    }
    fn cgram(&self, index: u8) -> u16 {
        self.cgram[usize::from(index)]
    }
    fn set_cgram(&mut self, index: u8, value: u16) {
        self.cgram[usize::from(index)] = value;
    }
    fn oam(&self, index: u8) -> u16 {
        self.oam[usize::from(index)]
    }
    fn set_oam(&mut self, index: u8, value: u16) {
        self.oam[usize::from(index)] = value;
    }
    fn high_oam(&self, index: u8) -> u8 {
        self.high_oam[usize::from(index & 31)]
    }
    fn set_high_oam(&mut self, index: u8, value: u8) {
        self.high_oam[usize::from(index & 31)] = value;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ports {
    /// OBSEL ($2101): the sprite size pair and the two sprite tile bases.
    pub sprite_size: u8,
    pub sprite_tile_words: [u16; 2],
    /// OAMADDH bit 7: priority rotation starts at the OAM address.
    pub sprite_priority: bool,
    pub oam_address: u8,
    pub oam_address_written: u8,
    pub oam_high: bool,
    pub oam_high_written: bool,
    pub oam_second: bool,
    pub oam_buffer: u8,
    pub vram_pointer: u16,
    pub vram_increment: u16,
    pub vram_remap: u8,
    pub vram_increment_on_high: bool,
    pub vram_read_buffer: u16,
    pub cgram_pointer: u8,
    pub cgram_second: bool,
    pub cgram_buffer: u8,
}

impl Ports {
    /// The VRAM word the pointer selects after VMAIN's address translation.
    pub fn vram_address(&self) -> u16 {
        let address = self.vram_pointer;
        match self.vram_remap {
            1 => (address & 0xff00) | ((address & 0xe0) >> 5) | ((address & 0x1f) << 3),
            2 => (address & 0xfe00) | ((address & 0x1c0) >> 6) | ((address & 0x3f) << 3),
            3 => (address & 0xfc00) | ((address & 0x380) >> 7) | ((address & 0x7f) << 3),
            _ => address,
        }
    }

    fn prefetch(&mut self, memory: &impl VideoMemory) {
        self.vram_read_buffer = memory.vram(self.vram_address() & 0x7fff);
    }

    /// Advance the OAM port after a byte at the high table or a low-table pair.
    fn step_oam(&mut self) {
        if self.oam_second {
            self.oam_address = self.oam_address.wrapping_add(1);
            if self.oam_address == 0 {
                self.oam_high = !self.oam_high;
            }
        }
    }

    /// Apply one write to $21xx, `address` being its low byte. Registers this
    /// model doesn't own are ignored.
    pub fn write(&mut self, memory: &mut impl VideoMemory, address: u8, value: u8) {
        match address {
            0x01 => {
                self.sprite_size = value >> 5;
                let first = u16::from(value & 7) << 13;
                self.sprite_tile_words = [
                    first,
                    first.wrapping_add(u16::from((value & 0x18) + 8) << 9),
                ];
            }
            0x02 => {
                self.oam_address = value;
                self.oam_address_written = value;
                self.oam_high = self.oam_high_written;
                self.oam_second = false;
            }
            0x03 => {
                self.sprite_priority = value & 0x80 != 0;
                self.oam_high = value & 1 != 0;
                self.oam_high_written = self.oam_high;
                self.oam_address = self.oam_address_written;
                self.oam_second = false;
            }
            0x04 => {
                if self.oam_high {
                    let index = ((self.oam_address & 15) << 1) | u8::from(self.oam_second);
                    memory.set_high_oam(index, value);
                } else if self.oam_second {
                    memory.set_oam(
                        self.oam_address,
                        u16::from_le_bytes([self.oam_buffer, value]),
                    );
                } else {
                    self.oam_buffer = value;
                }
                self.step_oam();
                self.oam_second = !self.oam_second;
            }
            0x15 => {
                self.vram_increment = match value & 3 {
                    0 => 1,
                    1 => 32,
                    _ => 128,
                };
                self.vram_remap = (value & 0x0c) >> 2;
                self.vram_increment_on_high = value & 0x80 != 0;
            }
            0x16 | 0x17 => {
                let mut bytes = self.vram_pointer.to_le_bytes();
                bytes[usize::from(address - 0x16)] = value;
                self.vram_pointer = u16::from_le_bytes(bytes);
                self.prefetch(memory);
            }
            0x18 | 0x19 => {
                let target = self.vram_address() & 0x7fff;
                let mut bytes = memory.vram(target).to_le_bytes();
                let high = address == 0x19;
                bytes[usize::from(high)] = value;
                memory.set_vram(target, u16::from_le_bytes(bytes));
                if high == self.vram_increment_on_high {
                    self.vram_pointer = self.vram_pointer.wrapping_add(self.vram_increment);
                }
            }
            0x21 => {
                self.cgram_pointer = value;
                self.cgram_second = false;
            }
            0x22 => {
                if self.cgram_second {
                    memory.set_cgram(
                        self.cgram_pointer,
                        u16::from_le_bytes([self.cgram_buffer, value]),
                    );
                    self.cgram_pointer = self.cgram_pointer.wrapping_add(1);
                } else {
                    self.cgram_buffer = value;
                }
                self.cgram_second = !self.cgram_second;
            }
            _ => {}
        }
    }

    /// Read a data port ($2138-$213B). `open_bus` is the PPU2 open-bus byte,
    /// which supplies bit 7 of a CGRAM high byte. Other addresses return None.
    pub fn read(&mut self, memory: &impl VideoMemory, address: u8, open_bus: u8) -> Option<u8> {
        let value = match address {
            0x38 => {
                let value = if self.oam_high {
                    memory.high_oam(((self.oam_address & 15) << 1) | u8::from(self.oam_second))
                } else {
                    memory.oam(self.oam_address).to_le_bytes()[usize::from(self.oam_second)]
                };
                self.step_oam();
                self.oam_second = !self.oam_second;
                value
            }
            0x39 | 0x3a => {
                let high = address == 0x3a;
                let value = self.vram_read_buffer.to_le_bytes()[usize::from(high)];
                if high == self.vram_increment_on_high {
                    self.prefetch(memory);
                    self.vram_pointer = self.vram_pointer.wrapping_add(self.vram_increment);
                }
                value
            }
            0x3b => {
                let word = memory.cgram(self.cgram_pointer);
                let value = if self.cgram_second {
                    self.cgram_pointer = self.cgram_pointer.wrapping_add(1);
                    (word.to_le_bytes()[1] & 0x7f) | (open_bus & 0x80)
                } else {
                    word.to_le_bytes()[0]
                };
                self.cgram_second = !self.cgram_second;
                value
            }
            _ => return None,
        };
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(writes: &[(u8, u8)]) -> (Ports, OwnedMemory) {
        let mut ports = Ports::default();
        let mut memory = OwnedMemory::default();
        for &(address, value) in writes {
            ports.write(&mut memory, address, value);
        }
        (ports, memory)
    }

    #[test]
    fn vram_writes_follow_the_increment_byte() {
        // Increment after the high byte by 32 words, from word $1234.
        let (ports, memory) = run(&[
            (0x15, 0x81),
            (0x16, 0x34),
            (0x17, 0x12),
            (0x18, 0xcd),
            (0x19, 0xab),
        ]);
        assert_eq!(memory.vram[0x1234], 0xabcd);
        assert_eq!(ports.vram_pointer, 0x1254);
        // Increment after the low byte.
        let (ports, memory) = run(&[(0x15, 0x00), (0x18, 0x11), (0x18, 0x22)]);
        assert_eq!(
            (memory.vram[0], memory.vram[1], ports.vram_pointer),
            (0x11, 0x22, 2)
        );
    }

    #[test]
    fn vram_remap_rotates_the_low_bits() {
        let mut ports = Ports {
            vram_pointer: 0x0021,
            vram_remap: 1,
            ..Ports::default()
        };
        // Mode 1: aaaaaaaa BBBccccc becomes aaaaaaaa cccccBBB.
        assert_eq!(ports.vram_address(), 0x0009);
        ports.vram_remap = 0;
        assert_eq!(ports.vram_address(), 0x0021);
    }

    #[test]
    fn vram_reads_return_the_prefetched_word() {
        let mut memory = OwnedMemory::default();
        memory.vram[5] = 0xbeef;
        memory.vram[6] = 0xcafe;
        let mut ports = Ports::default();
        ports.write(&mut memory, 0x15, 0x80);
        ports.write(&mut memory, 0x16, 5);
        assert_eq!(ports.read(&memory, 0x39, 0), Some(0xef));
        // The high read refills the buffer from the current word, then increments.
        assert_eq!(ports.read(&memory, 0x3a, 0), Some(0xbe));
        assert_eq!(ports.vram_pointer, 6);
        assert_eq!(ports.read(&memory, 0x39, 0), Some(0xef));
        assert_eq!(ports.read(&memory, 0x3a, 0), Some(0xbe));
        assert_eq!(ports.read(&memory, 0x39, 0), Some(0xfe));
    }

    #[test]
    fn cgram_pairs_bytes_and_masks_the_high_read() {
        let (mut ports, memory) = run(&[(0x21, 0x10), (0x22, 0x34), (0x22, 0xf2), (0x21, 0x10)]);
        assert_eq!(memory.cgram[0x10], 0xf234);
        assert_eq!(ports.read(&memory, 0x3b, 0x80), Some(0x34));
        assert_eq!(ports.read(&memory, 0x3b, 0x80), Some(0xf2));
        assert_eq!(ports.read(&memory, 0x3b, 0x00), Some(0x00));
    }

    #[test]
    fn oam_low_table_wraps_into_the_high_table() {
        let mut writes = vec![(0x02, 0xff), (0x03, 0x00), (0x04, 0x01), (0x04, 0x02)];
        writes.extend([(0x04, 0x77), (0x04, 0x88)]);
        let (ports, memory) = run(&writes);
        assert_eq!(memory.oam[0xff], 0x0201);
        assert!(ports.oam_high);
        assert_eq!(&memory.high_oam[..2], &[0x77, 0x88]);
        // Re-setting the address returns to the written table and address.
        let (ports, _) = run(&[(0x03, 0x81), (0x04, 0x01)]);
        assert!(ports.sprite_priority && ports.oam_high && ports.oam_second);
    }

    #[test]
    fn obsel_sets_size_and_tile_bases() {
        let (ports, _) = run(&[(0x01, 0x63)]);
        assert_eq!(ports.sprite_size, 3);
        assert_eq!(ports.sprite_tile_words, [0x6000, 0x7000]);
    }
}
