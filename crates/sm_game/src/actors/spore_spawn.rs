//! Spore Spawn, Brinstar's spore-spewing plant boss: the script instructions
//! that steer its swing (bank $A5:E75F-$A5:E8C9).
//!
//! They lived with Draygon as `increase_grab_speed`, `clear_grab_rotation`,
//! `grab_timers`, `place_grab_start` and `set_state`; Spore Spawn's slot
//! reuses the words Draygon calls its spiral.
use crate::actors::host::ActorHost;
use crate::actors::layouts::{EnemyBase, same_words};
use crate::enemy_script::EnemyScript;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

/// Spore Spawn's view of its slot.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct SporeSpawn {
    pub base: EnemyBase,
    pub function: u16,
    pad_0faa: [u8; 26732],
    pub max_x_radius: u16,
    pub angle_delta: u16,
    pad_781a: [u8; 2052],
    pub damaged: u16,
    pad_8020: [u8; 2080],
}

same_words!(SporeSpawn: EnemyLayout104 {
    function = var_a, max_x_radius = var_0b, angle_delta = var_0c, damaged = var_2f,
});

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
    if (slot!(game, SporeSpawn[0].max_x_radius) as i32 - RADIUS_LIMIT) & SIGN != 0 {
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
    let k = word!(game.CUR_ENEMY_INDEX) as i32;
    slot!(game, SporeSpawn[k].function = function);
}
