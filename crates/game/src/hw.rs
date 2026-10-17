//! Hardware side of the runtime.
//!
//! `RamBus` is a flat 16 MiB map for the demo and tests. The real implementation
//! is `snes::SnesBus`, wrapping RustySNES's `rustysnes-core::Bus` so reads/writes to
//! $2100-$21FF, $4200-$43FF and DMA go to its PPU/APU/DMA models, and ROM/WRAM
//! map normally. Its 65816 core becomes the `Fallback`.

use runtime::Bus;

pub struct RamBus {
    pub mem: Vec<u8>,
}

impl RamBus {
    pub fn new() -> Self {
        Self { mem: vec![0; 1 << 24] }
    }
}

impl Default for RamBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus for RamBus {
    fn read(&mut self, addr: u32) -> u8 {
        self.mem[addr as usize & 0xff_ffff]
    }
    fn write(&mut self, addr: u32, v: u8) {
        self.mem[addr as usize & 0xff_ffff] = v
    }
}

/// Memory regions compared each frame by the diff harness.
pub mod regions {
    pub const WRAM: std::ops::Range<u32> = 0x7e_0000..0x80_0000;
}
