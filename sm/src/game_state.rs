//! Explicit per-game ownership; RAM bytes retain their exact SNES layout.
use crate::cpu_infra::Snapshot;
use crate::rtl::{ApuWriteEnt, StateRecorder};
use crate::types::*;
use std::ffi::c_int;
/// State for one game and its differential oracle. Keep this heap allocation
/// fixed while the emulated hardware holds pointers into `g_ram`.
pub struct Game {
    pub g_ram: [u8; 131072],
    pub g_rom: *const u8,
    pub decompress_src: u32,
    pub eproj_spawn_rect: Rect16U,
    pub eproj_spawn_r22: u16,
    pub eproj_spawn_pt: Point16U,
    pub eproj_spawn_varE24: u16,
    pub cur_coll_amt32: *mut i32,
    pub message_box_das0l_value: u16,
    pub k_out: u16,

    pub g_sram: *mut uint8,
    pub g_rtl_memory_ptr: *mut uint8,
    pub g_rtl_runframe: Option<RunFrameFunc>,
    pub g_rtl_syncall: Option<SyncAllFunc>,
    pub state_recorder: StateRecorder,
    pub g_apu_write: ApuWriteEnt,
    pub g_apu_write_ents: [ApuWriteEnt; 16],
    pub g_apu_time_since_empty: uint8,
    pub g_apu_write_ent_pos: uint8,
    pub g_apu_queue_size: uint8,
    pub g_runmode: uint8,
    pub g_snes: *mut Snes,
    pub g_cpu: *mut Cpu,
    pub g_calling_asm_from_c: bool,
    pub g_calling_asm_from_c_ret: ::core::ffi::c_int,
    pub g_fail: bool,
    pub g_use_my_apu_code: bool,
    pub g_snapshot_mine: Snapshot,
    pub g_snapshot_theirs: Snapshot,
    pub g_snapshot_before: Snapshot,
    pub hookmode: uint32,
    pub hookcnt: uint32,
    pub hookadr: uint32,
    pub hook_orgbyte: [uint8; 1024],
    pub hook_fixbug_orgbyte: [uint8; 1024],
    pub kPatchedCarrysOrg: [uint8; 155],
    pub currently_installed_bug_fix_counter: uint16,
    pub VERIFY_ACTIVE: bool,
    pub VERIFY_FAILURES: u64,
    pub g_spc_player: *mut SpcPlayer,
    pub g_got_mismatch_count: c_int,
    pub g_other_image: bool,
    pub g_debug_flag: bool,
    pub g_new_ppu: bool,
    pub snes_frame_counter: ::core::ffi::c_int,
    pub pc_hist: [uint32_t; 8],
    pub pc_hist_ctr: uint32_t,
    pub pc_bp: uint32_t,
    pub g_debug_apu_cycles: bool,
}
// Initialize the large RAM and oracle snapshots directly on the heap.
impl Game {
    /// Current enemy slot offset ($7E:0E54 in ram_map).
    pub fn cur_enemy_index(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xe54], self.g_ram[0xe55]])
    }

    /// Preserve the original zero initialization, including RM_BOTH (0).
    pub fn new() -> Box<Self> {
        let mut storage = Box::<Self>::new_uninit();
        let game = storage.as_mut_ptr();
        // All fields in this mechanical ownership batch are integers,
        // integer arrays, bools, raw pointers, Option<fn>, or plain structs
        // composed only of those types. Zero is a valid representation for
        // each; zeroed function options are still written explicitly below.
        // The entire allocation is initialized in place, avoiding a stack
        // temporary containing the 128 KiB RAM and three large snapshots.
        unsafe {
            core::ptr::write_bytes(game, 0, 1);
            core::ptr::addr_of_mut!((*game).g_rtl_runframe).write(None);
            core::ptr::addr_of_mut!((*game).g_rtl_syncall).write(None);
            core::ptr::addr_of_mut!((*game).g_use_my_apu_code).write(true);
            core::ptr::addr_of_mut!((*game).g_new_ppu).write(true);
            storage.assume_init()
        }
    }
}
// Mutex ownership is explicit and supplied through SDL userdata. No global
// Game exists; callback and frame acquire exclusivity before creating &mut.
pub struct GameOwner {
    gate: std::sync::Mutex<()>,
    game: std::cell::UnsafeCell<Box<Game>>,
}
// SDL callback and main share this owner; private game access is gate-protected.
unsafe impl Sync for GameOwner {}
impl GameOwner {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            gate: std::sync::Mutex::new(()),
            game: std::cell::UnsafeCell::new(Game::new()),
        })
    }
    pub fn with<R>(&self, operation: impl FnOnce(&mut Game) -> R) -> R {
        let _guard = self.gate.lock().unwrap();
        operation(unsafe { &mut **self.game.get() })
    }
}
