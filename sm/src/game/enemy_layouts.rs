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

pub(crate) use enemy_layout;

use crate::types::EnemyBase;

enemy_layout! {
    EnemyLayout104 {
        base: EnemyBase,
        words: [
            var_A, var_B, var_C, var_D, var_E,
            var_F, parameter_1, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, var_03,
            var_04, var_05, var_06, var_07,
            var_08, var_09, var_0A, var_0B,
            var_0C, var_0D, var_0E, var_0F,
            var_10, var_11, var_12, var_13,
            var_14, var_15, var_16, var_17,
            var_18, var_19, var_1A, var_1B,
            var_1C, var_1D, var_1E, var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            var_20, var_21, var_22, var_23,
            var_24, var_25, var_26, var_27,
            var_28, var_29, var_2A, var_2B,
            var_2C, var_2D, var_2E, var_2F,
            var_30, var_31, var_32, var_33,
            var_34, var_35, var_36, var_37,
            var_38, var_39, var_3A, var_3B,
            var_3C, var_3D, var_3E, var_3F,
        ],
        pad3: [u8; 1984],
        words: [
            var_40, var_41, var_42, var_43,
            var_44, var_45, var_46, var_47,
            var_48, var_49, var_4A, var_4B,
            var_4C, var_4D, var_4E, var_4F,
            var_50, var_51, var_52, var_53,
            var_54, var_55, var_56, var_57,
            var_58, var_59, var_5A, var_5B,
            var_5C, var_5D, var_5E, var_5F,
        ],
    }
}

enemy_layout! {
    EnemyLayout72 {
        base: EnemyBase,
        words: [
            var_A, var_B, var_C, var_D, var_E, var_F,
            parameter_1, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, var_03, var_04, var_05,
            var_06, var_07, var_08, var_09, var_0A, var_0B,
            var_0C, var_0D, var_0E, var_0F, var_10, var_11,
            var_12, var_13, var_14, var_15, var_16, var_17,
            var_18, var_19, var_1A, var_1B, var_1C, var_1D,
            var_1E, var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            var_20, var_21, var_22, var_23, var_24, var_25,
            var_26, var_27, var_28, var_29, var_2A, var_2B,
            var_2C, var_2D, var_2E, var_2F, var_30, var_31,
            var_32, var_33, var_34, var_35, var_36, var_37,
            var_38, var_39, var_3A, var_3B, var_3C, var_3D,
            var_3E, var_3F,
        ],
    }
}

enemy_layout! {
    EnemyLayout40 {
        base: EnemyBase,
        words: [
            var_A, var_B, var_C, var_D, var_E, var_F,
            parameter_1, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, var_03, var_04, var_05,
            var_06, var_07, var_08, var_09, var_0A, var_0B,
            var_0C, var_0D, var_0E, var_0F, var_10, var_11,
            var_12, var_13, var_14, var_15, var_16, var_17,
            var_18, var_19, var_1A, var_1B, var_1C, var_1D,
            var_1E, var_1F,
        ],
    }
}

enemy_layout! {
    EnemyLayout8 {
        base: EnemyBase,
        words: [
            var_A, var_B, var_C, var_D, var_E, var_F,
            parameter_1, parameter_2,
        ],
    }
}

enemy_layout! {
    EnemyLayout72Late {
        base: EnemyBase,
        words: [
            var_A, var_B, var_C, var_D, var_E, var_F,
            parameter_1, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, var_03, var_04, var_05,
            var_06, var_07, var_08, var_09, var_0A, var_0B,
            var_0C, var_0D, var_0E, var_0F, var_10, var_11,
            var_12, var_13, var_14, var_15, var_16, var_17,
            var_18, var_19, var_1A, var_1B, var_1C, var_1D,
            var_1E, var_1F,
        ],
        pad3: [u8; 4032],
        words: [
            var_40, var_41, var_42, var_43, var_44, var_45,
            var_46, var_47, var_48, var_49, var_4A, var_4B,
            var_4C, var_4D, var_4E, var_4F, var_50, var_51,
            var_52, var_53, var_54, var_55, var_56, var_57,
            var_58, var_59, var_5A, var_5B, var_5C, var_5D,
            var_5E, var_5F,
        ],
    }
}

enemy_layout! {
    EnemyLayout40Late {
        base: EnemyBase,
        words: [
            var_A, var_B, var_C, var_D, var_E, var_F,
            parameter_1, parameter_2,
        ],
        pad2: [u8; 28744],
        words: [
            var_20, var_21, var_22, var_23, var_24, var_25,
            var_26, var_27, var_28, var_29, var_2A, var_2B,
            var_2C, var_2D, var_2E, var_2F, var_30, var_31,
            var_32, var_33, var_34, var_35, var_36, var_37,
            var_38, var_39, var_3A, var_3B, var_3C, var_3D,
            var_3E, var_3F,
        ],
    }
}
