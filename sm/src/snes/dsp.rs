use crate::initial_state::InitialState;
pub use crate::types::{size_t, Dsp, DspState};
use core::mem::size_of;

// The shared DSP state is also used by the native music sequencer and snapshots.
#[link(name = "lakesnes_backend", kind = "static", modifiers = "-bundle")]
unsafe extern "C" {
    #[link_name = "dsp_reset"]
    fn backend_reset(dsp: *mut DspState);
    #[link_name = "dsp_write"]
    fn backend_write(dsp: *mut DspState, address: u8, value: u8);
    fn sm_backend_dsp_cycle(dsp: *mut DspState) -> i32;
}

const _: () = {
    assert!(size_of::<crate::types::DspChannel>() == 86);
    assert!(core::mem::offset_of!(DspState, channel) == 136);
    assert!(core::mem::offset_of!(DspState, sampleBuffer) == 892);
    assert!(size_of::<DspState>() == 3032);
};

#[forbid(unsafe_code)]
pub fn dsp_init(ram: crate::apu_ram::ApuRam) -> *mut Dsp {
    Box::into_raw(Box::new(Dsp {
        state: DspState {
            apu_ram: ram.as_mut_ptr(),
            ..DspState::INITIAL
        },
        apu_memory: ram,
    }))
}
pub unsafe fn dsp_reset(dsp: *mut Dsp) {
    backend_reset(&raw mut (*dsp).state);
}
pub unsafe fn dsp_saveload(dsp: *mut Dsp, io: &mut crate::snapshot_io::SnapshotIo<'_>) {
    crate::rtl::snapshot_result(io.state(&mut (*dsp).state));
}
pub unsafe fn dsp_cycle(dsp: *mut Dsp) {
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

pub unsafe fn dsp_read(dsp: *mut Dsp, adr: u8) -> u8 {
    (*dsp).state.ram[adr as usize]
}

pub unsafe fn dsp_write(dsp: *mut Dsp, adr: u8, val: u8) {
    // Keep the existing bounds failure before crossing the native boundary.
    let _ = (*dsp).state.ram[adr as usize];
    backend_write(&raw mut (*dsp).state, adr, val);
    apply_keys(&mut (*dsp).state, &(*dsp).apu_memory, adr);
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
pub unsafe fn dsp_getSamples(dsp: *mut Dsp, sampleData: *mut i16, samplesPerFrame: i32) {
    let adder: f64 = 534.0f64 / samplesPerFrame as f64;
    let mut location: f64 = 0.0f64;
    let mut i: i32 = 0;
    while i < samplesPerFrame {
        *sampleData.offset((i * 2) as isize) =
            (*dsp).state.sampleBuffer[(location as i32 * 2) as usize];
        *sampleData.offset((i * 2 + 1) as isize) =
            (*dsp).state.sampleBuffer[(location as i32 * 2 + 1) as usize];
        location += adder;
        i += 1;
    }
    (*dsp).state.sampleOffset = 0;
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
