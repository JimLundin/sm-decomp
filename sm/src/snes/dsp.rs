use crate::c_library::{free, malloc, memset};
use crate::game_state::Game;
use crate::set_fields;
pub use crate::types::{size_t, Dsp, DspChannel, SaveLoadFunc};
use ::core::ffi::{c_double, c_void};
use ::core::mem::size_of;
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
static rateValues: [i32; 32] = [
    0, 2048, 1536, 1280, 1024, 768, 640, 512, 384, 320, 256, 192, 160, 128, 96, 80, 64, 48, 40, 32,
    24, 20, 16, 12, 10, 8, 6, 5, 4, 3, 2, 1,
];
static gaussValues: [u16; 512] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1, 0x1, 0x1, 0x1, 0x1, 0x1, 0x1, 0x1, 0x1,
    0x1, 0x1, 0x2, 0x2, 0x2, 0x2, 0x2, 0x2, 0x2, 0x3, 0x3, 0x3, 0x3, 0x3, 0x4, 0x4, 0x4, 0x4, 0x4,
    0x5, 0x5, 0x5, 0x5, 0x6, 0x6, 0x6, 0x6, 0x7, 0x7, 0x7, 0x8, 0x8, 0x8, 0x9, 0x9, 0x9, 0xa, 0xa,
    0xa, 0xb, 0xb, 0xb, 0xc, 0xc, 0xd, 0xd, 0xe, 0xe, 0xf, 0xf, 0xf, 0x10, 0x10, 0x11, 0x11, 0x12,
    0x13, 0x13, 0x14, 0x14, 0x15, 0x15, 0x16, 0x17, 0x17, 0x18, 0x18, 0x19, 0x1a, 0x1b, 0x1b, 0x1c,
    0x1d, 0x1d, 0x1e, 0x1f, 0x20, 0x20, 0x21, 0x22, 0x23, 0x24, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
    0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x3a,
    0x3b, 0x3c, 0x3d, 0x3e, 0x40, 0x41, 0x42, 0x43, 0x45, 0x46, 0x47, 0x49, 0x4a, 0x4c, 0x4d, 0x4e,
    0x50, 0x51, 0x53, 0x54, 0x56, 0x57, 0x59, 0x5a, 0x5c, 0x5e, 0x5f, 0x61, 0x63, 0x64, 0x66, 0x68,
    0x6a, 0x6b, 0x6d, 0x6f, 0x71, 0x73, 0x75, 0x76, 0x78, 0x7a, 0x7c, 0x7e, 0x80, 0x82, 0x84, 0x86,
    0x89, 0x8b, 0x8d, 0x8f, 0x91, 0x93, 0x96, 0x98, 0x9a, 0x9c, 0x9f, 0xa1, 0xa3, 0xa6, 0xa8, 0xab,
    0xad, 0xaf, 0xb2, 0xb4, 0xb7, 0xba, 0xbc, 0xbf, 0xc1, 0xc4, 0xc7, 0xc9, 0xcc, 0xcf, 0xd2, 0xd4,
    0xd7, 0xda, 0xdd, 0xe0, 0xe3, 0xe6, 0xe9, 0xec, 0xef, 0xf2, 0xf5, 0xf8, 0xfb, 0xfe, 0x101,
    0x104, 0x107, 0x10b, 0x10e, 0x111, 0x114, 0x118, 0x11b, 0x11e, 0x122, 0x125, 0x129, 0x12c,
    0x130, 0x133, 0x137, 0x13a, 0x13e, 0x141, 0x145, 0x148, 0x14c, 0x150, 0x153, 0x157, 0x15b,
    0x15f, 0x162, 0x166, 0x16a, 0x16e, 0x172, 0x176, 0x17a, 0x17d, 0x181, 0x185, 0x189, 0x18d,
    0x191, 0x195, 0x19a, 0x19e, 0x1a2, 0x1a6, 0x1aa, 0x1ae, 0x1b2, 0x1b7, 0x1bb, 0x1bf, 0x1c3,
    0x1c8, 0x1cc, 0x1d0, 0x1d5, 0x1d9, 0x1dd, 0x1e2, 0x1e6, 0x1eb, 0x1ef, 0x1f3, 0x1f8, 0x1fc,
    0x201, 0x205, 0x20a, 0x20f, 0x213, 0x218, 0x21c, 0x221, 0x226, 0x22a, 0x22f, 0x233, 0x238,
    0x23d, 0x241, 0x246, 0x24b, 0x250, 0x254, 0x259, 0x25e, 0x263, 0x267, 0x26c, 0x271, 0x276,
    0x27b, 0x280, 0x284, 0x289, 0x28e, 0x293, 0x298, 0x29d, 0x2a2, 0x2a6, 0x2ab, 0x2b0, 0x2b5,
    0x2ba, 0x2bf, 0x2c4, 0x2c9, 0x2ce, 0x2d3, 0x2d8, 0x2dc, 0x2e1, 0x2e6, 0x2eb, 0x2f0, 0x2f5,
    0x2fa, 0x2ff, 0x304, 0x309, 0x30e, 0x313, 0x318, 0x31d, 0x322, 0x326, 0x32b, 0x330, 0x335,
    0x33a, 0x33f, 0x344, 0x349, 0x34e, 0x353, 0x357, 0x35c, 0x361, 0x366, 0x36b, 0x370, 0x374,
    0x379, 0x37e, 0x383, 0x388, 0x38c, 0x391, 0x396, 0x39b, 0x39f, 0x3a4, 0x3a9, 0x3ad, 0x3b2,
    0x3b7, 0x3bb, 0x3c0, 0x3c5, 0x3c9, 0x3ce, 0x3d2, 0x3d7, 0x3dc, 0x3e0, 0x3e5, 0x3e9, 0x3ed,
    0x3f2, 0x3f6, 0x3fb, 0x3ff, 0x403, 0x408, 0x40c, 0x410, 0x415, 0x419, 0x41d, 0x421, 0x425,
    0x42a, 0x42e, 0x432, 0x436, 0x43a, 0x43e, 0x442, 0x446, 0x44a, 0x44e, 0x452, 0x455, 0x459,
    0x45d, 0x461, 0x465, 0x468, 0x46c, 0x470, 0x473, 0x477, 0x47a, 0x47e, 0x481, 0x485, 0x488,
    0x48c, 0x48f, 0x492, 0x496, 0x499, 0x49c, 0x49f, 0x4a2, 0x4a6, 0x4a9, 0x4ac, 0x4af, 0x4b2,
    0x4b5, 0x4b7, 0x4ba, 0x4bd, 0x4c0, 0x4c3, 0x4c5, 0x4c8, 0x4cb, 0x4cd, 0x4d0, 0x4d2, 0x4d5,
    0x4d7, 0x4d9, 0x4dc, 0x4de, 0x4e0, 0x4e3, 0x4e5, 0x4e7, 0x4e9, 0x4eb, 0x4ed, 0x4ef, 0x4f1,
    0x4f3, 0x4f5, 0x4f6, 0x4f8, 0x4fa, 0x4fb, 0x4fd, 0x4ff, 0x500, 0x502, 0x503, 0x504, 0x506,
    0x507, 0x508, 0x50a, 0x50b, 0x50c, 0x50d, 0x50e, 0x50f, 0x510, 0x511, 0x511, 0x512, 0x513,
    0x514, 0x514, 0x515, 0x516, 0x516, 0x517, 0x517, 0x517, 0x518, 0x518, 0x518, 0x518, 0x518,
    0x519, 0x519,
];
// The DSP clamps signed samples at each original mixing/decoding boundary.
fn saturate_sample(value: i32) -> i32 {
    value.clamp(i16::MIN as i32, i16::MAX as i32)
}

pub unsafe fn dsp_init(ram: *mut u8) -> *mut Dsp {
    let dsp = malloc(size_of::<Dsp>() as size_t) as *mut Dsp;
    (*dsp).apu_ram = ram;
    dsp
}
pub unsafe fn dsp_reset(dsp: *mut Dsp) {
    memset(
        &raw mut (*dsp).ram as *mut u8 as *mut c_void,
        0,
        size_of::<[u8; 128]>() as size_t,
    );
    (*dsp).ram[0x7ci32 as usize] = 0xff;
    let mut i: i32 = 0;
    while i < 8 {
        set_fields! {[(*dsp).channel[i as usize]], 0; pitch, pitchCounter}
        (*dsp).channel[i as usize].pitchModulation = false_0 != 0;
        memset(
            &raw mut (*(&raw mut (*dsp).channel as *mut DspChannel).offset(i as isize)).decodeBuffer
                as *mut i16 as *mut c_void,
            0,
            size_of::<[i16; 19]>() as size_t,
        );
        (*dsp).channel[i as usize].srcn = 0;
        (*dsp).channel[i as usize].decodeOffset = 0;
        (*dsp).channel[i as usize].previousFlags = 0;
        set_fields! {[(*dsp).channel[i as usize]], 0; old, older}
        (*dsp).channel[i as usize].useNoise = false_0 != 0;
        memset(
            &raw mut (*(&raw mut (*dsp).channel as *mut DspChannel).offset(i as isize)).adsrRates
                as *mut u16 as *mut c_void,
            0,
            size_of::<[u16; 4]>() as size_t,
        );
        (*dsp).channel[i as usize].rateCounter = 0;
        (*dsp).channel[i as usize].adsrState = 0;
        (*dsp).channel[i as usize].sustainLevel = 0;
        (*dsp).channel[i as usize].useGain = false_0 != 0;
        (*dsp).channel[i as usize].gainMode = 0;
        (*dsp).channel[i as usize].directGain = false_0 != 0;
        set_fields! {[(*dsp).channel[i as usize]], 0; gainValue, gain}
        (*dsp).channel[i as usize].keyOn = false_0 != 0;
        (*dsp).channel[i as usize].keyOff = false_0 != 0;
        (*dsp).channel[i as usize].sampleOut = 0;
        set_fields! {[(*dsp).channel[i as usize]], 0; volumeL, volumeR}
        (*dsp).channel[i as usize].echoEnable = false_0 != 0;
        i += 1;
    }
    (*dsp).dirPage = 0;
    (*dsp).evenCycle = false_0 != 0;
    (*dsp).mute = true_0 != 0;
    (*dsp).reset = true_0 != 0;
    set_fields! {[(*dsp)], 0; masterVolumeL, masterVolumeR}
    (*dsp).noiseSample = -0x4000i16;
    set_fields! {[(*dsp)], 0; noiseRate, noiseCounter}
    (*dsp).echoWrites = false_0 != 0;
    set_fields! {[(*dsp)], 0; echoVolumeL, echoVolumeR, feedbackVolume}
    (*dsp).echoBufferAdr = 0;
    set_fields! {[(*dsp)], 1; echoDelay, echoRemain}
    (*dsp).echoBufferIndex = 0;
    (*dsp).firBufferIndex = 0;
    memset(
        &raw mut (*dsp).firValues as *mut i8 as *mut c_void,
        0,
        size_of::<[i8; 8]>() as size_t,
    );
    memset(
        &raw mut (*dsp).firBufferL as *mut i16 as *mut c_void,
        0,
        size_of::<[i16; 8]>() as size_t,
    );
    memset(
        &raw mut (*dsp).firBufferR as *mut i16 as *mut c_void,
        0,
        size_of::<[i16; 8]>() as size_t,
    );
    memset(
        &raw mut (*dsp).sampleBuffer as *mut i16 as *mut c_void,
        0,
        size_of::<[i16; 1068]>() as size_t,
    );
    (*dsp).sampleOffset = 0;
}
pub unsafe fn dsp_saveload(
    dsp: *mut Dsp,
    func: Option<SaveLoadFunc>,
    ctx: *mut c_void,
    game: &mut Game,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*dsp).ram as *mut c_void,
        (size_of::<Dsp>() as size_t).wrapping_sub(8 as size_t),
        game,
    );
}
pub unsafe fn dsp_cycle(dsp: *mut Dsp) {
    let mut totalL: i32 = 0;
    let mut totalR: i32 = 0;
    let mut i: i32 = 0;
    while i < 8 {
        dsp_cycleChannel(dsp, i);
        totalL += (*dsp).channel[i as usize].sampleOut as i32
            * (*dsp).channel[i as usize].volumeL as i32
            >> 6;
        totalR += (*dsp).channel[i as usize].sampleOut as i32
            * (*dsp).channel[i as usize].volumeR as i32
            >> 6;
        totalL = saturate_sample(totalL);
        totalR = saturate_sample(totalR);
        i += 1;
    }
    totalL = totalL * (*dsp).masterVolumeL as i32 >> 7;
    totalR = totalR * (*dsp).masterVolumeR as i32 >> 7;
    totalL = saturate_sample(totalL);
    totalR = saturate_sample(totalR);
    dsp_handleEcho(dsp, &raw mut totalL, &raw mut totalR);
    if (*dsp).mute {
        totalL = 0;
        totalR = 0;
    }
    dsp_handleNoise(dsp);
    if ((*dsp).sampleOffset as i32) < 534 {
        (*dsp).sampleBuffer[((*dsp).sampleOffset as i32 * 2) as usize] = totalL as i16;
        (*dsp).sampleBuffer[((*dsp).sampleOffset as i32 * 2 + 1) as usize] = totalR as i16;
        (*dsp).sampleOffset = (*dsp).sampleOffset.wrapping_add(1);
    }
    (*dsp).evenCycle = !(*dsp).evenCycle;
}
unsafe fn dsp_handleEcho(dsp: *mut Dsp, outputL: *mut i32, outputR: *mut i32) {
    let adr: u16 = ((*dsp).echoBufferAdr as i32 + (*dsp).echoBufferIndex as i32 * 4) as u16;
    (*dsp).firBufferL[(*dsp).firBufferIndex as usize] = (*(*dsp).apu_ram.offset(adr as isize)
        as i32
        + ((*(*dsp).apu_ram.offset((adr as i32 + 1 & 0xffff) as isize) as i32) << 8))
        as i16;
    (*dsp).firBufferL[(*dsp).firBufferIndex as usize] =
        ((*dsp).firBufferL[(*dsp).firBufferIndex as usize] >> 1);
    (*dsp).firBufferR[(*dsp).firBufferIndex as usize] =
        (*(*dsp).apu_ram.offset((adr as i32 + 2 & 0xffff) as isize) as i32
            + ((*(*dsp).apu_ram.offset((adr as i32 + 3 & 0xffff) as isize) as i32) << 8))
            as i16;
    (*dsp).firBufferR[(*dsp).firBufferIndex as usize] =
        ((*dsp).firBufferR[(*dsp).firBufferIndex as usize] >> 1);
    let mut sumL: i32 = 0;
    let mut sumR: i32 = 0;
    let mut i: i32 = 0;
    while i < 8 {
        sumL += (*dsp).firBufferL[((*dsp).firBufferIndex as i32 + i + 1 & 0x7) as usize] as i32
            * (*dsp).firValues[i as usize] as i32
            >> 6;
        sumR += (*dsp).firBufferR[((*dsp).firBufferIndex as i32 + i + 1 & 0x7) as usize] as i32
            * (*dsp).firValues[i as usize] as i32
            >> 6;
        if i == 6 {
            sumL = (sumL & 0xffff) as i16 as i32;
            sumR = (sumR & 0xffff) as i16 as i32;
        }
        i += 1;
    }
    sumL = saturate_sample(sumL);
    sumR = saturate_sample(sumR);
    let outL: i32 = *outputL + (sumL * (*dsp).echoVolumeL as i32 >> 7);
    let outR: i32 = *outputR + (sumR * (*dsp).echoVolumeR as i32 >> 7);
    *outputL = saturate_sample(outL);
    *outputR = saturate_sample(outR);
    let mut inL: i32 = 0;
    let mut inR: i32 = 0;
    let mut i_0: i32 = 0;
    while i_0 < 8 {
        if (*dsp).channel[i_0 as usize].echoEnable {
            inL += (*dsp).channel[i_0 as usize].sampleOut as i32
                * (*dsp).channel[i_0 as usize].volumeL as i32
                >> 6;
            inR += (*dsp).channel[i_0 as usize].sampleOut as i32
                * (*dsp).channel[i_0 as usize].volumeR as i32
                >> 6;
            inL = saturate_sample(inL);
            inR = saturate_sample(inR);
        }
        i_0 += 1;
    }
    inL += sumL * (*dsp).feedbackVolume as i32 >> 7;
    inR += sumR * (*dsp).feedbackVolume as i32 >> 7;
    inL = saturate_sample(inL);
    inR = saturate_sample(inR);
    inL &= 0xfffe;
    inR &= 0xfffe;
    if (*dsp).echoWrites {
        *(*dsp).apu_ram.offset(adr as isize) = (inL & 0xff) as u8;
        *(*dsp).apu_ram.offset((adr as i32 + 1 & 0xffff) as isize) = (inL >> 8) as u8;
        *(*dsp).apu_ram.offset((adr as i32 + 2 & 0xffff) as isize) = (inR & 0xff) as u8;
        *(*dsp).apu_ram.offset((adr as i32 + 3 & 0xffff) as isize) = (inR >> 8) as u8;
    }
    (*dsp).firBufferIndex = (*dsp).firBufferIndex.wrapping_add(1);
    (*dsp).firBufferIndex = ((*dsp).firBufferIndex & 7);
    (*dsp).echoBufferIndex = (*dsp).echoBufferIndex.wrapping_add(1);
    (*dsp).echoRemain = (*dsp).echoRemain.wrapping_sub(1);
    if (*dsp).echoRemain as i32 == 0 {
        (*dsp).echoRemain = (*dsp).echoDelay;
        (*dsp).echoBufferIndex = 0;
    }
}
unsafe fn dsp_cycleChannel(dsp: *mut Dsp, ch: i32) {
    let mut pitch: u16 = (*dsp).channel[ch as usize].pitch;
    if ch > 0 && (*dsp).channel[ch as usize].pitchModulation as i32 != 0 {
        let factor: i32 = ((*dsp).channel[(ch - 1i32) as usize].sampleOut as i32 >> 4) + 0x400;
        pitch = (pitch as i32 * factor >> 10) as u16;
        if pitch as i32 > 0x3fff {
            pitch = 0x3fff;
        }
    }
    let newCounter: i32 = (*dsp).channel[ch as usize].pitchCounter as i32 + pitch as i32;
    if newCounter > 0xffff {
        dsp_decodeBrr(dsp, ch);
    }
    (*dsp).channel[ch as usize].pitchCounter = newCounter as u16;
    let mut sample: i16 = 0;
    if (*dsp).channel[ch as usize].useNoise {
        sample = (*dsp).noiseSample;
    } else {
        sample = dsp_getSample(
            dsp,
            ch,
            (*dsp).channel[ch as usize].pitchCounter as i32 >> 12,
            (*dsp).channel[ch as usize].pitchCounter as i32 >> 4 & 0xff,
        );
    }
    if (*dsp).reset {
        (*dsp).channel[ch as usize].adsrState = 4;
        (*dsp).channel[ch as usize].gain = 0;
    }
    let doingDirectGain: bool = (*dsp).channel[ch as usize].adsrState as i32 != 4
        && (*dsp).channel[ch as usize].useGain as i32 != 0
        && (*dsp).channel[ch as usize].directGain as i32 != 0;
    let rate: u16 = (if (*dsp).channel[ch as usize].adsrState as i32 == 4 {
        0
    } else {
        (*dsp).channel[ch as usize].adsrRates[(*dsp).channel[ch as usize].adsrState as usize] as i32
    }) as u16;
    if (*dsp).channel[ch as usize].adsrState as i32 != 4 && !doingDirectGain && rate as i32 != 0 {
        (*dsp).channel[ch as usize].rateCounter =
            (*dsp).channel[ch as usize].rateCounter.wrapping_add(1);
    }
    if (*dsp).channel[ch as usize].adsrState as i32 == 4
        || !doingDirectGain
            && (*dsp).channel[ch as usize].rateCounter as i32 >= rate as i32
            && rate as i32 != 0
    {
        if (*dsp).channel[ch as usize].adsrState as i32 != 4 {
            (*dsp).channel[ch as usize].rateCounter = 0;
        }
        dsp_handleGain(dsp, ch);
    }
    if doingDirectGain {
        (*dsp).channel[ch as usize].gain = (*dsp).channel[ch as usize].gainValue;
    }
    (*dsp).ram[(ch << 4 | 8) as usize] = ((*dsp).channel[ch as usize].gain >> 4) as u8;
    sample = (sample as i32 * (*dsp).channel[ch as usize].gain as i32 >> 11) as i16;
    (*dsp).ram[(ch << 4 | 9) as usize] = (sample >> 7) as u8;
    (*dsp).channel[ch as usize].sampleOut = sample;
}
unsafe fn dsp_handleGain(dsp: *mut Dsp, ch: i32) {
    match (*dsp).channel[ch as usize].adsrState as i32 {
        0 => {
            let rate: u16 = (*dsp).channel[ch as usize].adsrRates
                [(*dsp).channel[ch as usize].adsrState as usize];
            (*dsp).channel[ch as usize].gain = ((*dsp).channel[ch as usize].gain as i32
                + if rate as i32 == 1 { 1024 } else { 32 })
                as u16;
            if (*dsp).channel[ch as usize].gain as i32 >= 0x7e0 {
                (*dsp).channel[ch as usize].adsrState = 1;
            }
            if (*dsp).channel[ch as usize].gain as i32 > 0x7ff {
                (*dsp).channel[ch as usize].gain = 0x7ff;
            }
        }
        1 => {
            (*dsp).channel[ch as usize].gain = ((*dsp).channel[ch as usize].gain as i32
                - (((*dsp).channel[ch as usize].gain as i32 - 1i32 >> 8) + 1))
                as u16;
            if ((*dsp).channel[ch as usize].gain as i32)
                < (*dsp).channel[ch as usize].sustainLevel as i32
            {
                (*dsp).channel[ch as usize].adsrState = 2;
            }
        }
        2 => {
            (*dsp).channel[ch as usize].gain = ((*dsp).channel[ch as usize].gain as i32
                - (((*dsp).channel[ch as usize].gain as i32 - 1i32 >> 8) + 1))
                as u16
        }
        3 => match (*dsp).channel[ch as usize].gainMode as i32 {
            0 => {
                (*dsp).channel[ch as usize].gain =
                    ((*dsp).channel[ch as usize].gain as i32 - 32i32) as u16;
                if (*dsp).channel[ch as usize].gain as i32 > 0x7ff {
                    (*dsp).channel[ch as usize].gain = 0;
                }
            }
            1 => {
                (*dsp).channel[ch as usize].gain = ((*dsp).channel[ch as usize].gain as i32
                    - (((*dsp).channel[ch as usize].gain as i32 - 1i32 >> 8) + 1))
                    as u16
            }
            2 => {
                (*dsp).channel[ch as usize].gain =
                    ((*dsp).channel[ch as usize].gain as i32 + 32) as u16;
                if (*dsp).channel[ch as usize].gain as i32 > 0x7ff {
                    (*dsp).channel[ch as usize].gain = 0x7ff;
                }
            }
            3 => {
                (*dsp).channel[ch as usize].gain = ((*dsp).channel[ch as usize].gain as i32
                    + if ((*dsp).channel[ch as usize].gain as i32) < 0x600 {
                        32
                    } else {
                        8
                    }) as u16;
                if (*dsp).channel[ch as usize].gain as i32 > 0x7ff {
                    (*dsp).channel[ch as usize].gain = 0x7ff;
                }
            }
            _ => {}
        },
        4 => {
            (*dsp).channel[ch as usize].gain =
                ((*dsp).channel[ch as usize].gain as i32 - 8i32) as u16;
            if (*dsp).channel[ch as usize].gain as i32 > 0x7ff {
                (*dsp).channel[ch as usize].gain = 0;
            }
        }
        _ => {}
    };
}
unsafe fn dsp_getSample(dsp: *mut Dsp, ch: i32, sampleNum: i32, offset: i32) -> i16 {
    let news: i16 = (*dsp).channel[ch as usize].decodeBuffer[(sampleNum + 3) as usize];
    let olds: i16 = (*dsp).channel[ch as usize].decodeBuffer[(sampleNum + 2) as usize];
    let olders: i16 = (*dsp).channel[ch as usize].decodeBuffer[(sampleNum + 1) as usize];
    let oldests: i16 = (*dsp).channel[ch as usize].decodeBuffer[sampleNum as usize];
    let mut out: i32 = gaussValues[(0xff - offset) as usize] as i32 * oldests as i32 >> 10;
    out += gaussValues[(0x1ff - offset) as usize] as i32 * olders as i32 >> 10;
    out += gaussValues[(0x100 + offset) as usize] as i32 * olds as i32 >> 10;
    out = (out & 0xffff) as i16 as i32;
    out += gaussValues[offset as usize] as i32 * news as i32 >> 10;
    out = saturate_sample(out);
    (out >> 1) as i16
}
unsafe fn dsp_decodeBrr(dsp: *mut Dsp, ch: i32) {
    (*dsp).channel[ch as usize].decodeBuffer[0i32 as usize] =
        (*dsp).channel[ch as usize].decodeBuffer[16i32 as usize];
    (*dsp).channel[ch as usize].decodeBuffer[1i32 as usize] =
        (*dsp).channel[ch as usize].decodeBuffer[17i32 as usize];
    (*dsp).channel[ch as usize].decodeBuffer[2i32 as usize] =
        (*dsp).channel[ch as usize].decodeBuffer[18i32 as usize];
    if (*dsp).channel[ch as usize].previousFlags as i32 == 1
        || (*dsp).channel[ch as usize].previousFlags as i32 == 3
    {
        let samplePointer: u16 =
            ((*dsp).dirPage as i32 + 4 * (*dsp).channel[ch as usize].srcn as i32) as u16;
        (*dsp).channel[ch as usize].decodeOffset = *(*dsp)
            .apu_ram
            .offset((samplePointer as i32 + 2 & 0xffff) as isize)
            as u16;
        (*dsp).channel[ch as usize].decodeOffset = ((*dsp).channel[ch as usize].decodeOffset as i32
            | (*(*dsp)
                .apu_ram
                .offset((samplePointer as i32 + 3 & 0xffff) as isize) as i32)
                << 8) as u16;
        if (*dsp).channel[ch as usize].previousFlags as i32 == 1 {
            (*dsp).channel[ch as usize].adsrState = 4;
            (*dsp).channel[ch as usize].gain = 0;
        }
        (*dsp).ram[0x7ci32 as usize] = ((*dsp).ram[0x7ci32 as usize] as i32 | (1i32) << ch) as u8;
    }
    crate::update_place! { previous fresh0 = (*dsp).channel[ch as usize].decodeOffset => (*dsp).channel[ch as usize].decodeOffset.wrapping_add(1); }
    let header: u8 = *(*dsp).apu_ram.offset(fresh0 as isize);
    let shift: i32 = header as i32 >> 4;
    let filter: i32 = (header as i32 & 0xc) >> 2;
    (*dsp).channel[ch as usize].previousFlags = (header & 0x3);
    let mut curByte: u8 = 0;
    let mut old: i32 = (*dsp).channel[ch as usize].old as i32;
    let mut older: i32 = (*dsp).channel[ch as usize].older as i32;
    let mut i: i32 = 0;
    while i < 16 {
        let mut s: i32 = 0;
        if i & 1 != 0 {
            s = curByte as i32 & 0xf;
        } else {
            crate::update_place! { previous fresh1 = (*dsp).channel[ch as usize].decodeOffset => (*dsp).channel[ch as usize].decodeOffset.wrapping_add(1); }
            curByte = *(*dsp).apu_ram.offset(fresh1 as isize);
            s = curByte as i32 >> 4;
        }
        if s > 7 {
            s -= 16;
        }
        if shift <= 0xc {
            s = s << shift >> 1;
        } else {
            s = (s >> 3) << 12;
        }
        match filter {
            1 => s += old + (-old >> 4),
            2 => s += 2 * old + (3 * -old >> 5) - older + (older >> 4),
            3 => s += 2 * old + (13 * -old >> 6) - older + (3 * older >> 4),
            _ => {}
        }
        s = saturate_sample(s);
        s = ((s & 0x7fff) << 1) as i16 as i32 >> 1;
        older = old;
        old = s;
        (*dsp).channel[ch as usize].decodeBuffer[(i + 3) as usize] = s as i16;
        i += 1;
    }
    (*dsp).channel[ch as usize].older = older as i16;
    (*dsp).channel[ch as usize].old = old as i16;
}
unsafe fn dsp_handleNoise(dsp: *mut Dsp) {
    if (*dsp).noiseRate as i32 != 0 {
        (*dsp).noiseCounter = (*dsp).noiseCounter.wrapping_add(1);
    }
    if (*dsp).noiseCounter as i32 >= (*dsp).noiseRate as i32 && (*dsp).noiseRate as i32 != 0 {
        let bit: i32 = (*dsp).noiseSample as i32 & 1 ^ (*dsp).noiseSample as i32 >> 1 & 1;
        (*dsp).noiseSample = ((*dsp).noiseSample as i32 >> 1 & 0x3fff | bit << 14) as i16;
        (*dsp).noiseSample = ((((*dsp).noiseSample as i32 & 0x7fff) << 1) as i16 >> 1);
        (*dsp).noiseCounter = 0;
    }
}
pub unsafe fn dsp_read(dsp: *mut Dsp, adr: u8) -> u8 {
    (*dsp).ram[adr as usize]
}
pub unsafe fn dsp_write(dsp: *mut Dsp, adr: u8, mut val: u8) {
    let ch: i32 = adr as i32 >> 4;
    match adr as i32 {
        0 | 16 | 32 | 48 | 64 | 80 | 96 | 112 => (*dsp).channel[ch as usize].volumeL = val as i8,
        1 | 17 | 33 | 49 | 65 | 81 | 97 | 113 => (*dsp).channel[ch as usize].volumeR = val as i8,
        2 | 18 | 34 | 50 | 66 | 82 | 98 | 114 => {
            (*dsp).channel[ch as usize].pitch =
                ((*dsp).channel[ch as usize].pitch & 0x3f00 | val as u16)
        }
        3 | 19 | 35 | 51 | 67 | 83 | 99 | 115 => {
            (*dsp).channel[ch as usize].pitch = (((*dsp).channel[ch as usize].pitch as i32 & 0xff
                | (val as i32) << 8)
                & 0x3fff) as u16
        }
        4 | 20 | 36 | 52 | 68 | 84 | 100 | 116 => (*dsp).channel[ch as usize].srcn = val,
        5 | 21 | 37 | 53 | 69 | 85 | 101 | 117 => {
            (*dsp).channel[ch as usize].adsrRates[0i32 as usize] =
                rateValues[((val as i32 & 0xf) * 2 + 1) as usize] as u16;
            (*dsp).channel[ch as usize].adsrRates[1i32 as usize] =
                rateValues[(((val as i32 & 0x70) >> 4) * 2 + 16) as usize] as u16;
            (*dsp).channel[ch as usize].useGain = val & 0x80 == 0;
        }
        6 | 22 | 38 | 54 | 70 | 86 | 102 | 118 => {
            (*dsp).channel[ch as usize].adsrRates[2i32 as usize] =
                rateValues[(val as i32 & 0x1f) as usize] as u16;
            (*dsp).channel[ch as usize].sustainLevel =
                ((((val as i32 & 0xe0) >> 5) + 1) * 0x100) as u16;
        }
        7 | 23 | 39 | 55 | 71 | 87 | 103 | 119 => {
            (*dsp).channel[ch as usize].directGain = val & 0x80 == 0;
            if val & 0x80 != 0 {
                (*dsp).channel[ch as usize].gainMode = ((val as i32 & 0x60) >> 5) as u8;
                (*dsp).channel[ch as usize].adsrRates[3i32 as usize] =
                    rateValues[(val as i32 & 0x1f) as usize] as u16;
            } else {
                (*dsp).channel[ch as usize].gainValue = ((val as i32 & 0x7f) * 16) as u16;
            }
        }
        12 => (*dsp).masterVolumeL = val as i8,
        28 => (*dsp).masterVolumeR = val as i8,
        44 => (*dsp).echoVolumeL = val as i8,
        60 => (*dsp).echoVolumeR = val as i8,
        76 => {
            let mut ch_0: i32 = 0;
            while ch_0 < 8 {
                (*dsp).channel[ch_0 as usize].keyOn = val as i32 & (1i32) << ch_0 != 0;
                if (*dsp).channel[ch_0 as usize].keyOn {
                    (*dsp).channel[ch_0 as usize].keyOn = false_0 != 0;
                    (*dsp).channel[ch_0 as usize].previousFlags = 0;
                    let samplePointer: u16 = ((*dsp).dirPage as i32
                        + 4 * (*dsp).channel[ch_0 as usize].srcn as i32)
                        as u16;
                    (*dsp).channel[ch_0 as usize].decodeOffset =
                        *(*dsp).apu_ram.offset(samplePointer as isize) as u16;
                    (*dsp).channel[ch_0 as usize].decodeOffset =
                        ((*dsp).channel[ch_0 as usize].decodeOffset as i32
                            | (*(*dsp)
                                .apu_ram
                                .offset((samplePointer as i32 + 1 & 0xffff) as isize)
                                as i32)
                                << 8) as u16;
                    memset(
                        &raw mut (*(&raw mut (*dsp).channel as *mut DspChannel)
                            .offset(ch_0 as isize))
                        .decodeBuffer as *mut i16 as *mut c_void,
                        0,
                        size_of::<[i16; 19]>() as size_t,
                    );
                    (*dsp).channel[ch_0 as usize].gain = 0;
                    (*dsp).channel[ch_0 as usize].adsrState =
                        (if (*dsp).channel[ch_0 as usize].useGain as i32 != 0 {
                            3
                        } else {
                            0
                        }) as u8;
                }
                ch_0 += 1;
            }
        }
        92 => {
            let mut ch_1: i32 = 0;
            while ch_1 < 8 {
                (*dsp).channel[ch_1 as usize].keyOff = val as i32 & (1i32) << ch_1 != 0;
                if (*dsp).channel[ch_1 as usize].keyOff {
                    (*dsp).channel[ch_1 as usize].adsrState = 4;
                }
                ch_1 += 1;
            }
        }
        108 => {
            (*dsp).reset = val & 0x80 != 0;
            (*dsp).mute = val & 0x40 != 0;
            (*dsp).echoWrites = val & 0x20 == 0;
            (*dsp).noiseRate = rateValues[(val as i32 & 0x1f) as usize] as u16;
        }
        124 => val = 0,
        13 => (*dsp).feedbackVolume = val as i8,
        45 => {
            let mut i: i32 = 0;
            while i < 8 {
                (*dsp).channel[i as usize].pitchModulation = val as i32 & (1i32) << i != 0;
                i += 1;
            }
        }
        61 => {
            let mut i_0: i32 = 0;
            while i_0 < 8 {
                (*dsp).channel[i_0 as usize].useNoise = val as i32 & (1i32) << i_0 != 0;
                i_0 += 1;
            }
        }
        77 => {
            let mut i_1: i32 = 0;
            while i_1 < 8 {
                (*dsp).channel[i_1 as usize].echoEnable = val as i32 & (1i32) << i_1 != 0;
                i_1 += 1;
            }
        }
        93 => (*dsp).dirPage = ((val as i32) << 8) as u16,
        109 => (*dsp).echoBufferAdr = ((val as i32) << 8) as u16,
        125 => {
            (*dsp).echoDelay = ((val as i32 & 0xf) * 512) as u16;
            if (*dsp).echoDelay as i32 == 0 {
                (*dsp).echoDelay = 1;
            }
        }
        15 | 31 | 47 | 63 | 79 | 95 | 111 | 127 => (*dsp).firValues[ch as usize] = val as i8,
        _ => {}
    }
    (*dsp).ram[adr as usize] = val;
}
pub unsafe fn dsp_getSamples(dsp: *mut Dsp, sampleData: *mut i16, samplesPerFrame: i32) {
    let adder: c_double = 534.0f64 / samplesPerFrame as c_double;
    let mut location: c_double = 0.0f64;
    let mut i: i32 = 0;
    while i < samplesPerFrame {
        *sampleData.offset((i * 2) as isize) = (*dsp).sampleBuffer[(location as i32 * 2) as usize];
        *sampleData.offset((i * 2 + 1) as isize) =
            (*dsp).sampleBuffer[(location as i32 * 2 + 1) as usize];
        location += adder;
        i += 1;
    }
    (*dsp).sampleOffset = 0;
}
