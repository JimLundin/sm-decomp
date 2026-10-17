//! Room tilemap preparation and layer scrolling from pinned snesrev/sm bank $80.
//! See README.md for the scheduled boundary and remaining native responsibilities.

mod memory;
mod scroll;

pub use memory::WRAM_SIZE;
use memory::Wram;
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    DisplayViewablePartOfRoom,
    UpdateScrollVarsUpdateMap,
    CalculateLayer2PosAndScrollsWhenScrolling,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Handled,
    Refused,
}

/// Run actual NMI_ProcessVramWriteQueue synchronously against current WRAM.
/// Publish all writes before entering native code, preserve ordered DMA effects,
/// then refresh RAM before returning. Count a request before entering the service.
pub trait Host {
    fn flush_vram(&mut self, ram: &mut [u8]) -> Result<()>;
}

/// Execute one complete scheduled root using the host's existing transaction.
/// No ROM access, speculative execution or full-RAM copying occurs here.
/// Invalid memory extents and service failures are owned errors; callers must
/// terminate instead of rerunning the operation in C. Current valid inputs are
/// handled; Refused is retained for the shared integration ownership vocabulary.
pub fn run(entry: Entry, ram: &mut [u8], host: &mut impl Host) -> Result<Outcome> {
    let mut ram = Wram::new(ram)?;
    match entry {
        Entry::DisplayViewablePartOfRoom => scroll::display(&mut ram, host)?,
        Entry::UpdateScrollVarsUpdateMap => scroll::update_scroll_vars(&mut ram)?,
        Entry::CalculateLayer2PosAndScrollsWhenScrolling => scroll::layer2_and_scrolls(&mut ram)?,
    }
    Ok(Outcome::Handled)
}
