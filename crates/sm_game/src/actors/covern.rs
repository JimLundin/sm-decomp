//! Covern, the Wrecked Ship ghost, which materialises near Samus once she has
//! moved one way or stood still long enough, hovers, and fades away again
//! (bank $A8:99AC-$A8:9F4F). Was `wrecked_ship_ghost`.
use crate::actors::access::{AccessMode::Native, ActorKey as Key};
use crate::actors::host::ActorLease;
use crate::actors::slot_view::{SlotState, slot_view};

slot_view! {
    /// Covern's extended words: its hover centre and velocity (whole over sub
    /// words), Samus's previous X and Y movement directions and positions, the
    /// lock-on zone, and the lock-on and directed movement timers. Its function,
    /// function timer and palette delay index are its actor words ([`lane`]).
    Covern, CovernEdit {
        record: [],
        extended: [
            hover_centre_y = var_00, hover_y_subvelocity = var_01, hover_y_velocity = var_02,
            previous_x_direction = var_05, previous_samus_x = var_06, zone_left = var_07,
            zone_right = var_08, previous_y_direction = var_09, previous_samus_y = var_0a,
            zone_top = var_0b, zone_bottom = var_0c, lock_on_timer = var_0d,
            directed_movement_timer = var_0e,
        ],
    }
}

/// A lease the Coverns run on: its slots are the Session's.
pub trait Lease: ActorLease + SlotState {}
impl<T: ActorLease + SlotState + ?Sized> Lease for T {}

/// The Covern's actor words, by `variables` lane.
mod lane {
    pub const FUNCTION: usize = 0;
    pub const FUNCTION_TIMER: usize = 1;
    pub const PALETTE_DELAY_INDEX: usize = 2;
}

/// Bank $A8 Covern functions, stored in `function`.
pub mod function {
    pub const FADE_TO_WHITE: u16 = 0x9b42;
    pub const FADE_FROM_WHITE: u16 = 0x9bad;
    pub const DEMATERIALISING: u16 = 0x9c69;
    pub const MATERIALISED: u16 = 0x9c8a;
    pub const ASLEEP: u16 = 0x9d13;
    pub const AWAKE: u16 = 0x9d36;
}

/// $A8:9A8C: the Covern's instruction list.
const INSTRUCTION_LIST: u16 = 0x9a8c;
/// Covern constants ($A8:9A9C): the moves in one direction or frames standing
/// still that materialise it, the hover acceleration, the initial hover
/// velocity, and the frames asleep and materialised.
const DIRECTED_MOVES: u16 = 0x10;
const STATIONARY_FRAMES: u16 = 0x40;
const HOVER_ACCELERATION: u32 = 0x1800;
const INITIAL_HOVER_VELOCITY: u16 = 1;
const SLEEP_FRAMES: u16 = 120;
const MATERIALISED_FRAMES: u16 = 120;
/// The frames before the Covern first wakes.
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
/// $A8:99AC: the Covern's palette.
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

fn selected(runtime: &impl Lease) -> Key {
    Key::from_legacy(runtime.session().actors.controls.selected)
}
/// Samus's Y (`vertical`) or X position.
fn samus_position(runtime: &impl Lease, vertical: bool) -> u16 {
    let player = &runtime.session().player;
    if vertical { player.y } else { player.x }
}
fn palette_start(runtime: &impl Lease, key: Key) -> u16 {
    runtime
        .resolve_actor(key, Native)
        .palette
        .swap_bytes()
        .wrapping_mul(16)
        .wrapping_add(256)
}
fn set_target(runtime: &mut impl Lease, key: Key, colours: impl IntoIterator<Item = u16>) {
    let mut at = palette_start(runtime, key);
    for colour in colours {
        runtime.session_mut().presentation.visual_mut().targets[usize::from(at) / 2] = colour;
        at = at.wrapping_add(2);
    }
}
/// The Covern's 16.16 hover velocity.
fn hover_velocity(runtime: &impl Lease, covern: u16) -> u32 {
    let view = Covern::of(runtime, covern);
    (u32::from(view.hover_y_velocity()) << 16) | u32::from(view.hover_y_subvelocity())
}
/// Initialisation AI - Covern ($A8:9AEE).
pub fn initialize(runtime: &mut impl Lease) {
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
/// Process Covern palette transition delay ($A8:9C31): flicker visible.
fn process_palette_delay(runtime: &mut impl Lease, key: Key) {
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
/// Advance Covern palette transition ($A8:9E88): step each colour channel one
/// towards its target; the number of channels changed.
fn advance_palette_transition(runtime: &mut impl Lease) -> u16 {
    if runtime.session().room.entry.enemy_interlock != 0 {
        return 0;
    }
    let mut at = palette_start(runtime, selected(runtime));
    let mut changed = 0u16;
    for _ in 0..COLOURS {
        let index = usize::from(at) / 2;
        let visual = runtime.session_mut().presentation.visual_mut();
        let target = visual.targets[index];
        let colour = &mut visual.colors[index];
        if target != *colour {
            for shift in [0, 5, 10] {
                let mask = FULL << shift;
                let goal = target & mask;
                let value = *colour & mask;
                if value != goal {
                    let value = if (value as i16).wrapping_sub(goal as i16) >= 0 {
                        value.wrapping_sub(1 << shift)
                    } else {
                        value.wrapping_add(1 << shift)
                    };
                    *colour = value | (*colour & !mask);
                    changed = changed.wrapping_add(1);
                }
            }
        }
        at = at.wrapping_add(2);
    }
    changed
}
/// Covern function - materialising - fade to white ($A8:9B42).
fn fade_to_white(runtime: &mut impl Lease, key: Key) {
    process_palette_delay(runtime, key);
    let key = selected(runtime);
    let mut at = palette_start(runtime, key);
    let mut complete = true;
    for _ in 0..COLOURS {
        let colour =
            &mut runtime.session_mut().presentation.visual_mut().colors[usize::from(at) / 2];
        if *colour & FULL != FULL {
            *colour = colour.wrapping_add(ONE_EACH);
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
/// Covern function - materialising - fade from white ($A8:9BAD): once faded,
/// materialise and set up the hover and the lock-on zone.
fn fade_from_white(runtime: &mut impl Lease) {
    let changed = advance_palette_transition(runtime);
    let key = selected(runtime);
    process_palette_delay(runtime, key);
    let key = selected(runtime);
    if runtime.resolve_actor(key, Native).variables[lane::FUNCTION_TIMER] | changed != 0 {
        return;
    }
    let covern = key.byte_index();
    let y = runtime.edit_actor(key, Native, |actor| {
        actor.properties &= MATERIALISED_KEEP;
        actor.variables[lane::FUNCTION] = function::MATERIALISED;
        actor.y.whole
    });
    *CovernEdit::of(runtime, covern).hover_centre_y() = y;
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[lane::FUNCTION_TIMER] = 1;
        actor.variables[lane::PALETTE_DELAY_INDEX] = 2;
    });
    {
        let mut edit = CovernEdit::of(runtime, covern);
        *edit.hover_y_subvelocity() = 0;
        *edit.hover_y_velocity() = INITIAL_HOVER_VELOCITY;
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[lane::FUNCTION_TIMER] = MATERIALISED_FRAMES
    });
    *CovernEdit::of(runtime, covern).previous_x_direction() = X_DIRECTION_STEP;
    let x = runtime.player_x();
    {
        let mut edit = CovernEdit::of(runtime, covern);
        *edit.previous_samus_x() = x;
        *edit.zone_left() = x;
        *edit.zone_right() = x;
        *edit.previous_y_direction() = Y_DIRECTION_STEP;
    }
    let y = runtime.lease_native_word(
        SAMUS_Y_INDEXED + usize::from(runtime.session().actors.controls.selected),
    );
    let mut edit = CovernEdit::of(runtime, covern);
    *edit.previous_samus_y() = y;
    *edit.zone_top() = y;
    *edit.zone_bottom() = y;
    *edit.lock_on_timer() = STATIONARY_FRAMES;
    *edit.directed_movement_timer() = DIRECTED_MOVES;
}
/// Covern function - materialised ($A8:9C8A): hover about the centre until
/// the timer runs out, then dematerialise.
fn materialised(runtime: &mut impl Lease, key: Key) {
    let covern = key.byte_index();
    let speed = hover_velocity(runtime, covern);
    runtime.edit_actor(key, Native, |actor| actor.y.advance(speed as i32));
    let above = (runtime.resolve_actor(key, Native).y.whole as i16)
        .wrapping_sub(Covern::of(runtime, covern).hover_centre_y() as i16)
        < 0;
    let speed = hover_velocity(runtime, covern).wrapping_add(if above {
        HOVER_ACCELERATION
    } else {
        HOVER_ACCELERATION.wrapping_neg()
    });
    {
        let mut edit = CovernEdit::of(runtime, covern);
        *edit.hover_y_subvelocity() = speed as u16;
        *edit.hover_y_velocity() = (speed >> 16) as u16;
    }
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
/// long enough, for the Covern to materialise.
fn ready_to_materialise(runtime: &mut impl Lease, key: Key) -> bool {
    let covern = key.byte_index();
    let outside = (runtime.player_x() as i16)
        .wrapping_sub(Covern::of(runtime, covern).zone_left() as i16)
        < 0
        || (runtime.player_x() as i16)
            .wrapping_sub(Covern::of(runtime, covern).zone_right() as i16)
            >= 0
        || {
            let y =
                runtime.lease_native_word(SAMUS_Y_INDEXED + usize::from(key.byte_index())) as i16;
            let zone = Covern::of(runtime, covern);
            y.wrapping_sub(zone.zone_top() as i16) < 0
                || y.wrapping_sub(zone.zone_bottom() as i16) >= 0
        };
    let mut edit = CovernEdit::of(runtime, covern);
    if !outside {
        let timer = edit.lock_on_timer();
        *timer = timer.wrapping_sub(1);
        let finished = *timer == 0;
        if finished {
            *edit.previous_x_direction() = X_DIRECTION_STEP;
            *edit.previous_y_direction() = Y_DIRECTION_STEP;
        }
        return finished;
    }
    *edit.lock_on_timer() = STATIONARY_FRAMES;
    drop(edit);
    // Only inspect vertical movement if the horizontal direction still matches.
    for vertical in [false, true] {
        let (position, step) = if vertical {
            (samus_position(runtime, true), Y_DIRECTION_STEP)
        } else {
            (samus_position(runtime, false), X_DIRECTION_STEP)
        };
        let view = Covern::of(runtime, covern);
        let previous = if vertical {
            view.previous_samus_y()
        } else {
            view.previous_samus_x()
        };
        let delta = (position as i16).wrapping_sub(previous as i16);
        let value = if delta < 0 {
            0
        } else if delta == 0 {
            step
        } else {
            2 * step
        };
        let mut edit = CovernEdit::of(runtime, covern);
        let direction = if vertical {
            edit.previous_y_direction()
        } else {
            edit.previous_x_direction()
        };
        if value != *direction {
            *direction = value;
            *edit.directed_movement_timer() = DIRECTED_MOVES;
            return false;
        }
    }
    let mut edit = CovernEdit::of(runtime, covern);
    let timer = edit.directed_movement_timer();
    *timer = timer.wrapping_sub(1);
    *timer == 0
}
/// Covern function - dematerialised - awake ($A8:9D36): follow Samus's zone
/// and materialise near her when ready.
fn awake(runtime: &mut impl Lease, key: Key) {
    crate::probe::hit("covern.awake");
    let covern = key.byte_index();
    if ready_to_materialise(runtime, key) {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[lane::FUNCTION] = function::FADE_TO_WHITE
        });
        let index = {
            let mut edit = CovernEdit::of(runtime, covern);
            *edit.lock_on_timer() = STATIONARY_FRAMES;
            *edit.directed_movement_timer() = DIRECTED_MOVES;
            usize::from(
                edit.previous_y_direction()
                    .wrapping_add(*edit.previous_x_direction())
                    >> 1,
            )
        };
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
            let value = samus_position(runtime, vertical);
            let next = value.wrapping_add(1);
            let mut edit = CovernEdit::of(runtime, covern);
            if vertical {
                *edit.previous_samus_y() = value;
                *edit.zone_top() = value.wrapping_sub(1);
                *edit.zone_bottom() = next;
            } else {
                *edit.previous_samus_x() = value;
                *edit.zone_left() = value.wrapping_sub(1);
                *edit.zone_right() = next;
            }
        }
    }
}
/// Main AI - coven ($A8:9B3C): run its function. Was `advance`.
pub fn main_ai(runtime: &mut impl Lease) {
    crate::probe::hit("covern.main_ai");
    let key = selected(runtime);
    match runtime.resolve_actor(key, Native).variables[lane::FUNCTION] {
        function::FADE_TO_WHITE => fade_to_white(runtime, key),
        function::FADE_FROM_WHITE => fade_from_white(runtime),
        // Covern function - dematerialising ($A8:9C69).
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
        // Covern function - dematerialised - asleep ($A8:9D13).
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
