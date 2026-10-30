//! Spore Spawn, Brinstar's spore-spewing plant boss: the script instructions
//! that steer its swing (bank $A5:E75F-$A5:E8C9).
//!
//! They lived with Draygon as `increase_grab_speed`, `clear_grab_rotation`,
//! `grab_timers`, `place_grab_start` and `set_state`; Spore Spawn's slot
//! reuses the words Draygon calls its spiral.
#![forbid(unsafe_code)]
use crate::actors::host::ActorHost;
use crate::actors::layouts::part_view;
use crate::enemy_script::EnemyScript;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

part_view! {
    /// Spore Spawn's view of its slot.
    SporeSpawn: EnemyLayout104 {
        words: [function, var_B, var_C, var_D, var_E, var_F, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, var_03, var_04, var_05, var_06, var_07, var_08, var_09, var_0A,
            max_x_radius, angle_delta, var_0D, var_0E, var_0F, var_10, var_11, var_12, var_13,
            var_14, var_15, var_16, var_17, var_18, var_19, var_1A, var_1B, var_1C, var_1D, var_1E,
            var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            var_20, var_21, var_22, var_23, var_24, var_25, var_26, var_27, var_28, var_29, var_2A,
            var_2B, var_2C, var_2D, var_2E, damaged, var_30, var_31, var_32, var_33, var_34, var_35,
            var_36, var_37, var_38, var_39, var_3A, var_3B, var_3C, var_3D, var_3E, var_3F,
        ],
        pad3: [u8; 1984],
        words: [
            var_40, var_41, var_42, var_43, var_44, var_45, var_46, var_47, var_48, var_49, var_4A,
            var_4B, var_4C, var_4D, var_4E, var_4F, var_50, var_51, var_52, var_53, var_54, var_55,
            var_56, var_57, var_58, var_59, var_5A, var_5B, var_5C, var_5D, var_5E, var_5F,
        ],
    }
}

/// Bank $A5 Spore Spawn instructions.
pub mod instruction {
    pub const INCREASE_MAX_X_RADIUS: u32 = 0xa5_e75f;
    pub const CLEAR_DAMAGED_FLAG: u32 = 0xa5_e771;
    pub const SET_MAX_X_RADIUS_AND_ANGLE_DELTA: u32 = 0xa5_e82d;
    pub const HARDEN: u32 = 0xa5_e87c;
    pub const SET_FUNCTION: u32 = 0xa5_e8ba;
}

/// The max X radius grows by 8 while below 30h; it is always 30h or 40h, so
/// this never takes effect.
const RADIUS_STEP: u16 = 8;
const RADIUS_LIMIT: i32 = 0x30 - 8;
/// Hardening puts Spore Spawn at (80h, 270h), solid to Samus, processing its
/// instructions and tangible.
const HARD_X: u16 = 0x80;
const HARD_Y: u16 = 0x270;
const HARD_KEEP: u16 = 0x5bff;
const HARD_SET: u16 = 0xa000;
const SIGN: i32 = 0x8000;

/// Instruction - increase Spore Spawn max X radius ($A5:E75F). Was
/// `increase_grab_speed`.
pub fn increase_max_x_radius<H: ActorHost + ?Sized>(game: &mut H) {
    if slot!(game, SporeSpawn[0].max_x_radius) as i32 - RADIUS_LIMIT & SIGN != 0 {
        slot!(game, SporeSpawn[0].max_x_radius, |old| old
            .wrapping_add(RADIUS_STEP));
    }
}

/// Instruction - clear Spore Spawn damaged flag ($A5:E771). Was
/// `clear_grab_rotation`.
pub fn clear_damaged_flag<H: ActorHost + ?Sized>(game: &mut H) {
    slot!(game, SporeSpawn[0].damaged = 0);
}

/// Instruction - Spore Spawn max X radius = [[Y]], angle delta = [[Y] + 2]
/// ($A5:E82D). Was `grab_timers`.
pub fn set_max_x_radius_and_angle_delta<H: ActorHost + ?Sized>(
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    slot!(game, SporeSpawn[0]; max_x_radius = script.word(0), angle_delta = script.word(2));
    script.offset(2)
}

/// Instruction - Spore Spawn harden ($A5:E87C). Was `place_grab_start`.
pub fn harden<H: ActorHost + ?Sized>(game: &mut H) {
    slot!(game, SporeSpawn[0].base; x_pos = HARD_X, y_pos = HARD_Y,
        properties = slot!(game, SporeSpawn[0].base.properties) & HARD_KEEP | HARD_SET);
}

/// Instruction - enemy function = [[Y]] ($A5:E8BA), given [[Y]]. Was `set_state`.
pub fn set_function<H: ActorHost + ?Sized>(function: u16, game: &mut H) {
    let k = word!(game.cur_enemy_index) as i32;
    slot!(game, SporeSpawn[k].function = function);
}
