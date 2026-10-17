//! Ordinary Rio actor bodies from pinned recovered C 578f90b3.
#![forbid(unsafe_code)]
mod actor;
#[cfg(any(test, feature = "fixture-support"))]
pub mod fixtures;
mod memory;
mod service_protocol;
mod service_writes;
mod sine;
pub use service_protocol::{NativeService, ServiceProtocol};
pub use service_writes::{ServiceWrites, service_wram_matches};
#[cfg(test)]
mod tests;
pub const WRAM_SIZE: usize = 0x20000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Entry {
    Init = 0,
    Main,
    Instruction,
    State1,
    State2,
    State3,
    State4,
    State5,
    SelectAnimation,
    CallRioFunc,
}
impl Entry {
    pub const ALL: [Self; 10] = [
        Self::Init,
        Self::Main,
        Self::Instruction,
        Self::State1,
        Self::State2,
        Self::State3,
        Self::State4,
        Self::State5,
        Self::SelectAnimation,
        Self::CallRioFunc,
    ];
    pub const fn address(self) -> Option<u32> {
        Some(match self {
            Self::Init => 0xa2bbcd,
            Self::Main => 0xa2bbe3,
            Self::Instruction => 0xa2bbc3,
            Self::State1 => 0xa2bbed,
            Self::State2 => 0xa2bc32,
            Self::State3 => 0xa2bc48,
            Self::State4 => 0xa2bcb7,
            Self::State5 => 0xa2bcff,
            Self::SelectAnimation => 0xa2bd54,
            Self::CallRioFunc => return None,
        })
    }
}
impl TryFrom<u32> for Entry {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, String> {
        Self::ALL.get(value as usize).copied().ok_or_else(|| format!("unknown Rio entry {value}"))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Done,
    Continue(u16),
}

/// Availability is a side-effect-free admission check on the complete incoming
/// invocation. Services synchronously copy back complete canonical WRAM. An
/// error after admission is terminal and must never trigger native fallback.
pub trait Services {
    fn available(&self) -> bool;
    fn move_horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String>;
    fn move_vertical(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String>;
    fn angle(&mut self, ram: &mut [u8], k: u16) -> Result<u16, String>;
}

/// `arg` is jp for Instruction, a u16 list address for SelectAnimation, and
/// the exact 24-bit target for CallRioFunc. State roots use explicit k; other
/// roots use cur_enemy_index. Helpers continue to use cur_enemy_index, even
/// when it differs from the state root's cached actor pointer.
/// None refuses unchanged. Malformed admitted input and service failures error.
pub fn execute(
    entry: Entry,
    k: u16,
    arg: u32,
    ram: &mut [u8],
    rom: &[u8],
    services: &mut impl Services,
) -> Result<Option<Output>, String> {
    use memory::*;
    if ram.len() != SIZE {
        return Err("Rio requires complete canonical WRAM".into());
    }
    let current_base = current(ram)?;
    if entry == Entry::Instruction {
        slot(k)?;
        if !(0x8000..=0xfffe).contains(&arg) {
            return Err("invalid Rio bank-A2 continuation".into());
        }
        let at = 0x108000 + arg as usize;
        if rom.get(at..at + 2).is_none() {
            return Err("Rio continuation exceeds runtime ROM".into());
        }
        put(ram, current_base + E, 1);
        return Ok(Some(Output::Continue(arg as u16)));
    }
    if entry == Entry::SelectAnimation {
        let a = u16::try_from(arg).map_err(|_| "Rio animation address exceeds u16")?;
        animation(ram, a)?;
        return Ok(Some(Output::Done));
    }
    if entry == Entry::Init {
        put(ram, current_base + E, 0);
        put(ram, current_base + F, 0);
        put(ram, current_base + 0x1a, 0xbb4b);
        put(ram, current_base + B, STATES[0]);
        return Ok(Some(Output::Done));
    }
    let (state, actor_k) = match entry {
        Entry::Main => (word(ram, current_base + B), word(ram, CURRENT)),
        Entry::CallRioFunc => {
            if arg & 0xffff0000 != 0xa20000 {
                return Ok(None);
            }
            (arg as u16, word(ram, CURRENT))
        }
        Entry::State1 => (STATES[0], k),
        Entry::State2 => (STATES[1], k),
        Entry::State3 => (STATES[2], k),
        Entry::State4 => (STATES[3], k),
        Entry::State5 => (STATES[4], k),
        _ => unreachable!(),
    };
    let Some(state_index) = STATES.iter().position(|v| *v == state) else {
        return Ok(None);
    };
    let base = slot(actor_k)?;
    // Preflight every malformed table/queue condition before Main advances RNG.
    if state_index == 0 && near(ram, base) {
        rom_word(rom, 0x113bbb)?;
        rom_word(rom, 0x113bbf)?;
        if on_screen(ram, current_base) {
            sound_preflight(ram)?;
        }
    }
    let movement = state_index == 2
        || state_index == 3
        || (state_index == 4 && (word(ram, base + 6).wrapping_sub(word(ram, 0xafa)) as i16) < 0);
    if movement && !services.available() {
        return Ok(None);
    }
    if entry == Entry::Main {
        random(ram);
    }
    actor::state(state_index, actor_k, base, ram, rom, services)?;
    Ok(Some(Output::Done))
}
