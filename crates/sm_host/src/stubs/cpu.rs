//! The host has no CPU: its register block is inert, never stepped.
use crate::execution::Execution;
use crate::types::Cpu;
pub unsafe fn cpu_saveload(_cpu: *mut Cpu, _io: &mut crate::snapshot_io::SnapshotIo<'_>) {}
impl Execution<'_> {
    pub(crate) unsafe fn reset_cpu(&mut self) {}
    pub(crate) unsafe fn dump_cpu_history(&mut self) {}
    pub(crate) unsafe fn step_cpu(&mut self) -> i32 {
        panic!("the host has no CPU to step")
    }
}
