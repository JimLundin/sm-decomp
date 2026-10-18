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
    loaded_rom: Option<LoadedRom>,
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
// A registered allocation view. Cartridge reload/free are unsafe operations;
// they must invalidate or replace this view before releasing its allocation.
struct LoadedRom {
    bytes: *const u8,
    len: usize,
}

// Initialize the large RAM and oracle snapshots directly on the heap.
impl Game {
    /// Register the cartridge's ROM after a successful load.
    ///
    /// # Safety
    /// `bytes` must identify `len` initialized bytes that remain allocated
    /// until this view is replaced or the Game is discarded. Mutations must
    /// be serialized with Game access; no concurrent writer may touch them.
    pub(crate) unsafe fn register_rom(&mut self, bytes: *const u8, len: usize) {
        assert!(!bytes.is_null());
        self.loaded_rom = Some(LoadedRom { bytes, len });
    }

    /// Read a byte at a LoROM address plus a linear offset in the allocation.
    pub fn read_rom_byte(&self, address: u32, byte_offset: usize) -> u8 {
        let start = self.rom_offset(address, byte_offset, 1);
        let rom = self.loaded_rom.as_ref().expect("ROM must be loaded");
        // Registration guarantees allocation validity; rom_offset checks bounds.
        unsafe { *rom.bytes.add(start) }
    }

    /// Read a little-endian word at a LoROM address plus a linear byte offset.
    /// Adding the offset after mapping preserves table reads across bank edges.
    pub fn read_rom_word(&self, address: u32, byte_offset: usize) -> u16 {
        let start = self.rom_offset(address, byte_offset, 2);
        let rom = self.loaded_rom.as_ref().expect("ROM must be loaded");
        // Registration guarantees allocation validity; bounds cover both bytes.
        unsafe { u16::from_le_bytes([*rom.bytes.add(start), *rom.bytes.add(start + 1)]) }
    }

    fn rom_offset(&self, address: u32, byte_offset: usize, width: usize) -> usize {
        assert!(
            address & 0x8000 != 0,
            "ROM address must select a LoROM window"
        );
        let rom = self.loaded_rom.as_ref().expect("ROM must be loaded");
        let base = (((address >> 16) << 15 | address & 0x7fff) & 0x3fffff) as usize;
        let start = base.checked_add(byte_offset).expect("ROM offset overflow");
        let end = start.checked_add(width).expect("ROM offset overflow");
        assert!(end <= rom.len, "ROM read outside loaded allocation");
        start
    }

    /// Current collision block index ($7E:0DC4 in ram_map).
    pub fn cur_block_index(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xdc4], self.g_ram[0xdc5]])
    }

    pub fn set_cur_block_index(&mut self, index: u16) {
        self.g_ram[0xdc4..0xdc6].copy_from_slice(&index.to_le_bytes());
    }

    /// BTS behavior byte for the live collision block ($7F:6402 in ram_map).
    pub fn current_block_bts(&self) -> u8 {
        self.g_ram[0x16402 + usize::from(self.cur_block_index())]
    }

    /// BTS saved by horizontal nonsquare slope dispatch ($7E:1E77 in ram_map).
    pub fn set_current_slope_bts(&mut self, bts: u16) {
        self.g_ram[0x1e77..0x1e79].copy_from_slice(&bts.to_le_bytes());
    }

    pub fn current_slope_bts(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x1e77], self.g_ram[0x1e78]])
    }

    /// Live level-data word selected by the collision block index.
    pub fn current_level_block(&self) -> u16 {
        let offset = 0x10002 + usize::from(self.cur_block_index()) * 2;
        u16::from_le_bytes([self.g_ram[offset], self.g_ram[offset + 1]])
    }

    /// Room width in blocks ($7E:07A5 in ram_map).
    pub fn room_width_in_blocks(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x7a5], self.g_ram[0x7a6]])
    }

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
        // composed only of those types, plus the explicitly initialized ROM
        // view option. Zero is a valid representation for
        // each; zeroed function options are still written explicitly below.
        // The entire allocation is initialized in place, avoiding a stack
        // temporary containing the 128 KiB RAM and three large snapshots.
        unsafe {
            core::ptr::write_bytes(game, 0, 1);
            core::ptr::addr_of_mut!((*game).loaded_rom).write(None);
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
