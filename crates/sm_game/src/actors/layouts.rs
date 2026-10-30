//! Byte-compatible enemy slot layouts over $7E:0F78.
// Enemy overlays share byte-compatible layouts; named aliases identify their users.
// Field numbers and explicit padding retain their original SNES RAM offsets.
macro_rules! enemy_layout {
    ($name:ident {
        $base:ident: $base_type:ty,
        words: [$($head:ident),* $(,)?],
        $(
            $array:ident: [$element:ty; $length:expr],
            words: [$($word:ident),* $(,)?],
        )*
    }) => {
        #[derive(Copy, Clone)]
        #[repr(C)]
        pub struct $name {
            pub $base: $base_type,
            $(pub $head: ::core::primitive::u16,)*
            $(
                pub $array: [$element; $length],
                $(pub $word: ::core::primitive::u16,)*
            )*
        }
    };
}

/// A role's typed view of an enemy slot: the bytes of `$layout`, with the words
/// that role uses named and the others keeping their `var_` names. The size and
/// every array's offset are checked against the layout at compile time.
macro_rules! part_view {
    ($(#[$doc:meta])* $name:ident: $layout:ident {
        words: [$($head:ident),* $(,)?],
        $(
            $array:ident: [$element:ty; $length:expr],
            words: [$($word:ident),* $(,)?],
        )*
    }) => {
        $(#[$doc])*
        #[derive(Clone, Copy)]
        #[repr(C)]
        pub struct $name {
            pub base: $crate::actors::layouts::EnemyBase,
            $(pub $head: ::core::primitive::u16,)*
            $(
                pub $array: [$element; $length],
                $(pub $word: ::core::primitive::u16,)*
            )*
        }
        const _: () = {
            use $crate::actors::layouts::$layout as Layout;
            $(assert!(
                ::core::mem::offset_of!($name, $array) == ::core::mem::offset_of!(Layout, $array)
            );)*
            assert!(::core::mem::size_of::<$name>() == ::core::mem::size_of::<Layout>());
        };
    };
}
pub(crate) use part_view;

/// Checks a hand-written view of an enemy slot: its size and base, and each
/// named field's offset, against the layout word it names. Unused words are
/// the view's private padding.
macro_rules! same_words {
    ($view:ident: $layout:ident { $($field:ident = $word:ident),* $(,)? }) => {
        const _: () = {
            use ::core::mem::{offset_of, size_of};
            use crate::actors::layouts::$layout as Layout;
            assert!(size_of::<$view>() == size_of::<Layout>());
            assert!(offset_of!($view, base) == offset_of!(Layout, base));
            $(assert!(offset_of!($view, $field) == offset_of!(Layout, $word));)*
        };
    };
}
pub(crate) use same_words;

/// The common enemy slot fields at $7E:0F78 + slot; every layout starts with it.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyBase {
    pub enemy_ptr: u16,
    pub x_pos: u16,
    pub x_subpos: u16,
    pub y_pos: u16,
    pub y_subpos: u16,
    pub x_width: u16,
    pub y_height: u16,
    pub properties: u16,
    pub extra_properties: u16,
    pub ai_handler_bits: u16,
    pub health: u16,
    pub spritemap_pointer: u16,
    pub timer: u16,
    pub current_instruction: u16,
    pub instruction_timer: u16,
    pub palette_index: u16,
    pub vram_tiles_index: u16,
    pub layer: u16,
    pub flash_timer: u16,
    pub frozen_timer: u16,
    pub invincibility_timer: u16,
    pub shake_timer: u16,
    pub frame_counter: u16,
    pub bank: u8,
    pub field_2f: u8,
}

enemy_layout! {
    EnemyLayout104 {
        base: EnemyBase,
        words: [
            var_a, var_b, var_c, var_d, var_e,
            var_f, parameter_1, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, var_03,
            var_04, var_05, var_06, var_07,
            var_08, var_09, var_0a, var_0b,
            var_0c, var_0d, var_0e, var_0f,
            var_10, var_11, var_12, var_13,
            var_14, var_15, var_16, var_17,
            var_18, var_19, var_1a, var_1b,
            var_1c, var_1d, var_1e, var_1f,
        ],
        pad2: [u8; 1984],
        words: [
            var_20, var_21, var_22, var_23,
            var_24, var_25, var_26, var_27,
            var_28, var_29, var_2a, var_2b,
            var_2c, var_2d, var_2e, var_2f,
            var_30, var_31, var_32, var_33,
            var_34, var_35, var_36, var_37,
            var_38, var_39, var_3a, var_3b,
            var_3c, var_3d, var_3e, var_3f,
        ],
        pad3: [u8; 1984],
        words: [
            var_40, var_41, var_42, var_43,
            var_44, var_45, var_46, var_47,
            var_48, var_49, var_4a, var_4b,
            var_4c, var_4d, var_4e, var_4f,
            var_50, var_51, var_52, var_53,
            var_54, var_55, var_56, var_57,
            var_58, var_59, var_5a, var_5b,
            var_5c, var_5d, var_5e, var_5f,
        ],
    }
}

enemy_layout! {
    EnemyLayout72 {
        base: EnemyBase,
        words: [
            var_a, var_b, var_c, var_d, var_e, var_f,
            parameter_1, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, var_03, var_04, var_05,
            var_06, var_07, var_08, var_09, var_0a, var_0b,
            var_0c, var_0d, var_0e, var_0f, var_10, var_11,
            var_12, var_13, var_14, var_15, var_16, var_17,
            var_18, var_19, var_1a, var_1b, var_1c, var_1d,
            var_1e, var_1f,
        ],
        pad2: [u8; 1984],
        words: [
            var_20, var_21, var_22, var_23, var_24, var_25,
            var_26, var_27, var_28, var_29, var_2a, var_2b,
            var_2c, var_2d, var_2e, var_2f, var_30, var_31,
            var_32, var_33, var_34, var_35, var_36, var_37,
            var_38, var_39, var_3a, var_3b, var_3c, var_3d,
            var_3e, var_3f,
        ],
    }
}

enemy_layout! {
    EnemyLayout40 {
        base: EnemyBase,
        words: [
            var_a, var_b, var_c, var_d, var_e, var_f,
            parameter_1, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, var_03, var_04, var_05,
            var_06, var_07, var_08, var_09, var_0a, var_0b,
            var_0c, var_0d, var_0e, var_0f, var_10, var_11,
            var_12, var_13, var_14, var_15, var_16, var_17,
            var_18, var_19, var_1a, var_1b, var_1c, var_1d,
            var_1e, var_1f,
        ],
    }
}

enemy_layout! {
    EnemyLayout8 {
        base: EnemyBase,
        words: [
            var_a, var_b, var_c, var_d, var_e, var_f,
            parameter_1, parameter_2,
        ],
    }
}

enemy_layout! {
    EnemyLayout72Late {
        base: EnemyBase,
        words: [
            var_a, var_b, var_c, var_d, var_e, var_f,
            parameter_1, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, var_03, var_04, var_05,
            var_06, var_07, var_08, var_09, var_0a, var_0b,
            var_0c, var_0d, var_0e, var_0f, var_10, var_11,
            var_12, var_13, var_14, var_15, var_16, var_17,
            var_18, var_19, var_1a, var_1b, var_1c, var_1d,
            var_1e, var_1f,
        ],
        pad3: [u8; 4032],
        words: [
            var_40, var_41, var_42, var_43, var_44, var_45,
            var_46, var_47, var_48, var_49, var_4a, var_4b,
            var_4c, var_4d, var_4e, var_4f, var_50, var_51,
            var_52, var_53, var_54, var_55, var_56, var_57,
            var_58, var_59, var_5a, var_5b, var_5c, var_5d,
            var_5e, var_5f,
        ],
    }
}

enemy_layout! {
    EnemyLayout40Late {
        base: EnemyBase,
        words: [
            var_a, var_b, var_c, var_d, var_e, var_f,
            parameter_1, parameter_2,
        ],
        pad2: [u8; 28744],
        words: [
            var_20, var_21, var_22, var_23, var_24, var_25,
            var_26, var_27, var_28, var_29, var_2a, var_2b,
            var_2c, var_2d, var_2e, var_2f, var_30, var_31,
            var_32, var_33, var_34, var_35, var_36, var_37,
            var_38, var_39, var_3a, var_3b, var_3c, var_3d,
            var_3e, var_3f,
        ],
    }
}

// Kraid and his parts; field names follow the bank-log RAM map.
enemy_layout! {
    KraidLayout {
        base: EnemyBase,
        words: [
            kraid_var_a, kraid_var_b, kraid_var_c, kraid_var_d, kraid_var_e, kraid_var_f,
            kraid_parameter_1, kraid_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            kraid_next, field_2, field_4, kraid_thinking, kraid_min_y_pos_eject,
            kraid_mouth_flags,
        ],
        kraid_healths_8ths: [u16; 8],
        words: [field_1c, kraid_target_x],
        kraid_healths_4ths: [u16; 4],
        words: [field_28, kraid_hurt_frame, kraid_hurt_frame_timer],
        field_2e: [u16; 9],
        words: [],
    }
}
