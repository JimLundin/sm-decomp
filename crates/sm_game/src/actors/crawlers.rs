//! The wall and floor crawlers, which turn at edges and fall when shaken
//! loose: the Sciser, Zero, Viola, Zeela, Sova and Geemer, and the Wrecked
//! Ship's orange Geemer, which turns towards Samus (bank $A3:DFA2-$A3:E9AF).
//! The bank log calls the family "creepy crawlies" and the Geemer "zoomer"
//! (with a "stone zoomer" palette variant). Was `wall_crawlers`.
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api::{self, Axis, MovementResult, SlopePolicy};
use crate::actors::combat;
use crate::actors::host::{ActorLease, LeaseActors};
use crate::actors::layouts::part_view;
use crate::enemy_script::EnemyScript;
use crate::geometry::Point;
use crate::player_projectiles::content::{self, table};
use crate::ram::word_arrays;
use crate::session_access::SessionAccess as _;
use crate::session_slot;

/// Which crawler is initialising, by species.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// $D77F.
    Sciser,
    /// $D7BF.
    Zero,
    /// $DABF.
    Viola,
    /// $DC7F.
    Zeela,
    /// $DCBF.
    Sova,
    /// $DCFF, and its gray variant $DD3F (the bank log's zoomer and stone zoomer).
    Geemer,
    /// $DC3F, the Wrecked Ship's orange Geemer.
    OrangeGeemer,
}
impl Kind {
    /// Zeela, Sova and Geemer share one initial instruction list table.
    fn shares_geemer_lists(self) -> bool {
        matches!(self, Self::Zeela | Self::Sova | Self::Geemer)
    }
}

part_view! {
    /// A crawler's view of its slot and extended RAM.
    Crawler: EnemyLayout40 {
        words: [x_velocity, y_velocity, var_c, var_d, var_e, function, speed_index, instruction_lists],
        pad1: [u8; 26696],
        words: [
            var_00, falling_y_subvelocity, falling_y_velocity, non_falling_function, turn_counter,
        ],
        _: [u16; 27],
    }
}

/// The crawlers' velocities, by `variables` lane (X then Y); the function is
/// the pre-instruction word.
const X_VELOCITY: usize = 0;

/// Bank $A3 crawler functions (the bank log's creepy crawly functions), stored in `function`.
pub mod function {
    /// $A3:E6C1, an RTL: the function before the first instruction sets one.
    pub const NONE: u16 = 0xe6c1;
    pub const CRAWLING_VERTICALLY: u16 = 0xe6c8;
    pub const FALLING: u16 = 0xe785;
    /// The orange Geemer's initial function.
    pub const ORANGE_GEEMER_INITIAL: u16 = 0xe08a;
}

const BANK: u32 = 0xa3;
/// $A3:E5F0: the crawler speeds; $A3:E630: the instruction list
/// pointer tables, by axis and direction; $A3:E931: the slope speed scales.
const SPEEDS: u32 = 0xa3_e5f0;
const LISTS_X_NEGATIVE: u32 = 0xa3_e630;
const LISTS_X_POSITIVE: u32 = 0xa3_e63c;
const LISTS_Y_NEGATIVE: u32 = 0xa3_e648;
const LISTS_Y_POSITIVE: u32 = 0xa3_e654;
const SLOPE_SCALES: u32 = 0xa3_e931;
/// The orange Geemer's instruction lists: upside right, left, down and up.
const ORANGE_UPSIDE_RIGHT: u16 = 0xdfcb;
const ORANGE_UPSIDE_LEFT: u16 = 0xdfe7;
const ORANGE_UPSIDE_DOWN: u16 = 0xe003;
const ORANGE_UPSIDE_UP: u16 = 0xe01f;
/// $A3:804D: no spritemap.
const NO_SPRITEMAP: u16 = 0x804d;
/// A speed index of FFh keeps the speeds; landing gives them 80h.
const KEEP_SPEED: u16 = 255;
const LANDED_SPEED: u16 = 128;
/// Properties bits 0-1: the crawler's orientation.
const ORIENTATION: u16 = 3;
/// An earthquake of type 20 at timer 30 (a power bomb) shakes crawlers loose.
const SHAKE_TIMER: u16 = 30;
const SHAKE_KIND: u16 = 20;
/// Four consecutive turns without ground make a crawler fall.
const MAX_TURNS: u16 = 4;
/// Falling accelerates 0.8000h px a frame, up to 4 px.
const FALL_ACCELERATION: u32 = 0x8000;
const MAX_FALL_SPEED: u16 = 4;
/// $7E:7F0002: block BTS; blocks of type 1 (slope) with BTS 5 or more scale
/// the speed.
const BTS: usize = 0x16402;
const BLOCK_TYPE: u16 = 0xf000;
const SLOPE_BLOCK: u16 = 0x1000;
const SLOPE_BTS_BITS: u8 = 31;
const FIRST_STEEP_SLOPE: u8 = 5;
const SIGN: u16 = 0x8000;

/// Common crawler initialisation AI ($A3:E67A) with `common`, or the speed
/// half the orange Geemer shares.
fn set_speed(runtime: &mut impl ActorLease, key: Key, common: bool) {
    if common {
        runtime.edit_actor(key, Native, |actor| {
            actor.spritemap = NO_SPRITEMAP;
            actor.instruction_delay = 1;
            actor.pre_instruction = function::NONE;
        });
    }
    let parameter = runtime.resolve_actor(key, Native).parameters[0];
    if !common || parameter != KEEP_SPEED {
        let speed = table(runtime, SPEEDS, parameter as isize);
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[0] = speed;
            actor.variables[1] = speed;
        });
    }
    runtime.edit_actor(key, Native, |actor| match actor.properties & ORIENTATION {
        0 => actor.variables[0] = actor.variables[0].wrapping_neg(),
        2 => actor.variables[1] = actor.variables[1].wrapping_neg(),
        _ => (),
    });
}
/// Common crawler initialisation AI ($A3:E67A) for actor `raw`, as other
/// initialisations call it. Was `prepare`.
pub fn common_initialisation<H: LeaseActors + ?Sized>(raw: u16, game: &mut H) {
    set_speed(&mut game.lease(), Key::from_legacy(raw), true);
}
/// Initialisation AI - Sciser ($A3:96E3), Zero ($A3:993B), Viola ($A3:B66F),
/// orange Geemer ($A3:E043), Zeela ($A3:E2D4), Sova ($A3:E59C) and Geemer
/// ($A3:E669).
pub fn initialize<H: LeaseActors + ?Sized>(kind: Kind, game: &mut H) {
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    // The initial instruction list pointers, and the instruction list table
    // offset some kinds set.
    let (address, lists) = match kind {
        Kind::Sciser => (0xa3_96db, Some(8)),
        Kind::Zero => (0xa3_992b, Some(10)),
        Kind::Viola => (0xa3_b667, Some(6)),
        _ if kind.shares_geemer_lists() => (0xa3_e2cc, None),
        _ => (0xa3_e03b, None),
    };
    if let Some(value) = lists {
        runtime.edit_actor(key, Native, |actor| actor.parameters[1] = value);
    }
    let index = {
        let actor = runtime.resolve_actor(key, Native);
        (if kind == Kind::Viola {
            actor.properties
        } else {
            actor.instruction
        }) & ORIENTATION
    };
    let instruction = table(runtime, address, index as isize);
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
    let key = if kind == Kind::OrangeGeemer {
        runtime.edit_actor(key, Native, |actor| {
            actor.pre_instruction = function::ORANGE_GEEMER_INITIAL
        });
        key
    } else {
        combat::current(runtime)
    };
    set_speed(runtime, key, kind != Kind::OrangeGeemer);
}
/// Main AI ($A3:E6C2, and the orange Geemer's $A3:E08B): the function. Was
/// `state_address`.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) -> u32 {
    let runtime = game.lease();
    let key = combat::current(&runtime);
    u32::from(runtime.resolve_actor(key, Native).pre_instruction) | BANK << 16
}
/// Instruction - enemy function = [[Y]] ($A3:E660, $A3:DFC2). Was `set_state`.
pub fn set_function<H: LeaseActors + ?Sized>(
    raw: u16,
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    let next = script.word(0);
    game.lease()
        .edit_actor(Key::from_legacy(raw), Native, |actor| {
            actor.pre_instruction = next
        });
    script.offset(1)
}
/// Start falling, remembering the function to return to.
fn fall(runtime: &mut impl ActorLease, key: Key) {
    let crawler = i32::from(key.byte_index());
    let function = runtime.resolve_actor(key, Native).pre_instruction;
    session_slot!(runtime, Crawler[crawler].non_falling_function = function);
    runtime.edit_actor(key, Native, |actor| {
        actor.pre_instruction = function::FALLING
    });
}
fn move_axis(
    runtime: &mut impl ActorLease,
    key: Key,
    axis: usize,
    amount: i32,
    orange: bool,
) -> bool {
    let result = if axis == X_VELOCITY {
        let slopes = if orange {
            SlopePolicy::Follow
        } else {
            SlopePolicy::Ignore
        };
        api::move_horizontal(runtime, key, amount, slopes)
    } else {
        api::move_vertical(runtime, key, amount)
    };
    result == MovementResult::Blocked
}
/// Set the instruction list for the crawler's direction along `axis`.
fn set_instruction_list(runtime: &mut impl ActorLease, key: Key, axis: usize, orange: bool) {
    let actor = runtime.resolve_actor(key, Native);
    let negative = actor.variables[1 - axis] & SIGN != 0;
    let instruction = if orange {
        match (axis, negative) {
            (0, false) => ORANGE_UPSIDE_UP,
            (0, true) => ORANGE_UPSIDE_DOWN,
            (_, false) => ORANGE_UPSIDE_LEFT,
            (_, true) => ORANGE_UPSIDE_RIGHT,
        }
    } else {
        let address = match (axis, negative) {
            (0, false) => LISTS_X_POSITIVE,
            (0, true) => LISTS_X_NEGATIVE,
            (_, false) => LISTS_Y_POSITIVE,
            (_, true) => LISTS_Y_NEGATIVE,
        };
        table(runtime, address, (actor.parameters[1] >> 1) as isize)
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.instruction_delay = 1;
    });
}
/// Adjust enemy X velocity for slopes ($A3:E8A5): on steep slopes, scale the
/// speed by the slope's scale.
fn slope_x_velocity(runtime: &mut impl ActorLease) -> i32 {
    let key = combat::current(runtime);
    let point = {
        let actor = runtime.resolve_actor(key, Native);
        let y = if actor.variables[1] & SIGN != 0 {
            actor.y.whole.wrapping_sub(actor.radius.y)
        } else {
            actor.y.whole.wrapping_add(actor.radius.y).wrapping_sub(1)
        };
        Point {
            x: actor.x.whole,
            y,
        }
    };
    api::select_block_at(runtime, point);
    let index = usize::from(runtime.session().room.collision_cursor);
    if runtime.word(word_arrays::LEVEL_DATA + 2 * index) & BLOCK_TYPE == SLOPE_BLOCK
        && runtime.lease_byte(BTS + usize::from(runtime.session().room.collision_cursor))
            & SLOPE_BTS_BITS
            >= FIRST_STEEP_SLOPE
    {
        let image = runtime.leased_image().expect("ROM must be loaded");
        let behavior =
            runtime.lease_byte(BTS + usize::from(runtime.session().room.collision_cursor));
        let scale = image.table_word(
            content::offset(SLOPE_SCALES),
            ((4 * (u16::from(behavior) & u16::from(SLOPE_BTS_BITS))) >> 1) as isize,
        );
        let speed = runtime.resolve_actor(key, Native).variables[0];
        let magnitude = if speed & SIGN != 0 {
            speed.wrapping_neg()
        } else {
            speed
        };
        let product = runtime
            .session_mut()
            .multiply_scratch
            .multiply(magnitude, scale) as i32;
        if speed & SIGN != 0 { -product } else { product }
    } else {
        i32::from(runtime.resolve_actor(key, Native).variables[0] as i16) << 8
    }
}
/// Crawler function - crawling horizontally ($A3:E7F2) or vertically
/// ($A3:E6C8), or the orange Geemer's ($A3:E168, $A3:E091): crawl
/// along the surface, turning at corners and falling off edges. Was `crawl`.
pub fn crawl<H: LeaseActors + ?Sized>(raw: u16, axis: Axis, orange: bool, game: &mut H) {
    let axis = usize::from(axis == Axis::Y);
    let runtime = &mut game.lease();
    let key = Key::from_legacy(raw);
    let crawler = i32::from(raw);
    if runtime.session().feedback.earthquake_timer == SHAKE_TIMER
        && runtime.session().feedback.earthquake_kind == SHAKE_KIND
    {
        fall(runtime, key);
    }
    let speed = i32::from(runtime.resolve_actor(key, Native).variables[axis] as i16) << 8;
    let leading = speed + if speed < 0 { -0x10000 } else { 0x10000 };
    if move_axis(runtime, key, axis, leading, orange) {
        session_slot!(runtime, Crawler[crawler].turn_counter = 0);
        if axis == X_VELOCITY {
            api::snap_to_slopes(runtime, key);
        }
        let amount = if axis == 1 && !orange {
            slope_x_velocity(runtime)
        } else {
            i32::from(runtime.resolve_actor(key, Native).variables[1 - axis] as i16) << 8
        };
        if move_axis(runtime, key, 1 - axis, amount, orange) {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[axis] = actor.variables[axis].wrapping_neg()
            });
            set_instruction_list(runtime, key, axis, orange);
        } else {
            if axis == 1 {
                api::snap_to_slopes(runtime, key);
            }
            if orange {
                let actor = runtime.resolve_actor(key, Native);
                let (position, player) = if axis == X_VELOCITY {
                    (actor.y.whole, runtime.session().player.y)
                } else {
                    (actor.x.whole, runtime.session().player.x)
                };
                let positive = player.wrapping_sub(position) & SIGN == 0;
                runtime.edit_actor(key, Native, |actor| {
                    let speed = &mut actor.variables[1 - axis];
                    if positive == (*speed & SIGN != 0) {
                        *speed = speed.wrapping_neg();
                    }
                });
            }
        }
    } else {
        let turns = session_slot!(runtime, Crawler[crawler].turn_counter).wrapping_add(1);
        session_slot!(runtime, Crawler[crawler].turn_counter = turns);
        if turns.wrapping_sub(MAX_TURNS) & SIGN != 0 {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[1 - axis] = actor.variables[1 - axis].wrapping_neg()
            });
            set_instruction_list(runtime, key, axis, orange);
        } else {
            fall(runtime, key);
        }
    }
}
/// Crawler function - falling ($A3:E785): fall until landing, then
/// return to crawling. Was `falling`.
pub fn falling<H: LeaseActors + ?Sized>(raw: u16, game: &mut H) {
    let runtime = &mut game.lease();
    let key = Key::from_legacy(raw);
    let crawler = i32::from(raw);
    let velocity = session_slot!(
        runtime,
        Crawler[crawler](falling_y_velocity, falling_y_subvelocity)
    );
    if api::move_vertical(runtime, key, velocity as i32) == MovementResult::Blocked {
        runtime.edit_actor(key, Native, |actor| {
            if actor.parameters[0] == KEEP_SPEED {
                actor.variables[0] = LANDED_SPEED;
                actor.variables[1] = LANDED_SPEED;
            }
        });
        session_slot!(runtime, Crawler[crawler]; falling_y_subvelocity = 0, falling_y_velocity = 0,
            turn_counter = 0);
        let function = session_slot!(runtime, Crawler[crawler].non_falling_function);
        runtime.edit_actor(key, Native, |actor| actor.pre_instruction = function);
    } else {
        if session_slot!(runtime, Crawler[crawler].falling_y_velocity).wrapping_sub(MAX_FALL_SPEED)
            & SIGN
            != 0
        {
            let low = session_slot!(runtime, Crawler[crawler].falling_y_subvelocity);
            let high = session_slot!(runtime, Crawler[crawler].falling_y_velocity);
            let next = ((u32::from(high) << 16) | u32::from(low)).wrapping_add(FALL_ACCELERATION);
            session_slot!(runtime, Crawler[crawler]; falling_y_subvelocity = next as u16,
                falling_y_velocity = (next >> 16) as u16);
        }
        if session_slot!(runtime, Crawler[crawler].falling_y_subvelocity) == 0
            && session_slot!(runtime, Crawler[crawler].falling_y_velocity) == 0
        {
            runtime.edit_actor(key, Native, |actor| {
                actor.pre_instruction = function::CRAWLING_VERTICALLY
            });
        }
    }
}
