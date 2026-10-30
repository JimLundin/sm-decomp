//! Dachora's run and shinespark: it waits for Samus, blinks, runs back and
//! forth gaining speed and shine, charges at the room's right end and
//! shinesparks upward trailing four echoes (bank $A7:F4DD-$A7:F9C1).
//!
//! Was `Dachora_*` in the original decompilation. The dachora and its echoes
//! run the bank $A7 function in their `function` word; the compat host
//! dispatches it, so the function ids are unchanged.
#![forbid(unsafe_code)]
use crate::actors::dachora::load_palette;
use crate::actors::host::{Axis, Slopes};
use crate::actors::layouts::part_view;
use crate::constants::enemy_properties::{kEnemyProps_DisableSamusColl, kEnemyProps_Invisible};
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

/// What the dachora needs beyond the common actor operations and sounds.
pub trait Host:
    crate::actors::host::ActorHost
    + crate::actors::host::ActorRuntime
    + crate::effects::SoundEffects
    + crate::records::Records
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
        words: [spawn_timer, var_B, var_C, lifetime, var_E, function, parameter_1, parameter_2],
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

use instruction_list as list;

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

/// Whether `value` is below `limit`, as the original's signed 16-bit difference.
fn below(value: u16, limit: u16) -> bool {
    (value.wrapping_sub(limit) as i16) < 0
}

fn gravity<H: Host + ?Sized>(game: &H) -> u32 {
    ((word!(game.samus_y_accel) as u32) << 16) | word!(game.samus_y_subaccel) as u32
}

/// Initialisation AI: a dachora idles, an echo waits. Was `Dachora_Init`.
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    let me = word!(game.cur_enemy_index) as i32;
    slot!(
        game,
        Dachora[me].base.properties =
            (slot!(game, Dachora[me].base.properties)) | (kEnemyProps_DisableSamusColl as u16)
    );
    slot!(
        game,
        Dachora[me].base.spritemap_pointer = list::NOTHING_SPRITEMAP
    );
    slot!(game, Dachora[me].base.instruction_timer = 1);
    slot!(game, Dachora[me].base.timer = 0);
    let kind: i16 = slot!(game, Dachora[me].facing_right) as i16;
    if kind < 0 {
        if (kind as u16 & 1) != 0 {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::ECHO_FACING_RIGHT
            );
        } else {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::ECHO_FACING_LEFT
            );
        }
        slot!(game, Dachora[me].function = function::ECHO);
    } else {
        if kind != 0 {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::IDLING_FACING_RIGHT
            );
        } else {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::IDLING_FACING_LEFT
            );
        }
        slot!(game, Dachora[me].function = function::WAIT_FOR_SAMUS);
    }
}

/// Main AI: the function. Was `Dachora_Main`.
pub fn main_ai<H: Host + ?Sized>(game: &mut H) {
    let me = word!(game.cur_enemy_index) as i32;
    let address = (BANK << 16) | slot!(game, Dachora[me].function) as u32;
    game.run_function(address);
}

/// Falls to the floor and blinks once Samus is near. Was `Dachora_Func_2`.
pub fn wait_for_samus<H: Host + ?Sized>(game: &mut H, k: u16) {
    game.move_vertical(k, (1u32 << 16) as i32);
    if game.player_within(k, Axis::Y, 0x40) && game.player_within(k, Axis::X, WAKE_DISTANCE) {
        let me = k as i32;
        if slot!(game, Dachora[me].facing_right) != 0 {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::BLINKING_FACING_RIGHT
            );
        } else {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::BLINKING_FACING_LEFT
            );
        }
        slot!(game, Dachora[me].base.instruction_timer = 1);
        slot!(game, Dachora[me].function = function::START_RUNNING);
        slot!(game, Dachora[me].speed = BLINK_FRAMES);
        game.queue_sound(1, BLINK_SOUND, 15);
    }
}

/// Blinks, then runs the way it faces. Was `Dachora_Func_3`.
pub fn start_running<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    let left = slot!(game, Dachora[me].speed);
    slot!(
        game,
        Dachora[me].speed = slot!(game, Dachora[me].speed).wrapping_sub(1)
    );
    if left == 1 {
        if slot!(game, Dachora[me].facing_right) != 0 {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::RUNNING_RIGHT
            );
            slot!(game, Dachora[me].function = function::RUNNING_RIGHT);
        } else {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::RUNNING_LEFT
            );
            slot!(game, Dachora[me].function = function::RUNNING_LEFT);
        }
        slot!(game, Dachora[me].base.instruction_timer = 1);
        slot!(game, Dachora[me].shine = 1);
    }
}

/// Runs left, turning at the wall or the room's left end. Was
/// `Dachora_Func_4`.
pub fn running_left<H: Host + ?Sized>(game: &mut H, k: u16) {
    let amount: i32 = -accelerate(game, k);
    let me = k as i32;
    if game.move_horizontal(k, amount, Slopes::Ignore) || {
        game.snap_to_slopes(k);
        below(slot!(game, Dachora[me].base.x_pos), LEFT_END_X)
    } {
        slot!(
            game,
            Dachora[me].base.current_instruction = list::RUNNING_RIGHT
        );
        slot!(game, Dachora[me].function = function::RUNNING_RIGHT);
        slot!(game, Dachora[me]; shine = 1, facing_right = 1);
        slot!(game, Dachora[me].base.instruction_timer = 1);
        slot!(game, Dachora[me]; speed = 0, subspeed = 0);
        load_palette(list::PALETTE, k, game);
    }
}

/// Runs right, turning at a wall; at the room's right end it charges the
/// shinespark. Was `Dachora_Func_5`.
pub fn running_right<H: Host + ?Sized>(game: &mut H, k: u16) {
    let amount: i32 = accelerate(game, k);
    let me = k as i32;
    if game.move_horizontal(k, amount, Slopes::Ignore) {
        game.queue_sound(1, BUMP_SOUND, 15);
        slot!(
            game,
            Dachora[me].base.current_instruction = list::RUNNING_LEFT
        );
        slot!(game, Dachora[me].function = function::RUNNING_LEFT);
        slot!(game, Dachora[me]; facing_right = 0, speed = 0);
        load_palette(list::PALETTE, k, game);
    } else {
        game.snap_to_slopes(k);
        if !below(slot!(game, Dachora[me].base.x_pos), RIGHT_END_X) {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::CHARGE_SHINESPARK_FACING_RIGHT
            );
            slot!(game, Dachora[me].function = function::ACTIVATE_SHINESPARK);
            slot!(game, Dachora[me].speed = CHARGE_FRAMES);
            slot!(
                game,
                Dachora[me].base.y_pos = (slot!(game, Dachora[me].base.y_pos)).wrapping_add(8)
            );
            game.queue_sound(1, CHARGE_SOUND, 6);
        } else {
            return;
        }
    }
    slot!(game, Dachora[me]; shine = 0, subspeed = 0);
    slot!(game, Dachora[me].base.instruction_timer = 1);
}

/// Shines while it charges, then shinesparks with its echoes. Was
/// `Dachora_Func_7`.
pub fn activate_shinespark<H: Host + ?Sized>(game: &mut H, k: u16) {
    load_shine_palette(game, k);
    let me = k as i32;
    let left = slot!(game, Dachora[me].speed);
    slot!(
        game,
        Dachora[me].speed = slot!(game, Dachora[me].speed).wrapping_sub(1)
    );
    if left == 1 {
        slot!(game, Dachora[me].base; current_instruction = (slot!(game, Dachora[me].base.current_instruction)).wrapping_add(2), instruction_timer = 1);
        slot!(game, Dachora[me].function = function::SHINESPARKING);
        let first = me + ECHOES[0];
        slot!(game, Echo[first]; spawn_timer = 0, lifetime = 0);
        for echo in &ECHOES[1..] {
            slot!(game, Echo[me + echo].lifetime = 0);
        }
        slot!(game, Dachora[me]; acceleration = 0, subacceleration = 0);
        slot!(
            game,
            Dachora[me].base.y_pos = (slot!(game, Dachora[me].base.y_pos)).wrapping_sub(8)
        );
        game.queue_sound(1, SHINESPARK_SOUND, 6);
        let echo_list = if slot!(game, Dachora[me].facing_right) != 0 {
            list::ECHO_FACING_RIGHT
        } else {
            list::ECHO_FACING_LEFT
        };
        for echo in ECHOES {
            slot!(game, Echo[me + echo].base.current_instruction = echo_list);
        }
        for echo in ECHOES {
            slot!(game, Echo[me + echo].base.instruction_timer = 1);
        }
    }
}

/// Shinesparks upward until it hits the ceiling, then falls. Was
/// `Dachora_Func_8`.
pub fn shinesparking<H: Host + ?Sized>(game: &mut H, k: u16) {
    load_shine_palette(game, k);
    update_echoes(game, k);
    let me = k as i32;
    let pull = gravity(game);
    slot!(game, Dachora[me](acceleration, subacceleration) += pull);
    let acceleration = ((slot!(game, Dachora[me].acceleration) as u32) << 16)
        | slot!(game, Dachora[me].subacceleration) as u32;
    slot!(game, Dachora[me](speed, subspeed) += acceleration);
    let mut speed: u16 = slot!(game, Dachora[me].speed);
    if !below(speed, SHINESPARK_TOP_SPEED) {
        speed = SHINESPARK_TOP_SPEED;
    }
    let amount = (((speed as u32) << 16) | slot!(game, Dachora[me].subspeed) as u32) as i32;
    if game.move_vertical(k, -amount) {
        if slot!(game, Dachora[me].facing_right) != 0 {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::FALLING_FACING_LEFT
            );
            slot!(game, Dachora[me].facing_right = 0);
        } else {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::FALLING_FACING_RIGHT
            );
            slot!(game, Dachora[me].facing_right = 1);
        }
        slot!(game, Dachora[me].function = function::FALLING);
        slot!(game, Dachora[me].base.instruction_timer = 1);
        slot!(game, Dachora[me]; speed = 0, subspeed = 0, shine = 0);
        load_palette(list::PALETTE, k, game);
        game.queue_sound(1, LAND_SOUND, 6);
    }
}

/// Falls to the floor, then runs the way it faces. Was `Dachora_Func_11`.
pub fn falling<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    let pull = gravity(game);
    slot!(game, Dachora[me](speed, subspeed) += pull);
    let mut amount = (((slot!(game, Dachora[me].speed) as u32) << 16)
        | slot!(game, Dachora[me].subspeed) as u32) as i32;
    if !below((amount >> 16) as u16, FALL_TOP_SPEED) {
        amount = (FALL_TOP_SPEED as i32) << 16;
    }
    if game.move_vertical(k, amount) {
        if slot!(game, Dachora[me].facing_right) != 0 {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::RUNNING_RIGHT
            );
            slot!(game, Dachora[me].function = function::RUNNING_RIGHT);
        } else {
            slot!(
                game,
                Dachora[me].base.current_instruction = list::RUNNING_LEFT
            );
            slot!(game, Dachora[me].function = function::RUNNING_LEFT);
        }
        slot!(game, Dachora[me].base.instruction_timer = 1);
        slot!(game, Dachora[me]; speed = 0, subspeed = 0);
    }
}

/// An echo shows on alternate frames while it lasts, then hides. Was
/// `Dachora_Func_12`.
pub fn echo<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    let lifetime: u16 = slot!(game, Echo[me].lifetime);
    if lifetime != 0 {
        slot!(game, Echo[me].lifetime = lifetime.wrapping_sub(1));
        let odd_frame = (word!(game.nmi_frame_counter_word) & 1) != 0;
        // Odd echo slots show on odd frames, the others on even frames.
        let keep_hidden = if (k & 0x40) != 0 {
            !odd_frame
        } else {
            odd_frame
        };
        if !keep_hidden {
            slot!(
                game,
                Echo[me].base.properties = (slot!(game, Echo[me].base.properties) as i32
                    & !(kEnemyProps_Invisible as i32))
                    as u16
            );
            return;
        }
    }
    slot!(
        game,
        Echo[me].base.properties =
            (slot!(game, Echo[me].base.properties)) | (kEnemyProps_Invisible as u16)
    );
}

/// Speeds the run up to the top speed, shining once there, while falling
/// onto the floor; the horizontal step. Was `Dachora_Func_6`.
fn accelerate<H: Host + ?Sized>(game: &mut H, k: u16) -> i32 {
    let me = k as i32;
    if !below(slot!(game, Dachora[me].speed), TOP_SPEED) {
        if slot!(game, Dachora[me].shine) == 1 {
            game.queue_sound(1, SHINE_SOUND, 6);
        }
        let shine: u16 = (slot!(game, Dachora[me].shine)).wrapping_sub(1);
        slot!(game, Dachora[me].shine = shine);
        if shine as u8 == 0 {
            let palette = crate::boss_lists::dachora_palette(
                game,
                false,
                slot!(byte game, Dachora[me].shine[1]) as isize,
            );
            load_palette(palette, k, game);
            let next: u16 = (slot!(game, Dachora[me].shine)).wrapping_add(0x110);
            slot!(game, Dachora[me].shine = next);
            if !below(next, 0x410) {
                slot!(game, Dachora[me].shine = 0x310);
            }
        }
    }
    game.move_vertical(k, (1u32 << 16) as i32);
    if !below(slot!(game, Dachora[me].speed), TOP_SPEED)
        && (slot!(game, Dachora[me].subspeed) as i16) >= 0
    {
        slot!(game, Dachora[me].speed = TOP_SPEED);
        slot!(game, Dachora[me].subspeed = 0);
        return ((TOP_SPEED as u32) << 16) as i32;
    }
    slot!(game, Dachora[me](speed, subspeed) += RUN_ACCELERATION);
    if (slot!(game, Dachora[me].speed) == 4 || slot!(game, Dachora[me].speed) == 8)
        && slot!(game, Dachora[me].subspeed) == 0
    {
        slot!(
            game,
            Dachora[me].base.current_instruction =
                (slot!(game, Dachora[me].base.current_instruction)).wrapping_add(FASTER_RUN_STEP)
        );
    }
    (((slot!(game, Dachora[me].speed) as u32) << 16) | slot!(game, Dachora[me].subspeed) as u32)
        as i32
}

/// Places the next free echo where the dachora is, one each interval. Was
/// `Dachora_Func_9`.
fn update_echoes<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    let first = me + ECHOES[0];
    let timer: u16 = slot!(game, Echo[first].spawn_timer);
    if timer != 0 {
        slot!(game, Echo[first].spawn_timer = timer.wrapping_sub(1));
        return;
    }
    slot!(game, Echo[first].spawn_timer = ECHO_INTERVAL);
    for echo in ECHOES {
        let slot = me + echo;
        if slot!(game, Echo[slot].lifetime) == 0 {
            slot!(
                game,
                Echo[slot].base.x_pos = slot!(game, Dachora[me].base.x_pos)
            );
            slot!(
                game,
                Echo[slot].base.y_pos = slot!(game, Dachora[me].base.y_pos)
            );
            slot!(game, Echo[slot].lifetime = ECHO_LIFETIME);
            return;
        }
    }
}

/// Steps the shinespark palette, cycling through four. Was `Dachora_Func_10`.
fn load_shine_palette<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    let palette = crate::boss_lists::dachora_palette(
        game,
        true,
        slot!(byte game, Dachora[me].shine[1]) as isize,
    );
    load_palette(palette, k, game);
    let next: u16 = (slot!(game, Dachora[me].shine)).wrapping_add(256);
    slot!(game, Dachora[me].shine = next);
    if !below(next, 0x400) {
        slot!(game, Dachora[me].shine = 0);
    }
}
