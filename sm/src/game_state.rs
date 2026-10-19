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

// Fixed word fields share one little-endian read/write definition. Indexed
// fields and operations on other widths remain explicit methods below.
macro_rules! ram_word_accessors {
    (read; $( $(#[$meta:meta])* $name:ident: $address:literal; )*) => {
        $(
            $(#[$meta])*
            pub fn $name(&self) -> u16 {
                u16::from_le_bytes([self.g_ram[$address], self.g_ram[$address + 1]])
            }
        )*
    };
    (write; $( $(#[$meta:meta])* $name:ident: $address:literal; )*) => {
        $(
            $(#[$meta])*
            pub fn $name(&mut self, value: u16) {
                self.g_ram[$address..$address + 2].copy_from_slice(&value.to_le_bytes());
            }
        )*
    };
}

// Initialize the large RAM and oracle snapshots directly on the heap.
impl Game {
    ram_word_accessors! {
        read;
        /// super_missiles count/capacity in the original RAM layout.
        samus_super_missiles: 0x9ca;
        samus_max_super_missiles: 0x9cc;
        /// power_bombs count/capacity in the original RAM layout.
        samus_power_bombs: 0x9ce;
        samus_max_power_bombs: 0x9d0;
        /// Health and reserve fields retain their original byte-backed RAM layout.
        samus_health: 0x9c2;
        /// Low-health sound latch ($7E:0A6A in ram_map).
        samus_health_warning: 0xa6a;
        /// Missile fields in ram_map at $7E:09C6/09C8/09D8.
        samus_missiles: 0x9c6;
        samus_max_missiles: 0x9c8;
        samus_reserve_missiles: 0x9d8;
        samus_max_health: 0x9c4;
        samus_reserve_health: 0x9d6;
        samus_max_reserve_health: 0x9d4;
        reserve_health_mode: 0x9c0;
        /// random_number ($7E:05E5 in ram_map).
        random_number: 0x5e5;
        /// special_death_item_drop_x_origin_pos ($7E:F434 in ram_map).
        special_death_item_drop_x_origin_pos: 0xf434;
        /// special_death_item_drop_y_origin_pos ($7E:F436 in ram_map).
        special_death_item_drop_y_origin_pos: 0xf436;
        /// timer_status ($7E:0943 in ram_map).
        timer_status: 0x943;
        /// door_transition_function ($7E:099C in ram_map).
        door_transition_function: 0x99c;
        /// hdma_objects_enable_flag ($7E:18B0 in ram_map).
        hdma_objects_enable_flag: 0x18b0;
        /// room_loading_irq_handler ($7E:00A9 in ram_map).
        room_loading_irq_handler: 0xa9;
        layer1_x_pos: 0x911;
        layer1_y_pos: 0x915;
        /// door_direction ($7E:0791 in ram_map).
        door_direction: 0x791;
        /// elevator_door_properties_orientation ($7E:0793 in ram_map).
        elevator_door_properties_orientation: 0x793;
        /// elevator_direction ($7E:0799 in ram_map).
        elevator_direction: 0x799;
        /// elevator_flags ($7E:0E16 in ram_map).
        elevator_flags: 0xe16;
        /// cre_bitset ($7E:07B3 in ram_map).
        cre_bitset: 0x7b3;
        /// door_def_ptr ($7E:078D in ram_map).
        door_def_ptr: 0x78d;
        /// room_ptr ($7E:079B in ram_map).
        room_ptr: 0x79b;
        /// room_height_in_blocks ($7E:07A7 in ram_map).
        room_height_in_blocks: 0x7a7;
        /// room_width_in_scrolls ($7E:07A9 in ram_map).
        room_width_in_scrolls: 0x7a9;
        /// room_height_in_scrolls ($7E:07AB in ram_map).
        room_height_in_scrolls: 0x7ab;
        /// roomdefroomstate_ptr ($7E:07BB in ram_map).
        roomdefroomstate_ptr: 0x7bb;
        /// reg_BG1VOFS ($7E:00B3 in ram_map).
        reg_bg1vofs: 0xb3;
        /// samus_x_pos ($7E:0AF6 in ram_map).
        samus_x_pos: 0xaf6;
        /// samus_x_subpos ($7E:0AF8 in ram_map).
        samus_x_subpos: 0xaf8;
        /// samus_y_pos ($7E:0AFA in ram_map).
        samus_y_pos: 0xafa;
        /// distance_to_eject_samus_up ($7E:0E08 in ram_map).
        distance_to_eject_samus_up: 0xe08;
        /// distance_to_eject_samus_down ($7E:0E0A in ram_map).
        distance_to_eject_samus_down: 0xe0a;
        /// samus_y_subpos ($7E:0AFC in ram_map).
        samus_y_subpos: 0xafc;
        /// samus_x_radius ($7E:0AFE in ram_map).
        samus_x_radius: 0xafe;
        /// extra_samus_y_displacement ($7E:0B5C in ram_map).
        extra_samus_y_displacement: 0xb5c;
        /// samus_y_radius ($7E:0B00 in ram_map).
        samus_y_radius: 0xb00;
        /// samus_pose ($7E:0A1C in ram_map).
        samus_pose: 0xa1c;
        /// samus_anim_frame ($7E:0A96 in ram_map).
        samus_anim_frame: 0xa96;
        /// joypad1_lastkeys ($7E:008B in ram_map).
        joypad1_lastkeys: 0x8b;
        /// samus_prev_x_pos ($7E:0B10 in ram_map).
        samus_prev_x_pos: 0xb10;
        /// samus_prev_y_pos ($7E:0B14 in ram_map).
        samus_prev_y_pos: 0xb14;
        debug_invincibility: 0xde0;
        /// game_state ($7E:0998 in ram_map).
        game_state: 0x998;
        /// debug_disable_sounds ($7E:05F5 in ram_map).
        debug_disable_sounds: 0x5f5;
        /// power_bomb_explosion_status ($7E:0592 in ram_map).
        power_bomb_explosion_status: 0x592;
        pausemenu_button_label_mode: 0x753;
        /// loading_game_state ($7E:D914 in ram_map).
        loading_game_state: 0xd914;
        /// area_index ($7E:079F in ram_map).
        area_index: 0x79f;
        collected_items: 0x9a4;
        /// demo_set ($7E:1F55 in ram_map).
        demo_set: 0x1f55;
        /// demo_scene ($7E:1F57 in ram_map).
        demo_scene: 0x1f57;
        /// demo_input_pre_instr ($7E:0A7A in ram_map).
        demo_input_pre_instr: 0xa7a;
        /// demo_input_instr_timer ($7E:0A7C in ram_map).
        demo_input_instr_timer: 0xa7c;
        /// demo_input_instr_ptr ($7E:0A7E in ram_map).
        demo_input_instr_ptr: 0xa7e;
        /// demo_input ($7E:0A84 in ram_map).
        demo_input: 0xa84;
        /// demo_input_new ($7E:0A86 in ram_map).
        demo_input_new: 0xa86;
        /// demo_enable ($7E:0A88 in ram_map), shared with hdma_ptr_1.
        demo_enable: 0xa88;
        /// demo_input_prev ($7E:0A8C in ram_map).
        demo_input_prev: 0xa8c;
        /// demo_input_prev_new ($7E:0A8E in ram_map), shared with hdma_ptr_3.
        demo_input_prev_new: 0xa8e;
        /// button_config_jump_a ($7E:09B4 in ram_map).
        button_config_jump_a: 0x9b4;
        /// joypad1_newkeys ($7E:008F in ram_map).
        joypad1_newkeys: 0x8f;
        /// collision_detection_index ($7E:18A6 in ram_map).
        collision_detection_index: 0x18a6;
        /// interactive_enemy_indexes_write_ptr ($7E:17A6 in ram_map).
        interactive_enemy_indexes_write_ptr: 0x17a6;
        /// samus_collision_direction ($7E:0B02 in ram_map).
        samus_collision_direction: 0xb02;
        /// samus_collision_flag ($7E:0DD0 in ram_map).
        samus_collision_flag: 0xdd0;
        /// flag_samus_in_quicksand ($7E:1E71 in ram_map).
        flag_samus_in_quicksand: 0x1e71;
        /// inside_block_reaction_samus_point ($7E:1E73 in ram_map).
        inside_block_reaction_samus_point: 0x1e73;
        /// Current collision block index ($7E:0DC4 in ram_map).
        cur_block_index: 0xdc4;
        current_slope_bts: 0x1e77;
        /// enemy_tile_vram_src ($7E:0E7C in ram_map).
        enemy_tile_vram_src: 0xe7c;
        /// enemy_tile_vram_dst ($7E:0E7E in ram_map).
        enemy_tile_vram_dst: 0xe7e;
        /// nmi_flag_bg2_enemy_vram_transfer ($7E:0E1E in ram_map).
        nmi_flag_bg2_enemy_vram_transfer: 0xe1e;
        /// enemy_bg2_tilemap_size ($7E:179A in ram_map).
        enemy_bg2_tilemap_size: 0x179a;
        /// debug_time_frozen_for_enemies ($7E:185E in ram_map).
        debug_time_frozen_for_enemies: 0x185e;
        /// time_is_frozen_flag ($7E:0A78 in ram_map).
        time_is_frozen_flag: 0xa78;
        /// VRAM write queue byte cursor ($7E:0330 in ram_map).
        vram_write_queue_tail: 0x330;
        /// Room width in blocks ($7E:07A5 in ram_map).
        room_width_in_blocks: 0x7a5;
        /// Current enemy slot offset ($7E:0E54 in ram_map).
        cur_enemy_index: 0xe54;
    }

    ram_word_accessors! {
        write;
        set_samus_super_missiles: 0x9ca;
        set_samus_power_bombs: 0x9ce;
        set_samus_health_warning: 0xa6a;
        set_samus_missiles: 0x9c6;
        set_samus_reserve_missiles: 0x9d8;
        set_samus_health: 0x9c2;
        set_samus_reserve_health: 0x9d6;
        set_reserve_health_mode: 0x9c0;
        set_door_transition_function: 0x99c;
        set_hdma_objects_enable_flag: 0x18b0;
        /// irqhandler_next_handler ($7E:00A7 in ram_map).
        set_irqhandler_next_handler: 0xa7;
        /// debug_disable_sounds ($7E:05F5 in ram_map).
        set_debug_disable_sounds: 0x5f5;
        /// door_transition_flag_elevator_zebetites ($7E:0795 in ram_map).
        set_door_transition_flag_elevator_zebetites: 0x795;
        /// door_transition_flag_enemies ($7E:0797 in ram_map).
        set_door_transition_flag_enemies: 0x797;
        /// game_state ($7E:0998 in ram_map).
        set_game_state: 0x998;
        /// layer1_x_pos ($7E:0911 in ram_map).
        set_layer1_x_pos: 0x911;
        /// layer1_y_pos ($7E:0915 in ram_map).
        set_layer1_y_pos: 0x915;
        set_door_direction: 0x791;
        set_elevator_door_properties_orientation: 0x793;
        set_elevator_flags: 0xe16;
        /// previous_cre_bitset ($7E:07B1 in ram_map).
        set_previous_cre_bitset: 0x7b1;
        set_cre_bitset: 0x7b3;
        set_room_ptr: 0x79b;
        /// room_index ($7E:079D in ram_map).
        set_room_index: 0x79d;
        /// area_index ($7E:079F in ram_map).
        set_area_index: 0x79f;
        /// room_x_coordinate_on_map ($7E:07A1 in ram_map).
        set_room_x_coordinate_on_map: 0x7a1;
        /// room_y_coordinate_on_map ($7E:07A3 in ram_map).
        set_room_y_coordinate_on_map: 0x7a3;
        set_room_height_in_blocks: 0x7a7;
        set_room_width_in_scrolls: 0x7a9;
        set_room_height_in_scrolls: 0x7ab;
        /// up_scroller ($7E:07AD in ram_map).
        set_up_scroller: 0x7ad;
        /// down_scroller ($7E:07AF in ram_map).
        set_down_scroller: 0x7af;
        /// door_list_pointer ($7E:07B5 in ram_map).
        set_door_list_pointer: 0x7b5;
        /// room_size_in_blocks ($7E:07B9 in ram_map).
        set_room_size_in_blocks: 0x7b9;
        /// room_music_track_index ($7E:07C9 in ram_map).
        set_room_music_track_index: 0x7c9;
        /// room_music_data_index ($7E:07CB in ram_map).
        set_room_music_data_index: 0x7cb;
        /// room_layer3_asm_ptr ($7E:07CD in ram_map).
        set_room_layer3_asm_ptr: 0x7cd;
        /// room_enemy_population_ptr ($7E:07CF in ram_map).
        set_room_enemy_population_ptr: 0x7cf;
        /// room_enemy_tilesets_ptr ($7E:07D1 in ram_map).
        set_room_enemy_tilesets_ptr: 0x7d1;
        /// room_main_code_ptr ($7E:07DF in ram_map).
        set_room_main_code_ptr: 0x7df;
        set_reg_bg1vofs: 0xb3;
        /// reg_BG2HOFS ($7E:00B5 in ram_map).
        set_reg_bg2hofs: 0xb5;
        /// reg_BG2VOFS ($7E:00B7 in ram_map).
        set_reg_bg2vofs: 0xb7;
        /// scrolling_finished_hook ($7E:07E9 in ram_map).
        set_scrolling_finished_hook: 0x7e9;
        /// door_transition_frame_counter ($7E:0925 in ram_map).
        set_door_transition_frame_counter: 0x925;
        /// door_destination_x_pos ($7E:0927 in ram_map).
        set_door_destination_x_pos: 0x927;
        /// door_destination_y_pos ($7E:0929 in ram_map).
        set_door_destination_y_pos: 0x929;
        /// samus_door_transition_subspeed ($7E:092B in ram_map).
        set_samus_door_transition_subspeed: 0x92b;
        /// samus_door_transition_speed ($7E:092D in ram_map).
        set_samus_door_transition_speed: 0x92d;
        set_samus_x_pos: 0xaf6;
        /// door_transition_flag ($7E:0931 in ram_map).
        set_door_transition_flag: 0x931;
        set_samus_x_subpos: 0xaf8;
        set_samus_y_pos: 0xafa;
        set_samus_y_subpos: 0xafc;
        set_extra_samus_y_displacement: 0xb5c;
        /// samus_pose ($7E:0A1C in ram_map).
        set_samus_pose: 0xa1c;
        /// samus_prev_pose ($7E:0A20 in ram_map).
        set_samus_prev_pose: 0xa20;
        /// samus_last_different_pose ($7E:0A24 in ram_map).
        set_samus_last_different_pose: 0xa24;
        /// samus_anim_frame_timer ($7E:0A94 in ram_map).
        set_samus_anim_frame_timer: 0xa94;
        set_samus_anim_frame: 0xa96;
        set_joypad1_lastkeys: 0x8b;
        /// input_to_pose_calc ($7E:0DC6 in ram_map).
        set_input_to_pose_calc: 0xdc6;
        set_samus_prev_x_pos: 0xb10;
        set_samus_prev_y_pos: 0xb14;
        /// samus_momentum_routine_index ($7E:0A2E in ram_map).
        set_samus_momentum_routine_index: 0xa2e;
        /// samus_special_transgfx_index ($7E:0A30 in ram_map).
        set_samus_special_transgfx_index: 0xa30;
        /// samus_hurt_switch_index ($7E:0A32 in ram_map).
        set_samus_hurt_switch_index: 0xa32;
        /// absolute_moved_last_frame_x ($7E:0DA2 in ram_map).
        set_absolute_moved_last_frame_x: 0xda2;
        /// absolute_moved_last_frame_x_fract ($7E:0DA4 in ram_map).
        set_absolute_moved_last_frame_x_fract: 0xda4;
        /// absolute_moved_last_frame_y ($7E:0DA6 in ram_map).
        set_absolute_moved_last_frame_y: 0xda6;
        /// absolute_moved_last_frame_y_fract ($7E:0DA8 in ram_map).
        set_absolute_moved_last_frame_y_fract: 0xda8;
        /// samus_y_subaccel ($7E:0B32 in ram_map).
        set_samus_y_subaccel: 0xb32;
        /// samus_y_accel ($7E:0B34 in ram_map).
        set_samus_y_accel: 0xb34;
        /// debug_invincibility ($7E:0DE0 in ram_map).
        set_debug_invincibility: 0xde0;
        /// unused_word_7e0a5e ($7E:0A5E in ram_map).
        set_unused_word_7e0a5e: 0xa5e;
        /// samus_prev_health_for_flash ($7E:0A12 in ram_map).
        set_samus_prev_health_for_flash: 0xa12;
        /// samus_visor_palette_timer_index ($7E:0A72 in ram_map).
        set_samus_visor_palette_timer_index: 0xa72;
        /// enable_horiz_slope_coll ($7E:0A46 in ram_map).
        set_enable_horiz_slope_coll: 0xa46;
        /// samus_hurt_flash_counter ($7E:0A48 in ram_map).
        set_samus_hurt_flash_counter: 0xa48;
        /// samus_special_super_palette_flags ($7E:0A4A in ram_map).
        set_samus_special_super_palette_flags: 0xa4a;
        /// fx_y_pos ($7E:195E in ram_map).
        set_fx_y_pos: 0x195e;
        /// lava_acid_y_pos ($7E:1962 in ram_map).
        set_lava_acid_y_pos: 0x1962;
        /// cinematic_function ($7E:1F51 in ram_map).
        set_cinematic_function: 0x1f51;
        /// hud_item_index ($7E:09D2 in ram_map).
        set_hud_item_index: 0x9d2;
        /// samus_auto_cancel_hud_item_index ($7E:0A04 in ram_map).
        set_samus_auto_cancel_hud_item_index: 0xa04;
        /// samus_invincibility_timer ($7E:18A8 in ram_map).
        set_samus_invincibility_timer: 0x18a8;
        /// samus_knockback_timer ($7E:18AA in ram_map).
        set_samus_knockback_timer: 0x18aa;
        /// has_area_map ($7E:0789 in ram_map).
        set_has_area_map: 0x789;
        /// collected_items ($7E:09A4 in ram_map).
        set_collected_items: 0x9a4;
        /// equipped_items ($7E:09A2 in ram_map).
        set_equipped_items: 0x9a2;
        /// samus_max_missiles ($7E:09C8 in ram_map).
        set_samus_max_missiles: 0x9c8;
        /// samus_max_super_missiles ($7E:09CC in ram_map).
        set_samus_max_super_missiles: 0x9cc;
        /// samus_max_power_bombs ($7E:09D0 in ram_map).
        set_samus_max_power_bombs: 0x9d0;
        /// samus_max_health ($7E:09C4 in ram_map).
        set_samus_max_health: 0x9c4;
        /// collected_beams ($7E:09A8 in ram_map).
        set_collected_beams: 0x9a8;
        /// equipped_beams ($7E:09A6 in ram_map).
        set_equipped_beams: 0x9a6;
        /// button_config_up ($7E:09AA in ram_map).
        set_button_config_up: 0x9aa;
        /// button_config_down ($7E:09AC in ram_map).
        set_button_config_down: 0x9ac;
        /// button_config_left ($7E:09AE in ram_map).
        set_button_config_left: 0x9ae;
        /// button_config_right ($7E:09B0 in ram_map).
        set_button_config_right: 0x9b0;
        /// button_config_shoot_x ($7E:09B2 in ram_map).
        set_button_config_shoot_x: 0x9b2;
        /// button_config_jump_a ($7E:09B4 in ram_map).
        set_button_config_jump_a: 0x9b4;
        /// button_config_run_b ($7E:09B6 in ram_map).
        set_button_config_run_b: 0x9b6;
        /// button_config_itemcancel_y ($7E:09B8 in ram_map).
        set_button_config_itemcancel_y: 0x9b8;
        /// button_config_itemswitch ($7E:09BA in ram_map).
        set_button_config_itemswitch: 0x9ba;
        /// button_config_aim_up_r ($7E:09BE in ram_map).
        set_button_config_aim_up_r: 0x9be;
        /// button_config_aim_down_l ($7E:09BC in ram_map).
        set_button_config_aim_down_l: 0x9bc;
        /// unused_word_7e09e8 ($7E:09E8 in ram_map).
        set_unused_word_7e09e8: 0x9e8;
        /// debug_flag ($7E:09E6 in ram_map).
        set_debug_flag: 0x9e6;
        /// moonwalk_flag ($7E:09E4 in ram_map).
        set_moonwalk_flag: 0x9e4;
        /// unused_word_7e0df8 ($7E:0DF8 in ram_map).
        set_unused_word_7e0df8: 0xdf8;
        /// unused_word_7e0dfa ($7E:0DFA in ram_map).
        set_unused_word_7e0dfa: 0xdfa;
        /// unused_word_7e0dfc ($7E:0DFC in ram_map).
        set_unused_word_7e0dfc: 0xdfc;
        set_demo_input_pre_instr: 0xa7a;
        set_demo_input_instr_timer: 0xa7c;
        set_demo_input_instr_ptr: 0xa7e;
        /// demo_timer_counter ($7E:0A80 in ram_map).
        set_demo_timer_counter: 0xa80;
        /// xray_angle ($7E:0A82 in ram_map), also used by demo input objects.
        set_xray_angle: 0xa82;
        set_demo_input: 0xa84;
        set_demo_input_new: 0xa86;
        set_demo_enable: 0xa88;
        set_demo_input_prev: 0xa8c;
        set_demo_input_prev_new: 0xa8e;
        /// joypad1_input_samusfilter ($7E:0DFE in ram_map).
        set_joypad1_input_samusfilter: 0xdfe;
        /// joypad1_newinput_samusfilter ($7E:0E00 in ram_map).
        set_joypad1_newinput_samusfilter: 0xe00;
        /// frame_handler_alfa ($7E:0A42 in ram_map).
        set_frame_handler_alfa: 0xa42;
        /// frame_handler_beta ($7E:0A44 in ram_map).
        set_frame_handler_beta: 0xa44;
        /// frame_handler_gamma ($7E:0A5A in ram_map).
        set_frame_handler_gamma: 0xa5a;
        /// samus_movement_handler ($7E:0A58 in ram_map).
        set_samus_movement_handler: 0xa58;
        /// samus_draw_handler ($7E:0A5C in ram_map).
        set_samus_draw_handler: 0xa5c;
        /// samus_input_handler ($7E:0A60 in ram_map).
        set_samus_input_handler: 0xa60;
        /// grapple_beam_function ($7E:0D32 in ram_map).
        set_grapple_beam_function: 0xd32;
        /// samus_new_pose ($7E:0A28 in ram_map).
        set_samus_new_pose: 0xa28;
        /// samus_new_pose_interrupted ($7E:0A2A in ram_map).
        set_samus_new_pose_interrupted: 0xa2a;
        /// samus_new_pose_transitional ($7E:0A2C in ram_map).
        set_samus_new_pose_transitional: 0xa2c;
        set_joypad1_newkeys: 0x8f;
        set_collision_detection_index: 0x18a6;
        /// enemy_index_to_shake ($7E:0E1C in ram_map).
        set_enemy_index_to_shake: 0xe1c;
        set_samus_collision_direction: 0xb02;
        set_samus_collision_flag: 0xdd0;
        set_samus_pos_adjusted_by_slope_flag: 0xdba;
        set_flag_samus_in_quicksand: 0x1e71;
        set_inside_block_reaction_samus_point: 0x1e73;
        set_cur_block_index: 0xdc4;
        /// BTS saved by horizontal nonsquare slope dispatch ($7E:1E77 in ram_map).
        set_current_slope_bts: 0x1e77;
        set_enemy_tile_vram_src: 0xe7c;
        set_enemy_tile_vram_dst: 0xe7e;
        set_nmi_flag_bg2_enemy_vram_transfer: 0xe1e;
        set_vram_write_queue_tail: 0x330;
        set_room_width_in_blocks: 0x7a5;
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
        let base = crate::rtl::lorom_offset(address) as usize;
        let start = base.checked_add(byte_offset).expect("ROM offset overflow");
        let end = start.checked_add(width).expect("ROM offset overflow");
        assert!(end <= rom.len, "ROM read outside loaded allocation");
        start
    }

    /// sfx_readpos / sfx_writepos ($7E:0643/0646 in ram_map).
    pub fn sfx_queue_is_empty(&self, channel: usize) -> bool {
        self.g_ram[0x646 + channel].wrapping_sub(self.g_ram[0x643 + channel]) & 0xf == 0
    }

    /// coroutine_state_1 ($7E:0612 in ram_map).
    pub fn coroutine_state_1(&self) -> u8 {
        self.g_ram[0x612]
    }
    pub fn set_coroutine_state_1(&mut self, value: u8) {
        self.g_ram[0x612] = value;
    }

    fn long_pointer_bytes(pointer: LongPtr) -> [u8; 3] {
        let [low, high] = pointer.addr.to_le_bytes();
        [low, high, pointer.bank]
    }

    /// room_compr_level_data_ptr ($7E:07BD in ram_map).
    pub fn set_room_compr_level_data_ptr(&mut self, value: LongPtr) {
        self.g_ram[0x7bd..0x7c0].copy_from_slice(&Self::long_pointer_bytes(value));
    }

    /// tileset_tile_table_pointer ($7E:07C0 in ram_map).
    pub fn set_tileset_tile_table_pointer(&mut self, value: LongPtr) {
        self.g_ram[0x7c0..0x7c3].copy_from_slice(&Self::long_pointer_bytes(value));
    }

    /// Compressed tileset graphics address ($7E:07C3 in ram_map).
    pub fn tileset_tiles_pointer(&self) -> u32 {
        u32::from_le_bytes([self.g_ram[0x7c3], self.g_ram[0x7c4], self.g_ram[0x7c5], 0])
    }

    pub fn set_tileset_tiles_pointer(&mut self, value: LongPtr) {
        self.g_ram[0x7c3..0x7c6].copy_from_slice(&Self::long_pointer_bytes(value));
    }

    /// Compressed tileset palette address ($7E:07C6 in ram_map).
    pub fn tileset_compr_palette_ptr(&self) -> u32 {
        u32::from_le_bytes([self.g_ram[0x7c6], self.g_ram[0x7c7], self.g_ram[0x7c8], 0])
    }

    pub fn set_tileset_compr_palette_ptr(&mut self, value: LongPtr) {
        self.g_ram[0x7c6..0x7c9].copy_from_slice(&Self::long_pointer_bytes(value));
    }

    /// layer2_scroll_x/y ($7E:091B/091C in ram_map).
    pub fn set_layer2_scroll_x_and_y(&mut self, x: u8, y: u8) {
        self.g_ram[0x91b..0x91d].copy_from_slice(&[x, y]);
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

    /// samus_pose_x_dir ($7E:0A1E in ram_map); this is a byte, not a word.
    pub fn samus_pose_x_dir(&self) -> u8 {
        self.g_ram[0xa1e]
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

    /// BTS behavior byte for the live collision block ($7F:6402 in ram_map).
    pub fn current_block_bts(&self) -> u8 {
        self.g_ram[0x16402 + usize::from(self.cur_block_index())]
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

    pub fn write_vram_queue_byte(&mut self, cursor: u16, field: usize, value: u8) {
        self.g_ram[0xd0 + usize::from(cursor) + field] = value;
    }

    /// Write a word in the backing VRAM queue, including overlapping fields.
    pub fn write_vram_queue_word(&mut self, cursor: u16, field: usize, value: u16) {
        let offset = 0xd0 + usize::from(cursor) + field;
        self.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
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
