//! Beetom crawls, hops and lunges at Samus, then drains her until shaken off ($A8:B65E-$A8:C142).
use crate::actors::access::AccessMode::Native;
use crate::actors::api::{self, Axis, DeathHandling, MovementResult::Blocked, ShotHandling};
use crate::actors::combat::{self, current_slot};
use crate::actors::host::{ActorLease, LeaseActors};
use crate::actors::{layouts::part_view, sidehopper::initial_hop_phase};
use crate::enemy_projectiles::host::Effects as _;
use crate::player_projectiles::{content::table, host::Host as _};
use crate::ram::words;
use crate::session_slot;

part_view! {
    Beetom: EnemyLayout40 {
        words: [var_a, y_speed_index, function, function_timer, button_counter, previous_input, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [
            instruction_list, var_01, short_hop_y_speed_index, long_hop_y_speed_index, lunge_y_speed_index,
            falling, samus_x_offset, samus_y_offset, attached, direction, var_0a, var_0b, var_0c, var_0d,
            var_0e, var_0f, var_10, var_11, var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19,
            var_1a, var_1b, var_1c, var_1d, var_1e, var_1f,
        ],
    }
}

/// Bank $A8 beetom functions, stored in `function`.
pub mod function {
    pub const DECIDE_ACTION: u16 = 0xb814;
    pub const DECIDE_ACTION_SAMUS_FAR: u16 = 0xb82f;
    pub const START_IDLING: u16 = 0xb84f;
    pub const START_CRAWLING_LEFT: u16 = 0xb85f;
    pub const START_CRAWLING_RIGHT: u16 = 0xb873;
    pub const START_SHORT_HOP_LEFT: u16 = 0xb887;
    pub const START_SHORT_HOP_RIGHT: u16 = 0xb8a9;
    pub const START_LONG_HOP_LEFT: u16 = 0xb8cb;
    pub const START_LONG_HOP_RIGHT: u16 = 0xb8ed;
    pub const DECIDE_ACTION_SAMUS_NEAR: u16 = 0xb90f;
    pub const START_DRAINING_FACING_LEFT: u16 = 0xb952;
    pub const START_DRAINING_FACING_RIGHT: u16 = 0xb966;
    pub const START_DROPPING: u16 = 0xb97a;
    pub const START_BEING_FLUNG: u16 = 0xb9a2;
    pub const IDLING: u16 = 0xb9b2;
    pub const CRAWLING_LEFT: u16 = 0xb9c1;
    pub const CRAWLING_RIGHT: u16 = 0xba24;
    pub const SHORT_HOP_LEFT: u16 = 0xba84;
    pub const SHORT_HOP_RIGHT: u16 = 0xbab7;
    pub const LONG_HOP_LEFT: u16 = 0xbb55;
    pub const LONG_HOP_RIGHT: u16 = 0xbb88;
    pub const LUNGE_LEFT: u16 = 0xbc26;
    pub const LUNGE_RIGHT: u16 = 0xbc5a;
    pub const DRAINING_FACING_LEFT: u16 = 0xbcf8;
    pub const DRAINING_FACING_RIGHT: u16 = 0xbd42;
    pub const DROPPING: u16 = 0xbd9d;
    pub const BEING_FLUNG: u16 = 0xbdc5;
}

/// Bank $A8 beetom instruction lists, facing left then right.
const CRAWLING_LISTS: [u16; 2] = [0xb696, 0xb6f2];
const HOP_LISTS: [u16; 2] = [0xb6ac, 0xb708];
const DRAINING_LISTS: [u16; 2] = [0xb6cc, 0xb728];
/// $A8:B74E: the functions when Samus is far, by random number.
const FAR_FUNCTIONS: u32 = 0xa8_b74e;
/// The short hop, long hop and lunge heights and Y accelerations.
const HOPS: [(u16, u16); 3] = [(0x3000, 4), (0x4000, 5), (0x3000, 3)];
const BUTTON_PRESSES: u16 = 64;
const MAX_FALL_SPEED: u16 = 64;
const SIGN: u16 = 0x8000;

fn facing(runtime: &impl ActorLease, beetom: i32) -> usize {
    usize::from(session_slot!(runtime, Beetom[beetom].direction) != 0)
}
fn set_function(runtime: &mut impl ActorLease, beetom: i32, function: u16) {
    session_slot!(runtime, Beetom[beetom].function = function);
}
/// Set beetom instruction list ($A8:B762).
fn set_instruction_list(runtime: &mut impl ActorLease, beetom: i32, list: u16) {
    session_slot!(runtime, Beetom[beetom].instruction_list = list);
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
}
fn moved_horizontally(runtime: &mut impl ActorLease, amount: i32) -> bool {
    let key = combat::current(runtime);
    api::move_horizontal(runtime, key, amount, api::SlopePolicy::Ignore) == Blocked
}
fn blocked_vertically(runtime: &mut impl ActorLease, amount: i32) -> bool {
    let key = combat::current(runtime);
    api::move_vertical(runtime, key, amount) == Blocked
}
fn samus_left(runtime: &impl ActorLease) -> bool {
    api::player_delta(runtime, combat::current(runtime), Axis::X) & SIGN != 0
}
fn reverse(runtime: &mut impl ActorLease, beetom: i32) {
    session_slot!(runtime, Beetom[beetom].direction, |direction| direction ^ 1);
    set_function(runtime, beetom, function::START_DROPPING);
}
/// Initialisation AI - beetom ($A8:B776).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let beetom = current_slot(runtime);
    session_slot!(runtime, Beetom[beetom]; function_timer = 0, function = 0, falling = 0, attached = 0,
        var_a = 0, button_counter = BUTTON_PRESSES);
    let input = runtime.word(words::JOYPAD1_LASTKEYS);
    session_slot!(runtime, Beetom[beetom].previous_input = input);
    runtime.write_slice_le(words::RANDOM_NUMBER, 0x17);
    for (hop, (height, step)) in HOPS.into_iter().enumerate() {
        let phase = initial_hop_phase(height, step, runtime.native_image());
        match hop {
            0 => session_slot!(runtime, Beetom[beetom].short_hop_y_speed_index = phase),
            1 => session_slot!(runtime, Beetom[beetom].long_hop_y_speed_index = phase),
            _ => session_slot!(runtime, Beetom[beetom].lunge_y_speed_index = phase),
        }
    }
    let list = CRAWLING_LISTS[usize::from(!samus_left(runtime))];
    set_instruction_list(runtime, beetom, list);
    set_function(runtime, beetom, function::DECIDE_ACTION);
}
fn start_hop(runtime: &mut impl ActorLease, beetom: i32, long: bool, left: bool) {
    use function::*;
    let phase = match long {
        false => session_slot!(runtime, Beetom[beetom].short_hop_y_speed_index),
        true => session_slot!(runtime, Beetom[beetom].long_hop_y_speed_index),
    };
    let hops = [
        [SHORT_HOP_RIGHT, SHORT_HOP_LEFT],
        [LONG_HOP_RIGHT, LONG_HOP_LEFT],
    ];
    let function = hops[usize::from(long)][usize::from(left)];
    session_slot!(runtime, Beetom[beetom]; y_speed_index = phase, function = function, falling = 0);
    set_instruction_list(runtime, beetom, HOP_LISTS[usize::from(!left)]);
}
/// Beetom function - crawling left ($A8:B9C1) or right ($A8:BA24); turn at a wall or drop.
fn crawl(runtime: &mut impl ActorLease, beetom: i32, left: bool) {
    let timer = session_slot!(runtime, Beetom[beetom].function_timer).wrapping_sub(1);
    session_slot!(runtime, Beetom[beetom].function_timer = timer);
    if timer & SIGN != 0 {
        session_slot!(runtime, Beetom[beetom]; function_timer = 0x40, function = function::DECIDE_ACTION);
        return;
    }
    let turn = [
        function::START_CRAWLING_LEFT,
        function::START_CRAWLING_RIGHT,
    ];
    let turn = turn[usize::from(left)];
    let ahead: u16 = if left { 0xfff8 } else { 8 };
    let x = session_slot!(runtime, Beetom[beetom].base.x_pos);
    session_slot!(runtime, Beetom[beetom].base.x_pos = x.wrapping_add(ahead));
    if blocked_vertically(runtime, 1 << 16) {
        session_slot!(runtime, Beetom[beetom].base.x_pos, |x| x
            .wrapping_sub(ahead));
        if moved_horizontally(runtime, if left { -0x4000 } else { 0x4000 }) {
            set_function(runtime, beetom, turn);
        }
    } else {
        set_function(runtime, beetom, turn);
        session_slot!(runtime, Beetom[beetom].base.y_pos, |y| y.wrapping_sub(1));
        session_slot!(runtime, Beetom[beetom].base.x_pos, |x| x
            .wrapping_sub(ahead));
    }
}
/// Beetom Y movement - rising or falling, by Y speed index `step`.
fn move_vertically(runtime: &mut impl ActorLease, beetom: i32, falling: bool, step: u16) {
    let phase = session_slot!(runtime, Beetom[beetom].y_speed_index);
    let offset = 8 * usize::from(phase) + if falling { 0 } else { 4 };
    let amount = runtime.lease_enemy_speed32(offset) as i32;
    if blocked_vertically(runtime, amount) {
        let next = [function::START_DROPPING, function::DECIDE_ACTION][usize::from(falling)];
        return set_function(runtime, beetom, next);
    }
    let next = match falling {
        true => phase.wrapping_add(step),
        false => phase.wrapping_sub(step),
    };
    session_slot!(runtime, Beetom[beetom].y_speed_index = next);
    if falling && next.wrapping_sub(MAX_FALL_SPEED) & SIGN == 0 {
        session_slot!(runtime, Beetom[beetom].y_speed_index = MAX_FALL_SPEED);
    } else if !falling && next & SIGN != 0 {
        session_slot!(runtime, Beetom[beetom]; y_speed_index = 0, falling = 1);
    }
}
/// Beetom function - short hop, long hop or lunge, left or right; drop at a wall.
fn hop(runtime: &mut impl ActorLease, beetom: i32, long: bool, lunge: bool, left: bool) {
    let falling = session_slot!(runtime, Beetom[beetom].falling) != 0;
    let step = match (lunge && falling, long || lunge) {
        (true, _) => 3,
        (false, true) => 5,
        (false, false) => 4,
    };
    move_vertically(runtime, beetom, falling, step);
    let speed = if lunge { 0x30000 } else { 0x4000 };
    if moved_horizontally(runtime, if left { -speed } else { speed }) {
        reverse(runtime, beetom);
    }
}
/// Beetom function - draining Samus - facing left ($A8:BCF8) or right ($A8:BD42).
fn drain(runtime: &mut impl ActorLease, beetom: i32, push_left: bool) {
    if session_slot!(runtime, Beetom[beetom].button_counter) != 0 {
        let x = runtime.word(words::SAMUS_X_POS);
        session_slot!(runtime, Beetom[beetom].base.x_pos = x);
        let y = runtime.word(words::SAMUS_Y_POS).wrapping_sub(4);
        session_slot!(runtime, Beetom[beetom].base.y_pos = y);
        // Update beetom button counter ($A8:BD8C).
        let input = runtime.word(words::JOYPAD1_LASTKEYS);
        if input != session_slot!(runtime, Beetom[beetom].previous_input) {
            let count = session_slot!(runtime, Beetom[beetom].button_counter).wrapping_sub(1);
            session_slot!(runtime, Beetom[beetom]; previous_input = input, button_counter = count);
        }
        return;
    }
    let push = if push_left { -16 << 16 } else { 16 << 16 };
    if moved_horizontally(runtime, push) {
        session_slot!(runtime, Beetom[beetom].direction = u16::from(!push_left));
        moved_horizontally(runtime, -2 * push);
    }
    session_slot!(runtime, Beetom[beetom]; attached = 0, function = function::START_BEING_FLUNG);
}
/// Main AI - beetom ($A8:B80D): run its function.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    use function::*;
    let mut lease = game.lease();
    let runtime = &mut lease;
    let beetom = current_slot(runtime);
    let function = session_slot!(runtime, Beetom[beetom].function);
    match function {
        DECIDE_ACTION => {
            let key = combat::current(runtime);
            let near = api::player_within(runtime, key, Axis::X, 0x60);
            let next = [DECIDE_ACTION_SAMUS_FAR, DECIDE_ACTION_SAMUS_NEAR][usize::from(near)];
            set_function(runtime, beetom, next);
        }
        DECIDE_ACTION_SAMUS_FAR => {
            runtime.next_random();
            let random = runtime.word(words::RANDOM_NUMBER);
            let next = table(runtime, FAR_FUNCTIONS, (random & 7) as isize);
            set_function(runtime, beetom, next);
            session_slot!(runtime, Beetom[beetom].direction = random & 1);
        }
        START_IDLING => {
            session_slot!(runtime, Beetom[beetom]; function_timer = 0x20, function = IDLING);
        }
        START_CRAWLING_LEFT | START_CRAWLING_RIGHT => {
            let right = usize::from(function == START_CRAWLING_RIGHT);
            set_function(runtime, beetom, [CRAWLING_LEFT, CRAWLING_RIGHT][right]);
            set_instruction_list(runtime, beetom, CRAWLING_LISTS[right]);
        }
        START_SHORT_HOP_LEFT
        | START_SHORT_HOP_RIGHT
        | START_LONG_HOP_LEFT
        | START_LONG_HOP_RIGHT => {
            let long = matches!(function, START_LONG_HOP_LEFT | START_LONG_HOP_RIGHT);
            let left = matches!(function, START_SHORT_HOP_LEFT | START_LONG_HOP_LEFT);
            start_hop(runtime, beetom, long, left);
        }
        DECIDE_ACTION_SAMUS_NEAR => {
            let phase = session_slot!(runtime, Beetom[beetom].lunge_y_speed_index);
            let right = !samus_left(runtime);
            let lunge = [LUNGE_LEFT, LUNGE_RIGHT][usize::from(right)];
            session_slot!(runtime, Beetom[beetom]; y_speed_index = phase, function = lunge,
                direction = u16::from(right));
            set_instruction_list(runtime, beetom, HOP_LISTS[usize::from(right)]);
            session_slot!(runtime, Beetom[beetom].falling = 0);
        }
        START_DRAINING_FACING_LEFT | START_DRAINING_FACING_RIGHT => {
            let right = usize::from(function == START_DRAINING_FACING_RIGHT);
            set_instruction_list(runtime, beetom, DRAINING_LISTS[right]);
            let draining = [DRAINING_FACING_LEFT, DRAINING_FACING_RIGHT][right];
            set_function(runtime, beetom, draining);
        }
        START_DROPPING => {
            let list = CRAWLING_LISTS[facing(runtime, beetom)];
            set_instruction_list(runtime, beetom, list);
            session_slot!(runtime, Beetom[beetom]; falling = 0, function = DROPPING);
        }
        START_BEING_FLUNG => {
            session_slot!(runtime, Beetom[beetom]; y_speed_index = 0, function = BEING_FLUNG);
        }
        IDLING => {
            let timer = session_slot!(runtime, Beetom[beetom].function_timer).wrapping_sub(1);
            session_slot!(runtime, Beetom[beetom].function_timer = timer);
            if timer & SIGN != 0 {
                set_function(runtime, beetom, DECIDE_ACTION);
            }
        }
        CRAWLING_LEFT | CRAWLING_RIGHT => crawl(runtime, beetom, function == CRAWLING_LEFT),
        SHORT_HOP_LEFT | SHORT_HOP_RIGHT => {
            hop(runtime, beetom, false, false, function == SHORT_HOP_LEFT)
        }
        LONG_HOP_LEFT | LONG_HOP_RIGHT => {
            hop(runtime, beetom, true, false, function == LONG_HOP_LEFT)
        }
        LUNGE_LEFT | LUNGE_RIGHT => hop(runtime, beetom, false, true, function == LUNGE_LEFT),
        DRAINING_FACING_LEFT | DRAINING_FACING_RIGHT => {
            drain(runtime, beetom, function == DRAINING_FACING_RIGHT)
        }
        // Land, then crawl the way it faces.
        DROPPING => {
            if blocked_vertically(runtime, 3 << 16) {
                let next = [START_CRAWLING_LEFT, START_CRAWLING_RIGHT][facing(runtime, beetom)];
                set_function(runtime, beetom, next);
            }
        }
        // Being flung: fall, pushed 2 px a frame against its facing.
        BEING_FLUNG => {
            move_vertically(runtime, beetom, true, 1);
            let push = [2 << 16, -2 << 16][facing(runtime, beetom)];
            if moved_horizontally(runtime, push) {
                reverse(runtime, beetom);
            }
        }
        _ => runtime.unreachable(),
    }
}
/// Enemy touch - beetom ($A8:BE2E): latch on, and hurt Samus every 40h frames.
pub fn touch<H: LeaseActors + ?Sized>(game: &mut H) {
    use function::*;
    let mut lease = game.lease();
    let runtime = &mut lease;
    let beetom = current_slot(runtime);
    if session_slot!(runtime, Beetom[beetom].attached) == 0 {
        let start = [START_DRAINING_FACING_LEFT, START_DRAINING_FACING_RIGHT];
        set_function(runtime, beetom, start[facing(runtime, beetom)]);
        session_slot!(runtime, Beetom[beetom]; attached = 1, button_counter = BUTTON_PRESSES);
        session_slot!(runtime, Beetom[beetom].base.layer = 2);
        let x = session_slot!(runtime, Beetom[beetom].base.x_pos);
        let x = runtime.word(words::SAMUS_X_POS).wrapping_sub(x);
        session_slot!(runtime, Beetom[beetom].samus_x_offset = x);
        let y = session_slot!(runtime, Beetom[beetom].base.y_pos);
        let y = runtime.word(words::SAMUS_Y_POS).wrapping_sub(y);
        session_slot!(runtime, Beetom[beetom].samus_y_offset = y);
    }
    let hurt = runtime.word(words::SAMUS_CONTACT_DAMAGE_INDEX) != 0 || {
        if runtime.word(words::RANDOM_ENEMY_COUNTER) & 7 == 7
            && runtime.word(words::SAMUS_HEALTH).wrapping_sub(30) & SIGN == 0
        {
            runtime.queue_sound(3, 6, 0x2d); // Health tick.
        }
        session_slot!(runtime, Beetom[beetom].base.frame_counter) & 0x3f == 0x3f
    };
    if hurt {
        combat::contact_reaction(runtime, DeathHandling::Finalize);
        runtime.write_slice_le(words::SAMUS_INVINCIBILITY_TIMER, 0);
        runtime.write_slice_le(words::SAMUS_KNOCKBACK_TIMER, 0);
    }
}
/// Enemy shot - beetom ($A8:BEAC): a frozen beetom drops off Samus.
pub fn shot<H: LeaseActors + ?Sized>(game: &mut H) {
    use function::*;
    let mut lease = game.lease();
    let runtime = &mut lease;
    combat::shot_reaction(runtime, ShotHandling::Finalize);
    let beetom = current_slot(runtime);
    let function = session_slot!(runtime, Beetom[beetom].function);
    if session_slot!(runtime, Beetom[beetom].base.frozen_timer) != 0
        && matches!(function, DRAINING_FACING_LEFT | DRAINING_FACING_RIGHT)
    {
        set_function(runtime, beetom, START_DROPPING);
    }
    session_slot!(runtime, Beetom[beetom].attached = 0);
}
