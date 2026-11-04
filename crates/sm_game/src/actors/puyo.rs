//! Puyo, the blob that hops at Samus and drops back down off walls and ceilings
//! (bank $A2:998D-$A2:9E69).
use crate::actors::access::AccessMode::Native;
use crate::actors::api::{self, Axis, MovementResult::Blocked, SlopePolicy};
use crate::actors::combat;
use crate::actors::host::ActorLease;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::player_projectiles::content;

slot_view! {
    /// A puyo's slot: its Y speed table index, hop cooldown, function and
    /// airborne function, hop table index; its hop type, whether it is
    /// hopping, its direction, whether it is falling, whether (and to which
    /// direction) a wall turns it round next hop, and three quarters and half
    /// its initial speed. Its parameters are the hop cooldown and Samus's
    /// proximity.
    Puyo, PuyoEdit {
        record: [
            var_a = var_a, y_speed_index = var_b, hop_cooldown = var_c, function = var_d,
            airborne_function = var_e, hop_table_index = var_f, parameter_1 = parameter_1,
            parameter_2 = parameter_2,
        ],
        extended: [
            hop_type = var_00, hopping = var_01, direction = var_02, falling = var_03,
            invert_direction = var_04, inverted_direction = var_05,
            three_quarter_speed = var_06, half_speed = var_07,
        ],
    }
}

/// A lease the puyos run on: its slots are the Session's.
pub trait Lease: ActorLease + SlotState {}
impl<T: ActorLease + SlotState + ?Sized> Lease for T {}

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

fn rom_word(runtime: &impl Lease, address: u32, offset: u16) -> u16 {
    let at = content::offset(address) + usize::from(offset);
    let image = runtime.native_image();
    u16::from_ne_bytes([image.read(at), image.read(at + 1)])
}
/// Set enemy instruction list ($A2:9A6C).
fn set_instruction_list(runtime: &mut impl Lease, list: u16) {
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
}
/// Initialisation AI - puyo ($A2:9A3F).
pub fn initialize(runtime: &mut impl Lease) {
    let puyo = combat::current(runtime).byte_index();
    {
        let mut edit = PuyoEdit::of(runtime, puyo);
        edit.actor().spritemap = 0x804d;
        *edit.var_a() = 0;
    }
    set_instruction_list(runtime, GROUNDED_LISTS[0]);
    let mut edit = PuyoEdit::of(runtime, puyo);
    let cooldown = *edit.parameter_1();
    *edit.hop_type() = 0;
    *edit.function() = GROUNDED;
    *edit.hop_cooldown() = cooldown;
    *edit.invert_direction() = 0;
}
/// $1C = random number in 0..7 ($A2:9B06).
fn random_eighth(runtime: &mut impl Lease, puyo: u16) -> u16 {
    runtime.next_random();
    let frame = Puyo::of(runtime, puyo).actor().frame;
    frame.wrapping_add(runtime.word(crate::ram::words::RANDOM_NUMBER)) & 7
}
/// Puyo function - grounded ($A2:9B65): once the cooldown runs out, initiate hop ($A2:9A84).
fn grounded(runtime: &mut impl Lease, puyo: u16) {
    let cooldown = Puyo::of(runtime, puyo).hop_cooldown().wrapping_sub(1);
    *PuyoEdit::of(runtime, puyo).hop_cooldown() = cooldown;
    if cooldown & 0x8000 == 0 {
        return;
    }
    {
        let mut edit = PuyoEdit::of(runtime, puyo);
        let cooldown = *edit.parameter_1();
        *edit.function() = AIRBORNE;
        *edit.hop_cooldown() = cooldown;
        *edit.hopping() = 1;
    }
    // Check if Samus is in proximity ($A2:9A9B).
    if Puyo::of(runtime, puyo).hop_type().wrapping_sub(3) & 0x8000 != 0 {
        let distance = Puyo::of(runtime, puyo).parameter_2();
        let key = combat::current(runtime);
        let close = api::player_within(runtime, key, Axis::X, distance);
        *PuyoEdit::of(runtime, puyo).hop_type() = u16::from(close);
    }
    // Choose hop type ($A2:9AAA): face Samus, unless a wall turned it round.
    let key = combat::current(runtime);
    let right = api::player_delta(runtime, key, Axis::X) & 0x8000 == 0;
    {
        let mut edit = PuyoEdit::of(runtime, puyo);
        *edit.direction() = u16::from(!right);
        if *edit.invert_direction() != 0 {
            let inverted = *edit.inverted_direction();
            *edit.direction() = inverted;
        }
        *edit.invert_direction() = 0;
    }
    let mut random = random_eighth(runtime, puyo);
    let mut hop = Puyo::of(runtime, puyo).hop_type();
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
    {
        let mut edit = PuyoEdit::of(runtime, puyo);
        *edit.hop_table_index() = index;
        *edit.airborne_function() = function;
    }
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
    let mut edit = PuyoEdit::of(runtime, puyo);
    *edit.y_speed_index() = speed;
    *edit.falling() = 0;
    *edit.half_speed() = half;
    *edit.three_quarter_speed() = half.wrapping_add(half >> 1);
}
/// Set rising ($A2:9C71) or falling ($A2:9CBE) instruction list, by speed.
fn set_hopping_list(runtime: &mut impl Lease, puyo: u16, falling: bool) {
    let view = Puyo::of(runtime, puyo);
    let speed = view.y_speed_index();
    let at_least = |limit: u16| speed.wrapping_sub(limit) as i16 >= 0;
    let (half, three_quarter) = (view.half_speed(), view.three_quarter_speed());
    let level = match falling {
        true if !at_least(half) => 0,
        true if !at_least(three_quarter) => 1,
        false if !at_least(three_quarter) => usize::from(at_least(half)),
        _ => 2,
    };
    let left = Puyo::of(runtime, puyo).direction() != 0;
    set_instruction_list(runtime, HOPPING_LISTS[usize::from(left ^ falling)][level]);
}
/// Bounce off a wall or ceiling: turn round next hop if `invert` is nonzero, and drop.
fn start_dropping(runtime: &mut impl Lease, puyo: u16, invert: u16) {
    let mut edit = PuyoEdit::of(runtime, puyo);
    let inverted = *edit.direction() ^ 1;
    *edit.invert_direction() = invert;
    *edit.inverted_direction() = inverted;
    *edit.hop_type() = DROPPING_HOP;
    *edit.airborne_function() = DROPPING;
}
/// Puyo movement ($A2:9B88): rise or fall, then move sideways.
fn hop(runtime: &mut impl Lease, puyo: u16) {
    let speed = Puyo::of(runtime, puyo).y_speed_index();
    let speed = if speed.wrapping_sub(0x4000) & 0x8000 == 0 {
        0x4000
    } else {
        speed
    };
    let falling = Puyo::of(runtime, puyo).falling() != 0;
    let offset = (8 * (speed >> 8)).wrapping_add(if falling { 0 } else { 4 });
    let amount = runtime.lease_enemy_speed32(usize::from(offset >> 2) * 4) as i32;
    let key = combat::current(runtime);
    if api::move_vertical(runtime, key, amount) == Blocked {
        if falling {
            *PuyoEdit::of(runtime, puyo).function() = GROUNDED;
        } else {
            // The original loads the scratch word at $01 here ($A2:9BBF), not #$0001.
            let invert = runtime.session().pointer_scratch.word_at_01();
            start_dropping(runtime, puyo, invert);
        }
        *PuyoEdit::of(runtime, puyo).hopping() = 0;
        return;
    }
    let index = Puyo::of(runtime, puyo).hop_table_index();
    set_hopping_list(runtime, puyo, falling);
    let delta = rom_word(runtime, HOPS + Y_SPEED_DELTA, index);
    {
        let mut edit = PuyoEdit::of(runtime, puyo);
        let speed = edit.y_speed_index();
        *speed = if falling {
            speed.wrapping_add(delta)
        } else {
            speed.wrapping_sub(delta)
        };
        if *speed & 0x8000 != 0 {
            *edit.falling() = 1;
            *edit.y_speed_index() = 0;
        }
    }
    let index = Puyo::of(runtime, puyo).hop_table_index();
    let mut x_speed = rom_word(runtime, HOPS + X_SPEED, index) >> 8;
    if Puyo::of(runtime, puyo).direction() != 0 {
        x_speed = x_speed.wrapping_neg();
    }
    if api::move_horizontal(
        runtime,
        key,
        i32::from(x_speed as i16) << 16,
        SlopePolicy::Ignore,
    ) == Blocked
    {
        start_dropping(runtime, puyo, 1);
        *PuyoEdit::of(runtime, puyo).hopping() = 0;
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Landing {
    List(u16),
    Restart,
    Repeat,
}
/// Puyo airborne function - a hop ($A2:9D0B-$A2:9D6B) or dropped ($A2:9DCD): hop, and land.
fn land(runtime: &mut impl Lease, puyo: u16, landing: Landing) {
    hop(runtime, puyo);
    let interrupted = landing != Landing::Repeat && Puyo::of(runtime, puyo).invert_direction() != 0;
    if !interrupted && Puyo::of(runtime, puyo).hopping() != 0 {
        return;
    }
    {
        let mut edit = PuyoEdit::of(runtime, puyo);
        if landing == Landing::Restart && !interrupted {
            *edit.hop_type() = 0;
            *edit.function() = GROUNDED;
        }
        *edit.hopping() = 0;
        if landing == Landing::Repeat {
            *edit.hop_type() = 3;
            *edit.function() = GROUNDED;
        }
    }
    let list = if let Landing::List(list) = landing {
        list
    } else {
        GROUNDED_LISTS[2]
    };
    set_instruction_list(runtime, list);
}
/// Main AI - puyo ($A2:9A7D); false for an unknown function, as the source fails.
pub fn main_ai(runtime: &mut impl crate::actors::species::TableLease) -> bool {
    let puyo = combat::current(runtime).byte_index();
    match Puyo::of(runtime, puyo).function() {
        GROUNDED => grounded(runtime, puyo),
        AIRBORNE => match Puyo::of(runtime, puyo).airborne_function() {
            0x9d0b => land(runtime, puyo, Landing::List(GROUNDED_LISTS[2])),
            0x9d2b => land(runtime, puyo, Landing::List(GROUNDED_LISTS[1])),
            0x9d4b => land(runtime, puyo, Landing::List(GROUNDED_LISTS[0])),
            0x9d6b => land(runtime, puyo, Landing::Restart),
            0x9dcd => land(runtime, puyo, Landing::Repeat),
            // Puyo airborne function - dropping ($A2:9D98): drop until landing.
            DROPPING => {
                let index = Puyo::of(runtime, puyo).hop_table_index();
                let delta = rom_word(runtime, HOPS + Y_SPEED_DELTA, index);
                let speed = (u32::from(delta >> 8) << 16) | (u32::from(delta & 0xff) << 8);
                let key = combat::current(runtime);
                if api::move_vertical(runtime, key, speed as i32) == Blocked {
                    let hop = (random_eighth(runtime, puyo) & 1) + 5;
                    let mut edit = PuyoEdit::of(runtime, puyo);
                    *edit.hop_type() = hop;
                    *edit.function() = GROUNDED;
                }
            }
            _ => return false,
        },
        _ => return false,
    }
    true
}
