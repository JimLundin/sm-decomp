//! The original SRAM save format: three files copied word by word from the
//! player RAM block, each guarded by a word checksum stored twice with its
//! inverse, and a remembered menu selection.

/// The work RAM a save file holds: $7E:D7C0 up to $7E:DE1C.
pub const PLAYER_BLOCK: core::ops::Range<usize> = 0xd7c0..0xde1c;

/// The SRAM header (SRAM map): per-slot checksums and their complements,
/// mirrored near the end of SRAM, and the last selected slot.
pub mod header {
    pub const CHECKSUMS: usize = 0;
    pub const COMPLEMENTS: usize = 8;
    pub const MIRRORED_CHECKSUMS: usize = 0x1ff0;
    pub const MIRRORED_COMPLEMENTS: usize = 0x1ff8;
    /// The signature "supermetroid" that marks formatted SRAM.
    pub const SIGNATURE: core::ops::Range<usize> = 0x1fe0..0x1fec;
    pub const LAST_SLOT: usize = 0x1fec;
    pub const LAST_SLOT_COMPLEMENT: usize = 0x1fee;

    /// The byte offset of `slot`'s word in a per-slot header table.
    pub const fn slot(table: usize, slot: usize) -> usize {
        table + 2 * slot
    }
}

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
    let slot = usize::from(slot & 3);
    host.set_sram_word(header::slot(header::CHECKSUMS, slot), checksum);
    host.set_sram_word(header::slot(header::MIRRORED_CHECKSUMS, slot), checksum);
    host.set_sram_word(header::slot(header::COMPLEMENTS, slot), !checksum);
    host.set_sram_word(header::slot(header::MIRRORED_COMPLEMENTS, slot), !checksum);
}

/// Whether either header copy holds `checksum` and its inverse.
pub fn checksum_matches(host: &(impl SaveHost + ?Sized), slot: u16, checksum: u16) -> bool {
    let slot = usize::from(slot & 3);
    let word = |table| host.sram_word(header::slot(table, slot));
    checksum == word(header::CHECKSUMS) && !checksum == word(header::COMPLEMENTS)
        || checksum == word(header::MIRRORED_CHECKSUMS)
            && !checksum == word(header::MIRRORED_COMPLEMENTS)
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
    let slot = host.sram_word(header::LAST_SLOT);
    if slot < 3 && slot & host.sram_word(header::LAST_SLOT_COMPLEMENT) == 0 {
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

use std::{cell::Cell, ops::Range, rc::Rc};

/// The cartridge's battery-backed SRAM bytes, shared by the cartridge bus,
/// the save menus and snapshots.
#[derive(Clone, Default)]
pub struct SaveRam(Rc<[Cell<u8>]>);

impl SaveRam {
    pub fn zeroed(length: usize) -> Self {
        Self((0..length).map(|_| Cell::new(0)).collect())
    }

    pub fn read(&self, offset: usize) -> u8 {
        self.0[offset].get()
    }

    pub fn write(&self, offset: usize, value: u8) {
        self.0[offset].set(value);
    }

    pub fn word(&self, offset: usize) -> u16 {
        u16::from_ne_bytes([self.read(offset), self.read(offset + 1)])
    }

    pub fn set_word(&self, offset: usize, value: u16) {
        self.write_bytes(offset, &value.to_ne_bytes());
    }

    pub fn read_bytes(&self, offset: usize, output: &mut [u8]) {
        let cells = &self.0[offset..offset + output.len()];
        for (byte, cell) in output.iter_mut().zip(cells) {
            *byte = cell.get();
        }
    }

    pub fn write_bytes(&self, offset: usize, input: &[u8]) {
        for (cell, &byte) in self.0[offset..offset + input.len()].iter().zip(input) {
            cell.set(byte);
        }
    }

    pub fn bytes(&self, range: Range<usize>) -> Vec<u8> {
        self.0[range].iter().map(Cell::get).collect()
    }

    pub fn fill(&self, range: Range<usize>, value: u8) {
        for cell in &self.0[range] {
            cell.set(value);
        }
    }

    pub fn copy_within(&self, source: Range<usize>, destination: usize) {
        self.write_bytes(destination, &self.bytes(source));
    }
}

impl crate::menus::host::SaveRam for SaveRam {
    fn word(&self, at: usize) -> u16 {
        SaveRam::word(self, at)
    }
    fn set_word(&self, at: usize, value: u16) {
        SaveRam::set_word(self, at, value)
    }
    fn bytes(&self, range: Range<usize>) -> Vec<u8> {
        SaveRam::bytes(self, range)
    }
    fn write_bytes(&self, at: usize, input: &[u8]) {
        SaveRam::write_bytes(self, at, input)
    }
    fn copy_within(&self, source: Range<usize>, destination: usize) {
        SaveRam::copy_within(self, source, destination)
    }
    fn fill(&self, range: Range<usize>, value: u8) {
        SaveRam::fill(self, range, value)
    }
}
