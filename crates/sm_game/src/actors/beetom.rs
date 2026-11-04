//! Beetom crawls, hops and lunges at Samus, then drains her until shaken off ($A8:B65E-$A8:C142).
use crate::actors::access::AccessMode::Native;
use crate::actors::api::{self, Axis, DeathHandling, MovementResult::Blocked, ShotHandling};
use crate::actors::combat;
use crate::actors::host::ActorLease;
use crate::actors::sidehopper::initial_hop_phase;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::controller_state::Word as Input;
use crate::player_projectiles::content::table;

/// A lease on a game whose slots are the Session's typed storage.
pub trait Lease: ActorLease + SlotState {}
impl<T: ActorLease + SlotState + ?Sized> Lease for T {}

slot_view! {
    /// A beetom's slot.
    Beetom, BeetomEdit {
        record: [
            var_a = var_a, y_speed_index = var_b, function = var_c, function_timer = var_d,
            button_counter = var_e, previous_input = var_f,
        ],
        extended: [
            instruction_list = var_00, short_hop_y_speed_index = var_02,
            long_hop_y_speed_index = var_03, lunge_y_speed_index = var_04, falling = var_05,
            samus_x_offset = var_06, samus_y_offset = var_07, attached = var_08,
            direction = var_09,
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

fn facing(runtime: &impl Lease, beetom: u16) -> usize {
    usize::from(Beetom::of(runtime, beetom).direction() != 0)
}
fn set_function(runtime: &mut impl Lease, beetom: u16, function: u16) {
    *BeetomEdit::of(runtime, beetom).function() = function;
}
/// Set beetom instruction list ($A8:B762).
fn set_instruction_list(runtime: &mut impl Lease, beetom: u16, list: u16) {
    *BeetomEdit::of(runtime, beetom).instruction_list() = list;
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| actor.restart_animation(list));
}
fn moved_horizontally(runtime: &mut impl Lease, amount: i32) -> bool {
    let key = combat::current(runtime);
    api::move_horizontal(runtime, key, amount, api::SlopePolicy::Ignore) == Blocked
}
fn blocked_vertically(runtime: &mut impl Lease, amount: i32) -> bool {
    let key = combat::current(runtime);
    api::move_vertical(runtime, key, amount) == Blocked
}
fn samus_left(runtime: &impl Lease) -> bool {
    api::player_delta(runtime, combat::current(runtime), Axis::X) & SIGN != 0
}
fn reverse(runtime: &mut impl Lease, beetom: u16) {
    *BeetomEdit::of(runtime, beetom).direction() ^= 1;
    set_function(runtime, beetom, function::START_DROPPING);
}
/// Initialisation AI - beetom ($A8:B776).
pub fn initialize(runtime: &mut impl Lease) {
    let beetom = combat::current(runtime).byte_index();
    {
        let mut part = BeetomEdit::of(runtime, beetom);
        *part.function_timer() = 0;
        *part.function() = 0;
        *part.falling() = 0;
        *part.attached() = 0;
        *part.var_a() = 0;
        *part.button_counter() = BUTTON_PRESSES;
    }
    let input = *Input::Joypad1Lastkeys.get(&runtime.session().control);
    *BeetomEdit::of(runtime, beetom).previous_input() = input;
    runtime.session_mut().timing.random_number = 0x17;
    runtime.words_stored();
    for (hop, (height, step)) in HOPS.into_iter().enumerate() {
        let phase = initial_hop_phase(height, step, runtime.native_image());
        match hop {
            0 => *BeetomEdit::of(runtime, beetom).short_hop_y_speed_index() = phase,
            1 => *BeetomEdit::of(runtime, beetom).long_hop_y_speed_index() = phase,
            _ => *BeetomEdit::of(runtime, beetom).lunge_y_speed_index() = phase,
        }
    }
    let list = CRAWLING_LISTS[usize::from(!samus_left(runtime))];
    set_instruction_list(runtime, beetom, list);
    set_function(runtime, beetom, function::DECIDE_ACTION);
}
fn start_hop(runtime: &mut impl Lease, beetom: u16, long: bool, left: bool) {
    use function::*;
    let phase = match long {
        false => Beetom::of(runtime, beetom).short_hop_y_speed_index(),
        true => Beetom::of(runtime, beetom).long_hop_y_speed_index(),
    };
    let hops = [
        [SHORT_HOP_RIGHT, SHORT_HOP_LEFT],
        [LONG_HOP_RIGHT, LONG_HOP_LEFT],
    ];
    let function = hops[usize::from(long)][usize::from(left)];
    {
        let mut part = BeetomEdit::of(runtime, beetom);
        *part.y_speed_index() = phase;
        *part.function() = function;
        *part.falling() = 0;
    }
    set_instruction_list(runtime, beetom, HOP_LISTS[usize::from(!left)]);
}
/// Beetom function - crawling left ($A8:B9C1) or right ($A8:BA24); turn at a wall or drop.
fn crawl(runtime: &mut impl Lease, beetom: u16, left: bool) {
    let timer = Beetom::of(runtime, beetom).function_timer().wrapping_sub(1);
    *BeetomEdit::of(runtime, beetom).function_timer() = timer;
    if timer & SIGN != 0 {
        {
            let mut part = BeetomEdit::of(runtime, beetom);
            *part.function_timer() = 0x40;
            *part.function() = function::DECIDE_ACTION;
        }
        return;
    }
    let turn = [
        function::START_CRAWLING_LEFT,
        function::START_CRAWLING_RIGHT,
    ];
    let turn = turn[usize::from(left)];
    let ahead: u16 = if left { 0xfff8 } else { 8 };
    let x = Beetom::of(runtime, beetom).actor().x.whole;
    BeetomEdit::of(runtime, beetom).actor().x.whole = x.wrapping_add(ahead);
    if blocked_vertically(runtime, 1 << 16) {
        {
            let mut part = BeetomEdit::of(runtime, beetom);
            let x = part.actor().x.whole;
            part.actor().x.whole = x.wrapping_sub(ahead);
        }
        if moved_horizontally(runtime, if left { -0x4000 } else { 0x4000 }) {
            set_function(runtime, beetom, turn);
        }
    } else {
        set_function(runtime, beetom, turn);
        {
            let mut part = BeetomEdit::of(runtime, beetom);
            let old = part.actor().y.whole;
            part.actor().y.whole = old.wrapping_sub(1);
        }
        let mut part = BeetomEdit::of(runtime, beetom);
        let x = part.actor().x.whole;
        part.actor().x.whole = x.wrapping_sub(ahead);
    }
}
/// Beetom Y movement - rising or falling, by Y speed index `step`.
fn move_vertically(runtime: &mut impl Lease, beetom: u16, falling: bool, step: u16) {
    let phase = Beetom::of(runtime, beetom).y_speed_index();
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
    *BeetomEdit::of(runtime, beetom).y_speed_index() = next;
    if falling && next.wrapping_sub(MAX_FALL_SPEED) & SIGN == 0 {
        *BeetomEdit::of(runtime, beetom).y_speed_index() = MAX_FALL_SPEED;
    } else if !falling && next & SIGN != 0 {
        let mut part = BeetomEdit::of(runtime, beetom);
        *part.y_speed_index() = 0;
        *part.falling() = 1;
    }
}
/// Beetom function - short hop, long hop or lunge, left or right; drop at a wall.
fn hop(runtime: &mut impl Lease, beetom: u16, long: bool, lunge: bool, left: bool) {
    let falling = Beetom::of(runtime, beetom).falling() != 0;
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
fn drain(runtime: &mut impl Lease, beetom: u16, push_left: bool) {
    if Beetom::of(runtime, beetom).button_counter() != 0 {
        let x = runtime.session().player.x;
        BeetomEdit::of(runtime, beetom).actor().x.whole = x;
        let y = runtime.session().player.y.wrapping_sub(4);
        BeetomEdit::of(runtime, beetom).actor().y.whole = y;
        // Update beetom button counter ($A8:BD8C).
        let input = *Input::Joypad1Lastkeys.get(&runtime.session().control);
        if input != Beetom::of(runtime, beetom).previous_input() {
            let count = Beetom::of(runtime, beetom).button_counter().wrapping_sub(1);
            let mut part = BeetomEdit::of(runtime, beetom);
            *part.previous_input() = input;
            *part.button_counter() = count;
        }
        return;
    }
    let push = if push_left { -16 << 16 } else { 16 << 16 };
    if moved_horizontally(runtime, push) {
        *BeetomEdit::of(runtime, beetom).direction() = u16::from(!push_left);
        moved_horizontally(runtime, -2 * push);
    }
    let mut part = BeetomEdit::of(runtime, beetom);
    *part.attached() = 0;
    *part.function() = function::START_BEING_FLUNG;
}
/// Main AI - beetom ($A8:B80D): run its function.
pub fn main_ai(runtime: &mut impl crate::actors::species::TableLease) {
    use function::*;
    let beetom = combat::current(runtime).byte_index();
    let function = Beetom::of(runtime, beetom).function();
    match function {
        DECIDE_ACTION => {
            let key = combat::current(runtime);
            let near = api::player_within(runtime, key, Axis::X, 0x60);
            let next = [DECIDE_ACTION_SAMUS_FAR, DECIDE_ACTION_SAMUS_NEAR][usize::from(near)];
            set_function(runtime, beetom, next);
        }
        DECIDE_ACTION_SAMUS_FAR => {
            runtime.next_random();
            let random = runtime.session().timing.random_number;
            let next = table(runtime, FAR_FUNCTIONS, (random & 7) as isize);
            set_function(runtime, beetom, next);
            *BeetomEdit::of(runtime, beetom).direction() = random & 1;
        }
        START_IDLING => {
            let mut part = BeetomEdit::of(runtime, beetom);
            *part.function_timer() = 0x20;
            *part.function() = IDLING;
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
            let phase = Beetom::of(runtime, beetom).lunge_y_speed_index();
            let right = !samus_left(runtime);
            let lunge = [LUNGE_LEFT, LUNGE_RIGHT][usize::from(right)];
            {
                let mut part = BeetomEdit::of(runtime, beetom);
                *part.y_speed_index() = phase;
                *part.function() = lunge;
                *part.direction() = u16::from(right);
            }
            set_instruction_list(runtime, beetom, HOP_LISTS[usize::from(right)]);
            *BeetomEdit::of(runtime, beetom).falling() = 0;
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
            let mut part = BeetomEdit::of(runtime, beetom);
            *part.falling() = 0;
            *part.function() = DROPPING;
        }
        START_BEING_FLUNG => {
            let mut part = BeetomEdit::of(runtime, beetom);
            *part.y_speed_index() = 0;
            *part.function() = BEING_FLUNG;
        }
        IDLING => {
            let timer = Beetom::of(runtime, beetom).function_timer().wrapping_sub(1);
            *BeetomEdit::of(runtime, beetom).function_timer() = timer;
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
        _ => crate::diagnostics::Fatal::unreachable(runtime),
    }
}
/// Enemy touch - beetom ($A8:BE2E): latch on, and hurt Samus every 40h frames.
pub fn touch(runtime: &mut impl crate::actors::species::TableLease) {
    use function::*;
    let beetom = combat::current(runtime).byte_index();
    if Beetom::of(runtime, beetom).attached() == 0 {
        let start = [START_DRAINING_FACING_LEFT, START_DRAINING_FACING_RIGHT];
        set_function(runtime, beetom, start[facing(runtime, beetom)]);
        {
            let mut part = BeetomEdit::of(runtime, beetom);
            *part.attached() = 1;
            *part.button_counter() = BUTTON_PRESSES;
        }
        BeetomEdit::of(runtime, beetom).actor().layer = 2;
        let x = Beetom::of(runtime, beetom).actor().x.whole;
        let x = runtime.session().player.x.wrapping_sub(x);
        *BeetomEdit::of(runtime, beetom).samus_x_offset() = x;
        let y = Beetom::of(runtime, beetom).actor().y.whole;
        let y = runtime.session().player.y.wrapping_sub(y);
        *BeetomEdit::of(runtime, beetom).samus_y_offset() = y;
    }
    let hurt = runtime.session().player.contact.contact_attack != 0 || {
        if runtime.session().actors.controls.phase_counter & 7 == 7
            && runtime.session().inventory.health.current.wrapping_sub(30) & SIGN == 0
        {
            crate::enemy_projectiles::host::Effects::queue_sound(runtime, 3, 6, 0x2d); // Health tick.
        }
        Beetom::of(runtime, beetom).actor().frame & 0x3f == 0x3f
    };
    if hurt {
        combat::contact_reaction(runtime, DeathHandling::Finalize);
        let player = &mut runtime.session_mut().player;
        player.invincibility = 0;
        player.knockback_timer = 0;
    }
}
/// Enemy shot - beetom ($A8:BEAC): a frozen beetom drops off Samus.
pub fn shot(runtime: &mut impl crate::actors::species::TableLease) {
    use function::*;
    combat::shot_reaction(runtime, ShotHandling::Finalize);
    let beetom = combat::current(runtime).byte_index();
    let function = Beetom::of(runtime, beetom).function();
    if Beetom::of(runtime, beetom).actor().frozen != 0
        && matches!(function, DRAINING_FACING_LEFT | DRAINING_FACING_RIGHT)
    {
        set_function(runtime, beetom, START_DROPPING);
    }
    *BeetomEdit::of(runtime, beetom).attached() = 0;
}
