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
    /// super_missiles count/capacity in the original RAM layout.
    pub fn samus_super_missiles(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9ca], self.g_ram[0x9cb]])
    }

    pub fn set_samus_super_missiles(&mut self, value: u16) {
        self.g_ram[0x9ca..0x9cc].copy_from_slice(&value.to_le_bytes());
    }

    pub fn samus_max_super_missiles(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9cc], self.g_ram[0x9cd]])
    }

    /// power_bombs count/capacity in the original RAM layout.
    pub fn samus_power_bombs(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9ce], self.g_ram[0x9cf]])
    }

    pub fn set_samus_power_bombs(&mut self, value: u16) {
        self.g_ram[0x9ce..0x9d0].copy_from_slice(&value.to_le_bytes());
    }

    pub fn samus_max_power_bombs(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9d0], self.g_ram[0x9d1]])
    }

    /// Health and reserve fields retain their original byte-backed RAM layout.
    pub fn samus_health(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9c2], self.g_ram[0x9c3]])
    }

    /// Low-health sound latch ($7E:0A6A in ram_map).
    pub fn samus_health_warning(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xa6a], self.g_ram[0xa6b]])
    }

    pub fn set_samus_health_warning(&mut self, value: u16) {
        self.g_ram[0xa6a..0xa6c].copy_from_slice(&value.to_le_bytes());
    }

    /// Missile fields in ram_map at $7E:09C6/09C8/09D8.
    pub fn samus_missiles(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9c6], self.g_ram[0x9c7]])
    }

    pub fn set_samus_missiles(&mut self, value: u16) {
        self.g_ram[0x9c6..0x9c8].copy_from_slice(&value.to_le_bytes());
    }

    pub fn samus_max_missiles(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9c8], self.g_ram[0x9c9]])
    }

    pub fn samus_reserve_missiles(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9d8], self.g_ram[0x9d9]])
    }

    pub fn set_samus_reserve_missiles(&mut self, value: u16) {
        self.g_ram[0x9d8..0x9da].copy_from_slice(&value.to_le_bytes());
    }

    pub fn set_samus_health(&mut self, value: u16) {
        self.g_ram[0x9c2..0x9c4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn samus_max_health(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9c4], self.g_ram[0x9c5]])
    }

    pub fn samus_reserve_health(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9d6], self.g_ram[0x9d7]])
    }

    pub fn set_samus_reserve_health(&mut self, value: u16) {
        self.g_ram[0x9d6..0x9d8].copy_from_slice(&value.to_le_bytes());
    }

    pub fn samus_max_reserve_health(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9d4], self.g_ram[0x9d5]])
    }

    pub fn reserve_health_mode(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9c0], self.g_ram[0x9c1]])
    }

    pub fn set_reserve_health_mode(&mut self, value: u16) {
        self.g_ram[0x9c0..0x9c2].copy_from_slice(&value.to_le_bytes());
    }

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

    /// random_number ($7E:05E5 in ram_map).
    pub fn random_number(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x5e5], self.g_ram[0x5e6]])
    }

    /// special_death_item_drop_x_origin_pos ($7E:F434 in ram_map).
    pub fn special_death_item_drop_x_origin_pos(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xf434], self.g_ram[0xf435]])
    }

    /// special_death_item_drop_y_origin_pos ($7E:F436 in ram_map).
    pub fn special_death_item_drop_y_origin_pos(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xf436], self.g_ram[0xf437]])
    }

    /// layer1_x_pos ($7E:0911 in ram_map).
    pub fn layer1_x_pos(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x911], self.g_ram[0x912]])
    }

    /// layer1_y_pos ($7E:0915 in ram_map).
    pub fn layer1_y_pos(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x915], self.g_ram[0x916]])
    }

    /// samus_x_pos ($7E:0AF6 in ram_map).
    pub fn samus_x_pos(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xaf6], self.g_ram[0xaf7]])
    }

    /// samus_x_subpos ($7E:0AF8 in ram_map).
    pub fn samus_x_subpos(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xaf8], self.g_ram[0xaf9]])
    }

    pub fn set_samus_x_subpos(&mut self, value: u16) {
        self.g_ram[0xaf8..0xafa].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_y_pos ($7E:0AFA in ram_map).
    pub fn samus_y_pos(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xafa], self.g_ram[0xafb]])
    }

    pub fn set_samus_y_pos(&mut self, value: u16) {
        self.g_ram[0xafa..0xafc].copy_from_slice(&value.to_le_bytes());
    }

    /// distance_to_eject_samus_up ($7E:0E08 in ram_map).
    pub fn distance_to_eject_samus_up(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xe08], self.g_ram[0xe09]])
    }

    /// distance_to_eject_samus_down ($7E:0E0A in ram_map).
    pub fn distance_to_eject_samus_down(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xe0a], self.g_ram[0xe0b]])
    }

    /// samus_y_subpos ($7E:0AFC in ram_map).
    pub fn samus_y_subpos(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xafc], self.g_ram[0xafd]])
    }

    pub fn set_samus_y_subpos(&mut self, value: u16) {
        self.g_ram[0xafc..0xafe].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_x_radius ($7E:0AFE in ram_map).
    pub fn samus_x_radius(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xafe], self.g_ram[0xaff]])
    }

    /// extra_samus_y_displacement ($7E:0B5C in ram_map).
    pub fn extra_samus_y_displacement(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xb5c], self.g_ram[0xb5d]])
    }
    pub fn set_extra_samus_y_displacement(&mut self, value: u16) {
        self.g_ram[0xb5c..0xb5e].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_y_radius ($7E:0B00 in ram_map).
    pub fn samus_y_radius(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xb00], self.g_ram[0xb01]])
    }

    /// samus_pose ($7E:0A1C in ram_map).
    pub fn samus_pose(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xa1c], self.g_ram[0xa1d]])
    }

    /// samus_pose ($7E:0A1C in ram_map).
    pub fn set_samus_pose(&mut self, value: u16) {
        self.g_ram[0xa1c..0xa1e].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_prev_pose ($7E:0A20 in ram_map).
    pub fn set_samus_prev_pose(&mut self, value: u16) {
        self.g_ram[0xa20..0xa22].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_last_different_pose ($7E:0A24 in ram_map).
    pub fn set_samus_last_different_pose(&mut self, value: u16) {
        self.g_ram[0xa24..0xa26].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_pose_x_dir / samus_movement_type ($7E:0A1E/0A1F).
    /// Preserve the original paired-byte word store.
    pub fn set_samus_pose_direction_and_movement_type(&mut self, direction: u8, movement_type: u8) {
        self.g_ram[0xa1e..0xa20].copy_from_slice(&[direction, movement_type]);
    }

    /// samus_movement_type ($7E:0A1F in ram_map).
    pub fn samus_movement_type(&self) -> u8 {
        self.g_ram[0xa1f]
    }

    /// samus_prev_pose_x_dir / samus_prev_movement_type2 ($7E:0A22/0A23).
    /// Preserve the original paired-byte word store.
    pub fn set_samus_prev_pose_direction_and_movement_type(
        &mut self,
        direction: u8,
        movement_type: u8,
    ) {
        self.g_ram[0xa22..0xa24].copy_from_slice(&[direction, movement_type]);
    }

    /// samus_last_different_pose_x_dir / samus_last_different_pose_movement_type ($7E:0A26/0A27).
    /// Preserve the original paired-byte word store.
    pub fn set_samus_last_different_pose_direction_and_movement_type(
        &mut self,
        direction: u8,
        movement_type: u8,
    ) {
        self.g_ram[0xa26..0xa28].copy_from_slice(&[direction, movement_type]);
    }

    /// samus_last_different_pose_movement_type ($7E:0A27 in ram_map).
    pub fn samus_last_different_pose_movement_type(&self) -> u8 {
        self.g_ram[0xa27]
    }

    /// samus_anim_frame_timer ($7E:0A94 in ram_map).
    pub fn set_samus_anim_frame_timer(&mut self, value: u16) {
        self.g_ram[0xa94..0xa96].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_anim_frame ($7E:0A96 in ram_map).
    pub fn samus_anim_frame(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xa96], self.g_ram[0xa97]])
    }

    pub fn set_samus_anim_frame(&mut self, value: u16) {
        self.g_ram[0xa96..0xa98].copy_from_slice(&value.to_le_bytes());
    }

    /// joypad1_lastkeys ($7E:008B in ram_map).
    pub fn joypad1_lastkeys(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x8b], self.g_ram[0x8c]])
    }

    pub fn set_joypad1_lastkeys(&mut self, value: u16) {
        self.g_ram[0x8b..0x8d].copy_from_slice(&value.to_le_bytes());
    }

    /// input_to_pose_calc ($7E:0DC6 in ram_map).
    pub fn set_input_to_pose_calc(&mut self, value: u16) {
        self.g_ram[0xdc6..0xdc8].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_prev_x_pos ($7E:0B10 in ram_map).
    pub fn samus_prev_x_pos(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xb10], self.g_ram[0xb11]])
    }

    pub fn set_samus_prev_x_pos(&mut self, value: u16) {
        self.g_ram[0xb10..0xb12].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_prev_y_pos ($7E:0B14 in ram_map).
    pub fn samus_prev_y_pos(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xb14], self.g_ram[0xb15]])
    }

    pub fn set_samus_prev_y_pos(&mut self, value: u16) {
        self.g_ram[0xb14..0xb16].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_momentum_routine_index ($7E:0A2E in ram_map).
    pub fn set_samus_momentum_routine_index(&mut self, value: u16) {
        self.g_ram[0xa2e..0xa30].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_special_transgfx_index ($7E:0A30 in ram_map).
    pub fn set_samus_special_transgfx_index(&mut self, value: u16) {
        self.g_ram[0xa30..0xa32].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_hurt_switch_index ($7E:0A32 in ram_map).
    pub fn set_samus_hurt_switch_index(&mut self, value: u16) {
        self.g_ram[0xa32..0xa34].copy_from_slice(&value.to_le_bytes());
    }

    /// absolute_moved_last_frame_x ($7E:0DA2 in ram_map).
    pub fn set_absolute_moved_last_frame_x(&mut self, value: u16) {
        self.g_ram[0xda2..0xda4].copy_from_slice(&value.to_le_bytes());
    }

    /// absolute_moved_last_frame_x_fract ($7E:0DA4 in ram_map).
    pub fn set_absolute_moved_last_frame_x_fract(&mut self, value: u16) {
        self.g_ram[0xda4..0xda6].copy_from_slice(&value.to_le_bytes());
    }

    /// absolute_moved_last_frame_y ($7E:0DA6 in ram_map).
    pub fn set_absolute_moved_last_frame_y(&mut self, value: u16) {
        self.g_ram[0xda6..0xda8].copy_from_slice(&value.to_le_bytes());
    }

    /// absolute_moved_last_frame_y_fract ($7E:0DA8 in ram_map).
    pub fn set_absolute_moved_last_frame_y_fract(&mut self, value: u16) {
        self.g_ram[0xda8..0xdaa].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_y_subaccel ($7E:0B32 in ram_map).
    pub fn set_samus_y_subaccel(&mut self, value: u16) {
        self.g_ram[0xb32..0xb34].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_y_accel ($7E:0B34 in ram_map).
    pub fn set_samus_y_accel(&mut self, value: u16) {
        self.g_ram[0xb34..0xb36].copy_from_slice(&value.to_le_bytes());
    }

    /// debug_invincibility ($7E:0DE0 in ram_map).
    pub fn set_debug_invincibility(&mut self, value: u16) {
        self.g_ram[0xde0..0xde2].copy_from_slice(&value.to_le_bytes());
    }

    pub fn debug_invincibility(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xde0], self.g_ram[0xde1]])
    }

    /// unused_word_7e0a5e ($7E:0A5E in ram_map).
    pub fn set_unused_word_7e0a5e(&mut self, value: u16) {
        self.g_ram[0xa5e..0xa60].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_prev_health_for_flash ($7E:0A12 in ram_map).
    pub fn set_samus_prev_health_for_flash(&mut self, value: u16) {
        self.g_ram[0xa12..0xa14].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_visor_palette_timer_index ($7E:0A72 in ram_map).
    pub fn set_samus_visor_palette_timer_index(&mut self, value: u16) {
        self.g_ram[0xa72..0xa74].copy_from_slice(&value.to_le_bytes());
    }

    /// enable_horiz_slope_coll ($7E:0A46 in ram_map).
    pub fn set_enable_horiz_slope_coll(&mut self, value: u16) {
        self.g_ram[0xa46..0xa48].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_hurt_flash_counter ($7E:0A48 in ram_map).
    pub fn set_samus_hurt_flash_counter(&mut self, value: u16) {
        self.g_ram[0xa48..0xa4a].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_special_super_palette_flags ($7E:0A4A in ram_map).
    pub fn set_samus_special_super_palette_flags(&mut self, value: u16) {
        self.g_ram[0xa4a..0xa4c].copy_from_slice(&value.to_le_bytes());
    }

    /// fx_y_pos ($7E:195E in ram_map).
    pub fn set_fx_y_pos(&mut self, value: u16) {
        self.g_ram[0x195e..0x1960].copy_from_slice(&value.to_le_bytes());
    }

    /// lava_acid_y_pos ($7E:1962 in ram_map).
    pub fn set_lava_acid_y_pos(&mut self, value: u16) {
        self.g_ram[0x1962..0x1964].copy_from_slice(&value.to_le_bytes());
    }

    /// cinematic_function ($7E:1F51 in ram_map).
    pub fn set_cinematic_function(&mut self, value: u16) {
        self.g_ram[0x1f51..0x1f53].copy_from_slice(&value.to_le_bytes());
    }

    /// hud_item_index ($7E:09D2 in ram_map).
    pub fn set_hud_item_index(&mut self, value: u16) {
        self.g_ram[0x9d2..0x9d4].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_auto_cancel_hud_item_index ($7E:0A04 in ram_map).
    pub fn set_samus_auto_cancel_hud_item_index(&mut self, value: u16) {
        self.g_ram[0xa04..0xa06].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_invincibility_timer ($7E:18A8 in ram_map).
    pub fn set_samus_invincibility_timer(&mut self, value: u16) {
        self.g_ram[0x18a8..0x18aa].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_knockback_timer ($7E:18AA in ram_map).
    pub fn set_samus_knockback_timer(&mut self, value: u16) {
        self.g_ram[0x18aa..0x18ac].copy_from_slice(&value.to_le_bytes());
    }

    /// game_state ($7E:0998 in ram_map).
    pub fn game_state(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x998], self.g_ram[0x999]])
    }

    /// loading_game_state ($7E:D914 in ram_map).
    pub fn loading_game_state(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xd914], self.g_ram[0xd915]])
    }

    /// area_index ($7E:079F in ram_map).
    pub fn area_index(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x79f], self.g_ram[0x7a0]])
    }

    /// has_area_map ($7E:0789 in ram_map).
    pub fn set_has_area_map(&mut self, value: u16) {
        self.g_ram[0x789..0x78b].copy_from_slice(&value.to_le_bytes());
    }

    /// collected_items ($7E:09A4 in ram_map).
    pub fn set_collected_items(&mut self, value: u16) {
        self.g_ram[0x9a4..0x9a6].copy_from_slice(&value.to_le_bytes());
    }

    /// equipped_items ($7E:09A2 in ram_map).
    pub fn set_equipped_items(&mut self, value: u16) {
        self.g_ram[0x9a2..0x9a4].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_max_missiles ($7E:09C8 in ram_map).
    pub fn set_samus_max_missiles(&mut self, value: u16) {
        self.g_ram[0x9c8..0x9ca].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_max_super_missiles ($7E:09CC in ram_map).
    pub fn set_samus_max_super_missiles(&mut self, value: u16) {
        self.g_ram[0x9cc..0x9ce].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_max_power_bombs ($7E:09D0 in ram_map).
    pub fn set_samus_max_power_bombs(&mut self, value: u16) {
        self.g_ram[0x9d0..0x9d2].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_max_health ($7E:09C4 in ram_map).
    pub fn set_samus_max_health(&mut self, value: u16) {
        self.g_ram[0x9c4..0x9c6].copy_from_slice(&value.to_le_bytes());
    }

    /// collected_beams ($7E:09A8 in ram_map).
    pub fn set_collected_beams(&mut self, value: u16) {
        self.g_ram[0x9a8..0x9aa].copy_from_slice(&value.to_le_bytes());
    }

    /// equipped_beams ($7E:09A6 in ram_map).
    pub fn set_equipped_beams(&mut self, value: u16) {
        self.g_ram[0x9a6..0x9a8].copy_from_slice(&value.to_le_bytes());
    }

    pub fn collected_items(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9a4], self.g_ram[0x9a5]])
    }

    /// button_config_up ($7E:09AA in ram_map).
    pub fn set_button_config_up(&mut self, value: u16) {
        self.g_ram[0x9aa..0x9ac].copy_from_slice(&value.to_le_bytes());
    }

    /// button_config_down ($7E:09AC in ram_map).
    pub fn set_button_config_down(&mut self, value: u16) {
        self.g_ram[0x9ac..0x9ae].copy_from_slice(&value.to_le_bytes());
    }

    /// button_config_left ($7E:09AE in ram_map).
    pub fn set_button_config_left(&mut self, value: u16) {
        self.g_ram[0x9ae..0x9b0].copy_from_slice(&value.to_le_bytes());
    }

    /// button_config_right ($7E:09B0 in ram_map).
    pub fn set_button_config_right(&mut self, value: u16) {
        self.g_ram[0x9b0..0x9b2].copy_from_slice(&value.to_le_bytes());
    }

    /// button_config_shoot_x ($7E:09B2 in ram_map).
    pub fn set_button_config_shoot_x(&mut self, value: u16) {
        self.g_ram[0x9b2..0x9b4].copy_from_slice(&value.to_le_bytes());
    }

    /// button_config_jump_a ($7E:09B4 in ram_map).
    pub fn set_button_config_jump_a(&mut self, value: u16) {
        self.g_ram[0x9b4..0x9b6].copy_from_slice(&value.to_le_bytes());
    }

    /// button_config_run_b ($7E:09B6 in ram_map).
    pub fn set_button_config_run_b(&mut self, value: u16) {
        self.g_ram[0x9b6..0x9b8].copy_from_slice(&value.to_le_bytes());
    }

    /// button_config_itemcancel_y ($7E:09B8 in ram_map).
    pub fn set_button_config_itemcancel_y(&mut self, value: u16) {
        self.g_ram[0x9b8..0x9ba].copy_from_slice(&value.to_le_bytes());
    }

    /// button_config_itemswitch ($7E:09BA in ram_map).
    pub fn set_button_config_itemswitch(&mut self, value: u16) {
        self.g_ram[0x9ba..0x9bc].copy_from_slice(&value.to_le_bytes());
    }

    /// button_config_aim_up_r ($7E:09BE in ram_map).
    pub fn set_button_config_aim_up_r(&mut self, value: u16) {
        self.g_ram[0x9be..0x9c0].copy_from_slice(&value.to_le_bytes());
    }

    /// button_config_aim_down_l ($7E:09BC in ram_map).
    pub fn set_button_config_aim_down_l(&mut self, value: u16) {
        self.g_ram[0x9bc..0x9be].copy_from_slice(&value.to_le_bytes());
    }

    /// unused_word_7e09e8 ($7E:09E8 in ram_map).
    pub fn set_unused_word_7e09e8(&mut self, value: u16) {
        self.g_ram[0x9e8..0x9ea].copy_from_slice(&value.to_le_bytes());
    }

    /// debug_flag ($7E:09E6 in ram_map).
    pub fn set_debug_flag(&mut self, value: u16) {
        self.g_ram[0x9e6..0x9e8].copy_from_slice(&value.to_le_bytes());
    }

    /// moonwalk_flag ($7E:09E4 in ram_map).
    pub fn set_moonwalk_flag(&mut self, value: u16) {
        self.g_ram[0x9e4..0x9e6].copy_from_slice(&value.to_le_bytes());
    }

    /// unused_word_7e0df8 ($7E:0DF8 in ram_map).
    pub fn set_unused_word_7e0df8(&mut self, value: u16) {
        self.g_ram[0xdf8..0xdfa].copy_from_slice(&value.to_le_bytes());
    }

    /// unused_word_7e0dfa ($7E:0DFA in ram_map).
    pub fn set_unused_word_7e0dfa(&mut self, value: u16) {
        self.g_ram[0xdfa..0xdfc].copy_from_slice(&value.to_le_bytes());
    }

    /// unused_word_7e0dfc ($7E:0DFC in ram_map).
    pub fn set_unused_word_7e0dfc(&mut self, value: u16) {
        self.g_ram[0xdfc..0xdfe].copy_from_slice(&value.to_le_bytes());
    }

    /// demo_set ($7E:1F55 in ram_map).
    pub fn demo_set(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x1f55], self.g_ram[0x1f56]])
    }

    /// demo_scene ($7E:1F57 in ram_map).
    pub fn demo_scene(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x1f57], self.g_ram[0x1f58]])
    }

    /// demo_input_pre_instr ($7E:0A7A in ram_map).
    pub fn demo_input_pre_instr(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xa7a], self.g_ram[0xa7b]])
    }

    pub fn set_demo_input_pre_instr(&mut self, value: u16) {
        self.g_ram[0xa7a..0xa7c].copy_from_slice(&value.to_le_bytes());
    }

    /// demo_input_instr_timer ($7E:0A7C in ram_map).
    pub fn demo_input_instr_timer(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xa7c], self.g_ram[0xa7d]])
    }

    pub fn set_demo_input_instr_timer(&mut self, value: u16) {
        self.g_ram[0xa7c..0xa7e].copy_from_slice(&value.to_le_bytes());
    }

    /// demo_input_instr_ptr ($7E:0A7E in ram_map).
    pub fn demo_input_instr_ptr(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xa7e], self.g_ram[0xa7f]])
    }

    pub fn set_demo_input_instr_ptr(&mut self, value: u16) {
        self.g_ram[0xa7e..0xa80].copy_from_slice(&value.to_le_bytes());
    }

    /// demo_timer_counter ($7E:0A80 in ram_map).
    pub fn set_demo_timer_counter(&mut self, value: u16) {
        self.g_ram[0xa80..0xa82].copy_from_slice(&value.to_le_bytes());
    }

    /// xray_angle ($7E:0A82 in ram_map), also used by demo input objects.
    pub fn set_xray_angle(&mut self, value: u16) {
        self.g_ram[0xa82..0xa84].copy_from_slice(&value.to_le_bytes());
    }

    /// demo_input ($7E:0A84 in ram_map).
    pub fn demo_input(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xa84], self.g_ram[0xa85]])
    }

    pub fn set_demo_input(&mut self, value: u16) {
        self.g_ram[0xa84..0xa86].copy_from_slice(&value.to_le_bytes());
    }

    /// demo_input_new ($7E:0A86 in ram_map).
    pub fn demo_input_new(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xa86], self.g_ram[0xa87]])
    }

    pub fn set_demo_input_new(&mut self, value: u16) {
        self.g_ram[0xa86..0xa88].copy_from_slice(&value.to_le_bytes());
    }

    /// demo_enable ($7E:0A88 in ram_map), shared with hdma_ptr_1.
    pub fn demo_enable(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xa88], self.g_ram[0xa89]])
    }

    pub fn set_demo_enable(&mut self, value: u16) {
        self.g_ram[0xa88..0xa8a].copy_from_slice(&value.to_le_bytes());
    }

    /// demo_input_prev ($7E:0A8C in ram_map).
    pub fn demo_input_prev(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xa8c], self.g_ram[0xa8d]])
    }

    pub fn set_demo_input_prev(&mut self, value: u16) {
        self.g_ram[0xa8c..0xa8e].copy_from_slice(&value.to_le_bytes());
    }

    /// demo_input_prev_new ($7E:0A8E in ram_map), shared with hdma_ptr_3.
    pub fn demo_input_prev_new(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xa8e], self.g_ram[0xa8f]])
    }

    pub fn set_demo_input_prev_new(&mut self, value: u16) {
        self.g_ram[0xa8e..0xa90].copy_from_slice(&value.to_le_bytes());
    }

    /// joypad1_input_samusfilter ($7E:0DFE in ram_map).
    pub fn set_joypad1_input_samusfilter(&mut self, value: u16) {
        self.g_ram[0xdfe..0xe00].copy_from_slice(&value.to_le_bytes());
    }

    /// joypad1_newinput_samusfilter ($7E:0E00 in ram_map).
    pub fn set_joypad1_newinput_samusfilter(&mut self, value: u16) {
        self.g_ram[0xe00..0xe02].copy_from_slice(&value.to_le_bytes());
    }

    /// frame_handler_alfa ($7E:0A42 in ram_map).
    pub fn set_frame_handler_alfa(&mut self, value: u16) {
        self.g_ram[0xa42..0xa44].copy_from_slice(&value.to_le_bytes());
    }

    /// frame_handler_beta ($7E:0A44 in ram_map).
    pub fn set_frame_handler_beta(&mut self, value: u16) {
        self.g_ram[0xa44..0xa46].copy_from_slice(&value.to_le_bytes());
    }

    /// frame_handler_gamma ($7E:0A5A in ram_map).
    pub fn set_frame_handler_gamma(&mut self, value: u16) {
        self.g_ram[0xa5a..0xa5c].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_movement_handler ($7E:0A58 in ram_map).
    pub fn set_samus_movement_handler(&mut self, value: u16) {
        self.g_ram[0xa58..0xa5a].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_draw_handler ($7E:0A5C in ram_map).
    pub fn set_samus_draw_handler(&mut self, value: u16) {
        self.g_ram[0xa5c..0xa5e].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_input_handler ($7E:0A60 in ram_map).
    pub fn set_samus_input_handler(&mut self, value: u16) {
        self.g_ram[0xa60..0xa62].copy_from_slice(&value.to_le_bytes());
    }

    /// grapple_beam_function ($7E:0D32 in ram_map).
    pub fn set_grapple_beam_function(&mut self, value: u16) {
        self.g_ram[0xd32..0xd34].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_new_pose ($7E:0A28 in ram_map).
    pub fn set_samus_new_pose(&mut self, value: u16) {
        self.g_ram[0xa28..0xa2a].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_new_pose_interrupted ($7E:0A2A in ram_map).
    pub fn set_samus_new_pose_interrupted(&mut self, value: u16) {
        self.g_ram[0xa2a..0xa2c].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_new_pose_transitional ($7E:0A2C in ram_map).
    pub fn set_samus_new_pose_transitional(&mut self, value: u16) {
        self.g_ram[0xa2c..0xa2e].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_pose_x_dir ($7E:0A1E in ram_map); this is a byte, not a word.
    pub fn samus_pose_x_dir(&self) -> u8 {
        self.g_ram[0xa1e]
    }

    /// button_config_jump_a ($7E:09B4 in ram_map).
    pub fn button_config_jump_a(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x9b4], self.g_ram[0x9b5]])
    }

    /// joypad1_newkeys ($7E:008F in ram_map).
    pub fn joypad1_newkeys(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x8f], self.g_ram[0x90]])
    }

    pub fn set_joypad1_newkeys(&mut self, value: u16) {
        self.g_ram[0x8f..0x91].copy_from_slice(&value.to_le_bytes());
    }

    /// collision_detection_index ($7E:18A6 in ram_map).
    pub fn collision_detection_index(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x18a6], self.g_ram[0x18a7]])
    }

    pub fn set_collision_detection_index(&mut self, value: u16) {
        self.g_ram[0x18a6..0x18a8].copy_from_slice(&value.to_le_bytes());
    }

    /// interactive_enemy_indexes_write_ptr ($7E:17A6 in ram_map).
    pub fn interactive_enemy_indexes_write_ptr(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x17a6], self.g_ram[0x17a7]])
    }

    /// Read a word of interactive_enemy_indexes ($7E:17EC in ram_map).
    /// Sentinel traversal uses backing RAM, including words beyond the list.
    pub fn interactive_enemy_index(&self, index: usize) -> u16 {
        let offset = 0x17ec + index * 2;
        u16::from_le_bytes([self.g_ram[offset], self.g_ram[offset + 1]])
    }

    /// enemy_index_colliding_dirs ($7E:182C in ram_map).
    pub fn set_enemy_index_colliding_dir(&mut self, direction: u16, index: u16) {
        let offset = 0x182c + usize::from(direction & 3) * 2;
        self.g_ram[offset..offset + 2].copy_from_slice(&index.to_le_bytes());
    }

    /// enemy_index_to_shake ($7E:0E1C in ram_map).
    pub fn set_enemy_index_to_shake(&mut self, value: u16) {
        self.g_ram[0xe1c..0xe1e].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_collision_direction ($7E:0B02 in ram_map).
    pub fn samus_collision_direction(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xb02], self.g_ram[0xb03]])
    }

    pub fn set_samus_collision_direction(&mut self, value: u16) {
        self.g_ram[0xb02..0xb04].copy_from_slice(&value.to_le_bytes());
    }

    /// samus_collision_flag ($7E:0DD0 in ram_map).
    pub fn samus_collision_flag(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xdd0], self.g_ram[0xdd1]])
    }

    pub fn set_samus_collision_flag(&mut self, value: u16) {
        self.g_ram[0xdd0..0xdd2].copy_from_slice(&value.to_le_bytes());
    }

    pub fn set_samus_pos_adjusted_by_slope_flag(&mut self, value: u16) {
        self.g_ram[0xdba..0xdbc].copy_from_slice(&value.to_le_bytes());
    }

    /// flag_samus_in_quicksand ($7E:1E71 in ram_map).
    pub fn flag_samus_in_quicksand(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x1e71], self.g_ram[0x1e72]])
    }

    pub fn set_flag_samus_in_quicksand(&mut self, value: u16) {
        self.g_ram[0x1e71..0x1e73].copy_from_slice(&value.to_le_bytes());
    }

    /// inside_block_reaction_samus_point ($7E:1E73 in ram_map).
    pub fn inside_block_reaction_samus_point(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x1e73], self.g_ram[0x1e74]])
    }

    pub fn set_inside_block_reaction_samus_point(&mut self, value: u16) {
        self.g_ram[0x1e73..0x1e75].copy_from_slice(&value.to_le_bytes());
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

    /// projectile_bomb_pre_instructions ($7E:0C68); slots 5..10 are bomb_functions.
    pub fn set_projectile_bomb_pre_instruction(&mut self, slot: usize, value: u16) {
        assert!(slot < 10);
        let offset = 0xc68 + slot * 2;
        self.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    /// Enemy projectile X position ($7E:1A4B in ram_map); odd indexes round down.
    pub fn enemy_projectile_x_pos(&self, index: u16) -> u16 {
        let offset = 0x1a4b + usize::from(index & !1);
        u16::from_le_bytes([self.g_ram[offset], self.g_ram[offset + 1]])
    }

    /// Enemy projectile Y position ($7E:1A93 in ram_map); odd indexes round down.
    pub fn enemy_projectile_y_pos(&self, index: u16) -> u16 {
        let offset = 0x1a93 + usize::from(index & !1);
        u16::from_le_bytes([self.g_ram[offset], self.g_ram[offset + 1]])
    }

    /// enemy_tile_vram_src ($7E:0E7C in ram_map).
    pub fn enemy_tile_vram_src(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xe7c], self.g_ram[0xe7d]])
    }

    pub fn set_enemy_tile_vram_src(&mut self, value: u16) {
        self.g_ram[0xe7c..0xe7e].copy_from_slice(&value.to_le_bytes());
    }

    /// enemy_tile_vram_dst ($7E:0E7E in ram_map).
    pub fn enemy_tile_vram_dst(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xe7e], self.g_ram[0xe7f]])
    }

    pub fn set_enemy_tile_vram_dst(&mut self, value: u16) {
        self.g_ram[0xe7e..0xe80].copy_from_slice(&value.to_le_bytes());
    }

    /// nmi_flag_bg2_enemy_vram_transfer ($7E:0E1E in ram_map).
    pub fn nmi_flag_bg2_enemy_vram_transfer(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xe1e], self.g_ram[0xe1f]])
    }

    pub fn set_nmi_flag_bg2_enemy_vram_transfer(&mut self, value: u16) {
        self.g_ram[0xe1e..0xe20].copy_from_slice(&value.to_le_bytes());
    }

    /// enemy_bg2_tilemap_size ($7E:179A in ram_map).
    pub fn enemy_bg2_tilemap_size(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x179a], self.g_ram[0x179b]])
    }

    /// debug_time_frozen_for_enemies ($7E:185E in ram_map).
    pub fn debug_time_frozen_for_enemies(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x185e], self.g_ram[0x185f]])
    }

    /// time_is_frozen_flag ($7E:0A78 in ram_map).
    pub fn time_is_frozen_flag(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0xa78], self.g_ram[0xa79]])
    }

    pub fn write_vram_queue_byte(&mut self, cursor: u16, field: usize, value: u8) {
        self.g_ram[0xd0 + usize::from(cursor) + field] = value;
    }

    /// VRAM write queue byte cursor ($7E:0330 in ram_map).
    pub fn vram_write_queue_tail(&self) -> u16 {
        u16::from_le_bytes([self.g_ram[0x330], self.g_ram[0x331]])
    }

    pub fn set_vram_write_queue_tail(&mut self, value: u16) {
        self.g_ram[0x330..0x332].copy_from_slice(&value.to_le_bytes());
    }

    /// Write a word in the backing VRAM queue, including overlapping fields.
    pub fn write_vram_queue_word(&mut self, cursor: u16, field: usize, value: u16) {
        let offset = 0xd0 + usize::from(cursor) + field;
        self.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
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
