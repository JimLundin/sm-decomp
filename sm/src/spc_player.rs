use crate::c_library::printf;
use crate::initial_state::InitialState;
use crate::snes::dsp::{Dsp, dsp_cycle, dsp_reset, dsp_write};
pub use crate::types::{
    Channel, DspRegWriteHistory, NativePlayerState, SfxChan, SfxRegs, SpcPlayer, size_t,
};
use std::{cell::UnsafeCell, rc::Rc};
pub const EVOLL: DspReg = 44;
pub const EVOLR: DspReg = 60;
pub const EFB: DspReg = 13;
pub const EON: DspReg = 77;
pub const FLG: DspReg = 108;
pub const KON: DspReg = 76;
pub const KOF: DspReg = 92;
pub const NON: DspReg = 61;
pub const PMON: DspReg = 45;
pub const V0PITCHH: DspReg = 3;
pub const V0PITCHL: DspReg = 2;
pub const V0VOLL: DspReg = 0;
pub const V0GAIN: DspReg = 7;
pub const V0ADSR2: DspReg = 6;
pub const V0ADSR1: DspReg = 5;
pub const V0SRCN: DspReg = 4;
pub const FIR0: DspReg = 15;
pub const ESA: DspReg = 109;
pub const EDL: DspReg = 125;
pub const DIR: DspReg = 93;
pub const MVOLR: DspReg = 28;
pub const MVOLL: DspReg = 12;
pub use crate::types::DspReg;
pub const FIR7: DspReg = 127;
pub const ENDX: DspReg = 124;
pub const V7OUTX: DspReg = 121;
pub const V7ENVX: DspReg = 120;
pub const V7GAIN: DspReg = 119;
pub const V7ADSR2: DspReg = 118;
pub const V7ADSR1: DspReg = 117;
pub const V7SRCN: DspReg = 116;
pub const V7PH: DspReg = 115;
pub const V7PL: DspReg = 114;
pub const V7VOLR: DspReg = 113;
pub const V7VOLL: DspReg = 112;
pub const FIR6: DspReg = 111;
pub const V6OUTX: DspReg = 105;
pub const V6ENVX: DspReg = 104;
pub const V6GAIN: DspReg = 103;
pub const V6ADSR2: DspReg = 102;
pub const V6ADSR1: DspReg = 101;
pub const V6SRCN: DspReg = 100;
pub const V6PH: DspReg = 99;
pub const V6PL: DspReg = 98;
pub const V6VOLR: DspReg = 97;
pub const V6VOLL: DspReg = 96;
pub const FIR5: DspReg = 95;
pub const V5OUTX: DspReg = 89;
pub const V5ENVX: DspReg = 88;
pub const V5GAIN: DspReg = 87;
pub const V5ADSR2: DspReg = 86;
pub const V5ADSR1: DspReg = 85;
pub const V5SRCN: DspReg = 84;
pub const V5PH: DspReg = 83;
pub const V5PL: DspReg = 82;
pub const V5VOLR: DspReg = 81;
pub const V5VOLL: DspReg = 80;
pub const FIR4: DspReg = 79;
pub const V4OUTX: DspReg = 73;
pub const V4ENVX: DspReg = 72;
pub const V4GAIN: DspReg = 71;
pub const V4ADSR2: DspReg = 70;
pub const V4ADSR1: DspReg = 69;
pub const V4SRCN: DspReg = 68;
pub const V4PH: DspReg = 67;
pub const V4PL: DspReg = 66;
pub const V4VOLR: DspReg = 65;
pub const V4VOLL: DspReg = 64;
pub const FIR3: DspReg = 63;
pub const V3OUTX: DspReg = 57;
pub const V3ENVX: DspReg = 56;
pub const V3GAIN: DspReg = 55;
pub const V3ADSR2: DspReg = 54;
pub const V3ADSR1: DspReg = 53;
pub const V3SRCN: DspReg = 52;
pub const V3PH: DspReg = 51;
pub const V3PL: DspReg = 50;
pub const V3VOLR: DspReg = 49;
pub const V3VOLL: DspReg = 48;
pub const FIR2: DspReg = 47;
pub const V2OUTX: DspReg = 41;
pub const V2ENVX: DspReg = 40;
pub const V2GAIN: DspReg = 39;
pub const V2ADSR2: DspReg = 38;
pub const V2ADSR1: DspReg = 37;
pub const V2SRCN: DspReg = 36;
pub const V2PH: DspReg = 35;
pub const V2PL: DspReg = 34;
pub const V2VOLR: DspReg = 33;
pub const V2VOLL: DspReg = 32;
pub const FIR1: DspReg = 31;
pub const V1OUTX: DspReg = 25;
pub const V1ENVX: DspReg = 24;
pub const V1GAIN: DspReg = 23;
pub const V1ADSR2: DspReg = 22;
pub const V1ADSR1: DspReg = 21;
pub const V1SRCN: DspReg = 20;
pub const V1PH: DspReg = 19;
pub const V1PL: DspReg = 18;
pub const V1VOLR: DspReg = 17;
pub const V1VOLL: DspReg = 16;
pub const V0OUTX: DspReg = 9;
pub const V0ENVX: DspReg = 8;
pub const V0VOLR: DspReg = 1;
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
// C sees only NativePlayerState; types.rs asserts its zero-offset SpcPlayer prefix.
// The trailing ApuRam owner and the opaque Dsp pointer remain Rust-owned.
#[link(name = "sm_music", kind = "static", modifiers = "-bundle")]
unsafe extern "C-unwind" {
    #[allow(improper_ctypes)]
    #[link_name = "sm_music_SetupEchoParameter_EDL"]
    fn SetupEchoParameter_EDL(p: *mut SpcPlayer, a: u8);
    #[allow(improper_ctypes)]
    #[link_name = "sm_music_WriteVolumeToDsp"]
    fn WriteVolumeToDsp(p: *mut SpcPlayer, c: *mut Channel, volume: u16);
    #[allow(improper_ctypes)]
    #[link_name = "sm_music_HandlePanAndSweep"]
    fn HandlePanAndSweep(p: *mut SpcPlayer, c: *mut Channel);
    #[allow(improper_ctypes)]
    #[link_name = "sm_music_Music_HandleCmdFromSnes"]
    fn Music_HandleCmdFromSnes(p: *mut SpcPlayer);
}

#[unsafe(export_name = "sm_music_Dsp_Write")]
pub unsafe extern "C-unwind" fn Dsp_Write(p: *mut SpcPlayer, mut reg: u8, value: u8) {
    unsafe {
        let hist = (*p).state.reg_write_history;
        if !hist.is_null() && (*hist).count < 256 {
            if (*hist).count == 9 && reg as i32 == 0x5d {
                reg = (reg as i32 + 0) as u8;
            }
            (*hist).addr[(*hist).count as usize] = reg;
            (*hist).val[(*hist).count as usize] = value;
            (*hist).count = (*hist).count.wrapping_add(1);
        }
        if !(*p).state.dsp.is_null() {
            dsp_write((*p).state.dsp, reg, value);
        }
    }
}
#[unsafe(export_name = "sm_music_Not_Implemented")]
pub unsafe extern "C-unwind" fn Not_Implemented() {
    unsafe {
        crate::native_assertion! {
            '_c2rust_label;
            b"0\0", b"src/spc_player.c\0", 383, b"void Not_Implemented(void)\0"
        };
        printf(c"Not Implemented\n".as_ptr());
    }
}
#[unsafe(export_name = "sm_music_WritePitchInner")]
pub unsafe extern "C-unwind" fn WritePitchInner(p: *mut SpcPlayer, c: *mut Channel, pitch: u16) {
    unsafe {
        static kBaseNoteFreqs: [u16; 13] = [
            2143, 2270, 2405, 2548, 2700, 2860, 3030, 3211, 3402, 3604, 3818, 4045, 4286,
        ];
        let pp: u8 = (pitch as i32 >> 8 & 0x7f) as u8;
        let mut q: u8 = (pp as i32 / 12) as u8;
        let r: u8 = (pp as i32 % 12) as u8;
        let mut t: u16 = (kBaseNoteFreqs[r as usize] as i32
            + ((kBaseNoteFreqs[(r as i32 + 1) as usize] as i32 - kBaseNoteFreqs[r as usize] as i32)
                as u8 as i32
                * pitch as u8 as i32
                >> 8)) as u16;
        t = (t as i32 * 2) as u16;
        while q as i32 != 6 {
            t = t >> 1;
            q = q.wrapping_add(1);
        }
        t = ((*c).instrument_pitch_base as i32 * t as i32 >> 8) as u16;
        if (*p).state.cur_chan_bit & (*p).state.is_chan_on == 0 {
            let reg: u8 = ((*c).index as i32 * 16) as u8;
            Dsp_Write(p, (reg as i32 + V0PITCHL as i32) as u8, t as u8 & 0xff);
            Dsp_Write(p, (reg as i32 + V0PITCHH as i32) as u8, (t >> 8) as u8);
        }
    }
}
#[unsafe(export_name = "sm_music_Channel_SetInstrument_NoSetId")]
pub unsafe extern "C-unwind" fn Channel_SetInstrument_NoSetId(
    p: *mut SpcPlayer,
    c: *mut Channel,
    mut instrument: u8,
) {
    unsafe {
        if instrument & 0x80 != 0 {
            instrument = (instrument as i32 + 54 + (*p).state.percussion_base_id as i32) as u8;
        }
        let ram = (*p).ram.clone();
        let ip = usize::from(instrument) * 6 + 0x6c00;
        if (*p).state.is_chan_on & (*p).state.cur_chan_bit != 0 {
            return;
        }
        let reg: u8 = ((*c).index as i32 * 16) as u8;
        if ram.read_at(ip) & 0x80 != 0 {
            (*p).state.reg_FLG = (*p).state.reg_FLG & 0x20 | ram.read_at(ip) & 0x1f;
            (*p).state.reg_NON = (*p).state.reg_NON | (*p).state.cur_chan_bit;
            Dsp_Write(p, (reg as i32 + V0SRCN as i32) as u8, 0);
        } else {
            Dsp_Write(p, (reg as i32 + V0SRCN as i32) as u8, ram.read_at(ip));
        }
        Dsp_Write(p, (reg as i32 + V0ADSR1 as i32) as u8, ram.read_at(ip + 1));
        Dsp_Write(p, (reg as i32 + V0ADSR2 as i32) as u8, ram.read_at(ip + 2));
        Dsp_Write(p, (reg as i32 + V0GAIN as i32) as u8, ram.read_at(ip + 3));
        (*c).instrument_pitch_base =
            ((ram.read_at(ip + 4) as i32) << 8 | ram.read_at(ip + 5) as i32) as u16;
    }
}
static kEffectByteLength: [u8; 31] = [
    1, 1, 2, 3, 0, 1, 2, 1, 2, 1, 1, 3, 0, 1, 2, 3, 1, 3, 3, 0, 1, 3, 0, 3, 3, 3, 1, 2, 0, 0, 0,
];
#[unsafe(export_name = "sm_music_EffectByteLength")]
pub unsafe extern "C-unwind" fn EffectByteLength(effect: u8) -> u8 {
    kEffectByteLength[(effect as i32 - 0xe0) as usize]
}
#[unsafe(export_name = "sm_music_WantWriteKof")]
pub unsafe extern "C-unwind" fn WantWriteKof(p: *mut SpcPlayer, c: *mut Channel) -> bool {
    unsafe {
        let mut loops: i32 = (*c).subroutine_num_loops as i32;
        let mut ptr: i32 = (*c).pattern_order_ptr_for_chan as i32;
        loop {
            crate::update_place! { previous fresh7 = ptr => ptr + 1; }
            let mut cmd: u8 = (*p).ram.read_at(fresh7 as usize);
            if cmd as i32 == 0 {
                if loops == 0 {
                    return true_0 != 0;
                }
                loops -= 1;
                ptr = if loops == 0 {
                    (*c).saved_pattern_ptr as i32
                } else {
                    (*c).pattern_start_ptr as i32
                };
            } else {
                while cmd & 0x80 == 0 {
                    crate::update_place! { previous fresh8 = ptr => ptr + 1; }
                    cmd = (*p).ram.read_at(fresh8 as usize);
                }
                if cmd as i32 == 0xc8 {
                    return false_0 != 0;
                }
                if cmd as i32 == 0xef {
                    ptr = (*p).ram.read_at((ptr + 0) as usize) as i32
                        | ((*p).ram.read_at((ptr + 1) as usize) as i32) << 8;
                } else if cmd as i32 >= 0xe0 {
                    ptr += kEffectByteLength[(cmd as i32 - 0xe0i32) as usize] as i32;
                } else {
                    return true_0 != 0;
                }
            }
        }
    }
}
#[unsafe(export_name = "sm_music_CalcFinalVolume")]
pub unsafe extern "C-unwind" fn CalcFinalVolume(p: *mut SpcPlayer, c: *mut Channel, vol: u8) {
    unsafe {
        let mut t: i32 = ((*p).state.master_volume as i32 >> 8) * vol as i32 >> 8;
        t = t * (*c).channel_volume_master as i32 >> 8;
        t = t * ((*c).channel_volume as i32 >> 8) >> 8;
        (*c).final_volume = (t * t >> 8) as u8;
    }
}
unsafe fn Spc_Loop_Part1(p: *mut SpcPlayer) {
    unsafe {
        Dsp_Write(p, KOF as i32 as u8, (*p).state.key_OFF);
        Dsp_Write(p, PMON as i32 as u8, 0);
        Dsp_Write(p, NON as i32 as u8, (*p).state.reg_NON);
        Dsp_Write(p, KOF as i32 as u8, 0);
        Dsp_Write(p, KON as i32 as u8, (*p).state.key_ON);
        if (*p).state.echo_stored_time & 0x80 == 0 {
            Dsp_Write(p, FLG as i32 as u8, (*p).state.reg_FLG);
            if (*p).state.echo_stored_time as i32 == (*p).state.echo_parameter_EDL as i32 {
                Dsp_Write(p, EON as i32 as u8, (*p).state.reg_EON);
                Dsp_Write(p, EFB as i32 as u8, (*p).state.reg_EFB);
                Dsp_Write(
                    p,
                    EVOLR as i32 as u8,
                    ((*p).state.echo_volume_right >> 8) as u8,
                );
                Dsp_Write(
                    p,
                    EVOLL as i32 as u8,
                    ((*p).state.echo_volume_left >> 8) as u8,
                );
            }
        }
        (*p).state.key_ON = 0;
        (*p).state.key_OFF = (*p).state.key_ON;
    }
}
unsafe fn Spc_Loop_Part2(p: *mut SpcPlayer, ticks: u8) {
    unsafe {
        let mut t = u16::from((*p).state.sfx_timer_accum) + u16::from(ticks.wrapping_mul(0x20));
        (*p).state.sfx_timer_accum = t as u8;
        if t >= 0x100 {
            Sfx1_HandleCmdFromSnes(p);
            Sfx2_HandleCmdFromSnes(p);
            Sfx3_HandleCmdFromSnes(p);
            if (*p).state.echo_stored_time as i32 != (*p).state.echo_parameter_EDL as i32 {
                (*p).state.echo_stored_time = (*p).state.echo_stored_time.wrapping_add(1);
            }
        }
        t = u16::from((*p).state.main_tempo_accum)
            + u16::from(ticks.wrapping_mul((*p).state.tempo.to_ne_bytes()[1]));
        (*p).state.main_tempo_accum = t as u8;
        if t >= 256 {
            do_until! { (*p).state.fast_forward == 0 ;
                Music_HandleCmdFromSnes(p);
            }
        } else if (*p).state.port_to_snes[0i32 as usize] != 0 {
            let mut channel = 0;
            (*p).state.cur_chan_bit = 1;
            while (*p).state.cur_chan_bit as i32 != 0 {
                // Pan/sweep preserves the selector; this traverses all eight owned channels.
                let c = &mut (*p).state.channel[channel];
                if c.pattern_order_ptr_for_chan.to_ne_bytes()[1] != 0 {
                    HandlePanAndSweep(p, c);
                }
                (*p).state.cur_chan_bit = (((*p).state.cur_chan_bit as i32) << 1) as u8;
                channel += 1;
            }
        }
    }
}
unsafe fn Vector_Reset_Spc(p: *mut SpcPlayer) {
    unsafe {
        dsp_reset((*p).state.dsp);
        (*p).ram.fill_range(0x500..0x1500, 0);
        (*p).ram.fill_range(0x20..0x2f, 0);
        (*p).ram.fill_range(0xd0..0xef, 0);
        (*p).ram.fill_range(0x391..0x400, 0);
        (*p).ram.fill_range(0x440..0x4bf, 0);
        SpcPlayer_CopyVariablesFromRam(p);
        for i in 0i32..8 {
            (*p).state.channel[i as usize].index = i as u8;
        }
        SetupEchoParameter_EDL(p, 1);
        (*p).state.reg_FLG = (*p).state.reg_FLG | 0x20;
        Dsp_Write(p, MVOLL as i32 as u8, 0x60);
        Dsp_Write(p, MVOLR as i32 as u8, 0x60);
        Dsp_Write(p, DIR as i32 as u8, 0x6d);
        (*p).state.tempo = u16::from_ne_bytes([(*p).state.tempo.to_ne_bytes()[0], 16]);
        (*p).state.timer_cycles = 0;
    }
}
pub unsafe fn Sfx_Chan_Reset(p: *mut SpcPlayer, sr: *mut SfxRegs, s: *mut SfxChan) {
    unsafe {
        (*s).disable = 0xff;
        (*s).update_adsr = 0;
        (*sr).enabled_voices = (*sr).enabled_voices & (*s).channel_mask;
        (*p).state.is_chan_on = (*p).state.is_chan_on & (*s).channel_mask;
        (*p).state.cur_chan_bit = (*p).state.cur_chan_bit | (*s).voice_bitset;
        (*p).state.key_OFF = (*p).state.key_OFF | (*s).voice_bitset;
        let c =
            (&raw mut (*p).state.channel as *mut Channel).offset(((*s).voice_index >> 1) as isize);
        Channel_SetInstrument_NoSetId(p, c, (*c).instrument_id);
        (*c).final_volume = (*s).volume;
        (*c).pan_flag_with_phase_invert = (*s).phase_invert;
        if (*sr).enabled_voices == 0 {
            set_fields! {[(*sr)], 0; cur_sound, priority, init_flag}
        }
    }
}
static kSfx1InstrListPtrs: [u16; 66] = [
    0x2b71, 0x2baf, 0x2bb7, 0x2bc4, 0x2bd1, 0x2bfc, 0x2c2f, 0x2c37, 0x2cff, 0x2d12, 0x2d1a, 0x2d27,
    0x2d4b, 0x2d5d, 0x2d5f, 0x2d76, 0x2d95, 0x2d97, 0x2d99, 0x2da6, 0x2da8, 0x2daa, 0x2dac, 0x2dc8,
    0x2de7, 0x2dfe, 0x2e00, 0x2e17, 0x2e19, 0x2e1b, 0x2e1d, 0x2e34, 0x2e36, 0x2e38, 0x2e3a, 0x2e68,
    0x2eaf, 0x2ed0, 0x2ef6, 0x2f2c, 0x2f37, 0x2f3f, 0x2f47, 0x2f4a, 0x2f4d, 0x2f50, 0x2fb0, 0x2fb8,
    0x2fc3, 0x2fdd, 0x2fe5, 0x3040, 0x3048, 0x3055, 0x305d, 0x3065, 0x3070, 0x3078, 0x307b, 0x308b,
    0x309b, 0x30a8, 0x30d6, 0x30e1, 0x312a, 0x313f,
];
static kSfx1Conf: [u8; 66] = [
    5, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 2, 0, 0, 2, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3,
    2, 2,
];
static kSfx2InstrListPtrs: [u16; 127] = [
    0x3ab1, 0x3ac3, 0x3ad5, 0x3af1, 0x3af3, 0x3af5, 0x3b0c, 0x3b28, 0x3b2a, 0x3b3a, 0x3b42, 0x3b5e,
    0x3b73, 0x3b85, 0x3b92, 0x3ba9, 0x3bb4, 0x3bc1, 0x3be7, 0x3c08, 0x3c33, 0x3c3b, 0x3c43, 0x3c56,
    0x3c90, 0x3cff, 0x3d46, 0x3d4e, 0x3d65, 0x3d81, 0x3d9b, 0x3da8, 0x3dbf, 0x3dd6, 0x3ded, 0x3e04,
    0x3e20, 0x3e41, 0x3e66, 0x3e94, 0x3eb0, 0x3ecc, 0x3ece, 0x3f03, 0x3f18, 0x3f20, 0x3f44, 0x3f5e,
    0x3f75, 0x3f82, 0x3f8a, 0x3f97, 0x3f9f, 0x401f, 0x4036, 0x4066, 0x407c, 0x407e, 0x4086, 0x4088,
    0x408a, 0x408c, 0x409e, 0x40a6, 0x40b1, 0x40b4, 0x40bc, 0x40ce, 0x40d1, 0x40de, 0x40f5, 0x410c,
    0x412c, 0x4143, 0x415a, 0x4162, 0x416f, 0x41a9, 0x41c7, 0x41d9, 0x41ee, 0x421c, 0x4224, 0x4236,
    0x424d, 0x4268, 0x4288, 0x429a, 0x42af, 0x42bf, 0x42d4, 0x42d6, 0x42fe, 0x4310, 0x4322, 0x4334,
    0x433c, 0x4347, 0x4352, 0x43c1, 0x43cc, 0x43e8, 0x43fa, 0x4422, 0x442f, 0x4441, 0x4443, 0x4446,
    0x444e, 0x446f, 0x447f, 0x448f, 0x449c, 0x44a4, 0x44b9, 0x44ce, 0x44ed, 0x4598, 0x459a, 0x4618,
    0x462d, 0x4642, 0x4657, 0x466c, 0x4679, 0x4686, 0x468e,
];
static kSfx2Conf: [u8; 127] = [
    1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 2, 2, 0, 1, 0, 3, 1, 0,
    0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 3, 1, 3, 0, 0, 0, 0, 0, 0, 1, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 3, 3, 0, 0, 1, 0, 2, 0, 2, 3, 2, 0, 0, 0, 0, 0, 0,
    0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, 3, 3, 3, 3, 0, 3, 2, 2, 2, 2, 1, 1, 1, 2,
];
static kSfx3InstrListPtrs: [u16; 47] = [
    0x4eed, 0x4ef5, 0x4f00, 0x4f4b, 0x4f5b, 0x4f6b, 0x4f73, 0x4f89, 0x4f9f, 0x4fb6, 0x4fe1, 0x4ff7,
    0x4fff, 0x500c, 0x5047, 0x50a8, 0x50c4, 0x50d1, 0x50d4, 0x50f0, 0x511a, 0x5130, 0x5142, 0x5175,
    0x5178, 0x5188, 0x518b, 0x51b9, 0x51c1, 0x51d4, 0x51f0, 0x51fd, 0x5200, 0x5208, 0x5229, 0x5231,
    0x5239, 0x5241, 0x524e, 0x5256, 0x5277, 0x5293, 0x52a5, 0x52b0, 0x52c6, 0x52f1, 0x530b,
];
static kSfx3Conf: [u8; 47] = [
    0, 1, 2, 3, 3, 2, 4, 4, 2, 5, 3, 2, 2, 4, 3, 2, 2, 5, 2, 4, 4, 2, 2, 2, 4, 2, 4, 2, 2, 2, 2, 5,
    5, 2, 2, 5, 2, 2, 2, 2, 2, 2, 2, 4, 2, 4, 2,
];
pub unsafe fn SfxChan_GetByte(p: *mut SpcPlayer, s: *mut SfxChan) -> u8 {
    unsafe {
        crate::update_place! { previous fresh31 = (*s).ptr_idx => (*s).ptr_idx.wrapping_add(1); }
        (*p).ram
            .read_at(((*s).ptr as i32 + fresh31 as i32) as usize)
    }
}
pub unsafe fn Sfx_RunChan(p: *mut SpcPlayer, sr: *mut SfxRegs, s: *mut SfxChan) {
    unsafe {
        if (*s).disable as i32 == 0xff {
            return;
        }
        (*s).instr_timer = (*s).instr_timer.wrapping_sub(1);
        if (*s).instr_timer as i32 == 0 {
            if (*s).legato == 0 {
                set_fields! {[(*s)], 0; enable_pitch_slide, subnote_delta, target_note}
                if (*s).release_flag as i32 != 0xff {
                    (*p).state.key_OFF = (*p).state.key_OFF | (*s).voice_bitset;
                    (*s).release_timer = 2;
                    (*s).instr_timer = 1;
                    (*s).release_flag = 0xff;
                }
                (*s).release_timer = (*s).release_timer.wrapping_sub(1);
                if (*s).release_timer != 0 {
                    return;
                }
                (*s).release_flag = 0;
                (*p).state.cur_chan_bit = (*p).state.cur_chan_bit & (*s).channel_mask;
                (*p).state.reg_NON = (*p).state.reg_NON & (*s).channel_mask;
            }
            let mut a: u8 = 0;
            loop {
                a = SfxChan_GetByte(p, s);
                if a as i32 == 0xf9 {
                    (*s).adsr_settings = SfxChan_GetByte(p, s) as u16;
                    (*s).adsr_settings =
                        ((*s).adsr_settings as i32 | (SfxChan_GetByte(p, s) as i32) << 8) as u16;
                    (*s).ptr_idx = ((*s).ptr_idx as i32 + 2) as u8;
                    (*s).update_adsr = 0xff;
                } else {
                    if a as i32 == 0xf5 || a as i32 == 0xf8 {
                        (*s).enable_pitch_slide_legato =
                            (if a as i32 == 0xf5 { 0xf5 } else { 0 }) as u8;
                        (*s).subnote_delta = SfxChan_GetByte(p, s);
                        (*s).target_note = SfxChan_GetByte(p, s);
                        (*s).enable_pitch_slide = 0xff;
                        a = SfxChan_GetByte(p, s);
                    }
                    if a as i32 == 0xff {
                        Sfx_Chan_Reset(p, sr, s);
                        return;
                    }
                    if a as i32 == 0xfe {
                        (*s).repeat_ctr = SfxChan_GetByte(p, s);
                        (*s).repeat_pt = (*s).ptr_idx;
                        a = SfxChan_GetByte(p, s);
                    }
                    if a as i32 == 0xfb || a as i32 == 0xfd {
                        if a as i32 != 0xfb && {
                            (*s).repeat_ctr = (*s).repeat_ctr.wrapping_sub(1);
                            (*s).repeat_ctr as i32 == 0
                        } {
                            continue;
                        }
                        (*s).ptr_idx = (*s).repeat_pt;
                        a = SfxChan_GetByte(p, s);
                    }
                    if a as i32 != 0xfc {
                        break;
                    }
                    (*p).state.reg_NON = (*p).state.reg_NON | (*s).voice_bitset;
                }
            }
            let c = (&raw mut (*p).state.channel as *mut Channel)
                .offset(((*s).voice_index >> 1) as isize);
            Channel_SetInstrument_NoSetId(p, c, a);
            (*c).final_volume = SfxChan_GetByte(p, s);
            (*c).pan_flag_with_phase_invert = 0;
            WriteVolumeToDsp(p, c, ((SfxChan_GetByte(p, s) as i32) << 8) as u16);
            a = SfxChan_GetByte(p, s);
            if a as i32 != 0xf6 {
                (*s).note = a;
                (*s).subnote = 0;
            }
            WritePitchInner(p, c, (((*s).note as i32) << 8 | (*s).subnote as i32) as u16);
            (*s).instr_timer = SfxChan_GetByte(p, s);
            if (*s).update_adsr != 0 {
                Dsp_Write(
                    p,
                    ((*s).voice_index_mult8 as i32 | V0ADSR1 as i32) as u8,
                    (*s).adsr_settings as u8,
                );
                Dsp_Write(
                    p,
                    ((*s).voice_index_mult8 as i32 | V0ADSR2 as i32) as u8,
                    ((*s).adsr_settings >> 8) as u8,
                );
            }
            if (*s).legato == 0 {
                (*p).state.key_ON = (*p).state.key_ON | (*s).voice_bitset;
            }
        }
        if (*s).enable_pitch_slide as i32 != 0xff {
            return;
        }
        if (*s).enable_pitch_slide_legato != 0 {
            (*s).legato = 0xff;
        }
        if (*s).note as i32 >= (*s).target_note as i32 {
            let t: i32 = (*s).subnote as i32 - (*s).subnote_delta as i32;
            (*s).subnote = t as u8;
            if t & 256 != 0 && {
                (*s).note = (*s).note.wrapping_sub(1);
                (*s).note as i32 == (*s).target_note as i32
            } {
                (*s).legato = 0;
                (*s).enable_pitch_slide = (*s).legato;
            }
        } else {
            let t_0: i32 = (*s).subnote as i32 + (*s).subnote_delta as i32;
            (*s).subnote = t_0 as u8;
            if t_0 & 256 != 0 && {
                (*s).note = (*s).note.wrapping_add(1);
                (*s).note as i32 == (*s).target_note as i32
            } {
                (*s).legato = 0;
                (*s).enable_pitch_slide = (*s).legato;
            }
        }
        WritePitchInner(
            p,
            (&raw mut (*p).state.channel as *mut Channel).offset(((*s).voice_index >> 1) as isize),
            (((*s).note as i32) << 8 | (*s).subnote as i32) as u16,
        );
    }
}
unsafe fn Sfx_Init(p: *mut SpcPlayer, which_sfx: usize) {
    unsafe {
        let (sr, channels, _) = sound_effect_channels(p, which_sfx);
        static kSomeAddrs: [[u16; 3]; 3] = [
            [0x3a6, 0x3aa, 0x3af],
            [0x446, 0x448, 0x44b],
            [0x47e, 0x480, 0x483],
        ];
        set_fields! {[(*sr)]; voice_id = 9, voices_remaining = (*p).state.is_chan_on, init_flag = 0xff,
        chan_idx_x2 = 0, chan_idx = (*sr).chan_idx_x2, };
        {
            let channels: &mut [SfxChan] = match which_sfx {
                0 => &mut (*p).state.sfx_chans_1,
                1 => &mut (*p).state.sfx_chans_2,
                _ => &mut (*p).state.sfx_chans_3,
            };
            for channel in channels {
                channel.voice_bitset = 0;
                channel.voice_index = 0;
                channel.channel_mask = 0xff;
                channel.disable = 0xff;
            }
        }
        loop {
            (*sr).voice_id = (*sr).voice_id.wrapping_sub(1);
            if (*sr).voice_id == 0 {
                break;
            }
            let t: i32 = (*sr).voices_remaining as i32;
            (*sr).voices_remaining = (((*sr).voices_remaining as i32) << 1) as u8;
            if t & 0x80 == 0 {
                if (*sr).voices_to_setup == 0 {
                    return;
                }
                (*sr).voices_to_setup = (*sr).voices_to_setup.wrapping_sub(1);
                // Imported voice counts can cross groups; preserve the native allocation cursor.
                let c = channels.offset((*sr).chan_idx as isize);
                (*c).disable = 0;
                (*sr).chan_voice_bitset_ptr = (kSomeAddrs[which_sfx as usize][0i32 as usize] as i32
                    + (*sr).chan_idx as i32) as u16;
                (*sr).chan_voice_mask_ptr = (kSomeAddrs[which_sfx as usize][1i32 as usize] as i32
                    + (*sr).chan_idx as i32) as u16;
                (*sr).chan_voice_index_ptr = (kSomeAddrs[which_sfx as usize][2i32 as usize] as i32
                    + (*sr).chan_idx as i32) as u16;
                (*sr).chan_idx = ((*sr).chan_idx as i32 + 1) as u8;
                (*sr).chan_idx_x2 = ((*sr).chan_idx_x2 as i32 + 2) as u8;
                (*sr).voice_index = (((*sr).voice_id as i32 - 1i32) * 2) as u8;
                (*c).voice_index = (*sr).voice_index;
                let t_0: i32 = (*sr).voice_id as i32 - 1i32;
                (*c).volume = ((*p).state.channel[t_0 as usize].final_volume as i32
                    | ((*p).state.channel[t_0 as usize].pan_flag_with_phase_invert as i32) << 8)
                    as u8;
                let m: i32 = (1i32) << t_0;
                (*p).state.is_chan_on = ((*p).state.is_chan_on as i32 | m) as u8;
                (*p).state.cur_chan_bit = ((*p).state.cur_chan_bit as i32 & !m) as u8;
                (*p).state.reg_EON = ((*p).state.reg_EON as i32 & !m) as u8;
                (*sr).enabled_voices = ((*sr).enabled_voices as i32 | m) as u8;
                (*c).voice_bitset = m as u8;
                (*c).channel_mask = !m as u8;
            }
        }
    }
}
crate::forward_calls! { pub unsafe fn (p: *mut SpcPlayer);
    Sfx1_Init => Sfx_Init(p, 0);
    Sfx2_Init => Sfx_Init(p, 1);
    Sfx3_Init => Sfx_Init(p, 2);
}
/// Own the native player and its separate DSP without moving initialized nodes.
/// Raw views borrow these allocations; live native references must be exclusive.
pub(crate) struct NativePlayer {
    player: Rc<UnsafeCell<SpcPlayer>>,
    _dsp: Rc<UnsafeCell<Dsp>>,
}
#[forbid(unsafe_code)]
impl NativePlayer {
    pub(crate) fn new() -> Self {
        let ram = crate::apu_ram::ApuRam::new();
        let dsp = Rc::new(UnsafeCell::new(Dsp::new(ram.clone())));
        let state = NativePlayerState {
            dsp: dsp.get(),
            ram: ram.as_mut_ptr(),
            ..NativePlayerState::INITIAL
        };
        Self {
            player: Rc::new(UnsafeCell::new(SpcPlayer { state, ram })),
            _dsp: dsp,
        }
    }

    pub(crate) fn view(&self) -> *mut SpcPlayer {
        self.player.get()
    }
}
pub unsafe fn SpcPlayer_Initialize(p: *mut SpcPlayer) {
    unsafe {
        Vector_Reset_Spc(p);
        Spc_Loop_Part1(p);
    }
}
pub unsafe fn SpcPlayer_CopyVariablesToRam(p: *mut SpcPlayer) {
    unsafe {
        crate::native_player_ram::save(&(*p).state, &(*p).ram);
    }
}
pub unsafe fn SpcPlayer_CopyVariablesFromRam(p: *mut SpcPlayer) {
    unsafe {
        crate::native_player_ram::load(&mut (*p).state, &(*p).ram);
    }
}
pub unsafe fn SpcPlayer_GenerateSamples(p: *mut SpcPlayer) {
    unsafe {
        crate::native_assertion! {
            '_c2rust_label: (*p).state.timer_cycles as i32 <= 64;
            b"p->timer_cycles <= 64\0", b"src/spc_player.c\0", 1550,
            b"void SpcPlayer_GenerateSamples(SpcPlayer *)\0"
        };
        crate::native_assertion! {
            '_c2rust_label_0: (*(*p).state.dsp).state.sampleOffset as i32 <= 534;
            b"p->dsp->sampleOffset <= 534\0", b"src/spc_player.c\0", 1552,
            b"void SpcPlayer_GenerateSamples(SpcPlayer *)\0"
        };
        do_until! { (*(*p).state.dsp).state.sampleOffset as i32 == 534;
            if (*p).state.timer_cycles as i32 >= 64 {
                Spc_Loop_Part2(p, (*p).state.timer_cycles >> 6);
                Spc_Loop_Part1(p);
                (*p).state.timer_cycles = (*p).state.timer_cycles & 63;
            }
            let mut n: i32 = 534 - (*(*p).state.dsp).state.sampleOffset as i32;
            if n > 64 - (*p).state.timer_cycles as i32 {
                n = 64 - (*p).state.timer_cycles as i32;
            }
            (*p).state.timer_cycles = ((*p).state.timer_cycles as i32 + n) as u8;
            let mut i: i32 = 0;
            while i < n {
                dsp_cycle((*p).state.dsp);
                i += 1;
            }
        }
    }
}
pub unsafe fn SpcPlayer_Upload(p: *mut SpcPlayer, image: crate::rom::RomImage, source: usize) {
    unsafe {
        Dsp_Write(p, EVOLL as i32 as u8, 0);
        Dsp_Write(p, EVOLR as i32 as u8, 0);
        Dsp_Write(p, KOF as i32 as u8, 0xff);
        crate::audio_upload::transfer(image, source, |target, value| {
            (*p).ram.write(target, value);
        });
        (*p).state.port_to_snes[0i32 as usize] = 0;
        (*p).state.input_ports[3i32 as usize] = 255;
        (*p).state.input_ports[2i32 as usize] = (*p).state.input_ports[3i32 as usize];
        (*p).state.input_ports[1i32 as usize] = (*p).state.input_ports[2i32 as usize];
        (*p).state.input_ports[0i32 as usize] = (*p).state.input_ports[1i32 as usize];
        (*p).state.music_ptr_toplevel =
            u16::from_ne_bytes([(*p).ram.read_at(0x581e), (*p).ram.read_at(0x581f)]);
        (*p).state.counter_sf0c = 2;
        (*p).state.key_OFF = (*p).state.key_OFF | !((*p).state.is_chan_on);
    }
}

// The three command ports own four, two and two independent sound voices.
unsafe fn sound_effect_channels(
    p: *mut SpcPlayer,
    group: usize,
) -> (*mut SfxRegs, *mut SfxChan, usize) {
    unsafe {
        match group {
            0 => (
                &raw mut (*p).state.sfx1,
                &raw mut (*p).state.sfx_chans_1[0],
                4,
            ),
            1 => (
                &raw mut (*p).state.sfx2,
                &raw mut (*p).state.sfx_chans_2[0],
                2,
            ),
            2 => (
                &raw mut (*p).state.sfx3,
                &raw mut (*p).state.sfx_chans_3[0],
                2,
            ),
            _ => unreachable!(),
        }
    }
}

unsafe fn handle_sound_effect_command(p: *mut SpcPlayer, group: usize) {
    unsafe {
        crate::verify_probe::hit(["sfx.command.1", "sfx.command.2", "sfx.command.3"][group]);
        let (regs, channels, count) = sound_effect_channels(p, group);
        let sound = (*p).state.input_ports[group + 1];
        (*p).state.input_ports[group + 1] = 255;
        if sound == 255
            || sound == 0
            || match group {
                0 => sound != 1 && sound != 2 && (*regs).priority != 0,
                1 => sound != 0x71 && sound != 0x7e && (*regs).priority != 0,
                _ => sound != 1 && ((*regs).some_value == 2 || sound != 2 && (*regs).priority != 0),
            }
        {
            if (*regs).cur_sound != 0 {
                process_sound_effect(p, group);
            }
            return;
        }
        crate::verify_probe::hit(["sfx.start.1", "sfx.start.2", "sfx.start.3"][group]);
        if (*regs).cur_sound != 0 {
            (*regs).enabled_voices = 0;
            for i in 0..count {
                Sfx_Chan_Reset(p, regs, channels.add(i));
            }
        }
        for i in 0..count {
            (*channels.add(i)).legato = 0;
        }
        (*regs).cur_sound_index = ((sound as i32 - 1) * 2) as u8;
        let (instructions, configuration): (&[u16], &[u8]) = match group {
            0 => (&kSfx1InstrListPtrs, &kSfx1Conf),
            1 => (&kSfx2InstrListPtrs, &kSfx2Conf),
            _ => (&kSfx3InstrListPtrs, &kSfx3Conf),
        };
        (*regs).cur_ptr = instructions[((*regs).cur_sound_index >> 1) as usize];
        (*regs).cur_sound = sound;
        if group == 2 && (*regs).cur_sound >= 0xfe {
            process_sound_effect(p, group);
            return;
        }
        // Port 3's first two configurations also control its interruption gate.
        let (voices, priority) = match (group, configuration[(sound - 1) as usize]) {
            (2, 0) => {
                (*regs).voices_to_setup = 1;
                (*regs).some_value = 0;
                (*regs).priority = 1;
                return;
            }
            (2, 1) => {
                (*regs).voices_to_setup = 1;
                (*regs).some_value = 2;
                return;
            }
            (2, 2) | (0 | 1, 0) => (1, 0),
            (2, 3) | (0 | 1, 2) => (2, 0),
            (2, 4) | (1, 3) => (2, 1),
            (2, 5) | (0 | 1, 1) => (1, 1),
            (0, 3) => (3, 1),
            (0, 4 | 5) => (4, 0),
            _ => return,
        };
        (*regs).voices_to_setup = voices;
        (*regs).priority = priority;
    }
}

unsafe fn process_sound_effect(p: *mut SpcPlayer, group: usize) {
    unsafe {
        crate::verify_probe::hit(["sfx.process.1", "sfx.process.2", "sfx.process.3"][group]);
        let (regs, channels, count) = sound_effect_channels(p, group);
        if (*regs).init_flag != 0xff {
            crate::verify_probe::hit(["sfx.init.1", "sfx.init.2", "sfx.init.3"][group]);
            Sfx_Init(p, group);
            for i in 0..count {
                let channel = channels.add(i);
                let address = usize::from((*regs).cur_ptr) + i * 2;
                (*channel).ptr =
                    u16::from_ne_bytes([(*p).ram.read_at(address), (*p).ram.read_at(address + 1)]);
                (*channel).voice_index_mult8 = ((*channel).voice_index as i32 * 8) as u8;
                (*channel).ptr_idx = 0;
                (*channel).instr_timer = 1;
            }
        }
        for i in 0..count {
            Sfx_RunChan(p, regs, channels.add(i));
        }
    }
}

crate::forward_calls! { pub unsafe fn (p: *mut SpcPlayer);
    Sfx1_HandleCmdFromSnes => handle_sound_effect_command(p, 0);
    Sfx2_HandleCmdFromSnes => handle_sound_effect_command(p, 1);
    Sfx3_HandleCmdFromSnes => handle_sound_effect_command(p, 2);
    Sfx1_Process => process_sound_effect(p, 0);
    Sfx2_Process => process_sound_effect(p, 1);
    Sfx3_Process => process_sound_effect(p, 2);
}
