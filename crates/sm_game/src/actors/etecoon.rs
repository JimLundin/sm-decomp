//! Etecoons, the green monkeys that hop around the bottom of their room,
//! wall-jump up the shaft and leap the morph tunnel at its top (bank
//! $A7:E912-$A7:EEEA).
//!
//! Was `Etecoon_*` in the original decompilation. Each etecoon runs the bank
//! $A7 function in its `function` word, whose ids are the original's.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::BossEffects;
use crate::actors::layouts::part_view;

/// A lease on a game with the etecoons' effects.
pub trait Lease: BossLease<Game: BossEffects> {
    /// Whether moving actor `k` by `amount` (16.16) horizontally would be
    /// blocked, without moving it.
    fn probe_horizontal(&mut self, k: u16, amount: i32) -> bool;
}

part_view! {
    /// An etecoon's view of its slot: its 16.16 velocities, the function
    /// timer and function. `facing` is 1 facing right in its low byte and
    /// counts the hops at the top of the room in its high byte. `route` picks
    /// the run-up point in its low byte (bit 1 also lets the etecoon wait for
    /// Samus) and holds the quake freeze timer in its high byte.
    Etecoon: EnemyLayout8 {
        words: [y_speed, y_subspeed, x_speed, x_subspeed, timer, function, facing, route],
    }
}

/// Bank $A7 etecoon functions, stored in `function`.
pub mod function {
    pub const INITIAL: u16 = 0xe9af;
    pub const START_HOP_BOTTOM: u16 = 0xea00;
    pub const HOPPING_BOTTOM: u16 = 0xea37;
    pub const LOOK_AT_SAMUS: u16 = 0xeab5;
    pub const RUNNING_LEFT: u16 = 0xeb02;
    pub const RUNNING_RIGHT: u16 = 0xeb2c;
    pub const JUMPING: u16 = 0xeb50;
    pub const WALL_JUMP: u16 = 0xebcd;
    pub const LANDED_FROM_JUMP: u16 = 0xec1b;
    pub const RUN_TO_LEFT_RUN_UP: u16 = 0xec97;
    pub const RUN_TO_RIGHT_RUN_UP: u16 = 0xecbb;
    pub const RUNNING_FOR_TUNNEL_JUMP: u16 = 0xecdf;
    pub const TUNNEL_JUMP: u16 = 0xed09;
    pub const RUNNING_THROUGH_TUNNEL: u16 = 0xed2a;
    pub const FALLING_FROM_TUNNEL_LEDGE: u16 = 0xed54;
    pub const HOPPING_TOP: u16 = 0xed75;
    pub const START_HOP_TOP: u16 = 0xedc7;
    pub const HOP_UNTIL_SAMUS_NEAR: u16 = 0xee3e;
    pub const RUNNING_FOR_FAILED_TUNNEL_JUMP: u16 = 0xee9a;
    pub const FAILED_TUNNEL_JUMP: u16 = 0xeeb8;
}

/// Bank $A7 etecoon instruction lists.
pub mod instruction_list {
    pub const LOOK_RIGHT_AND_RUN_LEFT: u16 = 0xe81e;
    pub const RUNNING_LEFT: u16 = 0xe828;
    pub const WALL_JUMP_LEFT: u16 = 0xe83c;
    pub const HOPPING_FACING_LEFT: u16 = 0xe854;
    pub const HIT_CEILING: u16 = 0xe862;
    pub const WALL_JUMP_LEFT_ELIGIBLE: u16 = 0xe870;
    pub const LOOK_LEFT_AND_RUN_RIGHT: u16 = 0xe876;
    pub const RUNNING_RIGHT: u16 = 0xe880;
    pub const WALL_JUMP_RIGHT: u16 = 0xe894;
    pub const JUMPING_RIGHT: u16 = 0xe898;
    pub const HOPPING_FACING_RIGHT: u16 = 0xe8ac;
    pub const WALL_JUMP_RIGHT_ELIGIBLE: u16 = 0xe8c8;
    pub const INITIAL: u16 = 0xe8ce;
    pub const FLEXING: u16 = 0xe8d6;
    pub const NOTHING_SPRITEMAP: u16 = 0x804d;
}

/// The etecoon constants at $A7:E900: the hop and the tunnel jump's upward
/// speeds, the running speeds and how near Samus must be to be looked at.
const HOP_SPEED: u16 = 0xfffd;
const TUNNEL_JUMP_SPEED: u16 = 0xfffc;
const RUN_RIGHT_SPEED: u16 = 2;
const RUN_LEFT_SPEED: u16 = 0xfffe;
const LOOK_DISTANCE: u16 = 0x40;
/// Hops before the etecoon at the top of the room runs for the tunnel.
const TOP_HOPS: u16 = 4;
/// The run-up points, the tunnel and its ledge, by X position.
const LEFT_RUN_UP_X: u16 = 537;
const RIGHT_RUN_UP_X: u16 = 600;
const TUNNEL_X: u16 = 680;
const TUNNEL_LEDGE_X: u16 = 840;
const WAIT_FOR_SAMUS_X: u16 = 832;
/// Sounds play only while Samus is right of the first screen.
const SOUND_SAMUS_X: u16 = 256;
const FLEX_SOUND: u8 = 0x35;
const WALL_JUMP_SOUND: u8 = 0x32;
const HOP_SOUND: u8 = 0x33;

/// Counts down the timer; true when it reaches zero or below.

/// Freezes the etecoon for 128 frames while a quake shakes the room. Was
/// `Etecoon_Func_1`.

/// Moves by the X velocity; true when blocked. Was `Etecoon_Func_2`.

/// Moves by the Y velocity, then falls faster up to 5 px a frame; true when
/// blocked. Was `Etecoon_Func_3`.

/// Initialisation AI. Was `Etecoon_Init`.
pub fn initialize<L: Lease>(game: &mut L) {
    game.foreign(|game| {
        owned::initialize(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    });
}

/// Main AI: count down a quake freeze, else run the function. Was
/// `Etecoon_Main`.
pub fn main_ai<L: Lease>(game: &mut L) {
    game.foreign(|game| owned::main_ai(&mut crate::runtime_access::RuntimeAccess::runtime(game)));
}

/// Run the etecoon's bank $A7 function at `address` for the current enemy.

/// Waits, flexing when Samus comes near, then starts hopping. Was
/// `Etecoon_Func_4`.
pub fn initial<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::initial(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Starts a hop at the bottom of the room. Was `Etecoon_Func_5`.
pub fn start_hop_bottom<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::start_hop_bottom(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Hops; on landing near Samus it looks at her, else it hops again. Was
/// `Etecoon_Func_6`.
pub fn hopping_bottom<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::hopping_bottom(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Looks at Samus, then runs away from her. Was `Etecoon_Func_7`.
pub fn look_at_samus<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::look_at_samus(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Runs left until the wall, then turns right. Was `Etecoon_Func_8`.
pub fn running_left<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::running_left(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Runs right, jumping 32 px before the wall. Was `Etecoon_Func_9`.
pub fn running_right<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::running_right(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Jumps: against a wall it wall-jumps, on the floor it lands. Was
/// `Etecoon_Func_10`.
pub fn jumping<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::jumping(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Clings to the wall, then jumps off it the other way. Was `Etecoon_Func_11`.
pub fn wall_jump<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::wall_jump(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Landed at the top of the shaft: heads for its run-up point. Was
/// `Etecoon_Func_12`.
pub fn landed_from_jump<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::landed_from_jump(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Was `Etecoon_Func_13`.

/// Was `Etecoon_Func_14`.

/// Was `Etecoon_Func_15`.

/// Was `Etecoon_Func_16`.
pub fn run_to_left_run_up_point<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::run_to_left_run_up_point(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Was `Etecoon_Func_17`.
pub fn run_to_right_run_up_point<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::run_to_right_run_up_point(
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
            k,
        )
    });
}

/// Runs for the morph tunnel and jumps high at the run-up point. Was
/// `Etecoon_Func_18`.
pub fn running_for_tunnel_jump<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::running_for_tunnel_jump(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Was `Etecoon_Func_19`.
pub fn tunnel_jump<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::tunnel_jump(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Was `Etecoon_Func_20`.
pub fn running_through_tunnel<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::running_through_tunnel(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Was `Etecoon_Func_21`.
pub fn falling_from_tunnel_ledge<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::falling_from_tunnel_ledge(
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
            k,
        )
    });
}

/// Hops at the top of the room; a waiting etecoon left of the tunnel waits
/// for Samus. Was `Etecoon_Func_22`.
pub fn hopping_top<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::hopping_top(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Hops up to four times, then runs for the tunnel and fails the jump. Was
/// `Etecoon_Func_23`.
pub fn start_hop_top<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::start_hop_top(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Hops until Samus is near, then runs for the tunnel jump. Was
/// `Etecoon_Func_24`.
pub fn hop_until_samus_near<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::hop_until_samus_near(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

/// Was `Etecoon_Func_25`.
pub fn running_for_failed_tunnel_jump<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::running_for_failed_tunnel_jump(
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
            k,
        )
    });
}

/// Falls short of the tunnel and runs back left. Was `Etecoon_Func_26`.
pub fn failed_tunnel_jump<L: Lease>(game: &mut L, k: u16) {
    game.foreign(|game| {
        owned::failed_tunnel_jump(&mut crate::runtime_access::RuntimeAccess::runtime(game), k)
    });
}

#[path = "etecoon_owned.rs"]
pub mod owned;
