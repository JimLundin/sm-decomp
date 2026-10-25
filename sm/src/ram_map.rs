//! Names for work RAM ($7E:0000-$7F:FFFF) offsets.
//!
//! Extracted from snesrev/sm `variables.h` and `variables_extra.h` (rev 578f90b).
//! c2rust expanded those macros, so the transpiled code reads e.g.
//! `*(g_ram.offset(0x998) as *mut uint16)` where the C said `game_state`.
//! The verifier uses this table to name RAM mismatches. Use these names when
//! replacing raw gameplay accesses (phase 3 of docs/PLAN.md).

/// `Scalar`: the name is one value at `addr`. `Array`: the name is a pointer
/// to the first element of a table of `ty` starting at `addr`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Scalar,
    Array,
}

#[derive(Clone, Copy, Debug)]
pub struct RamVar {
    pub addr: u32,
    pub name: &'static str,
    /// C type name as written in snesrev (`uint16`, `EnemyData`, ...).
    pub ty: &'static str,
    pub shape: Shape,
}

// One row records address, name, original type and scalar/array shape.
// Each group retains ordered names at base + index * stride.
macro_rules! ram_vars {
    ($($base:literal $(stride $stride:literal)?, $ty:ident $shape:ident => [$($name:tt),+ $(,)?];)*) => {
        /// Sorted by address. Several names can share an address; order is significant.
        pub const RAM_VARS: &[RamVar] = &{
            let mut rows = [RamVar { addr: 0, name: "", ty: "", shape: Shape::Scalar };
                0 $(+ [$(ram_vars!(@name $name)),*].len())*];
            let mut index = 0;
            $(
                let mut addr = $base;
                let stride = ram_vars!(@stride $($stride)?);
                $(
                    rows[index] = RamVar {
                        addr,
                        name: ram_vars!(@name $name),
                        ty: stringify!($ty),
                        shape: Shape::$shape,
                    };
                    index += 1;
                    addr += stride;
                )*
            )*
            rows
        };

        /// Named scalar word offsets share the verifier's address table.
        #[allow(non_upper_case_globals)]
        pub(crate) mod words {
            $(ram_vars!(@offsets word, $base, ram_vars!(@stride $($stride)?), $ty, $shape; $($name),*);)*
        }

        /// Named word-array bases; indices are signed element offsets.
        #[allow(non_upper_case_globals)]
        pub(crate) mod word_arrays {
            $(ram_vars!(@offsets array, $base, ram_vars!(@stride $($stride)?), $ty, $shape; $($name),*);)*
        }
    };
    (@stride) => { 0 };
    (@stride $stride:literal) => { $stride };
    (@name $name:literal) => { $name };
    (@name $name:ident) => { stringify!($name) };
    (@offsets $kind:ident, $addr:expr, $stride:expr, $ty:ident, $shape:ident;) => {};
    (@offsets $kind:ident, $addr:expr, $stride:expr, $ty:ident, $shape:ident; $name:tt $(, $next:tt)*) => {
        ram_vars!(@$kind $addr, $name, $ty, $shape);
        ram_vars!(@offsets $kind, ($addr) + ($stride), $stride, $ty, $shape; $($next),*);
    };
    (@word $addr:expr, $name:ident, uint16, Scalar) => {
        pub(crate) const $name: usize = $addr;
    };
    (@word $addr:expr, $name:ident, uint16, Array) => {};
    (@word $addr:expr, $name:literal, $ty:ident, $shape:ident) => {};
    (@array $addr:expr, $name:ident, uint16, Array) => {
        pub(crate) const $name: usize = $addr;
    };
    (@array $addr:expr, $name:tt, $ty:ident, $shape:ident) => {};
}

// Named SNES word reads, assignments and updates share one caller-side interface.
// `word!(game.field)` reads; `word!(game.field = value)` writes. Arrays use
// `field[index]`. Updates use `(game.field, method(argument), reread)` or an
// expression `(game.field, |old| value)`; `reread` is optional. Ordered stores
// of one literal use `(game, value; field, other[index])`; different values use
// `(game; field = value; other[index] = next;)`.
//
// Checked byte places stay at the caller: borrowing the whole Game/RAM here
// could invalidate retained raw enemy pointers. No reference escapes an access.
// Odd addresses and signed array offsets outside a named table remain valid.
// Reads evaluate the index once; assignments evaluate RHS before the
// destination, retaining RHS temporaries through the store. Updates capture
// the destination and read its old value before evaluating the RHS. `reread`
// loads that destination after RHS temporaries are dropped. No value or index
// is hoisted across ordered stores; update expressions create no closure.
macro_rules! word {
    // Compound arithmetic wraps at 16 bits; bits/shifts keep ordinary u16 operations.
    // Delegate destination capture, old load, RHS evaluation and store to the update arm.
    ($game:ident.$field:ident$([$index:expr])? &= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |old| old & $value $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])? += $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |old| old.wrapping_add($value) $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])? -= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |old| old.wrapping_sub($value) $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])? >>= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |old| old >> $value $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])? ^= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |old| old ^ $value $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])? |= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |old| old | $value $(, $result)?)
    };
    // Explicit promoted operations retain their original RHS type and final u16 cast.
    // Checked operations use ordinary Rust overflow/shift rules before truncation.
    (bits_i32, $game:ident.$field:ident$([$index:expr])? |= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |old| ((old as i32) | $value) as u16 $(, $result)?)
    };
    (checked_i32, $game:ident.$field:ident$([$index:expr])? *= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |old| ((old as i32) * $value) as u16 $(, $result)?)
    };
    (checked_i32, $game:ident.$field:ident$([$index:expr])? += $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |old| ((old as i32) + $value) as u16 $(, $result)?)
    };
    (checked_i32, $game:ident.$field:ident$([$index:expr])? -= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |old| ((old as i32) - $value) as u16 $(, $result)?)
    };
    (checked_i32, $game:ident.$field:ident$([$index:expr])? <<= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |old| ((old as i32) << $value) as u16 $(, $result)?)
    };
    (wrapping_u32, $game:ident.$field:ident$([$index:expr])? += $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |old| (old as u32).wrapping_add($value) as u16 $(, $result)?)
    };

    ($game:ident.$field:ident[$index:expr]) => {
        $crate::ram_map::word!(@read $game,
            $crate::ram_map::word!(@offset $field[$index]))
    };
    ($game:ident.$field:ident) => {
        $crate::ram_map::word!(@read $game, $crate::ram_map::words::$field)
    };
    ($game:ident.$field:ident[$index:expr] = $value:expr $(,)?) => {
        $crate::ram_map::word!(@write $game,
            $crate::ram_map::word!(@offset $field[$index]), $value)
    };
    ($game:ident.$field:ident = $value:expr $(,)?) => {
        $crate::ram_map::word!(@write $game, $crate::ram_map::words::$field, $value)
    };
    ($game:ident; $($field:ident$([$index:expr])? = $value:expr;)+) => {
        $($crate::ram_map::word!($game.$field$([$index])? = $value);)+
    };
    ($game:ident, $value:literal; $($field:ident$([$index:expr])?),+ $(,)?) => {
        $($crate::ram_map::word!($game.$field$([$index])? = $value);)+
    };
    ($game:ident.$field:ident$([$index:expr])?, $method:ident($argument:expr $(,)?) $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |word| word.$method($argument) $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])?, |$old:ident| { $value:expr } $(, $result:ident)? $(,)?) => {
        $crate::ram_map::word!($game.$field$([$index])?, |$old| $value $(, $result)?)
    };
    ($game:ident.$field:ident[$index:expr], |$old:ident| $value:expr $(, $result:ident)? $(,)?) => {{
        let destination = $crate::ram_map::word!(@offset $field[$index]);
        let $old = $crate::ram_map::word!(@read $game, destination);
        $crate::ram_map::word!(@write $game, destination, $value);
        $crate::ram_map::word!(@result $game, destination $(, $result)?)
    }};
    ($game:ident.$field:ident, |$old:ident| $value:expr $(, $result:ident)? $(,)?) => {{
        let destination = $crate::ram_map::words::$field;
        let $old = $crate::ram_map::word!(@read $game, destination);
        $crate::ram_map::word!(@write $game, destination, $value);
        $crate::ram_map::word!(@result $game, destination $(, $result)?)
    }};
    (@offset $field:ident[$index:expr]) => {
        $crate::ram_map::word_arrays::$field
            .checked_add_signed(($index as isize).checked_mul(2).expect("word index overflow"))
            .expect("word address outside RAM")
    };
    (@read $game:ident, $offset:expr) => {
        match $offset {
            offset => {
                // Check both bytes without borrowing any part of the RAM array.
                assert!(offset < 131071, "word address outside RAM");
                u16::from_le_bytes([$game.g_ram[offset], $game.g_ram[offset + 1]])
            }
        }
    };
    (@write $game:ident, $offset:expr, $value:expr) => {
        // Match keeps RHS/index temporaries alive through both stores, unlike
        // a let-value statement. The caller's semicolon drops them before reread.
        match ($value, $offset) {
            (value, offset) => {
                assert!(offset < 131071, "word address outside RAM");
                let bytes = u16::to_le_bytes(value);
                $game.g_ram[offset] = bytes[0];
                $game.g_ram[offset + 1] = bytes[1];
            }
        }
    };
    (@result $game:ident, $destination:ident) => { () };
    (@result $game:ident, $destination:ident, reread) => {
        $crate::ram_map::word!(@read $game, $destination)
    };
}
pub(crate) use word;

// Checked word access for migrated callers that retain offsets, not pointers.
// Byte places avoid borrowing the whole RAM array. Assignments evaluate the
// value before the address; updates capture the address and old value first.
// This smaller interface retains its existing low-byte-first failure prefix.
macro_rules! checked_word {
    ($game:ident.$field:ident[$index:expr] $($tail:tt)*) => {
        $crate::ram_map::checked_word!($game,
            $crate::ram_map::word_arrays::$field.checked_add_signed(($index as isize).checked_mul(2).unwrap()).unwrap(); $($tail)*)
    };
    ($game:ident.$field:ident $($tail:tt)*) => {
        $crate::ram_map::checked_word!($game, $crate::ram_map::words::$field; $($tail)*)
    };
    ($game:ident, $offset:expr;) => {{
        let offset: usize = $offset;
        u16::from_le_bytes([$game.g_ram[offset], $game.g_ram[offset + 1]])
    }};
    ($game:ident, $offset:expr; = $value:expr) => {
        match ($value, $offset) {
            (value, offset) => {
                let bytes = u16::to_le_bytes(value);
                $game.g_ram[offset] = bytes[0];
                $game.g_ram[offset + 1] = bytes[1];
            }
        }
    };
    ($game:ident, $offset:expr; += $value:expr) => {{
        let offset: usize = $offset;
        let old = $crate::ram_map::checked_word!($game, offset;);
        $crate::ram_map::checked_word!($game, offset; = old.wrapping_add($value));
    }};
    ($game:ident, $offset:expr; ^= $value:expr) => {{
        let offset: usize = $offset;
        let old = $crate::ram_map::checked_word!($game, offset;);
        $crate::ram_map::checked_word!($game, offset; = old ^ $value);
    }};
}
pub(crate) use checked_word;

ram_vars! {
    0x00012 stride 2, uint16 Scalar => [REMOVED_R18, REMOVED_R20, REMOVED_R22];
    0x00051, uint8 Scalar => ["reg_INIDISP"];
    0x00052, uint8 Scalar => ["reg_OBSEL"];
    0x00053, uint16 Scalar => [reg_OAMaddr_UNUSED];
    0x00055 stride 1, uint8 Scalar => [
        "reg_BGMODE", "reg_BGMODE_fake", "reg_MOSAIC", "reg_BG1SC", "reg_BG2SC", "reg_BG3SC",
        "gameplay_BG3SC", "reg_BG4SC", "reg_BG12NBA", "reg_BG34NBA", "reg_M7SEL", "reg_W12SEL",
        "reg_W34SEL", "reg_WOBJSEL", "reg_WH0", "reg_WH1", "reg_WH2", "reg_WH3", "reg_WBGLOG",
        "reg_WOBJLOG", "reg_TM", "gameplay_TM", "reg_TS", "reg_TMW", "reg_TSW",
        "next_gameplay_CGWSEL", "reg_CGWSEL", "gameplay_CGWSEL", "next_gameplay_CGADSUB",
        "reg_CGADSUB", "gameplay_CGADSUB"
    ];
    0x00074, uint8 Array => ["reg_COLDATA"];
    0x00077, uint8 Scalar => ["reg_SETINI"];
    0x00078 stride 2, uint16 Scalar => [reg_M7A, reg_M7B, reg_M7C, reg_M7D, reg_M7X, reg_M7Y];
    0x00084 stride 1, uint8 Scalar => ["reg_NMITIMEN", "reg_HDMAEN", "reg_MEMSEL"];
    0x00087 stride 2, uint16 Scalar => [
        joypad_ctr_repeat_first, joypad_ctr_repeat_next, joypad1_lastkeys, joypad2_last,
        joypad1_newkeys, joypad2_new_keys, joypad1_newkeys2_UNUSED, joypad2_newkeys2, joypad1_prev,
        joypad2_prev
    ];
    0x000A3 stride 2, uint16 Scalar => [
        joypad1_keyrepeat_ctr, joypad2_keyrepeat_ctr, irqhandler_next_handler,
        room_loading_irq_handler, cur_irq_handler, REMOVED_ptr_to_retaddr_parameters,
        current_region_animated_palette_fx, reg_BG1HOFS, reg_BG1VOFS, reg_BG2HOFS, reg_BG2VOFS,
        reg_BG3HOFS, reg_BG3VOFS, reg_BG4HOFS, reg_BG4VOFS
    ];
    0x000C2, uint16 Scalar => [bug_fix_counter];
    0x000C4, uint16 Array => ["g_word_7E00C4"];
    0x000D0, VramWriteEntry Array => ["vram_write_queue"];
    0x002D0, Mode7VramWriteQueue Array => ["mode7_write_queue"];
    0x00330, uint16 Scalar => [vram_write_queue_tail];
    0x00334, uint16 Scalar => [mode7_vram_write_queue_tail];
    0x00340, VramReadQueueEnt Array => ["vram_read_queue"];
    0x00360, uint16 Scalar => [vram_read_queue_tail];
    0x00370, OamEnt Array => ["oam_ent"];
    0x00570, uint16 Array => [oam_ext];
    0x00590, uint16 Scalar => [oam_next_ptr];
    0x00592, uint16 Scalar => [power_bomb_explosion_status];
    0x00596 stride 2, uint16 Scalar => [
        g_word_7E0596, g_word_7E0598, set_to_e0_by_scrolling_sky, UNUSED_word_7E059C
    ];
    0x005A0 stride 2, uint16 Scalar => [
        UNUSED_hdma_contracting_flag, message_box_animation_y_radius, message_box_animation_y0,
        message_box_animation_y1, message_box_animation_y2, message_box_animation_y3,
        map_min_x_scroll, map_max_x_scroll, map_min_y_scroll, map_max_y_scroll
    ];
    0x005B4, uint8 Scalar => ["waiting_for_nmi"];
    0x005B5, uint8 Scalar => ["nmi_frame_counter_byte"];
    0x005B6, uint16 Scalar => [nmi_frame_counter_word];
    0x005B8, uint16 Scalar => [nmi_frame_counter_including_lag];
    0x005BA, uint8 Scalar => ["nmi_frames_missed"];
    0x005BB, uint8 Scalar => ["nmi_frames_missed_max"];
    0x005C5 stride 2, uint16 Scalar => [
        joypad_dbg_1, joypad_dbg_2, joypad_dbg_missiles_swap, joypad_dbg_super_missiles_swap,
        joypad_dbg_power_bombs_swap, joypad_dbg_flags, enable_debug,
        debug_saveload_scrollpos_toggle, debug_saved_xscroll, debug_saved_yscroll,
        joypad_released_keys, timed_held_input_timer, timed_held_input_timer_reset,
        timed_held_input, newly_held_down_timed_held_input, previous_timed_held_input,
        random_number, bitmask, REMOVED_mult_tmp_a, REMOVED_mult_tmp_y
    ];
    0x005F1 stride 2, uint16 Scalar => [
        REMOVED_mult_product_lo, REMOVED_mult_product_hi, debug_disable_sounds,
        debug_disable_minimap, save_confirmation_selection, map_scrolling_gear_switch_timer,
        map_scrolling_direction, map_scrolling_speed_index
    ];
    0x00601, LongPtr Scalar => ["pause_hook"];
    0x00604, LongPtr Scalar => ["unpause_hook"];
    0x00607, uint16 Scalar => [earthquake_sfx_index];
    0x00609, uint16 Scalar => [earthquake_sfx_timer];
    0x0060B, uint8 Scalar => ["eproj_init_param_3"];
    0x0060D, uint16 Scalar => [REMOVED_remaining_enemy_hitbox_entries];
    0x0060F, uint16 Scalar => [REMOVED_num_projectiles_to_check_enemy_coll];
    0x00611 stride 1, uint8 Scalar => [
        "coroutine_state_0", "coroutine_state_1", "coroutine_state_2", "coroutine_state_3",
        "coroutine_state_4"
    ];
    0x00617, uint16 Scalar => [is_uploading_apu];
    0x00619, uint16 Array => [music_queue_track];
    0x00629, uint16 Array => [music_queue_delay];
    0x00639 stride 2, uint16 Scalar => [
        music_queue_write_pos, music_queue_read_pos, music_entry, music_timer,
        REMOVED_apu_attempts_countdown
    ];
    0x00643 stride 3, uint8 Array => ["sfx_readpos", "sfx_writepos", "sfx_state"];
    0x0064C, uint8 Scalar => ["cur_music_track"];
    0x0064D stride 3, uint8 Array => ["sfx_cur", "sfx_clear_delay", "sfx_max_queued", "sfx1_queue"];
    0x00666, uint8 Array => ["sfx2_queue"];
    0x00676, uint8 Array => ["sfx3_queue"];
    0x00686 stride 2, uint16 Scalar => [sound_handler_downtime, croco_target_0688, g_word_7E068A];
    0x0068C, uint8 Scalar => ["croco_word_068C"];
    0x0068E stride 2, uint16 Scalar => [
        g_word_7E068E, croco_cur_vline_idx, g_word_7E0692, g_word_7E0694, g_word_7E0696,
        g_word_7E0698, croco_word_7E069A
    ];
    0x0069C, uint8 Array => ["croco_vline_height"];
    0x0071C, uint8 Scalar => ["UNUSED_byte_7E071C"];
    0x0071D stride 2, uint16 Scalar => [
        nmi_copy_samus_halves, nmi_copy_samus_top_half_src, nmi_copy_samus_bottom_half_src,
        screen_fade_delay, screen_fade_counter, menu_index,
        pausemenu_start_lr_pressed_highlight_timer, pausemenu_lr_animation_timer,
        pausemenu_item_selector_animation_timer, pausemenu_reserve_tank_animation_timer,
        UNUSED_word_7E0731, pausemenu_uparrow_animation_timer, pausemenu_downarrow_animation_timer,
        pausemenu_rightarrow_animation_timer, pausemenu_leftarrow_animation_timer,
        pausemenu_palette_animation_timer, UNUSED_word_7E073D, pausemenu_lr_animation_frame,
        pausemenu_item_selector_animation_frame, pausemenu_reserve_tank_animation_frame,
        UNUSED_word_7E0745, pausemenu_uparrow_animation_frame, pausemenu_downarrow_animation_frame,
        pausemenu_rightarrow_animation_frame, pausemenu_leftarrow_animation_frame,
        pausemenu_palette_animation_frame, pausemenu_shoulder_button_highlight,
        pausemenu_button_label_mode, pausemenu_equipment_category_item,
        pausemenu_reserve_tank_delay_ctr, UNUSED_word_7E0759, UNUSED_word_7E075B,
        UNUSED_word_7E075D, UNUSED_word_7E075F, UNUSED_word_7E0761, pause_screen_mode
    ];
    0x00765 stride 1, uint8 Scalar => [
        "pausemenu_bak_BG1SC", "pausemenu_bak_BG2SC", "pausemenu_bak_BG3SC",
        "pausemenu_bak_BG12NBA", "pausemenu_bak_BG34NBA", "pausemenu_bak_OBSEL",
        "pausemenu_bak_BG1HOFS", "pausemenu_bak_BG2HOFS", "pausemenu_bak_BG3HOFS",
        "pausemenu_bak_BG1VOFS", "pausemenu_bak_BG2VOFS", "pausemenu_bak_BG3VOFS",
        "pausemenu_bak_BGMODE", "pausemenu_bak_layer2_scroll_x", "pausemenu_bak_layer2_scroll_y",
        "pausemenu_bak_MOSAIC", "pausemenu_bak_CGADSUB"
    ];
    0x00776 stride 2, uint16 Scalar => [
        samus_position_indicator_animation_frame, samus_position_indicator_animation_timer,
        samus_position_indicator_animation_loop_counter, hud_item_tilemap_palette_bits, my_counter,
        my_counter2
    ];
    0x00782, uint8 Scalar => ["coroutine_completion_flags"];
    0x00783 stride 2, uint16 Scalar => [
        irq_enable_mode7, mode7_rotation_angle, g_word_7E0787, has_area_map, load_station_index,
        door_def_ptr, REMOVED_door_bts, door_direction, elevator_door_properties_orientation,
        door_transition_flag_elevator_zebetites, door_transition_flag_enemies, elevator_direction,
        room_ptr, room_index, area_index, room_x_coordinate_on_map, room_y_coordinate_on_map,
        room_width_in_blocks, room_height_in_blocks, room_width_in_scrolls, room_height_in_scrolls,
        up_scroller, down_scroller, previous_cre_bitset, cre_bitset, door_list_pointer,
        REMOVED_event_pointer, room_size_in_blocks, roomdefroomstate_ptr
    ];
    0x007BD stride 3, LongPtr Scalar => [
        "room_compr_level_data_ptr", "tileset_tile_table_pointer", "tileset_tiles_pointer",
        "tileset_compr_palette_ptr"
    ];
    0x007C9 stride 2, uint16 Scalar => [
        room_music_track_index, room_music_data_index, room_layer3_asm_ptr,
        room_enemy_population_ptr, room_enemy_tilesets_ptr
    ];
    0x007DF, uint16 Scalar => [room_main_code_ptr];
    0x007E1, uint8 Array => ["room_main_asm_variables"];
    0x007E9, uint16 Scalar => [scrolling_finished_hook];
    0x007EB, uint16 Scalar => [hdma_data_table_in_ceres];
    0x007F3, uint16 Scalar => [music_data_index];
    0x007F5, uint16 Scalar => [music_track_index];
    0x007F7, uint8 Array => ["map_tiles_explored"];
    0x008F7 stride 2, uint16 Scalar => [
        layer1_x_block, layer1_y_block, layer2_x_block, layer2_y_block, previous_layer1_x_block,
        previous_layer1_y_block, previous_layer2_x_block, previous_layer2_y_block, bg1_x_block,
        bg1_y_block, bg2_x_block, bg2_y_block, layer1_x_subpos, layer1_x_pos, layer1_y_subpos,
        layer1_y_pos, layer2_x_pos, layer2_y_pos
    ];
    0x0091B, uint8 Scalar => ["layer2_scroll_x"];
    0x0091C, uint8 Scalar => ["layer2_scroll_y"];
    0x0091D stride 2, uint16 Scalar => [
        bg1_x_offset, bg1_y_offset, bg2_x_scroll, bg2_y_scroll, door_transition_frame_counter,
        door_destination_x_pos, door_destination_y_pos, samus_door_transition_subspeed,
        samus_door_transition_speed, downwards_elevator_delay_timer, door_transition_flag,
        REMOVED_var933, REMOVED_var935, REMOVED_var937, REMOVED_var939, REMOVED_var93B
    ];
    0x0093F stride 2, uint16 Scalar => [ceres_status, camera_distance_index, timer_status];
    0x00945 stride 1, uint8 Scalar => ["timer_centiseconds", "timer_seconds", "timer_minutes"];
    0x00948, uint16 Scalar => [timer_x_pos];
    0x0094A, uint16 Scalar => [timer_y_pos];
    0x00950 stride 2, uint16 Scalar => [
        file_select_map_area_index, selected_save_slot, nonempty_save_slots,
        bg1_update_col_unwrapped_size, bg1_update_col_wrapped_size, bg1_update_col_unwrapped_dst,
        bg1_update_col_wrapped_dst, bg1_update_col_wrapped_left_src,
        bg1_update_col_wrapped_right_src, bg1_update_col_enable, bg1_update_row_unwrapped_size,
        bg1_update_row_wrapped_size, bg1_update_row_unwrapped_dst, bg1_update_row_wrapped_dst,
        bg1_update_row_wrapped_top_src, bg1_update_row_wrapped_bottom_src, bg1_update_row_enable,
        bg2_update_col_unwrapped_size, bg2_update_col_wrapped_size, bg2_update_col_unwrapped_dst,
        bg2_update_col_wrapped_dst, bg2_update_col_wrapped_left_src,
        bg2_update_col_wrapped_right_src, bg2_update_col_enable, bg2_update_row_unwrapped_size,
        bg2_update_row_wrapped_size, bg2_update_row_unwrapped_dst, bg2_update_row_wrapped_dst,
        bg2_update_row_wrapped_top_src, bg2_update_row_wrapped_bottom_src, bg2_update_row_enable,
        size_of_bg2, blocks_to_update_x_block, blocks_to_update_y_block,
        vram_blocks_to_update_x_block, vram_blocks_to_update_y_block, game_state
    ];
    0x0099C, uint16 Scalar => [door_transition_function];
    0x0099E, uint16 Scalar => [menu_option_index];
    0x009A2 stride 2, uint16 Scalar => [
        equipped_items, collected_items, equipped_beams, collected_beams, button_config_up,
        button_config_down, button_config_left, button_config_right, button_config_shoot_x,
        button_config_jump_a, button_config_run_b, button_config_itemcancel_y,
        button_config_itemswitch, button_config_aim_down_L, button_config_aim_up_R,
        reserve_health_mode, samus_health, samus_max_health, samus_missiles, samus_max_missiles,
        samus_super_missiles, samus_max_super_missiles, samus_power_bombs, samus_max_power_bombs,
        hud_item_index, samus_max_reserve_health, samus_reserve_health, samus_reserve_missiles,
        game_time_frames, game_time_seconds, game_time_minutes, game_time_hours, japanese_text_flag,
        moonwalk_flag, debug_flag, UNUSED_word_7E09E8, hud_auto_cancel_flag
    ];
    0x00A04 stride 2, uint16 Scalar => [
        samus_auto_cancel_hud_item_index, samus_prev_health, samus_prev_missiles,
        samus_prev_super_missiles, samus_prev_power_bombs, samus_prev_hud_item_index
    ];
    0x00A11, uint8 Scalar => ["samus_prev_movement_type"];
    0x00A12 stride 2, uint16 Scalar => [
        samus_prev_health_for_flash, controller1_input_for_demo, controller1_new_input_for_demo,
        UNUSED_word_7E0A18
    ];
    0x00A1C, uint16 Scalar => [samus_pose];
    0x00A1E, uint8 Scalar => ["samus_pose_x_dir"];
    0x00A1F, uint8 Scalar => ["samus_movement_type"];
    0x00A20, uint16 Scalar => [samus_prev_pose];
    0x00A22, uint8 Scalar => ["samus_prev_pose_x_dir"];
    0x00A23, uint8 Scalar => ["samus_prev_movement_type2"];
    0x00A24, uint16 Scalar => [samus_last_different_pose];
    0x00A26, uint8 Scalar => ["samus_last_different_pose_x_dir"];
    0x00A27, uint8 Scalar => ["samus_last_different_pose_movement_type"];
    0x00A28 stride 2, uint16 Scalar => [
        samus_new_pose, samus_new_pose_interrupted, samus_new_pose_transitional,
        samus_momentum_routine_index, samus_special_transgfx_index, samus_hurt_switch_index,
        solid_enemy_collision_flags, block_collision_flags, samus_space_to_move_up_blocks,
        samus_y_radius_diff, samus_space_to_move_down_blocks, samus_space_to_move_up_enemy,
        samus_space_to_move_down_enemy, frame_handler_alfa, frame_handler_beta,
        enable_horiz_slope_coll, samus_hurt_flash_counter, samus_special_super_palette_flags,
        samus_subunit_health, samus_periodic_subdamage, samus_periodic_damage, knockback_dir,
        knockback_x_dir, bomb_jump_dir, samus_movement_handler, frame_handler_gamma,
        samus_draw_handler, UNUSED_word_7E0A5E, samus_input_handler, samus_var62,
        samus_grapple_flags, samus_x_speed_divisor, samus_shine_timer, samus_health_warning,
        samus_x_speed_table_pointer, samus_contact_damage_index
    ];
    0x00A72 stride 2, uint16 Scalar => [
        samus_visor_palette_timer_index, samus_suit_palette_index, hyper_beam_flag,
        time_is_frozen_flag, demo_input_pre_instr, demo_input_instr_timer, demo_input_instr_ptr,
        demo_timer_counter, xray_angle, demo_input, demo_input_new, demo_enable
    ];
    0x00A88, LongPtr Scalar => ["hdma_ptr_1"];
    0x00A8A, uint16 Scalar => [demo_num_input_frames];
    0x00A8B, LongPtr Scalar => ["hdma_ptr_2"];
    0x00A8C, uint16 Scalar => [demo_input_prev];
    0x00A8E, uint16 Scalar => [demo_input_prev_new];
    0x00A8E, LongPtr Scalar => ["hdma_ptr_3"];
    0x00A90 stride 1, uint16 Scalar => [
        demo_backup_prev_controller_input, hdma_var_1, demo_backup_prev_controller_input_new
    ];
    0x00A94, uint16 Scalar => [samus_anim_frame_timer];
    0x00A96, uint16 Scalar => [samus_anim_frame];
    0x00A9A stride 2, uint16 Scalar => [
        samus_anim_frame_skip, samus_anim_frame_buffer, grapple_walljump_timer,
        reached_ceres_elevator_fade_timer, timer_for_shinesparks_startstop, UNUSEDword_7E0AA4
    ];
    0x00AA6, uint8 Scalar => ["flag_arm_cannon_open_or_opening"];
    0x00AA7, uint8 Scalar => ["flag_arm_cannon_opening_or_closing"];
    0x00AA8 stride 2, uint16 Scalar => [
        arm_cannon_frame, hud_item_changed_this_frame, arm_cannon_drawing_mode, speed_echoes_index
    ];
    0x00AB0 stride 8, uint16 Array => [speed_echo_xpos, speed_echo_ypos, speed_echo_xspeed];
    0x00AC8 stride 2, uint16 Scalar => [
        samus_top_half_spritemap_index, samus_bottom_half_spritemap_index, timer_for_shine_timer,
        special_samus_palette_frame, special_samus_palette_timer, liquid_physics_type
    ];
    0x00AD4 stride 8, uint16 Array => [
        atmospheric_gfx_anim_timer, atmospheric_gfx_x_pos, atmospheric_gfx_y_pos,
        atmospheric_gfx_frame_and_type
    ];
    0x00AF4 stride 2, uint16 Scalar => [
        autojump_timer, samus_x_pos, samus_x_subpos, samus_y_pos, samus_y_subpos, samus_x_radius,
        samus_y_radius, samus_collision_direction, samus_spritemap_x_pos, samus_spritemap_y_pos
    ];
    0x00B0A, uint16 Scalar => [ideal_layer1_xpos];
    0x00B0E stride 2, uint16 Scalar => [
        ideal_layer1_ypos, samus_prev_x_pos, samus_prev_x_subpos, samus_prev_y_pos,
        samus_prev_y_subpos, charged_shot_glow_timer, UNUSED_word_7E0B1A
    ];
    0x00B20 stride 2, uint16 Scalar => [
        used_for_ball_bounce_on_landing, samus_is_falling_flag, tempB24, tempB26
    ];
    0x00B2A stride 2, uint16 Scalar => [UNUSED_word_7E0B2A, samus_y_subspeed, samus_y_speed];
    0x00B32 stride 2, uint16 Scalar => [
        samus_y_subaccel, samus_y_accel, samus_y_dir, UNUSED_word_7E0B38
    ];
    0x00B3C stride 2, uint16 Scalar => [
        samus_has_momentum_flag, speed_boost_counter, samus_echoes_sound_flag,
        samus_x_extra_run_speed, samus_x_extra_run_subspeed, samus_x_base_speed,
        samus_x_base_subspeed, samus_x_accel_mode
    ];
    0x00B4C, uint8 Scalar => ["samus_x_decel_mult"];
    0x00B56 stride 2, uint16 Scalar => [
        extra_samus_x_subdisplacement, extra_samus_x_displacement, extra_samus_y_subdisplacement,
        extra_samus_y_displacement, new_projectile_direction_changed_pose, used_for_sba_attacksB60,
        samus_charge_palette_index
    ];
    0x00B64 stride 20, uint16 Array => [
        projectile_x_pos, projectile_y_pos, projectile_bomb_x_subpos, projectile_bomb_y_subpos,
        projectile_x_radius, projectile_y_radius, projectile_bomb_x_speed, projectile_bomb_y_speed,
        projectile_dir, projectile_type, projectile_damage, projectile_bomb_instruction_ptr,
        projectile_bomb_instruction_timers, projectile_bomb_pre_instructions
    ];
    0x00C72 stride 10, uint16 Array => [
        bomb_functions, projectile_variables, "bomb_timers", projectile_timers, "bomb_subspeeds",
        projectile_unk_A
    ];
    0x00CB8, uint16 Array => [projectile_spritemap_pointers];
    0x00CCC stride 2, uint16 Scalar => [
        cooldown_timer, projectile_counter, flare_counter, bomb_counter,
        bomb_spread_charge_timeout_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
        flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
        flare_fast_sparks_anim_timer, power_bomb_explosion_x_pos, power_bomb_explosion_y_pos,
        power_bomb_explosion_x_pos_plus_0x100, power_bomb_explosion_y_pos_rsub_0x1ff,
        power_bomb_explosion_radius, power_bomb_pre_explosion_flash_radius, power_bomb_flag,
        power_bomb_pre_explosion_radius_speed, pre_scaled_power_bomb_explosion_shape_def_ptr,
        grapple_beam_flags, grapple_varCF6, slow_grabble_scrolling_flag, grapple_beam_end_angle16
    ];
    0x00CFB, uint8 Scalar => ["grapple_beam_end_angle_hi"];
    0x00CFC stride 2, uint16 Scalar => [
        grapple_beam_end_angles_mirror, grapple_beam_length, grapple_beam_length_delta,
        grapple_beam_origin_x_offset, grapple_beam_origin_y_offset, grapple_beam_end_x_subpos,
        grapple_beam_end_x_pos, grapple_beam_end_y_subpos, grapple_beam_end_y_pos,
        grapple_beam_end_x_suboffset, grapple_beam_end_x_offset, grapple_beam_end_y_suboffset,
        grapple_beam_end_y_offset, x_pos_of_start_of_grapple_beam, y_pos_of_start_of_grapple_beam,
        x_pos_of_start_of_grapple_beam_prevframe, y_pos_of_start_of_grapple_beam_prevframe,
        grapple_beam_unkD1E, grapple_beam_unkD20, grapple_beam_extension_x_velocity,
        grapple_beam_extension_y_velocity, grapple_beam_unkD26, grapple_beam_unkD28,
        grapple_beam_unkD2A, grapple_beam_unkD2C, grapple_beam_unkD2E, grapple_beam_unkD30,
        grapple_beam_function, grapple_beam_direction, grapple_beam_unkD36, grapple_beam_unkD38,
        grapple_beam_unkD3A, grapple_beam_unkD3C, grapple_point_anim_timer, grapple_point_anim_ptr
    ];
    0x00D42, uint16 Array => [grapple_segment_anim_instr_timers];
    0x00D62, uint16 Array => [grapple_segment_anim_instr_ptrs];
    0x00D82 stride 2, uint16 Scalar => [
        button_config_shoot_x_saved, button_config_jump_a_saved, button_config_run_b_saved,
        button_config_itemcancel_y_saved, button_config_itemswitch_saved,
        button_config_aim_down_L_saved, button_config_aim_up_R_saved, grapple_beam_grapple_start_x,
        grapple_beam_grapple_start_y, grapple_beam_grapple_start_block_x,
        grapple_beam_grapple_start_block_y, g_word_7E0D98
    ];
    0x00D9C, uint16 Scalar => [g_word_7E0D9C];
    0x00D9E, uint8 Scalar => ["g_byte_7E0D9E"];
    0x00D9F, uint8 Scalar => ["g_byte_7E0D9F"];
    0x00DA0 stride 2, uint16 Scalar => [
        loop_counter_transfer_enemies_to_vram, absolute_moved_last_frame_x,
        absolute_moved_last_frame_x_fract, absolute_moved_last_frame_y,
        absolute_moved_last_frame_y_fract, projectile_init_speed_samus_moved_left,
        projectile_init_speed_samus_moved_left_fract, projectile_init_speed_samus_moved_right,
        projectile_init_speed_samus_moved_right_fract, projectile_init_speed_samus_moved_up,
        projectile_init_speed_samus_moved_up_fract, projectile_init_speed_samus_moved_down,
        projectile_init_speed_samus_moved_down_fract, samus_pos_adjusted_by_slope_flag,
        samus_total_x_speed, samus_total_x_subspeed, play_resume_charging_beam_sfx,
        prev_beam_charge_counter, cur_block_index, input_to_pose_calc, queued_message_box_index
    ];
    0x00DCE stride 2, uint16 Scalar => [
        samus_collides_with_solid_enemy, samus_collision_flag, REMOVED_temp_collision_DD2,
        REMOVED_temp_collision_DD4, REMOVED_temp_collision_DD6
    ];
    0x00DDC stride 2, uint16 Scalar => [
        suit_pickup_light_beam_widening_speed, projectile_index, debug_invincibility,
        game_options_screen_index, g_word_7E0DE4, g_word_7E0DE6, g_word_7E0DE8,
        which_item_to_pickup, substate, suit_pickup_light_beam_pos
    ];
    0x00DF0 stride 1, uint8 Scalar => [
        "suit_pickup_color_math_R", "suit_pickup_color_math_G", "suit_pickup_color_math_B",
        "suit_pickup_palette_transition_color"
    ];
    0x00DF4 stride 2, uint16 Scalar => [
        UNUSED_debug_spare_cpu_display_flag, UNUSED_word_7E0DF6, UNUSED_word_7E0DF8,
        UNUSED_word_7E0DFA, UNUSED_word_7E0DFC, joypad1_input_samusfilter,
        joypad1_newinput_samusfilter, REMOVED_samus_bottom_boundary_position,
        distance_to_eject_samus_left, distance_to_eject_samus_right, distance_to_eject_samus_up,
        distance_to_eject_samus_down
    ];
    0x00E0C, uint8 Array => ["g_byte_7E0E0C"];
    0x00E12, uint16 Scalar => [debug_disable_sprite_interact];
    0x00E16 stride 2, uint16 Scalar => [
        elevator_flags, elevator_status, health_drop_bias_flag, enemy_index_to_shake,
        nmi_flag_bg2_enemy_vram_transfer, REMOVED_varE20, REMOVED_varE22, REMOVED_varE24,
        REMOVED_varE26, REMOVED_varE28, REMOVED_varE2A, REMOVED_varE2C, REMOVED_varE2E
    ];
    0x00E32 stride 2, uint16 Scalar => [
        REMOVED_varE32, REMOVED_varE34, REMOVED_varE36, REMOVED_varE38, REMOVED_varE3A,
        REMOVED_varE3C, REMOVED_varE3E, last_enemy_power
    ];
    0x00E44 stride 2, uint16 Scalar => [
        random_enemy_counter, UNUSED_word_7E0E46, UNUSED_word_7E0E48, REMOVED_new_enemy_index,
        first_free_enemy_index, num_enemies_in_room, num_enemies_killed_in_room,
        num_enemy_deaths_left_to_clear, cur_enemy_index, REMOVED_cur_enemy_index_backup,
        REMOVED_enemy_data_ptr
    ];
    0x00E5A, EnemyTileLoadData Array => ["enemy_tile_load_data"];
    0x00E7A stride 2, uint16 Scalar => [
        enemy_tile_load_data_write_pos, enemy_tile_vram_src, enemy_tile_vram_dst
    ];
    0x00E84, uint16 Array => [enemy_drawing_queue];
    0x00F68, uint16 Array => [enemy_drawing_queue_sizes];
    0x00F78, EnemyData Array => ["enemy_data"];
    0x01778 stride 2, uint16 Scalar => [
        fireflea_flashing_timer, fireflea_flashing_index, UNUSED_word_7E177C,
        fireflea_darkness_level, UNUSED_word_7E1780, UNUSED_word_7E1782
    ];
    0x01784 stride 4, LongPtr Scalar => [
        "REMOVED_enemy_ai_pointer", "REMOVED_enemy_ai_pointer_backup", "enemy_gfx_drawn_hook",
        "REMOVED_set_to_rtl_when_loading_enemies_unused"
    ];
    0x01794, uint16 Array => [variables_for_enemy_graphics_drawn_hook];
    0x0179A stride 2, uint16 Scalar => [
        enemy_bg2_tilemap_size, boss_id, UNUSED_word_7E179E, UNUSED_word_7E17A0, UNUSED_word_7E17A2,
        active_enemy_indexes_write_ptr, interactive_enemy_indexes_write_ptr,
        REMOVED_active_enemy_indexes_index, REMOVED_interactive_enemy_indexes_index
    ];
    0x017AC stride 64, uint16 Array => [
        active_enemy_indexes, interactive_enemy_indexes, enemy_index_colliding_dirs
    ];
    0x01834, uint16 Array => ["REMOVED_distance_to_enemy_colliding_dirs"];
    0x0183C stride 2, uint16 Scalar => [
        flag_process_all_enemies, earthquake_type, earthquake_timer, frame_counter_every_frame,
        sprite_object_index, debug_enemy_index_UNUSED, log_index_UNUSED,
        REMOVED_samus_x_pos_colliding_solid, REMOVED_samus_x_subpos_colliding_solid,
        REMOVED_enemy_x_pos_colliding_solid, REMOVED_enemy_x_subpos_colliding_solid,
        REMOVED_samus_pos_delta_colliding_solid, REMOVED_samus_subpos_delta_colliding_solid,
        REMOVED_samus_y_pos_colliding_solid, REMOVED_samus_y_subpos_colliding_solid,
        REMOVED_solid_enemy_collision_type, debug_index, debug_time_frozen_for_enemies,
        debug_text_cursor_x, debug_text_cursor_y, debug_enemy_set_entry_index,
        debug_enemy_population_pointer, debug_enemy_spawn_x_pos, debug_enemy_spawn_y_pos
    ];
    0x0186E stride 2, uint16 Scalar => [
        REMOVED_enemy_spritemap_entry_pointer, REMOVED_enemy_left_border_collision,
        REMOVED_enemy_bottom_border_collision, REMOVED_enemy_right_border_collision,
        REMOVED_enemy_top_border_collision, REMOVED_hitbox_ptr,
        REMOVED_enemy_spritemap_entry_coll_x_pos, REMOVED_enemy_spritemap_entry_coll_y_pos,
        REMOVED_samus_right_border_coll, REMOVED_samus_left_border_coll,
        REMOVED_samus_bottom_border_coll, REMOVED_samus_top_border_coll
    ];
    0x0189A stride 2, uint16 Scalar => [
        REMOVED_samus_target_x_pos, REMOVED_samus_target_x_subpos, REMOVED_samus_target_y_pos,
        REMOVED_samus_target_y_subpos, REMOVED_samus_x_radius_mirror, REMOVED_samus_y_radius_mirror,
        collision_detection_index, samus_invincibility_timer, samus_knockback_timer,
        projectile_invincibility_timer, flag_disable_projectile_interaction,
        hdma_objects_enable_flag, hdma_object_index
    ];
    0x018B4 stride 12, uint16 Array => [
        hdma_object_channels_bitmask, hdma_object_bank_slot, hdma_object_instruction_list_pointers,
        hdma_object_table_pointers, hdma_object_instruction_timers, hdma_object_pre_instructions,
        hdma_object_pre_instruction_bank, hdma_object_timers, hdma_object_A, hdma_object_B,
        hdma_object_C, hdma_object_D, hdma_object_E, hdma_object_F
    ];
    0x0195C stride 2, uint16 Scalar => [
        fx_y_subpos, fx_y_pos, lava_acid_y_subpos, lava_acid_y_pos, fx_tilemap_ptr
    ];
    0x0196C stride 2, uint16 Scalar => [
        fx_rising_function_bank_88, fx_type, fx_y_suboffset, fx_y_offset, tide_phase,
        fx_base_y_subpos, fx_base_y_pos, fx_target_y_pos, fx_y_vel, fx_liquid_options, fx_timer,
        fx_layer_blending_config_a, fx_layer_blending_config_b, fx_layer_blending_config_c,
        phantom_related_layer_flag
    ];
    0x0198D, uint16 Scalar => [cinematic_var5];
    0x0198D, uint16 Scalar => [eproj_enable_flag];
    0x0198F, uint16 Scalar => [cinematic_var6];
    0x0198F, uint16 Scalar => [eproj_unk198F];
    0x01991, uint16 Scalar => [cinematic_var7];
    0x01991, uint16 Scalar => [eproj_index];
    0x01993, uint16 Scalar => [cinematic_var8];
    0x01993, uint16 Scalar => [eproj_init_param_1];
    0x01995, uint16 Scalar => [cinematic_var9];
    0x01995, uint16 Scalar => [eproj_unk1995];
    0x01997, uint16 Scalar => [cinematic_var10];
    0x01997, uint16 Array => [eproj_id];
    0x01999 stride 2, uint16 Scalar => [g_word_7E1999, g_word_7E199B, g_word_7E199D, g_word_7E199F];
    0x019A1 stride 4, uint16 Array => [
        mode7_obj_instr_ptr, mode7_obj_preinstr_func, mode7_obj_instr_timer, mode7_obj_goto_timer
    ];
    0x019B1, uint16 Scalar => [mode7_cur_index];
    0x019B3, uint16 Scalar => [mode7_spawn_param];
    0x019B5, uint16 Array => [cinematicbg_arr1];
    0x019BB, uint16 Array => [eproj_gfx_idx];
    0x019CD stride 8, uint16 Array => [
        cinematicbg_instr_ptr, cinematicbg_preinstr, cinematicbg_instr_timer
    ];
    0x019DF, uint16 Array => [eproj_timers];
    0x019E5, uint16 Array => [cinematicbg_arr6];
    0x019ED, uint16 Scalar => [cinematicbg_var2];
    0x019F1 stride 2, uint16 Scalar => [
        cinematic_enable_objs, cinematic_enable_bg_tilemap, cinematicbg_var3, cinematic_var21,
        cinematic_var22, cinematic_var23, cinematic_var24, cinematic_var25, cinematic_var26
    ];
    0x01A03, uint16 Array => [eproj_pre_instr];
    0x01A03, uint16 Array => ["eproj_pre_instr"];
    0x01A27, uint16 Array => [eproj_x_subpos];
    0x01A37, uint16 Scalar => [cinematic_var20];
    0x01A47 stride 2, uint16 Scalar => [cinematic_var19, cinematic_var4, cinematic_var13];
    0x01A4B, uint16 Array => [eproj_x_pos];
    0x01A4D stride 2, uint16 Scalar => [
        cinematic_var14, cinematic_var17, cinematic_var12, cinematic_var18
    ];
    0x01A57, uint16 Scalar => [cinematic_var15];
    0x01A59, uint16 Scalar => [cinematic_obj_index];
    0x01A5D, uint16 Array => [cinematicspr_whattodraw];
    0x01A6F, uint16 Array => [eproj_y_subpos];
    0x01A7D, uint16 Array => [cinematicbg_arr7];
    0x01A8F, uint16 Scalar => [optionsmenu_index];
    0x01A93, uint16 Array => [eproj_y_pos];
    0x01A93, uint16 Scalar => [options_menu_init_param];
    0x01A9D, uint16 Array => [cinematicbg_arr8];
    0x01A9D, uint16 Array => ["optionsmenu_cur_data"];
    0x01AAD, uint16 Array => [optionsmenu_arr8];
    0x01AB7, uint16 Array => [eproj_x_vel];
    0x01ABD, uint16 Array => [cinematicbg_arr9];
    0x01ABD, uint16 Array => ["optionsmenu_arr9"];
    0x01ACD, uint16 Array => [optionsmenu_arr10];
    0x01ADB, uint16 Array => [eproj_y_vel];
    0x01ADD, uint16 Array => [cinematicspr_arr6];
    0x01ADD stride 16, uint16 Array => ["optionsmenu_arr6", optionsmenu_arr7, cinematicspr_arr7];
    0x01AFD, uint16 Array => ["optionsmenu_instr_ptr"];
    0x01AFF, uint16 Array => [eproj_E];
    0x01B0D, uint16 Array => [optionsmenu_arr1];
    0x01B1D, uint16 Array => [cinematicspr_instr_ptr];
    0x01B1D, uint16 Array => ["optionsmenu_instr_timer"];
    0x01B23, uint16 Array => [eproj_F];
    0x01B2D, uint16 Array => [optionsmenu_arr5];
    0x01B3D, uint16 Array => [cinematicspr_preinstr_func];
    0x01B3D, uint16 Array => ["optionsmenu_arr11"];
    0x01B47, uint16 Array => [eproj_instr_list_ptr];
    0x01B5D, uint16 Array => [cinematicspr_instr_timer];
    0x01B6B stride 18, uint16 Array => [
        eproj_spritemap_ptr, cinematicspr_goto_timer, eproj_instr_timers
    ];
    0x01B9D stride 2, uint16 Scalar => [
        cinematic_spawn_param, cinematic_var11, cinematicbg_var1, cinematic_var16
    ];
    0x01BB3 stride 36, uint16 Array => [eproj_radius, eproj_properties, eproj_G];
    0x01C1F, uint16 Scalar => [message_box_index];
    0x01C23 stride 2, uint16 Scalar => [
        plm_flag, plm_draw_tilemap_index, plm_id, plm_x_block, plm_y_block, plm_item_gfx_index
    ];
    0x01C2F, uint16 Array => [plm_item_gfx_ptrs];
    0x01C37 stride 80, uint16 Array => [
        plm_header_ptr, plm_block_indices, plm_pre_instrs, plm_instr_list_ptrs, plm_timers,
        plm_room_arguments, plm_variable
    ];
    0x01E67 stride 2, uint16 Scalar => [
        custom_draw_instr_num_blocks, custom_draw_instr_plm_block,
        custom_draw_instr_zero_terminator, tourian_entrance_statue_finished,
        tourian_entrance_statue_animstate, flag_samus_in_quicksand,
        inside_block_reaction_samus_point, save_station_lockout_flag, current_slope_bts,
        flag_for_palette_fx_objects, palettefx_index
    ];
    0x01E7D, uint16 Array => [palettefx_ids];
    0x01E8D, uint16 Array => [palettefx_color_indexes];
    0x01EAD stride 16, uint16 Array => [
        palettefx_pre_instr, palettefx_instr_list_ptrs, palettefx_instr_timers, palettefx_timers
    ];
    0x01EED stride 2, uint16 Scalar => [
        samus_in_heat_palfx_index, samus_in_heat_palettefx_prev_index, animtiles_enable_flag,
        animtiles_object_index
    ];
    0x01EF5 stride 12, uint16 Array => [
        animtiles_ids, animtiles_timers, animtiles_instr_list_ptrs, animtiles_instr_timers,
        animtiles_src_ptr, animtiles_sizes, animtiles_vram_ptr
    ];
    0x01F49, uint16 Scalar => [animtiles_instruction];
    0x01F51 stride 2, uint16 Scalar => [
        cinematic_function, demo_timer, demo_set, demo_scene, num_demo_sets
    ];
    0x02000, uint16 Array => [tilemap_stuff];
    0x03000, Ram3000 Scalar => ["ram3000"];
    0x03800, Ram3800 Scalar => ["ram3800"];
    0x04000, Ram4000 Scalar => ["ram4000"];
    0x07000, EnemySpawnData Array => ["enemy_spawn_data"];
    0x07800, ExtraEnemyRam7800 Array => ["enemy_ram7800"];
    0x08000, ExtraEnemyRam8000 Array => ["extra_enemy_ram8000"];
    0x08800, ExtraEnemyRam8800 Array => ["extra_enemy_ram8800"];
    0x09000 stride 2, uint16 Scalar => [
        kraid_unk9000, g_word_7E9002, g_word_7E9004, g_word_7E9006, g_word_7E9008, g_word_7E900A,
        g_word_7E900C
    ];
    0x0900F stride 3, uint16 Scalar => [g_word_7E900F, g_word_7E9012, g_word_7E9015, g_word_7E9018];
    0x09020, uint16 Scalar => [g_word_7E9020];
    0x09022, uint16 Scalar => [g_word_7E9022];
    0x09030, uint16 Scalar => [g_word_7E9030];
    0x09032, uint16 Scalar => [g_word_7E9032];
    0x09040, uint16 Scalar => [g_word_7E9040];
    0x09042, uint16 Scalar => [g_word_7E9042];
    0x09050, uint16 Scalar => [g_word_7E9050];
    0x09052, uint16 Scalar => [g_word_7E9052];
    0x09060, uint16 Scalar => [g_word_7E9060];
    0x09062, uint16 Scalar => [g_word_7E9062];
    0x09070, uint16 Scalar => [g_word_7E9070];
    0x09072, uint16 Scalar => [g_word_7E9072];
    0x09080, uint16 Scalar => [g_word_7E9080];
    0x09082, uint16 Scalar => [g_word_7E9082];
    0x09090, uint16 Scalar => [g_word_7E9090];
    0x09092, uint16 Scalar => [g_word_7E9092];
    0x090A0, uint16 Scalar => [g_word_7E90A0];
    0x090A2, uint16 Scalar => [g_word_7E90A2];
    0x090B0, uint16 Scalar => [g_word_7E90B0];
    0x090B2, uint16 Scalar => [g_word_7E90B2];
    0x090C0, uint16 Scalar => [g_word_7E90C0];
    0x090C2, uint16 Scalar => [g_word_7E90C2];
    0x090D0, uint16 Scalar => [g_word_7E90D0];
    0x090D2, uint16 Scalar => [g_word_7E90D2];
    0x09100 stride 64, uint16 Scalar => [g_word_7E9100, g_word_7E9140, g_word_7E9180];
    0x09500 stride 16, uint8 Scalar => [
        "g_byte_7E9500", "g_byte_7E9510", "g_byte_7E9520", "g_byte_7E9530", "g_byte_7E9540",
        "g_byte_7E9550", "g_byte_7E9560", "g_byte_7E9570", "g_byte_7E9580", "g_byte_7E9590",
        "g_byte_7E95A0", "g_byte_7E95B0", "g_byte_7E95C0", "g_byte_7E95D0", "g_byte_7E95E0",
        "g_byte_7E95F0", "g_byte_7E9600", "g_byte_7E9610"
    ];
    0x097DC, uint16 Array => [g_word_7E97DC];
    0x09800, uint16 Array => [hdma_table_1];
    0x09C00, uint8 Array => ["mother_brain_indirect_hdma"];
    0x09C44, uint16 Scalar => [g_word_7E9C44];
    0x09C46, uint16 Array => [g_word_7E9C46];
    0x09D00, uint16 Array => [hdma_table_2];
    0x09E00, ExpandingSquareTransitionHdma Array => ["hdma_window_1_left_pos"];
    0x09E10, ExpandingSquareTransitionHdma Array => ["hdma_window_1_right_pos"];
    0x09E20, uint16 Scalar => [expand_sq_topbottom_margin_right_pos];
    0x09E22, uint16 Scalar => [expand_sq_topbottom_margin_left_pos];
    0x09E30 stride 2, uint16 Scalar => [
        expand_sq_left_subpos, expand_sq_left_pos, expand_sq_right_subpos, expand_sq_right_pos,
        expand_sq_top_subpos, expand_sq_top_pos, expand_sq_bottom_subpos, expand_sq_bottom_pos,
        expand_sq_left_subvel, expand_sq_left_vel, expand_sq_right_subvel, expand_sq_right_vel,
        expand_sq_top_subvel, expand_sq_top_vel, expand_sq_bottom_subvel, expand_sq_bottom_vel,
        expand_sq_timer
    ];
    0x09E80, uint16 Array => [g_word_7E9E80];
    0x09F00, uint8 Array => ["scrolling_sky_bg2_indirect_hdma"];
    0x09F80, uint16 Array => [scrolling_sky_bg2_hdma_data];
    0x0A000, TileTables Scalar => ["tile_table"];
    0x0C000, uint16 Array => [palette_buffer];
    0x0C200, uint16 Array => [target_palettes];
    0x0C400 stride 2, uint16 Scalar => [palette_change_num, palette_change_denom, g_word_7EC404];
    0x0C406, uint8 Array => ["power_bomb_explosion_left_hdma"];
    0x0C506, uint8 Array => ["power_bomb_explosion_right_hdma"];
    0x0C606, uint8 Scalar => ["offscreen_power_bomb_left_hdma"];
    0x0C607, uint8 Scalar => ["offscreen_power_bomb_right_hdma"];
    0x0C608, uint16 Array => [hud_tilemap];
    0x0C6C8, uint16 Array => ["plm_draw_tilemap"];
    0x0C8C8 stride 64, uint16 Array => [
        bg1_column_update_tilemap_left_halves, bg1_column_update_tilemap_right_halves,
        bg1_column_update_tilemap_top_halves
    ];
    0x0C98C, uint16 Array => [bg1_column_update_tilemap_bottom_halves];
    0x0C9D0, uint8 Array => ["bg2_column_update_tilemap_left_halves"];
    0x0CA10, uint16 Array => ["bg2_column_update_tilemap_right_halves"];
    0x0CA50, uint8 Array => ["bg2_column_update_tilemap_top_halves"];
    0x0CA94, uint8 Array => ["bg2_column_update_tilemap_bottom_halves"];
    0x0CAD8 stride 2, uint16 Scalar => [hud_bg3_xpos, hud_bg3_ypos, bg3_xpos, bg3_ypos];
    0x0CAE0, uint8 Array => ["crocomire_bg2_scroll_hdma_indirect"];
    0x0CAF0, uint16 Array => [crocomire_bg2_scroll_hdma_data];
    0x0CD20, uint8 Array => ["scrolls"];
    0x0CD52, uint16 Array => [explored_map_tiles_saved];
    0x0D552, uint8 Array => ["debug_enemy_set_name"];
    0x0D55A, uint8 Array => ["debug_enemy_set_data"];
    0x0D652, uint16 Scalar => [wrecked_ship_robot_palanim_palindex];
    0x0D654, uint16 Scalar => [wrecked_ship_robot_palanim_timer];
    0x0D656, uint8 Scalar => ["wrecked_ship_robot_palanim_table_index"];
    0x0D658 stride 36, uint16 Array => [
        projectiletrail_left_instr_timer, projectiletrail_right_instr_timer,
        projectiletrail_left_instr_list_ptr, projectiletrail_right_instr_list_ptr,
        projectiletrail_left_tile_and_attribs, projectiletrail_right_tile_and_attribs,
        projectiletrail_left_x_pos, projectiletrail_right_x_pos, projectiletrail_left_y_pos,
        projectiletrail_right_y_pos, player_data_saved
    ];
    0x0D820 stride 8, uint8 Array => [
        "events_that_happened", "boss_bits_for_area", "room_chozo_bits"
    ];
    0x0D870, uint8 Array => ["item_bit_array"];
    0x0D8B0, uint8 Array => ["opened_door_bit_array"];
    0x0D8F0, uint16 Array => [UNUSED_word_7ED8F0];
    0x0D8F8, uint8 Array => ["used_save_stations_and_elevators"];
    0x0D908, uint8 Array => ["map_station_byte_array"];
    0x0D914 stride 2, uint16 Scalar => [
        loading_game_state, sram_save_station_index, sram_area_index,
        global_number_of_items_loaded_ctr
    ];
    0x0D91C, uint8 Array => ["compressed_map_data"];
    0x0DE1C stride 80, uint16 Array => [
        plm_instruction_timer, plm_instruction_draw_ptr, plm_instruction_list_link_reg,
        plm_variables, "backup_of_bg2_during_pause"
    ];
    0x0EF5C stride 8, uint16 Array => [
        enemy_def_ptr, enemy_gfxdata_tiles_index, enemy_gfxdata_vram_ptr
    ];
    0x0EF74, uint16 Scalar => [REMOVED_enemy_gfx_data_write_ptr];
    0x0EF76, uint16 Scalar => [REMOVED_next_enemy_tiles_index];
    0x0EF78 stride 128, uint16 Array => [
        sprite_instr_list_ptrs, sprite_instr_timer, sprite_palettes, sprite_x_pos, sprite_x_subpos,
        sprite_y_pos, sprite_y_subpos, sprite_disable_flag
    ];
    0x0F378, uint16 Scalar => [enemy_processing_stage];
    0x0F380 stride 72, uint16 Array => [
        eproj_flags, eproj_enemy_header_ptr, eproj_killed_enemy_index
    ];
    0x0F434, uint16 Scalar => [special_death_item_drop_x_origin_pos];
    0x0F436, uint16 Scalar => [special_death_item_drop_y_origin_pos];
    0x10000, uint16 Scalar => [ram7F_start];
    0x10002, uint16 Array => [level_data];
    0x16402, uint8 Array => ["BTS"];
    0x19602, uint16 Array => [custom_background];
    0x1FF00, uint16 Scalar => [bug_fix_counter_BAD];
}

// Declare typed views of RAM without changing the getter's borrow or pointer type.
// Each list retains successive byte offsets, including the original integer casts.
macro_rules! ram_pointer_functions {
    ($($name:ident($index:ident: $index_type:ty) -> $target:ty = [$($offset:expr),+ $(,)?];)*) => {
        $(
            #[inline]
            unsafe fn $name($index: $index_type, game: &mut $crate::game_state::Game) -> *mut $target { unsafe {
                (&raw mut game.g_ram as *mut ::core::primitive::u8)
                    $(.offset($offset))+ as *mut $target
            }}
        )*
    };
}
pub(crate) use ram_pointer_functions;

// Keep raw RAM address construction at the caller, preserving offset order.
macro_rules! ram_ptr {
    ($game:ident; $($offset:expr),+ $(,)?) => {
        (&raw mut $game.g_ram as *mut ::core::primitive::u8)$(.offset($offset))+
    };
    ($game:ident, $target:ty; $($offset:expr),+ $(,)?) => {
        $crate::ram_map::ram_ptr!($game; $($offset),+) as *mut $target
    };
}
pub(crate) use ram_ptr;
