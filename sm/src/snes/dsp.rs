//! The DSP now lives in `sm_runtime::audio::dsp`; these wrappers keep the pointer-based callers.
pub use sm_runtime::audio::dsp::{Dsp, DspChannel, DspState, dsp_read, write_register};

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
/// The native APU/player must hold exclusive access to this stable DSP allocation.
pub unsafe fn dsp_write(dsp: *mut Dsp, adr: u8, val: u8) {
    unsafe {
        (*dsp).write_register(adr, val);
    }
}
