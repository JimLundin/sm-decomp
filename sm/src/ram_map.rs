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
macro_rules! ram_vars {
    ($($addr:literal, $name:tt, $ty:ident, $shape:ident;)*) => {
        /// Sorted by address. Several names can share an address; order is significant.
        pub const RAM_VARS: &[RamVar] = &[$(RamVar {
            addr: $addr,
            name: ram_vars!(@name $name),
            ty: stringify!($ty),
            shape: Shape::$shape,
        },)*];

        /// Named scalar word offsets share the verifier's address table.
        #[allow(non_upper_case_globals)]
        pub(crate) mod words {
            $(ram_vars!(@word $addr, $name, $ty, $shape);)*
        }

        /// Named word-array bases; indices are signed element offsets.
        #[allow(non_upper_case_globals)]
        pub(crate) mod word_arrays {
            $(ram_vars!(@array $addr, $name, $ty, $shape);)*
        }
    };
    (@name $name:literal) => { $name };
    (@name $name:ident) => { stringify!($name) };
    (@word $addr:literal, $name:ident, uint16, Scalar) => {
        pub(crate) const $name: usize = $addr;
    };
    (@word $addr:literal, $name:ident, uint16, Array) => {};
    (@word $addr:literal, $name:literal, $ty:ident, $shape:ident) => {};
    (@array $addr:literal, $name:ident, uint16, Array) => {
        pub(crate) const $name: usize = $addr;
    };
    (@array $addr:literal, $name:tt, $ty:ident, $shape:ident) => {};
}

// Named SNES word reads, assignments and updates share one caller-side interface.
// `word!(game.field)` reads; `word!(game.field = value)` writes. Arrays use
// `field[index]`. Updates use `(game.field, method(argument), reread)` or an
// expression `(game.field, |old| value)`; `reread` is optional. Ordered stores
// of one literal use `(game, value; field, other[index])`; different values use
// `(game; field = value; other[index] = next;)`.
//
// These operations require an unsafe context. Keep raw loads/stores at the
// caller: a whole-Game/RAM borrow can invalidate retained raw enemy pointers.
// Odd addresses and signed array offsets outside a named table remain valid.
// Reads form the base before the index; assignments evaluate RHS before the
// destination, retaining RHS temporaries through the store. Updates capture
// the destination and read its old value before evaluating the RHS. `reread`
// loads that destination after RHS temporaries are dropped. No value or index
// is hoisted across ordered stores; update expressions create no closure.
macro_rules! word {
    // Compound arithmetic wraps at 16 bits; bits/shifts keep ordinary u16 operations.
    // Delegate pointer capture, old load, RHS evaluation and store to the update arm.
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
        u16::from_le(
            (&raw const $game.g_ram)
                .cast::<u8>()
                .add($crate::ram_map::word_arrays::$field)
                .cast::<u16>()
                .offset(($index) as isize)
                .read_unaligned(),
        )
    };
    ($game:ident.$field:ident) => {
        u16::from_le(
            (&raw const $game.g_ram)
                .cast::<u8>()
                .add($crate::ram_map::words::$field)
                .cast::<u16>()
                .read_unaligned(),
        )
    };
    ($game:ident.$field:ident[$index:expr] = $value:expr $(,)?) => {
        $crate::ram_map::write_word(
            $value,
            (&raw mut $game.g_ram)
                .cast::<u8>()
                .add($crate::ram_map::word_arrays::$field)
                .cast::<u16>()
                .offset(($index) as isize),
        )
    };
    ($game:ident.$field:ident = $value:expr $(,)?) => {
        $crate::ram_map::write_word(
            $value,
            (&raw mut $game.g_ram)
                .cast::<u8>()
                .add($crate::ram_map::words::$field)
                .cast::<u16>(),
        )
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
        let destination = (&raw mut $game.g_ram)
            .cast::<u8>()
            .add($crate::ram_map::word_arrays::$field)
            .cast::<u16>()
            .offset(($index) as isize);
        let $old = u16::from_le(destination.read_unaligned());
        $crate::ram_map::write_word($value, destination);
        $crate::ram_map::word!(@result destination $(, $result)?)
    }};
    ($game:ident.$field:ident, |$old:ident| $value:expr $(, $result:ident)? $(,)?) => {{
        let destination = (&raw mut $game.g_ram)
            .cast::<u8>()
            .add($crate::ram_map::words::$field)
            .cast::<u16>();
        let $old = u16::from_le(destination.read_unaligned());
        $crate::ram_map::write_word($value, destination);
        $crate::ram_map::word!(@result destination $(, $result)?)
    }};
    (@result $destination:ident) => { () };
    (@result $destination:ident, reread) => {
        u16::from_le($destination.read_unaligned())
    };
}
pub(crate) use word;

/// Store one SNES word without imposing host alignment.
///
/// # Safety
/// `destination` must be valid for writing two bytes. It may be unaligned.
#[inline]
pub(crate) unsafe fn write_word(value: u16, destination: *mut u16) {
    destination.write_unaligned(value.to_le());
}

ram_vars! {
    0x00012, REMOVED_R18, uint16, Scalar;
    0x00014, REMOVED_R20, uint16, Scalar;
    0x00016, REMOVED_R22, uint16, Scalar;
    0x00051, "reg_INIDISP", uint8, Scalar;
    0x00052, "reg_OBSEL", uint8, Scalar;
    0x00053, reg_OAMaddr_UNUSED, uint16, Scalar;
    0x00055, "reg_BGMODE", uint8, Scalar;
    0x00056, "reg_BGMODE_fake", uint8, Scalar;
    0x00057, "reg_MOSAIC", uint8, Scalar;
    0x00058, "reg_BG1SC", uint8, Scalar;
    0x00059, "reg_BG2SC", uint8, Scalar;
    0x0005A, "reg_BG3SC", uint8, Scalar;
    0x0005B, "gameplay_BG3SC", uint8, Scalar;
    0x0005C, "reg_BG4SC", uint8, Scalar;
    0x0005D, "reg_BG12NBA", uint8, Scalar;
    0x0005E, "reg_BG34NBA", uint8, Scalar;
    0x0005F, "reg_M7SEL", uint8, Scalar;
    0x00060, "reg_W12SEL", uint8, Scalar;
    0x00061, "reg_W34SEL", uint8, Scalar;
    0x00062, "reg_WOBJSEL", uint8, Scalar;
    0x00063, "reg_WH0", uint8, Scalar;
    0x00064, "reg_WH1", uint8, Scalar;
    0x00065, "reg_WH2", uint8, Scalar;
    0x00066, "reg_WH3", uint8, Scalar;
    0x00067, "reg_WBGLOG", uint8, Scalar;
    0x00068, "reg_WOBJLOG", uint8, Scalar;
    0x00069, "reg_TM", uint8, Scalar;
    0x0006A, "gameplay_TM", uint8, Scalar;
    0x0006B, "reg_TS", uint8, Scalar;
    0x0006C, "reg_TMW", uint8, Scalar;
    0x0006D, "reg_TSW", uint8, Scalar;
    0x0006E, "next_gameplay_CGWSEL", uint8, Scalar;
    0x0006F, "reg_CGWSEL", uint8, Scalar;
    0x00070, "gameplay_CGWSEL", uint8, Scalar;
    0x00071, "next_gameplay_CGADSUB", uint8, Scalar;
    0x00072, "reg_CGADSUB", uint8, Scalar;
    0x00073, "gameplay_CGADSUB", uint8, Scalar;
    0x00074, "reg_COLDATA", uint8, Array;
    0x00077, "reg_SETINI", uint8, Scalar;
    0x00078, reg_M7A, uint16, Scalar;
    0x0007A, reg_M7B, uint16, Scalar;
    0x0007C, reg_M7C, uint16, Scalar;
    0x0007E, reg_M7D, uint16, Scalar;
    0x00080, reg_M7X, uint16, Scalar;
    0x00082, reg_M7Y, uint16, Scalar;
    0x00084, "reg_NMITIMEN", uint8, Scalar;
    0x00085, "reg_HDMAEN", uint8, Scalar;
    0x00086, "reg_MEMSEL", uint8, Scalar;
    0x00087, joypad_ctr_repeat_first, uint16, Scalar;
    0x00089, joypad_ctr_repeat_next, uint16, Scalar;
    0x0008B, joypad1_lastkeys, uint16, Scalar;
    0x0008D, joypad2_last, uint16, Scalar;
    0x0008F, joypad1_newkeys, uint16, Scalar;
    0x00091, joypad2_new_keys, uint16, Scalar;
    0x00093, joypad1_newkeys2_UNUSED, uint16, Scalar;
    0x00095, joypad2_newkeys2, uint16, Scalar;
    0x00097, joypad1_prev, uint16, Scalar;
    0x00099, joypad2_prev, uint16, Scalar;
    0x000A3, joypad1_keyrepeat_ctr, uint16, Scalar;
    0x000A5, joypad2_keyrepeat_ctr, uint16, Scalar;
    0x000A7, irqhandler_next_handler, uint16, Scalar;
    0x000A9, room_loading_irq_handler, uint16, Scalar;
    0x000AB, cur_irq_handler, uint16, Scalar;
    0x000AD, REMOVED_ptr_to_retaddr_parameters, uint16, Scalar;
    0x000AF, current_region_animated_palette_fx, uint16, Scalar;
    0x000B1, reg_BG1HOFS, uint16, Scalar;
    0x000B3, reg_BG1VOFS, uint16, Scalar;
    0x000B5, reg_BG2HOFS, uint16, Scalar;
    0x000B7, reg_BG2VOFS, uint16, Scalar;
    0x000B9, reg_BG3HOFS, uint16, Scalar;
    0x000BB, reg_BG3VOFS, uint16, Scalar;
    0x000BD, reg_BG4HOFS, uint16, Scalar;
    0x000BF, reg_BG4VOFS, uint16, Scalar;
    0x000C2, bug_fix_counter, uint16, Scalar;
    0x000C4, "g_word_7E00C4", uint16, Array;
    0x000D0, "vram_write_queue", VramWriteEntry, Array;
    0x002D0, "mode7_write_queue", Mode7VramWriteQueue, Array;
    0x00330, vram_write_queue_tail, uint16, Scalar;
    0x00334, mode7_vram_write_queue_tail, uint16, Scalar;
    0x00340, "vram_read_queue", VramReadQueueEnt, Array;
    0x00360, vram_read_queue_tail, uint16, Scalar;
    0x00370, "oam_ent", OamEnt, Array;
    0x00570, oam_ext, uint16, Array;
    0x00590, oam_next_ptr, uint16, Scalar;
    0x00592, power_bomb_explosion_status, uint16, Scalar;
    0x00596, g_word_7E0596, uint16, Scalar;
    0x00598, g_word_7E0598, uint16, Scalar;
    0x0059A, set_to_e0_by_scrolling_sky, uint16, Scalar;
    0x0059C, UNUSED_word_7E059C, uint16, Scalar;
    0x005A0, UNUSED_hdma_contracting_flag, uint16, Scalar;
    0x005A2, message_box_animation_y_radius, uint16, Scalar;
    0x005A4, message_box_animation_y0, uint16, Scalar;
    0x005A6, message_box_animation_y1, uint16, Scalar;
    0x005A8, message_box_animation_y2, uint16, Scalar;
    0x005AA, message_box_animation_y3, uint16, Scalar;
    0x005AC, map_min_x_scroll, uint16, Scalar;
    0x005AE, map_max_x_scroll, uint16, Scalar;
    0x005B0, map_min_y_scroll, uint16, Scalar;
    0x005B2, map_max_y_scroll, uint16, Scalar;
    0x005B4, "waiting_for_nmi", uint8, Scalar;
    0x005B5, "nmi_frame_counter_byte", uint8, Scalar;
    0x005B6, nmi_frame_counter_word, uint16, Scalar;
    0x005B8, nmi_frame_counter_including_lag, uint16, Scalar;
    0x005BA, "nmi_frames_missed", uint8, Scalar;
    0x005BB, "nmi_frames_missed_max", uint8, Scalar;
    0x005C5, joypad_dbg_1, uint16, Scalar;
    0x005C7, joypad_dbg_2, uint16, Scalar;
    0x005C9, joypad_dbg_missiles_swap, uint16, Scalar;
    0x005CB, joypad_dbg_super_missiles_swap, uint16, Scalar;
    0x005CD, joypad_dbg_power_bombs_swap, uint16, Scalar;
    0x005CF, joypad_dbg_flags, uint16, Scalar;
    0x005D1, enable_debug, uint16, Scalar;
    0x005D3, debug_saveload_scrollpos_toggle, uint16, Scalar;
    0x005D5, debug_saved_xscroll, uint16, Scalar;
    0x005D7, debug_saved_yscroll, uint16, Scalar;
    0x005D9, joypad_released_keys, uint16, Scalar;
    0x005DB, timed_held_input_timer, uint16, Scalar;
    0x005DD, timed_held_input_timer_reset, uint16, Scalar;
    0x005DF, timed_held_input, uint16, Scalar;
    0x005E1, newly_held_down_timed_held_input, uint16, Scalar;
    0x005E3, previous_timed_held_input, uint16, Scalar;
    0x005E5, random_number, uint16, Scalar;
    0x005E7, bitmask, uint16, Scalar;
    0x005E9, REMOVED_mult_tmp_a, uint16, Scalar;
    0x005EB, REMOVED_mult_tmp_y, uint16, Scalar;
    0x005F1, REMOVED_mult_product_lo, uint16, Scalar;
    0x005F3, REMOVED_mult_product_hi, uint16, Scalar;
    0x005F5, debug_disable_sounds, uint16, Scalar;
    0x005F7, debug_disable_minimap, uint16, Scalar;
    0x005F9, save_confirmation_selection, uint16, Scalar;
    0x005FB, map_scrolling_gear_switch_timer, uint16, Scalar;
    0x005FD, map_scrolling_direction, uint16, Scalar;
    0x005FF, map_scrolling_speed_index, uint16, Scalar;
    0x00601, "pause_hook", LongPtr, Scalar;
    0x00604, "unpause_hook", LongPtr, Scalar;
    0x00607, earthquake_sfx_index, uint16, Scalar;
    0x00609, earthquake_sfx_timer, uint16, Scalar;
    0x0060B, "eproj_init_param_3", uint8, Scalar;
    0x0060D, REMOVED_remaining_enemy_hitbox_entries, uint16, Scalar;
    0x0060F, REMOVED_num_projectiles_to_check_enemy_coll, uint16, Scalar;
    0x00611, "coroutine_state_0", uint8, Scalar;
    0x00612, "coroutine_state_1", uint8, Scalar;
    0x00613, "coroutine_state_2", uint8, Scalar;
    0x00614, "coroutine_state_3", uint8, Scalar;
    0x00615, "coroutine_state_4", uint8, Scalar;
    0x00617, is_uploading_apu, uint16, Scalar;
    0x00619, music_queue_track, uint16, Array;
    0x00629, music_queue_delay, uint16, Array;
    0x00639, music_queue_write_pos, uint16, Scalar;
    0x0063B, music_queue_read_pos, uint16, Scalar;
    0x0063D, music_entry, uint16, Scalar;
    0x0063F, music_timer, uint16, Scalar;
    0x00641, REMOVED_apu_attempts_countdown, uint16, Scalar;
    0x00643, "sfx_readpos", uint8, Array;
    0x00646, "sfx_writepos", uint8, Array;
    0x00649, "sfx_state", uint8, Array;
    0x0064C, "cur_music_track", uint8, Scalar;
    0x0064D, "sfx_cur", uint8, Array;
    0x00650, "sfx_clear_delay", uint8, Array;
    0x00653, "sfx_max_queued", uint8, Array;
    0x00656, "sfx1_queue", uint8, Array;
    0x00666, "sfx2_queue", uint8, Array;
    0x00676, "sfx3_queue", uint8, Array;
    0x00686, sound_handler_downtime, uint16, Scalar;
    0x00688, croco_target_0688, uint16, Scalar;
    0x0068A, g_word_7E068A, uint16, Scalar;
    0x0068C, "croco_word_068C", uint8, Scalar;
    0x0068E, g_word_7E068E, uint16, Scalar;
    0x00690, croco_cur_vline_idx, uint16, Scalar;
    0x00692, g_word_7E0692, uint16, Scalar;
    0x00694, g_word_7E0694, uint16, Scalar;
    0x00696, g_word_7E0696, uint16, Scalar;
    0x00698, g_word_7E0698, uint16, Scalar;
    0x0069A, croco_word_7E069A, uint16, Scalar;
    0x0069C, "croco_vline_height", uint8, Array;
    0x0071C, "UNUSED_byte_7E071C", uint8, Scalar;
    0x0071D, nmi_copy_samus_halves, uint16, Scalar;
    0x0071F, nmi_copy_samus_top_half_src, uint16, Scalar;
    0x00721, nmi_copy_samus_bottom_half_src, uint16, Scalar;
    0x00723, screen_fade_delay, uint16, Scalar;
    0x00725, screen_fade_counter, uint16, Scalar;
    0x00727, menu_index, uint16, Scalar;
    0x00729, pausemenu_start_lr_pressed_highlight_timer, uint16, Scalar;
    0x0072B, pausemenu_lr_animation_timer, uint16, Scalar;
    0x0072D, pausemenu_item_selector_animation_timer, uint16, Scalar;
    0x0072F, pausemenu_reserve_tank_animation_timer, uint16, Scalar;
    0x00731, UNUSED_word_7E0731, uint16, Scalar;
    0x00733, pausemenu_uparrow_animation_timer, uint16, Scalar;
    0x00735, pausemenu_downarrow_animation_timer, uint16, Scalar;
    0x00737, pausemenu_rightarrow_animation_timer, uint16, Scalar;
    0x00739, pausemenu_leftarrow_animation_timer, uint16, Scalar;
    0x0073B, pausemenu_palette_animation_timer, uint16, Scalar;
    0x0073D, UNUSED_word_7E073D, uint16, Scalar;
    0x0073F, pausemenu_lr_animation_frame, uint16, Scalar;
    0x00741, pausemenu_item_selector_animation_frame, uint16, Scalar;
    0x00743, pausemenu_reserve_tank_animation_frame, uint16, Scalar;
    0x00745, UNUSED_word_7E0745, uint16, Scalar;
    0x00747, pausemenu_uparrow_animation_frame, uint16, Scalar;
    0x00749, pausemenu_downarrow_animation_frame, uint16, Scalar;
    0x0074B, pausemenu_rightarrow_animation_frame, uint16, Scalar;
    0x0074D, pausemenu_leftarrow_animation_frame, uint16, Scalar;
    0x0074F, pausemenu_palette_animation_frame, uint16, Scalar;
    0x00751, pausemenu_shoulder_button_highlight, uint16, Scalar;
    0x00753, pausemenu_button_label_mode, uint16, Scalar;
    0x00755, pausemenu_equipment_category_item, uint16, Scalar;
    0x00757, pausemenu_reserve_tank_delay_ctr, uint16, Scalar;
    0x00759, UNUSED_word_7E0759, uint16, Scalar;
    0x0075B, UNUSED_word_7E075B, uint16, Scalar;
    0x0075D, UNUSED_word_7E075D, uint16, Scalar;
    0x0075F, UNUSED_word_7E075F, uint16, Scalar;
    0x00761, UNUSED_word_7E0761, uint16, Scalar;
    0x00763, pause_screen_mode, uint16, Scalar;
    0x00765, "pausemenu_bak_BG1SC", uint8, Scalar;
    0x00766, "pausemenu_bak_BG2SC", uint8, Scalar;
    0x00767, "pausemenu_bak_BG3SC", uint8, Scalar;
    0x00768, "pausemenu_bak_BG12NBA", uint8, Scalar;
    0x00769, "pausemenu_bak_BG34NBA", uint8, Scalar;
    0x0076A, "pausemenu_bak_OBSEL", uint8, Scalar;
    0x0076B, "pausemenu_bak_BG1HOFS", uint8, Scalar;
    0x0076C, "pausemenu_bak_BG2HOFS", uint8, Scalar;
    0x0076D, "pausemenu_bak_BG3HOFS", uint8, Scalar;
    0x0076E, "pausemenu_bak_BG1VOFS", uint8, Scalar;
    0x0076F, "pausemenu_bak_BG2VOFS", uint8, Scalar;
    0x00770, "pausemenu_bak_BG3VOFS", uint8, Scalar;
    0x00771, "pausemenu_bak_BGMODE", uint8, Scalar;
    0x00772, "pausemenu_bak_layer2_scroll_x", uint8, Scalar;
    0x00773, "pausemenu_bak_layer2_scroll_y", uint8, Scalar;
    0x00774, "pausemenu_bak_MOSAIC", uint8, Scalar;
    0x00775, "pausemenu_bak_CGADSUB", uint8, Scalar;
    0x00776, samus_position_indicator_animation_frame, uint16, Scalar;
    0x00778, samus_position_indicator_animation_timer, uint16, Scalar;
    0x0077A, samus_position_indicator_animation_loop_counter, uint16, Scalar;
    0x0077C, hud_item_tilemap_palette_bits, uint16, Scalar;
    0x0077E, my_counter, uint16, Scalar;
    0x00780, my_counter2, uint16, Scalar;
    0x00782, "coroutine_completion_flags", uint8, Scalar;
    0x00783, irq_enable_mode7, uint16, Scalar;
    0x00785, mode7_rotation_angle, uint16, Scalar;
    0x00787, g_word_7E0787, uint16, Scalar;
    0x00789, has_area_map, uint16, Scalar;
    0x0078B, load_station_index, uint16, Scalar;
    0x0078D, door_def_ptr, uint16, Scalar;
    0x0078F, REMOVED_door_bts, uint16, Scalar;
    0x00791, door_direction, uint16, Scalar;
    0x00793, elevator_door_properties_orientation, uint16, Scalar;
    0x00795, door_transition_flag_elevator_zebetites, uint16, Scalar;
    0x00797, door_transition_flag_enemies, uint16, Scalar;
    0x00799, elevator_direction, uint16, Scalar;
    0x0079B, room_ptr, uint16, Scalar;
    0x0079D, room_index, uint16, Scalar;
    0x0079F, area_index, uint16, Scalar;
    0x007A1, room_x_coordinate_on_map, uint16, Scalar;
    0x007A3, room_y_coordinate_on_map, uint16, Scalar;
    0x007A5, room_width_in_blocks, uint16, Scalar;
    0x007A7, room_height_in_blocks, uint16, Scalar;
    0x007A9, room_width_in_scrolls, uint16, Scalar;
    0x007AB, room_height_in_scrolls, uint16, Scalar;
    0x007AD, up_scroller, uint16, Scalar;
    0x007AF, down_scroller, uint16, Scalar;
    0x007B1, previous_cre_bitset, uint16, Scalar;
    0x007B3, cre_bitset, uint16, Scalar;
    0x007B5, door_list_pointer, uint16, Scalar;
    0x007B7, REMOVED_event_pointer, uint16, Scalar;
    0x007B9, room_size_in_blocks, uint16, Scalar;
    0x007BB, roomdefroomstate_ptr, uint16, Scalar;
    0x007BD, "room_compr_level_data_ptr", LongPtr, Scalar;
    0x007C0, "tileset_tile_table_pointer", LongPtr, Scalar;
    0x007C3, "tileset_tiles_pointer", LongPtr, Scalar;
    0x007C6, "tileset_compr_palette_ptr", LongPtr, Scalar;
    0x007C9, room_music_track_index, uint16, Scalar;
    0x007CB, room_music_data_index, uint16, Scalar;
    0x007CD, room_layer3_asm_ptr, uint16, Scalar;
    0x007CF, room_enemy_population_ptr, uint16, Scalar;
    0x007D1, room_enemy_tilesets_ptr, uint16, Scalar;
    0x007DF, room_main_code_ptr, uint16, Scalar;
    0x007E1, "room_main_asm_variables", uint8, Array;
    0x007E9, scrolling_finished_hook, uint16, Scalar;
    0x007EB, hdma_data_table_in_ceres, uint16, Scalar;
    0x007F3, music_data_index, uint16, Scalar;
    0x007F5, music_track_index, uint16, Scalar;
    0x007F7, "map_tiles_explored", uint8, Array;
    0x008F7, layer1_x_block, uint16, Scalar;
    0x008F9, layer1_y_block, uint16, Scalar;
    0x008FB, layer2_x_block, uint16, Scalar;
    0x008FD, layer2_y_block, uint16, Scalar;
    0x008FF, previous_layer1_x_block, uint16, Scalar;
    0x00901, previous_layer1_y_block, uint16, Scalar;
    0x00903, previous_layer2_x_block, uint16, Scalar;
    0x00905, previous_layer2_y_block, uint16, Scalar;
    0x00907, bg1_x_block, uint16, Scalar;
    0x00909, bg1_y_block, uint16, Scalar;
    0x0090B, bg2_x_block, uint16, Scalar;
    0x0090D, bg2_y_block, uint16, Scalar;
    0x0090F, layer1_x_subpos, uint16, Scalar;
    0x00911, layer1_x_pos, uint16, Scalar;
    0x00913, layer1_y_subpos, uint16, Scalar;
    0x00915, layer1_y_pos, uint16, Scalar;
    0x00917, layer2_x_pos, uint16, Scalar;
    0x00919, layer2_y_pos, uint16, Scalar;
    0x0091B, "layer2_scroll_x", uint8, Scalar;
    0x0091C, "layer2_scroll_y", uint8, Scalar;
    0x0091D, bg1_x_offset, uint16, Scalar;
    0x0091F, bg1_y_offset, uint16, Scalar;
    0x00921, bg2_x_scroll, uint16, Scalar;
    0x00923, bg2_y_scroll, uint16, Scalar;
    0x00925, door_transition_frame_counter, uint16, Scalar;
    0x00927, door_destination_x_pos, uint16, Scalar;
    0x00929, door_destination_y_pos, uint16, Scalar;
    0x0092B, samus_door_transition_subspeed, uint16, Scalar;
    0x0092D, samus_door_transition_speed, uint16, Scalar;
    0x0092F, downwards_elevator_delay_timer, uint16, Scalar;
    0x00931, door_transition_flag, uint16, Scalar;
    0x00933, REMOVED_var933, uint16, Scalar;
    0x00935, REMOVED_var935, uint16, Scalar;
    0x00937, REMOVED_var937, uint16, Scalar;
    0x00939, REMOVED_var939, uint16, Scalar;
    0x0093B, REMOVED_var93B, uint16, Scalar;
    0x0093F, ceres_status, uint16, Scalar;
    0x00941, camera_distance_index, uint16, Scalar;
    0x00943, timer_status, uint16, Scalar;
    0x00945, "timer_centiseconds", uint8, Scalar;
    0x00946, "timer_seconds", uint8, Scalar;
    0x00947, "timer_minutes", uint8, Scalar;
    0x00948, timer_x_pos, uint16, Scalar;
    0x0094A, timer_y_pos, uint16, Scalar;
    0x00950, file_select_map_area_index, uint16, Scalar;
    0x00952, selected_save_slot, uint16, Scalar;
    0x00954, nonempty_save_slots, uint16, Scalar;
    0x00956, bg1_update_col_unwrapped_size, uint16, Scalar;
    0x00958, bg1_update_col_wrapped_size, uint16, Scalar;
    0x0095A, bg1_update_col_unwrapped_dst, uint16, Scalar;
    0x0095C, bg1_update_col_wrapped_dst, uint16, Scalar;
    0x0095E, bg1_update_col_wrapped_left_src, uint16, Scalar;
    0x00960, bg1_update_col_wrapped_right_src, uint16, Scalar;
    0x00962, bg1_update_col_enable, uint16, Scalar;
    0x00964, bg1_update_row_unwrapped_size, uint16, Scalar;
    0x00966, bg1_update_row_wrapped_size, uint16, Scalar;
    0x00968, bg1_update_row_unwrapped_dst, uint16, Scalar;
    0x0096A, bg1_update_row_wrapped_dst, uint16, Scalar;
    0x0096C, bg1_update_row_wrapped_top_src, uint16, Scalar;
    0x0096E, bg1_update_row_wrapped_bottom_src, uint16, Scalar;
    0x00970, bg1_update_row_enable, uint16, Scalar;
    0x00972, bg2_update_col_unwrapped_size, uint16, Scalar;
    0x00974, bg2_update_col_wrapped_size, uint16, Scalar;
    0x00976, bg2_update_col_unwrapped_dst, uint16, Scalar;
    0x00978, bg2_update_col_wrapped_dst, uint16, Scalar;
    0x0097A, bg2_update_col_wrapped_left_src, uint16, Scalar;
    0x0097C, bg2_update_col_wrapped_right_src, uint16, Scalar;
    0x0097E, bg2_update_col_enable, uint16, Scalar;
    0x00980, bg2_update_row_unwrapped_size, uint16, Scalar;
    0x00982, bg2_update_row_wrapped_size, uint16, Scalar;
    0x00984, bg2_update_row_unwrapped_dst, uint16, Scalar;
    0x00986, bg2_update_row_wrapped_dst, uint16, Scalar;
    0x00988, bg2_update_row_wrapped_top_src, uint16, Scalar;
    0x0098A, bg2_update_row_wrapped_bottom_src, uint16, Scalar;
    0x0098C, bg2_update_row_enable, uint16, Scalar;
    0x0098E, size_of_bg2, uint16, Scalar;
    0x00990, blocks_to_update_x_block, uint16, Scalar;
    0x00992, blocks_to_update_y_block, uint16, Scalar;
    0x00994, vram_blocks_to_update_x_block, uint16, Scalar;
    0x00996, vram_blocks_to_update_y_block, uint16, Scalar;
    0x00998, game_state, uint16, Scalar;
    0x0099C, door_transition_function, uint16, Scalar;
    0x0099E, menu_option_index, uint16, Scalar;
    0x009A2, equipped_items, uint16, Scalar;
    0x009A4, collected_items, uint16, Scalar;
    0x009A6, equipped_beams, uint16, Scalar;
    0x009A8, collected_beams, uint16, Scalar;
    0x009AA, button_config_up, uint16, Scalar;
    0x009AC, button_config_down, uint16, Scalar;
    0x009AE, button_config_left, uint16, Scalar;
    0x009B0, button_config_right, uint16, Scalar;
    0x009B2, button_config_shoot_x, uint16, Scalar;
    0x009B4, button_config_jump_a, uint16, Scalar;
    0x009B6, button_config_run_b, uint16, Scalar;
    0x009B8, button_config_itemcancel_y, uint16, Scalar;
    0x009BA, button_config_itemswitch, uint16, Scalar;
    0x009BC, button_config_aim_down_L, uint16, Scalar;
    0x009BE, button_config_aim_up_R, uint16, Scalar;
    0x009C0, reserve_health_mode, uint16, Scalar;
    0x009C2, samus_health, uint16, Scalar;
    0x009C4, samus_max_health, uint16, Scalar;
    0x009C6, samus_missiles, uint16, Scalar;
    0x009C8, samus_max_missiles, uint16, Scalar;
    0x009CA, samus_super_missiles, uint16, Scalar;
    0x009CC, samus_max_super_missiles, uint16, Scalar;
    0x009CE, samus_power_bombs, uint16, Scalar;
    0x009D0, samus_max_power_bombs, uint16, Scalar;
    0x009D2, hud_item_index, uint16, Scalar;
    0x009D4, samus_max_reserve_health, uint16, Scalar;
    0x009D6, samus_reserve_health, uint16, Scalar;
    0x009D8, samus_reserve_missiles, uint16, Scalar;
    0x009DA, game_time_frames, uint16, Scalar;
    0x009DC, game_time_seconds, uint16, Scalar;
    0x009DE, game_time_minutes, uint16, Scalar;
    0x009E0, game_time_hours, uint16, Scalar;
    0x009E2, japanese_text_flag, uint16, Scalar;
    0x009E4, moonwalk_flag, uint16, Scalar;
    0x009E6, debug_flag, uint16, Scalar;
    0x009E8, UNUSED_word_7E09E8, uint16, Scalar;
    0x009EA, hud_auto_cancel_flag, uint16, Scalar;
    0x00A04, samus_auto_cancel_hud_item_index, uint16, Scalar;
    0x00A06, samus_prev_health, uint16, Scalar;
    0x00A08, samus_prev_missiles, uint16, Scalar;
    0x00A0A, samus_prev_super_missiles, uint16, Scalar;
    0x00A0C, samus_prev_power_bombs, uint16, Scalar;
    0x00A0E, samus_prev_hud_item_index, uint16, Scalar;
    0x00A11, "samus_prev_movement_type", uint8, Scalar;
    0x00A12, samus_prev_health_for_flash, uint16, Scalar;
    0x00A14, controller1_input_for_demo, uint16, Scalar;
    0x00A16, controller1_new_input_for_demo, uint16, Scalar;
    0x00A18, UNUSED_word_7E0A18, uint16, Scalar;
    0x00A1C, samus_pose, uint16, Scalar;
    0x00A1E, "samus_pose_x_dir", uint8, Scalar;
    0x00A1F, "samus_movement_type", uint8, Scalar;
    0x00A20, samus_prev_pose, uint16, Scalar;
    0x00A22, "samus_prev_pose_x_dir", uint8, Scalar;
    0x00A23, "samus_prev_movement_type2", uint8, Scalar;
    0x00A24, samus_last_different_pose, uint16, Scalar;
    0x00A26, "samus_last_different_pose_x_dir", uint8, Scalar;
    0x00A27, "samus_last_different_pose_movement_type", uint8, Scalar;
    0x00A28, samus_new_pose, uint16, Scalar;
    0x00A2A, samus_new_pose_interrupted, uint16, Scalar;
    0x00A2C, samus_new_pose_transitional, uint16, Scalar;
    0x00A2E, samus_momentum_routine_index, uint16, Scalar;
    0x00A30, samus_special_transgfx_index, uint16, Scalar;
    0x00A32, samus_hurt_switch_index, uint16, Scalar;
    0x00A34, solid_enemy_collision_flags, uint16, Scalar;
    0x00A36, block_collision_flags, uint16, Scalar;
    0x00A38, samus_space_to_move_up_blocks, uint16, Scalar;
    0x00A3A, samus_y_radius_diff, uint16, Scalar;
    0x00A3C, samus_space_to_move_down_blocks, uint16, Scalar;
    0x00A3E, samus_space_to_move_up_enemy, uint16, Scalar;
    0x00A40, samus_space_to_move_down_enemy, uint16, Scalar;
    0x00A42, frame_handler_alfa, uint16, Scalar;
    0x00A44, frame_handler_beta, uint16, Scalar;
    0x00A46, enable_horiz_slope_coll, uint16, Scalar;
    0x00A48, samus_hurt_flash_counter, uint16, Scalar;
    0x00A4A, samus_special_super_palette_flags, uint16, Scalar;
    0x00A4C, samus_subunit_health, uint16, Scalar;
    0x00A4E, samus_periodic_subdamage, uint16, Scalar;
    0x00A50, samus_periodic_damage, uint16, Scalar;
    0x00A52, knockback_dir, uint16, Scalar;
    0x00A54, knockback_x_dir, uint16, Scalar;
    0x00A56, bomb_jump_dir, uint16, Scalar;
    0x00A58, samus_movement_handler, uint16, Scalar;
    0x00A5A, frame_handler_gamma, uint16, Scalar;
    0x00A5C, samus_draw_handler, uint16, Scalar;
    0x00A5E, UNUSED_word_7E0A5E, uint16, Scalar;
    0x00A60, samus_input_handler, uint16, Scalar;
    0x00A62, samus_var62, uint16, Scalar;
    0x00A64, samus_grapple_flags, uint16, Scalar;
    0x00A66, samus_x_speed_divisor, uint16, Scalar;
    0x00A68, samus_shine_timer, uint16, Scalar;
    0x00A6A, samus_health_warning, uint16, Scalar;
    0x00A6C, samus_x_speed_table_pointer, uint16, Scalar;
    0x00A6E, samus_contact_damage_index, uint16, Scalar;
    0x00A72, samus_visor_palette_timer_index, uint16, Scalar;
    0x00A74, samus_suit_palette_index, uint16, Scalar;
    0x00A76, hyper_beam_flag, uint16, Scalar;
    0x00A78, time_is_frozen_flag, uint16, Scalar;
    0x00A7A, demo_input_pre_instr, uint16, Scalar;
    0x00A7C, demo_input_instr_timer, uint16, Scalar;
    0x00A7E, demo_input_instr_ptr, uint16, Scalar;
    0x00A80, demo_timer_counter, uint16, Scalar;
    0x00A82, xray_angle, uint16, Scalar;
    0x00A84, demo_input, uint16, Scalar;
    0x00A86, demo_input_new, uint16, Scalar;
    0x00A88, demo_enable, uint16, Scalar;
    0x00A88, "hdma_ptr_1", LongPtr, Scalar;
    0x00A8A, demo_num_input_frames, uint16, Scalar;
    0x00A8B, "hdma_ptr_2", LongPtr, Scalar;
    0x00A8C, demo_input_prev, uint16, Scalar;
    0x00A8E, demo_input_prev_new, uint16, Scalar;
    0x00A8E, "hdma_ptr_3", LongPtr, Scalar;
    0x00A90, demo_backup_prev_controller_input, uint16, Scalar;
    0x00A91, hdma_var_1, uint16, Scalar;
    0x00A92, demo_backup_prev_controller_input_new, uint16, Scalar;
    0x00A94, samus_anim_frame_timer, uint16, Scalar;
    0x00A96, samus_anim_frame, uint16, Scalar;
    0x00A9A, samus_anim_frame_skip, uint16, Scalar;
    0x00A9C, samus_anim_frame_buffer, uint16, Scalar;
    0x00A9E, grapple_walljump_timer, uint16, Scalar;
    0x00AA0, reached_ceres_elevator_fade_timer, uint16, Scalar;
    0x00AA2, timer_for_shinesparks_startstop, uint16, Scalar;
    0x00AA4, UNUSEDword_7E0AA4, uint16, Scalar;
    0x00AA6, "flag_arm_cannon_open_or_opening", uint8, Scalar;
    0x00AA7, "flag_arm_cannon_opening_or_closing", uint8, Scalar;
    0x00AA8, arm_cannon_frame, uint16, Scalar;
    0x00AAA, hud_item_changed_this_frame, uint16, Scalar;
    0x00AAC, arm_cannon_drawing_mode, uint16, Scalar;
    0x00AAE, speed_echoes_index, uint16, Scalar;
    0x00AB0, speed_echo_xpos, uint16, Array;
    0x00AB8, speed_echo_ypos, uint16, Array;
    0x00AC0, speed_echo_xspeed, uint16, Array;
    0x00AC8, samus_top_half_spritemap_index, uint16, Scalar;
    0x00ACA, samus_bottom_half_spritemap_index, uint16, Scalar;
    0x00ACC, timer_for_shine_timer, uint16, Scalar;
    0x00ACE, special_samus_palette_frame, uint16, Scalar;
    0x00AD0, special_samus_palette_timer, uint16, Scalar;
    0x00AD2, liquid_physics_type, uint16, Scalar;
    0x00AD4, atmospheric_gfx_anim_timer, uint16, Array;
    0x00ADC, atmospheric_gfx_x_pos, uint16, Array;
    0x00AE4, atmospheric_gfx_y_pos, uint16, Array;
    0x00AEC, atmospheric_gfx_frame_and_type, uint16, Array;
    0x00AF4, autojump_timer, uint16, Scalar;
    0x00AF6, samus_x_pos, uint16, Scalar;
    0x00AF8, samus_x_subpos, uint16, Scalar;
    0x00AFA, samus_y_pos, uint16, Scalar;
    0x00AFC, samus_y_subpos, uint16, Scalar;
    0x00AFE, samus_x_radius, uint16, Scalar;
    0x00B00, samus_y_radius, uint16, Scalar;
    0x00B02, samus_collision_direction, uint16, Scalar;
    0x00B04, samus_spritemap_x_pos, uint16, Scalar;
    0x00B06, samus_spritemap_y_pos, uint16, Scalar;
    0x00B0A, ideal_layer1_xpos, uint16, Scalar;
    0x00B0E, ideal_layer1_ypos, uint16, Scalar;
    0x00B10, samus_prev_x_pos, uint16, Scalar;
    0x00B12, samus_prev_x_subpos, uint16, Scalar;
    0x00B14, samus_prev_y_pos, uint16, Scalar;
    0x00B16, samus_prev_y_subpos, uint16, Scalar;
    0x00B18, charged_shot_glow_timer, uint16, Scalar;
    0x00B1A, UNUSED_word_7E0B1A, uint16, Scalar;
    0x00B20, used_for_ball_bounce_on_landing, uint16, Scalar;
    0x00B22, samus_is_falling_flag, uint16, Scalar;
    0x00B24, tempB24, uint16, Scalar;
    0x00B26, tempB26, uint16, Scalar;
    0x00B2A, UNUSED_word_7E0B2A, uint16, Scalar;
    0x00B2C, samus_y_subspeed, uint16, Scalar;
    0x00B2E, samus_y_speed, uint16, Scalar;
    0x00B32, samus_y_subaccel, uint16, Scalar;
    0x00B34, samus_y_accel, uint16, Scalar;
    0x00B36, samus_y_dir, uint16, Scalar;
    0x00B38, UNUSED_word_7E0B38, uint16, Scalar;
    0x00B3C, samus_has_momentum_flag, uint16, Scalar;
    0x00B3E, speed_boost_counter, uint16, Scalar;
    0x00B40, samus_echoes_sound_flag, uint16, Scalar;
    0x00B42, samus_x_extra_run_speed, uint16, Scalar;
    0x00B44, samus_x_extra_run_subspeed, uint16, Scalar;
    0x00B46, samus_x_base_speed, uint16, Scalar;
    0x00B48, samus_x_base_subspeed, uint16, Scalar;
    0x00B4A, samus_x_accel_mode, uint16, Scalar;
    0x00B4C, "samus_x_decel_mult", uint8, Scalar;
    0x00B56, extra_samus_x_subdisplacement, uint16, Scalar;
    0x00B58, extra_samus_x_displacement, uint16, Scalar;
    0x00B5A, extra_samus_y_subdisplacement, uint16, Scalar;
    0x00B5C, extra_samus_y_displacement, uint16, Scalar;
    0x00B5E, new_projectile_direction_changed_pose, uint16, Scalar;
    0x00B60, used_for_sba_attacksB60, uint16, Scalar;
    0x00B62, samus_charge_palette_index, uint16, Scalar;
    0x00B64, projectile_x_pos, uint16, Array;
    0x00B78, projectile_y_pos, uint16, Array;
    0x00B8C, projectile_bomb_x_subpos, uint16, Array;
    0x00BA0, projectile_bomb_y_subpos, uint16, Array;
    0x00BB4, projectile_x_radius, uint16, Array;
    0x00BC8, projectile_y_radius, uint16, Array;
    0x00BDC, projectile_bomb_x_speed, uint16, Array;
    0x00BF0, projectile_bomb_y_speed, uint16, Array;
    0x00C04, projectile_dir, uint16, Array;
    0x00C18, projectile_type, uint16, Array;
    0x00C2C, projectile_damage, uint16, Array;
    0x00C40, projectile_bomb_instruction_ptr, uint16, Array;
    0x00C54, projectile_bomb_instruction_timers, uint16, Array;
    0x00C68, projectile_bomb_pre_instructions, uint16, Array;
    0x00C72, bomb_functions, uint16, Array;
    0x00C7C, projectile_variables, uint16, Array;
    0x00C86, "bomb_timers", uint16, Array;
    0x00C90, projectile_timers, uint16, Array;
    0x00C9A, "bomb_subspeeds", uint16, Array;
    0x00CA4, projectile_unk_A, uint16, Array;
    0x00CB8, projectile_spritemap_pointers, uint16, Array;
    0x00CCC, cooldown_timer, uint16, Scalar;
    0x00CCE, projectile_counter, uint16, Scalar;
    0x00CD0, flare_counter, uint16, Scalar;
    0x00CD2, bomb_counter, uint16, Scalar;
    0x00CD4, bomb_spread_charge_timeout_counter, uint16, Scalar;
    0x00CD6, flare_animation_frame, uint16, Scalar;
    0x00CD8, flare_slow_sparks_anim_frame, uint16, Scalar;
    0x00CDA, flare_fast_sparks_anim_frame, uint16, Scalar;
    0x00CDC, flare_animation_timer, uint16, Scalar;
    0x00CDE, flare_slow_sparks_anim_timer, uint16, Scalar;
    0x00CE0, flare_fast_sparks_anim_timer, uint16, Scalar;
    0x00CE2, power_bomb_explosion_x_pos, uint16, Scalar;
    0x00CE4, power_bomb_explosion_y_pos, uint16, Scalar;
    0x00CE6, power_bomb_explosion_x_pos_plus_0x100, uint16, Scalar;
    0x00CE8, power_bomb_explosion_y_pos_rsub_0x1ff, uint16, Scalar;
    0x00CEA, power_bomb_explosion_radius, uint16, Scalar;
    0x00CEC, power_bomb_pre_explosion_flash_radius, uint16, Scalar;
    0x00CEE, power_bomb_flag, uint16, Scalar;
    0x00CF0, power_bomb_pre_explosion_radius_speed, uint16, Scalar;
    0x00CF2, pre_scaled_power_bomb_explosion_shape_def_ptr, uint16, Scalar;
    0x00CF4, grapple_beam_flags, uint16, Scalar;
    0x00CF6, grapple_varCF6, uint16, Scalar;
    0x00CF8, slow_grabble_scrolling_flag, uint16, Scalar;
    0x00CFA, grapple_beam_end_angle16, uint16, Scalar;
    0x00CFB, "grapple_beam_end_angle_hi", uint8, Scalar;
    0x00CFC, grapple_beam_end_angles_mirror, uint16, Scalar;
    0x00CFE, grapple_beam_length, uint16, Scalar;
    0x00D00, grapple_beam_length_delta, uint16, Scalar;
    0x00D02, grapple_beam_origin_x_offset, uint16, Scalar;
    0x00D04, grapple_beam_origin_y_offset, uint16, Scalar;
    0x00D06, grapple_beam_end_x_subpos, uint16, Scalar;
    0x00D08, grapple_beam_end_x_pos, uint16, Scalar;
    0x00D0A, grapple_beam_end_y_subpos, uint16, Scalar;
    0x00D0C, grapple_beam_end_y_pos, uint16, Scalar;
    0x00D0E, grapple_beam_end_x_suboffset, uint16, Scalar;
    0x00D10, grapple_beam_end_x_offset, uint16, Scalar;
    0x00D12, grapple_beam_end_y_suboffset, uint16, Scalar;
    0x00D14, grapple_beam_end_y_offset, uint16, Scalar;
    0x00D16, x_pos_of_start_of_grapple_beam, uint16, Scalar;
    0x00D18, y_pos_of_start_of_grapple_beam, uint16, Scalar;
    0x00D1A, x_pos_of_start_of_grapple_beam_prevframe, uint16, Scalar;
    0x00D1C, y_pos_of_start_of_grapple_beam_prevframe, uint16, Scalar;
    0x00D1E, grapple_beam_unkD1E, uint16, Scalar;
    0x00D20, grapple_beam_unkD20, uint16, Scalar;
    0x00D22, grapple_beam_extension_x_velocity, uint16, Scalar;
    0x00D24, grapple_beam_extension_y_velocity, uint16, Scalar;
    0x00D26, grapple_beam_unkD26, uint16, Scalar;
    0x00D28, grapple_beam_unkD28, uint16, Scalar;
    0x00D2A, grapple_beam_unkD2A, uint16, Scalar;
    0x00D2C, grapple_beam_unkD2C, uint16, Scalar;
    0x00D2E, grapple_beam_unkD2E, uint16, Scalar;
    0x00D30, grapple_beam_unkD30, uint16, Scalar;
    0x00D32, grapple_beam_function, uint16, Scalar;
    0x00D34, grapple_beam_direction, uint16, Scalar;
    0x00D36, grapple_beam_unkD36, uint16, Scalar;
    0x00D38, grapple_beam_unkD38, uint16, Scalar;
    0x00D3A, grapple_beam_unkD3A, uint16, Scalar;
    0x00D3C, grapple_beam_unkD3C, uint16, Scalar;
    0x00D3E, grapple_point_anim_timer, uint16, Scalar;
    0x00D40, grapple_point_anim_ptr, uint16, Scalar;
    0x00D42, grapple_segment_anim_instr_timers, uint16, Array;
    0x00D62, grapple_segment_anim_instr_ptrs, uint16, Array;
    0x00D82, button_config_shoot_x_saved, uint16, Scalar;
    0x00D84, button_config_jump_a_saved, uint16, Scalar;
    0x00D86, button_config_run_b_saved, uint16, Scalar;
    0x00D88, button_config_itemcancel_y_saved, uint16, Scalar;
    0x00D8A, button_config_itemswitch_saved, uint16, Scalar;
    0x00D8C, button_config_aim_down_L_saved, uint16, Scalar;
    0x00D8E, button_config_aim_up_R_saved, uint16, Scalar;
    0x00D90, grapple_beam_grapple_start_x, uint16, Scalar;
    0x00D92, grapple_beam_grapple_start_y, uint16, Scalar;
    0x00D94, grapple_beam_grapple_start_block_x, uint16, Scalar;
    0x00D96, grapple_beam_grapple_start_block_y, uint16, Scalar;
    0x00D98, g_word_7E0D98, uint16, Scalar;
    0x00D9C, g_word_7E0D9C, uint16, Scalar;
    0x00D9E, "g_byte_7E0D9E", uint8, Scalar;
    0x00D9F, "g_byte_7E0D9F", uint8, Scalar;
    0x00DA0, loop_counter_transfer_enemies_to_vram, uint16, Scalar;
    0x00DA2, absolute_moved_last_frame_x, uint16, Scalar;
    0x00DA4, absolute_moved_last_frame_x_fract, uint16, Scalar;
    0x00DA6, absolute_moved_last_frame_y, uint16, Scalar;
    0x00DA8, absolute_moved_last_frame_y_fract, uint16, Scalar;
    0x00DAA, projectile_init_speed_samus_moved_left, uint16, Scalar;
    0x00DAC, projectile_init_speed_samus_moved_left_fract, uint16, Scalar;
    0x00DAE, projectile_init_speed_samus_moved_right, uint16, Scalar;
    0x00DB0, projectile_init_speed_samus_moved_right_fract, uint16, Scalar;
    0x00DB2, projectile_init_speed_samus_moved_up, uint16, Scalar;
    0x00DB4, projectile_init_speed_samus_moved_up_fract, uint16, Scalar;
    0x00DB6, projectile_init_speed_samus_moved_down, uint16, Scalar;
    0x00DB8, projectile_init_speed_samus_moved_down_fract, uint16, Scalar;
    0x00DBA, samus_pos_adjusted_by_slope_flag, uint16, Scalar;
    0x00DBC, samus_total_x_speed, uint16, Scalar;
    0x00DBE, samus_total_x_subspeed, uint16, Scalar;
    0x00DC0, play_resume_charging_beam_sfx, uint16, Scalar;
    0x00DC2, prev_beam_charge_counter, uint16, Scalar;
    0x00DC4, cur_block_index, uint16, Scalar;
    0x00DC6, input_to_pose_calc, uint16, Scalar;
    0x00DC8, queued_message_box_index, uint16, Scalar;
    0x00DCE, samus_collides_with_solid_enemy, uint16, Scalar;
    0x00DD0, samus_collision_flag, uint16, Scalar;
    0x00DD2, REMOVED_temp_collision_DD2, uint16, Scalar;
    0x00DD4, REMOVED_temp_collision_DD4, uint16, Scalar;
    0x00DD6, REMOVED_temp_collision_DD6, uint16, Scalar;
    0x00DDC, suit_pickup_light_beam_widening_speed, uint16, Scalar;
    0x00DDE, projectile_index, uint16, Scalar;
    0x00DE0, debug_invincibility, uint16, Scalar;
    0x00DE2, game_options_screen_index, uint16, Scalar;
    0x00DE4, g_word_7E0DE4, uint16, Scalar;
    0x00DE6, g_word_7E0DE6, uint16, Scalar;
    0x00DE8, g_word_7E0DE8, uint16, Scalar;
    0x00DEA, which_item_to_pickup, uint16, Scalar;
    0x00DEC, substate, uint16, Scalar;
    0x00DEE, suit_pickup_light_beam_pos, uint16, Scalar;
    0x00DF0, "suit_pickup_color_math_R", uint8, Scalar;
    0x00DF1, "suit_pickup_color_math_G", uint8, Scalar;
    0x00DF2, "suit_pickup_color_math_B", uint8, Scalar;
    0x00DF3, "suit_pickup_palette_transition_color", uint8, Scalar;
    0x00DF4, UNUSED_debug_spare_cpu_display_flag, uint16, Scalar;
    0x00DF6, UNUSED_word_7E0DF6, uint16, Scalar;
    0x00DF8, UNUSED_word_7E0DF8, uint16, Scalar;
    0x00DFA, UNUSED_word_7E0DFA, uint16, Scalar;
    0x00DFC, UNUSED_word_7E0DFC, uint16, Scalar;
    0x00DFE, joypad1_input_samusfilter, uint16, Scalar;
    0x00E00, joypad1_newinput_samusfilter, uint16, Scalar;
    0x00E02, REMOVED_samus_bottom_boundary_position, uint16, Scalar;
    0x00E04, distance_to_eject_samus_left, uint16, Scalar;
    0x00E06, distance_to_eject_samus_right, uint16, Scalar;
    0x00E08, distance_to_eject_samus_up, uint16, Scalar;
    0x00E0A, distance_to_eject_samus_down, uint16, Scalar;
    0x00E0C, "g_byte_7E0E0C", uint8, Array;
    0x00E12, debug_disable_sprite_interact, uint16, Scalar;
    0x00E16, elevator_flags, uint16, Scalar;
    0x00E18, elevator_status, uint16, Scalar;
    0x00E1A, health_drop_bias_flag, uint16, Scalar;
    0x00E1C, enemy_index_to_shake, uint16, Scalar;
    0x00E1E, nmi_flag_bg2_enemy_vram_transfer, uint16, Scalar;
    0x00E20, REMOVED_varE20, uint16, Scalar;
    0x00E22, REMOVED_varE22, uint16, Scalar;
    0x00E24, REMOVED_varE24, uint16, Scalar;
    0x00E26, REMOVED_varE26, uint16, Scalar;
    0x00E28, REMOVED_varE28, uint16, Scalar;
    0x00E2A, REMOVED_varE2A, uint16, Scalar;
    0x00E2C, REMOVED_varE2C, uint16, Scalar;
    0x00E2E, REMOVED_varE2E, uint16, Scalar;
    0x00E32, REMOVED_varE32, uint16, Scalar;
    0x00E34, REMOVED_varE34, uint16, Scalar;
    0x00E36, REMOVED_varE36, uint16, Scalar;
    0x00E38, REMOVED_varE38, uint16, Scalar;
    0x00E3A, REMOVED_varE3A, uint16, Scalar;
    0x00E3C, REMOVED_varE3C, uint16, Scalar;
    0x00E3E, REMOVED_varE3E, uint16, Scalar;
    0x00E40, last_enemy_power, uint16, Scalar;
    0x00E44, random_enemy_counter, uint16, Scalar;
    0x00E46, UNUSED_word_7E0E46, uint16, Scalar;
    0x00E48, UNUSED_word_7E0E48, uint16, Scalar;
    0x00E4A, REMOVED_new_enemy_index, uint16, Scalar;
    0x00E4C, first_free_enemy_index, uint16, Scalar;
    0x00E4E, num_enemies_in_room, uint16, Scalar;
    0x00E50, num_enemies_killed_in_room, uint16, Scalar;
    0x00E52, num_enemy_deaths_left_to_clear, uint16, Scalar;
    0x00E54, cur_enemy_index, uint16, Scalar;
    0x00E56, REMOVED_cur_enemy_index_backup, uint16, Scalar;
    0x00E58, REMOVED_enemy_data_ptr, uint16, Scalar;
    0x00E5A, "enemy_tile_load_data", EnemyTileLoadData, Array;
    0x00E7A, enemy_tile_load_data_write_pos, uint16, Scalar;
    0x00E7C, enemy_tile_vram_src, uint16, Scalar;
    0x00E7E, enemy_tile_vram_dst, uint16, Scalar;
    0x00E84, enemy_drawing_queue, uint16, Array;
    0x00F68, enemy_drawing_queue_sizes, uint16, Array;
    0x00F78, "enemy_data", EnemyData, Array;
    0x01778, fireflea_flashing_timer, uint16, Scalar;
    0x0177A, fireflea_flashing_index, uint16, Scalar;
    0x0177C, UNUSED_word_7E177C, uint16, Scalar;
    0x0177E, fireflea_darkness_level, uint16, Scalar;
    0x01780, UNUSED_word_7E1780, uint16, Scalar;
    0x01782, UNUSED_word_7E1782, uint16, Scalar;
    0x01784, "REMOVED_enemy_ai_pointer", LongPtr, Scalar;
    0x01788, "REMOVED_enemy_ai_pointer_backup", LongPtr, Scalar;
    0x0178C, "enemy_gfx_drawn_hook", LongPtr, Scalar;
    0x01790, "REMOVED_set_to_rtl_when_loading_enemies_unused", LongPtr, Scalar;
    0x01794, variables_for_enemy_graphics_drawn_hook, uint16, Array;
    0x0179A, enemy_bg2_tilemap_size, uint16, Scalar;
    0x0179C, boss_id, uint16, Scalar;
    0x0179E, UNUSED_word_7E179E, uint16, Scalar;
    0x017A0, UNUSED_word_7E17A0, uint16, Scalar;
    0x017A2, UNUSED_word_7E17A2, uint16, Scalar;
    0x017A4, active_enemy_indexes_write_ptr, uint16, Scalar;
    0x017A6, interactive_enemy_indexes_write_ptr, uint16, Scalar;
    0x017A8, REMOVED_active_enemy_indexes_index, uint16, Scalar;
    0x017AA, REMOVED_interactive_enemy_indexes_index, uint16, Scalar;
    0x017AC, active_enemy_indexes, uint16, Array;
    0x017EC, interactive_enemy_indexes, uint16, Array;
    0x0182C, enemy_index_colliding_dirs, uint16, Array;
    0x01834, "REMOVED_distance_to_enemy_colliding_dirs", uint16, Array;
    0x0183C, flag_process_all_enemies, uint16, Scalar;
    0x0183E, earthquake_type, uint16, Scalar;
    0x01840, earthquake_timer, uint16, Scalar;
    0x01842, frame_counter_every_frame, uint16, Scalar;
    0x01844, sprite_object_index, uint16, Scalar;
    0x01846, debug_enemy_index_UNUSED, uint16, Scalar;
    0x01848, log_index_UNUSED, uint16, Scalar;
    0x0184A, REMOVED_samus_x_pos_colliding_solid, uint16, Scalar;
    0x0184C, REMOVED_samus_x_subpos_colliding_solid, uint16, Scalar;
    0x0184E, REMOVED_enemy_x_pos_colliding_solid, uint16, Scalar;
    0x01850, REMOVED_enemy_x_subpos_colliding_solid, uint16, Scalar;
    0x01852, REMOVED_samus_pos_delta_colliding_solid, uint16, Scalar;
    0x01854, REMOVED_samus_subpos_delta_colliding_solid, uint16, Scalar;
    0x01856, REMOVED_samus_y_pos_colliding_solid, uint16, Scalar;
    0x01858, REMOVED_samus_y_subpos_colliding_solid, uint16, Scalar;
    0x0185A, REMOVED_solid_enemy_collision_type, uint16, Scalar;
    0x0185C, debug_index, uint16, Scalar;
    0x0185E, debug_time_frozen_for_enemies, uint16, Scalar;
    0x01860, debug_text_cursor_x, uint16, Scalar;
    0x01862, debug_text_cursor_y, uint16, Scalar;
    0x01864, debug_enemy_set_entry_index, uint16, Scalar;
    0x01866, debug_enemy_population_pointer, uint16, Scalar;
    0x01868, debug_enemy_spawn_x_pos, uint16, Scalar;
    0x0186A, debug_enemy_spawn_y_pos, uint16, Scalar;
    0x0186E, REMOVED_enemy_spritemap_entry_pointer, uint16, Scalar;
    0x01870, REMOVED_enemy_left_border_collision, uint16, Scalar;
    0x01872, REMOVED_enemy_bottom_border_collision, uint16, Scalar;
    0x01874, REMOVED_enemy_right_border_collision, uint16, Scalar;
    0x01876, REMOVED_enemy_top_border_collision, uint16, Scalar;
    0x01878, REMOVED_hitbox_ptr, uint16, Scalar;
    0x0187A, REMOVED_enemy_spritemap_entry_coll_x_pos, uint16, Scalar;
    0x0187C, REMOVED_enemy_spritemap_entry_coll_y_pos, uint16, Scalar;
    0x0187E, REMOVED_samus_right_border_coll, uint16, Scalar;
    0x01880, REMOVED_samus_left_border_coll, uint16, Scalar;
    0x01882, REMOVED_samus_bottom_border_coll, uint16, Scalar;
    0x01884, REMOVED_samus_top_border_coll, uint16, Scalar;
    0x0189A, REMOVED_samus_target_x_pos, uint16, Scalar;
    0x0189C, REMOVED_samus_target_x_subpos, uint16, Scalar;
    0x0189E, REMOVED_samus_target_y_pos, uint16, Scalar;
    0x018A0, REMOVED_samus_target_y_subpos, uint16, Scalar;
    0x018A2, REMOVED_samus_x_radius_mirror, uint16, Scalar;
    0x018A4, REMOVED_samus_y_radius_mirror, uint16, Scalar;
    0x018A6, collision_detection_index, uint16, Scalar;
    0x018A8, samus_invincibility_timer, uint16, Scalar;
    0x018AA, samus_knockback_timer, uint16, Scalar;
    0x018AC, projectile_invincibility_timer, uint16, Scalar;
    0x018AE, flag_disable_projectile_interaction, uint16, Scalar;
    0x018B0, hdma_objects_enable_flag, uint16, Scalar;
    0x018B2, hdma_object_index, uint16, Scalar;
    0x018B4, hdma_object_channels_bitmask, uint16, Array;
    0x018C0, hdma_object_bank_slot, uint16, Array;
    0x018CC, hdma_object_instruction_list_pointers, uint16, Array;
    0x018D8, hdma_object_table_pointers, uint16, Array;
    0x018E4, hdma_object_instruction_timers, uint16, Array;
    0x018F0, hdma_object_pre_instructions, uint16, Array;
    0x018FC, hdma_object_pre_instruction_bank, uint16, Array;
    0x01908, hdma_object_timers, uint16, Array;
    0x01914, hdma_object_A, uint16, Array;
    0x01920, hdma_object_B, uint16, Array;
    0x0192C, hdma_object_C, uint16, Array;
    0x01938, hdma_object_D, uint16, Array;
    0x01944, hdma_object_E, uint16, Array;
    0x01950, hdma_object_F, uint16, Array;
    0x0195C, fx_y_subpos, uint16, Scalar;
    0x0195E, fx_y_pos, uint16, Scalar;
    0x01960, lava_acid_y_subpos, uint16, Scalar;
    0x01962, lava_acid_y_pos, uint16, Scalar;
    0x01964, fx_tilemap_ptr, uint16, Scalar;
    0x0196C, fx_rising_function_bank_88, uint16, Scalar;
    0x0196E, fx_type, uint16, Scalar;
    0x01970, fx_y_suboffset, uint16, Scalar;
    0x01972, fx_y_offset, uint16, Scalar;
    0x01974, tide_phase, uint16, Scalar;
    0x01976, fx_base_y_subpos, uint16, Scalar;
    0x01978, fx_base_y_pos, uint16, Scalar;
    0x0197A, fx_target_y_pos, uint16, Scalar;
    0x0197C, fx_y_vel, uint16, Scalar;
    0x0197E, fx_liquid_options, uint16, Scalar;
    0x01980, fx_timer, uint16, Scalar;
    0x01982, fx_layer_blending_config_a, uint16, Scalar;
    0x01984, fx_layer_blending_config_b, uint16, Scalar;
    0x01986, fx_layer_blending_config_c, uint16, Scalar;
    0x01988, phantom_related_layer_flag, uint16, Scalar;
    0x0198D, cinematic_var5, uint16, Scalar;
    0x0198D, eproj_enable_flag, uint16, Scalar;
    0x0198F, cinematic_var6, uint16, Scalar;
    0x0198F, eproj_unk198F, uint16, Scalar;
    0x01991, cinematic_var7, uint16, Scalar;
    0x01991, eproj_index, uint16, Scalar;
    0x01993, cinematic_var8, uint16, Scalar;
    0x01993, eproj_init_param_1, uint16, Scalar;
    0x01995, cinematic_var9, uint16, Scalar;
    0x01995, eproj_unk1995, uint16, Scalar;
    0x01997, cinematic_var10, uint16, Scalar;
    0x01997, eproj_id, uint16, Array;
    0x01999, g_word_7E1999, uint16, Scalar;
    0x0199B, g_word_7E199B, uint16, Scalar;
    0x0199D, g_word_7E199D, uint16, Scalar;
    0x0199F, g_word_7E199F, uint16, Scalar;
    0x019A1, mode7_obj_instr_ptr, uint16, Array;
    0x019A5, mode7_obj_preinstr_func, uint16, Array;
    0x019A9, mode7_obj_instr_timer, uint16, Array;
    0x019AD, mode7_obj_goto_timer, uint16, Array;
    0x019B1, mode7_cur_index, uint16, Scalar;
    0x019B3, mode7_spawn_param, uint16, Scalar;
    0x019B5, cinematicbg_arr1, uint16, Array;
    0x019BB, eproj_gfx_idx, uint16, Array;
    0x019CD, cinematicbg_instr_ptr, uint16, Array;
    0x019D5, cinematicbg_preinstr, uint16, Array;
    0x019DD, cinematicbg_instr_timer, uint16, Array;
    0x019DF, eproj_timers, uint16, Array;
    0x019E5, cinematicbg_arr6, uint16, Array;
    0x019ED, cinematicbg_var2, uint16, Scalar;
    0x019F1, cinematic_enable_objs, uint16, Scalar;
    0x019F3, cinematic_enable_bg_tilemap, uint16, Scalar;
    0x019F5, cinematicbg_var3, uint16, Scalar;
    0x019F7, cinematic_var21, uint16, Scalar;
    0x019F9, cinematic_var22, uint16, Scalar;
    0x019FB, cinematic_var23, uint16, Scalar;
    0x019FD, cinematic_var24, uint16, Scalar;
    0x019FF, cinematic_var25, uint16, Scalar;
    0x01A01, cinematic_var26, uint16, Scalar;
    0x01A03, eproj_pre_instr, uint16, Array;
    0x01A03, "eproj_pre_instr", uint16, Array;
    0x01A27, eproj_x_subpos, uint16, Array;
    0x01A37, cinematic_var20, uint16, Scalar;
    0x01A47, cinematic_var19, uint16, Scalar;
    0x01A49, cinematic_var4, uint16, Scalar;
    0x01A4B, cinematic_var13, uint16, Scalar;
    0x01A4B, eproj_x_pos, uint16, Array;
    0x01A4D, cinematic_var14, uint16, Scalar;
    0x01A4F, cinematic_var17, uint16, Scalar;
    0x01A51, cinematic_var12, uint16, Scalar;
    0x01A53, cinematic_var18, uint16, Scalar;
    0x01A57, cinematic_var15, uint16, Scalar;
    0x01A59, cinematic_obj_index, uint16, Scalar;
    0x01A5D, cinematicspr_whattodraw, uint16, Array;
    0x01A6F, eproj_y_subpos, uint16, Array;
    0x01A7D, cinematicbg_arr7, uint16, Array;
    0x01A8F, optionsmenu_index, uint16, Scalar;
    0x01A93, eproj_y_pos, uint16, Array;
    0x01A93, options_menu_init_param, uint16, Scalar;
    0x01A9D, cinematicbg_arr8, uint16, Array;
    0x01A9D, "optionsmenu_cur_data", uint16, Array;
    0x01AAD, optionsmenu_arr8, uint16, Array;
    0x01AB7, eproj_x_vel, uint16, Array;
    0x01ABD, cinematicbg_arr9, uint16, Array;
    0x01ABD, "optionsmenu_arr9", uint16, Array;
    0x01ACD, optionsmenu_arr10, uint16, Array;
    0x01ADB, eproj_y_vel, uint16, Array;
    0x01ADD, cinematicspr_arr6, uint16, Array;
    0x01ADD, "optionsmenu_arr6", uint16, Array;
    0x01AED, optionsmenu_arr7, uint16, Array;
    0x01AFD, cinematicspr_arr7, uint16, Array;
    0x01AFD, "optionsmenu_instr_ptr", uint16, Array;
    0x01AFF, eproj_E, uint16, Array;
    0x01B0D, optionsmenu_arr1, uint16, Array;
    0x01B1D, cinematicspr_instr_ptr, uint16, Array;
    0x01B1D, "optionsmenu_instr_timer", uint16, Array;
    0x01B23, eproj_F, uint16, Array;
    0x01B2D, optionsmenu_arr5, uint16, Array;
    0x01B3D, cinematicspr_preinstr_func, uint16, Array;
    0x01B3D, "optionsmenu_arr11", uint16, Array;
    0x01B47, eproj_instr_list_ptr, uint16, Array;
    0x01B5D, cinematicspr_instr_timer, uint16, Array;
    0x01B6B, eproj_spritemap_ptr, uint16, Array;
    0x01B7D, cinematicspr_goto_timer, uint16, Array;
    0x01B8F, eproj_instr_timers, uint16, Array;
    0x01B9D, cinematic_spawn_param, uint16, Scalar;
    0x01B9F, cinematic_var11, uint16, Scalar;
    0x01BA1, cinematicbg_var1, uint16, Scalar;
    0x01BA3, cinematic_var16, uint16, Scalar;
    0x01BB3, eproj_radius, uint16, Array;
    0x01BD7, eproj_properties, uint16, Array;
    0x01BFB, eproj_G, uint16, Array;
    0x01C1F, message_box_index, uint16, Scalar;
    0x01C23, plm_flag, uint16, Scalar;
    0x01C25, plm_draw_tilemap_index, uint16, Scalar;
    0x01C27, plm_id, uint16, Scalar;
    0x01C29, plm_x_block, uint16, Scalar;
    0x01C2B, plm_y_block, uint16, Scalar;
    0x01C2D, plm_item_gfx_index, uint16, Scalar;
    0x01C2F, plm_item_gfx_ptrs, uint16, Array;
    0x01C37, plm_header_ptr, uint16, Array;
    0x01C87, plm_block_indices, uint16, Array;
    0x01CD7, plm_pre_instrs, uint16, Array;
    0x01D27, plm_instr_list_ptrs, uint16, Array;
    0x01D77, plm_timers, uint16, Array;
    0x01DC7, plm_room_arguments, uint16, Array;
    0x01E17, plm_variable, uint16, Array;
    0x01E67, custom_draw_instr_num_blocks, uint16, Scalar;
    0x01E69, custom_draw_instr_plm_block, uint16, Scalar;
    0x01E6B, custom_draw_instr_zero_terminator, uint16, Scalar;
    0x01E6D, tourian_entrance_statue_finished, uint16, Scalar;
    0x01E6F, tourian_entrance_statue_animstate, uint16, Scalar;
    0x01E71, flag_samus_in_quicksand, uint16, Scalar;
    0x01E73, inside_block_reaction_samus_point, uint16, Scalar;
    0x01E75, save_station_lockout_flag, uint16, Scalar;
    0x01E77, current_slope_bts, uint16, Scalar;
    0x01E79, flag_for_palette_fx_objects, uint16, Scalar;
    0x01E7B, palettefx_index, uint16, Scalar;
    0x01E7D, palettefx_ids, uint16, Array;
    0x01E8D, palettefx_color_indexes, uint16, Array;
    0x01EAD, palettefx_pre_instr, uint16, Array;
    0x01EBD, palettefx_instr_list_ptrs, uint16, Array;
    0x01ECD, palettefx_instr_timers, uint16, Array;
    0x01EDD, palettefx_timers, uint16, Array;
    0x01EED, samus_in_heat_palfx_index, uint16, Scalar;
    0x01EEF, samus_in_heat_palettefx_prev_index, uint16, Scalar;
    0x01EF1, animtiles_enable_flag, uint16, Scalar;
    0x01EF3, animtiles_object_index, uint16, Scalar;
    0x01EF5, animtiles_ids, uint16, Array;
    0x01F01, animtiles_timers, uint16, Array;
    0x01F0D, animtiles_instr_list_ptrs, uint16, Array;
    0x01F19, animtiles_instr_timers, uint16, Array;
    0x01F25, animtiles_src_ptr, uint16, Array;
    0x01F31, animtiles_sizes, uint16, Array;
    0x01F3D, animtiles_vram_ptr, uint16, Array;
    0x01F49, animtiles_instruction, uint16, Scalar;
    0x01F51, cinematic_function, uint16, Scalar;
    0x01F53, demo_timer, uint16, Scalar;
    0x01F55, demo_set, uint16, Scalar;
    0x01F57, demo_scene, uint16, Scalar;
    0x01F59, num_demo_sets, uint16, Scalar;
    0x02000, tilemap_stuff, uint16, Array;
    0x03000, "ram3000", Ram3000, Scalar;
    0x03800, "ram3800", Ram3800, Scalar;
    0x04000, "ram4000", Ram4000, Scalar;
    0x07000, "enemy_spawn_data", EnemySpawnData, Array;
    0x07800, "enemy_ram7800", ExtraEnemyRam7800, Array;
    0x08000, "extra_enemy_ram8000", ExtraEnemyRam8000, Array;
    0x08800, "extra_enemy_ram8800", ExtraEnemyRam8800, Array;
    0x09000, kraid_unk9000, uint16, Scalar;
    0x09002, g_word_7E9002, uint16, Scalar;
    0x09004, g_word_7E9004, uint16, Scalar;
    0x09006, g_word_7E9006, uint16, Scalar;
    0x09008, g_word_7E9008, uint16, Scalar;
    0x0900A, g_word_7E900A, uint16, Scalar;
    0x0900C, g_word_7E900C, uint16, Scalar;
    0x0900F, g_word_7E900F, uint16, Scalar;
    0x09012, g_word_7E9012, uint16, Scalar;
    0x09015, g_word_7E9015, uint16, Scalar;
    0x09018, g_word_7E9018, uint16, Scalar;
    0x09020, g_word_7E9020, uint16, Scalar;
    0x09022, g_word_7E9022, uint16, Scalar;
    0x09030, g_word_7E9030, uint16, Scalar;
    0x09032, g_word_7E9032, uint16, Scalar;
    0x09040, g_word_7E9040, uint16, Scalar;
    0x09042, g_word_7E9042, uint16, Scalar;
    0x09050, g_word_7E9050, uint16, Scalar;
    0x09052, g_word_7E9052, uint16, Scalar;
    0x09060, g_word_7E9060, uint16, Scalar;
    0x09062, g_word_7E9062, uint16, Scalar;
    0x09070, g_word_7E9070, uint16, Scalar;
    0x09072, g_word_7E9072, uint16, Scalar;
    0x09080, g_word_7E9080, uint16, Scalar;
    0x09082, g_word_7E9082, uint16, Scalar;
    0x09090, g_word_7E9090, uint16, Scalar;
    0x09092, g_word_7E9092, uint16, Scalar;
    0x090A0, g_word_7E90A0, uint16, Scalar;
    0x090A2, g_word_7E90A2, uint16, Scalar;
    0x090B0, g_word_7E90B0, uint16, Scalar;
    0x090B2, g_word_7E90B2, uint16, Scalar;
    0x090C0, g_word_7E90C0, uint16, Scalar;
    0x090C2, g_word_7E90C2, uint16, Scalar;
    0x090D0, g_word_7E90D0, uint16, Scalar;
    0x090D2, g_word_7E90D2, uint16, Scalar;
    0x09100, g_word_7E9100, uint16, Scalar;
    0x09140, g_word_7E9140, uint16, Scalar;
    0x09180, g_word_7E9180, uint16, Scalar;
    0x09500, "g_byte_7E9500", uint8, Scalar;
    0x09510, "g_byte_7E9510", uint8, Scalar;
    0x09520, "g_byte_7E9520", uint8, Scalar;
    0x09530, "g_byte_7E9530", uint8, Scalar;
    0x09540, "g_byte_7E9540", uint8, Scalar;
    0x09550, "g_byte_7E9550", uint8, Scalar;
    0x09560, "g_byte_7E9560", uint8, Scalar;
    0x09570, "g_byte_7E9570", uint8, Scalar;
    0x09580, "g_byte_7E9580", uint8, Scalar;
    0x09590, "g_byte_7E9590", uint8, Scalar;
    0x095A0, "g_byte_7E95A0", uint8, Scalar;
    0x095B0, "g_byte_7E95B0", uint8, Scalar;
    0x095C0, "g_byte_7E95C0", uint8, Scalar;
    0x095D0, "g_byte_7E95D0", uint8, Scalar;
    0x095E0, "g_byte_7E95E0", uint8, Scalar;
    0x095F0, "g_byte_7E95F0", uint8, Scalar;
    0x09600, "g_byte_7E9600", uint8, Scalar;
    0x09610, "g_byte_7E9610", uint8, Scalar;
    0x097DC, g_word_7E97DC, uint16, Array;
    0x09800, hdma_table_1, uint16, Array;
    0x09C00, "mother_brain_indirect_hdma", uint8, Array;
    0x09C44, g_word_7E9C44, uint16, Scalar;
    0x09C46, g_word_7E9C46, uint16, Array;
    0x09D00, hdma_table_2, uint16, Array;
    0x09E00, "hdma_window_1_left_pos", ExpandingSquareTransitionHdma, Array;
    0x09E10, "hdma_window_1_right_pos", ExpandingSquareTransitionHdma, Array;
    0x09E20, expand_sq_topbottom_margin_right_pos, uint16, Scalar;
    0x09E22, expand_sq_topbottom_margin_left_pos, uint16, Scalar;
    0x09E30, expand_sq_left_subpos, uint16, Scalar;
    0x09E32, expand_sq_left_pos, uint16, Scalar;
    0x09E34, expand_sq_right_subpos, uint16, Scalar;
    0x09E36, expand_sq_right_pos, uint16, Scalar;
    0x09E38, expand_sq_top_subpos, uint16, Scalar;
    0x09E3A, expand_sq_top_pos, uint16, Scalar;
    0x09E3C, expand_sq_bottom_subpos, uint16, Scalar;
    0x09E3E, expand_sq_bottom_pos, uint16, Scalar;
    0x09E40, expand_sq_left_subvel, uint16, Scalar;
    0x09E42, expand_sq_left_vel, uint16, Scalar;
    0x09E44, expand_sq_right_subvel, uint16, Scalar;
    0x09E46, expand_sq_right_vel, uint16, Scalar;
    0x09E48, expand_sq_top_subvel, uint16, Scalar;
    0x09E4A, expand_sq_top_vel, uint16, Scalar;
    0x09E4C, expand_sq_bottom_subvel, uint16, Scalar;
    0x09E4E, expand_sq_bottom_vel, uint16, Scalar;
    0x09E50, expand_sq_timer, uint16, Scalar;
    0x09E80, g_word_7E9E80, uint16, Array;
    0x09F00, "scrolling_sky_bg2_indirect_hdma", uint8, Array;
    0x09F80, scrolling_sky_bg2_hdma_data, uint16, Array;
    0x0A000, "tile_table", TileTables, Scalar;
    0x0C000, palette_buffer, uint16, Array;
    0x0C200, target_palettes, uint16, Array;
    0x0C400, palette_change_num, uint16, Scalar;
    0x0C402, palette_change_denom, uint16, Scalar;
    0x0C404, g_word_7EC404, uint16, Scalar;
    0x0C406, "power_bomb_explosion_left_hdma", uint8, Array;
    0x0C506, "power_bomb_explosion_right_hdma", uint8, Array;
    0x0C606, "offscreen_power_bomb_left_hdma", uint8, Scalar;
    0x0C607, "offscreen_power_bomb_right_hdma", uint8, Scalar;
    0x0C608, hud_tilemap, uint16, Array;
    0x0C6C8, "plm_draw_tilemap", uint16, Array;
    0x0C8C8, bg1_column_update_tilemap_left_halves, uint16, Array;
    0x0C908, bg1_column_update_tilemap_right_halves, uint16, Array;
    0x0C948, bg1_column_update_tilemap_top_halves, uint16, Array;
    0x0C98C, bg1_column_update_tilemap_bottom_halves, uint16, Array;
    0x0C9D0, "bg2_column_update_tilemap_left_halves", uint8, Array;
    0x0CA10, "bg2_column_update_tilemap_right_halves", uint16, Array;
    0x0CA50, "bg2_column_update_tilemap_top_halves", uint8, Array;
    0x0CA94, "bg2_column_update_tilemap_bottom_halves", uint8, Array;
    0x0CAD8, hud_bg3_xpos, uint16, Scalar;
    0x0CADA, hud_bg3_ypos, uint16, Scalar;
    0x0CADC, bg3_xpos, uint16, Scalar;
    0x0CADE, bg3_ypos, uint16, Scalar;
    0x0CAE0, "crocomire_bg2_scroll_hdma_indirect", uint8, Array;
    0x0CAF0, crocomire_bg2_scroll_hdma_data, uint16, Array;
    0x0CD20, "scrolls", uint8, Array;
    0x0CD52, explored_map_tiles_saved, uint16, Array;
    0x0D552, "debug_enemy_set_name", uint8, Array;
    0x0D55A, "debug_enemy_set_data", uint8, Array;
    0x0D652, wrecked_ship_robot_palanim_palindex, uint16, Scalar;
    0x0D654, wrecked_ship_robot_palanim_timer, uint16, Scalar;
    0x0D656, "wrecked_ship_robot_palanim_table_index", uint8, Scalar;
    0x0D658, projectiletrail_left_instr_timer, uint16, Array;
    0x0D67C, projectiletrail_right_instr_timer, uint16, Array;
    0x0D6A0, projectiletrail_left_instr_list_ptr, uint16, Array;
    0x0D6C4, projectiletrail_right_instr_list_ptr, uint16, Array;
    0x0D6E8, projectiletrail_left_tile_and_attribs, uint16, Array;
    0x0D70C, projectiletrail_right_tile_and_attribs, uint16, Array;
    0x0D730, projectiletrail_left_x_pos, uint16, Array;
    0x0D754, projectiletrail_right_x_pos, uint16, Array;
    0x0D778, projectiletrail_left_y_pos, uint16, Array;
    0x0D79C, projectiletrail_right_y_pos, uint16, Array;
    0x0D7C0, player_data_saved, uint16, Array;
    0x0D820, "events_that_happened", uint8, Array;
    0x0D828, "boss_bits_for_area", uint8, Array;
    0x0D830, "room_chozo_bits", uint8, Array;
    0x0D870, "item_bit_array", uint8, Array;
    0x0D8B0, "opened_door_bit_array", uint8, Array;
    0x0D8F0, UNUSED_word_7ED8F0, uint16, Array;
    0x0D8F8, "used_save_stations_and_elevators", uint8, Array;
    0x0D908, "map_station_byte_array", uint8, Array;
    0x0D914, loading_game_state, uint16, Scalar;
    0x0D916, sram_save_station_index, uint16, Scalar;
    0x0D918, sram_area_index, uint16, Scalar;
    0x0D91A, global_number_of_items_loaded_ctr, uint16, Scalar;
    0x0D91C, "compressed_map_data", uint8, Array;
    0x0DE1C, plm_instruction_timer, uint16, Array;
    0x0DE6C, plm_instruction_draw_ptr, uint16, Array;
    0x0DEBC, plm_instruction_list_link_reg, uint16, Array;
    0x0DF0C, plm_variables, uint16, Array;
    0x0DF5C, "backup_of_bg2_during_pause", uint16, Array;
    0x0EF5C, enemy_def_ptr, uint16, Array;
    0x0EF64, enemy_gfxdata_tiles_index, uint16, Array;
    0x0EF6C, enemy_gfxdata_vram_ptr, uint16, Array;
    0x0EF74, REMOVED_enemy_gfx_data_write_ptr, uint16, Scalar;
    0x0EF76, REMOVED_next_enemy_tiles_index, uint16, Scalar;
    0x0EF78, sprite_instr_list_ptrs, uint16, Array;
    0x0EFF8, sprite_instr_timer, uint16, Array;
    0x0F078, sprite_palettes, uint16, Array;
    0x0F0F8, sprite_x_pos, uint16, Array;
    0x0F178, sprite_x_subpos, uint16, Array;
    0x0F1F8, sprite_y_pos, uint16, Array;
    0x0F278, sprite_y_subpos, uint16, Array;
    0x0F2F8, sprite_disable_flag, uint16, Array;
    0x0F378, enemy_processing_stage, uint16, Scalar;
    0x0F380, eproj_flags, uint16, Array;
    0x0F3C8, eproj_enemy_header_ptr, uint16, Array;
    0x0F410, eproj_killed_enemy_index, uint16, Array;
    0x0F434, special_death_item_drop_x_origin_pos, uint16, Scalar;
    0x0F436, special_death_item_drop_y_origin_pos, uint16, Scalar;
    0x10000, ram7F_start, uint16, Scalar;
    0x10002, level_data, uint16, Array;
    0x16402, "BTS", uint8, Array;
    0x19602, custom_background, uint16, Array;
    0x1FF00, bug_fix_counter_BAD, uint16, Scalar;
}

// Declare typed views of RAM without changing the getter's borrow or pointer type.
// Each list retains successive byte offsets, including the original integer casts.
macro_rules! ram_pointer_functions {
    ($($name:ident($index:ident: $index_type:ty) -> $target:ty = [$($offset:expr),+ $(,)?];)*) => {
        $(
            #[inline]
            unsafe fn $name($index: $index_type, game: &mut $crate::game_state::Game) -> *mut $target {
                (&raw mut game.g_ram as *mut ::core::primitive::u8)
                    $(.offset($offset))+ as *mut $target
            }
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
