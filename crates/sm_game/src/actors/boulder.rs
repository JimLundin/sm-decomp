//! The boulder: it waits for Samus to pass below, drops, bounces twice and
//! rolls until it hits a wall. The shattering kind breaks on its first landing.
//!
//! Was `Boulder` in the original decompilation (bank $A6:86F5-$A6:89FC). Its
//! state words name the slot fields over the same RAM bytes as `EnemyLayout40`.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Axis, Slopes};
use crate::actors::slot_view::{SlotState, slot_view};
use crate::constants::enemy_properties::{K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_INVISIBLE};
use sm_data::enemy_speeds::SpeedTable;

/// What the boulder needs beyond the shared boss effects: the 32-bit speeds.
pub trait Lease: BossLease + SlotState {
    /// The native-order 32-bit enemy speed at byte `offset` from $A0:CBC7.
    fn enemy_speed32(&self, offset: usize) -> u32;
}

slot_view! {
    /// The boulder's slot: its bank $A6 `state` function, one of [`state`];
    /// the rolling and vertical speeds, whose high bytes select the speed
    /// table row; the landings left before it starts rolling; its [`kind`]:
    /// which way it rolls, and whether it shatters; the signed horizontal
    /// distance from Samus that releases it; how far each rolling frame
    /// lifts it before gravity pulls it down; its previous position; its home
    /// x; the height it lands at; how far below it Samus must be to release
    /// it; and whether it starts on the ground and rolls straight away.
    Boulder, BoulderEdit {
        record: [
            state = var_a, roll_speed = var_b, fall_speed = var_c, bounces_left = var_d,
            kind = var_e, parameter_1 = parameter_1, parameter_2 = parameter_2,
        ],
        extended: [
            trigger_distance = var_00, roll_lift = var_01, previous_y_subpos = var_02,
            previous_y = var_03, home_x = var_04, ground_y = var_05, trigger_height = var_06,
            starts_rolling = var_07,
        ],
    }
}

/// Bank $A6 state functions, stored in the `state` word.
pub mod state {
    pub const WAIT_FOR_SAMUS: u16 = 0x879a;
    pub const FALL: u16 = 0x87ed;
    pub const BOUNCE_RISE: u16 = 0x8832;
    pub const BOUNCE_FALL: u16 = 0x888b;
    pub const ROLL: u16 = 0x8942;
    pub const STOPPED: u16 = 0x89fc;
}

/// The `kind` word: the parameter 1 high byte.
pub mod kind {
    /// Rolls right.
    pub const ROLL_RIGHT: u16 = 0;
    /// Rolls left and breaks into dust on its first landing.
    pub const SHATTER: u16 = 2;
}

/// Bank $A6 instruction lists.
mod animation {
    pub const ROLL_LEFT: u16 = 0x86a7;
    pub const ROLL_RIGHT: u16 = 0x86cb;
}

/// The dust cloud projectile and its parameter.
const DUST_CLOUD: u16 = 0xe509;
const DUST_CLOUD_PARAMETER: u16 = 0x11;
/// The sounds of landing and of breaking apart.
const LAND_SOUND: u8 = 0x42;
const SHATTER_SOUND: u8 = 0x43;
/// Speeds the boulder bounces back up with, by landings left.
const BOUNCE_SPEEDS: [u16; 2] = [0x1000, 0x1800];
/// Speed caps: falling and bounce rolls, then the final roll.
const FALL_SPEED_CAP: u16 = 0x5000;
const BOUNCE_ROLL_CAP: u16 = 0x5000;
const ROLL_SPEED_CAP: u16 = 0x4000;

/// Sets up a boulder from its parameters. Was `Boulder_Init`.
pub fn initialize<L: Lease + ?Sized>(game: &mut L) {
    let k = game.current_enemy();
    *BoulderEdit::of(game, k).fall_speed() = 0;
    *BoulderEdit::of(game, k).roll_speed() = 0;
    *BoulderEdit::of(game, k).bounces_left() = 2;
    *BoulderEdit::of(game, k).state() = state::WAIT_FOR_SAMUS;
    *BoulderEdit::of(game, k).home_x() = Boulder::of(game, k).actor().x.whole;
    let y_pos: u16 = Boulder::of(game, k).actor().y.whole;
    *BoulderEdit::of(game, k).ground_y() = y_pos;
    *BoulderEdit::of(game, k).previous_y() = y_pos;
    *BoulderEdit::of(game, k).previous_y_subpos() = Boulder::of(game, k).actor().y.fraction;
    let mut drop_height: i16 = Boulder::of(game, k).parameter_2().to_le_bytes()[1] as i16;
    if Boulder::of(game, k).parameter_2().to_le_bytes()[1] == 0 {
        drop_height = 1;
        *BoulderEdit::of(game, k).starts_rolling() = 1;
    }
    BoulderEdit::of(game, k).actor().y.whole =
        (Boulder::of(game, k).ground_y()).wrapping_sub(drop_height as u16);
    *BoulderEdit::of(game, k).trigger_distance() =
        -(Boulder::of(game, k).parameter_2().to_le_bytes()[0] as i32) as u16;
    *BoulderEdit::of(game, k).trigger_height() =
        Boulder::of(game, k).actor().instruction.to_le_bytes()[0] as u16;
    BoulderEdit::of(game, k).actor().instruction = animation::ROLL_LEFT;
    let kind: u16 = Boulder::of(game, k).parameter_1().to_le_bytes()[1] as u16;
    *BoulderEdit::of(game, k).kind() = kind;
    if kind == kind::ROLL_RIGHT {
        *BoulderEdit::of(game, k).trigger_distance() =
            -(Boulder::of(game, k).trigger_distance() as i32) as u16;
        BoulderEdit::of(game, k).actor().instruction = animation::ROLL_RIGHT;
    }
    *BoulderEdit::of(game, k).roll_lift() = 2;
    if Boulder::of(game, k).parameter_1().to_le_bytes()[0] != 0 {
        *BoulderEdit::of(game, k).roll_lift() = 5;
    }
}

/// Runs the current state. Was `Boulder_Main`.
pub fn main_ai<L: Lease + ?Sized>(game: &mut L) {
    crate::probe::hit("boulder.main_ai");
    let k = game.current_enemy();
    match Boulder::of(game, k).state() {
        state::WAIT_FOR_SAMUS => wait_for_samus(game),
        state::FALL => fall(game),
        state::BOUNCE_RISE => bounce_rise(game),
        state::BOUNCE_FALL => bounce_fall(game),
        state::ROLL => roll(game),
        state::STOPPED => {}
        _ => game.unreachable(),
    }
}

/// The state that follows a release: a drop, or a roll when on the ground.
fn release<L: Lease + ?Sized>(game: &mut L, k: u16) {
    *BoulderEdit::of(game, k).state() = state::FALL;
    if Boulder::of(game, k).starts_rolling() != 0 {
        *BoulderEdit::of(game, k).state() = state::ROLL;
    }
}

/// Waits for Samus to be below the boulder and close enough on its side.
/// Was `Boulder_Func_1`.
fn wait_for_samus<L: Lease + ?Sized>(game: &mut L) {
    let below: i16 = {
        let actor = game.current_enemy();
        game.player_delta(actor, Axis::Y)
    } as i16;
    if below >= 0 {
        let k = game.current_enemy();
        if below.wrapping_sub(Boulder::of(game, k).trigger_height() as i16) < 0 {
            if Boulder::of(game, k).kind() != kind::ROLL_RIGHT {
                let beside: i16 = {
                    let actor = game.current_enemy();
                    game.player_delta(actor, Axis::X)
                } as i16;
                if beside < 0
                    && beside.wrapping_sub(Boulder::of(game, k).trigger_distance() as i16) >= 0
                {
                    release(game, k);
                }
            } else {
                let beside: i16 = {
                    let actor = game.current_enemy();
                    game.player_delta(actor, Axis::X)
                } as i16;
                if beside >= 0
                    && beside.wrapping_sub(Boulder::of(game, k).trigger_distance() as i16) < 0
                {
                    release(game, k);
                }
            }
        }
    }
}

/// Speeds up `speed` by `step`, holding it at `cap`.
fn accelerate<L: Lease + ?Sized>(game: &mut L, k: u16, roll: bool, step: u16, cap: u16) {
    let speed = if roll {
        Boulder::of(game, k).roll_speed()
    } else {
        Boulder::of(game, k).fall_speed()
    }
    .wrapping_add(step);
    if roll {
        *BoulderEdit::of(game, k).roll_speed() = speed;
    } else {
        *BoulderEdit::of(game, k).fall_speed() = speed;
    }
    if (speed as i32 - cap as i32) & 0x8000 == 0 {
        if roll {
            *BoulderEdit::of(game, k).roll_speed() = cap;
        } else {
            *BoulderEdit::of(game, k).fall_speed() = cap;
        }
    }
}

/// The speed table row for a speed word's high byte; `+ 4` selects the
/// negated speed, moving up or left.
fn row<L: Lease + ?Sized>(game: &L, k: u16, roll: bool) -> u16 {
    let speed = if roll {
        Boulder::of(game, k).roll_speed().to_le_bytes()[1]
    } else {
        Boulder::of(game, k).fall_speed().to_le_bytes()[1]
    };
    8u16.wrapping_mul(speed as u16)
}

/// Rolls one frame at the current rolling speed. Was part of `Boulder_Func_3`.
fn roll_step<L: Lease + ?Sized>(game: &mut L, k: u16) {
    let mut row = row(game, k, true);
    if Boulder::of(game, k).kind() != kind::ROLL_RIGHT {
        row = row.wrapping_add(4);
    }
    move_x(game, row);
    accelerate(game, k, true, 32, BOUNCE_ROLL_CAP);
}

/// Falls to the ground, speeding up. Was `Boulder_Func_2`.
fn fall<L: Lease + ?Sized>(game: &mut L) {
    let k = game.current_enemy();
    move_y(game, row(game, k, false));
    if ((Boulder::of(game, k).actor().y.whole) as i16)
        .wrapping_sub((Boulder::of(game, k).ground_y()) as i16)
        < 0
    {
        accelerate(game, k, false, 256, FALL_SPEED_CAP);
    } else {
        BoulderEdit::of(game, k).actor().y.whole = Boulder::of(game, k).ground_y();
        *BoulderEdit::of(game, k).state() = state::BOUNCE_RISE;
        *BoulderEdit::of(game, k).fall_speed() = 0x2000;
    }
}

/// Rises after a landing while rolling, slowing down. Was `Boulder_Func_3`.
fn bounce_rise<L: Lease + ?Sized>(game: &mut L) {
    crate::probe::hit("boulder.bounce_rise");
    let k = game.current_enemy();
    move_y(
        game,
        (8 * Boulder::of(game, k).fall_speed().to_le_bytes()[1] as i32 + 4) as u16,
    );
    let speed: i16 = ((Boulder::of(game, k).fall_speed()) as i16).wrapping_sub(256);
    *BoulderEdit::of(game, k).fall_speed() = speed as u16;
    if speed >= 0 {
        roll_step(game, k);
    } else {
        *BoulderEdit::of(game, k).fall_speed() = 0;
        *BoulderEdit::of(game, k).state() = state::BOUNCE_FALL;
    }
}

/// Falls back from a bounce while rolling; landing bounces again, starts the
/// final roll, or shatters. Was `Boulder_Func_4`.
fn bounce_fall<L: Lease + ?Sized>(game: &mut L) {
    let k = game.current_enemy();
    let row: i32 = (8 * Boulder::of(game, k).fall_speed().to_le_bytes()[1] as i32) >> 1;
    let landed = {
        let actor = game.current_enemy();
        let amount = game.enemy_speed32(4 * (row >> 1) as usize) as i32;
        game.move_vertical(actor, amount)
    };
    if landed {
        game.queue_sound(1, LAND_SOUND, 6);
        if Boulder::of(game, k).kind() == kind::SHATTER {
            BoulderEdit::of(game, k).actor().properties =
                (Boulder::of(game, k).actor().properties) | (K_ENEMY_PROPS_DELETED as u16);
            shatter_dust(game, k);
        } else {
            *BoulderEdit::of(game, k).state() = state::BOUNCE_RISE;
            *BoulderEdit::of(game, k).fall_speed() =
                BOUNCE_SPEEDS[(Boulder::of(game, k).bounces_left() as i32 - 1i32) as usize];
            *BoulderEdit::of(game, k).bounces_left() =
                Boulder::of(game, k).bounces_left().wrapping_sub(1);
            if (Boulder::of(game, k).bounces_left() & 0x8000) != 0 {
                *BoulderEdit::of(game, k).previous_y() = Boulder::of(game, k).actor().y.whole;
                *BoulderEdit::of(game, k).previous_y_subpos() =
                    Boulder::of(game, k).actor().y.fraction;
                *BoulderEdit::of(game, k).state() = state::ROLL;
            }
        }
    } else {
        *BoulderEdit::of(game, k).fall_speed() =
            (Boulder::of(game, k).fall_speed()).wrapping_add(256);
        roll_step(game, k);
    }
}

/// Rolls along the ground until a wall stops it. Was `Boulder_Func_5`.
fn roll<L: Lease + ?Sized>(game: &mut L) {
    crate::probe::hit("boulder.roll");
    let k = game.current_enemy();
    let mut row = row(game, k, true);
    let index: i32 = row as i32 >> 1;
    {
        let actor = game.current_enemy();
        let amount = game
            .enemy_speed32(4 * (index >> 1) as usize)
            .wrapping_add(((Boulder::of(game, k).roll_lift() as i32) << 16) as u32)
            as i32;
        game.move_vertical(actor, amount);
    }
    BoulderEdit::of(game, k).actor().y.whole =
        (Boulder::of(game, k).actor().y.whole).wrapping_sub(Boulder::of(game, k).roll_lift());
    if Boulder::of(game, k).kind() != kind::ROLL_RIGHT {
        row = row.wrapping_add(4);
    }
    let index: i32 = row as i32 >> 1;
    let blocked = {
        let actor = game.current_enemy();
        let amount = game.enemy_speed32(4 * (index >> 1) as usize) as i32;
        game.move_horizontal(actor, amount, Slopes::Ignore)
    };
    if blocked {
        BoulderEdit::of(game, k).actor().properties = (Boulder::of(game, k).actor().properties)
            | ((K_ENEMY_PROPS_INVISIBLE | K_ENEMY_PROPS_DELETED) as u16);
        *BoulderEdit::of(game, k).state() = state::STOPPED;
        game.queue_sound(1, LAND_SOUND, 6);
        shatter_dust(game, k);
    } else {
        accelerate(game, k, true, 64, ROLL_SPEED_CAP);
        if Boulder::of(game, k).actor().y.whole == Boulder::of(game, k).previous_y()
            && Boulder::of(game, k).actor().y.fraction == Boulder::of(game, k).previous_y_subpos()
        {
            *BoulderEdit::of(game, k).roll_lift() = 0;
        }
    }
    *BoulderEdit::of(game, k).previous_y() = Boulder::of(game, k).actor().y.whole;
    *BoulderEdit::of(game, k).previous_y_subpos() = Boulder::of(game, k).actor().y.fraction;
}

/// A dust cloud where the boulder is, and the breaking sound.
fn shatter_dust<L: Lease + ?Sized>(game: &mut L, k: u16) {
    let (x, y) = (
        Boulder::of(game, k).actor().x.whole,
        Boulder::of(game, k).actor().y.whole,
    );
    game.place_projectile_spawn(x, y);
    game.spawn_room_eproj(DUST_CLOUD, DUST_CLOUD_PARAMETER);
    game.queue_sound(1, SHATTER_SOUND, 6);
}

/// The quadratic speed at word `row`: the 16.16 speed as (high, low) words.
fn quadratic<L: Lease + ?Sized>(game: &L, row: u16) -> u32 {
    let index: i32 = row as i32 >> 1;
    let high = game.enemy_speed(SpeedTable::Quadratic, (index + 1) as isize);
    let low = game.enemy_speed(SpeedTable::Quadratic, index as isize);
    (u32::from(high) << 16) | u32::from(low)
}

/// Moves the boulder horizontally by the quadratic speed at `row`, without
/// collision. Was `Boulder_Func_7`.
fn move_x<L: Lease + ?Sized>(game: &mut L, row: u16) {
    let k = game.current_enemy();
    {
        let value: u32 = quadratic(game, row);
        let mut edit = BoulderEdit::of(game, k);
        let sum = ((u32::from(edit.actor().x.whole) << 16) | u32::from(edit.actor().x.fraction))
            .wrapping_add(value);
        edit.actor().x.fraction = sum as u16;
        edit.actor().x.whole = (sum >> 16) as u16;
    }
}

/// Moves the boulder vertically by the quadratic speed at `row`, without
/// collision. Was `Boulder_Func_9`.
fn move_y<L: Lease + ?Sized>(game: &mut L, row: u16) {
    let k = game.current_enemy();
    {
        let value: u32 = quadratic(game, row);
        let mut edit = BoulderEdit::of(game, k);
        let sum = ((u32::from(edit.actor().y.whole) << 16) | u32::from(edit.actor().y.fraction))
            .wrapping_add(value);
        edit.actor().y.fraction = sum as u16;
        edit.actor().y.whole = (sum >> 16) as u16;
    }
}
