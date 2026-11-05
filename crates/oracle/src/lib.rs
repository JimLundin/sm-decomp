//! The oracle harness: the original, unmodified ROM on an emulator, recorded
//! one video frame at a time (docs/oracle/HARNESS.md).
//!
//! An [`Oracle`] is one emulator backend. Each [`Oracle::step`] runs one video
//! frame with one pad word and returns the [`FrameRecord`] of the closed
//! interval that ends at the frame's cut: the RTI of the NMI serviced in that
//! frame's vblank. The record holds the raw machine state at the cut and what
//! happened in the interval. Nothing here knows the engine; the decoder from
//! these observations to the engine's State lives with the comparison.

pub mod cli;
pub mod corpus;
pub mod diagnose;
pub mod game;
pub mod inputs;
pub mod lakesnes;
pub mod lock;
pub mod lsnes;
pub mod measure;
pub mod record;
pub mod seed;
pub mod streams;

pub use record::{
    ApuAccess, ApuAccessKind, Beam, CpuRegisters, Cut, FrameRecord, Hit, Nmi, NmiKind, Pad,
    Picture, PpuRegisters, RegisterWrite, SpcState, WriteSource,
};

/// Who produced a record: locks and records are keyed by it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OracleId {
    /// The emulator, for example `lakesnes`.
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
    /// Power on, with work RAM filled with `wram_fill` (hardware leaves it
    /// unspecified; bsnes v085, which recorded the movies, fills 0x55).
    PowerOn { wram_fill: u8 },
    /// Resume this oracle's own saved machine state.
    State(Vec<u8>),
    /// Load a migration seed captured on another emulator (`seed::Capture`).
    Capture(Box<seed::Capture>),
}

/// The optional observation tiers a step records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tiers {
    /// The picture scanned out in the interval.
    pub picture: bool,
    /// The APU port log of the interval.
    pub apu_log: bool,
    /// The APU side at the cut: SPC700 registers, SPC RAM and DSP registers.
    pub apu_state: bool,
    /// Instructions executed per program bank in the interval.
    pub profile: bool,
    /// Every B-bus PPU register write in the interval, with its beam position and source.
    pub write_log: bool,
}

impl Default for Tiers {
    fn default() -> Self {
        Self {
            picture: true,
            apu_log: true,
            apu_state: false,
            profile: false,
            write_log: false,
        }
    }
}

/// Why an oracle could not start, or stopped.
#[derive(Debug)]
pub enum OracleError {
    /// The emulator rejected the cartridge image.
    Rom,
    /// The emulator rejected a saved state.
    State,
    /// A migration seed could not be applied.
    Seed(String),
    /// The lsnes process stopped serving.
    Lsnes(lsnes::Failure),
}

impl core::fmt::Display for OracleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Rom => write!(f, "the emulator rejected the ROM"),
            Self::State => write!(f, "the emulator rejected the saved state"),
            Self::Seed(reason) => write!(f, "the seed could not be applied: {reason}"),
            Self::Lsnes(failure) => write!(f, "{failure}"),
        }
    }
}

impl std::error::Error for OracleError {}

/// One emulator running the original game.
pub trait Oracle: Sized {
    /// Who this oracle is.
    fn identity(&self) -> OracleId;
    /// Power on with `rom`, or resume `start`.
    fn open(rom: &[u8], start: Start) -> Result<Self, OracleError>;
    /// Run one video frame with `pad` on controller 1, to the frame's cut.
    /// An emulator in another process can stop serving; the error says why.
    fn step(&mut self, pad: Pad) -> Result<&FrameRecord, OracleError>;
    /// The machine at the last cut, as this oracle's own resumable state.
    fn save_state(&self) -> Vec<u8>;
    /// Select the observation tiers for the following steps.
    fn set_tiers(&mut self, tiers: Tiers);
    /// Press the console's reset button before the next step.
    fn reset(&mut self);
    /// Record the beam position and four WRAM bytes at `peek` whenever the
    /// CPU reaches one of `addresses` (at most 8), in `FrameRecord::hits`.
    fn set_watch(&mut self, addresses: &[u32], peek: u32);
}

/// A backend chosen at run time.
pub enum Backend {
    LakeSnes(Box<lakesnes::LakeSnes>),
    Lsnes(Box<lsnes::Lsnes>),
}

impl Backend {
    /// Open the backend `name` names: `lakesnes` or `lsnes`.
    pub fn open(name: &str, rom: &[u8], start: Start) -> Result<Self, OracleError> {
        match name {
            "lsnes" => lsnes::Lsnes::open(rom, start).map(|oracle| Self::Lsnes(Box::new(oracle))),
            _ => {
                lakesnes::LakeSnes::open(rom, start).map(|oracle| Self::LakeSnes(Box::new(oracle)))
            }
        }
    }

    pub fn step(&mut self, pad: Pad) -> Result<&FrameRecord, OracleError> {
        match self {
            Self::LakeSnes(oracle) => oracle.step(pad),
            Self::Lsnes(oracle) => oracle.step(pad),
        }
    }

    pub fn set_tiers(&mut self, tiers: Tiers) {
        match self {
            Self::LakeSnes(oracle) => oracle.set_tiers(tiers),
            Self::Lsnes(oracle) => oracle.set_tiers(tiers),
        }
    }

    pub fn set_watch(&mut self, addresses: &[u32], peek: u32) {
        match self {
            Self::LakeSnes(oracle) => oracle.set_watch(addresses, peek),
            Self::Lsnes(oracle) => oracle.set_watch(addresses, peek),
        }
    }

    pub fn identity(&self) -> OracleId {
        match self {
            Self::LakeSnes(oracle) => oracle.identity(),
            Self::Lsnes(oracle) => oracle.identity(),
        }
    }
}
