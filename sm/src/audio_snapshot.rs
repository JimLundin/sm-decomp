//! Rewind the emulated APU without transferring ownership of its allocations.

use crate::game_state::Game;
use crate::types::{
    ApuRegisters, Dsp, DspChannel, DspRegWriteHistory, Snes, Spc, SpcPlayer, Timer,
};

pub(crate) use crate::apu_queue::ApuQueue as ApuQueueSnapshot;

impl ApuQueueSnapshot {
    pub(crate) fn capture(game: &Game) -> Self {
        game.apu_queue
    }

    pub(crate) fn restore(self, game: &mut Game) {
        game.apu_queue = self;
    }

    pub(crate) fn select_runtime(self, theirs: Self, game: &mut Game) {
        if game.g_use_my_apu_code {
            self.restore(game);
            game.apu_queue.pending = theirs.pending;
        } else {
            theirs.restore(game);
        }
    }
}

crate::typed_struct! {
    initial; [#[derive(Copy, Clone)]];
    pub struct PlayerSnapshot {
        valid: bool;
        player: SpcPlayer;
        dsp: Dsp;
        has_history: bool;
        history: DspRegWriteHistory;
    }
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

crate::typed_struct! {
    initial; [#[derive(Copy, Clone)]];
    pub struct AudioSnapshot {
        ram: [u8; 65536];
        apu: ApuRegisters;
        spc: Spc;
        dsp: Dsp;
        catchup_cycles: f64;
    }
}

impl AudioSnapshot {
    /// Compare emulated values, excluding allocation links and layout padding.
    #[forbid(unsafe_code)]
    pub(crate) fn first_difference(&self, other: &Self) -> Option<&'static str> {
        macro_rules! fields {
            ($ty:ident as $label:ident, $left:expr, $right:expr, [$($excluded:ident),*], $($field:ident),+ $(,)?) => {
                // Exhaustive patterns make added fields require an explicit choice.
                let $ty { $($excluded: _,)* $($field: _,)+ } = $left;
                let left = $left;
                let right = $right;
                $(if left.$field != right.$field {
                    return Some(concat!(stringify!($label), ".", stringify!($field)));
                })+
            };
            ($ty:ident, $($args:tt)*) => { fields!($ty as $ty, $($args)*); };
        }
        if self.ram != other.ram {
            return Some("Apu.ram");
        }
        fields!(
            ApuRegisters as Apu,
            &self.apu,
            &other.apu,
            [pad, snapshot_padding0, timer, history],
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
            [
                apu_ram,
                channel,
                snapshot_padding0,
                snapshot_padding1,
                snapshot_padding2
            ],
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
                [
                    snapshot_padding0,
                    snapshot_padding1,
                    snapshot_padding2,
                    snapshot_padding3,
                    snapshot_padding4,
                    snapshot_padding5,
                    snapshot_padding6
                ],
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
        (*apu).ram.copy_to(&mut self.ram);
        self.apu = (*apu).registers;
        self.spc = *(*apu).spc;
        self.dsp = *(*apu).dsp;
        self.catchup_cycles = (*snes).apuCatchupCycles;
        // These links belong to the live emulator, never to a snapshot.
        self.spc.apu = core::ptr::null_mut();
        self.dsp.apu_ram = core::ptr::null_mut();
    }

    /// The destination allocations must be live and exclusively accessible.
    pub(crate) unsafe fn restore(&self, snes: *mut Snes) {
        let apu = (*snes).apu;
        let spc = (*apu).spc;
        let dsp = (*apu).dsp;
        (*apu).ram.copy_from(&self.ram);
        (*apu).registers = self.apu;
        *spc = self.spc;
        (*spc).apu = apu;
        *dsp = self.dsp;
        (*dsp).apu_ram = (*apu).ram.as_mut_ptr();
        (*snes).apuCatchupCycles = self.catchup_cycles;
    }
}
