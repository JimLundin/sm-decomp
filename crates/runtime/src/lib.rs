//! Runtime that recompiled Super Metroid code runs on.
//!
//! Model: recompile the CPU, emulate the silicon.
//! - [`Cpu`] holds 65816 registers. Generated code mutates it directly.
//! - [`Bus`] is the rest of the console (WRAM, ROM, PPU/APU registers, DMA).
//!   The plan is to back it with RustySNES's `rustysnes-core::Bus`.
//! - [`Machine`] ties CPU + bus + the table of generated functions together and
//!   routes calls. A call to an address/width combination with no generated
//!   function goes to the [`Fallback`] interpreter and is recorded as a *miss*,
//!   which feeds the next round of the pipeline.

pub mod cpu;
pub mod ea;
pub mod machine;
pub mod ops;

pub use cpu::{flag, Cpu};
pub use machine::{Bus, Fallback, GenFn, Machine, Miss, TableEntry};

/// Width/flag key used everywhere: bit 1 = M (8-bit A), bit 0 = X (8-bit X/Y).
/// `0b11` means both 8-bit ("m1x1"), `0b00` both 16-bit ("m0x0").
pub type Mx = u8;

pub fn mx_name(mx: Mx) -> &'static str {
    match mx & 3 {
        0 => "m0x0",
        1 => "m0x1",
        2 => "m1x0",
        _ => "m1x1",
    }
}
