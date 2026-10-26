//! Named native-audio fields and their byte-compatible SPC RAM representation.
#![forbid(unsafe_code)]
use crate::{apu_ram::ApuRam, types::NativePlayerState};

trait RamValue {
    fn save(&self, ram: &ApuRam, address: usize) -> usize;
    fn load(&mut self, ram: &ApuRam, address: usize) -> usize;
}
macro_rules! scalar {
    ($($ty:ty),*) => {$(
        impl RamValue for $ty {
            fn save(&self, ram: &ApuRam, address: usize) -> usize {
                let bytes = self.to_ne_bytes();
                for (i, byte) in bytes.iter().enumerate() { ram.write_at(address + i, *byte); }
                address + bytes.len()
            }
            fn load(&mut self, ram: &ApuRam, address: usize) -> usize {
                let mut bytes = self.to_ne_bytes();
                for (i, byte) in bytes.iter_mut().enumerate() { *byte = ram.read_at(address + i); }
                *self = Self::from_ne_bytes(bytes);
                address + bytes.len()
            }
        }
    )*};
}
scalar!(u8, u16);
impl<T: RamValue, const N: usize> RamValue for [T; N] {
    fn save(&self, ram: &ApuRam, mut address: usize) -> usize {
        for value in self {
            address = value.save(ram, address);
        }
        address
    }
    fn load(&mut self, ram: &ApuRam, mut address: usize) -> usize {
        for value in self {
            address = value.load(ram, address);
        }
        address
    }
}

// Each row is an ordered span of complete scalar/array values.
macro_rules! spans {
    ($method:ident, $state:ident, $ram:ident; $($address:expr => $($field:ident $(.$child:ident)?),+;)+) => {$(
        let mut address = $address;
        $(address = $state.$field$(.$child)?.$method($ram, address);)+
        let _ = address;
    )+};
}

// No callbacks occur during these transfers. The mapped native fields and SPC
// destinations are disjoint, so grouping by owner preserves every observed byte.
macro_rules! transfer {
    ($method:ident, $state:ident, $ram:ident, $iter:ident $(, $mutable:tt)?) => {
        for (i, channel) in $state.channel.$iter().enumerate() {
            spans! { $method, channel, $ram;
                0x30 + i * 2 => pattern_order_ptr_for_chan;
                0x70 + i * 2 => note_ticks_left, note_keyoff_ticks_left;
                0x80 + i * 2 => subroutine_num_loops;
                0x90 + i * 2 => volume_fade_ticks, pan_num_ticks;
                0xa0 + i * 2 => pitch_slide_length, pitch_slide_delay_left;
                0xb0 + i * 2 => vibrato_hold_count, vib_depth;
                0xc0 + i * 2 => tremolo_hold_count, tremolo_depth;
                0x100 + i * 2 => vibrato_change_count;
                0x200 + i * 2 => note_length, note_gate_off_fixedpt;
                0x210 + i * 2 => channel_volume_master, instrument_id;
                0x220 + i * 2 => instrument_pitch_base;
                0x230 + i * 2 => saved_pattern_ptr;
                0x240 + i * 2 => pattern_start_ptr;
                0x280 + i * 2 => pitch_envelope_num_ticks, pitch_envelope_delay;
                0x290 + i * 2 => pitch_envelope_direction, pitch_envelope_slide_value;
                0x2a0 + i * 2 => vibrato_count, vibrato_rate;
                0x2b0 + i * 2 => vibrato_delay_ticks, vibrato_fade_num_ticks;
                0x2c0 + i * 2 => vibrato_fade_add_per_tick, vibrato_depth_target;
                0x2d0 + i * 2 => tremolo_count, tremolo_rate;
                0x2e0 + i * 2 => tremolo_delay_ticks;
                0x2f0 + i * 2 => channel_transposition;
                0x300 + i * 2 => channel_volume;
                0x310 + i * 2 => volume_fade_addpertick;
                0x320 + i * 2 => volume_fade_target, final_volume;
                0x330 + i * 2 => pan_value;
                0x340 + i * 2 => pan_add_per_tick;
                0x350 + i * 2 => pan_target_value, pan_flag_with_phase_invert;
                0x360 + i * 2 => pitch;
                0x370 + i * 2 => pitch_add_per_tick;
                0x380 + i * 2 => pitch_target, fine_tune;
                0x400 + i * 2 => cutk;
            }
        }
        spans! { $method, $state, $ram;
            0x0 => input_ports, port_to_snes, last_value_from_snes, counter_sf0c, always_zero,
                temp_accum, ttt, did_affect_volumepitch_flag, addr0, addr1, lfsr_value, is_chan_on,
                fast_forward, timer_cycles;
            0x20 => word_20;
            0x40 => music_ptr_toplevel, block_count, sfx_timer_accum, chn, key_ON, key_OFF,
                cur_chan_bit, reg_FLG, reg_NON, reg_EON;
            0x4c => echo_stored_time, echo_parameter_EDL, reg_EFB;
            0x50 => global_transposition, main_tempo_accum, tempo, tempo_fade_num_ticks,
                tempo_fade_final, tempo_fade_add, master_volume, master_volume_fade_ticks,
                master_volume_fade_target, master_volume_fade_add_per_tick, vol_dirty,
                percussion_base_id, echo_volume_left, echo_volume_right, echo_volume_fade_add_left,
                echo_volume_fade_add_right, echo_volume_fade_ticks, echo_volume_fade_target_left,
                echo_volume_fade_target_right;
            0xd2 => p_echoBuffer;
            0xee => memset_ptr;
            0x390 => memset_ctr;
            0x4a9 => disable_sfx2;
            0x4b1 => byte_4B1;
            0x4ba => sfx3.some_value;
            0x4be => last_written_edl;
        }
        for (regs, [word, sound, setup, index, voices, priority]) in [
            (&$($mutable)? $state.sfx1, [0x22, 0x392, 0x3a0, 0x3ae, 0x3b3, 0x4bb]),
            (&$($mutable)? $state.sfx2, [0xd4, 0x3f8, 0x440, 0x44a, 0x44d, 0x4bc]),
            (&$($mutable)? $state.sfx3, [0xe0, 0x470, 0x478, 0x482, 0x485, 0x4bd]),
        ] {
            spans! { $method, regs, $ram;
                word => cur_ptr, chan_voice_bitset_ptr, chan_voice_mask_ptr, chan_voice_index_ptr;
                sound => cur_sound, cur_sound_index;
                setup => chan_idx, voices_to_setup, voice_index, voices_remaining, init_flag, voice_id;
                index => chan_idx_x2;
                voices => enabled_voices;
                priority => priority;
            }
        }
        for (channels, pointers, layout) in [
            (
                &$($mutable)? $state.sfx_chans_1[..], &[0x2a, 0x2c, 0x2e, 0xd0][..],
                [
                    0x394, 0x398, 0x39c, 0x3a6, 0x3aa, 0x3af, 0x3b4, 0x3b8, 0x3c0, 0x3c8, 0x3cc,
                    0x3d0, 0x3d8, 0x3dc
                ],
            ),
            (
                &$($mutable)? $state.sfx_chans_2[..], &[0xdc, 0xde][..],
                [
                    0x3fa, 0x3fc, 0x3fe, 0x446, 0x448, 0x44b, 0x44e, 0x450, 0x454, 0x458, 0x45a,
                    0x45c, 0x460, 0x462
                ],
            ),
            (
                &$($mutable)? $state.sfx_chans_3[..], &[0xe8, 0xea][..],
                [
                    0x472, 0x474, 0x476, 0x47e, 0x480, 0x483, 0x486, 0x488, 0x48c, 0x490, 0x492,
                    0x494, 0x498, 0x49a
                ],
            ),
        ] {
            let [ptr_idx, instr_timer, disable, voice_bitset, channel_mask, voice_index,
                voice_index_mult8, volume, release_flag, repeat_ctr, repeat_pt,
                adsr_settings, update_adsr, note] = layout;
            for (i, channel) in channels.$iter().enumerate() {
                spans! { $method, channel, $ram;
                    pointers[i] => ptr;
                    ptr_idx + i => ptr_idx;
                    instr_timer + i => instr_timer;
                    disable + i => disable;
                    voice_bitset + i => voice_bitset;
                    channel_mask + i => channel_mask;
                    voice_index + i => voice_index;
                    voice_index_mult8 + i => voice_index_mult8;
                    volume + i * 2 => volume, phase_invert;
                    release_flag + i * 2 => release_flag, release_timer;
                    repeat_ctr + i => repeat_ctr;
                    repeat_pt + i => repeat_pt;
                    adsr_settings + i * 2 => adsr_settings;
                    update_adsr + i => update_adsr;
                    note + i * 7 => note, subnote, subnote_delta, target_note,
                        enable_pitch_slide, legato, enable_pitch_slide_legato;
                }
            }
        }
    };
}

pub fn save(state: &NativePlayerState, ram: &ApuRam) {
    transfer!(save, state, ram, iter);
}

pub fn load(state: &mut NativePlayerState, ram: &ApuRam) {
    transfer!(load, state, ram, iter_mut, mut);
    for (i, channel) in state.channel.iter_mut().enumerate() {
        channel.index = i as u8;
    }
}
