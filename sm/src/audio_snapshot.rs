//! Rewind the emulated APU without transferring ownership of its allocations.

use crate::types::{Apu, Dsp, Snes, Spc};
use crate::{game_state::Game, rtl::ApuWriteEnt};

#[derive(Copy, Clone)]
pub(crate) struct ApuQueueSnapshot {
    pub pending: ApuWriteEnt,
    entries: [ApuWriteEnt; 16],
    pub size: u8,
    pub position: u8,
    pub time_since_empty: u8,
}

impl ApuQueueSnapshot {
    pub(crate) fn capture(game: &Game) -> Self {
        Self {
            pending: game.g_apu_write,
            entries: game.g_apu_write_ents,
            size: game.g_apu_queue_size,
            position: game.g_apu_write_ent_pos,
            time_since_empty: game.g_apu_time_since_empty,
        }
    }

    pub(crate) fn restore(self, game: &mut Game) {
        game.g_apu_write = self.pending;
        game.g_apu_write_ents = self.entries;
        game.g_apu_queue_size = self.size;
        game.g_apu_write_ent_pos = self.position;
        game.g_apu_time_since_empty = self.time_since_empty;
    }

    pub(crate) fn select_runtime(self, theirs: Self, game: &mut Game) {
        if game.g_use_my_apu_code {
            self.restore(game);
            game.g_apu_write = theirs.pending;
        } else {
            theirs.restore(game);
        }
    }
}

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
