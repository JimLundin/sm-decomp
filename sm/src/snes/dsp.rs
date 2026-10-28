use crate::initial_state::InitialState;
pub use crate::types::{Dsp, DspState, size_t};
use core::mem::size_of;

// The shared DSP state is also used by the native music sequencer and snapshots.
#[link(name = "lakesnes_backend", kind = "static", modifiers = "-bundle")]
unsafe extern "C" {
    fn sm_backend_dsp_cycle(dsp: *mut DspState) -> i32;
}

const _: () = {
    assert!(size_of::<crate::types::DspChannel>() == 86);
    assert!(core::mem::offset_of!(DspState, channel) == 136);
    assert!(core::mem::offset_of!(DspState, sampleBuffer) == 892);
    assert!(size_of::<DspState>() == 3032);
};

#[forbid(unsafe_code)]
impl Dsp {
    pub(crate) fn new(ram: crate::apu_ram::ApuRam) -> Self {
        Self {
            state: DspState {
                apu_ram: ram.as_mut_ptr(),
                ..DspState::new()
            },
            apu_memory: ram,
        }
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
pub unsafe fn dsp_cycle(dsp: *mut Dsp) {
    unsafe {
        // KON/KOFF take effect at register-write time here, not on alternate cycles.
        let even = (*dsp).state.evenCycle;
        (*dsp).state.evenCycle = false;
        // Upstream repeatedly overwrites sample 533; this engine stops at 534.
        let offset = (*dsp).state.sampleOffset;
        let tail = [
            (*dsp).state.sampleBuffer[1066],
            (*dsp).state.sampleBuffer[1067],
        ];
        (*dsp).state.sampleOffset = offset.min(533);
        let failure = sm_backend_dsp_cycle(&raw mut (*dsp).state);
        if failure != 0 {
            (*dsp).state.sampleOffset = offset;
            (*dsp).state.evenCycle = even;
            if failure <= 8 {
                let ch = &(*dsp).state.channel[(failure - 1) as usize];
                let _ = ch.adsrRates[ch.adsrState as usize];
            } else {
                let _ = (*dsp).state.firBufferL[(*dsp).state.firBufferIndex as usize];
            }
            unreachable!();
        }
        if offset >= 534 {
            (&mut (*dsp).state.sampleBuffer)[1066..].copy_from_slice(&tail);
        }
        (*dsp).state.sampleOffset = if offset < 534 { offset + 1 } else { offset };
        (*dsp).state.evenCycle = !even;
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
            apu_ram: self.apu_ram,
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
        self.state.apu_ram = self.apu_memory.as_mut_ptr();
    }

    pub(crate) fn copy_state_from(&mut self, source: &Self) {
        self.restore_state(&source.state);
    }
}
