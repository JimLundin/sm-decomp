use crate::snes::dsp::{Dsp, dsp_cycle, dsp_reset, dsp_write};
pub use crate::types::{
    Channel, DspRegWriteHistory, NativePlayerState, SfxChan, SfxRegs, SpcPlayer, size_t,
};
pub use sequencer::{
    Music as MusicSequencer, Sfx as SfxChannelSequencer, SfxAction, SfxGroupAction,
    SfxGroups as SfxGroupSequencer,
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
// The output allocations are external to NativePlayerState. Capture their links
// before borrowing sequencing state; writes never reenter the player owner.
unsafe fn write_register(hist: *mut DspRegWriteHistory, dsp: *mut Dsp, mut reg: u8, value: u8) {
    unsafe {
        if !hist.is_null() && (*hist).count < 256 {
            if (*hist).count == 9 && reg as i32 == 0x5d {
                reg = (reg as i32 + 0) as u8;
            }
            (*hist).addr[(*hist).count as usize] = reg;
            (*hist).val[(*hist).count as usize] = value;
            (*hist).count = (*hist).count.wrapping_add(1);
        }
        if !dsp.is_null() {
            dsp_write(dsp, reg, value);
        }
    }
}
pub unsafe fn Dsp_Write(p: *mut SpcPlayer, reg: u8, value: u8) {
    unsafe {
        write_register((*p).state.reg_write_history, (*p).state.dsp, reg, value);
    }
}
pub unsafe fn WritePitchInner(p: *mut SpcPlayer, c: *mut Channel, pitch: u16) {
    unsafe {
        sequencer::pitch(
            (*c).instrument_pitch_base,
            pitch,
            || (*p).state.cur_chan_bit & (*p).state.is_chan_on != 0,
            || (*c).index,
            &mut |reg, value| Dsp_Write(p, reg, value),
        );
    }
}
pub unsafe fn Channel_SetInstrument_NoSetId(p: *mut SpcPlayer, c: *mut Channel, instrument: u8) {
    unsafe {
        let ram = (*p).ram.clone();
        let (hist, dsp) = ((*p).state.reg_write_history, (*p).state.dsp);
        let percussion = (*p).state.percussion_base_id;
        let disabled = (*p).state.is_chan_on & (*p).state.cur_chan_bit != 0;
        let bit = (*p).state.cur_chan_bit;
        // Imported SFX cursors can alias later fields. Read only the captured
        // channel's individual fields; do not form a broad Channel reference.
        if let Some(base) = sequencer::instrument(
            &ram,
            instrument,
            percussion,
            disabled,
            &mut (*p).state.reg_FLG,
            &mut (*p).state.reg_NON,
            bit,
            || (*c).index,
            &mut |reg, value| write_register(hist, dsp, reg, value),
        ) {
            (*c).instrument_pitch_base = base;
        }
    }
}
unsafe fn WriteVolumeToDsp(p: *mut SpcPlayer, c: *mut Channel, volume: u16) {
    unsafe {
        let ram = (*p).ram.clone();
        let (hist, dsp) = ((*p).state.reg_write_history, (*p).state.dsp);
        let disabled = (*p).state.is_chan_on & (*p).state.cur_chan_bit != 0;
        sequencer::volume(
            |address| {
                let origin = (*p).state.ram.addr().checked_sub(ram.as_mut_ptr().addr());
                sequencer::read_at(&ram, origin, address)
            },
            disabled,
            volume,
            || {
                (
                    (*c).index,
                    (*c).final_volume,
                    (*c).pan_flag_with_phase_invert,
                )
            },
            &mut |reg, value| write_register(hist, dsp, reg, value),
        );
    }
}
unsafe fn SetupEchoParameter_EDL(p: *mut SpcPlayer, value: u8) {
    unsafe {
        let ram = (*p).ram.clone();
        let (hist, dsp) = ((*p).state.reg_write_history, (*p).state.dsp);
        let origin = (*p).state.ram.addr().checked_sub(ram.as_mut_ptr().addr());
        sequencer::Music::with_origin(&mut (*p).state, &ram, origin, |reg, value| {
            write_register(hist, dsp, reg, value)
        })
        .setup_echo(value);
    }
}
unsafe fn HandlePanAndSweep(p: *mut SpcPlayer, channel: usize) {
    unsafe {
        let ram = (*p).ram.clone();
        let (hist, dsp) = ((*p).state.reg_write_history, (*p).state.dsp);
        let origin = (*p).state.ram.addr().checked_sub(ram.as_mut_ptr().addr());
        sequencer::Music::with_origin(&mut (*p).state, &ram, origin, |reg, value| {
            write_register(hist, dsp, reg, value)
        })
        .pan_and_sweep(channel);
    }
}
unsafe fn Music_HandleCmdFromSnes(p: *mut SpcPlayer) {
    unsafe {
        let ram = (*p).ram.clone();
        let (hist, dsp) = ((*p).state.reg_write_history, (*p).state.dsp);
        let origin = (*p).state.ram.addr().checked_sub(ram.as_mut_ptr().addr());
        sequencer::Music::with_origin(&mut (*p).state, &ram, origin, |reg, value| {
            write_register(hist, dsp, reg, value)
        })
        .tick();
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
                if (*p).state.channel[channel]
                    .pattern_order_ptr_for_chan
                    .to_ne_bytes()[1]
                    != 0
                {
                    HandlePanAndSweep(p, channel);
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
        sequencer::prepare_initialization(&mut (*p).state, &(*p).ram);
        SetupEchoParameter_EDL(p, 1);
        // These three scalar borrows are disjoint from the live output links.
        // Each write retains the original fresh history/DSP selection.
        sequencer::finish_initialization(
            &mut (*p).state.reg_FLG,
            &mut (*p).state.tempo,
            &mut (*p).state.timer_cycles,
            |register, value| Dsp_Write(p, register, value),
        );
    }
}
// Only fixed, disjoint SFX group fields reach these private adapters.
// Off-table Channel effects run after every sequencing-state borrow ends.
// Only imported off-table resets reach this original native field adapter.
unsafe fn Sfx_Chan_Reset(p: *mut SpcPlayer, group: usize, index: usize, channel: usize) {
    unsafe {
        let (sr, channels, _) = sound_effect_channels(p, group);
        let s = channels.add(index);
        let c = (&raw mut (*p).state.channel as *mut Channel).offset(channel as isize);
        Channel_SetInstrument_NoSetId(p, c, (*c).instrument_id);
        (*c).final_volume = (*s).volume;
        (*c).pan_flag_with_phase_invert = (*s).phase_invert;
        sequencer::Sfx::finish_reset(&mut *sr);
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
unsafe fn SfxChan_GetByte(p: *mut SpcPlayer, s: *mut SfxChan) -> u8 {
    unsafe { sequencer::Sfx::read_byte(&(*p).ram, &mut *s) }
}
unsafe fn Sfx_RunChan(p: *mut SpcPlayer, group: usize, index: usize) {
    unsafe {
        let effect = {
            let (state, ram) = (&mut (*p).state, &(*p).ram);
            sequencer::Sfx::step_control(
                state,
                ram,
                group,
                index,
                |state| state.ram.addr().checked_sub(ram.as_mut_ptr().addr()),
                &mut |state: &NativePlayerState| {
                    let (hist, dsp) = (state.reg_write_history, state.dsp);
                    move |reg, value| write_register(hist, dsp, reg, value)
                },
            )
        };
        match effect {
            None => {}
            Some(sequencer::SfxNativeEffect::Reset(channel)) => {
                Sfx_Chan_Reset(p, group, index, channel);
            }
            Some(sequencer::SfxNativeEffect::Pitch(channel, pitch)) => {
                WritePitchInner(
                    p,
                    (&raw mut (*p).state.channel as *mut Channel).offset(channel as isize),
                    pitch,
                );
            }
            Some(sequencer::SfxNativeEffect::Note(channel, a)) => {
                // This block ends every native cursor before owned state resumes.
                {
                    let (_, channels, _) = sound_effect_channels(p, group);
                    let s = channels.add(index);
                    let c = (&raw mut (*p).state.channel as *mut Channel).offset(channel as isize);
                    Channel_SetInstrument_NoSetId(p, c, a);
                    (*c).final_volume = SfxChan_GetByte(p, s);
                    (*c).pan_flag_with_phase_invert = 0;
                    WriteVolumeToDsp(p, c, ((SfxChan_GetByte(p, s) as i32) << 8) as u16);
                    let note = SfxChan_GetByte(p, s);
                    sequencer::Sfx::set_note(&mut *s, note);
                    WritePitchInner(p, c, (((*s).note as i32) << 8 | (*s).subnote as i32) as u16);
                    (*s).instr_timer = SfxChan_GetByte(p, s);
                    sequencer::Sfx::finish_note(&*s, &mut (*p).state.key_ON, |reg, value| {
                        Dsp_Write(p, reg, value)
                    });
                }
                let pitch = {
                    sequencer::Sfx::finish_slide(
                        &mut (*p).state,
                        group,
                        index,
                        &mut |state: &NativePlayerState| {
                            let (hist, dsp) = (state.reg_write_history, state.dsp);
                            move |reg, value| write_register(hist, dsp, reg, value)
                        },
                    )
                };
                if let Some((channel, pitch)) = pitch {
                    WritePitchInner(
                        p,
                        (&raw mut (*p).state.channel as *mut Channel).offset(channel as isize),
                        pitch,
                    );
                }
            }
        }
    }
}
unsafe fn Sfx_Init(p: *mut SpcPlayer, which_sfx: usize) {
    unsafe {
        sequencer::SfxGroups::initialize(&mut (*p).state, which_sfx);
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
            ..NativePlayerState::default()
        };
        Self {
            player: Rc::new(UnsafeCell::new(SpcPlayer { state, ram })),
            _dsp: dsp,
        }
    }

    // These allocations have one owner; native views never clone their Rc.
    pub(crate) fn apply_ports(&mut self, writes: [u8; 4]) {
        let player = Rc::get_mut(&mut self.player)
            .expect("player owner must not be shared")
            .get_mut();
        crate::apu_queue::ApuQueue::apply_ports(writes, &mut player.state.input_ports);
    }

    pub(crate) fn save_state(&mut self, queued: Option<&crate::apu_queue::ApuQueue>) {
        let player = Rc::get_mut(&mut self.player)
            .expect("player owner must not be shared")
            .get_mut();
        if let Some(queued) = queued {
            queued.apply_queued(&mut player.state.input_ports);
        }
        crate::native_player_ram::save(&player.state, &player.ram);
    }
    pub(crate) fn load_state(&mut self) {
        let player = Rc::get_mut(&mut self.player)
            .expect("player owner must not be shared")
            .get_mut();
        crate::native_player_ram::load(&mut player.state, &player.ram);
    }
    pub(crate) fn view(&self) -> *mut SpcPlayer {
        self.player.get()
    }
}
#[forbid(unsafe_code)]
impl NativePlayer {
    /// Classify native output links before resetting or changing any state.
    pub(crate) fn try_initialize(&mut self) -> bool {
        let player = Rc::get_mut(&mut self.player)
            .expect("player owner must not be shared")
            .get_mut();
        let dsp = Rc::get_mut(&mut self._dsp)
            .expect("DSP owner must not be shared")
            .get_mut();
        if !player.state.reg_write_history.is_null()
            || !core::ptr::eq(player.state.dsp, dsp)
            || player.state.ram != player.ram.as_mut_ptr()
        {
            return false;
        }
        sequencer::PlayerOwner::new(&mut player.state, &player.ram, dsp).initialize();
        true
    }
    /// Return false before effects when selected native links/history/aliases
    /// require the caller's native interval; default playback borrows this owner.
    pub(crate) fn try_render_samples(&mut self, samples: &mut [[i16; 2]]) -> bool {
        let native = {
            let player = Rc::get_mut(&mut self.player)
                .expect("player owner must not be shared")
                .get_mut();
            let dsp = Rc::get_mut(&mut self._dsp)
                .expect("DSP owner must not be shared")
                .get_mut();
            // Classify only retained native links, assertion entries and aliases;
            // no RAM/table reads, changes or output borrows happen here.
            !player.state.reg_write_history.is_null()
                || !core::ptr::eq(player.state.dsp, dsp)
                || player.state.ram != player.ram.as_mut_ptr()
                || player.state.timer_cycles > 64
                || dsp.state.sampleOffset > 534
                || player
                    .state
                    .sfx_chans_1
                    .iter()
                    .chain(&player.state.sfx_chans_2)
                    .chain(&player.state.sfx_chans_3)
                    .any(|channel| channel.voice_index >> 1 >= 8)
        };
        if native {
            return false;
        }
        let player = Rc::get_mut(&mut self.player)
            .expect("player owner must not be shared")
            .get_mut();
        let dsp = Rc::get_mut(&mut self._dsp)
            .expect("DSP owner must not be shared")
            .get_mut();
        sequencer::PlayerOwner::new(&mut player.state, &player.ram, dsp).render(samples);
        true
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
        let ram = (*p).ram.clone();
        let (hist, dsp) = ((*p).state.reg_write_history, (*p).state.dsp);
        crate::audio_upload::install_bank(&mut (*p).state, &ram, image, source, |reg, value| {
            write_register(hist, dsp, reg, value)
        });
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
        let count = sequencer::SfxGroups::fields(&mut (*p).state, group).1.len();
        let sound = match sequencer::SfxGroups::take_command(&mut (*p).state, group) {
            sequencer::SfxGroupAction::Idle => return,
            sequencer::SfxGroupAction::Process => {
                process_sound_effect(p, group);
                return;
            }
            sequencer::SfxGroupAction::Start(sound) => sound,
        };
        crate::verify_probe::hit(["sfx.start.1", "sfx.start.2", "sfx.start.3"][group]);
        if sequencer::SfxGroups::begin_replace(
            sequencer::SfxGroups::fields(&mut (*p).state, group).0,
        ) {
            for i in 0..count {
                let channel = {
                    let (state, ram) = (&mut (*p).state, &(*p).ram);
                    sequencer::Sfx::reset_control(
                        state,
                        ram,
                        group,
                        i,
                        &mut |state: &NativePlayerState| {
                            let (hist, dsp) = (state.reg_write_history, state.dsp);
                            move |reg, value| write_register(hist, dsp, reg, value)
                        },
                    )
                };
                if let Some(channel) = channel {
                    Sfx_Chan_Reset(p, group, i, channel);
                }
            }
        }
        if sequencer::SfxGroups::configure(&mut (*p).state, group, sound) {
            process_sound_effect(p, group);
        }
    }
}

unsafe fn process_sound_effect(p: *mut SpcPlayer, group: usize) {
    unsafe {
        crate::verify_probe::hit(["sfx.process.1", "sfx.process.2", "sfx.process.3"][group]);
        let count = sequencer::SfxGroups::fields(&mut (*p).state, group).1.len();
        if sequencer::SfxGroups::needs_initialize(
            sequencer::SfxGroups::fields(&mut (*p).state, group).0,
        ) {
            crate::verify_probe::hit(["sfx.init.1", "sfx.init.2", "sfx.init.3"][group]);
            Sfx_Init(p, group);
            sequencer::SfxGroups::install_pointers(&mut (*p).state, &(*p).ram, group);
        }
        for i in 0..count {
            Sfx_RunChan(p, group, i);
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

// The compatibility player owns these fields and RAM. Sequencing emits each
// register write immediately; it does not access native pointers or a device.
#[forbid(unsafe_code)]
mod sequencer {
    use super::{Channel, Dsp, NativePlayerState, SfxChan, SfxRegs};
    use super::{
        kSfx1Conf, kSfx1InstrListPtrs, kSfx2Conf, kSfx2InstrListPtrs, kSfx3Conf, kSfx3InstrListPtrs,
    };
    use crate::apu_ram::ApuRam;
    use crate::initial_state::InitialState;
    use std::cell::RefCell;

    /// The sole default playback interval borrows the existing player/RAM/DSP.
    /// Native history, links and off-table voices are selected before creation.
    pub(super) struct PlayerOwner<'a> {
        state: &'a mut NativePlayerState,
        ram: &'a ApuRam,
        dsp: RefCell<&'a mut Dsp>,
    }
    impl<'a> PlayerOwner<'a> {
        pub(super) fn new(
            state: &'a mut NativePlayerState,
            ram: &'a ApuRam,
            dsp: &'a mut Dsp,
        ) -> Self {
            Self {
                state,
                ram,
                dsp: RefCell::new(dsp),
            }
        }
        fn write(output: &RefCell<&mut Dsp>, register: u8, value: u8) {
            let mut output = output.borrow_mut();
            let output = &mut **output;
            crate::snes::dsp::write_register(
                &mut output.state,
                &output.apu_memory,
                register,
                value,
            );
        }
        pub(super) fn initialize(mut self) {
            self.dsp.get_mut().state.reset();
            prepare_initialization(self.state, self.ram);
            let dsp = &self.dsp;
            Music::new(self.state, self.ram, |register, value| {
                Self::write(dsp, register, value)
            })
            .setup_echo(1);
            finish_initialization(
                &mut self.state.reg_FLG,
                &mut self.state.tempo,
                &mut self.state.timer_cycles,
                |register, value| Self::write(dsp, register, value),
            );
            self.commit_registers();
        }
        pub(super) fn render(mut self, samples: &mut [[i16; 2]]) {
            // A full534-frame entry still executes one scheduling body.
            loop {
                if self.state.timer_cycles >= 64 {
                    self.advance(self.state.timer_cycles >> 6);
                    self.commit_registers();
                    self.state.timer_cycles &= 63;
                }
                let count = (534 - self.dsp.get_mut().state.sampleOffset)
                    .min(u16::from(64 - self.state.timer_cycles));
                self.state.timer_cycles += count as u8;
                for _ in 0..count {
                    self.dsp.get_mut().cycle();
                }
                if self.dsp.get_mut().state.sampleOffset == 534 {
                    break;
                }
            }
            self.dsp.get_mut().state.write_samples(samples);
        }
        fn commit_registers(&mut self) {
            Self::write(&self.dsp, super::KOF as u8, self.state.key_OFF);
            Self::write(&self.dsp, super::PMON as u8, 0);
            Self::write(&self.dsp, super::NON as u8, self.state.reg_NON);
            Self::write(&self.dsp, super::KOF as u8, 0);
            Self::write(&self.dsp, super::KON as u8, self.state.key_ON);
            if self.state.echo_stored_time & 0x80 == 0 {
                Self::write(&self.dsp, super::FLG as u8, self.state.reg_FLG);
                if self.state.echo_stored_time == self.state.echo_parameter_EDL {
                    Self::write(&self.dsp, super::EON as u8, self.state.reg_EON);
                    Self::write(&self.dsp, super::EFB as u8, self.state.reg_EFB);
                    Self::write(
                        &self.dsp,
                        super::EVOLR as u8,
                        (self.state.echo_volume_right >> 8) as u8,
                    );
                    Self::write(
                        &self.dsp,
                        super::EVOLL as u8,
                        (self.state.echo_volume_left >> 8) as u8,
                    );
                }
            }
            self.state.key_ON = 0;
            self.state.key_OFF = self.state.key_ON;
        }
        fn advance(&mut self, ticks: u8) {
            let mut total =
                u16::from(self.state.sfx_timer_accum) + u16::from(ticks.wrapping_mul(0x20));
            self.state.sfx_timer_accum = total as u8;
            if total >= 0x100 {
                for group in 0..3 {
                    self.command(group);
                }
                if self.state.echo_stored_time != self.state.echo_parameter_EDL {
                    self.state.echo_stored_time = self.state.echo_stored_time.wrapping_add(1);
                }
            }
            total = u16::from(self.state.main_tempo_accum)
                + u16::from(ticks.wrapping_mul(self.state.tempo.to_ne_bytes()[1]));
            self.state.main_tempo_accum = total as u8;
            if total >= 256 {
                loop {
                    let dsp = &self.dsp;
                    Music::new(self.state, self.ram, |register, value| {
                        Self::write(dsp, register, value)
                    })
                    .tick();
                    if self.state.fast_forward == 0 {
                        break;
                    }
                }
            } else if self.state.port_to_snes[0] != 0 {
                let mut channel = 0;
                self.state.cur_chan_bit = 1;
                while self.state.cur_chan_bit != 0 {
                    if self.state.channel[channel]
                        .pattern_order_ptr_for_chan
                        .to_ne_bytes()[1]
                        != 0
                    {
                        let dsp = &self.dsp;
                        Music::new(self.state, self.ram, |register, value| {
                            Self::write(dsp, register, value)
                        })
                        .pan_and_sweep(channel);
                    }
                    self.state.cur_chan_bit = self.state.cur_chan_bit.wrapping_shl(1);
                    channel += 1;
                }
            }
        }
        fn command(&mut self, group: usize) {
            crate::verify_probe::hit(["sfx.command.1", "sfx.command.2", "sfx.command.3"][group]);
            let count = SfxGroups::fields(self.state, group).1.len();
            let sound = match SfxGroups::take_command(self.state, group) {
                SfxGroupAction::Idle => return,
                SfxGroupAction::Process => {
                    self.process(group);
                    return;
                }
                SfxGroupAction::Start(sound) => sound,
            };
            crate::verify_probe::hit(["sfx.start.1", "sfx.start.2", "sfx.start.3"][group]);
            if SfxGroups::begin_replace(SfxGroups::fields(self.state, group).0) {
                for index in 0..count {
                    let dsp = &self.dsp;
                    let effect = Sfx::reset_control(
                        self.state,
                        self.ram,
                        group,
                        index,
                        &mut |_: &NativePlayerState| {
                            move |register, value| Self::write(dsp, register, value)
                        },
                    );
                    assert!(
                        effect.is_none(),
                        "native SFX alias requires native playback"
                    );
                }
            }
            if SfxGroups::configure(self.state, group, sound) {
                self.process(group);
            }
        }
        fn process(&mut self, group: usize) {
            crate::verify_probe::hit(["sfx.process.1", "sfx.process.2", "sfx.process.3"][group]);
            let count = SfxGroups::fields(self.state, group).1.len();
            if SfxGroups::needs_initialize(SfxGroups::fields(self.state, group).0) {
                crate::verify_probe::hit(["sfx.init.1", "sfx.init.2", "sfx.init.3"][group]);
                SfxGroups::initialize(self.state, group);
                SfxGroups::install_pointers(self.state, self.ram, group);
            }
            for index in 0..count {
                let dsp = &self.dsp;
                let effect = Sfx::step_control(
                    self.state,
                    self.ram,
                    group,
                    index,
                    |_| Some(0),
                    &mut |_: &NativePlayerState| {
                        move |register, value| Self::write(dsp, register, value)
                    },
                );
                assert!(
                    effect.is_none(),
                    "native SFX alias requires native playback"
                );
            }
        }
    }

    /// RAM transfer has no callbacks; load already restores every channel index.
    pub(super) fn prepare_initialization(state: &mut NativePlayerState, ram: &ApuRam) {
        ram.fill_range(0x500..0x1500, 0);
        ram.fill_range(0x20..0x2f, 0);
        ram.fill_range(0xd0..0xef, 0);
        ram.fill_range(0x391..0x400, 0);
        ram.fill_range(0x440..0x4bf, 0);
        crate::native_player_ram::load(state, ram);
    }
    /// Keep the tempo read and timer store after all immediate output writes.
    pub(super) fn finish_initialization(
        flags: &mut u8,
        tempo: &mut u16,
        timer: &mut u8,
        mut write: impl FnMut(u8, u8),
    ) {
        *flags |= 0x20;
        write(super::MVOLL as u8, 0x60);
        write(super::MVOLR as u8, 0x60);
        write(super::DIR as u8, 0x6d);
        *tempo = u16::from_ne_bytes([tempo.to_ne_bytes()[0], 16]);
        *timer = 0;
    }

    /// Group phases finish before any raw reset/step effect can alias state.
    pub enum SfxGroupAction {
        Idle,
        Process,
        Start(u8),
    }
    pub struct SfxGroups;
    impl SfxGroups {
        pub fn fields(
            state: &mut NativePlayerState,
            group: usize,
        ) -> (&mut SfxRegs, &mut [SfxChan]) {
            match group {
                0 => (&mut state.sfx1, &mut state.sfx_chans_1),
                1 => (&mut state.sfx2, &mut state.sfx_chans_2),
                2 => (&mut state.sfx3, &mut state.sfx_chans_3),
                _ => unreachable!(),
            }
        }
        fn regs(state: &mut NativePlayerState, group: usize) -> &mut SfxRegs {
            match group {
                0 => &mut state.sfx1,
                1 => &mut state.sfx2,
                2 => &mut state.sfx3,
                _ => unreachable!(),
            }
        }
        fn allocated_channel(
            state: &mut NativePlayerState,
            group: usize,
            index: usize,
        ) -> &mut SfxChan {
            let index = [0usize, 4, 6][group]
                .checked_add(index)
                .expect("SFX channel index overflow");
            if index < 4 {
                &mut state.sfx_chans_1[index]
            } else if index < 6 {
                &mut state.sfx_chans_2[index - 4]
            } else {
                &mut state.sfx_chans_3[index - 6]
            }
        }
        pub fn initialize(state: &mut NativePlayerState, group: usize) {
            const ADDRESSES: [[u16; 3]; 3] = [
                [0x3a6, 0x3aa, 0x3af],
                [0x446, 0x448, 0x44b],
                [0x47e, 0x480, 0x483],
            ];
            Self::regs(state, group).voice_id = 9;
            Self::regs(state, group).voices_remaining = state.is_chan_on;
            Self::regs(state, group).init_flag = 0xff;
            Self::regs(state, group).chan_idx_x2 = 0;
            Self::regs(state, group).chan_idx = Self::regs(state, group).chan_idx_x2;
            for channel in Self::fields(state, group).1 {
                channel.voice_bitset = 0;
                channel.voice_index = 0;
                channel.channel_mask = 0xff;
                channel.disable = 0xff;
            }
            loop {
                Self::regs(state, group).voice_id =
                    Self::regs(state, group).voice_id.wrapping_sub(1);
                if Self::regs(state, group).voice_id == 0 {
                    break;
                }
                let remaining = Self::regs(state, group).voices_remaining;
                Self::regs(state, group).voices_remaining = remaining.wrapping_shl(1);
                if remaining & 0x80 != 0 {
                    continue;
                }
                if Self::regs(state, group).voices_to_setup == 0 {
                    return;
                }
                Self::regs(state, group).voices_to_setup =
                    Self::regs(state, group).voices_to_setup.wrapping_sub(1);
                // Imported counts can select another group's typed channel.
                // The first outside-state member access is this disable store.
                let index = usize::from(Self::regs(state, group).chan_idx);
                Self::allocated_channel(state, group, index).disable = 0;
                Self::regs(state, group).chan_voice_bitset_ptr =
                    ADDRESSES[group][0] + u16::from(Self::regs(state, group).chan_idx);
                Self::regs(state, group).chan_voice_mask_ptr =
                    ADDRESSES[group][1] + u16::from(Self::regs(state, group).chan_idx);
                Self::regs(state, group).chan_voice_index_ptr =
                    ADDRESSES[group][2] + u16::from(Self::regs(state, group).chan_idx);
                Self::regs(state, group).chan_idx =
                    Self::regs(state, group).chan_idx.wrapping_add(1);
                Self::regs(state, group).chan_idx_x2 =
                    Self::regs(state, group).chan_idx_x2.wrapping_add(2);
                Self::regs(state, group).voice_index = (Self::regs(state, group).voice_id - 1) * 2;
                Self::allocated_channel(state, group, index).voice_index =
                    Self::regs(state, group).voice_index;
                let voice = usize::from(Self::regs(state, group).voice_id - 1);
                Self::allocated_channel(state, group, index).volume =
                    (u16::from(state.channel[voice].final_volume)
                        | u16::from(state.channel[voice].pan_flag_with_phase_invert) << 8)
                        as u8;
                let mask = 1u8 << voice;
                state.is_chan_on |= mask;
                state.cur_chan_bit &= !mask;
                state.reg_EON &= !mask;
                Self::regs(state, group).enabled_voices |= mask;
                Self::allocated_channel(state, group, index).voice_bitset = mask;
                Self::allocated_channel(state, group, index).channel_mask = !mask;
            }
        }
        pub fn take_command(state: &mut NativePlayerState, group: usize) -> SfxGroupAction {
            let sound = state.input_ports[group + 1];
            state.input_ports[group + 1] = 255;
            let regs = Self::regs(state, group);
            if sound == 255
                || sound == 0
                || match group {
                    0 => sound != 1 && sound != 2 && regs.priority != 0,
                    1 => sound != 0x71 && sound != 0x7e && regs.priority != 0,
                    _ => sound != 1 && (regs.some_value == 2 || sound != 2 && regs.priority != 0),
                }
            {
                if regs.cur_sound != 0 {
                    SfxGroupAction::Process
                } else {
                    SfxGroupAction::Idle
                }
            } else {
                SfxGroupAction::Start(sound)
            }
        }
        pub fn begin_replace(regs: &mut SfxRegs) -> bool {
            if regs.cur_sound == 0 {
                false
            } else {
                regs.enabled_voices = 0;
                true
            }
        }
        pub fn configure(state: &mut NativePlayerState, group: usize, sound: u8) -> bool {
            let (regs, channels) = Self::fields(state, group);
            for channel in channels {
                channel.legato = 0;
            }
            regs.cur_sound_index = sound.wrapping_sub(1).wrapping_mul(2);
            let (instructions, configuration): (&[u16], &[u8]) = match group {
                0 => (&kSfx1InstrListPtrs, &kSfx1Conf),
                1 => (&kSfx2InstrListPtrs, &kSfx2Conf),
                _ => (&kSfx3InstrListPtrs, &kSfx3Conf),
            };
            regs.cur_ptr = instructions[usize::from(regs.cur_sound_index >> 1)];
            regs.cur_sound = sound;
            if group == 2 && regs.cur_sound >= 0xfe {
                return true;
            }
            let (voices, priority) = match (group, configuration[usize::from(sound - 1)]) {
                (2, 0) => {
                    regs.voices_to_setup = 1;
                    regs.some_value = 0;
                    regs.priority = 1;
                    return false;
                }
                (2, 1) => {
                    regs.voices_to_setup = 1;
                    regs.some_value = 2;
                    return false;
                }
                (2, 2) | (0 | 1, 0) => (1, 0),
                (2, 3) | (0 | 1, 2) => (2, 0),
                (2, 4) | (1, 3) => (2, 1),
                (2, 5) | (0 | 1, 1) => (1, 1),
                (0, 3) => (3, 1),
                (0, 4 | 5) => (4, 0),
                _ => return false,
            };
            regs.voices_to_setup = voices;
            regs.priority = priority;
            false
        }
        pub fn needs_initialize(regs: &SfxRegs) -> bool {
            regs.init_flag != 0xff
        }
        pub fn install_pointers(state: &mut NativePlayerState, ram: &ApuRam, group: usize) {
            let (regs, channels) = Self::fields(state, group);
            for (index, channel) in channels.iter_mut().enumerate() {
                let address = usize::from(regs.cur_ptr) + index * 2;
                channel.ptr = u16::from_ne_bytes([ram.read_at(address), ram.read_at(address + 1)]);
                channel.voice_index_mult8 = channel.voice_index.wrapping_mul(8);
                channel.ptr_idx = 0;
                channel.instr_timer = 1;
            }
        }
    }

    /// A sequencing phase never retains a channel borrow across an adapter
    /// effect. Imported Channel cursors may alias the SFX state between phases.
    pub enum SfxAction {
        Stop,
        Slide,
        Instrument(u8),
    }
    // Owned command execution returns only captured imported effects. Native
    // field aliases are applied after the state borrow ends, then slide resumes.
    pub(super) enum SfxNativeEffect {
        Reset(usize),
        Note(usize, u8),
        Pitch(usize, u16),
    }

    pub struct Sfx;
    impl Sfx {
        pub(super) fn reset_control<W: FnMut(u8, u8)>(
            state: &mut NativePlayerState,
            ram: &ApuRam,
            group: usize,
            index: usize,
            make_write: &mut impl FnMut(&NativePlayerState) -> W,
        ) -> Option<usize> {
            let channel = Self::reset_slot(state, group, index);
            if channel >= 8 {
                return Some(channel);
            }
            let mut write = make_write(state);
            Self::restore_voice(state, ram, group, index, channel, &mut write);
            None
        }
        pub(super) fn finish_slide<W: FnMut(u8, u8)>(
            state: &mut NativePlayerState,
            group: usize,
            index: usize,
            make_write: &mut impl FnMut(&NativePlayerState) -> W,
        ) -> Option<(usize, u16)> {
            if !Self::slide(Self::slot(state, group, index)) {
                return None;
            }
            let s = Self::slot(state, group, index);
            let channel = usize::from(s.voice_index >> 1);
            let pitch = u16::from(s.note) << 8 | u16::from(s.subnote);
            if channel >= 8 {
                return Some((channel, pitch));
            }
            let mut write = make_write(state);
            pitch_slot(state, channel, pitch, &mut write);
            None
        }
        pub(super) fn step_control<W: FnMut(u8, u8)>(
            state: &mut NativePlayerState,
            ram: &ApuRam,
            group: usize,
            index: usize,
            mut origin: impl FnMut(&NativePlayerState) -> Option<usize>,
            make_write: &mut impl FnMut(&NativePlayerState) -> W,
        ) -> Option<SfxNativeEffect> {
            match Self::begin_slot(state, ram, group, index) {
                SfxAction::Stop => return None,
                SfxAction::Slide => {}
                SfxAction::Instrument(a) => {
                    if a == 0xff {
                        return Self::reset_control(state, ram, group, index, make_write)
                            .map(SfxNativeEffect::Reset);
                    }
                    let channel = usize::from(Self::slot(state, group, index).voice_index >> 1);
                    if channel >= 8 {
                        return Some(SfxNativeEffect::Note(channel, a));
                    }
                    let mut write = make_write(state);
                    Self::program_note(
                        state,
                        ram,
                        group,
                        index,
                        channel,
                        a,
                        &mut origin,
                        &mut write,
                    );
                }
            }
            Self::finish_slide(state, group, index, make_write)
                .map(|(channel, pitch)| SfxNativeEffect::Pitch(channel, pitch))
        }

        pub(super) fn slot(
            state: &mut NativePlayerState,
            group: usize,
            index: usize,
        ) -> &mut SfxChan {
            &mut SfxGroups::fields(state, group).1[index]
        }
        pub(super) fn reset_slot(
            state: &mut NativePlayerState,
            group: usize,
            index: usize,
        ) -> usize {
            let (sr, s) = match group {
                0 => (&mut state.sfx1, &mut state.sfx_chans_1[index]),
                1 => (&mut state.sfx2, &mut state.sfx_chans_2[index]),
                2 => (&mut state.sfx3, &mut state.sfx_chans_3[index]),
                _ => unreachable!(),
            };
            usize::from(Self::reset(
                s,
                sr,
                &mut state.is_chan_on,
                &mut state.cur_chan_bit,
                &mut state.key_OFF,
            ))
        }
        pub(super) fn begin_slot(
            state: &mut NativePlayerState,
            ram: &ApuRam,
            group: usize,
            index: usize,
        ) -> SfxAction {
            let s = match group {
                0 => &mut state.sfx_chans_1[index],
                1 => &mut state.sfx_chans_2[index],
                2 => &mut state.sfx_chans_3[index],
                _ => unreachable!(),
            };
            Self::begin(
                ram,
                s,
                &mut state.key_OFF,
                &mut state.cur_chan_bit,
                &mut state.reg_NON,
            )
        }
        pub(super) fn restore_voice(
            state: &mut NativePlayerState,
            ram: &ApuRam,
            group: usize,
            index: usize,
            channel: usize,
            write: &mut impl FnMut(u8, u8),
        ) {
            let instrument = state.channel[channel].instrument_id;
            instrument_slot(state, ram, channel, instrument, write);
            state.channel[channel].final_volume = Self::slot(state, group, index).volume;
            state.channel[channel].pan_flag_with_phase_invert =
                Self::slot(state, group, index).phase_invert;
            Self::finish_reset(SfxGroups::regs(state, group));
        }
        pub(super) fn program_note(
            state: &mut NativePlayerState,
            ram: &ApuRam,
            group: usize,
            index: usize,
            channel: usize,
            instrument_id: u8,
            mut origin: impl FnMut(&NativePlayerState) -> Option<usize>,
            write: &mut impl FnMut(u8, u8),
        ) {
            instrument_slot(state, ram, channel, instrument_id, write);
            state.channel[channel].final_volume =
                Self::read_byte(ram, Self::slot(state, group, index));
            state.channel[channel].pan_flag_with_phase_invert = 0;
            let pan = u16::from(Self::read_byte(ram, Self::slot(state, group, index))) << 8;
            volume(
                |address| read_at(ram, origin(state), address),
                state.is_chan_on & state.cur_chan_bit != 0,
                pan,
                || {
                    (
                        state.channel[channel].index,
                        state.channel[channel].final_volume,
                        state.channel[channel].pan_flag_with_phase_invert,
                    )
                },
                write,
            );
            let note = Self::read_byte(ram, Self::slot(state, group, index));
            Self::set_note(Self::slot(state, group, index), note);
            let s = Self::slot(state, group, index);
            let pitch = u16::from(s.note) << 8 | u16::from(s.subnote);
            pitch_slot(state, channel, pitch, write);
            Self::slot(state, group, index).instr_timer =
                Self::read_byte(ram, Self::slot(state, group, index));
            let s = match group {
                0 => &state.sfx_chans_1[index],
                1 => &state.sfx_chans_2[index],
                2 => &state.sfx_chans_3[index],
                _ => unreachable!(),
            };
            Self::finish_note(s, &mut state.key_ON, write);
        }

        pub fn reset(
            s: &mut SfxChan,
            sr: &mut SfxRegs,
            on: &mut u8,
            current: &mut u8,
            key_off: &mut u8,
        ) -> u8 {
            s.disable = 0xff;
            s.update_adsr = 0;
            sr.enabled_voices &= s.channel_mask;
            *on &= s.channel_mask;
            *current |= s.voice_bitset;
            *key_off |= s.voice_bitset;
            s.voice_index >> 1
        }
        pub fn finish_reset(sr: &mut SfxRegs) {
            if sr.enabled_voices == 0 {
                sr.cur_sound = 0;
                sr.priority = 0;
                sr.init_flag = 0;
            }
        }
        pub fn read_byte(ram: &ApuRam, s: &mut SfxChan) -> u8 {
            let previous = s.ptr_idx;
            s.ptr_idx = previous.wrapping_add(1);
            ram.read_at(usize::from(s.ptr) + usize::from(previous))
        }
        pub fn begin(
            ram: &ApuRam,
            s: &mut SfxChan,
            key_off: &mut u8,
            current: &mut u8,
            noise: &mut u8,
        ) -> SfxAction {
            if s.disable == 0xff {
                return SfxAction::Stop;
            }
            s.instr_timer = s.instr_timer.wrapping_sub(1);
            if s.instr_timer != 0 {
                return SfxAction::Slide;
            }
            if s.legato == 0 {
                s.enable_pitch_slide = 0;
                s.subnote_delta = 0;
                s.target_note = 0;
                if s.release_flag != 0xff {
                    *key_off |= s.voice_bitset;
                    s.release_timer = 2;
                    s.instr_timer = 1;
                    s.release_flag = 0xff;
                }
                s.release_timer = s.release_timer.wrapping_sub(1);
                if s.release_timer != 0 {
                    return SfxAction::Stop;
                }
                s.release_flag = 0;
                *current &= s.channel_mask;
                *noise &= s.channel_mask;
            }
            loop {
                let mut a = Self::read_byte(ram, s);
                if a == 0xf9 {
                    s.adsr_settings = u16::from(Self::read_byte(ram, s));
                    s.adsr_settings =
                        (s.adsr_settings as i32 | (Self::read_byte(ram, s) as i32) << 8) as u16;
                    s.ptr_idx = s.ptr_idx.wrapping_add(2);
                    s.update_adsr = 0xff;
                    continue;
                }
                if a == 0xf5 || a == 0xf8 {
                    s.enable_pitch_slide_legato = if a == 0xf5 { 0xf5 } else { 0 };
                    s.subnote_delta = Self::read_byte(ram, s);
                    s.target_note = Self::read_byte(ram, s);
                    s.enable_pitch_slide = 0xff;
                    a = Self::read_byte(ram, s);
                }
                if a == 0xff {
                    return SfxAction::Instrument(a);
                }
                if a == 0xfe {
                    s.repeat_ctr = Self::read_byte(ram, s);
                    s.repeat_pt = s.ptr_idx;
                    a = Self::read_byte(ram, s);
                }
                if a == 0xfb || a == 0xfd {
                    if a != 0xfb {
                        s.repeat_ctr = s.repeat_ctr.wrapping_sub(1);
                        if s.repeat_ctr == 0 {
                            continue;
                        }
                    }
                    s.ptr_idx = s.repeat_pt;
                    a = Self::read_byte(ram, s);
                }
                if a != 0xfc {
                    return SfxAction::Instrument(a);
                }
                *noise |= s.voice_bitset;
            }
        }
        pub fn set_note(s: &mut SfxChan, note: u8) {
            if note != 0xf6 {
                s.note = note;
                s.subnote = 0;
            }
        }
        pub fn finish_note(s: &SfxChan, key_on: &mut u8, mut write: impl FnMut(u8, u8)) {
            if s.update_adsr != 0 {
                write(s.voice_index_mult8 | 5, s.adsr_settings as u8);
                write(s.voice_index_mult8 | 6, (s.adsr_settings >> 8) as u8);
            }
            if s.legato == 0 {
                *key_on |= s.voice_bitset;
            }
        }
        pub fn slide(s: &mut SfxChan) -> bool {
            if s.enable_pitch_slide != 0xff {
                return false;
            }
            if s.enable_pitch_slide_legato != 0 {
                s.legato = 0xff;
            }
            let carry = if s.note >= s.target_note {
                let value = i32::from(s.subnote) - i32::from(s.subnote_delta);
                s.subnote = value as u8;
                if value & 256 != 0 {
                    s.note = s.note.wrapping_sub(1);
                    true
                } else {
                    false
                }
            } else {
                let value = i32::from(s.subnote) + i32::from(s.subnote_delta);
                s.subnote = value as u8;
                if value & 256 != 0 {
                    s.note = s.note.wrapping_add(1);
                    true
                } else {
                    false
                }
            };
            if carry && s.note == s.target_note {
                s.legato = 0;
                s.enable_pitch_slide = s.legato;
            }
            true
        }
    }

    impl Default for NativePlayerState {
        fn default() -> Self {
            Self::INITIAL
        }
    }

    const EFFECT_LENGTH: [u8; 31] = [
        1, 1, 2, 3, 0, 1, 2, 1, 2, 1, 1, 3, 0, 1, 2, 3, 1, 3, 3, 0, 1, 3, 0, 3, 3, 3, 1, 2, 0, 0, 0,
    ];
    fn effect_length(effect: u8) -> u8 {
        EFFECT_LENGTH[(i32::from(effect) - 0xe0) as usize]
    }
    fn divide(mut value: i32, ticks: u8) -> u16 {
        let original = value;
        if value & 0x100 != 0 {
            value = -value;
        }
        let quotient = if ticks != 0 {
            (value & 0xff) / i32::from(ticks)
        } else {
            0xff
        };
        let remainder = if ticks != 0 {
            (value & 0xff) % i32::from(ticks)
        } else {
            value & 0xff
        };
        let result = (quotient << 8)
            + if ticks != 0 {
                (remainder << 8) / i32::from(ticks) & 0xff
            } else {
                0xff
            };
        (if original & 0x100 != 0 {
            -result
        } else {
            result
        }) as u16
    }
    fn fade(value: u16, add: u16, target: u8, remaining: u8) -> u16 {
        if remaining == 0 {
            u16::from(target) << 8
        } else {
            value.wrapping_add(add)
        }
    }
    fn high(value: u16, byte: u8) -> u16 {
        u16::from_ne_bytes([value.to_ne_bytes()[0], byte])
    }

    pub(super) fn read_at(ram: &ApuRam, origin: Option<usize>, address: usize) -> u8 {
        let origin = origin.expect("native music RAM origin precedes owned allocation");
        let address = origin
            .checked_add(address)
            .expect("native music RAM address overflow");
        ram.read_at(address)
    }
    fn next_byte(ram: &ApuRam, origin: Option<usize>, c: &mut Channel) -> u8 {
        let address = c.pattern_order_ptr_for_chan;
        c.pattern_order_ptr_for_chan = address.wrapping_add(1);
        read_at(ram, origin, usize::from(address))
    }
    fn compute_pitch_add(c: &mut Channel, pitch: u8) {
        c.pitch_target = pitch & 0x7f;
        c.pitch_add_per_tick = divide(
            i32::from(c.pitch_target) - i32::from(c.pitch >> 8),
            c.pitch_slide_length,
        );
    }

    // A slot identity selects owned storage; Channel.index is queried lazily
    // and independently to select DSP registers. No native cursor enters here.
    pub(super) fn pitch_slot(
        state: &NativePlayerState,
        channel: usize,
        value: u16,
        write: &mut impl FnMut(u8, u8),
    ) {
        pitch(
            state.channel[channel].instrument_pitch_base,
            value,
            || state.cur_chan_bit & state.is_chan_on != 0,
            || state.channel[channel].index,
            write,
        );
    }
    pub(super) fn pitch(
        base: u16,
        value: u16,
        disabled: impl FnOnce() -> bool,
        index: impl FnOnce() -> u8,
        write: &mut impl FnMut(u8, u8),
    ) {
        let value = pitch_value(base, value);
        if !disabled() {
            let reg = index().wrapping_mul(16);
            write(reg.wrapping_add(2), value as u8);
            write(reg.wrapping_add(3), (value >> 8) as u8);
        }
    }
    pub(super) fn instrument_slot(
        state: &mut NativePlayerState,
        ram: &ApuRam,
        channel: usize,
        value: u8,
        write: &mut impl FnMut(u8, u8),
    ) {
        if let Some(base) = instrument(
            ram,
            value,
            state.percussion_base_id,
            state.is_chan_on & state.cur_chan_bit != 0,
            &mut state.reg_FLG,
            &mut state.reg_NON,
            state.cur_chan_bit,
            || state.channel[channel].index,
            write,
        ) {
            state.channel[channel].instrument_pitch_base = base;
        }
    }

    pub(super) fn pitch_value(base: u16, pitch: u16) -> u16 {
        const FREQUENCY: [u16; 13] = [
            2143, 2270, 2405, 2548, 2700, 2860, 3030, 3211, 3402, 3604, 3818, 4045, 4286,
        ];
        let note = (pitch >> 8 & 0x7f) as u8;
        let mut octave = note / 12;
        let remainder = usize::from(note % 12);
        let mut value = (i32::from(FREQUENCY[remainder])
            + ((FREQUENCY[remainder + 1] as i32 - FREQUENCY[remainder] as i32) as u8 as i32
                * pitch as u8 as i32
                >> 8)) as u16;
        value = (i32::from(value) * 2) as u16;
        while octave != 6 {
            value >>= 1;
            octave = octave.wrapping_add(1);
        }
        (i32::from(base) * i32::from(value) >> 8) as u16
    }

    pub(super) fn instrument(
        ram: &ApuRam,
        mut instrument: u8,
        percussion: u8,
        disabled: bool,
        flag: &mut u8,
        noise: &mut u8,
        bit: u8,
        index: impl FnOnce() -> u8,
        write: &mut impl FnMut(u8, u8),
    ) -> Option<u16> {
        if instrument & 0x80 != 0 {
            instrument = instrument.wrapping_add(54).wrapping_add(percussion);
        }
        let address = usize::from(instrument) * 6 + 0x6c00;
        if disabled {
            return None;
        }
        let reg = index().wrapping_mul(16);
        if ram.read_at(address) & 0x80 != 0 {
            *flag = *flag & 0x20 | ram.read_at(address) & 0x1f;
            *noise |= bit;
            write(reg.wrapping_add(4), 0);
        } else {
            write(reg.wrapping_add(4), ram.read_at(address));
        }
        write(reg.wrapping_add(5), ram.read_at(address + 1));
        write(reg.wrapping_add(6), ram.read_at(address + 2));
        write(reg.wrapping_add(7), ram.read_at(address + 3));
        Some(u16::from(ram.read_at(address + 4)) << 8 | u16::from(ram.read_at(address + 5)))
    }

    pub(super) fn volume(
        mut read: impl FnMut(usize) -> u8,
        disabled: bool,
        mut volume: u16,
        mut channel: impl FnMut() -> (u8, u8, u8),
        write: &mut impl FnMut(u8, u8),
    ) {
        const TABLE: [u8; 22] = [
            0, 1, 3, 7, 13, 21, 30, 41, 52, 66, 81, 94, 103, 110, 115, 119, 122, 124, 125, 126,
            127, 127,
        ];
        if disabled {
            return;
        }
        for side in 0..2u8 {
            let position = usize::from(volume >> 8);
            let mut value = if position >= 21 {
                (i32::from(read(position + 0x1e1d))
                    + ((i32::from(read(position + 0x1e1e)) - i32::from(read(position + 0x1e1d)))
                        * i32::from(volume as u8)
                        >> 8)) as u8
            } else {
                (i32::from(TABLE[position])
                    + ((i32::from(TABLE[position + 1]) - i32::from(TABLE[position]))
                        * i32::from(volume as u8)
                        >> 8)) as u8
            };
            let (index, final_volume, phase) = channel();
            value = (u16::from(value) * u16::from(final_volume) >> 8) as u8;
            if (u16::from(phase) << side) & 0x80 != 0 {
                value = value.wrapping_neg();
            }
            write(side.wrapping_add(index.wrapping_mul(16)), value);
            volume = 0x1400u16.wrapping_sub(volume);
        }
    }

    /// Borrow the existing compatibility state and emit register writes in order.
    /// RAM reads remain lazy, including when the writer changes shared APU RAM.
    /// Invalid table/cursor/channel accesses fail through checked Rust indexing.
    pub struct Music<'a, W> {
        state: &'a mut NativePlayerState,
        ram: &'a ApuRam,
        origin: Option<usize>,
        write: W,
    }
    impl<'a, W: FnMut(u8, u8)> Music<'a, W> {
        pub fn new(state: &'a mut NativePlayerState, ram: &'a ApuRam, write: W) -> Self {
            Self::with_origin(state, ram, Some(0), write)
        }
        /// Use a compatibility RAM origin. Invalid origins fail only at a RAM
        /// read, preserving earlier state changes and operations that read none.
        pub fn with_origin(
            state: &'a mut NativePlayerState,
            ram: &'a ApuRam,
            origin: Option<usize>,
            write: W,
        ) -> Self {
            Self {
                state,
                ram,
                origin,
                write,
            }
        }
        fn word(&self, address: usize) -> u16 {
            u16::from_ne_bytes([
                read_at(self.ram, self.origin, address),
                read_at(self.ram, self.origin, address + 1),
            ])
        }
        fn byte(&mut self, channel: usize) -> u8 {
            next_byte(self.ram, self.origin, &mut self.state.channel[channel])
        }
        pub fn setup_echo(&mut self, mut value: u8) {
            self.state.echo_parameter_EDL = value;
            if value != self.state.last_written_edl {
                value = (self.state.last_written_edl & 0xf) ^ 0xff;
                if self.state.echo_stored_time & 0x80 != 0 {
                    value = value.wrapping_add(self.state.echo_stored_time);
                }
                self.state.echo_stored_time = value;
                (self.write)(77, 0);
                (self.write)(13, 0);
                (self.write)(60, 0);
                (self.write)(44, 0);
                (self.write)(108, self.state.reg_FLG | 0x20);
                self.state.last_written_edl = self.state.echo_parameter_EDL;
                (self.write)(125, self.state.echo_parameter_EDL);
            }
            (self.write)(
                109,
                ((u16::from(self.state.echo_parameter_EDL) * 8 ^ 0xff) + 0x16) as u8,
            );
        }
        fn write_volume(&mut self, channel: usize, value: u16) {
            let state = &self.state;
            let ram = self.ram;
            let origin = self.origin;
            volume(
                |address| read_at(ram, origin, address),
                state.is_chan_on & state.cur_chan_bit != 0,
                value,
                || {
                    let c = &state.channel[channel];
                    (c.index, c.final_volume, c.pan_flag_with_phase_invert)
                },
                &mut self.write,
            );
        }
        fn write_pitch(&mut self, channel: usize, mut pitch: u16) {
            if pitch >> 8 >= 0x34 {
                pitch = pitch.wrapping_add((pitch >> 8) - 0x34);
            } else if pitch >> 8 < 0x13 {
                pitch = (i32::from(pitch) + ((i32::from(pitch >> 8) - 0x13) * 2) as u8 as i32 - 256)
                    as u16;
            }
            pitch_slot(self.state, channel, pitch, &mut self.write);
        }
        fn set_instrument(&mut self, channel: usize, value: u8) {
            self.state.channel[channel].instrument_id = value;
            instrument_slot(self.state, self.ram, channel, value, &mut self.write);
        }
        fn reset_channels(&mut self) {
            self.state.cur_chan_bit = 0x80;
            for channel in (0..8).rev() {
                let c = &mut self.state.channel[channel];
                c.channel_volume = high(c.channel_volume, 0xff);
                c.pan_flag_with_phase_invert = 10;
                c.pan_value = 10 << 8;
                c.instrument_id = 0;
                c.fine_tune = 0;
                c.channel_transposition = 0;
                c.pitch_envelope_num_ticks = 0;
                c.cutk = 0;
                c.vib_depth = 0;
                c.tremolo_depth = 0;
                self.state.cur_chan_bit >>= 1;
            }
            self.state.master_volume_fade_ticks = 0;
            self.state.echo_volume_fade_ticks = 0;
            self.state.tempo_fade_num_ticks = 0;
            self.state.global_transposition = 0;
            self.state.block_count = 0;
            self.state.percussion_base_id = 0;
            self.state.master_volume = high(self.state.master_volume, 0xc0);
            self.state.tempo = high(self.state.tempo, 0x20);
        }
        fn pitch_slide(&mut self, channel: usize) {
            let c = &mut self.state.channel[channel];
            if c.pitch_slide_length != 0
                || read_at(
                    self.ram,
                    self.origin,
                    usize::from(c.pattern_order_ptr_for_chan),
                ) != 0xf9
            {
                return;
            }
            c.pattern_order_ptr_for_chan = c.pattern_order_ptr_for_chan.wrapping_add(1);
            c.pitch_slide_delay_left = next_byte(self.ram, self.origin, c);
            c.pitch_slide_length = next_byte(self.ram, self.origin, c);
            let pitch = next_byte(self.ram, self.origin, c)
                .wrapping_add(self.state.global_transposition)
                .wrapping_add(c.channel_transposition);
            compute_pitch_add(c, pitch);
        }
        fn effect(&mut self, channel: usize, effect: u8) {
            let arg = if effect_length(effect) != 0 {
                self.byte(channel)
            } else {
                0
            };
            if effect == 0xe0 {
                self.set_instrument(channel, arg);
                return;
            }
            if effect == 0xf7 {
                self.setup_echo(arg);
                self.state.reg_EFB = self.byte(channel);
                let selector = self.byte(channel);
                self.state.byte_4B1 = selector;
                // The SPC indexes live APU RAM with the low byte of selector * 8.
                let mut offset = selector.wrapping_mul(8);
                for tap in 0..8 {
                    (self.write)(
                        15 + tap as u8 * 16,
                        self.ram.read_at(0x1e32 + usize::from(offset)),
                    );
                    offset = offset.wrapping_add(1);
                }
                return;
            }
            let p = &mut self.state;
            let c = &mut p.channel[channel];
            let ram = self.ram;
            let origin = self.origin;
            match effect {
                0xe1 => {
                    c.pan_flag_with_phase_invert = arg;
                    c.pan_value = u16::from(arg & 0x1f) << 8;
                }
                0xe2 => {
                    c.pan_num_ticks = arg;
                    c.pan_target_value = next_byte(ram, origin, c);
                    c.pan_add_per_tick = divide(
                        i32::from(c.pan_target_value) - i32::from(c.pan_value >> 8),
                        arg,
                    );
                }
                0xe3 => {
                    c.vibrato_delay_ticks = arg;
                    c.vibrato_rate = next_byte(ram, origin, c);
                    c.vib_depth = next_byte(ram, origin, c);
                    c.vibrato_depth_target = c.vib_depth;
                    c.vibrato_fade_num_ticks = 0;
                }
                0xe4 => {
                    c.vib_depth = 0;
                    c.vibrato_depth_target = 0;
                    c.vibrato_fade_num_ticks = 0;
                }
                0xe5 => p.master_volume = u16::from(arg) << 8,
                0xe6 => {
                    p.master_volume_fade_ticks = arg;
                    p.master_volume_fade_target = next_byte(ram, origin, c);
                    p.master_volume_fade_add_per_tick = divide(
                        i32::from(p.master_volume_fade_target) - i32::from(p.master_volume >> 8),
                        arg,
                    );
                }
                0xe7 => p.tempo = u16::from(arg) << 8,
                0xe8 => {
                    p.tempo_fade_num_ticks = arg;
                    p.tempo_fade_final = next_byte(ram, origin, c);
                    p.tempo_fade_add =
                        divide(i32::from(p.tempo_fade_final) - i32::from(p.tempo >> 8), arg);
                }
                0xe9 => p.global_transposition = arg,
                0xea => c.channel_transposition = arg,
                0xeb => {
                    c.tremolo_delay_ticks = arg;
                    c.tremolo_rate = next_byte(ram, origin, c);
                    c.tremolo_depth = next_byte(ram, origin, c);
                }
                0xec => c.tremolo_depth = 0,
                0xed => c.channel_volume = u16::from(arg) << 8,
                0xee => {
                    c.volume_fade_ticks = arg;
                    c.volume_fade_target = next_byte(ram, origin, c);
                    c.volume_fade_addpertick = divide(
                        i32::from(c.volume_fade_target) - i32::from(c.channel_volume >> 8),
                        arg,
                    );
                }
                0xef => {
                    c.pattern_start_ptr =
                        u16::from(next_byte(ram, origin, c)) << 8 | u16::from(arg);
                    c.subroutine_num_loops = next_byte(ram, origin, c);
                    c.saved_pattern_ptr = c.pattern_order_ptr_for_chan;
                    c.pattern_order_ptr_for_chan = c.pattern_start_ptr;
                }
                0xf0 => {
                    c.vibrato_fade_num_ticks = arg;
                    c.vibrato_fade_add_per_tick = if arg != 0 { c.vib_depth / arg } else { 0xff };
                }
                0xf1 | 0xf2 => {
                    c.pitch_envelope_direction = u8::from(effect == 0xf1);
                    c.pitch_envelope_delay = arg;
                    c.pitch_envelope_num_ticks = next_byte(ram, origin, c);
                    c.pitch_envelope_slide_value = next_byte(ram, origin, c);
                }
                0xf3 => c.pitch_envelope_num_ticks = 0,
                0xf4 => c.fine_tune = arg,
                0xf5 => {
                    p.reg_EON = arg;
                    p.echo_volume_left = u16::from(next_byte(ram, origin, c)) << 8;
                    p.echo_volume_right = u16::from(next_byte(ram, origin, c)) << 8;
                    p.reg_FLG &= !0x20;
                }
                0xf6 => {
                    p.echo_volume_left = 0;
                    p.echo_volume_right = 0;
                    p.reg_FLG |= 0x20;
                }
                0xf8 => {
                    p.echo_volume_fade_ticks = arg;
                    p.echo_volume_fade_target_left = next_byte(ram, origin, c);
                    p.echo_volume_fade_target_right = next_byte(ram, origin, c);
                    p.echo_volume_fade_add_left = divide(
                        i32::from(p.echo_volume_fade_target_left)
                            - i32::from(p.echo_volume_left >> 8),
                        arg,
                    );
                    p.echo_volume_fade_add_right = divide(
                        i32::from(p.echo_volume_fade_target_right)
                            - i32::from(p.echo_volume_right >> 8),
                        arg,
                    );
                }
                0xf9 => {
                    c.pitch_slide_delay_left = arg;
                    c.pitch_slide_length = next_byte(ram, origin, c);
                    let pitch = next_byte(ram, origin, c)
                        .wrapping_add(p.global_transposition)
                        .wrapping_add(c.channel_transposition);
                    compute_pitch_add(c, pitch);
                }
                0xfa => p.percussion_base_id = arg,
                0xfb => c.pattern_order_ptr_for_chan = c.pattern_order_ptr_for_chan.wrapping_add(1),
                0xfc => c.cutk = arg.wrapping_add(1),
                0xfd => {
                    p.fast_forward = arg.wrapping_add(1);
                    p.key_OFF |= !p.is_chan_on;
                }
                0xfe => {
                    p.fast_forward = arg;
                    p.key_OFF |= !p.is_chan_on;
                }
                _ => unreachable!(), // Invalid indices already fail in effect_length.
            }
        }
        fn final_volume(&mut self, channel: usize, volume: u8) {
            let c = &mut self.state.channel[channel];
            let mut value = i32::from(self.state.master_volume >> 8) * i32::from(volume) >> 8;
            value = value * i32::from(c.channel_volume_master) >> 8;
            value = value * i32::from(c.channel_volume >> 8) >> 8;
            c.final_volume = (value * value >> 8) as u8;
        }
        fn tremolo(&mut self, channel: usize, phase: u8) {
            let mut wave = phase.wrapping_mul(2) ^ if phase & 0x80 != 0 { 0xff } else { 0 };
            wave =
                (u16::from(wave) * u16::from(self.state.channel[channel].tremolo_depth) >> 8) as u8;
            self.final_volume(channel, wave ^ 0xff);
        }
        fn vibrato_pitch(&mut self, channel: usize, pitch: u16, phase: u8) {
            let mut value = i32::from(phase) << 2;
            value ^= if value & 0x100 != 0 { 0xff } else { 0 };
            let depth = self.state.channel[channel].vib_depth;
            let delta = if depth >= 0xf1 {
                i32::from(value as u8) * i32::from(depth & 0xf)
            } else {
                i32::from(value as u8) * i32::from(depth) >> 8
            };
            self.write_pitch(
                channel,
                (i32::from(pitch) + if phase & 0x80 != 0 { -delta } else { delta }) as u16,
            );
        }
        pub fn pan_and_sweep(&mut self, channel: usize) {
            self.state.did_affect_volumepitch_flag = 0;
            let c = &self.state.channel[channel];
            if c.tremolo_depth != 0 && c.tremolo_hold_count == c.tremolo_delay_ticks {
                self.state.did_affect_volumepitch_flag |= 0x80;
                let phase =
                    (u16::from(c.tremolo_rate) * u16::from(self.state.main_tempo_accum) >> 8) as u8;
                self.tremolo(channel, phase);
            }
            let c = &self.state.channel[channel];
            let mut volume = c.pan_value;
            if c.pan_num_ticks != 0 {
                self.state.did_affect_volumepitch_flag = 0x80;
                volume = volume.wrapping_add(
                    (i32::from(self.state.main_tempo_accum) * i32::from(c.pan_add_per_tick as i16)
                        / 256) as u16,
                );
            }
            if self.state.did_affect_volumepitch_flag != 0 {
                self.write_volume(channel, volume);
            }
            self.state.did_affect_volumepitch_flag = 0;
            let c = &self.state.channel[channel];
            let mut pitch = c.pitch;
            if c.pitch_slide_length != 0 && c.pitch_slide_delay_left == 0 {
                self.state.did_affect_volumepitch_flag |= 0x80;
                pitch = pitch.wrapping_add(
                    (i32::from(self.state.main_tempo_accum)
                        * i32::from(c.pitch_add_per_tick as i16)
                        / 256) as u16,
                );
            }
            if c.vib_depth != 0 && c.vibrato_delay_ticks == c.vibrato_hold_count {
                let phase = ((u16::from(self.state.main_tempo_accum) * u16::from(c.vibrato_rate)
                    >> 8) as u8)
                    .wrapping_add(c.vibrato_count);
                self.vibrato_pitch(channel, pitch, phase);
                return;
            }
            if self.state.did_affect_volumepitch_flag != 0 {
                self.write_pitch(channel, pitch);
            }
        }
        fn want_key_off(&self, channel: usize) -> bool {
            let c: &Channel = &self.state.channel[channel];
            let mut loops = i32::from(c.subroutine_num_loops);
            let mut address = i32::from(c.pattern_order_ptr_for_chan);
            loop {
                let previous = address;
                address += 1;
                let mut command = self.ram.read_at(previous as usize);
                if command == 0 {
                    if loops == 0 {
                        return true;
                    }
                    loops -= 1;
                    address = i32::from(if loops == 0 {
                        c.saved_pattern_ptr
                    } else {
                        c.pattern_start_ptr
                    });
                } else {
                    while command & 0x80 == 0 {
                        let previous = address;
                        address += 1;
                        command = self.ram.read_at(previous as usize);
                    }
                    if command == 0xc8 {
                        return false;
                    }
                    if command == 0xef {
                        address = i32::from(self.ram.read_at(address as usize))
                            | i32::from(self.ram.read_at((address + 1) as usize)) << 8;
                    } else if command >= 0xe0 {
                        address += i32::from(effect_length(command));
                    } else {
                        return true;
                    }
                }
            }
        }
        fn note_tick(&mut self, channel: usize) {
            let c = &mut self.state.channel[channel];
            if c.note_keyoff_ticks_left != 0 {
                c.note_keyoff_ticks_left = c.note_keyoff_ticks_left.wrapping_sub(1);
                let key_off = c.note_keyoff_ticks_left == 0 || c.note_ticks_left == 2;
                if key_off
                    && self.want_key_off(channel)
                    && self.state.cur_chan_bit & self.state.is_chan_on == 0
                {
                    (self.write)(92, self.state.cur_chan_bit);
                }
            }
            self.state.did_affect_volumepitch_flag = 0;
            let c = &mut self.state.channel[channel];
            if c.pitch_slide_length != 0 {
                if c.pitch_slide_delay_left != 0 {
                    c.pitch_slide_delay_left = c.pitch_slide_delay_left.wrapping_sub(1);
                } else {
                    self.state.did_affect_volumepitch_flag = 0x80;
                    c.pitch_slide_length = c.pitch_slide_length.wrapping_sub(1);
                    c.pitch = if c.pitch_slide_length != 0 {
                        c.pitch.wrapping_add(c.pitch_add_per_tick)
                    } else {
                        u16::from(c.pitch_target) << 8 | u16::from(c.fine_tune)
                    };
                }
            }
            let pitch = c.pitch;
            if c.vib_depth != 0 {
                if c.vibrato_delay_ticks == c.vibrato_hold_count {
                    if c.vibrato_change_count == c.vibrato_fade_num_ticks {
                        c.vib_depth = c.vibrato_depth_target;
                    } else {
                        let previous = c.vibrato_change_count;
                        c.vibrato_change_count = previous.wrapping_add(1);
                        c.vib_depth = (if previous == 0 { 0u8 } else { c.vib_depth })
                            .wrapping_add(c.vibrato_fade_add_per_tick);
                    }
                    c.vibrato_count = c.vibrato_count.wrapping_add(c.vibrato_rate);
                    let phase = c.vibrato_count;
                    self.vibrato_pitch(channel, pitch, phase);
                    return;
                }
                c.vibrato_hold_count = c.vibrato_hold_count.wrapping_add(1);
            }
            if self.state.did_affect_volumepitch_flag != 0 {
                self.write_pitch(channel, pitch);
            }
        }
        fn channel_tick(&mut self, channel: usize) {
            let c = &mut self.state.channel[channel];
            if c.volume_fade_ticks != 0 {
                c.volume_fade_ticks = c.volume_fade_ticks.wrapping_sub(1);
                c.channel_volume = fade(
                    c.channel_volume,
                    c.volume_fade_addpertick,
                    c.volume_fade_target,
                    c.volume_fade_ticks,
                );
                self.state.vol_dirty |= self.state.cur_chan_bit;
            }
            if c.tremolo_depth != 0 {
                if c.tremolo_delay_ticks == c.tremolo_hold_count {
                    self.state.vol_dirty |= self.state.cur_chan_bit;
                    c.tremolo_count = if c.tremolo_count & 0x80 != 0 && c.tremolo_depth == 0xff {
                        0x80
                    } else {
                        c.tremolo_count.wrapping_add(c.tremolo_rate)
                    };
                    let phase = c.tremolo_count;
                    self.tremolo(channel, phase);
                } else {
                    c.tremolo_hold_count = c.tremolo_hold_count.wrapping_add(1);
                    self.final_volume(channel, 0xff);
                }
            } else {
                self.final_volume(channel, 0xff);
            }
            let c = &mut self.state.channel[channel];
            if c.pan_num_ticks != 0 {
                c.pan_num_ticks = c.pan_num_ticks.wrapping_sub(1);
                c.pan_value = fade(
                    c.pan_value,
                    c.pan_add_per_tick,
                    c.pan_target_value,
                    c.pan_num_ticks,
                );
                self.state.vol_dirty |= self.state.cur_chan_bit;
            }
            let pan = c.pan_value;
            if self.state.vol_dirty & self.state.cur_chan_bit != 0 {
                self.write_volume(channel, pan);
            }
        }
        fn play_note(&mut self, channel: usize, mut note: u8) {
            if note >= 0xca {
                self.set_instrument(channel, note);
                note = 0xa4;
            }
            if note >= 0xc8 || self.state.is_chan_on & self.state.cur_chan_bit != 0 {
                return;
            }
            let c = &mut self.state.channel[channel];
            c.pitch = ((u16::from(note & 0x7f)
                + u16::from(self.state.global_transposition)
                + u16::from(c.channel_transposition))
                << 8)
                | u16::from(c.fine_tune);
            c.vibrato_count = c.vibrato_fade_num_ticks.wrapping_shl(7);
            c.vibrato_hold_count = 0;
            c.vibrato_change_count = 0;
            c.tremolo_count = 0;
            c.tremolo_hold_count = 0;
            self.state.vol_dirty |= self.state.cur_chan_bit;
            self.state.key_ON |= self.state.cur_chan_bit;
            c.pitch_slide_length = c.pitch_envelope_num_ticks;
            if c.pitch_slide_length != 0 {
                c.pitch_slide_delay_left = c.pitch_envelope_delay;
                if c.pitch_envelope_direction == 0 {
                    c.pitch = c
                        .pitch
                        .wrapping_sub(u16::from(c.pitch_envelope_slide_value) << 8);
                }
                let pitch = ((c.pitch >> 8) as u8).wrapping_add(c.pitch_envelope_slide_value);
                compute_pitch_add(&mut self.state.channel[channel], pitch);
            }
            self.write_pitch(channel, self.state.channel[channel].pitch);
        }
        fn new_song(&mut self, song: u8) {
            self.state.port_to_snes[0] = song;
            self.state.music_ptr_toplevel =
                self.word((0x5820 + (i32::from(song) - 1) * 2) as usize);
            self.state.counter_sf0c = 2;
            self.state.key_OFF |= self.state.is_chan_on ^ 0xff;
        }
        fn next_phrase(&mut self) -> bool {
            let mut table;
            loop {
                table = self.word(usize::from(self.state.music_ptr_toplevel));
                self.state.music_ptr_toplevel = self.state.music_ptr_toplevel.wrapping_add(2);
                if table >> 8 != 0 {
                    break;
                }
                if table == 0 {
                    self.new_song(0);
                    return false;
                }
                if table == 0x80 {
                    self.state.fast_forward = 0x80;
                } else if table == 0x81 {
                    self.state.fast_forward = 0;
                } else {
                    self.state.block_count = self.state.block_count.wrapping_sub(1);
                    if (self.state.block_count as i8) < 0 {
                        self.state.block_count = table as u8;
                    }
                    let target = self.word(usize::from(self.state.music_ptr_toplevel));
                    self.state.music_ptr_toplevel = self.state.music_ptr_toplevel.wrapping_add(2);
                    if self.state.block_count != 0 {
                        self.state.music_ptr_toplevel = target;
                    }
                }
            }
            let mut address = usize::from(table);
            for channel in 0..8 {
                self.state.channel[channel].pattern_order_ptr_for_chan = self.word(address);
                address += 2;
            }
            self.state.cur_chan_bit = 1;
            for channel in 0..8 {
                if self.state.channel[channel]
                    .pattern_order_ptr_for_chan
                    .to_ne_bytes()[1]
                    != 0
                    && self.state.channel[channel].instrument_id == 0
                {
                    self.set_instrument(channel, 0);
                }
                let c = &mut self.state.channel[channel];
                c.subroutine_num_loops = 0;
                c.volume_fade_ticks = 0;
                c.pan_num_ticks = 0;
                c.note_ticks_left = 1;
                self.state.cur_chan_bit = self.state.cur_chan_bit.wrapping_shl(1);
            }
            true
        }
        pub fn tick(&mut self) {
            let command = self.state.input_ports[0];
            self.state.input_ports[0] = 255;
            if command == 0xf0 {
                self.state.key_OFF |= !self.state.is_chan_on;
                return;
            }
            if command != 0xf1 && command != 255 && command != self.state.port_to_snes[0] {
                self.new_song(command);
                return;
            }
            if self.state.port_to_snes[0] == 0 {
                return;
            }
            let mut phrase = false;
            if self.state.counter_sf0c != 0 {
                self.state.counter_sf0c = self.state.counter_sf0c.wrapping_sub(1);
                if self.state.counter_sf0c != 0 {
                    self.reset_channels();
                    return;
                }
                phrase = true;
            }
            'phrase: loop {
                if phrase && !self.next_phrase() {
                    return;
                }
                self.state.vol_dirty = 0;
                self.state.cur_chan_bit = 1;
                for channel in 0..8 {
                    if self.state.channel[channel]
                        .pattern_order_ptr_for_chan
                        .to_ne_bytes()[1]
                        != 0
                    {
                        self.state.channel[channel].note_ticks_left =
                            self.state.channel[channel].note_ticks_left.wrapping_sub(1);
                        if self.state.channel[channel].note_ticks_left == 0 {
                            loop {
                                let mut note = self.byte(channel);
                                if note == 0 {
                                    if self.state.channel[channel].subroutine_num_loops == 0 {
                                        phrase = true;
                                        continue 'phrase;
                                    }
                                    let c = &mut self.state.channel[channel];
                                    c.subroutine_num_loops = c.subroutine_num_loops.wrapping_sub(1);
                                    c.pattern_order_ptr_for_chan = if c.subroutine_num_loops == 0 {
                                        c.saved_pattern_ptr
                                    } else {
                                        c.pattern_start_ptr
                                    };
                                    continue;
                                }
                                if note & 0x80 == 0 {
                                    const VOLUME: [u8; 16] = [
                                        25, 50, 76, 101, 114, 127, 140, 152, 165, 178, 191, 203,
                                        216, 229, 242, 252,
                                    ];
                                    const GATE: [u8; 8] = [50, 101, 127, 152, 178, 203, 229, 252];
                                    self.state.channel[channel].note_length = note;
                                    note = self.byte(channel);
                                    if note & 0x80 == 0 {
                                        self.state.channel[channel].note_gate_off_fixedpt =
                                            GATE[usize::from(note >> 4 & 7)];
                                        self.state.channel[channel].channel_volume_master =
                                            VOLUME[usize::from(note & 0xf)];
                                        note = self.byte(channel);
                                    }
                                }
                                if note >= 0xe0 {
                                    self.effect(channel, note);
                                    continue;
                                }
                                if self.state.channel[channel].cutk | self.state.fast_forward == 0 {
                                    self.play_note(channel, note);
                                }
                                let c = &mut self.state.channel[channel];
                                c.note_ticks_left = c.note_length;
                                let gate = (u16::from(c.note_ticks_left)
                                    * u16::from(c.note_gate_off_fixedpt)
                                    >> 8) as u8;
                                c.note_keyoff_ticks_left = if gate != 0 { gate } else { 1 };
                                self.pitch_slide(channel);
                                break;
                            }
                        } else if self.state.fast_forward == 0 {
                            self.note_tick(channel);
                            self.pitch_slide(channel);
                        }
                    }
                    self.state.cur_chan_bit = self.state.cur_chan_bit.wrapping_shl(1);
                }
                if self.state.tempo_fade_num_ticks != 0 {
                    self.state.tempo_fade_num_ticks =
                        self.state.tempo_fade_num_ticks.wrapping_sub(1);
                    self.state.tempo = fade(
                        self.state.tempo,
                        self.state.tempo_fade_add,
                        self.state.tempo_fade_final,
                        self.state.tempo_fade_num_ticks,
                    );
                }
                if self.state.echo_volume_fade_ticks != 0 {
                    self.state.echo_volume_left = self
                        .state
                        .echo_volume_left
                        .wrapping_add(self.state.echo_volume_fade_add_left);
                    self.state.echo_volume_right = self
                        .state
                        .echo_volume_right
                        .wrapping_add(self.state.echo_volume_fade_add_right);
                    self.state.echo_volume_fade_ticks =
                        self.state.echo_volume_fade_ticks.wrapping_sub(1);
                    if self.state.echo_volume_fade_ticks == 0 {
                        self.state.echo_volume_left =
                            u16::from(self.state.echo_volume_fade_target_left) << 8;
                        self.state.echo_volume_right =
                            u16::from(self.state.echo_volume_fade_target_right) << 8;
                    }
                }
                if self.state.master_volume_fade_ticks != 0 {
                    self.state.master_volume_fade_ticks =
                        self.state.master_volume_fade_ticks.wrapping_sub(1);
                    self.state.master_volume = fade(
                        self.state.master_volume,
                        self.state.master_volume_fade_add_per_tick,
                        self.state.master_volume_fade_target,
                        self.state.master_volume_fade_ticks,
                    );
                    self.state.vol_dirty = 0xff;
                }
                self.state.cur_chan_bit = 1;
                for channel in 0..8 {
                    if self.state.channel[channel]
                        .pattern_order_ptr_for_chan
                        .to_ne_bytes()[1]
                        != 0
                    {
                        self.channel_tick(channel);
                    }
                    self.state.cur_chan_bit = self.state.cur_chan_bit.wrapping_shl(1);
                }
                return;
            }
        }
    }
}
