//! The boulder: it waits for Samus to pass below, drops, bounces twice and
//! rolls until it hits a wall. The shattering kind breaks on its first landing.
//!
//! Was `Boulder` in the original decompilation (bank $A6:86F5-$A6:89FC). Its
//! state words name the slot fields over the same RAM bytes as `EnemyLayout40`.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Axis, BossEffects, Slopes};
use crate::actors::layouts::{EnemyBase, EnemyLayout40};
use crate::constants::enemy_properties::{K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_INVISIBLE};
use crate::session_slot;
use sm_data::enemy_speeds::SpeedTable;

/// What the boulder needs beyond the shared boss effects: the 32-bit speeds.
pub trait Lease: BossLease<Game: BossEffects> {
    /// The native-order 32-bit enemy speed at byte `offset` from $A0:CBC7.
    fn enemy_speed32(&self, offset: usize) -> u32;
}

/// The boulder's view of its `EnemyLayout40` slot.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Boulder {
    pub base: EnemyBase,
    /// Bank $A6 state function, one of [`state`].
    pub state: u16,
    /// Rolling speed: its high byte selects the speed table row.
    pub roll_speed: u16,
    /// Vertical speed: its high byte selects the speed table row.
    pub fall_speed: u16,
    /// Landings left before the boulder starts rolling.
    pub bounces_left: u16,
    /// One of [`kind`]: which way the boulder rolls, and whether it shatters.
    pub kind: u16,
    pub var_f: u16,
    pub parameter_1: u16,
    pub parameter_2: u16,
    pub pad: [u8; 26696],
    /// Horizontal distance from Samus that releases the boulder (signed).
    pub trigger_distance: u16,
    /// How far each rolling frame lifts the boulder before gravity pulls it down.
    pub roll_lift: u16,
    pub previous_y_subpos: u16,
    pub previous_y: u16,
    pub home_x: u16,
    /// The height the boulder lands at.
    pub ground_y: u16,
    /// How far below the boulder Samus must be to release it.
    pub trigger_height: u16,
    /// Nonzero when the boulder starts on the ground and rolls straight away.
    pub starts_rolling: u16,
    /// The slot's remaining words, unused by the boulder.
    pub unused: [u16; 24],
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(offset_of!(Boulder, state) == offset_of!(EnemyLayout40, var_a));
    assert!(offset_of!(Boulder, parameter_2) == offset_of!(EnemyLayout40, parameter_2));
    assert!(offset_of!(Boulder, pad) == offset_of!(EnemyLayout40, pad1));
    assert!(offset_of!(Boulder, trigger_distance) == offset_of!(EnemyLayout40, var_00));
    assert!(offset_of!(Boulder, starts_rolling) == offset_of!(EnemyLayout40, var_07));
    assert!(size_of::<Boulder>() == size_of::<EnemyLayout40>());
};

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

fn current<L: Lease + ?Sized>(game: &L) -> i32 {
    i32::from(game.current_enemy())
}

/// Sets up a boulder from its parameters. Was `Boulder_Init`.
pub fn initialize<L: Lease + ?Sized>(game: &mut L) {
    let me = current(game);
    session_slot!(game, Boulder[me]; fall_speed = 0, roll_speed = 0);
    session_slot!(game, Boulder[me]; bounces_left = 2, state = state::WAIT_FOR_SAMUS);
    session_slot!(
        game,
        Boulder[me].home_x = session_slot!(game, Boulder[me].base.x_pos)
    );
    let y_pos: u16 = session_slot!(game, Boulder[me].base.y_pos);
    session_slot!(game, Boulder[me]; ground_y = y_pos, previous_y = y_pos);
    session_slot!(
        game,
        Boulder[me].previous_y_subpos = session_slot!(game, Boulder[me].base.y_subpos)
    );
    let mut drop_height: i16 = session_slot!(byte game, Boulder[me].parameter_2[1]) as i16;
    if session_slot!(byte game, Boulder[me].parameter_2[1]) == 0 {
        drop_height = 1;
        session_slot!(game, Boulder[me].starts_rolling = 1);
    }
    session_slot!(
        game,
        Boulder[me].base.y_pos =
            (session_slot!(game, Boulder[me].ground_y)).wrapping_sub(drop_height as u16)
    );
    session_slot!(
        game,
        Boulder[me].trigger_distance =
            -(session_slot!(byte game, Boulder[me].parameter_2[0]) as i32) as u16
    );
    session_slot!(
        game,
        Boulder[me].trigger_height =
            session_slot!(byte game, Boulder[me].base.current_instruction[0]) as u16
    );
    session_slot!(
        game,
        Boulder[me].base.current_instruction = animation::ROLL_LEFT
    );
    let kind: u16 = session_slot!(byte game, Boulder[me].parameter_1[1]) as u16;
    session_slot!(game, Boulder[me].kind = kind);
    if kind == kind::ROLL_RIGHT {
        session_slot!(
            game,
            Boulder[me].trigger_distance =
                -(session_slot!(game, Boulder[me].trigger_distance) as i32) as u16
        );
        session_slot!(
            game,
            Boulder[me].base.current_instruction = animation::ROLL_RIGHT
        );
    }
    session_slot!(game, Boulder[me].roll_lift = 2);
    if session_slot!(byte game, Boulder[me].parameter_1[0]) != 0 {
        session_slot!(game, Boulder[me].roll_lift = 5);
    }
}

/// Runs the current state. Was `Boulder_Main`.
pub fn main_ai<L: Lease + ?Sized>(game: &mut L) {
    crate::probe::hit("boulder.main_ai");
    let me = current(game);
    match session_slot!(game, Boulder[me].state) {
        state::WAIT_FOR_SAMUS => wait_for_samus(game),
        state::FALL => fall(game),
        state::BOUNCE_RISE => bounce_rise(game),
        state::BOUNCE_FALL => bounce_fall(game),
        state::ROLL => roll(game),
        state::STOPPED => {}
        _ => game.foreign(|game| game.unreachable()),
    }
}

/// The state that follows a release: a drop, or a roll when on the ground.
fn release<L: Lease + ?Sized>(game: &mut L, me: i32) {
    session_slot!(game, Boulder[me].state = state::FALL);
    if session_slot!(game, Boulder[me].starts_rolling) != 0 {
        session_slot!(game, Boulder[me].state = state::ROLL);
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
        let me = current(game);
        if below.wrapping_sub(session_slot!(game, Boulder[me].trigger_height) as i16) < 0 {
            if session_slot!(game, Boulder[me].kind) != kind::ROLL_RIGHT {
                let beside: i16 = {
                    let actor = game.current_enemy();
                    game.player_delta(actor, Axis::X)
                } as i16;
                if beside < 0
                    && beside.wrapping_sub(session_slot!(game, Boulder[me].trigger_distance) as i16)
                        >= 0
                {
                    release(game, me);
                }
            } else {
                let beside: i16 = {
                    let actor = game.current_enemy();
                    game.player_delta(actor, Axis::X)
                } as i16;
                if beside >= 0
                    && beside.wrapping_sub(session_slot!(game, Boulder[me].trigger_distance) as i16)
                        < 0
                {
                    release(game, me);
                }
            }
        }
    }
}

/// Speeds up `speed` by `step`, holding it at `cap`.
fn accelerate<L: Lease + ?Sized>(game: &mut L, me: i32, roll: bool, step: u16, cap: u16) {
    let speed = if roll {
        session_slot!(game, Boulder[me].roll_speed)
    } else {
        session_slot!(game, Boulder[me].fall_speed)
    }
    .wrapping_add(step);
    if roll {
        session_slot!(game, Boulder[me].roll_speed = speed);
    } else {
        session_slot!(game, Boulder[me].fall_speed = speed);
    }
    if (speed as i32 - cap as i32) & 0x8000 == 0 {
        if roll {
            session_slot!(game, Boulder[me].roll_speed = cap);
        } else {
            session_slot!(game, Boulder[me].fall_speed = cap);
        }
    }
}

/// The speed table row for a speed word's high byte; `+ 4` selects the
/// negated speed, moving up or left.
fn row<L: Lease + ?Sized>(game: &L, me: i32, roll: bool) -> u16 {
    let speed = if roll {
        session_slot!(byte game, Boulder[me].roll_speed[1])
    } else {
        session_slot!(byte game, Boulder[me].fall_speed[1])
    };
    8u16.wrapping_mul(speed as u16)
}

/// Rolls one frame at the current rolling speed. Was part of `Boulder_Func_3`.
fn roll_step<L: Lease + ?Sized>(game: &mut L, me: i32) {
    let mut row = row(game, me, true);
    if session_slot!(game, Boulder[me].kind) != kind::ROLL_RIGHT {
        row = row.wrapping_add(4);
    }
    move_x(game, row);
    accelerate(game, me, true, 32, BOUNCE_ROLL_CAP);
}

/// Falls to the ground, speeding up. Was `Boulder_Func_2`.
fn fall<L: Lease + ?Sized>(game: &mut L) {
    let me = current(game);
    move_y(game, row(game, me, false));
    if ((session_slot!(game, Boulder[me].base.y_pos)) as i16)
        .wrapping_sub((session_slot!(game, Boulder[me].ground_y)) as i16)
        < 0
    {
        accelerate(game, me, false, 256, FALL_SPEED_CAP);
    } else {
        session_slot!(
            game,
            Boulder[me].base.y_pos = session_slot!(game, Boulder[me].ground_y)
        );
        session_slot!(game, Boulder[me]; state = state::BOUNCE_RISE, fall_speed = 0x2000);
    }
}

/// Rises after a landing while rolling, slowing down. Was `Boulder_Func_3`.
fn bounce_rise<L: Lease + ?Sized>(game: &mut L) {
    crate::probe::hit("boulder.bounce_rise");
    let me = current(game);
    move_y(
        game,
        (8 * session_slot!(byte game, Boulder[me].fall_speed[1]) as i32 + 4) as u16,
    );
    let speed: i16 = ((session_slot!(game, Boulder[me].fall_speed)) as i16).wrapping_sub(256);
    session_slot!(game, Boulder[me].fall_speed = speed as u16);
    if speed >= 0 {
        roll_step(game, me);
    } else {
        session_slot!(game, Boulder[me]; fall_speed = 0, state = state::BOUNCE_FALL);
    }
}

/// Falls back from a bounce while rolling; landing bounces again, starts the
/// final roll, or shatters. Was `Boulder_Func_4`.
fn bounce_fall<L: Lease + ?Sized>(game: &mut L) {
    let me = current(game);
    let row: i32 = (8 * session_slot!(byte game, Boulder[me].fall_speed[1]) as i32) >> 1;
    let landed = {
        let actor = game.current_enemy();
        let amount = game.enemy_speed32(4 * (row >> 1) as usize) as i32;
        game.move_vertical(actor, amount)
    };
    if landed {
        game.queue_sound(1, LAND_SOUND, 6);
        if session_slot!(game, Boulder[me].kind) == kind::SHATTER {
            session_slot!(
                game,
                Boulder[me].base.properties = (session_slot!(game, Boulder[me].base.properties))
                    | (K_ENEMY_PROPS_DELETED as u16)
            );
            shatter_dust(game, me);
        } else {
            session_slot!(game, Boulder[me].state = state::BOUNCE_RISE);
            session_slot!(
                game,
                Boulder[me].fall_speed = BOUNCE_SPEEDS
                    [(session_slot!(game, Boulder[me].bounces_left) as i32 - 1i32) as usize]
            );
            session_slot!(game, Boulder[me].bounces_left, |old| old.wrapping_sub(1));
            if (session_slot!(game, Boulder[me].bounces_left) & 0x8000) != 0 {
                session_slot!(
                    game,
                    Boulder[me].previous_y = session_slot!(game, Boulder[me].base.y_pos)
                );
                session_slot!(
                    game,
                    Boulder[me].previous_y_subpos = session_slot!(game, Boulder[me].base.y_subpos)
                );
                session_slot!(game, Boulder[me].state = state::ROLL);
            }
        }
    } else {
        session_slot!(
            game,
            Boulder[me].fall_speed =
                (session_slot!(game, Boulder[me].fall_speed)).wrapping_add(256)
        );
        roll_step(game, me);
    }
}

/// Rolls along the ground until a wall stops it. Was `Boulder_Func_5`.
fn roll<L: Lease + ?Sized>(game: &mut L) {
    crate::probe::hit("boulder.roll");
    let me = current(game);
    let mut row = row(game, me, true);
    let index: i32 = row as i32 >> 1;
    {
        let actor = game.current_enemy();
        let amount = game
            .enemy_speed32(4 * (index >> 1) as usize)
            .wrapping_add(((session_slot!(game, Boulder[me].roll_lift) as i32) << 16) as u32)
            as i32;
        game.move_vertical(actor, amount);
    }
    session_slot!(
        game,
        Boulder[me].base.y_pos = (session_slot!(game, Boulder[me].base.y_pos))
            .wrapping_sub(session_slot!(game, Boulder[me].roll_lift))
    );
    if session_slot!(game, Boulder[me].kind) != kind::ROLL_RIGHT {
        row = row.wrapping_add(4);
    }
    let index: i32 = row as i32 >> 1;
    let blocked = {
        let actor = game.current_enemy();
        let amount = game.enemy_speed32(4 * (index >> 1) as usize) as i32;
        game.move_horizontal(actor, amount, Slopes::Ignore)
    };
    if blocked {
        session_slot!(
            game,
            Boulder[me].base.properties = (session_slot!(game, Boulder[me].base.properties))
                | ((K_ENEMY_PROPS_INVISIBLE | K_ENEMY_PROPS_DELETED) as u16)
        );
        session_slot!(game, Boulder[me].state = state::STOPPED);
        game.queue_sound(1, LAND_SOUND, 6);
        shatter_dust(game, me);
    } else {
        accelerate(game, me, true, 64, ROLL_SPEED_CAP);
        if session_slot!(game, Boulder[me].base.y_pos)
            == session_slot!(game, Boulder[me].previous_y)
            && session_slot!(game, Boulder[me].base.y_subpos)
                == session_slot!(game, Boulder[me].previous_y_subpos)
        {
            session_slot!(game, Boulder[me].roll_lift = 0);
        }
    }
    session_slot!(
        game,
        Boulder[me].previous_y = session_slot!(game, Boulder[me].base.y_pos)
    );
    session_slot!(
        game,
        Boulder[me].previous_y_subpos = session_slot!(game, Boulder[me].base.y_subpos)
    );
}

/// A dust cloud where the boulder is, and the breaking sound.
fn shatter_dust<L: Lease + ?Sized>(game: &mut L, me: i32) {
    let (x, y) = (
        session_slot!(game, Boulder[me].base.x_pos),
        session_slot!(game, Boulder[me].base.y_pos),
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
    let me = current(game);
    session_slot!(
        game,
        Boulder[me].base(x_pos, x_subpos) += quadratic(game, row)
    );
}

/// Moves the boulder vertically by the quadratic speed at `row`, without
/// collision. Was `Boulder_Func_9`.
fn move_y<L: Lease + ?Sized>(game: &mut L, row: u16) {
    let me = current(game);
    session_slot!(
        game,
        Boulder[me].base(y_pos, y_subpos) += quadratic(game, row)
    );
}
