//! The original SRAM save format: three files copied word by word from the
//! player RAM block, each guarded by a word checksum stored twice with its
//! inverse, and a remembered menu selection.

/// The work RAM a save file holds: $7E:D7C0 up to $7E:DE1C.
pub const PLAYER_BLOCK: core::ops::Range<usize> = 0xd7c0..0xde1c;

/// Work RAM and SRAM as the save format reads and writes them.
pub trait SaveHost {
    /// A native-order word of work RAM.
    fn ram_word(&self, at: usize) -> u16;
    fn set_ram_word(&mut self, at: usize, value: u16);
    fn sram_word(&self, at: usize) -> u16;
    fn set_sram_word(&mut self, at: usize, value: u16);
    /// File `slot`'s SRAM byte offset, as the $81:812B table gives it.
    fn slot_offset(&self, slot: isize) -> u16;
}

#[derive(Clone, Copy)]
enum Direction {
    Save,
    Load,
}

/// Copy the player block one word at a time, each word read then written
/// before the next, and return the words' wrapping sum.
fn transfer(host: &mut (impl SaveHost + ?Sized), slot: u16, direction: Direction) -> u16 {
    let mut cursor = host.slot_offset((slot & 3) as isize);
    let mut checksum = 0u16;
    for at in PLAYER_BLOCK.step_by(2) {
        let sram = usize::from(cursor & !1);
        let value = match direction {
            Direction::Save => host.ram_word(at),
            Direction::Load => host.sram_word(sram),
        };
        match direction {
            Direction::Save => host.set_sram_word(sram, value),
            Direction::Load => host.set_ram_word(at, value),
        }
        checksum = checksum.wrapping_add(value);
        cursor = cursor.wrapping_add(2);
    }
    checksum
}

/// Write the player block into file `slot`; returns its checksum.
pub fn save(host: &mut (impl SaveHost + ?Sized), slot: u16) -> u16 {
    transfer(host, slot, Direction::Save)
}

/// Read file `slot` into the player block; returns the checksum of what was read.
pub fn load(host: &mut (impl SaveHost + ?Sized), slot: u16) -> u16 {
    transfer(host, slot, Direction::Load)
}

/// Store `checksum` and its inverse in both header copies, in the original order.
pub fn write_checksum(host: &mut (impl SaveHost + ?Sized), slot: u16, checksum: u16) {
    let header = 2 * usize::from(slot & 3);
    host.set_sram_word(header, checksum);
    host.set_sram_word(header + 0x1ff0, checksum);
    host.set_sram_word(header + 8, !checksum);
    host.set_sram_word(header + 0x1ff8, !checksum);
}

/// Whether either header copy holds `checksum` and its inverse.
pub fn checksum_matches(host: &(impl SaveHost + ?Sized), slot: u16, checksum: u16) -> bool {
    let header = 2 * usize::from(slot & 3);
    checksum == host.sram_word(header) && !checksum == host.sram_word(header + 8)
        || checksum == host.sram_word(header + 0x1ff0)
            && !checksum == host.sram_word(header + 0x1ff8)
}

/// Zero file `slot`'s words.
pub fn clear(host: &mut (impl SaveHost + ?Sized), slot: u16) {
    let mut cursor = host.slot_offset((slot & 3) as isize);
    for _ in PLAYER_BLOCK.step_by(2) {
        host.set_sram_word(usize::from(cursor & !1), 0);
        cursor = cursor.wrapping_add(2);
    }
}

/// The stored menu selection, when its range and inverse-bit check agree.
pub fn last_selected(host: &(impl SaveHost + ?Sized)) -> u16 {
    let slot = host.sram_word(0x1fec);
    if slot < 3 && slot & host.sram_word(0x1fee) == 0 {
        slot
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Memory {
        ram: Vec<u8>,
        sram: Vec<u8>,
    }

    impl SaveHost for Memory {
        fn ram_word(&self, at: usize) -> u16 {
            u16::from_ne_bytes([self.ram[at], self.ram[at + 1]])
        }
        fn set_ram_word(&mut self, at: usize, value: u16) {
            self.ram[at..at + 2].copy_from_slice(&value.to_ne_bytes());
        }
        fn sram_word(&self, at: usize) -> u16 {
            u16::from_le_bytes([self.sram[at], self.sram[at + 1]])
        }
        fn set_sram_word(&mut self, at: usize, value: u16) {
            self.sram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        fn slot_offset(&self, slot: isize) -> u16 {
            [0x10, 0x66c, 0xcc8][slot as usize]
        }
    }

    #[test]
    fn a_saved_file_loads_back_with_a_valid_checksum() {
        let mut memory = Memory {
            ram: vec![0; 0x10000],
            sram: vec![0; 0x2000],
        };
        for (index, at) in PLAYER_BLOCK.enumerate() {
            memory.ram[at] = index as u8;
        }
        let checksum = save(&mut memory, 1);
        write_checksum(&mut memory, 1, checksum);
        let saved = memory.ram.clone();
        memory.ram.fill(0);
        let loaded = load(&mut memory, 1);
        assert_eq!(loaded, checksum);
        assert!(checksum_matches(&memory, 1, loaded));
        assert_eq!(memory.ram, saved);
        clear(&mut memory, 1);
        assert_ne!(load(&mut memory, 1), checksum);
    }
}
