//! The host has no sound CPU: its ports hold what the engine wrote, and the
//! native driver in `sm_runtime` plays the commands.
use crate::execution::Execution;
pub use crate::types::{Apu, ApuRegisters, DspRegWriteHistory, Timer};
pub unsafe fn apu_reset(apu: *mut Apu) {
    unsafe {
        (*apu).registers.inPorts = [0; 6];
        (*apu).registers.outPorts = [0; 4];
    }
}
pub unsafe fn apu_saveload(_apu: *mut Apu, _io: &mut crate::snapshot_io::SnapshotIo<'_>) {}
pub(crate) unsafe fn apu_cycle_execution(_game: &mut Execution<'_>) {}
pub(crate) fn snapshot_layout(layout: &mut String) {
    layout.push_str("apu-snapshot{}");
}
