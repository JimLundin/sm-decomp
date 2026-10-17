//! Ripper/JetPowerRipper initialization and patrol; lifecycle and terrain remain native.
#![forbid(unsafe_code)]

#[cfg(any(test, feature = "fixture-support"))]
mod fixtures;
#[cfg(feature = "fixture-support")]
pub use fixtures::{BodyFixture, body_fixtures};
mod memory;
mod patrol;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Entry {
    JetInit = 0,
    JetMain = 1,
    RipperInit = 2,
    RipperMain = 3,
}
impl Entry {
    pub const ALL: [Self; 4] = [Self::JetInit, Self::JetMain, Self::RipperInit, Self::RipperMain];
    pub const fn address(self) -> u32 {
        match self {
            Self::JetInit => 0xa2e318,
            Self::JetMain => 0xa2e353,
            Self::RipperInit => 0xa2e49f,
            Self::RipperMain => 0xa2e4da,
        }
    }
}
impl TryFrom<u32> for Entry {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, String> {
        Self::ALL.get(value as usize).copied().ok_or_else(|| format!("unknown Ripper entry {value}"))
    }
}

pub trait Services {
    /// Immutable admission from the incoming canonical Main state. No calls or effects.
    fn available(&self) -> bool;
    /// Executes A0:C6AB exactly once, with full synchronous WRAM copyback.
    /// Accepted errors are fatal, never a request to retry the actor in C.
    fn move_horizontal(&mut self, ram: &mut [u8], slot: u16, amount: i32) -> Result<u8, String>;
}

/// `None` refuses unchanged before requesting a service. Runtime ROM belongs
/// to the caller's selected native identity; all65 table rows must be present.
pub fn execute(entry: Entry, ram: &mut [u8], rom: &[u8], services: &mut impl Services) -> Result<Option<()>, String> {
    use memory::*;
    if ram.len() != SIZE || rom.get(TABLE..TABLE + TABLE_SIZE).is_none() {
        return Ok(None);
    }
    let slot = word(ram, CURRENT);
    if !valid_slot(slot) {
        return Ok(None);
    }
    let base = ENEMIES + usize::from(slot);
    let init = matches!(entry, Entry::JetInit | Entry::RipperInit);
    let row = if init { word(ram, base + PARAMETER1).wrapping_mul(8) } else { word(ram, base + E) };
    if !valid_row(row) || (!init && !services.available()) {
        return Ok(None);
    }
    let jet = matches!(entry, Entry::JetInit | Entry::JetMain);
    if init {
        patrol::init(ram, rom, base, jet);
    } else {
        patrol::main(ram, rom, slot, base, jet, services)?;
    }
    Ok(Some(()))
}
