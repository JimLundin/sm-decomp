//! `sm_tools` (and the oracle side): one projection from a Session state to the
//! compared observations, fed by the engine's state or by the oracle decoder.
use crate::sm_data::display::VideoMemory;
use crate::sm_game::{Checkpoint, State};

/// The streams of a lock row, by name, in lock order (no `wram`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    Sram,
    Vram,
    Cgram,
    Oam,
    Ppu,
    Video,
    Apu,
    Input,
    Samus,
    Camera,
    Room,
    Progress,
    Actors,
    Effects,
}

/// A projection of one stream from a committed state.
pub trait Project {
    const STREAM: Stream;
    fn project(state: &State, out: &mut Vec<u8>);
}

pub struct VramStream;
impl Project for VramStream {
    const STREAM: Stream = Stream::Vram;
    fn project(state: &State, out: &mut Vec<u8>) {
        let VideoMemory { vram, .. } = &state.video.memory;
        out.extend(vram.iter().flat_map(|word| word.to_le_bytes()));
    }
}

/// What the oracle side supplies: its machine after a frame, as raw images.
#[derive(Debug)]
pub struct OracleFrame<'a> {
    pub wram: &'a [u8],
    pub vram: &'a [u8],
    pub cgram: &'a [u8],
    pub oam: &'a [u8],
    pub apu_ram: &'a [u8],
    pub cpu_pc: u32,
    pub stack: &'a [u8],
}

#[derive(Debug)]
pub enum DecodeError {
    /// The CPU is waiting at a site no `Frame` variant names.
    UnknownYieldSite(u32),
    /// A ROM address the address map does not hold (instruction pointer, AI state).
    UnmappedAddress { field: &'static str, address: u32 },
}

/// The RAM → Session decoder: reads the oracle's machine through `ram_map.rs`
/// and the content decoder's address map, never the engine's internals.
pub trait Decode {
    fn decode(&self, frame: &OracleFrame<'_>) -> Result<Checkpoint, DecodeError>;
}
