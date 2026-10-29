//! Rewind the emulated APU without transferring ownership of its allocations.

use crate::game_state::Game;
use crate::types::{
    ApuRegisters, DspChannel, DspRegWriteHistory, DspState, NativePlayerState, Snes, Spc,
    SpcPlayer, Timer,
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
        player: NativePlayerState;
        ram: [u8; 65536];
        dsp: DspState;
        has_history: bool;
        history: DspRegWriteHistory;
    }
}

#[forbid(unsafe_code)]
impl PlayerSnapshot {
    pub(crate) fn begin_capture(&mut self, valid: bool) -> bool {
        self.valid = valid;
        valid
    }
    pub(crate) fn valid(&self) -> bool {
        self.valid
    }

    pub(crate) fn capture_parts(
        &mut self,
        player: NativePlayerState,
        ram: &crate::apu_ram::ApuRam,
        dsp: impl FnOnce() -> DspState,
        has_history: impl FnOnce() -> bool,
        history: impl FnOnce() -> DspRegWriteHistory,
    ) {
        self.player = player;
        ram.copy_to(&mut self.ram);
        self.dsp = dsp();
        self.has_history = has_history();
        if self.has_history {
            self.history = history();
        }
        self.player.dsp = core::ptr::null_mut();
        self.player.ram = core::ptr::null_mut();
        self.player.reg_write_history = core::ptr::null_mut();
        self.dsp.apu_ram = core::ptr::null_mut();
    }

    // Capture destination identities before the first restore effect. Returned
    // links are inert values; only the retained native adapter follows them.
    pub(crate) fn restore_player(
        &self,
        player: &mut NativePlayerState,
        ram: &crate::apu_ram::ApuRam,
    ) -> (*mut crate::types::Dsp, *mut DspRegWriteHistory) {
        let dsp = player.dsp;
        let history = player.reg_write_history;
        assert_eq!(self.has_history, !history.is_null());
        *player = self.player;
        ram.copy_from(&self.ram);
        player.ram = ram.as_mut_ptr();
        player.dsp = dsp;
        player.reg_write_history = history;
        (dsp, history)
    }
    pub(crate) fn restore_dsp(&self, dsp: &mut crate::types::Dsp) {
        dsp.restore_state(&self.dsp);
    }
    fn restore_history(&self, history: &mut DspRegWriteHistory) {
        *history = self.history;
    }
}

impl PlayerSnapshot {
    /// Native allocations and this snapshot must be disjoint, live, exclusive,
    /// stopped and non-reentrant. DSP/history selections remain fresh and late.
    pub(crate) unsafe fn capture(&mut self, player: *mut SpcPlayer) {
        unsafe {
            if !self.begin_capture(!player.is_null()) {
                return;
            }
            self.capture_parts(
                (*player).state,
                &(*player).ram,
                || (*(*player).state.dsp).state,
                || !(*player).state.reg_write_history.is_null(),
                || *(*player).state.reg_write_history,
            );
        }
    }
    /// Keep the same disjoint lifetimes and history topology until the tail.
    pub(crate) unsafe fn restore(&self, player: *mut SpcPlayer) {
        unsafe {
            if !self.valid() {
                return;
            }
            let (dsp, history) = self.restore_player(&mut (*player).state, &(*player).ram);
            self.restore_dsp(&mut *dsp);
            if self.has_history {
                self.restore_history(&mut *history);
            }
        }
    }
    /// Select the actual owner; external/rebound/history intervals use the same
    /// phased native adapter and its retained lifetime/exclusivity contracts.
    pub(crate) unsafe fn capture_with_owner(
        &mut self,
        owner: Option<&mut crate::spc_player::NativePlayer>,
        player: *mut SpcPlayer,
    ) {
        unsafe {
            if let Some(owner) = owner.filter(|owner| owner.view() == player) {
                if owner.try_capture_snapshot(self) {
                    return;
                }
            }
            self.capture(player);
        }
    }
    pub(crate) unsafe fn restore_with_owner(
        &self,
        owner: Option<&mut crate::spc_player::NativePlayer>,
        player: *mut SpcPlayer,
    ) {
        unsafe {
            if !self.valid() {
                return;
            }
            if let Some(owner) = owner.filter(|owner| owner.view() == player) {
                if owner.try_restore_snapshot(self) {
                    return;
                }
            }
            self.restore(player);
        }
    }
}

crate::typed_struct! {
    initial; [#[derive(Copy, Clone)]];
    pub struct AudioSnapshot {
        ram: [u8; 65536];
        apu: ApuRegisters;
        spc: Spc;
        dsp: DspState;
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
            DspState as Dsp,
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
        unsafe {
            let apu = (*snes).apu;
            (*apu).ram.copy_to(&mut self.ram);
            self.apu = (*apu).registers;
            self.spc = *(*apu).spc;
            self.dsp = (*(*apu).dsp).state;
            self.catchup_cycles = (*snes).apuCatchupCycles;
            // These links belong to the live emulator, never to a snapshot.
            self.spc.apu = core::ptr::null_mut();
            self.dsp.apu_ram = core::ptr::null_mut();
        }
    }

    /// The destination allocations must be live and exclusively accessible.
    pub(crate) unsafe fn restore(&self, snes: *mut Snes) {
        unsafe {
            let apu = (*snes).apu;
            let spc = (*apu).spc;
            let dsp = (*apu).dsp;
            (*apu).ram.copy_from(&self.ram);
            (*apu).registers = self.apu;
            *spc = self.spc;
            (*spc).apu = apu;
            (*dsp).restore_state(&self.dsp);
            (*snes).apuCatchupCycles = self.catchup_cycles;
        }
    }
}
