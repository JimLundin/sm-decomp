//! The oracle harness: the original, unmodified ROM on lsnes, recorded one
//! video frame at a time (docs/ORACLE.md).
//!
//! Each [`lsnes::Lsnes::step`] runs one video frame with one pad word and
//! returns the [`FrameRecord`] of the closed interval that ends at the frame's
//! cut: the RTI of the NMI serviced in that frame's vblank. The record holds
//! the raw machine state at the cut and what happened in the interval. Nothing
//! here knows the engine; the decoder from these observations to the engine's
//! State lives with the comparison.

pub mod cache;
pub mod cli;
pub mod corpus;
pub use oracle_record::game;
pub use oracle_record::inputs;
pub mod ladder;
pub use data::oracle_lock as lock;
pub mod lsnes;
pub mod measure;
pub mod png;
pub use oracle_record as record;
pub mod seed;

pub use record::{
    ApuAccess, ApuAccessKind, Beam, CpuRegisters, Cut, FrameRecord, Hit, Nmi, NmiKind, Pad,
    Picture, PpuRegisters,
};

/// Who produced a record: locks and records are keyed by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OracleId {
    /// The emulator, `lsnes`.
    pub emulator: &'static str,
    /// The emulator's pinned upstream revision.
    pub version: &'static str,
    /// The ROM patches the oracle applies. Always empty: the ROM runs unmodified.
    pub patches: &'static [&'static str],
    /// The named fixes applied to the emulator's pinned sources.
    pub fixes: &'static [&'static str],
}

/// Where a run begins.
#[derive(Clone, Debug)]
pub enum Start {
    /// Power on. bsnes v085, which recorded the movies, fills work RAM with 0x55.
    PowerOn,
    /// Resume lsnes's own saved machine state.
    State(Vec<u8>),
}

/// The optional streams a step records besides the PPU registers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tiers {
    /// The picture scanned out in the interval.
    pub picture: bool,
    /// The APU port log of the interval.
    pub apu_log: bool,
}

/// Why an oracle could not start, or stopped.
#[derive(Debug)]
pub enum OracleError {
    /// The emulator rejected the cartridge image.
    Rom,
    /// The emulator rejected a saved state.
    State,
    /// The lsnes process stopped serving.
    Lsnes(lsnes::Failure),
}

impl core::fmt::Display for OracleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Rom => write!(f, "the emulator rejected the ROM"),
            Self::State => write!(f, "the emulator rejected the saved state"),
            Self::Lsnes(failure) => write!(f, "{failure}"),
        }
    }
}

impl std::error::Error for OracleError {}
