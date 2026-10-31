use crate::initial_state::InitialState;
pub use crate::types::{Dsp, DspState};
use core::mem::size_of;

const _: () = {
    assert!(size_of::<crate::types::DspChannel>() == 86);
    assert!(core::mem::offset_of!(DspState, channel) == 128);
    assert!(core::mem::offset_of!(DspState, sampleBuffer) == 884);
    assert!(size_of::<DspState>() == 3024);
};

#[forbid(unsafe_code)]
impl Dsp {
    pub(crate) fn new(ram: crate::apu_ram::ApuRam) -> Self {
        Self {
            state: DspState::new(),
            apu_memory: ram,
        }
    }

    pub fn cycle(&mut self) {
        self.state.cycle(&self.apu_memory);
    }
}

#[forbid(unsafe_code)]
impl DspState {
    pub fn cycle(&mut self, ram: &crate::apu_ram::ApuRam) {
        // Register writes already apply KON/KOFF; cycle evaluation ignores them.
        let state = self;
        let sample = match cycle_sample(state, ram) {
            Ok(sample) => sample,
            Err(channel) if channel < 8 => {
                let voice = &state.channel[channel];
                let _ = voice.adsrRates[usize::from(voice.adsrState)];
                unreachable!();
            }
            Err(_) => {
                let _ = state.firBufferL[usize::from(state.firBufferIndex)];
                unreachable!();
            }
        };
        // The engine fills 534 frames and then stops committing samples.
        if state.sampleOffset < 534 {
            let index = usize::from(state.sampleOffset) * 2;
            state.sampleBuffer[index] = sample[0];
            state.sampleBuffer[index + 1] = sample[1];
            state.sampleOffset += 1;
        }
        state.evenCycle = !state.evenCycle;
    }
}
pub unsafe fn dsp_reset(dsp: *mut Dsp) {
    unsafe {
        (*dsp).state.reset();
    }
}
pub unsafe fn dsp_saveload(dsp: *mut Dsp, io: &mut crate::snapshot_io::SnapshotIo<'_>) {
    unsafe {
        crate::rtl::snapshot_result(io.state(&mut (*dsp).state));
    }
}
/// The native APU/player must hold exclusive access to this stable DSP allocation.
pub unsafe fn dsp_cycle(dsp: *mut Dsp) {
    unsafe {
        (*dsp).cycle();
    }
}

#[forbid(unsafe_code)]
fn cycle_sample(state: &mut DspState, ram: &crate::apu_ram::ApuRam) -> Result<[i16; 2], usize> {
    let mut total = [0i32; 2];
    for index in 0..8 {
        cycle_voice(state, ram, index)?;
        let voice = &state.channel[index];
        total[0] = (total[0] + ((i32::from(voice.sampleOut) * i32::from(voice.volumeL)) >> 6))
            .clamp(-0x8000, 0x7fff);
        total[1] = (total[1] + ((i32::from(voice.sampleOut) * i32::from(voice.volumeR)) >> 6))
            .clamp(-0x8000, 0x7fff);
    }
    total[0] = ((total[0] * i32::from(state.masterVolumeL)) >> 7).clamp(-0x8000, 0x7fff);
    total[1] = ((total[1] * i32::from(state.masterVolumeR)) >> 7).clamp(-0x8000, 0x7fff);
    // Invalid imported FIR state fails after all voices, before echo RAM writes.
    if state.firBufferIndex >= 8 {
        return Err(8);
    }
    handle_echo(state, ram, &mut total);
    if state.mute {
        total = [0; 2];
    }
    handle_noise(state);
    Ok([total[0] as i16, total[1] as i16])
}

#[forbid(unsafe_code)]
fn cycle_voice(
    state: &mut DspState,
    ram: &crate::apu_ram::ApuRam,
    index: usize,
) -> Result<(), usize> {
    let mut pitch = state.channel[index].pitch;
    if index > 0 && state.channel[index].pitchModulation {
        let previous = state.channel[index - 1].sampleOut;
        // Assignment narrows to native u16 before the pitch clamp, including negative factors.
        pitch = ((i32::from(pitch) * ((i32::from(previous) >> 4) + 0x400)) >> 10) as u16;
        pitch = pitch.min(0x3fff);
    }
    let voice = &mut state.channel[index];
    let counter = u32::from(voice.pitchCounter) + u32::from(pitch);
    if counter > 0xffff {
        decode_brr(voice, ram, state.dirPage, &mut state.ram[0x7c], index);
    }
    voice.pitchCounter = counter as u16;
    // Match checked_dsp_cycle's failure prefix: decoding and pitch commit survive.
    if voice.adsrState > 4 && !state.reset {
        return Err(index);
    }
    let mut sample = if voice.useNoise {
        state.noiseSample
    } else {
        interpolate(
            voice,
            usize::from(voice.pitchCounter >> 12),
            usize::from((voice.pitchCounter >> 4) & 0xff),
        )
    };
    if state.reset {
        voice.adsrState = 4;
        voice.gain = 0;
    }
    let direct = voice.adsrState != 4 && voice.useGain && voice.directGain;
    let rate = if voice.adsrState == 4 {
        0
    } else {
        voice.adsrRates[usize::from(voice.adsrState)]
    };
    if voice.adsrState != 4 && !direct && rate != 0 {
        voice.rateCounter = voice.rateCounter.wrapping_add(1);
    }
    if voice.adsrState == 4 || (!direct && voice.rateCounter >= rate && rate != 0) {
        if voice.adsrState != 4 {
            voice.rateCounter = 0;
        }
        handle_gain(voice);
    }
    if direct {
        voice.gain = voice.gainValue;
    }
    state.ram[(index << 4) | 8] = (voice.gain >> 4) as u8;
    sample = ((i32::from(sample) * i32::from(voice.gain)) >> 11) as i16;
    state.ram[(index << 4) | 9] = (sample >> 7) as u8;
    voice.sampleOut = sample;
    Ok(())
}

#[forbid(unsafe_code)]
fn handle_gain(voice: &mut crate::types::DspChannel) {
    // Native subtraction promotes u16 to signed int; gain zero stays zero here.
    let exponential = (((i32::from(voice.gain) - 1) >> 8) + 1) as u16;
    match voice.adsrState {
        0 => {
            voice.gain = voice
                .gain
                .wrapping_add(if voice.adsrRates[0] == 1 { 1024 } else { 32 });
            if voice.gain >= 0x7e0 {
                voice.adsrState = 1;
            }
            voice.gain = voice.gain.min(0x7ff);
        }
        1 => {
            voice.gain = voice.gain.wrapping_sub(exponential);
            if voice.gain < voice.sustainLevel {
                voice.adsrState = 2;
            }
        }
        2 => voice.gain = voice.gain.wrapping_sub(exponential),
        3 => match voice.gainMode {
            0 => {
                voice.gain = voice.gain.wrapping_sub(32);
                if voice.gain > 0x7ff {
                    voice.gain = 0;
                }
            }
            1 => voice.gain = voice.gain.wrapping_sub(exponential),
            2 => voice.gain = voice.gain.wrapping_add(32).min(0x7ff),
            3 => {
                voice.gain = voice
                    .gain
                    .wrapping_add(if voice.gain < 0x600 { 32 } else { 8 })
                    .min(0x7ff)
            }
            _ => {}
        },
        4 => {
            voice.gain = voice.gain.wrapping_sub(8);
            if voice.gain > 0x7ff {
                voice.gain = 0;
            }
        }
        _ => {}
    }
}

#[forbid(unsafe_code)]
fn decode_brr(
    voice: &mut crate::types::DspChannel,
    ram: &crate::apu_ram::ApuRam,
    directory: u16,
    end: &mut u8,
    index: usize,
) {
    voice.decodeBuffer[0] = voice.decodeBuffer[16];
    voice.decodeBuffer[1] = voice.decodeBuffer[17];
    voice.decodeBuffer[2] = voice.decodeBuffer[18];
    if voice.previousFlags == 1 || voice.previousFlags == 3 {
        let pointer = directory.wrapping_add(4 * u16::from(voice.srcn));
        voice.decodeOffset = u16::from_le_bytes([
            ram.read(pointer.wrapping_add(2)),
            ram.read(pointer.wrapping_add(3)),
        ]);
        if voice.previousFlags == 1 {
            voice.adsrState = 4;
            voice.gain = 0;
        }
        *end |= 1 << index;
    }
    let header = ram.read(voice.decodeOffset);
    voice.decodeOffset = voice.decodeOffset.wrapping_add(1);
    let shift = header >> 4;
    let filter = (header & 0x0c) >> 2;
    voice.previousFlags = header & 3;
    let mut byte = 0;
    let mut old = i32::from(voice.old);
    let mut older = i32::from(voice.older);
    for sample_index in 0..16 {
        let mut sample = if sample_index & 1 != 0 {
            i32::from(byte & 0x0f)
        } else {
            byte = ram.read(voice.decodeOffset);
            voice.decodeOffset = voice.decodeOffset.wrapping_add(1);
            i32::from(byte >> 4)
        };
        if sample > 7 {
            sample -= 16;
        }
        sample = if shift <= 12 {
            (sample << shift) >> 1
        } else {
            (sample >> 3) << 12
        };
        sample += match filter {
            1 => old + (-old >> 4),
            2 => 2 * old + ((3 * -old) >> 5) - older + (older >> 4),
            3 => 2 * old + ((13 * -old) >> 6) - older + ((3 * older) >> 4),
            _ => 0,
        };
        sample = sample.clamp(-0x8000, 0x7fff);
        sample = i32::from(((sample & 0x7fff) << 1) as i16 >> 1);
        older = old;
        old = sample;
        voice.decodeBuffer[sample_index + 3] = sample as i16;
    }
    voice.older = older as i16;
    voice.old = old as i16;
}

#[forbid(unsafe_code)]
fn interpolate(voice: &crate::types::DspChannel, number: usize, offset: usize) -> i16 {
    let newest = i32::from(voice.decodeBuffer[number + 3]);
    let old = i32::from(voice.decodeBuffer[number + 2]);
    let older = i32::from(voice.decodeBuffer[number + 1]);
    let oldest = i32::from(voice.decodeBuffer[number]);
    let mut output = (GAUSS[0xff - offset] * oldest) >> 10;
    output += (GAUSS[0x1ff - offset] * older) >> 10;
    output += (GAUSS[0x100 + offset] * old) >> 10;
    output = i32::from(output as i16);
    output += (GAUSS[offset] * newest) >> 10;
    (output.clamp(-0x8000, 0x7fff) >> 1) as i16
}

#[forbid(unsafe_code)]
fn handle_echo(state: &mut DspState, ram: &crate::apu_ram::ApuRam, output: &mut [i32; 2]) {
    let address = state
        .echoBufferAdr
        .wrapping_add(state.echoBufferIndex.wrapping_mul(4));
    let index = usize::from(state.firBufferIndex);
    state.firBufferL[index] =
        i16::from_le_bytes([ram.read(address), ram.read(address.wrapping_add(1))]);
    state.firBufferL[index] >>= 1;
    state.firBufferR[index] = i16::from_le_bytes([
        ram.read(address.wrapping_add(2)),
        ram.read(address.wrapping_add(3)),
    ]);
    state.firBufferR[index] >>= 1;
    let mut sum = [0i32; 2];
    for tap in 0..8 {
        let buffer_index = (index + tap + 1) & 7;
        sum[0] +=
            (i32::from(state.firBufferL[buffer_index]) * i32::from(state.firValues[tap])) >> 6;
        sum[1] +=
            (i32::from(state.firBufferR[buffer_index]) * i32::from(state.firValues[tap])) >> 6;
        if tap == 6 {
            sum[0] = i32::from(sum[0] as i16);
            sum[1] = i32::from(sum[1] as i16);
        }
    }
    sum[0] = sum[0].clamp(-0x8000, 0x7fff);
    sum[1] = sum[1].clamp(-0x8000, 0x7fff);
    output[0] = (output[0] + ((sum[0] * i32::from(state.echoVolumeL)) >> 7)).clamp(-0x8000, 0x7fff);
    output[1] = (output[1] + ((sum[1] * i32::from(state.echoVolumeR)) >> 7)).clamp(-0x8000, 0x7fff);
    let mut input = [0i32; 2];
    for voice in &state.channel {
        if voice.echoEnable {
            input[0] = (input[0] + ((i32::from(voice.sampleOut) * i32::from(voice.volumeL)) >> 6))
                .clamp(-0x8000, 0x7fff);
            input[1] = (input[1] + ((i32::from(voice.sampleOut) * i32::from(voice.volumeR)) >> 6))
                .clamp(-0x8000, 0x7fff);
        }
    }
    input[0] = (input[0] + ((sum[0] * i32::from(state.feedbackVolume)) >> 7))
        .clamp(-0x8000, 0x7fff)
        & 0xfffe;
    input[1] = (input[1] + ((sum[1] * i32::from(state.feedbackVolume)) >> 7))
        .clamp(-0x8000, 0x7fff)
        & 0xfffe;
    if state.echoWrites {
        ram.write(address, input[0] as u8);
        ram.write(address.wrapping_add(1), (input[0] >> 8) as u8);
        ram.write(address.wrapping_add(2), input[1] as u8);
        ram.write(address.wrapping_add(3), (input[1] >> 8) as u8);
    }
    state.firBufferIndex = state.firBufferIndex.wrapping_add(1) & 7;
    state.echoBufferIndex = state.echoBufferIndex.wrapping_add(1);
    state.echoRemain = state.echoRemain.wrapping_sub(1);
    if state.echoRemain == 0 {
        state.echoRemain = state.echoDelay;
        state.echoBufferIndex = 0;
    }
}

#[forbid(unsafe_code)]
fn handle_noise(state: &mut DspState) {
    if state.noiseRate != 0 {
        state.noiseCounter = state.noiseCounter.wrapping_add(1);
    }
    if state.noiseCounter >= state.noiseRate && state.noiseRate != 0 {
        let bit = (state.noiseSample & 1) ^ ((state.noiseSample >> 1) & 1);
        state.noiseSample = ((state.noiseSample >> 1) & 0x3fff) | (bit << 14);
        state.noiseSample = ((i32::from(state.noiseSample) & 0x7fff) << 1) as i16 >> 1;
        state.noiseCounter = 0;
    }
}

#[forbid(unsafe_code)]
pub fn dsp_read(state: &DspState, address: u8) -> u8 {
    state.ram[usize::from(address)]
}

/// The native APU/player must hold exclusive access to this stable DSP allocation.
pub unsafe fn dsp_write(dsp: *mut Dsp, adr: u8, val: u8) {
    unsafe {
        write_register(&mut (*dsp).state, &(*dsp).apu_memory, adr, val);
    }
}

#[forbid(unsafe_code)]
pub fn write_register(
    state: &mut DspState,
    ram: &crate::apu_ram::ApuRam,
    address: u8,
    mut value: u8,
) {
    const RATES: [u16; 32] = [
        0, 2048, 1536, 1280, 1024, 768, 640, 512, 384, 320, 256, 192, 160, 128, 96, 80, 64, 48, 40,
        32, 24, 20, 16, 12, 10, 8, 6, 5, 4, 3, 2, 1,
    ];
    // Full-u8 callers retain the original bounds failure before any state mutation.
    let _ = state.ram[usize::from(address)];
    let channel = usize::from(address >> 4);
    match address {
        address if address & 0x0f <= 7 => {
            let voice = &mut state.channel[channel];
            match address & 0x0f {
                0 => voice.volumeL = value as i8,
                1 => voice.volumeR = value as i8,
                2 => voice.pitch = (voice.pitch & 0x3f00) | u16::from(value),
                3 => voice.pitch = ((voice.pitch & 0x00ff) | (u16::from(value) << 8)) & 0x3fff,
                4 => voice.srcn = value,
                5 => {
                    voice.adsrRates[0] = RATES[usize::from((value & 0x0f) * 2 + 1)];
                    voice.adsrRates[1] = RATES[usize::from(((value & 0x70) >> 4) * 2 + 16)];
                    voice.useGain = value & 0x80 == 0;
                }
                6 => {
                    voice.adsrRates[2] = RATES[usize::from(value & 0x1f)];
                    voice.sustainLevel = (u16::from((value & 0xe0) >> 5) + 1) * 0x100;
                }
                7 => {
                    voice.directGain = value & 0x80 == 0;
                    if value & 0x80 != 0 {
                        voice.gainMode = (value & 0x60) >> 5;
                        voice.adsrRates[3] = RATES[usize::from(value & 0x1f)];
                    } else {
                        voice.gainValue = u16::from(value & 0x7f) * 16;
                    }
                }
                _ => unreachable!(),
            }
        }
        0x0c => state.masterVolumeL = value as i8,
        0x1c => state.masterVolumeR = value as i8,
        0x2c => state.echoVolumeL = value as i8,
        0x3c => state.echoVolumeR = value as i8,
        0x4c => {
            for (index, voice) in state.channel.iter_mut().enumerate() {
                voice.keyOn = value & (1 << index) != 0;
            }
        }
        0x5c => {
            for (index, voice) in state.channel.iter_mut().enumerate() {
                voice.keyOff = value & (1 << index) != 0;
            }
        }
        0x6c => {
            state.reset = value & 0x80 != 0;
            state.mute = value & 0x40 != 0;
            state.echoWrites = value & 0x20 == 0;
            state.noiseRate = RATES[usize::from(value & 0x1f)];
        }
        0x7c => value = 0, // Every write clears the ENDx register.
        0x0d => state.feedbackVolume = value as i8,
        0x2d => {
            for (index, voice) in state.channel.iter_mut().enumerate() {
                voice.pitchModulation = value & (1 << index) != 0;
            }
        }
        0x3d => {
            for (index, voice) in state.channel.iter_mut().enumerate() {
                voice.useNoise = value & (1 << index) != 0;
            }
        }
        0x4d => {
            for (index, voice) in state.channel.iter_mut().enumerate() {
                voice.echoEnable = value & (1 << index) != 0;
            }
        }
        0x5d => state.dirPage = u16::from(value) << 8,
        0x6d => state.echoBufferAdr = u16::from(value) << 8,
        0x7d => state.echoDelay = (u16::from(value & 0x0f) * 512).max(1),
        address if address & 0x0f == 0x0f => state.firValues[channel] = value as i8,
        _ => {}
    }
    state.ram[usize::from(address)] = value;
    apply_keys(state, ram, address);
}

#[forbid(unsafe_code)]
fn apply_keys(state: &mut DspState, ram: &crate::apu_ram::ApuRam, adr: u8) {
    if adr == 0x4c {
        for ch in &mut state.channel {
            if ch.keyOn {
                ch.keyOn = false;
                ch.previousFlags = 0;
                let pointer = state.dirPage.wrapping_add(4 * u16::from(ch.srcn));
                ch.decodeOffset =
                    u16::from_le_bytes([ram.read(pointer), ram.read(pointer.wrapping_add(1))]);
                ch.decodeBuffer.fill(0);
                ch.gain = 0;
                ch.adsrState = if ch.useGain { 3 } else { 0 };
            }
        }
    } else if adr == 0x5c {
        for ch in &mut state.channel {
            if ch.keyOff {
                ch.adsrState = 4;
            }
        }
    }
}
#[forbid(unsafe_code)]
impl DspState {
    pub fn new() -> Self {
        Self::INITIAL
    }

    pub fn reset(&mut self) {
        self.ram.fill(0);
        self.ram[0x7c] = 0xff;
        for voice in &mut self.channel {
            *voice = crate::types::DspChannel {
                snapshot_padding0: voice.snapshot_padding0,
                snapshot_padding1: voice.snapshot_padding1,
                snapshot_padding2: voice.snapshot_padding2,
                snapshot_padding3: voice.snapshot_padding3,
                snapshot_padding4: voice.snapshot_padding4,
                snapshot_padding5: voice.snapshot_padding5,
                snapshot_padding6: voice.snapshot_padding6,
                ..crate::types::DspChannel::INITIAL
            };
        }
        *self = Self {
            ram: self.ram,
            channel: self.channel,
            snapshot_padding0: self.snapshot_padding0,
            snapshot_padding1: self.snapshot_padding1,
            snapshot_padding2: self.snapshot_padding2,
            mute: true,
            reset: true,
            noiseSample: -0x4000,
            echoDelay: 1,
            echoRemain: 1,
            ..Self::new()
        };
    }

    pub fn write_samples(&mut self, output: &mut [[i16; 2]]) {
        let adder = 534.0 / output.len() as f64;
        let mut location = 0.0;
        for frame in output {
            let index = (location as i32 * 2) as usize;
            *frame = [self.sampleBuffer[index], self.sampleBuffer[index + 1]];
            location += adder;
        }
        self.sampleOffset = 0;
    }
}

#[forbid(unsafe_code)]
impl Dsp {
    pub(crate) fn restore_state(&mut self, source: &DspState) {
        self.state = *source;
    }

    pub(crate) fn copy_state_from(&mut self, source: &Self) {
        self.restore_state(&source.state);
    }
}

// Exact interpolation coefficients from the retained DSP reference.
const GAUSS: [i32; 512] = [
    0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000,
    0x000, 0x000, 0x000, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001,
    0x001, 0x002, 0x002, 0x002, 0x002, 0x002, 0x002, 0x002, 0x003, 0x003, 0x003, 0x003, 0x003,
    0x004, 0x004, 0x004, 0x004, 0x004, 0x005, 0x005, 0x005, 0x005, 0x006, 0x006, 0x006, 0x006,
    0x007, 0x007, 0x007, 0x008, 0x008, 0x008, 0x009, 0x009, 0x009, 0x00A, 0x00A, 0x00A, 0x00B,
    0x00B, 0x00B, 0x00C, 0x00C, 0x00D, 0x00D, 0x00E, 0x00E, 0x00F, 0x00F, 0x00F, 0x010, 0x010,
    0x011, 0x011, 0x012, 0x013, 0x013, 0x014, 0x014, 0x015, 0x015, 0x016, 0x017, 0x017, 0x018,
    0x018, 0x019, 0x01A, 0x01B, 0x01B, 0x01C, 0x01D, 0x01D, 0x01E, 0x01F, 0x020, 0x020, 0x021,
    0x022, 0x023, 0x024, 0x024, 0x025, 0x026, 0x027, 0x028, 0x029, 0x02A, 0x02B, 0x02C, 0x02D,
    0x02E, 0x02F, 0x030, 0x031, 0x032, 0x033, 0x034, 0x035, 0x036, 0x037, 0x038, 0x03A, 0x03B,
    0x03C, 0x03D, 0x03E, 0x040, 0x041, 0x042, 0x043, 0x045, 0x046, 0x047, 0x049, 0x04A, 0x04C,
    0x04D, 0x04E, 0x050, 0x051, 0x053, 0x054, 0x056, 0x057, 0x059, 0x05A, 0x05C, 0x05E, 0x05F,
    0x061, 0x063, 0x064, 0x066, 0x068, 0x06A, 0x06B, 0x06D, 0x06F, 0x071, 0x073, 0x075, 0x076,
    0x078, 0x07A, 0x07C, 0x07E, 0x080, 0x082, 0x084, 0x086, 0x089, 0x08B, 0x08D, 0x08F, 0x091,
    0x093, 0x096, 0x098, 0x09A, 0x09C, 0x09F, 0x0A1, 0x0A3, 0x0A6, 0x0A8, 0x0AB, 0x0AD, 0x0AF,
    0x0B2, 0x0B4, 0x0B7, 0x0BA, 0x0BC, 0x0BF, 0x0C1, 0x0C4, 0x0C7, 0x0C9, 0x0CC, 0x0CF, 0x0D2,
    0x0D4, 0x0D7, 0x0DA, 0x0DD, 0x0E0, 0x0E3, 0x0E6, 0x0E9, 0x0EC, 0x0EF, 0x0F2, 0x0F5, 0x0F8,
    0x0FB, 0x0FE, 0x101, 0x104, 0x107, 0x10B, 0x10E, 0x111, 0x114, 0x118, 0x11B, 0x11E, 0x122,
    0x125, 0x129, 0x12C, 0x130, 0x133, 0x137, 0x13A, 0x13E, 0x141, 0x145, 0x148, 0x14C, 0x150,
    0x153, 0x157, 0x15B, 0x15F, 0x162, 0x166, 0x16A, 0x16E, 0x172, 0x176, 0x17A, 0x17D, 0x181,
    0x185, 0x189, 0x18D, 0x191, 0x195, 0x19A, 0x19E, 0x1A2, 0x1A6, 0x1AA, 0x1AE, 0x1B2, 0x1B7,
    0x1BB, 0x1BF, 0x1C3, 0x1C8, 0x1CC, 0x1D0, 0x1D5, 0x1D9, 0x1DD, 0x1E2, 0x1E6, 0x1EB, 0x1EF,
    0x1F3, 0x1F8, 0x1FC, 0x201, 0x205, 0x20A, 0x20F, 0x213, 0x218, 0x21C, 0x221, 0x226, 0x22A,
    0x22F, 0x233, 0x238, 0x23D, 0x241, 0x246, 0x24B, 0x250, 0x254, 0x259, 0x25E, 0x263, 0x267,
    0x26C, 0x271, 0x276, 0x27B, 0x280, 0x284, 0x289, 0x28E, 0x293, 0x298, 0x29D, 0x2A2, 0x2A6,
    0x2AB, 0x2B0, 0x2B5, 0x2BA, 0x2BF, 0x2C4, 0x2C9, 0x2CE, 0x2D3, 0x2D8, 0x2DC, 0x2E1, 0x2E6,
    0x2EB, 0x2F0, 0x2F5, 0x2FA, 0x2FF, 0x304, 0x309, 0x30E, 0x313, 0x318, 0x31D, 0x322, 0x326,
    0x32B, 0x330, 0x335, 0x33A, 0x33F, 0x344, 0x349, 0x34E, 0x353, 0x357, 0x35C, 0x361, 0x366,
    0x36B, 0x370, 0x374, 0x379, 0x37E, 0x383, 0x388, 0x38C, 0x391, 0x396, 0x39B, 0x39F, 0x3A4,
    0x3A9, 0x3AD, 0x3B2, 0x3B7, 0x3BB, 0x3C0, 0x3C5, 0x3C9, 0x3CE, 0x3D2, 0x3D7, 0x3DC, 0x3E0,
    0x3E5, 0x3E9, 0x3ED, 0x3F2, 0x3F6, 0x3FB, 0x3FF, 0x403, 0x408, 0x40C, 0x410, 0x415, 0x419,
    0x41D, 0x421, 0x425, 0x42A, 0x42E, 0x432, 0x436, 0x43A, 0x43E, 0x442, 0x446, 0x44A, 0x44E,
    0x452, 0x455, 0x459, 0x45D, 0x461, 0x465, 0x468, 0x46C, 0x470, 0x473, 0x477, 0x47A, 0x47E,
    0x481, 0x485, 0x488, 0x48C, 0x48F, 0x492, 0x496, 0x499, 0x49C, 0x49F, 0x4A2, 0x4A6, 0x4A9,
    0x4AC, 0x4AF, 0x4B2, 0x4B5, 0x4B7, 0x4BA, 0x4BD, 0x4C0, 0x4C3, 0x4C5, 0x4C8, 0x4CB, 0x4CD,
    0x4D0, 0x4D2, 0x4D5, 0x4D7, 0x4D9, 0x4DC, 0x4DE, 0x4E0, 0x4E3, 0x4E5, 0x4E7, 0x4E9, 0x4EB,
    0x4ED, 0x4EF, 0x4F1, 0x4F3, 0x4F5, 0x4F6, 0x4F8, 0x4FA, 0x4FB, 0x4FD, 0x4FF, 0x500, 0x502,
    0x503, 0x504, 0x506, 0x507, 0x508, 0x50A, 0x50B, 0x50C, 0x50D, 0x50E, 0x50F, 0x510, 0x511,
    0x511, 0x512, 0x513, 0x514, 0x514, 0x515, 0x516, 0x516, 0x517, 0x517, 0x517, 0x518, 0x518,
    0x518, 0x518, 0x518, 0x519, 0x519,
];
