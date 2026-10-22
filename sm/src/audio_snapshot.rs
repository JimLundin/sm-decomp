//! Rewind the emulated APU without transferring ownership of its allocations.

use crate::types::{Apu, Dsp, DspChannel, DspRegWriteHistory, Snes, Spc, SpcPlayer, Timer};
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
    pub(crate) fn to_bytes(self) -> [u8; 71] {
        let mut bytes = [0; 71];
        bytes[..4].copy_from_slice(&self.pending.ports);
        for (entry, ports) in self.entries.iter().zip(bytes[4..68].chunks_exact_mut(4)) {
            ports.copy_from_slice(&entry.ports);
        }
        bytes[68..].copy_from_slice(&[self.size, self.position, self.time_since_empty]);
        bytes
    }

    pub(crate) fn from_bytes(bytes: &[u8; 71]) -> Self {
        assert!(bytes[68] <= 16, "invalid saved APU queue size");
        let mut entries = [ApuWriteEnt { ports: [0; 4] }; 16];
        for (entry, ports) in entries.iter_mut().zip(bytes[4..68].chunks_exact(4)) {
            entry.ports.copy_from_slice(ports);
        }
        Self {
            pending: ApuWriteEnt {
                ports: bytes[..4].try_into().unwrap(),
            },
            entries,
            size: bytes[68],
            position: bytes[69],
            time_since_empty: bytes[70],
        }
    }

    pub(crate) fn first_difference(&self, other: &Self) -> Option<&'static str> {
        let Self {
            pending: ApuWriteEnt { ports: pending },
            entries,
            size,
            position,
            time_since_empty,
        } = self;
        if *pending != other.pending.ports {
            return Some("pending.ports");
        }
        for (ApuWriteEnt { ports }, other) in entries.iter().zip(&other.entries) {
            if *ports != other.ports {
                return Some("entries.ports");
            }
        }
        if *size != other.size {
            return Some("size");
        }
        if *position != other.position {
            return Some("position");
        }
        if *time_since_empty != other.time_since_empty {
            return Some("time_since_empty");
        }
        None
    }

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
pub struct PlayerSnapshot {
    valid: bool,
    player: SpcPlayer,
    dsp: Dsp,
    has_history: bool,
    history: DspRegWriteHistory,
}

impl PlayerSnapshot {
    /// The player, DSP and any attached history must be live and exclusive.
    pub(crate) unsafe fn capture(&mut self, player: *mut SpcPlayer) {
        self.valid = !player.is_null();
        if !self.valid {
            return;
        }
        self.player = *player;
        self.dsp = *(*player).dsp;
        self.has_history = !(*player).reg_write_history.is_null();
        if self.has_history {
            self.history = *(*player).reg_write_history;
        }
        self.player.dsp = core::ptr::null_mut();
        self.player.reg_write_history = core::ptr::null_mut();
        self.dsp.apu_ram = core::ptr::null_mut();
    }

    /// Destination allocations must be live with the same history topology.
    pub(crate) unsafe fn restore(&self, player: *mut SpcPlayer) {
        if !self.valid {
            return;
        }
        let dsp = (*player).dsp;
        let history = (*player).reg_write_history;
        assert_eq!(self.has_history, !history.is_null());
        *player = self.player;
        (*player).dsp = dsp;
        (*player).reg_write_history = history;
        *dsp = self.dsp;
        (*dsp).apu_ram = (*player).ram.as_mut_ptr();
        if self.has_history {
            *history = self.history;
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
    /// Compare emulated values, excluding allocation links and layout padding.
    #[forbid(unsafe_code)]
    pub(crate) fn first_difference(&self, other: &Self) -> Option<&'static str> {
        macro_rules! fields {
            ($ty:ident, $left:expr, $right:expr, [$($excluded:ident),*], $($field:ident),+ $(,)?) => {
                // Exhaustive patterns make added fields require an explicit choice.
                let $ty { $($excluded: _,)* $($field: _,)+ } = $left;
                let left = $left;
                let right = $right;
                $(if left.$field != right.$field {
                    return Some(concat!(stringify!($ty), ".", stringify!($field)));
                })+
            };
        }
        fields!(
            Apu,
            &self.apu,
            &other.apu,
            [spc, dsp, pad, timer, history],
            ram,
            romReadable,
            dspAdr,
            cycles,
            inPorts,
            outPorts,
            cpuCyclesLeft
        );
        for (left, right) in self.apu.timer.iter().zip(&other.apu.timer) {
            fields!(
                Timer,
                left,
                right,
                [],
                cycles,
                divider,
                target,
                counter,
                enabled
            );
        }
        fields!(
            Spc,
            &self.spc,
            &other.spc,
            [apu],
            a,
            x,
            y,
            sp,
            pc,
            c,
            z,
            v,
            n,
            i,
            h,
            p,
            b,
            stopped,
            cyclesUsed
        );
        fields!(
            Dsp,
            &self.dsp,
            &other.dsp,
            [apu_ram, channel],
            ram,
            dirPage,
            evenCycle,
            mute,
            reset,
            masterVolumeL,
            masterVolumeR,
            noiseSample,
            noiseRate,
            noiseCounter,
            echoWrites,
            echoVolumeL,
            echoVolumeR,
            feedbackVolume,
            echoBufferAdr,
            echoDelay,
            echoRemain,
            echoBufferIndex,
            firBufferIndex,
            firValues,
            firBufferL,
            firBufferR,
            sampleBuffer,
            sampleOffset
        );
        for (left, right) in self.dsp.channel.iter().zip(&other.dsp.channel) {
            fields!(
                DspChannel,
                left,
                right,
                [],
                pitch,
                pitchCounter,
                pitchModulation,
                decodeBuffer,
                srcn,
                decodeOffset,
                previousFlags,
                old,
                older,
                useNoise,
                adsrRates,
                rateCounter,
                adsrState,
                sustainLevel,
                useGain,
                gainMode,
                directGain,
                gainValue,
                gain,
                keyOn,
                keyOff,
                sampleOut,
                volumeL,
                volumeR,
                echoEnable
            );
        }
        fields!(
            DspRegWriteHistory,
            &self.apu.history,
            &other.apu.history,
            [],
            count,
            addr,
            val
        );
        if self.catchup_cycles.to_bits() != other.catchup_cycles.to_bits() {
            return Some("Snes.apuCatchupCycles");
        }
        None
    }

    /// The SNES and all three audio allocations must be initialized and live.
    pub(crate) unsafe fn capture(&mut self, snes: *const Snes) {
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
