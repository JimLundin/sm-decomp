use crate::c_library::malloc;
use crate::game_state::Game;
pub use crate::types::{size_t, Dsp, SaveLoadFunc};
use core::{ffi::c_void, mem::size_of};

// The shared DSP state is also used by the native music sequencer and snapshots.
#[link(name = "lakesnes_backend", kind = "static", modifiers = "-bundle")]
unsafe extern "C" {
    #[link_name = "dsp_reset"]
    fn backend_reset(dsp: *mut Dsp);
    #[link_name = "dsp_write"]
    fn backend_write(dsp: *mut Dsp, address: u8, value: u8);
    fn sm_backend_dsp_cycle(dsp: *mut Dsp) -> i32;
}

const _: () = {
    assert!(size_of::<crate::types::DspChannel>() == 86);
    assert!(core::mem::offset_of!(Dsp, channel) == 136);
    assert!(core::mem::offset_of!(Dsp, sampleBuffer) == 892);
    assert!(size_of::<Dsp>() == 3032);
};

pub unsafe fn dsp_init(ram: *mut u8) -> *mut Dsp {
    let dsp = malloc(size_of::<Dsp>() as size_t) as *mut Dsp;
    (*dsp).apu_ram = ram;
    dsp
}
pub unsafe fn dsp_reset(dsp: *mut Dsp) {
    backend_reset(dsp);
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
    // KON/KOFF take effect at register-write time here, not on alternate cycles.
    let even = (*dsp).evenCycle;
    (*dsp).evenCycle = false;
    // Upstream repeatedly overwrites sample 533; this engine stops at 534.
    let offset = (*dsp).sampleOffset;
    let tail = [(*dsp).sampleBuffer[1066], (*dsp).sampleBuffer[1067]];
    (*dsp).sampleOffset = offset.min(533);
    let failure = sm_backend_dsp_cycle(dsp);
    if failure != 0 {
        (*dsp).sampleOffset = offset;
        (*dsp).evenCycle = even;
        if failure <= 8 {
            let ch = &(*dsp).channel[(failure - 1) as usize];
            let _ = ch.adsrRates[ch.adsrState as usize];
        } else {
            let _ = (*dsp).firBufferL[(*dsp).firBufferIndex as usize];
        }
        unreachable!();
    }
    if offset >= 534 {
        (&mut (*dsp).sampleBuffer)[1066..].copy_from_slice(&tail);
    }
    (*dsp).sampleOffset = if offset < 534 { offset + 1 } else { offset };
    (*dsp).evenCycle = !even;
}

pub unsafe fn dsp_read(dsp: *mut Dsp, adr: u8) -> u8 {
    (*dsp).ram[adr as usize]
}

pub unsafe fn dsp_write(dsp: *mut Dsp, adr: u8, val: u8) {
    // Keep the existing bounds failure before crossing the native boundary.
    let _ = (*dsp).ram[adr as usize];
    backend_write(dsp, adr, val);
    if adr == 0x4c {
        for ch in &mut (*dsp).channel {
            if ch.keyOn {
                ch.keyOn = false;
                ch.previousFlags = 0;
                let pointer = (*dsp).dirPage.wrapping_add(4 * u16::from(ch.srcn));
                ch.decodeOffset = u16::from_le_bytes([
                    *(*dsp).apu_ram.add(usize::from(pointer)),
                    *(*dsp).apu_ram.add(usize::from(pointer.wrapping_add(1))),
                ]);
                ch.decodeBuffer.fill(0);
                ch.gain = 0;
                ch.adsrState = if ch.useGain { 3 } else { 0 };
            }
        }
    } else if adr == 0x5c {
        for ch in &mut (*dsp).channel {
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
        *sampleData.offset((i * 2) as isize) = (*dsp).sampleBuffer[(location as i32 * 2) as usize];
        *sampleData.offset((i * 2 + 1) as isize) =
            (*dsp).sampleBuffer[(location as i32 * 2 + 1) as usize];
        location += adder;
        i += 1;
    }
    (*dsp).sampleOffset = 0;
}
