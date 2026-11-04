//! Yard, Maridia's shelled snail, which crawls around walls and ceilings, hides from
//! Samus, and is kicked or shot into the air (bank $A3:C8A6-$A3:DA9B). Was `maridia_snail`.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess as _, ActorKey as Key};
use crate::actors::api::{
    self, Axis, DeathHandling, MovementResult::Blocked, ShotHandling, SlopePolicy,
};
use crate::actors::combat;
use crate::actors::host::{ActorLease, LeaseActors};
use crate::actors::slot_view::{SlotState, slot_view};
use crate::controller_state::Word as Input;
use crate::enemy_projectiles::host::Effects as _;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::{self, table};
use crate::player_projectiles::{access::shot, host::Host as _};
use crate::session_access::SessionAccess as _;

/// A lease on a game whose slots are the Session's typed storage.
pub trait Lease: ActorLease + SlotState {}
impl<T: ActorLease + SlotState + ?Sized> Lease for T {}

slot_view! {
    /// A yard's extended RAM, with its airborne velocities. Its crawling X and Y velocities,
    /// facing, hiding instruction list and consecutive turn count are its first five actor
    /// words, and its movement function its pre-instruction word. `bounced` ($7E:8000 in
    /// slot 0, a word past the first extended block) flags the airborne yard hitting a wall.
    Yard, YardEdit {
        record: [],
        extended: [
            y_subvelocity = var_00, y_velocity = var_01, x_subvelocity = var_02,
            x_velocity = var_03, turn_disable_counter = var_04, turn_disabled = var_05,
            crawling_speed = var_06, direction = var_07, behaviour = var_08,
            bounced = var_20,
        ],
    }
}

/// Behaviours, in `behaviour`.
const IDLE_CRAWLING: u16 = 0;
const SAMUS_ON_TOP: u16 = 1;
const HIDING: u16 = 2;
const DROPPED: u16 = 3;
const KICKED: u16 = 4;
const SHOT: u16 = 5;
/// Movement functions: an RTL (hiding or idle) and airborne.
const NO_MOVEMENT: u16 = 0xcf5f;
const AIRBORNE: u16 = 0xd1b3;
/// $A3:CCA2: crawling speeds; $A3:CD42: direction data, 8 bytes each; $A3:CD82: crawling
/// velocity signs; $A3:CDC2: opposite directions; $A3:CDD2: crawling movement functions.
const SPEEDS: u32 = 0xa3_cca2;
const DIRECTIONS: u32 = 0xa3_cd42;
const SIGNS: u32 = 0xa3_cd82;
const OPPOSITES: u32 = 0xa3_cdc2;
const MOVEMENT_FUNCTIONS: u32 = 0xa3_cdd2;
/// Samus's pose X direction ($7E:0A1E), the movement and facing word's low byte.
fn pose_x_direction(runtime: &impl Lease) -> u8 {
    runtime.session().player.movement_and_facing.to_le_bytes()[0]
}

fn slot(key: Key) -> u16 {
    key.byte_index()
}
fn direction_data(runtime: &impl Lease, direction: u16, field: isize) -> u16 {
    let record = content::captured(runtime.native_image(), DIRECTIONS);
    record.table_word(isize::try_from(direction).unwrap() * 4 + field)
}
/// A crawling speed with its velocity sign pair from $A3:CD82: (0, 0) keeps it positive
/// and (FFFFh, 1) negates it, as speed XOR the first plus the second.
pub fn signed_speed(speed: u16, [flip, add]: [u16; 2]) -> u16 {
    (speed ^ flip).wrapping_add(add)
}
/// Set yard crawling velocities ($A3:CE27).
fn set_velocities(runtime: &mut impl Lease, key: Key, offset: u16) {
    let index = isize::try_from(offset >> 1).unwrap();
    let signs = [0, 1, 2, 3].map(|i| table(runtime, SIGNS, index + i));
    for axis in 0..2 {
        let parameter = runtime.resolve_actor(key, Native).parameters[0];
        let speed = table(runtime, SPEEDS, parameter as isize);
        let value = signed_speed(speed, [signs[axis * 2], signs[axis * 2 + 1]]);
        runtime.edit_actor(key, Native, |actor| actor.variables[axis] = value);
    }
}
/// Set the instruction list and hiding instruction list from the pair at `index`.
fn set_list_pair(runtime: &mut impl Lease, key: Key, address: u32, index: isize) {
    let instruction = table(runtime, address, index);
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
    let hiding = table(runtime, address, index + 1);
    runtime.edit_actor(key, Native, |actor| actor.variables[3] = hiding);
}
/// Set yard airborne instruction list ($A3:D2FA) for `behaviour`, and go airborne.
fn go_airborne(runtime: &mut impl Lease, key: Key, behaviour: u16, address: u32) {
    *YardEdit::of(runtime, slot(key)).behaviour() = behaviour;
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = AIRBORNE);
    let facing = runtime.resolve_actor(key, Native).variables[2];
    let index = if behaviour == DROPPED {
        2 * facing as isize
    } else {
        (facing.wrapping_mul(4) >> 1) as isize
    };
    set_list_pair(runtime, key, address, index);
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction_delay = 1;
        actor.timer = 0;
    });
}
/// Drop yard ($A3:D164).
fn drop(runtime: &mut impl Lease, key: Key) {
    if Yard::of(runtime, slot(key)).behaviour() != DROPPED {
        go_airborne(runtime, key, DROPPED, 0xa3_d1ab);
        let mut part = YardEdit::of(runtime, slot(key));
        *part.x_subvelocity() = 0;
        *part.x_velocity() = 0;
        *part.y_subvelocity() = 0;
        *part.y_velocity() = 0;
    }
}
fn orient(runtime: &mut impl Lease, key: Key, direction: u16, replace: bool) {
    let instruction = direction_data(runtime, direction, 0);
    runtime.edit_actor(key, Native, |actor| actor.instruction = instruction);
    let properties = direction_data(runtime, direction, 1);
    runtime.edit_actor(key, Native, |actor| {
        actor.properties = properties
            | if replace {
                actor.properties & !3
            } else {
                actor.properties
            }
    });
    for (index, field) in [(3, 2), (2, 3)] {
        let value = direction_data(runtime, direction, field);
        runtime.edit_actor(key, Native, |actor| actor.variables[index] = value);
    }
}
/// Turn yard around ($A3:D356), unless hiding or still.
fn turn_around(runtime: &mut impl Lease, key: Key) {
    if Yard::of(runtime, slot(key)).behaviour() == HIDING
        || runtime.resolve_actor(key, Native).pre_instruction == NO_MOVEMENT
    {
        return;
    }
    let direction = Yard::of(runtime, slot(key)).direction() as isize;
    let direction = table(runtime, OPPOSITES, direction);
    *YardEdit::of(runtime, slot(key)).direction() = direction;
    orient(runtime, key, direction, true);
    set_velocities(runtime, key, direction.wrapping_mul(8));
    // Set yard crawling movement function ($A3:CE57).
    let function = table(runtime, MOVEMENT_FUNCTIONS, direction as isize);
    runtime.edit_actor(key, Native, |actor| actor.pre_instruction = function);
    disable_turns(runtime, key, true);
}
/// Make yard face Samus horizontally ($A3:D33E).
fn face_samus(runtime: &mut impl Lease, key: Key) {
    let actor = runtime.resolve_actor(key, Native);
    let x = runtime.session().player.x;
    let turn = if actor.variables[2] != 0 {
        actor.x.whole >= x
    } else {
        actor.x.whole < x
    };
    if turn {
        turn_around(runtime, key);
    }
}
/// Check if player is directing towards enemy ($A3:D421).
fn pressing_towards(runtime: &impl Lease, key: Key) -> bool {
    let facing = runtime.resolve_actor(key, Native).variables[2] & 1;
    if (*Input::Joypad1Lastkeys.get(&runtime.session().control) & 0x300) >> 8 == 1 {
        facing == 0
    } else {
        facing != 0
    }
}
/// Handle turn transition disabling ($A3:D124): disable for 10h frames after a turn.
fn disable_turns(runtime: &mut impl Lease, key: Key, turned: bool) {
    let yard = slot(key);
    if turned {
        let mut part = YardEdit::of(runtime, yard);
        *part.turn_disabled() = 1;
        *part.turn_disable_counter() = 0;
    } else if Yard::of(runtime, yard)
        .turn_disable_counter()
        .wrapping_add(1)
        >= 16
    {
        *YardEdit::of(runtime, yard).turn_disabled() = 0;
    } else {
        let mut part = YardEdit::of(runtime, yard);
        let c = *part.turn_disable_counter();
        *part.turn_disable_counter() = c.wrapping_add(1);
    }
}
fn snap(runtime: &mut impl Lease, key: Key) {
    let on_slope = api::snap_to_slopes(runtime, key);
    disable_turns(runtime, key, on_slope);
}
/// Set enemy instruction list and disable turn transition ($A3:D14C).
fn turn_list(runtime: &mut impl Lease, key: Key, instruction: u16) {
    runtime.edit_actor(key, Native, |actor| {
        actor.instruction = instruction;
        actor.instruction_delay = 1;
    });
    disable_turns(runtime, key, true);
}
/// Move enemy ahead for ($A3:D0F8) or back from ($A3:D10D) the outside turn check.
fn shift(runtime: &mut impl Lease, key: Key, read: impl Fn(usize) -> u16, back: bool) {
    for axis in 0..2 {
        let actor = runtime.resolve_actor(key, Native);
        let position = [actor.x.whole, actor.y.whole][axis];
        let offset = read(axis * 2);
        let position = if back {
            position.wrapping_sub(offset)
        } else {
            position.wrapping_add(offset)
        };
        runtime.edit_actor(key, Native, |actor| {
            *[&mut actor.x.whole, &mut actor.y.whole][axis] = position
        });
    }
}
/// $14.$12 = ±[A] / 100h + `leading` * sgn([A]) ($A3:CF8F) for the crawling velocity on `axis`.
fn velocity(runtime: &impl Lease, key: Key, axis: usize, leading: i32) -> i32 {
    let speed = i32::from(runtime.resolve_actor(key, Native).variables[axis] as i16) << 8;
    speed
        + if speed < 0 {
            -(leading << 16)
        } else {
            leading << 16
        }
}
fn move_axis(runtime: &mut impl Lease, key: Key, axis: usize, amount: i32) -> bool {
    let result = match axis {
        0 => api::move_horizontal(runtime, key, amount, SlopePolicy::Follow),
        _ => api::move_vertical(runtime, key, amount),
    };
    result == Blocked
}
/// Yard movement function - crawling ($A3:CFA6-$A3:CFFC), by its turn data at
/// `turn`: Yard crawling movement - vertical ($A3:D002) or horizontal ($A3:D07E).
pub fn crawl<H: LeaseActors + ?Sized>(raw: u16, axis: Axis, turn: u32, game: &mut H) {
    let runtime = &mut game.lease();
    let key = Key::from_legacy(raw);
    let axis = usize::from(axis == Axis::Y);
    let turn = if axis == 1 && Yard::of(runtime, slot(key)).turn_disabled() != 0 {
        0xa3_cd22 + (turn - 0xa3_cce2) / 2
    } else {
        turn
    };
    let record = content::captured(runtime.native_image(), turn);
    shift(runtime, key, |offset| record.word(offset), false);
    let amount = velocity(runtime, key, axis, 1);
    let blocked = move_axis(runtime, key, axis, amount);
    shift(runtime, key, |offset| record.word(offset), true);
    if blocked {
        runtime.edit_actor(key, Native, |actor| actor.variables[4] = 0);
        if axis == 0 {
            snap(runtime, key);
        }
        let amount = velocity(runtime, key, 1 - axis, 0);
        if move_axis(runtime, key, 1 - axis, amount) {
            runtime.edit_actor(key, Native, |actor| {
                actor.variables[axis] = actor.variables[axis].wrapping_neg()
            });
            turn_list(runtime, key, record.word(6));
        } else if axis == 1 {
            snap(runtime, key);
        }
        return;
    }
    let turns = runtime.edit_actor(key, Native, |actor| {
        actor.variables[4] = actor.variables[4].wrapping_add(1);
        actor.variables[4]
    });
    if turns.wrapping_sub(4) & 0x8000 != 0 {
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[1 - axis] = actor.variables[1 - axis].wrapping_neg()
        });
        turn_list(runtime, key, record.word(4));
    } else {
        drop(runtime, key);
    }
}
/// Initialisation AI - yard ($A3:CDE2).
pub fn initialize(runtime: &mut impl Lease) {
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.pre_instruction = NO_MOVEMENT;
        actor.spritemap = 0x804d;
        actor.instruction_delay = 1;
    });
    let direction = runtime.resolve_actor(key, Native).instruction;
    orient(runtime, key, direction, false);
    let speed = runtime.resolve_actor(key, Native).parameters[0];
    {
        let mut part = YardEdit::of(runtime, slot(key));
        *part.behaviour() = IDLE_CRAWLING;
        *part.crawling_speed() = speed;
    }
    let key = combat::current(runtime);
    set_velocities(runtime, key, direction.wrapping_mul(8));
}
/// Handle yard hiding ($A3:CE9A): hide from Samus facing it from the side within 60h px
/// above, or crawl again.
fn handle_hiding(runtime: &mut impl Lease, key: Key) {
    let behaviour = Yard::of(runtime, slot(key)).behaviour();
    if matches!(behaviour, SAMUS_ON_TOP | DROPPED | KICKED | SHOT) {
        return;
    }
    let actor = runtime.resolve_actor(key, Native);
    let player = (runtime.session().player.x, runtime.session().player.y);
    if (i32::from(actor.y.whole) - i32::from(player.1) + 96) & 0x8000 == 0 {
        let facing_it = if (actor.x.whole as i16).wrapping_sub(player.0 as i16) < 0 {
            4
        } else {
            8
        };
        if pose_x_direction(runtime) == facing_it {
            if behaviour == HIDING {
                return;
            }
            if runtime.resolve_actor(key, Native).variables[3] != NO_MOVEMENT {
                runtime.edit_actor(key, Native, |actor| {
                    actor.instruction = actor.variables[3];
                    actor.spritemap = 0x804d;
                    actor.instruction_delay = 1;
                    actor.timer = 0;
                });
                *YardEdit::of(runtime, slot(key)).behaviour() = HIDING;
                return;
            }
        }
    }
    *YardEdit::of(runtime, slot(key)).behaviour() = IDLE_CRAWLING;
}
/// Determine if yard hitbox is solid to Samus ($A3:CF11).
fn set_solidity(runtime: &mut impl Lease, key: Key) {
    let mut solid = false;
    if runtime.resolve_actor(key, Native).pre_instruction != AIRBORNE {
        let still = runtime.session().player.contact.momentum == 0;
        let behaviour = Yard::of(runtime, slot(key)).behaviour();
        solid = if behaviour == SAMUS_ON_TOP {
            still && runtime.resolve_actor(key, Native).pre_instruction == NO_MOVEMENT
        } else {
            still && {
                let behaviour = Yard::of(runtime, slot(key)).behaviour();
                behaviour == HIDING || behaviour != DROPPED && behaviour != SHOT
            }
        };
    }
    runtime.edit_actor(key, Native, |actor| {
        if solid {
            actor.properties |= 0x8000;
        } else {
            actor.properties &= 0x7fff;
        }
    });
}
/// Main AI - yard ($A3:CE64): drop it on a super missile quake ($A3:CE73), handle
/// hiding and solidity; its movement function, which runs next.
pub fn prepare<H: LeaseActors + ?Sized>(game: &mut H) -> u32 {
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    let feedback = &runtime.session().feedback;
    let quake = feedback.earthquake_timer == 30 && feedback.earthquake_kind == 20;
    let behaviour = Yard::of(runtime, slot(key)).behaviour();
    if !matches!(behaviour, DROPPED | KICKED | SHOT) && quake {
        drop(runtime, key);
    }
    let key = combat::current(runtime);
    handle_hiding(runtime, key);
    let key = combat::current(runtime);
    set_solidity(runtime, key);
    let key = combat::current(runtime);
    u32::from(runtime.resolve_actor(key, Native).pre_instruction) | 0xa3_0000
}
/// Yard movement function - hiding ($A3:CF60): drop when nothing is ahead of it.
pub fn hiding<H: LeaseActors + ?Sized>(raw: u16, game: &mut H) {
    let runtime = &mut game.lease();
    let key = Key::from_legacy(raw);
    let axis = usize::from(Yard::of(runtime, slot(key)).direction() >= 4);
    let amount = velocity(runtime, key, axis, 7);
    let result = match axis {
        1 => api::probe_vertical(runtime, key, amount),
        _ => api::probe_horizontal(runtime, key, amount),
    };
    if result != Blocked {
        drop(runtime, key);
    }
}
/// Yard `yard`'s 16.16 airborne velocity on `axis`, the velocity read first.
fn velocity_pair(runtime: &impl Lease, yard: u16, axis: Axis) -> u32 {
    let yard = Yard::of(runtime, yard);
    let (whole, sub) = match axis {
        Axis::X => (yard.x_velocity(), yard.x_subvelocity()),
        Axis::Y => (yard.y_velocity(), yard.y_subvelocity()),
    };
    (u32::from(whole) << 16) | u32::from(sub)
}
/// Negate the subvelocity and then the velocity of yard `yard`'s `axis`, each as a word.
fn negate_pair(runtime: &mut impl Lease, yard: u16, axis: Axis) {
    let mut yard = YardEdit::of(runtime, yard);
    let negate = |word: &mut u16| *word = word.wrapping_neg();
    match axis {
        Axis::X => {
            negate(yard.x_subvelocity());
            negate(yard.x_velocity());
        }
        Axis::Y => {
            negate(yard.y_subvelocity());
            negate(yard.y_velocity());
        }
    }
}
fn bounce_sound(runtime: &mut impl Lease) {
    // SFX queue 2, at most 3 queued.
    runtime.queue_sound(2, 3, 0x70);
}
/// Yard movement function - airborne ($A3:D1B3): fly, bouncing off walls, then land.
pub fn airborne<H: LeaseActors + ?Sized>(raw: u16, game: &mut H) {
    let runtime = &mut game.lease();
    let key = Key::from_legacy(raw);
    let yard = slot(key);
    if Yard::of(runtime, yard).behaviour() != DROPPED {
        let amount = velocity_pair(runtime, yard, Axis::X) as i32;
        if api::move_horizontal(runtime, key, amount, SlopePolicy::Ignore) == Blocked {
            negate_pair(runtime, yard, Axis::X);
            *YardEdit::of(runtime, yard).bounced() = 1;
            bounce_sound(runtime);
        } else {
            let delta = if Yard::of(runtime, yard).x_velocity() & 0x8000 != 0 {
                0x1000
            } else {
                -0x1000i32 as u32
            };
            let next = velocity_pair(runtime, yard, Axis::X);
            let next = next.wrapping_add(delta);
            *YardEdit::of(runtime, yard).x_subvelocity() = next as u16;
            if next >> 16 != 0 {
                *YardEdit::of(runtime, yard).x_velocity() = (next >> 16) as u16;
            }
        }
    }
    let amount = velocity_pair(runtime, yard, Axis::Y) as i32;
    if api::move_vertical(runtime, key, amount) != Blocked {
        let next = velocity_pair(runtime, yard, Axis::Y).wrapping_add(0x2000);
        *YardEdit::of(runtime, yard).y_subvelocity() = next as u16;
        if (next >> 16).wrapping_sub(4) & 0x8000 != 0 {
            *YardEdit::of(runtime, yard).y_velocity() = (next >> 16) as u16;
        }
        return;
    }
    if !(0..3).contains(&(Yard::of(runtime, yard).y_velocity() as i16)) {
        negate_pair(runtime, yard, Axis::Y);
        *YardEdit::of(runtime, yard).bounced() = 0;
        return;
    }
    {
        let mut part = YardEdit::of(runtime, yard);
        *part.x_velocity() = 0;
        *part.x_subvelocity() = 0;
        *part.y_velocity() = 0;
        *part.y_subvelocity() = 0;
    }
    runtime.edit_actor(key, Native, |actor| actor.variables[4] = 0);
    {
        let mut part = YardEdit::of(runtime, yard);
        *part.turn_disable_counter() = 0;
        *part.turn_disabled() = 1;
    }
    if Yard::of(runtime, yard).behaviour() == DROPPED {
        *YardEdit::of(runtime, yard).behaviour() = IDLE_CRAWLING;
    } else {
        *YardEdit::of(runtime, yard).behaviour() = SAMUS_ON_TOP;
        runtime.edit_actor(key, Native, |actor| actor.parameters[0] = 8);
        face_samus(runtime, key);
        let index = (runtime.resolve_actor(key, Native).variables[2].wrapping_mul(4) >> 1) as isize;
        set_list_pair(runtime, key, 0xa3_d30d, index);
    }
    runtime.edit_actor(key, Native, |actor| {
        actor.properties = actor.variables[2] | actor.properties & !3
    });
    runtime
        .lease_owned_step(|runtime| crate::actors::crawlers::common_initialisation(raw, runtime));
    runtime.edit_actor(key, Native, |actor| {
        actor.pre_instruction = NO_MOVEMENT;
        actor.instruction = actor.variables[3];
        actor.instruction_delay = 1;
        actor.timer = 0;
    });
}
/// Kick yard into air ($A3:D49F), at the speed Samus moved last frame.
fn kick(runtime: &mut impl Lease, key: Key) {
    go_airborne(runtime, key, KICKED, 0xa3_d50f);
    let yard = slot(key);
    let fraction = runtime.session().player.camera_distance[0].fraction;
    *YardEdit::of(runtime, yard).x_subvelocity() = fraction;
    let distance = runtime.session().player.camera_distance[0].whole;
    *YardEdit::of(runtime, yard).x_velocity() = distance;
    let index = (distance.min(15).wrapping_mul(4) >> 1) as isize;
    let sub = table(runtime, 0xa3_d517, index);
    let whole = table(runtime, 0xa3_d517, index + 1);
    {
        let mut part = YardEdit::of(runtime, yard);
        *part.y_subvelocity() = sub;
        *part.y_velocity() = whole;
    }
    if pose_x_direction(runtime) & 4 != 0 {
        negate_pair(runtime, yard, Axis::X);
    }
}
/// Enemy touch - yard ($A3:D3B0): kicked when Samus runs into it or it is airborne;
/// otherwise it hurts her and turns.
pub fn touch(runtime: &mut impl Lease) {
    let key = combat::current(runtime);
    let yard = slot(key);
    let function = runtime.resolve_actor(key, Native).pre_instruction;
    let behaviour = Yard::of(runtime, yard).behaviour();
    let ridden = behaviour == SAMUS_ON_TOP
        && function != NO_MOVEMENT
        && pressing_towards(runtime, combat::current(runtime));
    if !ridden && (function == AIRBORNE || runtime.session().player.contact.momentum != 0) {
        let current = combat::current(runtime);
        kick(runtime, current);
        if runtime.resolve_actor(key, Native).pre_instruction == AIRBORNE {
            bounce_sound(runtime);
        }
    } else if function != NO_MOVEMENT && behaviour != KICKED && behaviour != DROPPED {
        api::contact_reaction(runtime, DeathHandling::Finalize);
        let speed = Yard::of(runtime, yard).crawling_speed();
        runtime.edit_actor(key, Native, |actor| actor.parameters[0] = speed);
        if Yard::of(runtime, yard).behaviour() != IDLE_CRAWLING {
            let current = combat::current(runtime);
            turn_around(runtime, current);
        }
        *YardEdit::of(runtime, yard).behaviour() = IDLE_CRAWLING;
    }
}
/// Enemy shot - yard ($A3:D469): power bombs and bombs hurt it; anything else shoots it
/// into the air ($A3:D557).
pub fn shot(runtime: &mut impl Lease) {
    let raw = runtime
        .session()
        .room
        .collision_detection_index
        .wrapping_mul(2);
    let kind = shot!(runtime, raw, kind) & 0xff00;
    if kind == 0x300 || kind == 0x500 {
        return combat::shot_reaction(runtime, ShotHandling::Finalize);
    }
    let key = combat::current(runtime);
    let behaviour = Yard::of(runtime, slot(key)).behaviour();
    if !matches!(behaviour, DROPPED | KICKED) {
        let current = combat::current(runtime);
        go_airborne(runtime, current, SHOT, 0xa3_d5a4);
        let away = if pose_x_direction(runtime) == 4 {
            0xffff
        } else {
            1
        };
        let mut part = YardEdit::of(runtime, slot(key));
        *part.y_velocity() = 0xffff;
        *part.x_velocity() = away;
    }
    bounce_sound(runtime);
}
/// The yard instructions: enemy movement function ($A3:CC36) or hiding instruction list
/// ($A3:CC3F) = [[Y]], set enemy direction ($A3:CC48), move ($A3:CC5F), and go back 4
/// bytes if hiding or with 50% chance ($A3:CC78).
pub fn instruction<H: LeaseActors + ?Sized>(
    ea: u32,
    k: u16,
    j: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    let runtime = &mut game.lease();
    let key = Key::from_legacy(k);
    match ea {
        0xa3_cc36 | 0xa3_cc3f => {
            let value = j.word(0);
            runtime.edit_actor(key, Native, |actor| match ea {
                0xa3_cc36 => actor.pre_instruction = value,
                _ => actor.variables[3] = value,
            });
        }
        0xa3_cc48 => {
            *YardEdit::of(runtime, slot(key)).direction() = j.word(0);
            let direction = Yard::of(runtime, slot(key)).direction();
            let facing = direction_data(runtime, direction, 3);
            runtime.edit_actor(key, Native, |actor| actor.variables[2] = facing);
        }
        0xa3_cc5f => {
            shift(runtime, key, |offset| j.word(offset as isize), false);
            return j.offset(2);
        }
        _ => {
            let hiding = Yard::of(runtime, slot(key)).behaviour() == HIDING;
            return if hiding || runtime.next_random() & 1 != 0 {
                j.offset(-3)
            } else {
                j
            };
        }
    }
    j.offset(1)
}
