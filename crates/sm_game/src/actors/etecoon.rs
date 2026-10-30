//! Etecoons, the green monkeys that hop around the bottom of their room,
//! wall-jump up the shaft and leap the morph tunnel at its top (bank
//! $A7:E912-$A7:EEEA).
//!
//! Was `Etecoon_*` in the original decompilation. Each etecoon runs the bank
//! $A7 function in its `function` word; the compat host dispatches it, so the
//! function ids are unchanged.
use crate::actors::api::PlayerDirection;
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Axis, Slopes, below};
use crate::actors::layouts::part_view;
use crate::actors::session_slots::session_slot;
use crate::constants::enemy_properties::K_ENEMY_PROPS_DISABLE_SAMUS_COLL;
use crate::ram::word;

/// What the etecoons need beyond the common actor operations and sounds.
pub trait Host: crate::actors::host::ActorHost + BossLease<Game = Self> {
    /// Run the bank $A7 function at `address` for the current etecoon.
    fn run_function(&mut self, address: u32);
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

use instruction_list as list;

/// A lease on a game with the etecoons' effects.
pub trait Lease: BossLease<Game: Host> {}
impl<T: BossLease<Game: Host> + ?Sized> Lease for T {}

const BANK: u32 = 0xa7;
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
fn timer_expired<L: Lease>(game: &mut L, k: u16) -> bool {
    let me = k as i32;
    let last = session_slot!(game, Etecoon[me].timer) == 1;
    session_slot!(game, Etecoon[me].timer, |old| old.wrapping_sub(1));
    let negative = (session_slot!(game, Etecoon[me].timer) & 0x8000) != 0;
    last || negative
}

fn sound_near_samus<L: Lease>(game: &mut L, sound: u8) {
    if !below(game.player_x(), SOUND_SAMUS_X) {
        game.queue_sound(1, sound, 6);
    }
}

fn set_hop<L: Lease>(game: &mut L, me: i32) {
    session_slot!(game, Etecoon[me].y_speed = HOP_SPEED);
    session_slot!(game, Etecoon[me].y_subspeed = 0);
}

fn set_run<L: Lease>(game: &mut L, me: i32, speed: u16) {
    session_slot!(game, Etecoon[me].x_speed = speed);
    session_slot!(game, Etecoon[me].x_subspeed = 0);
}

/// Freezes the etecoon for 128 frames while a quake shakes the room. Was
/// `Etecoon_Func_1`.
fn freeze_if_quake<L: Lease>(game: &mut L, k: u16) {
    if word!(game.EARTHQUAKE_TIMER) != 0 {
        let me = k as i32;
        session_slot!(
            game,
            Etecoon[me].route = (session_slot!(byte game, Etecoon[me].route[0]) as u16) | 0x8000
        );
        session_slot!(
            game,
            Etecoon[me].base.instruction_timer =
                (session_slot!(game, Etecoon[me].base.instruction_timer)).wrapping_add(128)
        );
    }
}

/// Moves by the X velocity; true when blocked. Was `Etecoon_Func_2`.
fn move_horizontally<L: Lease>(game: &mut L, k: u16) -> bool {
    let me = k as i32;
    let amount = (((session_slot!(game, Etecoon[me].x_speed) as u32) << 16)
        | session_slot!(game, Etecoon[me].x_subspeed) as u32) as i32;
    game.move_horizontal(k, amount, Slopes::Ignore)
}

/// Moves by the Y velocity, then falls faster up to 5 px a frame; true when
/// blocked. Was `Etecoon_Func_3`.
fn move_vertically<L: Lease>(game: &mut L, k: u16) -> bool {
    let me = k as i32;
    let amount = (((session_slot!(game, Etecoon[me].y_speed) as u32) << 16)
        | session_slot!(game, Etecoon[me].y_subspeed) as u32) as i32;
    if below(session_slot!(game, Etecoon[me].y_speed), 5) {
        let gravity =
            ((word!(game.SAMUS_Y_ACCEL) as u32) << 16) | word!(game.SAMUS_Y_SUBACCEL) as u32;
        session_slot!(game, Etecoon[me](y_speed, y_subspeed) += gravity);
    }
    game.move_vertical(k, amount)
}

/// Initialisation AI. Was `Etecoon_Init`.
pub fn initialize<L: Lease>(game: &mut L) {
    let me = game.current_enemy() as i32;
    session_slot!(
        game,
        Etecoon[me].base.properties = (session_slot!(game, Etecoon[me].base.properties))
            | (K_ENEMY_PROPS_DISABLE_SAMUS_COLL as u16)
    );
    session_slot!(
        game,
        Etecoon[me].base.spritemap_pointer = list::NOTHING_SPRITEMAP
    );
    session_slot!(game, Etecoon[me].base.instruction_timer = 1);
    session_slot!(game, Etecoon[me].base.timer = 0);
    session_slot!(game, Etecoon[me].base.current_instruction = list::INITIAL);
    session_slot!(game, Etecoon[me].function = function::INITIAL);
    session_slot!(game, Etecoon[me].timer = 0xffff);
}

/// Main AI: count down a quake freeze, else run the function. Was
/// `Etecoon_Main`.
pub fn main_ai<L: Lease>(game: &mut L) {
    let me = game.current_enemy() as i32;
    if session_slot!(byte game, Etecoon[me].route[1]) != 0 {
        session_slot!(game, Etecoon[me].route, |old| old.wrapping_sub(256));
    } else {
        let address = (BANK << 16) | session_slot!(game, Etecoon[me].function) as u32;
        game.foreign(|game| game.run_function(address));
    }
}

/// Waits, flexing when Samus comes near, then starts hopping. Was
/// `Etecoon_Func_4`.
pub fn initial<L: Lease>(game: &mut L, k: u16) {
    if word!(game.DOOR_TRANSITION_FLAG_ENEMIES) == 0 {
        let me = k as i32;
        if (session_slot!(game, Etecoon[me].timer) & 0x8000) == 0 {
            if timer_expired(game, k) {
                session_slot!(
                    game,
                    Etecoon[me].base.current_instruction = list::HOPPING_FACING_LEFT
                );
                session_slot!(game, Etecoon[me].function = function::START_HOP_BOTTOM);
                session_slot!(game, Etecoon[me].timer = 11);
            }
        } else if game.player_within(k, Axis::Y, 0x80) {
            if (session_slot!(game, Etecoon[me].route) & 3) == 0 {
                game.queue_sound(1, FLEX_SOUND, 15);
            }
            session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::FLEXING);
            session_slot!(game, Etecoon[me].timer = 256);
        }
    }
}

/// Starts a hop at the bottom of the room. Was `Etecoon_Func_5`.
pub fn start_hop_bottom<L: Lease>(game: &mut L, k: u16) {
    if !timer_expired(game, k) {
        return;
    }
    let me = k as i32;
    set_hop(game, me);
    session_slot!(game, Etecoon[me].base; current_instruction = (session_slot!(game, Etecoon[me].base.current_instruction)).wrapping_add(2), instruction_timer = 1);
    session_slot!(game, Etecoon[me].function = function::HOPPING_BOTTOM);
    sound_near_samus(game, HOP_SOUND);
}

/// Hops; on landing near Samus it looks at her, else it hops again. Was
/// `Etecoon_Func_6`.
pub fn hopping_bottom<L: Lease>(game: &mut L, k: u16) {
    if move_vertically(game, k) {
        let me = k as i32;
        if (session_slot!(game, Etecoon[me].y_speed) & 0x8000) == 0 {
            if game.player_within(k, Axis::Y, 0x40) && game.player_within(k, Axis::X, LOOK_DISTANCE)
            {
                let samus_right_or_level = matches!(
                    game.player_direction(),
                    PlayerDirection::Up
                        | PlayerDirection::UpRight
                        | PlayerDirection::Right
                        | PlayerDirection::DownRight
                        | PlayerDirection::Down
                );
                if samus_right_or_level {
                    session_slot!(
                        game,
                        Etecoon[me].base.current_instruction = list::LOOK_RIGHT_AND_RUN_LEFT
                    );
                    session_slot!(game, Etecoon[me].facing = 0);
                } else {
                    session_slot!(
                        game,
                        Etecoon[me].base.current_instruction = list::LOOK_LEFT_AND_RUN_RIGHT
                    );
                    session_slot!(game, Etecoon[me].facing = 1);
                }
                session_slot!(game, Etecoon[me].timer = 32);
                session_slot!(game, Etecoon[me].base.instruction_timer = 1);
                session_slot!(game, Etecoon[me].function = function::LOOK_AT_SAMUS);
            } else {
                session_slot!(game, Etecoon[me].timer = 11);
                session_slot!(game, Etecoon[me].function = function::START_HOP_BOTTOM);
                session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::HOPPING_FACING_LEFT);
            }
        } else {
            hit_ceiling(game, me);
        }
    }
}

fn hit_ceiling<L: Lease>(game: &mut L, me: i32) {
    session_slot!(game, Etecoon[me]; y_speed = 0, y_subspeed = 0);
    session_slot!(game, Etecoon[me].base; instruction_timer = 3, current_instruction = list::HIT_CEILING);
}

/// Looks at Samus, then runs away from her. Was `Etecoon_Func_7`.
pub fn look_at_samus<L: Lease>(game: &mut L, k: u16) {
    if !timer_expired(game, k) {
        return;
    }
    let me = k as i32;
    session_slot!(game, Etecoon[me].base; current_instruction = (session_slot!(game, Etecoon[me].base.current_instruction)).wrapping_add(2), instruction_timer = 1);
    if session_slot!(game, Etecoon[me].facing) != 0 {
        set_run(game, me, RUN_RIGHT_SPEED);
        session_slot!(game, Etecoon[me].function = function::RUNNING_RIGHT);
    } else {
        set_run(game, me, RUN_LEFT_SPEED);
        session_slot!(game, Etecoon[me].function = function::RUNNING_LEFT);
    }
    set_hop(game, me);
}

/// Runs left until the wall, then turns right. Was `Etecoon_Func_8`.
pub fn running_left<L: Lease>(game: &mut L, k: u16) {
    if move_horizontally(game, k) {
        let me = k as i32;
        set_run(game, me, RUN_RIGHT_SPEED);
        session_slot!(game, Etecoon[me].function = function::RUNNING_RIGHT);
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::RUNNING_RIGHT);
        session_slot!(game, Etecoon[me].facing = 1);
    }
}

/// Runs right, jumping 32 px before the wall. Was `Etecoon_Func_9`.
pub fn running_right<L: Lease>(game: &mut L, k: u16) {
    if game.foreign(|game| game.probe_horizontal(k, (32u32 << 16) as i32)) {
        let me = k as i32;
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::JUMPING_RIGHT);
        session_slot!(game, Etecoon[me].function = function::JUMPING);
    } else {
        move_horizontally(game, k);
    }
}

/// Jumps: against a wall it wall-jumps, on the floor it lands. Was
/// `Etecoon_Func_10`.
pub fn jumping<L: Lease>(game: &mut L, k: u16) {
    freeze_if_quake(game, k);
    if move_horizontally(game, k) {
        let me = k as i32;
        if session_slot!(game, Etecoon[me].facing) != 0 {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::WALL_JUMP_LEFT_ELIGIBLE
            );
            session_slot!(game, Etecoon[me].facing = 0);
        } else {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::WALL_JUMP_RIGHT_ELIGIBLE
            );
            session_slot!(game, Etecoon[me].facing = 1);
        }
        session_slot!(game, Etecoon[me].base.instruction_timer = 1);
        session_slot!(game, Etecoon[me].function = function::WALL_JUMP);
        session_slot!(game, Etecoon[me].timer = 8);
        sound_near_samus(game, WALL_JUMP_SOUND);
    } else if move_vertically(game, k) {
        let me = k as i32;
        if session_slot!(game, Etecoon[me].facing) != 0 {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::HOPPING_FACING_LEFT
            );
        } else {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::HOPPING_FACING_RIGHT
            );
        }
        session_slot!(game, Etecoon[me].base.instruction_timer = 1);
        session_slot!(game, Etecoon[me]; timer = 11, function = function::LANDED_FROM_JUMP, y_speed = HOP_SPEED, y_subspeed = 0);
    }
}

/// Clings to the wall, then jumps off it the other way. Was `Etecoon_Func_11`.
pub fn wall_jump<L: Lease>(game: &mut L, k: u16) {
    freeze_if_quake(game, k);
    if timer_expired(game, k) {
        let me = k as i32;
        if session_slot!(game, Etecoon[me].facing) != 0 {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::WALL_JUMP_RIGHT
            );
            set_run(game, me, RUN_RIGHT_SPEED);
        } else {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::WALL_JUMP_LEFT
            );
            set_run(game, me, RUN_LEFT_SPEED);
        }
        session_slot!(game, Etecoon[me].base.instruction_timer = 1);
        session_slot!(game, Etecoon[me].function = function::JUMPING);
        set_hop(game, me);
    }
}

/// Landed at the top of the shaft: heads for its run-up point. Was
/// `Etecoon_Func_12`.
pub fn landed_from_jump<L: Lease>(game: &mut L, k: u16) {
    if !timer_expired(game, k) {
        return;
    }
    let me = k as i32;
    set_hop(game, me);
    let routes: [fn(&mut L, i32); 3] = [
        run_to_left_run_up,
        run_to_right_run_up,
        hop_at_centre_run_up,
    ];
    routes[session_slot!(byte game, Etecoon[me].route[0]) as usize](game, me);
    session_slot!(game, Etecoon[me].base.instruction_timer = 1);
}

/// Was `Etecoon_Func_13`.
fn run_to_left_run_up<L: Lease>(game: &mut L, me: i32) {
    session_slot!(game, Etecoon[me].function = function::RUN_TO_LEFT_RUN_UP);
    session_slot!(
        game,
        Etecoon[me].base.current_instruction = list::RUNNING_LEFT
    );
    set_run(game, me, RUN_LEFT_SPEED);
}

/// Was `Etecoon_Func_14`.
fn run_to_right_run_up<L: Lease>(game: &mut L, me: i32) {
    session_slot!(game, Etecoon[me].function = function::RUN_TO_RIGHT_RUN_UP);
    session_slot!(
        game,
        Etecoon[me].base.current_instruction = list::RUNNING_RIGHT
    );
    set_run(game, me, RUN_RIGHT_SPEED);
}

/// Was `Etecoon_Func_15`.
fn hop_at_centre_run_up<L: Lease>(game: &mut L, me: i32) {
    session_slot!(game, Etecoon[me].function = function::HOPPING_TOP);
    session_slot!(
        game,
        Etecoon[me].base.current_instruction =
            (session_slot!(game, Etecoon[me].base.current_instruction)).wrapping_add(2)
    );
    set_run(game, me, RUN_RIGHT_SPEED);
}

fn start_top_hops<L: Lease>(game: &mut L, me: i32) {
    session_slot!(game, Etecoon[me].timer = 11);
    session_slot!(game, Etecoon[me].function = function::START_HOP_TOP);
    session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::HOPPING_FACING_LEFT);
}

/// Was `Etecoon_Func_16`.
pub fn run_to_left_run_up_point<L: Lease>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    let me = k as i32;
    if below(session_slot!(game, Etecoon[me].base.x_pos), LEFT_RUN_UP_X) {
        start_top_hops(game, me);
    }
}

/// Was `Etecoon_Func_17`.
pub fn run_to_right_run_up_point<L: Lease>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    let me = k as i32;
    if !below(session_slot!(game, Etecoon[me].base.x_pos), RIGHT_RUN_UP_X) {
        start_top_hops(game, me);
    }
}

/// Runs for the morph tunnel and jumps high at the run-up point. Was
/// `Etecoon_Func_18`.
pub fn running_for_tunnel_jump<L: Lease>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    let me = k as i32;
    if !below(session_slot!(game, Etecoon[me].base.x_pos), RIGHT_RUN_UP_X) {
        session_slot!(game, Etecoon[me].function = function::TUNNEL_JUMP);
        session_slot!(game, Etecoon[me].y_speed = TUNNEL_JUMP_SPEED);
        session_slot!(game, Etecoon[me].y_subspeed = 0);
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::JUMPING_RIGHT);
    }
}

/// Was `Etecoon_Func_19`.
pub fn tunnel_jump<L: Lease>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    move_vertically(game, k);
    let me = k as i32;
    if !below(session_slot!(game, Etecoon[me].base.x_pos), TUNNEL_X) {
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::RUNNING_RIGHT);
        session_slot!(
            game,
            Etecoon[me].function = function::RUNNING_THROUGH_TUNNEL
        );
    }
}

/// Was `Etecoon_Func_20`.
pub fn running_through_tunnel<L: Lease>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    let me = k as i32;
    if !below(session_slot!(game, Etecoon[me].base.x_pos), TUNNEL_LEDGE_X) {
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::JUMPING_RIGHT);
        session_slot!(
            game,
            Etecoon[me].function = function::FALLING_FROM_TUNNEL_LEDGE
        );
        session_slot!(game, Etecoon[me].y_speed = 0xffff);
        session_slot!(game, Etecoon[me].y_subspeed = 0);
    }
}

/// Was `Etecoon_Func_21`.
pub fn falling_from_tunnel_ledge<L: Lease>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    if move_vertically(game, k) {
        let me = k as i32;
        session_slot!(game, Etecoon[me].timer = 11);
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::HOPPING_FACING_LEFT);
        session_slot!(game, Etecoon[me].function = function::START_HOP_TOP);
    }
}

/// Hops at the top of the room; a waiting etecoon left of the tunnel waits
/// for Samus. Was `Etecoon_Func_22`.
pub fn hopping_top<L: Lease>(game: &mut L, k: u16) {
    freeze_if_quake(game, k);
    if move_vertically(game, k) {
        let me = k as i32;
        if (session_slot!(game, Etecoon[me].y_speed) & 0x8000) == 0 {
            session_slot!(game, Etecoon[me].timer = 11);
            session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::HOPPING_FACING_LEFT);
            if (session_slot!(game, Etecoon[me].route) & 2) != 0
                && below(
                    session_slot!(game, Etecoon[me].base.x_pos),
                    WAIT_FOR_SAMUS_X,
                )
            {
                session_slot!(game, Etecoon[me].function = function::HOP_UNTIL_SAMUS_NEAR);
            } else {
                session_slot!(game, Etecoon[me].function = function::START_HOP_TOP);
            }
        } else {
            hit_ceiling(game, me);
        }
    }
}

/// Hops up to four times, then runs for the tunnel and fails the jump. Was
/// `Etecoon_Func_23`.
pub fn start_hop_top<L: Lease>(game: &mut L, k: u16) {
    freeze_if_quake(game, k);
    if timer_expired(game, k) {
        let me = k as i32;
        let hops: u16 = (session_slot!(game, Etecoon[me].facing)).wrapping_add(256);
        session_slot!(game, Etecoon[me].facing = hops);
        if below(hops & 0xff00, TOP_HOPS << 8) || {
            session_slot!(
                game,
                Etecoon[me].facing = session_slot!(byte game, Etecoon[me].facing[0]) as u16
            );
            (session_slot!(game, Etecoon[me].route) & 2) != 0
        } {
            session_slot!(game, Etecoon[me].function = function::HOPPING_TOP);
            session_slot!(
                game,
                Etecoon[me].base.current_instruction =
                    (session_slot!(game, Etecoon[me].base.current_instruction)).wrapping_add(2)
            );
        } else {
            session_slot!(
                game,
                Etecoon[me].function = function::RUNNING_FOR_FAILED_TUNNEL_JUMP
            );
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::RUNNING_RIGHT
            );
            set_run(game, me, RUN_RIGHT_SPEED);
        }
        set_hop(game, me);
        session_slot!(game, Etecoon[me].base.instruction_timer = 1);
        sound_near_samus(game, HOP_SOUND);
    }
}

/// Hops until Samus is near, then runs for the tunnel jump. Was
/// `Etecoon_Func_24`.
pub fn hop_until_samus_near<L: Lease>(game: &mut L, k: u16) {
    freeze_if_quake(game, k);
    if timer_expired(game, k) {
        let me = k as i32;
        if game.player_within(k, Axis::Y, 0x40) && game.player_within(k, Axis::X, 0x30) {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::RUNNING_RIGHT
            );
            session_slot!(
                game,
                Etecoon[me].function = function::RUNNING_FOR_TUNNEL_JUMP
            );
        } else {
            set_hop(game, me);
            session_slot!(
                game,
                Etecoon[me].base.current_instruction =
                    (session_slot!(game, Etecoon[me].base.current_instruction)).wrapping_add(2)
            );
            session_slot!(game, Etecoon[me].function = function::HOPPING_TOP);
            sound_near_samus(game, HOP_SOUND);
        }
        session_slot!(game, Etecoon[me].base.instruction_timer = 1);
    }
}

/// Was `Etecoon_Func_25`.
pub fn running_for_failed_tunnel_jump<L: Lease>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    let me = k as i32;
    if !below(session_slot!(game, Etecoon[me].base.x_pos), RIGHT_RUN_UP_X) {
        session_slot!(game, Etecoon[me].function = function::FAILED_TUNNEL_JUMP);
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::JUMPING_RIGHT);
    }
}

/// Falls short of the tunnel and runs back left. Was `Etecoon_Func_26`.
pub fn failed_tunnel_jump<L: Lease>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    if move_vertically(game, k) {
        let me = k as i32;
        session_slot!(game, Etecoon[me]; x_speed = RUN_LEFT_SPEED, x_subspeed = 0, function = function::RUNNING_LEFT, y_speed = HOP_SPEED, y_subspeed = 0);
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::RUNNING_LEFT);
    }
}
