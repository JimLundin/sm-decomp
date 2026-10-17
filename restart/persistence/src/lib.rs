//! Bounded four-routine SRAM codec from recovered C revision 578f90b.
//! Disk I/O remains an ordered native service after both state buffers change.
#![forbid(unsafe_code)]

#[cfg(any(test, feature = "fixture-support"))]
pub mod fixtures;
mod tables;
#[cfg(any(test, feature = "fixture-support"))]
pub use fixtures::{BodyFixture, body_fixtures};
#[cfg(test)]
mod tests;

pub const WRAM_SIZE: usize = 0x20000;
pub const SRAM_SIZE: usize = 0x2000;
const PLAYER: usize = 0x09a2;
const SAVED: usize = 0xd7c0;
const PAYLOAD_END: usize = 0xde1c;
const PAYLOAD_SIZE: usize = PAYLOAD_END - SAVED;
const MAP: usize = 0xcd52;
const COMPRESSED: usize = 0xd91c;
const SLOTS: [usize; 3] = [0x0010, 0x066c, 0x0cc8];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Entry {
    Save = 0,
    Load = 1,
    Pack = 2,
    Unpack = 3,
}
impl Entry {
    pub const ALL: [Self; 4] = [Self::Save, Self::Load, Self::Pack, Self::Unpack];
    pub const fn address(self) -> u32 {
        match self {
            Self::Save => 0x818000,
            Self::Load => 0x818085,
            Self::Pack => 0x81834b,
            Self::Unpack => 0x8182e4,
        }
    }
}
impl TryFrom<u32> for Entry {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::ALL.get(value as usize).copied().ok_or_else(|| format!("unknown persistence entry {value}"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Done,
    Loaded(u8),
}

/// Availability is checked before writes. A write error is terminal for this
/// invocation: callers must never retry the already-mutated state through C.
pub trait Services {
    fn available(&self) -> bool;
    fn write_sram(&mut self, ram: &mut [u8], sram: &mut [u8]) -> Result<(), String>;
}

/// Execute against complete canonical state and sealed ROM tables. `None` is an
/// unchanged domain refusal; malformed state/table/service inputs are errors.
pub fn execute(
    entry: Entry,
    argument: u16,
    ram: &mut [u8],
    sram: &mut [u8],
    rom: &[u8],
    services: &mut impl Services,
) -> Result<Option<Output>, String> {
    if ram.len() != WRAM_SIZE || sram.len() != SRAM_SIZE {
        return Err("persistence requires complete WRAM and SRAM".into());
    }
    tables::validate(rom)?;
    let slot = usize::from(argument & 3);
    if matches!(entry, Entry::Save | Entry::Load) && slot == 3 {
        return Ok(None);
    }
    if entry == Entry::Save && word(ram, 0x079f) > 5 {
        return Ok(None);
    }
    if matches!(entry, Entry::Pack | Entry::Unpack) && argument != 0 {
        return Err("direct map helper requires argument zero".into());
    }
    if entry == Entry::Save && !services.available() {
        return Err("native SRAM write service unavailable".into());
    }
    // Every potentially failing read and admission check precedes mutation.
    let output = match entry {
        Entry::Save => {
            for offset in (0..96).step_by(2).rev() {
                put(ram, SAVED + offset, word(ram, PLAYER + offset));
            }
            let destination = MAP + usize::from(word(ram, 0x079f)) * 256;
            for offset in (0..256).step_by(2) {
                put(ram, destination + offset, word(ram, 0x07f7 + offset));
            }
            pack(ram);
            put(ram, 0xd916, word(ram, 0x078b));
            put(ram, 0xd918, word(ram, 0x079f));
            let mut sum = 0_u16;
            for offset in (0..PAYLOAD_SIZE).step_by(2) {
                let value = word(ram, SAVED + offset);
                put(sram, SLOTS[slot] + offset, value);
                sum = sum.wrapping_add(value);
            }
            put(sram, slot * 2, sum);
            put(sram, 0x1ff0 + slot * 2, sum);
            put(sram, 8 + slot * 2, !sum);
            put(sram, 0x1ff8 + slot * 2, !sum);
            services.write_sram(ram, sram)?;
            Output::Done
        }
        Entry::Load => {
            let mut sum = 0_u16;
            for offset in (0..PAYLOAD_SIZE).step_by(2) {
                let value = word(sram, SLOTS[slot] + offset);
                put(ram, SAVED + offset, value);
                sum = sum.wrapping_add(value);
            }
            let valid = (word(sram, slot * 2) == sum && word(sram, 8 + slot * 2) == !sum)
                || (word(sram, 0x1ff0 + slot * 2) == sum && word(sram, 0x1ff8 + slot * 2) == !sum);
            if valid {
                for offset in (0..96).step_by(2).rev() {
                    put(ram, PLAYER + offset, word(ram, SAVED + offset));
                }
                unpack(ram);
                put(ram, 0x078b, word(ram, 0xd916));
                put(ram, 0x079f, word(ram, 0xd918));
                Output::Loaded(0)
            } else {
                sram[SLOTS[slot]..SLOTS[slot] + PAYLOAD_SIZE].fill(0);
                put(ram, 0x078b, 0);
                put(ram, 0x079f, 0);
                Output::Loaded(1)
            }
        }
        Entry::Pack => {
            pack(ram);
            Output::Done
        }
        Entry::Unpack => {
            unpack(ram);
            Output::Done
        }
    };
    Ok(Some(output))
}

fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn pack(ram: &mut [u8]) {
    for (area, indices) in tables::INDICES.iter().enumerate() {
        for (position, &index) in indices.iter().enumerate() {
            ram[COMPRESSED + tables::PACKED[area] + position] = ram[MAP + area * 256 + usize::from(index)];
        }
    }
}
fn unpack(ram: &mut [u8]) {
    // The inclusive native word loop clears two bytes past seven map pages.
    ram[MAP..MAP + 0x702].fill(0);
    for (area, indices) in tables::INDICES.iter().enumerate() {
        for (position, &index) in indices.iter().enumerate() {
            ram[MAP + area * 256 + usize::from(index)] = ram[COMPRESSED + tables::PACKED[area] + position];
        }
    }
}
