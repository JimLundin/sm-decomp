//! Legacy-compatible SPC700 execution through the pinned LakeSnes dependency.

use crate::initial_state::InitialState;
use crate::snes::apu::apu_cpuRead;
use crate::types::{Apu, Spc};
use core::mem::{offset_of, size_of};

const _: () = {
    assert!(offset_of!(Spc, a) == size_of::<*mut Apu>());
    assert!(offset_of!(Spc, pc) == size_of::<*mut Apu>() + 4);
    assert!(offset_of!(Spc, c) == size_of::<*mut Apu>() + 6);
    assert!(offset_of!(Spc, cyclesUsed) == size_of::<*mut Apu>() + 15);
};

// The dependency only follows Spc fields; its Apu pointer stays opaque.
// Keep the live Rust layout and legacy save span, including reset read order.
#[link(name = "lakesnes_spc", kind = "static", modifiers = "-bundle")]
unsafe extern "C-unwind" {
    pub fn spc_runOpcode(spc: *mut Spc) -> i32;
}

#[forbid(unsafe_code)]
pub fn spc_init(apu: *mut Apu) -> *mut Spc {
    Box::into_raw(Box::new(Spc {
        apu,
        ..Spc::INITIAL
    }))
}
pub unsafe fn spc_reset(spc: *mut Spc) {
    unsafe {
        set_fields! {[(*spc)], 0; a, x, y, sp}
        (*spc).pc = (apu_cpuRead((*spc).apu, 0xfffe) as i32
            | (apu_cpuRead((*spc).apu, 0xffff) as i32) << 8) as u16;
        set_fields! {[(*spc)]; c = false, z = false, v = false,
        n = false, i = false, h = false, p = false, b = false,
        stopped = false, cyclesUsed = 0, };
    }
}
pub unsafe fn spc_saveload(spc: *mut Spc, io: &mut crate::snapshot_io::SnapshotIo<'_>) {
    unsafe {
        crate::rtl::snapshot_result(io.state(&mut *spc));
    }
}
