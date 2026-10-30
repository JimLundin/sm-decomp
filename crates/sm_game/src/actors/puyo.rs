//! Puyo, the blob that hops at Samus and drops back down off walls and ceilings
//! (bank $A2:998D-$A2:9E69). Was `hopping_blobs`.
use crate::actors::access::AccessMode::Native;
use crate::actors::api::{self, Axis, MovementResult::Blocked, SlopePolicy};
use crate::actors::combat::{self, current_slot};
use crate::actors::host::{ActorLease, LeaseActors};
use crate::actors::layouts::part_view;
use crate::player_projectiles::content;
use crate::session_slot;

part_view! {
    Puyo: EnemyLayout40 {
        words: [var_a, y_speed_index, hop_cooldown, function, airborne_function, hop_table_index, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [hop_type, hopping, direction, falling, invert_direction, inverted_direction, three_quarter_speed, half_speed,
            var_08, var_09, var_0a, var_0b, var_0c, var_0d, var_0e, var_0f, var_10, var_11, var_12, var_13, var_14, var_15,
            var_16, var_17, var_18, var_19, var_1a, var_1b, var_1c, var_1d, var_1e, var_1f],
    }
}

const GROUNDED: u16 = 0x9b65;
const AIRBORNE: u16 = 0x9b81;
const DROPPING: u16 = 0x9d98;
const DROPPING_HOP: u16 = 4;
/// Grounded/dropping lists: fast, medium, slow.
const GROUNDED_LISTS: [u16; 3] = [0x99ad, 0x99c1, 0x99d5];
/// Hopping lists by speed, rising right or falling left, then rising left or falling right.
const HOPPING_LISTS: [[u16; 3]; 2] = [[0x99f5, 0x99ef, 0x99e9], [0x99f5, 0x99fb, 0x9a01]];
/// $A2:9A07: the hop table, 8 bytes a hop: jump height, X speed, Y speed table index
/// delta and airborne function.
const HOPS: u32 = 0xa2_9a07;
const X_SPEED: u32 = 2;
const Y_SPEED_DELTA: u32 = 4;
const AIRBORNE_FUNCTION: u32 = 6;
const QUADRATIC_SPEEDS: u32 = 0xa2_838f;

fn rom_word(runtime: &impl ActorLease, address: u32, offset: u16) -> u16 {
    let at = content::offset(address) + usize::from(offset);
    let image = runtime.native_image();
    u16::from_ne_bytes([image.read(at), image.read(at + 1)])
}
/// Set enemy instruction list ($A2:9A6C).
fn set_instruction_list(runtime: &mut impl ActorLease, list: u16) {
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
}
/// Initialisation AI - puyo ($A2:9A3F).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let puyo = current_slot(runtime);
    session_slot!(runtime, Puyo[puyo].base.spritemap_pointer = 0x804d);
    session_slot!(runtime, Puyo[puyo].var_a = 0);
    set_instruction_list(runtime, GROUNDED_LISTS[0]);
    let cooldown = session_slot!(runtime, Puyo[puyo].parameter_1);
    session_slot!(runtime, Puyo[puyo]; hop_type = 0, function = GROUNDED, hop_cooldown = cooldown,
        invert_direction = 0);
}
/// $1C = random number in 0..7 ($A2:9B06).
fn random_eighth(runtime: &mut impl ActorLease, puyo: i32) -> u16 {
    runtime.next_random();
    let frame = session_slot!(runtime, Puyo[puyo].base.frame_counter);
    frame.wrapping_add(runtime.word(crate::ram::words::RANDOM_NUMBER)) & 7
}
/// Puyo function - grounded ($A2:9B65): once the cooldown runs out, initiate hop ($A2:9A84).
fn grounded(runtime: &mut impl ActorLease, puyo: i32) {
    let cooldown = session_slot!(runtime, Puyo[puyo].hop_cooldown).wrapping_sub(1);
    session_slot!(runtime, Puyo[puyo].hop_cooldown = cooldown);
    if cooldown & 0x8000 == 0 {
        return;
    }
    let cooldown = session_slot!(runtime, Puyo[puyo].parameter_1);
    session_slot!(runtime, Puyo[puyo]; function = AIRBORNE, hop_cooldown = cooldown, hopping = 1);
    // Check if Samus is in proximity ($A2:9A9B).
    if session_slot!(runtime, Puyo[puyo].hop_type).wrapping_sub(3) & 0x8000 != 0 {
        let distance = session_slot!(runtime, Puyo[puyo].parameter_2);
        let key = combat::current(runtime);
        let close = api::player_within(runtime, key, Axis::X, distance);
        session_slot!(runtime, Puyo[puyo].hop_type = u16::from(close));
    }
    // Choose hop type ($A2:9AAA): face Samus, unless a wall turned it round.
    let key = combat::current(runtime);
    let right = api::player_delta(runtime, key, Axis::X) & 0x8000 == 0;
    session_slot!(runtime, Puyo[puyo].direction = u16::from(!right));
    if session_slot!(runtime, Puyo[puyo].invert_direction) != 0 {
        let inverted = session_slot!(runtime, Puyo[puyo].inverted_direction);
        session_slot!(runtime, Puyo[puyo].direction = inverted);
    }
    session_slot!(runtime, Puyo[puyo].invert_direction = 0);
    let mut random = random_eighth(runtime, puyo);
    let mut hop = session_slot!(runtime, Puyo[puyo].hop_type);
    if hop.wrapping_sub(3) & 0x8000 != 0 {
        if hop == 0 {
            random &= 1;
        }
        hop = if random.wrapping_sub(2) & 0x8000 == 0 {
            2
        } else {
            random
        };
    }
    let index = hop.wrapping_mul(8);
    let function = rom_word(runtime, HOPS + AIRBORNE_FUNCTION, index);
    session_slot!(runtime, Puyo[puyo]; hop_table_index = index, airborne_function = function);
    // Calculate initial hop speed ($A2:9B1A).
    let (mut speed, mut height) = (0u16, 0u16);
    loop {
        speed = speed.wrapping_add(rom_word(runtime, HOPS + X_SPEED, index));
        height = height.wrapping_add(rom_word(runtime, QUADRATIC_SPEEDS, 8 * (speed >> 8) + 1));
        let target = rom_word(runtime, HOPS, index).swap_bytes();
        if target.wrapping_sub(height) & 0x8000 != 0 {
            break;
        }
    }
    let half = speed >> 1;
    session_slot!(runtime, Puyo[puyo]; y_speed_index = speed, falling = 0, half_speed = half,
        three_quarter_speed = half.wrapping_add(half >> 1));
}
/// Set rising ($A2:9C71) or falling ($A2:9CBE) instruction list, by speed.
fn set_hopping_list(runtime: &mut impl ActorLease, puyo: i32, falling: bool) {
    let speed = session_slot!(runtime, Puyo[puyo].y_speed_index);
    let at_least = |limit: u16| speed.wrapping_sub(limit) as i16 >= 0;
    let (half, three_quarter) = (
        session_slot!(runtime, Puyo[puyo].half_speed),
        session_slot!(runtime, Puyo[puyo].three_quarter_speed),
    );
    let level = match falling {
        true if !at_least(half) => 0,
        true if !at_least(three_quarter) => 1,
        false if !at_least(three_quarter) => usize::from(at_least(half)),
        _ => 2,
    };
    let left = session_slot!(runtime, Puyo[puyo].direction) != 0;
    set_instruction_list(runtime, HOPPING_LISTS[usize::from(left ^ falling)][level]);
}
/// Bounce off a wall or ceiling: turn round next hop and drop.
fn start_dropping(runtime: &mut impl ActorLease, puyo: i32) {
    let inverted = session_slot!(runtime, Puyo[puyo].direction) ^ 1;
    session_slot!(runtime, Puyo[puyo]; invert_direction = 1, inverted_direction = inverted);
    session_slot!(runtime, Puyo[puyo]; hop_type = DROPPING_HOP, airborne_function = DROPPING);
}
/// Puyo movement ($A2:9B88): rise or fall, then move sideways.
fn hop(runtime: &mut impl ActorLease, puyo: i32) {
    let speed = session_slot!(runtime, Puyo[puyo].y_speed_index);
    let speed = if speed.wrapping_sub(0x4000) & 0x8000 == 0 {
        0x4000
    } else {
        speed
    };
    let falling = session_slot!(runtime, Puyo[puyo].falling) != 0;
    let offset = (8 * (speed >> 8)).wrapping_add(if falling { 0 } else { 4 });
    let amount = runtime.lease_enemy_speed32(usize::from(offset >> 2) * 4) as i32;
    let key = combat::current(runtime);
    if api::move_vertical(runtime, key, amount) == Blocked {
        if falling {
            session_slot!(runtime, Puyo[puyo].function = GROUNDED);
        } else {
            start_dropping(runtime, puyo);
        }
        session_slot!(runtime, Puyo[puyo].hopping = 0);
        return;
    }
    let index = session_slot!(runtime, Puyo[puyo].hop_table_index);
    set_hopping_list(runtime, puyo, falling);
    let delta = rom_word(runtime, HOPS + Y_SPEED_DELTA, index);
    let speed = session_slot!(runtime, Puyo[puyo].y_speed_index);
    let speed = if falling {
        speed.wrapping_add(delta)
    } else {
        speed.wrapping_sub(delta)
    };
    session_slot!(runtime, Puyo[puyo].y_speed_index = speed);
    if speed & 0x8000 != 0 {
        session_slot!(runtime, Puyo[puyo]; falling = 1, y_speed_index = 0);
    }
    let index = session_slot!(runtime, Puyo[puyo].hop_table_index);
    let mut x_speed = rom_word(runtime, HOPS + X_SPEED, index) >> 8;
    if session_slot!(runtime, Puyo[puyo].direction) != 0 {
        x_speed = x_speed.wrapping_neg();
    }
    if api::move_horizontal(
        runtime,
        key,
        i32::from(x_speed as i16) << 16,
        SlopePolicy::Ignore,
    ) == Blocked
    {
        start_dropping(runtime, puyo);
        session_slot!(runtime, Puyo[puyo].hopping = 0);
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Landing {
    List(u16),
    Restart,
    Repeat,
}
/// Puyo airborne function - a hop ($A2:9D0B-$A2:9D6B) or dropped ($A2:9DCD): hop, and land.
fn land(runtime: &mut impl ActorLease, puyo: i32, landing: Landing) {
    hop(runtime, puyo);
    let interrupted =
        landing != Landing::Repeat && session_slot!(runtime, Puyo[puyo].invert_direction) != 0;
    if !interrupted && session_slot!(runtime, Puyo[puyo].hopping) != 0 {
        return;
    }
    if landing == Landing::Restart && !interrupted {
        session_slot!(runtime, Puyo[puyo]; hop_type = 0, function = GROUNDED);
    }
    session_slot!(runtime, Puyo[puyo].hopping = 0);
    if landing == Landing::Repeat {
        session_slot!(runtime, Puyo[puyo]; hop_type = 3, function = GROUNDED);
    }
    let list = if let Landing::List(list) = landing {
        list
    } else {
        GROUNDED_LISTS[2]
    };
    set_instruction_list(runtime, list);
}
/// Main AI - puyo ($A2:9A7D); false for an unknown function, as the source fails.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) -> bool {
    let runtime = &mut game.lease();
    let puyo = current_slot(runtime);
    match session_slot!(runtime, Puyo[puyo].function) {
        GROUNDED => grounded(runtime, puyo),
        AIRBORNE => match session_slot!(runtime, Puyo[puyo].airborne_function) {
            0x9d0b => land(runtime, puyo, Landing::List(GROUNDED_LISTS[2])),
            0x9d2b => land(runtime, puyo, Landing::List(GROUNDED_LISTS[1])),
            0x9d4b => land(runtime, puyo, Landing::List(GROUNDED_LISTS[0])),
            0x9d6b => land(runtime, puyo, Landing::Restart),
            0x9dcd => land(runtime, puyo, Landing::Repeat),
            // Puyo airborne function - dropping ($A2:9D98): drop until landing.
            DROPPING => {
                let index = session_slot!(runtime, Puyo[puyo].hop_table_index);
                let delta = rom_word(runtime, HOPS + Y_SPEED_DELTA, index);
                let speed = (u32::from(delta >> 8) << 16) | (u32::from(delta & 0xff) << 8);
                let key = combat::current(runtime);
                if api::move_vertical(runtime, key, speed as i32) == Blocked {
                    let hop = (random_eighth(runtime, puyo) & 1) + 5;
                    session_slot!(runtime, Puyo[puyo]; hop_type = hop, function = GROUNDED);
                }
            }
            _ => return false,
        },
        _ => return false,
    }
    true
}
