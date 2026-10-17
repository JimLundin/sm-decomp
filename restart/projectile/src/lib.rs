//! Ordinary uncharged beam lifetime from pinned snesrev/sm recovered C.
//! See README.md for the admitted domain, scheduled boundaries and provenance.
//! No mutable state lives outside the supplied canonical WRAM.

mod audio;
mod collision;
mod core;
mod draw;
mod memory;
mod trails;

pub use memory::WRAM_SIZE;
use memory::Wram;
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    FireUnchargedBeam,
    HandleProjectile,
    DrawPlayerExplosions2,
    DrawBombAndProjectileExplosions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Handled,
    Refused,
}

/// The only temporary gameplay service. It executes against the latest WRAM.
/// The caller counts requests before entering C and never retries an owned failure.
pub trait Host {
    fn spawn_plm(&mut self, ram: &mut [u8], header: u16) -> Result<u8>;
}

fn admitted(entry: Entry, r: &Wram<'_>) -> bool {
    if r.word(0x9a6) != 0 || r.word(0xa76) != 0 {
        return false;
    }
    // Only update owns transformed trail placement. Fire and both rendering
    // boundaries retain their existing Ceres escape exclusion.
    if entry != Entry::HandleProjectile && r.word(0x93f) & 0x8000 != 0 {
        return false;
    }
    if r.word(0xac4) != 0 || r.word(0xac6) != 0 {
        return false;
    }
    for k in (0..20).step_by(2) {
        if r.word(core::INSTRUCTION + k) == 0 {
            continue;
        }
        if k >= 10 {
            return false;
        }
        let kind = r.word(core::TYPE + k);
        let pre = r.word(core::PRE + k);
        if !matches!((kind, pre), (0x8000, 0xaef3) | (0x8700, 0xb169)) {
            return false;
        }
    }
    true
}

/// Run one actual scheduled operation. Refusal reads state only; after admission,
/// malformed data or a failed foreign request is an error, never C fallback.
/// The native integration supplies its one transaction; this crate makes no RAM copy.
pub fn run(entry: Entry, ram: &mut [u8], rom: &[u8], host: &mut impl Host) -> Result<Outcome> {
    let mut r = Wram::new(ram)?;
    if !admitted(entry, &r) {
        return Ok(Outcome::Refused);
    }
    match entry {
        Entry::FireUnchargedBeam => core::fire(&mut r, rom, host)?,
        Entry::HandleProjectile => core::update(&mut r, rom, host)?,
        Entry::DrawPlayerExplosions2 => draw::player(&mut r, rom)?,
        Entry::DrawBombAndProjectileExplosions => draw::explosions(&mut r, rom)?,
    }
    Ok(Outcome::Handled)
}

pub(crate) fn rb(rom: &[u8], address: u32) -> Result<u8> {
    if address & 0xffff < 0x8000 {
        return Err("ROM address below LoROM window".into());
    }
    let offset = (((address >> 16) & 0x7f) * 0x8000 + (address & 0x7fff)) as usize;
    rom.get(offset).copied().ok_or_else(|| format!("ROM extent at {address:06x}"))
}

pub(crate) fn rw(rom: &[u8], address: u32) -> Result<u16> {
    if address & 0xffff == 0xffff {
        return Err("ROM word crosses bank".into());
    }
    Ok(u16::from_le_bytes([rb(rom, address)?, rb(rom, address + 1)?]))
}

#[cfg(test)]
mod tests;
