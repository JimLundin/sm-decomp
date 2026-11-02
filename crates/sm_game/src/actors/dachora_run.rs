//! Dachora's run and shinespark: it waits for Samus, blinks, runs back and
//! forth gaining speed and shine, charges at the room's right end and
//! shinesparks upward trailing four echoes (bank $A7:F4DD-$A7:F9C1).
//!
//! Was `Dachora_*` in the original decompilation. The dachora and its echoes
//! run the bank $A7 function in their `function` word, whose ids are the
//! original's.
use crate::actors::boss_lease::BossLease;
use crate::actors::dachora::load_palette;
use crate::actors::host::BossEffects;
use crate::actors::layouts::part_view;
use crate::actors::session_slots::session_slot;

part_view! {
    /// The dachora's view of its slot. `speed` is its 16.16 running speed,
    /// and while it shinesparks its upward speed; it also counts the frames
    /// before running and before the shinespark. `acceleration` is the
    /// shinespark's 16.16 upward acceleration. `shine` counts down to the
    /// next shine palette in its low byte and holds the palette step in its
    /// high byte. `facing_right` is 1 facing right.
    Dachora: EnemyLayout8 {
        words: [
            speed, subspeed, acceleration, subacceleration, shine, function, facing_right,
            parameter_2,
        ],
    }
}

part_view! {
    /// An echo's view of its slot: how long it stays, and, in the first echo,
    /// the frames until the next echo is placed. `parameter_1` is negative,
    /// bit 0 facing right.
    Echo: EnemyLayout8 {
        words: [spawn_timer, var_b, var_c, lifetime, var_e, function, parameter_1, parameter_2],
    }
}

/// Bank $A7 dachora functions, stored in `function`.
pub mod function {
    pub const WAIT_FOR_SAMUS: u16 = 0xf570;
    pub const START_RUNNING: u16 = 0xf5bc;
    pub const RUNNING_LEFT: u16 = 0xf5ed;
    pub const RUNNING_RIGHT: u16 = 0xf65e;
    pub const ACTIVATE_SHINESPARK: u16 = 0xf78f;
    pub const SHINESPARKING: u16 = 0xf806;
    pub const FALLING: u16 = 0xf935;
    pub const ECHO: u16 = 0xf98c;
}

/// Bank $A7 dachora instruction lists and its palette.
pub mod instruction_list {
    pub const RUNNING_LEFT: u16 = 0xf345;
    pub const IDLING_FACING_LEFT: u16 = 0xf399;
    pub const BLINKING_FACING_LEFT: u16 = 0xf3c9;
    pub const ECHO_FACING_LEFT: u16 = 0xf3f7;
    pub const FALLING_FACING_LEFT: u16 = 0xf3ff;
    pub const RUNNING_RIGHT: u16 = 0xf407;
    pub const IDLING_FACING_RIGHT: u16 = 0xf45b;
    pub const BLINKING_FACING_RIGHT: u16 = 0xf48b;
    pub const CHARGE_SHINESPARK_FACING_RIGHT: u16 = 0xf4b3;
    pub const ECHO_FACING_RIGHT: u16 = 0xf4b9;
    pub const FALLING_FACING_RIGHT: u16 = 0xf4c1;
    pub const NOTHING_SPRITEMAP: u16 = 0x804d;
    /// $A7:F225, the dachora's normal palette.
    pub const PALETTE: u16 = 0xf225;
}

use crate::actors::host::below;

/// A lease on a game with the dachora's effects.
pub trait Lease: BossLease<Game: BossEffects> {}
impl<T: BossLease<Game: BossEffects> + ?Sized> Lease for T {}

/// The dachora constants at $A7:F4C9: how near Samus must be, the blink,
/// the shinespark charge, the echo interval and lifetime, the top running
/// speed and the running acceleration.
const WAKE_DISTANCE: u16 = 0x60;
const BLINK_FRAMES: u16 = 0x78;
const CHARGE_FRAMES: u16 = 0x3c;
const ECHO_INTERVAL: u16 = 1;
const ECHO_LIFETIME: u16 = 8;
const TOP_SPEED: u16 = 8;
const RUN_ACCELERATION: u32 = 0x1000;
/// The shinespark rises at most 15 px a frame; the fall reaches at most 10.
const SHINESPARK_TOP_SPEED: u16 = 15;
const FALL_TOP_SPEED: u16 = 10;
/// The run turns at the room's left end and charges at its right end.
const LEFT_END_X: u16 = 96;
const RIGHT_END_X: u16 = 1152;
/// Instruction list step when the run speeds up at 4 and 8 px a frame.
const FASTER_RUN_STEP: u16 = 28;
/// The first echo's slot follows the dachora's.
const ECHOES: [i32; 4] = [0x40, 0x80, 0xc0, 0x100];
const SHINE_SOUND: u8 = 0x39;
const BLINK_SOUND: u8 = 0x1d;
const BUMP_SOUND: u8 = 0x71;
const CHARGE_SOUND: u8 = 0x3d;
const SHINESPARK_SOUND: u8 = 0x3b;
const LAND_SOUND: u8 = 0x3c;

/// Initialisation AI: a dachora idles, an echo waits. Was `Dachora_Init`.
pub fn initialize<L: Lease>(game: &mut L) {
    game.foreign(|game| {
        owned::initialize(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    });
}

/// Main AI: the function. Was `Dachora_Main`.
pub fn main_ai<L: Lease>(game: &mut L) {
    game.foreign(|game| owned::main_ai(&mut crate::runtime_access::RuntimeAccess::runtime(game)));
}

/// Falls to the floor and blinks once Samus is near. Was `Dachora_Func_2`.
pub fn wait_for_samus<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::wait_for_samus(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Blinks, then runs the way it faces. Was `Dachora_Func_3`.
pub fn start_running<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::start_running(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Runs left, turning at the wall or the room's left end. Was
/// `Dachora_Func_4`.
pub fn running_left<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::running_left(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Runs right, turning at a wall; at the room's right end it charges the
/// shinespark. Was `Dachora_Func_5`.
pub fn running_right<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::running_right(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Shines while it charges, then shinesparks with its echoes. Was
/// `Dachora_Func_7`.
pub fn activate_shinespark<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::activate_shinespark(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Shinesparks upward until it hits the ceiling, then falls. Was
/// `Dachora_Func_8`.
pub fn shinesparking<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::shinesparking(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Falls to the floor, then runs the way it faces. Was `Dachora_Func_11`.
pub fn falling<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::falling(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// An echo shows on alternate frames while it lasts, then hides. Was
/// `Dachora_Func_12`.
pub fn echo<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| owned::echo(&mut crate::runtime_access::RuntimeAccess::runtime(game), k));
}

/// Steps the shinespark palette, cycling through four. Was `Dachora_Func_10`.
pub fn load_shine_palette<L: Lease>(game: &mut L, k: u16) {
    let me = k as i32;
    let palette = crate::boss_lists::dachora_palette(
        game,
        true,
        session_slot!(byte game, Dachora[me].shine[1]) as isize,
    );
    load_palette(palette, k, game);
    let next: u16 = (session_slot!(game, Dachora[me].shine)).wrapping_add(256);
    session_slot!(game, Dachora[me].shine = next);
    if !below(next, 0x400) {
        session_slot!(game, Dachora[me].shine = 0);
    }
}

#[path = "dachora_owned.rs"]
pub mod owned;
