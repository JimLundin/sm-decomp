//! Rewind the emulated APU without transferring ownership of its allocations.

use crate::types::{Apu, Dsp, Snes, Spc};

#[derive(Copy, Clone)]
pub struct AudioSnapshot {
    apu: Apu,
    spc: Spc,
    dsp: Dsp,
    catchup_cycles: f64,
}

impl AudioSnapshot {
    /// The SNES and all three audio allocations must be initialized and live.
    pub(crate) unsafe fn capture(&mut self, snes: *mut Snes) {
        let apu = (*snes).apu;
        self.apu = *apu;
        self.spc = *(*apu).spc;
        self.dsp = *(*apu).dsp;
        self.catchup_cycles = (*snes).apuCatchupCycles;
        // These links belong to the live emulator, never to a snapshot.
        self.apu.spc = core::ptr::null_mut();
        self.apu.dsp = core::ptr::null_mut();
        self.spc.apu = core::ptr::null_mut();
        self.dsp.apu_ram = core::ptr::null_mut();
    }

    /// The destination allocations must be live and exclusively accessible.
    pub(crate) unsafe fn restore(&self, snes: *mut Snes) {
        let apu = (*snes).apu;
        let spc = (*apu).spc;
        let dsp = (*apu).dsp;
        *apu = self.apu;
        (*apu).spc = spc;
        (*apu).dsp = dsp;
        *spc = self.spc;
        (*spc).apu = apu;
        *dsp = self.dsp;
        (*dsp).apu_ram = (*apu).ram.as_mut_ptr();
        (*snes).apuCatchupCycles = self.catchup_cycles;
    }
}
