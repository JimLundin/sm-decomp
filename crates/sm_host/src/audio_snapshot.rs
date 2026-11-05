//! Rewind the emulated APU without transferring ownership of its allocations.

use crate::game_state::Game;
use crate::types::{
    ApuRegisters, DspChannel, DspRegWriteHistory, DspState, NativePlayerState, Snes, Spc, Timer,
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
    }
}

impl PlayerSnapshot {
    pub(crate) fn capture(
        &mut self,
        player: &NativePlayerState,
        ram: &crate::apu_ram::ApuRam,
        dsp: &DspState,
    ) {
        self.valid = true;
        self.player = *player;
        ram.copy_to(&mut self.ram);
        self.dsp = *dsp;
    }
    /// Capture `player`; without one the snapshot is invalid.
    pub(crate) fn capture_from(&mut self, player: Option<&crate::spc_player::NativePlayer>) {
        match player {
            Some(player) => self.capture(&player.state, &player.ram, &player.dsp.state),
            None => self.valid = false,
        }
    }
    pub(crate) fn restore(
        &self,
        player: &mut NativePlayerState,
        ram: &crate::apu_ram::ApuRam,
        dsp: &mut crate::types::Dsp,
    ) {
        *player = self.player;
        ram.copy_from(&self.ram);
        dsp.restore_state(&self.dsp);
    }
    /// Restore a valid snapshot into `player`; an invalid one, or no player, changes nothing.
    pub(crate) fn restore_into(&self, player: Option<&mut crate::spc_player::NativePlayer>) {
        if let (true, Some(player)) = (self.valid, player) {
            self.restore(&mut player.state, &player.ram, &mut player.dsp);
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
                channel,
                snapshot_padding0,
                snapshot_padding1,
                snapshot_padding2
            ],
            ram,
            dir_page,
            even_cycle,
            mute,
            reset,
            master_volume_l,
            master_volume_r,
            noise_sample,
            noise_rate,
            noise_counter,
            echo_writes,
            echo_volume_l,
            echo_volume_r,
            feedback_volume,
            echo_buffer_adr,
            echo_delay,
            echo_remain,
            echo_buffer_index,
            fir_buffer_index,
            fir_values,
            fir_buffer_l,
            fir_buffer_r,
            sample_buffer,
            sample_offset
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
                pitch_counter,
                pitch_modulation,
                decode_buffer,
                srcn,
                decode_offset,
                previous_flags,
                old,
                older,
                use_noise,
                adsr_rates,
                rate_counter,
                adsr_state,
                sustain_level,
                use_gain,
                gain_mode,
                direct_gain,
                gain_value,
                gain,
                key_on,
                key_off,
                sample_out,
                volume_l,
                volume_r,
                echo_enable
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
