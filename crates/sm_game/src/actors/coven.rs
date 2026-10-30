//! Coven, the Wrecked Ship ghost, which materialises near Samus once she has
//! moved one way or stood still long enough, hovers, and fades away again
//! (bank $A8:99AC-$A8:9F4F). Was `wrecked_ship_ghost`.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::host::{ActorLease, LeaseActors};
use crate::actors::layouts::part_view;
use crate::enemy_projectiles::host::Effects as _;
use crate::ram::{word_arrays as A, words as W};
use crate::session_slot;

part_view! {
    /// Coven's view of its slot and extended RAM.
    Coven: EnemyLayout40 {
        words: [function, function_timer, palette_delay_index, var_D, var_E, var_F, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [
            hover_centre_y, hover_y_subvelocity, hover_y_velocity, var_03, var_04,
            previous_x_direction, previous_samus_x, zone_left, zone_right, previous_y_direction,
            previous_samus_y, zone_top, zone_bottom, lock_on_timer, directed_movement_timer, var_0F,
            var_10, var_11, var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19, var_1A,
            var_1B, var_1C, var_1D, var_1E, var_1F,
        ],
    }
}

/// The coven's actor words, by `variables` lane.
mod lane {
    pub const FUNCTION: usize = 0;
    pub const FUNCTION_TIMER: usize = 1;
    pub const PALETTE_DELAY_INDEX: usize = 2;
}

/// Bank $A8 coven functions, stored in `function`.
pub mod function {
    pub const FADE_TO_WHITE: u16 = 0x9b42;
    pub const FADE_FROM_WHITE: u16 = 0x9bad;
    pub const DEMATERIALISING: u16 = 0x9c69;
    pub const MATERIALISED: u16 = 0x9c8a;
    pub const ASLEEP: u16 = 0x9d13;
    pub const AWAKE: u16 = 0x9d36;
}

/// $A8:9A8C: the coven's instruction list.
const INSTRUCTION_LIST: u16 = 0x9a8c;
/// Coven constants ($A8:9A9C): the moves in one direction or frames standing
/// still that materialise it, the hover acceleration, the initial hover
/// velocity, and the frames asleep and materialised.
const DIRECTED_MOVES: u16 = 0x10;
const STATIONARY_FRAMES: u16 = 0x40;
const HOVER_ACCELERATION: u32 = 0x1800;
const INITIAL_HOVER_VELOCITY: u16 = 1;
const SLEEP_FRAMES: u16 = 120;
const MATERIALISED_FRAMES: u16 = 120;
/// The frames before the coven first wakes.
const INITIAL_SLEEP: u16 = 280;
/// $A8:9AA8: the X and Y offsets from Samus to materialise at, by the
/// direction she has been moving (Y direction times 3 plus X direction).
const OFFSETS: [u16; 18] = [
    0xffc0, 0xffc0, 0, 0xffc0, 0x40, 0, 0xffc0, 0, 0, 0, 0x40, 0, 0xffc0, 0x40, 0, 0x40, 0x40, 0x40,
];
/// Movement direction values: less, none, greater; Y directions step 12.
const X_DIRECTION_STEP: u16 = 4;
const Y_DIRECTION_STEP: u16 = 12;
/// $A8:9ACC: the palette transition delays, ended by FFFFh.
const PALETTE_DELAYS: [u16; 17] = [1, 8, 1, 8, 1, 7, 1, 7, 2, 6, 2, 6, 3, 5, 3, 5, 0xffff];
/// $A8:99AC: the coven's palette.
const PALETTE: [u16; 16] = [
    0x3800, 0x57ff, 0x42f7, 0x929, 0xa5, 0x4f5a, 0x36b5, 0x2610, 0x1dce, 0x1df, 0x1f, 0x18, 0xa,
    0x6b9, 0xea, 0x45,
];
const COLOURS: usize = 16;
const WHITE: u16 = 0x7fff;
/// One step of red, green and blue.
const ONE_EACH: u16 = 0x421;
const FULL: u16 = 31;
/// Properties: process instructions, intangible and invisible ($2500);
/// bit 10 intangible and bit 8 invisible.
const INITIAL_PROPERTIES: u16 = 0x2500;
const INTANGIBLE: u16 = 0x400;
const INVISIBLE: u16 = 0x100;
const MATERIALISED_KEEP: u16 = 0xfaff;
/// $7E:0AF6 and $7E:0AFA: Samus's X and Y position, read here with the
/// enemy's index added, as the original's buggy reads did.
const SAMUS_X_INDEXED: usize = 0xaf6;
const SAMUS_Y_INDEXED: usize = 0xafa;

fn selected(runtime: &impl ActorLease) -> Key {
    Key::from_legacy(runtime.word(W::cur_enemy_index))
}
fn palette_start(runtime: &impl ActorLease, key: Key) -> u16 {
    runtime
        .resolve_actor(key, Native)
        .palette
        .swap_bytes()
        .wrapping_mul(16)
        .wrapping_add(256)
}
fn set_target(runtime: &mut impl ActorLease, key: Key, colours: impl IntoIterator<Item = u16>) {
    let mut at = palette_start(runtime, key);
    for colour in colours {
        runtime.write_slice_le(A::target_palettes + usize::from(at), colour);
        at = at.wrapping_add(2);
    }
}
/// Initialisation AI - coven ($A8:9AEE).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = selected(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.properties |= INITIAL_PROPERTIES;
        actor.instruction_delay = 1;
        actor.timer = 0;
        actor.instruction = INSTRUCTION_LIST;
        actor.variables[lane::FUNCTION] = function::ASLEEP;
        actor.variables[lane::FUNCTION_TIMER] = INITIAL_SLEEP;
    });
    // Initialization and disappearance intentionally include the seventeenth color.
    set_target(runtime, key, [0; COLOURS + 1]);
}
/// Process coven palette transition delay ($A8:9C31): flicker visible.
fn process_palette_delay(runtime: &mut impl ActorLease, key: Key) {
    runtime.edit_actor(key, Native, |actor| {
        let mut visible = actor.variables[lane::FUNCTION_TIMER] == 0;
        if !visible {
            actor.variables[lane::FUNCTION_TIMER] =
                actor.variables[lane::FUNCTION_TIMER].wrapping_sub(1);
            if actor.variables[lane::FUNCTION_TIMER] != 0 {
                return;
            }
            let phase = actor.variables[lane::PALETTE_DELAY_INDEX];
            let delay = PALETTE_DELAYS[usize::from(phase >> 1)];
            if delay & 0x8000 != 0 {
                actor.variables[lane::FUNCTION_TIMER] = 0;
                actor.variables[lane::PALETTE_DELAY_INDEX] = 0;
                return;
            }
            actor.variables[lane::FUNCTION_TIMER] = delay;
            actor.variables[lane::PALETTE_DELAY_INDEX] = phase.wrapping_add(2);
            visible = phase & 2 == 0;
        }
        if visible {
            actor.properties &= !INVISIBLE;
        }
    });
}
/// Advance coven palette transition ($A8:9E88): step each colour channel one
/// towards its target; the number of channels changed.
fn advance_palette_transition(runtime: &mut impl ActorLease) -> u16 {
    if runtime.word(W::door_transition_flag_enemies) != 0 {
        return 0;
    }
    let mut at = palette_start(runtime, selected(runtime));
    let mut changed = 0u16;
    for _ in 0..COLOURS {
        let source = A::palette_buffer + usize::from(at);
        let target = A::target_palettes + usize::from(at);
        if runtime.word(target) != runtime.word(source) {
            for shift in [0, 5, 10] {
                let mask = FULL << shift;
                let goal = runtime.word(target) & mask;
                let value = runtime.word(source) & mask;
                if value != goal {
                    let value = if (value as i16).wrapping_sub(goal as i16) >= 0 {
                        value.wrapping_sub(1 << shift)
                    } else {
                        value.wrapping_add(1 << shift)
                    };
                    runtime.write_slice_le(source, value | (runtime.word(source) & !mask));
                    changed = changed.wrapping_add(1);
                }
            }
        }
        at = at.wrapping_add(2);
    }
    changed
}
/// Coven function - materialising - fade to white ($A8:9B42).
fn fade_to_white(runtime: &mut impl ActorLease, key: Key) {
    process_palette_delay(runtime, key);
    let key = selected(runtime);
    let mut at = palette_start(runtime, key);
    let mut complete = true;
    for _ in 0..COLOURS {
        let source = A::palette_buffer + usize::from(at);
        if runtime.word(source) & FULL != FULL {
            runtime.write_slice_le(source, runtime.word(source).wrapping_add(ONE_EACH));
            complete = false;
        }
        at = at.wrapping_add(2);
    }
    if complete {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[lane::FUNCTION] = function::FADE_FROM_WHITE
        });
        set_target(runtime, key, PALETTE);
    }
}
/// Coven function - materialising - fade from white ($A8:9BAD): once faded,
/// materialise and set up the hover and the lock-on zone.
fn fade_from_white(runtime: &mut impl ActorLease) {
    let changed = advance_palette_transition(runtime);
    let key = selected(runtime);
    process_palette_delay(runtime, key);
    let key = selected(runtime);
    if runtime.resolve_actor(key, Native).variables[lane::FUNCTION_TIMER] | changed != 0 {
        return;
    }
    let coven = i32::from(key.byte_index());
    let y = runtime.edit_actor(key, Native, |actor| {
        actor.properties &= MATERIALISED_KEEP;
        actor.variables[lane::FUNCTION] = function::MATERIALISED;
        actor.y.whole
    });
    session_slot!(runtime, Coven[coven].hover_centre_y = y);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[lane::FUNCTION_TIMER] = 1;
        actor.variables[lane::PALETTE_DELAY_INDEX] = 2;
    });
    session_slot!(runtime, Coven[coven]; hover_y_subvelocity = 0,
        hover_y_velocity = INITIAL_HOVER_VELOCITY);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[lane::FUNCTION_TIMER] = MATERIALISED_FRAMES
    });
    session_slot!(
        runtime,
        Coven[coven].previous_x_direction = X_DIRECTION_STEP
    );
    let x = runtime.player_x();
    session_slot!(runtime, Coven[coven]; previous_samus_x = x, zone_left = x, zone_right = x);
    session_slot!(
        runtime,
        Coven[coven].previous_y_direction = Y_DIRECTION_STEP
    );
    let y =
        runtime.lease_native_word(SAMUS_Y_INDEXED + usize::from(runtime.word(W::cur_enemy_index)));
    session_slot!(runtime, Coven[coven]; previous_samus_y = y, zone_top = y, zone_bottom = y,
        lock_on_timer = STATIONARY_FRAMES, directed_movement_timer = DIRECTED_MOVES);
}
/// Coven function - materialised ($A8:9C8A): hover about the centre until
/// the timer runs out, then dematerialise.
fn materialised(runtime: &mut impl ActorLease, key: Key) {
    let coven = i32::from(key.byte_index());
    let speed = session_slot!(runtime, Coven[coven](hover_y_velocity, hover_y_subvelocity));
    runtime.edit_actor(key, Native, |actor| actor.y.advance(speed as i32));
    let above = (runtime.resolve_actor(key, Native).y.whole as i16)
        .wrapping_sub(session_slot!(runtime, Coven[coven].hover_centre_y) as i16)
        < 0;
    let low = session_slot!(runtime, Coven[coven].hover_y_subvelocity);
    let high = session_slot!(runtime, Coven[coven].hover_y_velocity);
    let speed = ((u32::from(high) << 16) | u32::from(low)).wrapping_add(if above {
        HOVER_ACCELERATION
    } else {
        HOVER_ACCELERATION.wrapping_neg()
    });
    session_slot!(runtime, Coven[coven]; hover_y_subvelocity = speed as u16,
        hover_y_velocity = (speed >> 16) as u16);
    let timer = runtime.edit_actor(key, Native, |actor| {
        actor.variables[lane::FUNCTION_TIMER] =
            actor.variables[lane::FUNCTION_TIMER].wrapping_sub(1);
        actor.variables[lane::FUNCTION_TIMER]
    });
    if timer == 0 {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[lane::FUNCTION] = function::DEMATERIALISING;
            actor.properties |= INTANGIBLE;
        });
        set_target(runtime, key, [WHITE; COLOURS + 1]);
    }
}
/// Whether Samus has stayed in the lock-on zone long enough, or moved one way
/// long enough, for the coven to materialise.
fn ready_to_materialise(runtime: &mut impl ActorLease, key: Key) -> bool {
    let coven = i32::from(key.byte_index());
    let outside = (runtime.player_x() as i16)
        .wrapping_sub(session_slot!(runtime, Coven[coven].zone_left) as i16)
        < 0
        || (runtime.player_x() as i16)
            .wrapping_sub(session_slot!(runtime, Coven[coven].zone_right) as i16)
            >= 0
        || {
            let y =
                runtime.lease_native_word(SAMUS_Y_INDEXED + usize::from(key.byte_index())) as i16;
            y.wrapping_sub(session_slot!(runtime, Coven[coven].zone_top) as i16) < 0
                || y.wrapping_sub(session_slot!(runtime, Coven[coven].zone_bottom) as i16) >= 0
        };
    if !outside {
        let timer = session_slot!(runtime, Coven[coven].lock_on_timer).wrapping_sub(1);
        session_slot!(runtime, Coven[coven].lock_on_timer = timer);
        if timer == 0 {
            session_slot!(
                runtime,
                Coven[coven].previous_x_direction = X_DIRECTION_STEP
            );
            session_slot!(
                runtime,
                Coven[coven].previous_y_direction = Y_DIRECTION_STEP
            );
        }
        return timer == 0;
    }
    session_slot!(runtime, Coven[coven].lock_on_timer = STATIONARY_FRAMES);
    // Only inspect vertical movement if the horizontal direction still matches.
    for vertical in [false, true] {
        let (position, step) = if vertical {
            (W::samus_y_pos, Y_DIRECTION_STEP)
        } else {
            (W::samus_x_pos, X_DIRECTION_STEP)
        };
        let previous = if vertical {
            session_slot!(runtime, Coven[coven].previous_samus_y)
        } else {
            session_slot!(runtime, Coven[coven].previous_samus_x)
        };
        let delta = (runtime.word(position) as i16).wrapping_sub(previous as i16);
        let value = if delta < 0 {
            0
        } else if delta == 0 {
            step
        } else {
            2 * step
        };
        let direction = if vertical {
            session_slot!(runtime, Coven[coven].previous_y_direction)
        } else {
            session_slot!(runtime, Coven[coven].previous_x_direction)
        };
        if value != direction {
            if vertical {
                session_slot!(runtime, Coven[coven].previous_y_direction = value);
            } else {
                session_slot!(runtime, Coven[coven].previous_x_direction = value);
            }
            session_slot!(
                runtime,
                Coven[coven].directed_movement_timer = DIRECTED_MOVES
            );
            return false;
        }
    }
    let timer = session_slot!(runtime, Coven[coven].directed_movement_timer).wrapping_sub(1);
    session_slot!(runtime, Coven[coven].directed_movement_timer = timer);
    timer == 0
}
/// Coven function - dematerialised - awake ($A8:9D36): follow Samus's zone
/// and materialise near her when ready.
fn awake(runtime: &mut impl ActorLease, key: Key) {
    let coven = i32::from(key.byte_index());
    if ready_to_materialise(runtime, key) {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[lane::FUNCTION] = function::FADE_TO_WHITE
        });
        session_slot!(runtime, Coven[coven]; lock_on_timer = STATIONARY_FRAMES,
            directed_movement_timer = DIRECTED_MOVES);
        let index = usize::from(
            session_slot!(runtime, Coven[coven].previous_y_direction)
                .wrapping_add(session_slot!(runtime, Coven[coven].previous_x_direction))
                >> 1,
        );
        let x = OFFSETS[index].wrapping_add(
            runtime.lease_native_word(SAMUS_X_INDEXED + usize::from(key.byte_index())),
        );
        runtime.edit_actor(key, Native, |actor| actor.x.whole = x);
        let y = OFFSETS[index + 1].wrapping_add(
            runtime.lease_native_word(SAMUS_Y_INDEXED + usize::from(key.byte_index())),
        );
        runtime.edit_actor(key, Native, |actor| actor.y.whole = y);
    } else {
        for vertical in [false, true] {
            let position = if vertical {
                W::samus_y_pos
            } else {
                W::samus_x_pos
            };
            let value = runtime.word(position);
            let next = runtime.word(position).wrapping_add(1);
            if vertical {
                session_slot!(runtime, Coven[coven]; previous_samus_y = value,
                    zone_top = value.wrapping_sub(1));
                session_slot!(runtime, Coven[coven].zone_bottom = next);
            } else {
                session_slot!(runtime, Coven[coven]; previous_samus_x = value,
                    zone_left = value.wrapping_sub(1));
                session_slot!(runtime, Coven[coven].zone_right = next);
            }
        }
    }
}
/// Main AI - coven ($A8:9B3C): run its function. Was `advance`.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut lease = game.lease();
    let runtime = &mut lease;
    let key = selected(runtime);
    match runtime.resolve_actor(key, Native).variables[lane::FUNCTION] {
        function::FADE_TO_WHITE => fade_to_white(runtime, key),
        function::FADE_FROM_WHITE => fade_from_white(runtime),
        // Coven function - dematerialising ($A8:9C69).
        function::DEMATERIALISING => {
            if advance_palette_transition(runtime) == 0 {
                let key = selected(runtime);
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[lane::FUNCTION] = function::ASLEEP;
                    actor.properties |= INVISIBLE;
                    actor.variables[lane::FUNCTION_TIMER] = SLEEP_FRAMES;
                });
            }
        }
        function::MATERIALISED => materialised(runtime, key),
        // Coven function - dematerialised - asleep ($A8:9D13).
        function::ASLEEP => {
            runtime.edit_actor(key, Native, |actor| {
                if actor.variables[lane::FUNCTION_TIMER] != 0 {
                    actor.variables[lane::FUNCTION_TIMER] =
                        actor.variables[lane::FUNCTION_TIMER].wrapping_sub(1);
                }
                if actor.variables[lane::FUNCTION_TIMER] == 0 {
                    actor.variables[lane::FUNCTION_TIMER] = 1;
                    actor.variables[lane::PALETTE_DELAY_INDEX] = 2;
                    actor.variables[lane::FUNCTION] = function::AWAKE;
                }
            });
        }
        function::AWAKE => awake(runtime, key),
        _ => runtime.unreachable(),
    }
}
