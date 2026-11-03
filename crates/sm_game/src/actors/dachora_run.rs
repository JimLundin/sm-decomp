//! Dachora's run and shinespark: it waits for Samus, blinks, runs back and
//! forth gaining speed and shine, charges at the room's right end and
//! shinesparks upward trailing four echoes (bank $A7:F4DD-$A7:F9C1).
//!
//! Was `Dachora_*` in the original decompilation. The dachora and its echoes
//! run the bank $A7 function in their `function` word, whose ids are the
//! original's.
use crate::actors::boss_lease::BossLease;
use crate::actors::dachora::load_palette;
use crate::actors::host::{Axis, BossEffects, Slopes};
use crate::actors::slot_view::{SlotState, slot_view};
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_DISABLE_SAMUS_COLL, K_ENEMY_PROPS_INVISIBLE,
};

slot_view! {
    /// The dachora's slot. `speed` is its 16.16 running speed, and while it
    /// shinesparks its upward speed; it also counts the frames before running
    /// and before the shinespark. `acceleration` is the shinespark's 16.16
    /// upward acceleration. `shine` counts down to the next shine palette in
    /// its low byte and holds the palette step in its high byte.
    /// `facing_right` is 1 facing right.
    Dachora, DachoraEdit {
        record: [
            speed = var_a, subspeed = var_b, acceleration = var_c, subacceleration = var_d,
            shine = var_e, function = var_f, facing_right = parameter_1,
            parameter_2 = parameter_2,
        ],
        extended: [],
    }
}

slot_view! {
    /// An echo's slot: how long it stays, and, in the first echo, the frames
    /// until the next echo is placed. `parameter_1` is negative, bit 0 facing
    /// right.
    Echo, EchoEdit {
        record: [
            spawn_timer = var_a, lifetime = var_d, function = var_f, parameter_1 = parameter_1,
        ],
        extended: [],
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
pub trait Lease:
    BossLease<Game: BossEffects> + crate::actors::boss_interaction::Access + SlotState
{
}
impl<T> Lease for T where
    T: BossLease<Game: BossEffects> + crate::actors::boss_interaction::Access + SlotState + ?Sized
{
}

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
const ECHOES: [u16; 4] = [0x40, 0x80, 0xc0, 0x100];
const SHINE_SOUND: u8 = 0x39;
const BLINK_SOUND: u8 = 0x1d;
const BUMP_SOUND: u8 = 0x71;
const CHARGE_SOUND: u8 = 0x3d;
const SHINESPARK_SOUND: u8 = 0x3b;
const LAND_SOUND: u8 = 0x3c;

fn gravity<L: Lease>(game: &L) -> u32 {
    let acceleration = &game.session().player.contact.y_acceleration;
    (u32::from(acceleration.whole) << 16) | u32::from(acceleration.fraction)
}

/// Initialisation AI: a dachora idles, an echo waits. Was `Dachora_Init`.
pub fn initialize<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    DachoraEdit::of(game, k).actor().properties |= K_ENEMY_PROPS_DISABLE_SAMUS_COLL as u16;
    DachoraEdit::of(game, k).actor().spritemap = list::NOTHING_SPRITEMAP;
    DachoraEdit::of(game, k).actor().instruction_delay = 1;
    DachoraEdit::of(game, k).actor().timer = 0;
    let kind: i16 = Dachora::of(game, k).facing_right() as i16;
    if kind < 0 {
        if (kind as u16 & 1) != 0 {
            DachoraEdit::of(game, k).actor().instruction = list::ECHO_FACING_RIGHT;
        } else {
            DachoraEdit::of(game, k).actor().instruction = list::ECHO_FACING_LEFT;
        }
        *DachoraEdit::of(game, k).function() = function::ECHO;
    } else {
        if kind != 0 {
            DachoraEdit::of(game, k).actor().instruction = list::IDLING_FACING_RIGHT;
        } else {
            DachoraEdit::of(game, k).actor().instruction = list::IDLING_FACING_LEFT;
        }
        *DachoraEdit::of(game, k).function() = function::WAIT_FOR_SAMUS;
    }
}

/// Main AI: the function. Was `Dachora_Main`.
pub fn main_ai<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    match Dachora::of(game, k).function() {
        function::WAIT_FOR_SAMUS => wait_for_samus(game, k),
        function::START_RUNNING => start_running(game, k),
        function::RUNNING_LEFT => running_left(game, k),
        function::RUNNING_RIGHT => running_right(game, k),
        function::ACTIVATE_SHINESPARK => activate_shinespark(game, k),
        function::SHINESPARKING => shinesparking(game, k),
        function::FALLING => falling(game, k),
        function::ECHO => echo(game, k),
        _ => game.foreign(|game| game.unreachable()),
    }
}

/// Falls to the floor and blinks once Samus is near. Was `Dachora_Func_2`.
pub fn wait_for_samus<L: Lease>(game: &mut L, k: u16) {
    game.move_vertical(k, (1u32 << 16) as i32);
    if game.player_within(k, Axis::Y, 0x40) && game.player_within(k, Axis::X, WAKE_DISTANCE) {
        if Dachora::of(game, k).facing_right() != 0 {
            DachoraEdit::of(game, k).actor().instruction = list::BLINKING_FACING_RIGHT;
        } else {
            DachoraEdit::of(game, k).actor().instruction = list::BLINKING_FACING_LEFT;
        }
        DachoraEdit::of(game, k).actor().instruction_delay = 1;
        *DachoraEdit::of(game, k).function() = function::START_RUNNING;
        *DachoraEdit::of(game, k).speed() = BLINK_FRAMES;
        game.queue_sound(1, BLINK_SOUND, 15);
    }
}

/// Blinks, then runs the way it faces. Was `Dachora_Func_3`.
pub fn start_running<L: Lease>(game: &mut L, k: u16) {
    let left = Dachora::of(game, k).speed();
    let copied = Dachora::of(game, k).speed().wrapping_sub(1);
    *DachoraEdit::of(game, k).speed() = copied;
    if left == 1 {
        if Dachora::of(game, k).facing_right() != 0 {
            DachoraEdit::of(game, k).actor().instruction = list::RUNNING_RIGHT;
            *DachoraEdit::of(game, k).function() = function::RUNNING_RIGHT;
        } else {
            DachoraEdit::of(game, k).actor().instruction = list::RUNNING_LEFT;
            *DachoraEdit::of(game, k).function() = function::RUNNING_LEFT;
        }
        DachoraEdit::of(game, k).actor().instruction_delay = 1;
        *DachoraEdit::of(game, k).shine() = 1;
    }
}

/// Runs left, turning at the wall or the room's left end. Was
/// `Dachora_Func_4`.
pub fn running_left<L: Lease>(game: &mut L, k: u16) {
    let amount: i32 = -accelerate(game, k);
    if game.move_horizontal(k, amount, Slopes::Ignore) || {
        game.snap_to_slopes(k);
        below(Dachora::of(game, k).actor().x.whole, LEFT_END_X)
    } {
        DachoraEdit::of(game, k).actor().instruction = list::RUNNING_RIGHT;
        *DachoraEdit::of(game, k).function() = function::RUNNING_RIGHT;
        *DachoraEdit::of(game, k).shine() = 1;
        *DachoraEdit::of(game, k).facing_right() = 1;
        DachoraEdit::of(game, k).actor().instruction_delay = 1;
        *DachoraEdit::of(game, k).speed() = 0;
        *DachoraEdit::of(game, k).subspeed() = 0;
        load_palette(list::PALETTE, k, game);
    }
}

/// Runs right, turning at a wall; at the room's right end it charges the
/// shinespark. Was `Dachora_Func_5`.
pub fn running_right<L: Lease>(game: &mut L, k: u16) {
    let amount: i32 = accelerate(game, k);
    if game.move_horizontal(k, amount, Slopes::Ignore) {
        game.queue_sound(1, BUMP_SOUND, 15);
        DachoraEdit::of(game, k).actor().instruction = list::RUNNING_LEFT;
        *DachoraEdit::of(game, k).function() = function::RUNNING_LEFT;
        *DachoraEdit::of(game, k).facing_right() = 0;
        *DachoraEdit::of(game, k).speed() = 0;
        load_palette(list::PALETTE, k, game);
    } else {
        game.snap_to_slopes(k);
        if !below(Dachora::of(game, k).actor().x.whole, RIGHT_END_X) {
            DachoraEdit::of(game, k).actor().instruction = list::CHARGE_SHINESPARK_FACING_RIGHT;
            *DachoraEdit::of(game, k).function() = function::ACTIVATE_SHINESPARK;
            *DachoraEdit::of(game, k).speed() = CHARGE_FRAMES;
            DachoraEdit::of(game, k).actor().y.whole =
                Dachora::of(game, k).actor().y.whole.wrapping_add(8);
            game.queue_sound(1, CHARGE_SOUND, 6);
        } else {
            return;
        }
    }
    *DachoraEdit::of(game, k).shine() = 0;
    *DachoraEdit::of(game, k).subspeed() = 0;
    DachoraEdit::of(game, k).actor().instruction_delay = 1;
}

/// Shines while it charges, then shinesparks with its echoes. Was
/// `Dachora_Func_7`.
pub fn activate_shinespark<L: Lease>(game: &mut L, k: u16) {
    load_shine_palette(game, k);
    let left = Dachora::of(game, k).speed();
    let copied = Dachora::of(game, k).speed().wrapping_sub(1);
    *DachoraEdit::of(game, k).speed() = copied;
    if left == 1 {
        DachoraEdit::of(game, k).actor().instruction =
            (Dachora::of(game, k).actor().instruction).wrapping_add(2);
        DachoraEdit::of(game, k).actor().instruction_delay = 1;
        *DachoraEdit::of(game, k).function() = function::SHINESPARKING;
        let first = k + ECHOES[0];
        *EchoEdit::of(game, first).spawn_timer() = 0;
        *EchoEdit::of(game, first).lifetime() = 0;
        for echo in &ECHOES[1..] {
            *EchoEdit::of(game, k + echo).lifetime() = 0;
        }
        *DachoraEdit::of(game, k).acceleration() = 0;
        *DachoraEdit::of(game, k).subacceleration() = 0;
        DachoraEdit::of(game, k).actor().y.whole =
            Dachora::of(game, k).actor().y.whole.wrapping_sub(8);
        game.queue_sound(1, SHINESPARK_SOUND, 6);
        let echo_list = if Dachora::of(game, k).facing_right() != 0 {
            list::ECHO_FACING_RIGHT
        } else {
            list::ECHO_FACING_LEFT
        };
        for echo in ECHOES {
            EchoEdit::of(game, k + echo).actor().instruction = echo_list;
        }
        for echo in ECHOES {
            EchoEdit::of(game, k + echo).actor().instruction_delay = 1;
        }
    }
}

/// Shinesparks upward until it hits the ceiling, then falls. Was
/// `Dachora_Func_8`.
pub fn shinesparking<L: Lease>(game: &mut L, k: u16) {
    load_shine_palette(game, k);
    update_echoes(game, k);
    let pull = gravity(game);
    {
        let mut edit = DachoraEdit::of(game, k);
        let sum = ((u32::from(*edit.acceleration()) << 16) | u32::from(*edit.subacceleration()))
            .wrapping_add(pull);
        *edit.subacceleration() = sum as u16;
        *edit.acceleration() = (sum >> 16) as u16;
    }
    let acceleration = ((Dachora::of(game, k).acceleration() as u32) << 16)
        | Dachora::of(game, k).subacceleration() as u32;
    {
        let mut edit = DachoraEdit::of(game, k);
        let sum = ((u32::from(*edit.speed()) << 16) | u32::from(*edit.subspeed()))
            .wrapping_add(acceleration);
        *edit.subspeed() = sum as u16;
        *edit.speed() = (sum >> 16) as u16;
    }
    let mut speed: u16 = Dachora::of(game, k).speed();
    if !below(speed, SHINESPARK_TOP_SPEED) {
        speed = SHINESPARK_TOP_SPEED;
    }
    let amount = (((speed as u32) << 16) | Dachora::of(game, k).subspeed() as u32) as i32;
    if game.move_vertical(k, -amount) {
        if Dachora::of(game, k).facing_right() != 0 {
            DachoraEdit::of(game, k).actor().instruction = list::FALLING_FACING_LEFT;
            *DachoraEdit::of(game, k).facing_right() = 0;
        } else {
            DachoraEdit::of(game, k).actor().instruction = list::FALLING_FACING_RIGHT;
            *DachoraEdit::of(game, k).facing_right() = 1;
        }
        *DachoraEdit::of(game, k).function() = function::FALLING;
        DachoraEdit::of(game, k).actor().instruction_delay = 1;
        *DachoraEdit::of(game, k).speed() = 0;
        *DachoraEdit::of(game, k).subspeed() = 0;
        *DachoraEdit::of(game, k).shine() = 0;
        load_palette(list::PALETTE, k, game);
        game.queue_sound(1, LAND_SOUND, 6);
    }
}

/// Falls to the floor, then runs the way it faces. Was `Dachora_Func_11`.
pub fn falling<L: Lease>(game: &mut L, k: u16) {
    let pull = gravity(game);
    {
        let mut edit = DachoraEdit::of(game, k);
        let sum =
            ((u32::from(*edit.speed()) << 16) | u32::from(*edit.subspeed())).wrapping_add(pull);
        *edit.subspeed() = sum as u16;
        *edit.speed() = (sum >> 16) as u16;
    }
    let mut amount = (((Dachora::of(game, k).speed() as u32) << 16)
        | Dachora::of(game, k).subspeed() as u32) as i32;
    if !below((amount >> 16) as u16, FALL_TOP_SPEED) {
        amount = (FALL_TOP_SPEED as i32) << 16;
    }
    if game.move_vertical(k, amount) {
        if Dachora::of(game, k).facing_right() != 0 {
            DachoraEdit::of(game, k).actor().instruction = list::RUNNING_RIGHT;
            *DachoraEdit::of(game, k).function() = function::RUNNING_RIGHT;
        } else {
            DachoraEdit::of(game, k).actor().instruction = list::RUNNING_LEFT;
            *DachoraEdit::of(game, k).function() = function::RUNNING_LEFT;
        }
        DachoraEdit::of(game, k).actor().instruction_delay = 1;
        *DachoraEdit::of(game, k).speed() = 0;
        *DachoraEdit::of(game, k).subspeed() = 0;
    }
}

/// An echo shows on alternate frames while it lasts, then hides. Was
/// `Dachora_Func_12`.
pub fn echo<L: Lease>(game: &mut L, k: u16) {
    let lifetime: u16 = Echo::of(game, k).lifetime();
    if lifetime != 0 {
        *EchoEdit::of(game, k).lifetime() = lifetime.wrapping_sub(1);
        let odd_frame = (game.frame() & 1) != 0;
        // Odd echo slots show on odd frames, the others on even frames.
        let keep_hidden = if (k & 0x40) != 0 {
            !odd_frame
        } else {
            odd_frame
        };
        if !keep_hidden {
            EchoEdit::of(game, k).actor().properties = (Echo::of(game, k).actor().properties as i32
                & !(K_ENEMY_PROPS_INVISIBLE as i32))
                as u16;
            return;
        }
    }
    EchoEdit::of(game, k).actor().properties |= K_ENEMY_PROPS_INVISIBLE as u16;
}

/// Speeds the run up to the top speed, shining once there, while falling
/// onto the floor; the horizontal step. Was `Dachora_Func_6`.
fn accelerate<L: Lease>(game: &mut L, k: u16) -> i32 {
    crate::probe::hit("dachora.accelerate");
    if !below(Dachora::of(game, k).speed(), TOP_SPEED) {
        if Dachora::of(game, k).shine() == 1 {
            game.queue_sound(1, SHINE_SOUND, 6);
        }
        let shine: u16 = (Dachora::of(game, k).shine()).wrapping_sub(1);
        *DachoraEdit::of(game, k).shine() = shine;
        if shine as u8 == 0 {
            let palette = crate::boss_lists::dachora_palette(
                game,
                false,
                Dachora::of(game, k).shine().to_le_bytes()[1] as isize,
            );
            load_palette(palette, k, game);
            let next: u16 = (Dachora::of(game, k).shine()).wrapping_add(0x110);
            *DachoraEdit::of(game, k).shine() = next;
            if !below(next, 0x410) {
                *DachoraEdit::of(game, k).shine() = 0x310;
            }
        }
    }
    game.move_vertical(k, (1u32 << 16) as i32);
    if !below(Dachora::of(game, k).speed(), TOP_SPEED)
        && (Dachora::of(game, k).subspeed() as i16) >= 0
    {
        *DachoraEdit::of(game, k).speed() = TOP_SPEED;
        *DachoraEdit::of(game, k).subspeed() = 0;
        return ((TOP_SPEED as u32) << 16) as i32;
    }
    {
        let mut edit = DachoraEdit::of(game, k);
        let sum = ((u32::from(*edit.speed()) << 16) | u32::from(*edit.subspeed()))
            .wrapping_add(RUN_ACCELERATION);
        *edit.subspeed() = sum as u16;
        *edit.speed() = (sum >> 16) as u16;
    }
    if (Dachora::of(game, k).speed() == 4 || Dachora::of(game, k).speed() == 8)
        && Dachora::of(game, k).subspeed() == 0
    {
        DachoraEdit::of(game, k).actor().instruction = Dachora::of(game, k)
            .actor()
            .instruction
            .wrapping_add(FASTER_RUN_STEP);
    }
    (((Dachora::of(game, k).speed() as u32) << 16) | Dachora::of(game, k).subspeed() as u32) as i32
}

/// Places the next free echo where the dachora is, one each interval. Was
/// `Dachora_Func_9`.
fn update_echoes<L: Lease>(game: &mut L, k: u16) {
    let first = k + ECHOES[0];
    let timer: u16 = Echo::of(game, first).spawn_timer();
    if timer != 0 {
        *EchoEdit::of(game, first).spawn_timer() = timer.wrapping_sub(1);
        return;
    }
    *EchoEdit::of(game, first).spawn_timer() = ECHO_INTERVAL;
    for echo in ECHOES {
        let slot = k + echo;
        if Echo::of(game, slot).lifetime() == 0 {
            let copied = Dachora::of(game, k).actor().x.whole;
            EchoEdit::of(game, slot).actor().x.whole = copied;
            let copied = Dachora::of(game, k).actor().y.whole;
            EchoEdit::of(game, slot).actor().y.whole = copied;
            *EchoEdit::of(game, slot).lifetime() = ECHO_LIFETIME;
            return;
        }
    }
}

/// Steps the shinespark palette, cycling through four. Was `Dachora_Func_10`.
fn load_shine_palette<L: Lease>(game: &mut L, k: u16) {
    let palette = crate::boss_lists::dachora_palette(
        game,
        true,
        Dachora::of(game, k).shine().to_le_bytes()[1] as isize,
    );
    load_palette(palette, k, game);
    let next: u16 = (Dachora::of(game, k).shine()).wrapping_add(256);
    *DachoraEdit::of(game, k).shine() = next;
    if !below(next, 0x400) {
        *DachoraEdit::of(game, k).shine() = 0;
    }
}
