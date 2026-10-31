//! Dachora's run and shinespark: it waits for Samus, blinks, runs back and
//! forth gaining speed and shine, charges at the room's right end and
//! shinesparks upward trailing four echoes (bank $A7:F4DD-$A7:F9C1).
//!
//! Was `Dachora_*` in the original decompilation. The dachora and its echoes
//! run the bank $A7 function in their `function` word; the compat host
//! dispatches it, so the function ids are unchanged.
use crate::actors::boss_lease::BossLease;
use crate::actors::dachora::load_palette;
use crate::actors::host::{Axis, Slopes};
use crate::actors::layouts::part_view;
use crate::actors::session_slots::session_slot;
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_DISABLE_SAMUS_COLL, K_ENEMY_PROPS_INVISIBLE,
};
use crate::ram::word;

/// What the dachora needs beyond the common actor operations and sounds.
pub trait Host:
    crate::actors::boss_lease::BossLease<Game = Self> + crate::actors::host::ActorHost
{
    /// Run the bank $A7 function at `address` for the current dachora.
    fn run_function(&mut self, address: u32);
}

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
use instruction_list as list;

/// A lease on a game with the dachora's effects.
pub trait Lease: BossLease<Game: Host> {}
impl<T: BossLease<Game: Host> + ?Sized> Lease for T {}

const BANK: u32 = 0xa7;
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

fn gravity<L: Lease>(game: &L) -> u32 {
    ((word!(game.SAMUS_Y_ACCEL) as u32) << 16) | word!(game.SAMUS_Y_SUBACCEL) as u32
}

/// Initialisation AI: a dachora idles, an echo waits. Was `Dachora_Init`.
pub fn initialize<L: Lease>(game: &mut L) {
    let me = game.current_enemy() as i32;
    session_slot!(game, Dachora[me].base.properties, |old| old
        | (K_ENEMY_PROPS_DISABLE_SAMUS_COLL as u16));
    session_slot!(
        game,
        Dachora[me].base.spritemap_pointer = list::NOTHING_SPRITEMAP
    );
    session_slot!(game, Dachora[me].base; instruction_timer = 1, timer = 0);
    let kind: i16 = session_slot!(game, Dachora[me].facing_right) as i16;
    if kind < 0 {
        if (kind as u16 & 1) != 0 {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::ECHO_FACING_RIGHT
            );
        } else {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::ECHO_FACING_LEFT
            );
        }
        session_slot!(game, Dachora[me].function = function::ECHO);
    } else {
        if kind != 0 {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::IDLING_FACING_RIGHT
            );
        } else {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::IDLING_FACING_LEFT
            );
        }
        session_slot!(game, Dachora[me].function = function::WAIT_FOR_SAMUS);
    }
}

/// Main AI: the function. Was `Dachora_Main`.
pub fn main_ai<L: Lease>(game: &mut L) {
    let me = game.current_enemy() as i32;
    let address = (BANK << 16) | session_slot!(game, Dachora[me].function) as u32;
    game.foreign(|game| game.run_function(address));
}

/// Falls to the floor and blinks once Samus is near. Was `Dachora_Func_2`.
pub fn wait_for_samus<L: Lease>(game: &mut L, k: u16) {
    game.move_vertical(k, (1u32 << 16) as i32);
    if game.player_within(k, Axis::Y, 0x40) && game.player_within(k, Axis::X, WAKE_DISTANCE) {
        let me = k as i32;
        if session_slot!(game, Dachora[me].facing_right) != 0 {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::BLINKING_FACING_RIGHT
            );
        } else {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::BLINKING_FACING_LEFT
            );
        }
        session_slot!(game, Dachora[me].base.instruction_timer = 1);
        session_slot!(game, Dachora[me]; function = function::START_RUNNING, speed = BLINK_FRAMES);
        game.queue_sound(1, BLINK_SOUND, 15);
    }
}

/// Blinks, then runs the way it faces. Was `Dachora_Func_3`.
pub fn start_running<L: Lease>(game: &mut L, k: u16) {
    let me = k as i32;
    let left = session_slot!(game, Dachora[me].speed);
    let copied = session_slot!(game, Dachora[me].speed).wrapping_sub(1);
    session_slot!(game, Dachora[me].speed = copied);
    if left == 1 {
        if session_slot!(game, Dachora[me].facing_right) != 0 {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::RUNNING_RIGHT
            );
            session_slot!(game, Dachora[me].function = function::RUNNING_RIGHT);
        } else {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::RUNNING_LEFT
            );
            session_slot!(game, Dachora[me].function = function::RUNNING_LEFT);
        }
        session_slot!(game, Dachora[me].base.instruction_timer = 1);
        session_slot!(game, Dachora[me].shine = 1);
    }
}

/// Runs left, turning at the wall or the room's left end. Was
/// `Dachora_Func_4`.
pub fn running_left<L: Lease>(game: &mut L, k: u16) {
    let amount: i32 = -accelerate(game, k);
    let me = k as i32;
    if game.move_horizontal(k, amount, Slopes::Ignore) || {
        game.snap_to_slopes(k);
        below(session_slot!(game, Dachora[me].base.x_pos), LEFT_END_X)
    } {
        session_slot!(
            game,
            Dachora[me].base.current_instruction = list::RUNNING_RIGHT
        );
        session_slot!(game, Dachora[me].function = function::RUNNING_RIGHT);
        session_slot!(game, Dachora[me]; shine = 1, facing_right = 1);
        session_slot!(game, Dachora[me].base.instruction_timer = 1);
        session_slot!(game, Dachora[me]; speed = 0, subspeed = 0);
        load_palette(list::PALETTE, k, game);
    }
}

/// Runs right, turning at a wall; at the room's right end it charges the
/// shinespark. Was `Dachora_Func_5`.
pub fn running_right<L: Lease>(game: &mut L, k: u16) {
    let amount: i32 = accelerate(game, k);
    let me = k as i32;
    if game.move_horizontal(k, amount, Slopes::Ignore) {
        game.queue_sound(1, BUMP_SOUND, 15);
        session_slot!(
            game,
            Dachora[me].base.current_instruction = list::RUNNING_LEFT
        );
        session_slot!(game, Dachora[me].function = function::RUNNING_LEFT);
        session_slot!(game, Dachora[me]; facing_right = 0, speed = 0);
        load_palette(list::PALETTE, k, game);
    } else {
        game.snap_to_slopes(k);
        if !below(session_slot!(game, Dachora[me].base.x_pos), RIGHT_END_X) {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::CHARGE_SHINESPARK_FACING_RIGHT
            );
            session_slot!(game, Dachora[me]; function = function::ACTIVATE_SHINESPARK,
                speed = CHARGE_FRAMES);
            session_slot!(game, Dachora[me].base.y_pos, |old| old.wrapping_add(8));
            game.queue_sound(1, CHARGE_SOUND, 6);
        } else {
            return;
        }
    }
    session_slot!(game, Dachora[me]; shine = 0, subspeed = 0);
    session_slot!(game, Dachora[me].base.instruction_timer = 1);
}

/// Shines while it charges, then shinesparks with its echoes. Was
/// `Dachora_Func_7`.
pub fn activate_shinespark<L: Lease>(game: &mut L, k: u16) {
    load_shine_palette(game, k);
    let me = k as i32;
    let left = session_slot!(game, Dachora[me].speed);
    let copied = session_slot!(game, Dachora[me].speed).wrapping_sub(1);
    session_slot!(game, Dachora[me].speed = copied);
    if left == 1 {
        session_slot!(game, Dachora[me].base; current_instruction = (session_slot!(game, Dachora[me].base.current_instruction)).wrapping_add(2), instruction_timer = 1);
        session_slot!(game, Dachora[me].function = function::SHINESPARKING);
        let first = me + ECHOES[0];
        session_slot!(game, Echo[first]; spawn_timer = 0, lifetime = 0);
        for echo in &ECHOES[1..] {
            session_slot!(game, Echo[me + echo].lifetime = 0);
        }
        session_slot!(game, Dachora[me]; acceleration = 0, subacceleration = 0);
        session_slot!(game, Dachora[me].base.y_pos, |old| old.wrapping_sub(8));
        game.queue_sound(1, SHINESPARK_SOUND, 6);
        let echo_list = if session_slot!(game, Dachora[me].facing_right) != 0 {
            list::ECHO_FACING_RIGHT
        } else {
            list::ECHO_FACING_LEFT
        };
        for echo in ECHOES {
            session_slot!(game, Echo[me + echo].base.current_instruction = echo_list);
        }
        for echo in ECHOES {
            session_slot!(game, Echo[me + echo].base.instruction_timer = 1);
        }
    }
}

/// Shinesparks upward until it hits the ceiling, then falls. Was
/// `Dachora_Func_8`.
pub fn shinesparking<L: Lease>(game: &mut L, k: u16) {
    load_shine_palette(game, k);
    update_echoes(game, k);
    let me = k as i32;
    let pull = gravity(game);
    session_slot!(game, Dachora[me](acceleration, subacceleration) += pull);
    let acceleration = ((session_slot!(game, Dachora[me].acceleration) as u32) << 16)
        | session_slot!(game, Dachora[me].subacceleration) as u32;
    session_slot!(game, Dachora[me](speed, subspeed) += acceleration);
    let mut speed: u16 = session_slot!(game, Dachora[me].speed);
    if !below(speed, SHINESPARK_TOP_SPEED) {
        speed = SHINESPARK_TOP_SPEED;
    }
    let amount = (((speed as u32) << 16) | session_slot!(game, Dachora[me].subspeed) as u32) as i32;
    if game.move_vertical(k, -amount) {
        if session_slot!(game, Dachora[me].facing_right) != 0 {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::FALLING_FACING_LEFT
            );
            session_slot!(game, Dachora[me].facing_right = 0);
        } else {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::FALLING_FACING_RIGHT
            );
            session_slot!(game, Dachora[me].facing_right = 1);
        }
        session_slot!(game, Dachora[me].function = function::FALLING);
        session_slot!(game, Dachora[me].base.instruction_timer = 1);
        session_slot!(game, Dachora[me]; speed = 0, subspeed = 0, shine = 0);
        load_palette(list::PALETTE, k, game);
        game.queue_sound(1, LAND_SOUND, 6);
    }
}

/// Falls to the floor, then runs the way it faces. Was `Dachora_Func_11`.
pub fn falling<L: Lease>(game: &mut L, k: u16) {
    let me = k as i32;
    let pull = gravity(game);
    session_slot!(game, Dachora[me](speed, subspeed) += pull);
    let mut amount = (((session_slot!(game, Dachora[me].speed) as u32) << 16)
        | session_slot!(game, Dachora[me].subspeed) as u32) as i32;
    if !below((amount >> 16) as u16, FALL_TOP_SPEED) {
        amount = (FALL_TOP_SPEED as i32) << 16;
    }
    if game.move_vertical(k, amount) {
        if session_slot!(game, Dachora[me].facing_right) != 0 {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::RUNNING_RIGHT
            );
            session_slot!(game, Dachora[me].function = function::RUNNING_RIGHT);
        } else {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::RUNNING_LEFT
            );
            session_slot!(game, Dachora[me].function = function::RUNNING_LEFT);
        }
        session_slot!(game, Dachora[me].base.instruction_timer = 1);
        session_slot!(game, Dachora[me]; speed = 0, subspeed = 0);
    }
}

/// An echo shows on alternate frames while it lasts, then hides. Was
/// `Dachora_Func_12`.
pub fn echo<L: Lease>(game: &mut L, k: u16) {
    let me = k as i32;
    let lifetime: u16 = session_slot!(game, Echo[me].lifetime);
    if lifetime != 0 {
        session_slot!(game, Echo[me].lifetime = lifetime.wrapping_sub(1));
        let odd_frame = (game.frame() & 1) != 0;
        // Odd echo slots show on odd frames, the others on even frames.
        let keep_hidden = if (k & 0x40) != 0 {
            !odd_frame
        } else {
            odd_frame
        };
        if !keep_hidden {
            session_slot!(game, Echo[me].base.properties, |old| (old as i32
                & !(K_ENEMY_PROPS_INVISIBLE as i32))
                as u16);
            return;
        }
    }
    session_slot!(game, Echo[me].base.properties, |old| old
        | (K_ENEMY_PROPS_INVISIBLE as u16));
}

/// Speeds the run up to the top speed, shining once there, while falling
/// onto the floor; the horizontal step. Was `Dachora_Func_6`.
fn accelerate<L: Lease>(game: &mut L, k: u16) -> i32 {
    let me = k as i32;
    if !below(session_slot!(game, Dachora[me].speed), TOP_SPEED) {
        if session_slot!(game, Dachora[me].shine) == 1 {
            game.queue_sound(1, SHINE_SOUND, 6);
        }
        let shine: u16 = (session_slot!(game, Dachora[me].shine)).wrapping_sub(1);
        session_slot!(game, Dachora[me].shine = shine);
        if shine as u8 == 0 {
            let palette = crate::boss_lists::dachora_palette(
                game,
                false,
                session_slot!(byte game, Dachora[me].shine[1]) as isize,
            );
            load_palette(palette, k, game);
            let next: u16 = (session_slot!(game, Dachora[me].shine)).wrapping_add(0x110);
            session_slot!(game, Dachora[me].shine = next);
            if !below(next, 0x410) {
                session_slot!(game, Dachora[me].shine = 0x310);
            }
        }
    }
    game.move_vertical(k, (1u32 << 16) as i32);
    if !below(session_slot!(game, Dachora[me].speed), TOP_SPEED)
        && (session_slot!(game, Dachora[me].subspeed) as i16) >= 0
    {
        session_slot!(game, Dachora[me]; speed = TOP_SPEED, subspeed = 0);
        return ((TOP_SPEED as u32) << 16) as i32;
    }
    session_slot!(game, Dachora[me](speed, subspeed) += RUN_ACCELERATION);
    if (session_slot!(game, Dachora[me].speed) == 4 || session_slot!(game, Dachora[me].speed) == 8)
        && session_slot!(game, Dachora[me].subspeed) == 0
    {
        session_slot!(game, Dachora[me].base.current_instruction, |old| old
            .wrapping_add(FASTER_RUN_STEP));
    }
    (((session_slot!(game, Dachora[me].speed) as u32) << 16)
        | session_slot!(game, Dachora[me].subspeed) as u32) as i32
}

/// Places the next free echo where the dachora is, one each interval. Was
/// `Dachora_Func_9`.
fn update_echoes<L: Lease>(game: &mut L, k: u16) {
    let me = k as i32;
    let first = me + ECHOES[0];
    let timer: u16 = session_slot!(game, Echo[first].spawn_timer);
    if timer != 0 {
        session_slot!(game, Echo[first].spawn_timer = timer.wrapping_sub(1));
        return;
    }
    session_slot!(game, Echo[first].spawn_timer = ECHO_INTERVAL);
    for echo in ECHOES {
        let slot = me + echo;
        if session_slot!(game, Echo[slot].lifetime) == 0 {
            let copied = session_slot!(game, Dachora[me].base.x_pos);
            session_slot!(game, Echo[slot].base.x_pos = copied);
            let copied = session_slot!(game, Dachora[me].base.y_pos);
            session_slot!(game, Echo[slot].base.y_pos = copied);
            session_slot!(game, Echo[slot].lifetime = ECHO_LIFETIME);
            return;
        }
    }
}

/// Steps the shinespark palette, cycling through four. Was `Dachora_Func_10`.
fn load_shine_palette<L: Lease>(game: &mut L, k: u16) {
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
