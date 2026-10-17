//! Complete Waver actor logic at recovered C revision 578f90b.
//! Scheduler, terrain, sine, contact, damage and rendering remain native.
#![forbid(unsafe_code)]
mod actor;
#[cfg(any(test, feature = "fixture-support"))]
pub mod fixtures;
mod memory;
#[cfg(any(test, feature = "fixture-support"))]
pub use fixtures::{BodyFixture, body_fixtures};
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Entry {
    Init = 0,
    Main = 1,
    Instruction = 2,
    SelectAnimation = 3,
}
impl Entry {
    pub const ALL: [Self; 4] = [Self::Init, Self::Main, Self::Instruction, Self::SelectAnimation];
    pub const fn address(self) -> u32 {
        match self {
            Self::Init => 0xa386ed,
            Self::Main => 0xa3874c,
            Self::Instruction => 0xa386e3,
            Self::SelectAnimation => 0xa387fe,
        }
    }
}
impl TryFrom<u32> for Entry {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, String> {
        Self::ALL.get(value as usize).copied().ok_or_else(|| format!("unknown Waver entry {value}"))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Done,
    Continue(u16),
}

/// Availability describes the complete incoming Main invocation without effects.
/// Native service failures after any request are terminal, never C fallback.
pub trait Services {
    fn available(&self) -> bool;
    fn move_horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String>;
    fn sine(&mut self, ram: &mut [u8], angle: u16, amplitude: u16) -> Result<u16, String>;
    fn move_vertical(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String>;
}

/// `k` is the instruction's explicit slot, not the current-slot source of its
/// event write. `jp` is a bank-A3 byte offset; real continuations may be odd.
/// Noninstruction entries ignore k/jp and obtain their slot from canonical WRAM.
/// `None` refuses unchanged; malformed state/ROM and post-request errors fail.
pub fn execute(
    entry: Entry,
    k: u16,
    jp: u16,
    ram: &mut [u8],
    rom: &[u8],
    services: &mut impl Services,
) -> Result<Option<Output>, String> {
    use memory::*;
    if ram.len() != SIZE {
        return Err("Waver requires complete canonical WRAM".into());
    }
    let (_, base) = current(ram)?;
    if entry == Entry::Instruction {
        if !valid_slot(k) || !(0x8000..=0xfffe).contains(&jp) {
            return Err("invalid Waver instruction slot or bank-A3 continuation".into());
        }
        let offset = 0x110000 + usize::from(jp);
        if rom.get(offset..offset + 2).is_none() {
            return Err("Waver instruction continuation exceeds ROM".into());
        }
        put(ram, base + E, 1);
        return Ok(Some(Output::Continue(jp)));
    }
    if entry == Entry::SelectAnimation && word(ram, base + F) == word(ram, base + C) {
        // Original helper does not look up ROM for an unchanged selector.
        return Ok(Some(Output::Done));
    }
    if matches!(entry, Entry::Main | Entry::SelectAnimation) && word(ram, base + F) > 3 {
        return Ok(None);
    }
    table_available(rom)?;
    if entry == Entry::Main && !services.available() {
        return Ok(None);
    }
    match entry {
        Entry::Init => actor::init(ram, rom)?,
        Entry::Main => actor::main(ram, rom, services)?,
        Entry::SelectAnimation => actor::select_animation(ram, rom)?,
        Entry::Instruction => unreachable!(),
    }
    Ok(Some(Output::Done))
}
