//! Borrowed host capabilities for one retained backend execution interval.
use crate::apu_queue::ApuQueue;
use crate::game_state::WorkRam;
use crate::rom::RomImage;
use crate::snes::snes::Backend;
use crate::types::{Cpu, Point16U, Rect16U};

pub(crate) struct HostContext<'a> {
    pub(crate) ram: &'a WorkRam,
    pub(crate) loaded_rom: &'a mut Option<RomImage>,
    pub(crate) native_rom: &'a Option<RomImage>,
    pub(crate) g_fail: &'a mut bool,
    pub(crate) hookmode: &'a mut u32,
    pub(crate) hookcnt: &'a mut u32,
    pub(crate) hookadr: &'a mut u32,
    pub(crate) hook_fixbug_orgbyte: &'a mut [u8; 1024],
    pub(crate) kPatchedCarrysOrg: &'a [u8; 155],
    pub(crate) g_calling_asm_from_c: &'a mut bool,
    pub(crate) g_calling_asm_from_c_ret: &'a mut i32,
    pub(crate) pc_hist: &'a mut [u32; 8],
    pub(crate) pc_hist_ctr: &'a mut u32,
    pub(crate) snes_frame_counter: &'a i32,
    pub(crate) apu_queue: &'a mut ApuQueue,
    pub(crate) g_debug_apu_cycles: &'a bool,
    pub(crate) g_new_ppu: &'a bool,
    pub(crate) decompress_src: &'a u32,
    pub(crate) eproj_spawn_rect: &'a Rect16U,
    pub(crate) eproj_spawn_r22: &'a u16,
    pub(crate) eproj_spawn_pt: &'a Point16U,
    pub(crate) eproj_spawn_varE24: &'a u16,
}

/// All recursive bus operations share this borrow. No Game borrow or device
/// reference may cross a recursive call or a native callback.
pub(crate) struct Execution<'a> {
    pub(crate) backend: &'a mut Backend,
    pub(crate) host: HostContext<'a>,
}

impl Execution<'_> {
    pub(crate) fn ram(&self) -> &WorkRam {
        self.host.ram
    }

    // This retained raw view is captured afresh, never a copied routing link.
    // The complete bus/native/lifecycle cohort still owns its unsafe contract.
    pub(crate) fn cpu(&self) -> *mut Cpu {
        self.backend.views().1
    }

    // The tracing selector is read after its ordered bus observations.
    pub(crate) unsafe fn selected_cpu(&self) -> *mut Cpu {
        unsafe { (*self.backend.views().0).cpu }
    }

    pub(crate) unsafe fn cycle_apu(&mut self) {
        unsafe { crate::snes::apu::apu_cycle_execution(self) }
    }
}

impl HostContext<'_> {
    pub(crate) fn reborrow(&mut self) -> HostContext<'_> {
        HostContext {
            ram: self.ram,
            loaded_rom: &mut *self.loaded_rom,
            native_rom: self.native_rom,
            g_fail: &mut *self.g_fail,
            hookmode: &mut *self.hookmode,
            hookcnt: &mut *self.hookcnt,
            hookadr: &mut *self.hookadr,
            hook_fixbug_orgbyte: &mut *self.hook_fixbug_orgbyte,
            kPatchedCarrysOrg: self.kPatchedCarrysOrg,
            g_calling_asm_from_c: &mut *self.g_calling_asm_from_c,
            g_calling_asm_from_c_ret: &mut *self.g_calling_asm_from_c_ret,
            pc_hist: &mut *self.pc_hist,
            pc_hist_ctr: &mut *self.pc_hist_ctr,
            snes_frame_counter: self.snes_frame_counter,
            apu_queue: &mut *self.apu_queue,
            g_debug_apu_cycles: self.g_debug_apu_cycles,
            g_new_ppu: self.g_new_ppu,
            decompress_src: self.decompress_src,
            eproj_spawn_rect: self.eproj_spawn_rect,
            eproj_spawn_r22: self.eproj_spawn_r22,
            eproj_spawn_pt: self.eproj_spawn_pt,
            eproj_spawn_varE24: self.eproj_spawn_varE24,
        }
    }
}
