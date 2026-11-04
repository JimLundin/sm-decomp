//! Tatori, Maridia's sleeping turtle, which wakes to hover and carry Samus, and the
//! mini-tatoris that crawl beside it (bank $A2:8B60-$A2:998C). Was `maridia_turtles`.
use crate::actors::access::{
    AccessMode::{LittleEndian as Le, Native},
    ActorKey as Key,
};
use crate::actors::api::{self, DeathHandling, MovementResult::Blocked, ShotHandling, SlopePolicy};
use crate::actors::combat;
use crate::actors::host::{ActorLease, abs16};
use crate::actors::slot_view::{SlotState, slot_view};
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content;

slot_view! {
    /// A tatori's slot: its function, its timer, its acceleration (sub and whole
    /// words), its X subvelocity (under the velocity word in its fifth actor
    /// word), its Y velocity (whole over sub), its hide timer and the height of
    /// a mini-tatori it carries. The first word of a mini-tatori is its function,
    /// its second its tatori's slot, then its spawn X and top Y, then its X
    /// velocity.
    Tatori, TatoriEdit {
        record: [function = var_a, parent = var_b],
        extended: [
            timer = var_00, acceleration_sub = var_01, acceleration = var_02,
            x_subvelocity = var_03, y_velocity = var_04, hide_timer = var_05,
            child_height = var_06, y_subvelocity = var_07,
        ],
    }
}

/// A lease the tatoris run on: its slots are the Session's.
pub trait Lease: ActorLease + SlotState {}
impl<T: ActorLease + SlotState + ?Sized> Lease for T {}

const INITIAL: u16 = 0x8dd8;
const IDLE: u16 = 0x8e09; // An RTS, while its instruction list runs.
const ASLEEP: u16 = 0x8e0a;
const LEAVE_SHELL: u16 = 0x8ee0;
const ENTER_SHELL: u16 = 0x8f3f;
const RISE_TO_HOVER: u16 = 0x8f8d;
const HOVERING: u16 = 0x8feb;
const RISING_TO_PEAK: u16 = 0x9083;
const HOVERING_AT_PEAK: u16 = 0x90cc;
const FALLING: u16 = 0x90e1;
const CRAWLING: u16 = 0x9142;
const HIDING_CARRYING: u16 = 0x916e;
const HIDING: u16 = 0x9198;
const SPINNING_UNSTOPPABLE: u16 = 0x91f8;
const SPINNING_STOPPABLE: u16 = 0x9239;
const CRAWLING_CARRYING: u16 = 0x925e;
const CRAWLING_LEFT: u16 = 0x8b80;
const CRAWLING_RIGHT: u16 = 0x8c72;
/// Samus's extra displacement, X ($7E:0B58) or Y ($7E:0B5C).
#[derive(Clone, Copy)]
enum Displacement {
    X,
    Y,
}
/// $7E:05B5: the NMI frame counter's low byte.
const NMI_FRAME_COUNTER: usize = 0x5b5;
/// $A2:8E80: the sleeping tatori shell's height either side of its centre, right then left.
const SHELL_HEIGHT: [i16; 48] = [
    -16, -16, -16, -16, -15, -15, -15, -15, -15, -14, -13, -13, -12, -11, -10, -9, -8, -7, -6, -5,
    -4, -4, 0, 0, -16, -16, -16, -15, -15, -15, -14, -13, -12, -11, -10, -9, -8, -7, -6, -5, -4,
    -3, -3, -2, 0, 0, 0, 0,
];

fn set_list(runtime: &mut impl Lease, key: Key, list: u16) {
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = list;
        actor.instruction_delay = 1;
    });
}
fn set_function(runtime: &mut impl Lease, key: Key, function: u16) {
    runtime.edit_actor(key, Native, |actor| actor.variables[0] = function);
}
fn forward(runtime: &impl Lease, key: Key) -> bool {
    runtime.resolve_actor(key, Native).variables[4] & 0x8000 == 0
}
fn samus_right(runtime: &impl Lease, key: Key) -> bool {
    let x = runtime.resolve_actor(key, Native).x.whole;
    (x.wrapping_sub(runtime.session().player.x) as i16) < 0
}
fn crawl_again(runtime: &mut impl Lease, key: Key) {
    let list = [CRAWLING_LEFT, CRAWLING_RIGHT][usize::from(forward(runtime, key))];
    set_list(runtime, key, list);
    set_function(runtime, key, CRAWLING);
}
fn displace(runtime: &mut impl Lease, axis: Displacement, delta: u16) {
    let contact = &mut runtime.session_mut().player.contact;
    let whole = match axis {
        Displacement::X => &mut contact.external_x.whole,
        Displacement::Y => &mut contact.external_y.whole,
    };
    *whole = whole.wrapping_add(delta);
}
fn horizontal(runtime: &mut impl Lease, key: Key, speed: i32) -> bool {
    api::move_horizontal(runtime, key, speed, SlopePolicy::Ignore) == Blocked
}
fn vertical(runtime: &mut impl Lease, key: Key, speed: i32) -> bool {
    api::move_vertical(runtime, key, speed) == Blocked
}
fn riding(runtime: &impl Lease) -> bool {
    api::touches_player_from_above(runtime, combat::current(runtime))
}
/// The velocity whole word in the fifth actor word over `x_subvelocity`.
fn x_velocity(runtime: &impl Lease, key: Key) -> u32 {
    let whole = runtime.resolve_actor(key, Native).variables[4];
    (u32::from(whole) << 16) | u32::from(Tatori::of(runtime, key.byte_index()).x_subvelocity())
}
fn set_x_velocity(runtime: &mut impl Lease, key: Key, velocity: u32) {
    *TatoriEdit::of(runtime, key.byte_index()).x_subvelocity() = velocity as u16;
    let whole = (velocity >> 16) as u16;
    runtime.edit_actor(key, Native, |actor| actor.variables[4] = whole);
}
/// The tatori's 16.16 acceleration.
fn acceleration(runtime: &impl Lease, slot: u16) -> u32 {
    let tatori = Tatori::of(runtime, slot);
    (u32::from(tatori.acceleration()) << 16) | u32::from(tatori.acceleration_sub())
}
/// Tatori / Samus collision detection ($A2:9315), against the tatori in slot 0.
fn collide(runtime: &mut impl Lease) {
    let (left, right, top, bottom) = {
        let actor = runtime.resolve_actor(Key::from_legacy(0), Native);
        (
            actor.x.whole.wrapping_sub(actor.radius.x).wrapping_sub(8),
            actor.x.whole.wrapping_add(actor.radius.x).wrapping_add(8),
            actor.y.whole.wrapping_sub(actor.radius.y).wrapping_add(4),
            actor.y.whole.wrapping_add(actor.radius.y).wrapping_sub(4),
        )
    };
    let player = &runtime.session().player;
    let x = i32::from(player.x);
    let width = i32::from(player.contact.x_radius);
    let y = i32::from(player.y);
    let height = i32::from(player.contact.radius);
    if (x - width - 1 - i32::from(right)) & 0x8000 != 0
        && (width + x - i32::from(left)) & 0x8000 == 0
        && (y - height + 1 - i32::from(bottom)) & 0x8000 != 0
        && (height + y - i32::from(top)) & 0x8000 == 0
        && runtime.session().player.invincibility == 0
    {
        combat::contact_reaction(runtime, DeathHandling::Finalize);
    }
}
/// One fewer mini-tatori for its tatori to wait on.
fn detach_child(runtime: &mut impl Lease) {
    let key = combat::current(runtime);
    let parent = Key::from_legacy(runtime.resolve_actor(key, Native).variables[1]);
    runtime.edit_actor(parent, Native, |actor| {
        if actor.pre_instruction != 0 {
            actor.pre_instruction = actor.pre_instruction.wrapping_sub(1);
        }
    });
}
/// Initialisation AI - tatori ($A2:8D6C) or mini-tatori ($A2:8D9D).
pub fn initialize(child: bool, lease: &mut impl Lease) {
    let key = combat::current(lease);
    lease.edit_actor(key, Le, |actor| {
        actor.instruction_delay = 1;
        actor.timer = 0;
        if child {
            actor.variables[2] = actor.x.whole;
            actor.variables[3] = actor.y.whole.wrapping_sub(actor.radius.y);
            actor.variables[0] = CRAWLING;
            actor.variables[4] = actor.parameters[0];
            actor.instruction = if actor.parameters[0] as i16 >= 0 {
                CRAWLING_RIGHT
            } else {
                CRAWLING_LEFT
            };
        } else {
            actor.properties |= 0x2000;
            actor.spritemap = 0x804d;
            actor.radius.y = 0;
            actor.instruction = 0x8c44; // Asleep.
            actor.variables[0] = INITIAL;
            actor.pre_instruction = 1;
        }
    });
}
/// Main AI - tatori ($A2:8DD2) or mini-tatori ($A2:912E); false for an unknown function.
pub fn main_ai(child: bool, runtime: &mut impl crate::actors::species::TableLease) -> bool {
    let key = combat::current(runtime);
    if child {
        let parent = runtime.resolve_actor(key, Le).variables[1];
        *TatoriEdit::of(runtime, parent).child_height() = 0;
    }
    let function = runtime.resolve_actor(key, Le).variables[0];
    run_function(runtime, key.byte_index(), function)
}
fn run_function(runtime: &mut impl Lease, raw: u16, function: u16) -> bool {
    let key = Key::from_legacy(raw);
    let slot = raw;
    match function {
        // Hand its palette, tiles and slot to its four mini-tatoris.
        INITIAL => {
            const CHILDREN: [u16; 4] = [64, 128, 192, 256];
            for tiles in [false, true] {
                let actor = Tatori::of(runtime, slot).actor();
                let value = if tiles { actor.tile } else { actor.palette };
                for child in CHILDREN {
                    let mut edit = TatoriEdit::of(runtime, slot + child);
                    let actor = edit.actor();
                    *if tiles {
                        &mut actor.tile
                    } else {
                        &mut actor.palette
                    } = value;
                }
            }
            for child in CHILDREN {
                *TatoriEdit::of(runtime, slot + child).parent() = raw;
            }
            *TatoriEdit::of(runtime, slot).function() = ASLEEP;
        }
        ASLEEP if runtime.resolve_actor(key, Le).pre_instruction == 0 => {
            runtime.edit_actor(key, Le, |actor| {
                actor.variables[0] = LEAVE_SHELL;
                actor.properties &= !0x400;
            })
        }
        // Shape its shell under Samus.
        ASLEEP => {
            runtime.edit_actor(key, Le, |actor| actor.radius.y = 0);
            let x = runtime.resolve_actor(key, Le).x.whole;
            let delta = x.wrapping_sub(runtime.session().player.x);
            let magnitude = abs16(delta);
            if magnitude < 24 {
                let index = magnitude + if delta & 0x8000 != 0 { 24 } else { 0 };
                let live = combat::current(runtime);
                let radius = SHELL_HEIGHT[usize::from(index)].wrapping_neg() as u16;
                runtime.edit_actor(key, Le, |actor| {
                    actor.radius.y = radius;
                    actor.properties |= 0x8000;
                });
                if api::touches_player_from_above(runtime, live) {
                    let actor = runtime.resolve_actor(key, Le);
                    let top = actor.y.whole.wrapping_sub(actor.radius.y);
                    let player = &runtime.session().player;
                    let feet = player.contact.radius.wrapping_add(player.y);
                    if feet.wrapping_sub(top) as i16 >= 0 {
                        displace(runtime, Displacement::Y, top.wrapping_sub(feet));
                    }
                }
            }
        }
        LEAVE_SHELL => {
            collide(runtime);
            if runtime.lease_byte(NMI_FRAME_COUNTER) & 1 == 0 {
                if api::touches_player_from_above(runtime, key) {
                    displace(runtime, Displacement::X, u16::MAX);
                }
                runtime.edit_actor(key, Native, |actor| {
                    actor.y.whole = actor.y.whole.wrapping_sub(1);
                    actor.radius.y = 16;
                    let step = if actor.y.whole & 1 != 0 { u16::MAX } else { 1 };
                    actor.x.whole = actor.x.whole.wrapping_add(step);
                });
                if !horizontal(runtime, key, 1 << 16) {
                    set_list(runtime, key, 0x8c4a); // Facing left - leave shell.
                    set_function(runtime, key, IDLE);
                }
            }
        }
        ENTER_SHELL => {
            // Facing right or left - enter shell.
            let list = [0x8c1c, 0x8d00][usize::from(samus_right(runtime, key))];
            set_list(runtime, key, list);
            set_function(runtime, key, IDLE);
        }
        RISE_TO_HOVER => {
            collide(runtime);
            if !vertical(runtime, key, -1 << 16) {
                if api::touches_player_from_above(runtime, key) {
                    displace(runtime, Displacement::Y, u16::MAX);
                }
                let timer = Tatori::of(runtime, slot).timer().wrapping_sub(1);
                *TatoriEdit::of(runtime, slot).timer() = timer;
                if timer == 0 {
                    let index = if samus_right(runtime, key) { 2 } else { 0 };
                    let image = runtime.native_image();
                    let low = image.table_word(content::offset(0xa2_8d56), index);
                    *TatoriEdit::of(runtime, slot).acceleration_sub() = low;
                    let high = image.table_word(content::offset(0xa2_8d58), index);
                    *TatoriEdit::of(runtime, slot).acceleration() = high;
                    runtime.edit_actor(key, Native, |actor| actor.variables[4] = 0);
                    *TatoriEdit::of(runtime, slot).x_subvelocity() = 0;
                    set_function(runtime, key, HOVERING);
                }
            }
        }
        HOVERING => {
            collide(runtime);
            if horizontal(runtime, key, x_velocity(runtime, key) as i32) {
                let speed = (-(x_velocity(runtime, key) as i32)) as u32;
                set_x_velocity(runtime, key, speed);
                let accel = acceleration(runtime, slot).wrapping_neg();
                let mut edit = TatoriEdit::of(runtime, slot);
                *edit.acceleration_sub() = accel as u16;
                *edit.acceleration() = (accel >> 16) as u16;
                drop(edit);
                let feedback = &mut runtime.session_mut().feedback;
                feedback.earthquake_kind = 0;
                feedback.earthquake_timer = 16;
                runtime.queue_sound(2, 6, 0x1b);
            } else {
                // Handle Samus landing on hovering tatori ($A2:8F5F): carry her along.
                if api::touches_player_from_above(runtime, key) {
                    set_function(runtime, key, RISING_TO_PEAK);
                    let delta = (-(x_velocity(runtime, key) as i32)) as u32;
                    let external = &mut runtime.session_mut().player.contact.external_x;
                    let low = u32::from(external.fraction);
                    let high = u32::from(external.whole);
                    let moved = (low | (high << 16)).wrapping_add(delta);
                    external.fraction = moved as u16;
                    external.whole = (moved >> 16) as u16;
                    if (i32::from(external.whole) + 16) & 0x8000 != 0 {
                        external.whole = (-16i16) as u16;
                    }
                }
                let accel = acceleration(runtime, slot);
                let speed = x_velocity(runtime, key).wrapping_add(accel);
                set_x_velocity(runtime, key, speed);
                runtime.edit_actor(key, Native, |actor| {
                    let speed = actor.variables[4];
                    if (abs16(speed) as i16).wrapping_sub(3) >= 0 {
                        actor.variables[4] = if speed & 0x8000 != 0 {
                            (-3i16) as u16
                        } else {
                            3
                        };
                    }
                });
            }
        }
        RISING_TO_PEAK => {
            collide(runtime);
            let next =
                if (runtime.resolve_actor(key, Native).y.whole as i16).wrapping_sub(0x1e8) < 0 {
                    *TatoriEdit::of(runtime, slot).timer() = 30;
                    HOVERING_AT_PEAK
                } else if api::touches_player_from_above(runtime, key) {
                    runtime.edit_actor(key, Native, |actor| {
                        actor.y.whole = actor.y.whole.wrapping_sub(7)
                    });
                    displace(runtime, Displacement::Y, (-7i16) as u16);
                    return true;
                } else {
                    FALLING
                };
            set_function(runtime, key, next);
            let mut edit = TatoriEdit::of(runtime, slot);
            *edit.y_subvelocity() = 0;
            *edit.y_velocity() = 0;
        }
        HOVERING_AT_PEAK => {
            collide(runtime);
            let timer = Tatori::of(runtime, slot).timer().wrapping_sub(1);
            *TatoriEdit::of(runtime, slot).timer() = timer;
            if timer == 0 {
                set_function(runtime, key, FALLING);
            }
        }
        FALLING => {
            collide(runtime);
            // As the source does, the slot 0 tatori's fall speed caps every one.
            if (Tatori::of(runtime, 0).y_velocity() as i16).wrapping_sub(4) < 0 {
                let mut edit = TatoriEdit::of(runtime, slot);
                let speed = ((u32::from(*edit.y_velocity()) << 16)
                    | u32::from(*edit.y_subvelocity()))
                .wrapping_add(0x2000);
                *edit.y_subvelocity() = speed as u16;
                *edit.y_velocity() = (speed >> 16) as u16;
            }
            let speed = i32::from(Tatori::of(runtime, slot).y_velocity() as i16) << 16;
            if vertical(runtime, key, speed) {
                // Facing right or left - leave shell.
                let list = [0x8c4a, 0x8d28][usize::from(forward(runtime, key))];
                set_list(runtime, key, list);
                set_function(runtime, key, IDLE);
            }
        }
        CRAWLING if api::touches_player_from_above(runtime, key) => {
            set_function(runtime, key, HIDING_CARRYING);
            *TatoriEdit::of(runtime, slot).hide_timer() = 4;
            // Facing right or left - hiding.
            let list = [0x8c30, 0x8d14][usize::from(forward(runtime, key))];
            set_list(runtime, key, list);
        }
        CRAWLING => {}
        HIDING_CARRYING if api::touches_player_from_above(runtime, key) => {
            *TatoriEdit::of(runtime, slot).hide_timer() = 4;
        }
        HIDING_CARRYING => {
            let timer = Tatori::of(runtime, slot).hide_timer().wrapping_sub(1);
            *TatoriEdit::of(runtime, slot).hide_timer() = timer;
            if timer == 0 {
                set_function(runtime, key, HIDING);
                *TatoriEdit::of(runtime, slot).timer() = 60;
            }
        }
        HIDING if api::touches_player_from_above(runtime, key) => {
            set_function(runtime, key, SPINNING_UNSTOPPABLE);
            set_list(runtime, key, 0x8bd2); // Spinning.
            *TatoriEdit::of(runtime, slot).y_velocity() = 1;
            let facing = runtime.session().player.movement_and_facing.to_le_bytes()[0];
            let speed = if facing & 15 == 8 { 3 } else { (-3i16) as u16 };
            runtime.edit_actor(key, Native, |actor| actor.variables[4] = speed);
        }
        HIDING => {
            let timer = Tatori::of(runtime, slot).timer().wrapping_sub(1);
            *TatoriEdit::of(runtime, slot).timer() = timer;
            if timer == 0 {
                crawl_again(runtime, key);
            }
        }
        SPINNING_STOPPABLE if api::touches_player_from_above(runtime, key) => {
            crawl_again(runtime, key)
        }
        SPINNING_UNSTOPPABLE | SPINNING_STOPPABLE => {
            let speed = i32::from(runtime.resolve_actor(key, Native).variables[4] as i16) << 16;
            if horizontal(runtime, key, speed) {
                runtime.edit_actor(key, Native, |actor| {
                    actor.variables[4] = actor.variables[4].wrapping_neg()
                });
            } else {
                let speed = i32::from(Tatori::of(runtime, slot).y_velocity() as i16) << 16;
                vertical(runtime, key, speed);
            }
        }
        CRAWLING_CARRYING => {
            let key = combat::current(runtime);
            let parent = runtime.resolve_actor(key, Native).variables[1];
            let height = runtime.resolve_actor(key, Native).radius.y;
            *TatoriEdit::of(runtime, parent).child_height() = height;
            if !riding(runtime) {
                let current = combat::current(runtime);
                set_function(runtime, current, CRAWLING);
            }
        }
        IDLE => {}
        _ => return false,
    }
    true
}
/// Enemy touch - tatori ($A2:9281) or mini-tatori ($A2:929F).
pub fn touch(child: bool, runtime: &mut impl Lease) {
    crate::probe::hit("tatori.touch");
    let key = combat::current(runtime);
    if !child {
        // Awaken tatori ($A2:92FF) when Samus hits it outside its shell.
        if runtime.resolve_actor(key, Native).properties & 0x8000 == 0 {
            combat::contact_reaction(runtime, DeathHandling::Finalize);
            set_function(runtime, key, FALLING);
            *TatoriEdit::of(runtime, key.byte_index()).y_velocity() = 2;
        }
        return;
    }
    if runtime.resolve_actor(key, Native).variables[0] == CRAWLING_CARRYING {
        return;
    }
    // Turn round and step out from under Samus.
    runtime.edit_actor(key, Native, |actor| {
        let right = actor.variables[4] & 0x8000 != 0;
        actor.instruction = if right { CRAWLING_RIGHT } else { CRAWLING_LEFT };
        actor.variables[4] = if right { 1 } else { u16::MAX };
        actor.instruction_delay = 1;
    });
    let player = &runtime.session().player;
    let (samus, radius) = (player.x, player.contact.x_radius);
    let x = if !samus_right(runtime, key) {
        runtime
            .resolve_actor(key, Native)
            .radius
            .x
            .wrapping_add(radius)
            .wrapping_add(samus)
    } else {
        // Preserve the original packed subtraction, including its low-word borrow.
        let packed = (u32::from(samus).wrapping_sub(u32::from(radius)) << 16) | u32::from(samus);
        let width = u32::from(runtime.resolve_actor(key, Native).radius.x);
        (packed.wrapping_sub((width << 16) | u32::from(radius)) >> 16) as u16
    };
    runtime.edit_actor(key, Native, |actor| {
        actor.x.whole = x;
        actor.variables[0] = CRAWLING;
    });
    let speed = i32::from(runtime.resolve_actor(key, Native).variables[4] as i16) << 16;
    let current = combat::current(runtime);
    horizontal(runtime, current, speed);
    detach_child(runtime);
}
/// Enemy shot - mini-tatori ($A2:930F).
pub fn shot(lease: &mut impl Lease) {
    combat::shot_reaction(lease, ShotHandling::Finalize);
    detach_child(lease);
}
/// The tatori instructions ($A2:9381-$A2:94C7).
pub fn instruction(
    ea: u32,
    j: EnemyScript,
    runtime: &mut impl crate::actors::species::TableLease,
) -> EnemyScript {
    let key = combat::current(runtime);
    match ea {
        // Instruction - mini-tatori - crawl ($A2:9381): over the sleeping tatori's shell.
        0xa2_9381 => {
            let riding = riding(runtime);
            if riding {
                let speed = runtime.resolve_actor(key, Native).variables[4];
                displace(runtime, Displacement::X, speed);
            }
            let old_y = runtime.edit_actor(key, Native, |actor| {
                core::mem::replace(&mut actor.y.whole, actor.variables[3])
            });
            let speed = i32::from(runtime.resolve_actor(key, Native).variables[4] as i16) << 16;
            let current = combat::current(runtime);
            horizontal(runtime, current, speed);
            let parent = Key::from_legacy(runtime.resolve_actor(key, Native).variables[1]);
            if runtime.resolve_actor(parent, Native).variables[0] == ASLEEP {
                let delta = runtime
                    .resolve_actor(parent, Native)
                    .x
                    .whole
                    .wrapping_sub(runtime.resolve_actor(key, Native).x.whole);
                let magnitude = abs16(delta);
                let step = match magnitude < 24 {
                    true => {
                        SHELL_HEIGHT
                            [usize::from(magnitude + if delta & 0x8000 != 0 { 24 } else { 0 })]
                    }
                    false => 1,
                };
                let current = combat::current(runtime);
                vertical(runtime, current, i32::from(step) << 16);
                if riding {
                    let y = runtime
                        .resolve_actor(combat::current(runtime), Native)
                        .y
                        .whole;
                    displace(runtime, Displacement::Y, y.wrapping_sub(old_y));
                }
            }
        }
        // Instruction - mini-tatori - loop or turn around if moved too far ($A2:9412).
        0xa2_9412 => {
            runtime.edit_actor(key, Native, |actor| {
                let delta = actor.variables[2].wrapping_sub(actor.x.whole);
                if (abs16(delta) as i16).wrapping_sub(48) >= 0 {
                    actor.variables[4] = if delta & 0x8000 == 0 { 1 } else { u16::MAX };
                }
            });
            return EnemyScript::jump(if forward(runtime, key) {
                CRAWLING_RIGHT
            } else {
                CRAWLING_LEFT
            });
        }
        0xa2_9447 => set_function(runtime, key, ENTER_SHELL),
        0xa2_94c7 => set_function(runtime, key, SPINNING_STOPPABLE),
        // Instruction - tatori - rise to hover rightwards ($A2:9451) or leftwards ($A2:946B).
        0xa2_9451 | 0xa2_946b => {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[0] = RISE_TO_HOVER;
                actor.variables[4] = if ea == 0xa2_9451 { u16::MAX } else { 1 };
            });
            let current = combat::current(runtime).byte_index();
            *TatoriEdit::of(runtime, current).timer() = 16;
            return EnemyScript::jump(0x8c02); // Spinning.
        }
        // Instruction - mini-tatori - leave shell ($A2:9485), facing right or left.
        0xa2_9485 if riding(runtime) => {
            return EnemyScript::jump(if forward(runtime, key) {
                0x8d40
            } else {
                0x8c62
            });
        }
        0xa2_9485 => {}
        // Instruction - mini-tatori - left shell ($A2:94A1).
        0xa2_94a1 => {
            let function = if riding(runtime) {
                CRAWLING_CARRYING
            } else {
                CRAWLING
            };
            set_function(runtime, key, function);
            return EnemyScript::jump(if forward(runtime, key) {
                CRAWLING_RIGHT
            } else {
                CRAWLING_LEFT
            });
        }
        _ => unreachable!("unknown tatori instruction"),
    }
    j
}
