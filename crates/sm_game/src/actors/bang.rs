//! Bang, the electric orb that grows as it is shot and flies at Samus (bank
//! $A3:B702-$A3:BF2F): its movement functions and shot reaction.
use crate::actors::access::{AccessMode::Native, ActorAccess};
use crate::actors::api::{self, DeathKind};
use crate::actors::combat::{self, current_slot};
use crate::actors::host::{ActorLease, BossEffects as _, LeaseActors, sign_extend8};
use crate::actors::layouts::part_view;
use crate::enemy_projectiles::access::shot_word;
use crate::enemy_projectiles::host::ShotMember;
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::{self, table};
use crate::player_projectiles::motion;
use crate::ram::words as W;
use crate::session_access::SessionAccess as _;
use crate::session_slot;
use crate::shot;

part_view! {
    /// Bang's view of its slot and extended RAM.
    Bang: EnemyLayout72 {
        words: [
            var_a, idle_palette, var_c, unread_0fae, damage, movement_function, ai_function,
            parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            movement_delay_timer, angle, angle_to_samus, var_03, var_04, var_05, var_06,
            moving_left, moving_up, acceleration, speed, max_speed, acceleration_interval_timer,
            acceleration_interval_reset, deceleration_interval_reset, var_0f, var_10, var_11,
            var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19, var_1a, var_1b, var_1c,
            var_1d, var_1e, var_1f,
        ],
        pad2: [u8; 1984],
        words: [
            new_instruction_list_index, instruction_list_index, finished_growing,
        ],
        _: [u16; 29],
    }
}

/// Bank $A3 bang movement functions, stored in `movement_function`.
pub mod function {
    pub const MOVEMENT_DELAY: u16 = 0xbca5;
    /// An RTS: the core-less bang never moves.
    pub const NO_MOVEMENT: u16 = 0xbcc1;
    pub const TARGET_SAMUS: u16 = 0xbcc5;
    pub const ACCELERATING: u16 = 0xbd1c;
    pub const DECELERATING: u16 = 0xbd2c;
}

/// Bank $A3 bang AI functions, stored in `ai_function` (parameter 1).
pub mod ai {
    pub const ELECTRICITY: u16 = 0xbb2b;
    pub const SHELL: u16 = 0xbb4a;
    pub const CORE: u16 = 0xbb66;
}

/// $A3:B722: the instruction list for each instruction list index; $A3:B75E:
/// the core, growth level 0, idling.
const INSTRUCTION_LISTS: u32 = 0xa3_b722;
const CORE_IDLING: u16 = 0xb75e;
/// $A3:BA84: the max speeds; $A3:BC6A: the acceleration and deceleration
/// interval reset values; $A3:BA94: the angle each projectile direction knocks
/// a bang to; $A0:B1C3: the sine table.
const MAX_SPEEDS: u32 = 0xa3_ba84;
const INTERVALS: u32 = 0xa3_bc6a;
const SHOT_ANGLES: u32 = 0xa3_ba94;
const SINE: u32 = 0xa0_b1c3;
/// Properties bit 12: block plasma beam.
const BLOCK_PLASMA: u16 = 0x1000;
const MOVEMENT_DELAY_FRAMES: u16 = 16;
/// Angles are a byte from the top; 40h less is from the positive X axis.
const QUARTER_TURN: u8 = 0x40;
/// Off target by 30h or more, a bang decelerates.
const OFF_TARGET: u16 = 0x30;
const ACCELERATION_STEP: u16 = 0x16;
/// A shot knocks a bang away at speed 600h, acceleration 100h.
const KNOCKED_SPEED: u16 = 0x600;
const KNOCKED_ACCELERATION: u16 = 0x100;
/// The last growth level's instruction list index.
const FULLY_GROWN: u16 = 9;
const SIGN: u16 = 0x8000;
/// The electricity, core and shell are consecutive slots.
const PART: i32 = 64;
/// The electricity's and shell's instruction lists follow the core's ten.
const ELECTRICITY_LISTS: u16 = 10;
const SHELL_LISTS: u16 = 20;
/// Palette 6 (beams), which a growing core flashes.
const GROWING_PALETTE: u16 = 0xc00;
/// Properties: intangible, and deleted.
const INTANGIBLE: u16 = 0x400;
const DELETED: u16 = 0x200;
/// Projectile type bit 4: a charged beam.
const CHARGED: u16 = 0x10;
/// $A3:BC4A: each charged beam's sound, by beam type.
const CHARGED_SOUNDS: u32 = 0xa3_bc4a;

/// Sign-extend the low byte when bit 7 is set; otherwise keep the high byte,
/// as the original did.
/// Set bang instruction list: start the list for the new index if it changed.
fn set_instruction_list(runtime: &mut impl ActorLease) {
    let bang = current_slot(runtime);
    let index = session_slot!(runtime, Bang[bang].new_instruction_list_index);
    if index != session_slot!(runtime, Bang[bang].instruction_list_index) {
        session_slot!(runtime, Bang[bang].instruction_list_index = index);
        let instruction = table(runtime, INSTRUCTION_LISTS, index as isize);
        let key = combat::current(runtime);
        runtime.edit_actor(key, Native, |actor| actor.restart_animation(instruction));
    }
}
/// Initialisation AI - bang ($A3:BAB3).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    let bang = current_slot(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.variables[1] = actor.palette;
        actor.properties |= BLOCK_PLASMA;
        actor.pre_instruction = function::MOVEMENT_DELAY;
    });
    session_slot!(
        runtime,
        Bang[bang].movement_delay_timer = MOVEMENT_DELAY_FRAMES
    );
    session_slot!(runtime, Bang[bang]; angle = 0, angle_to_samus = 0,
        new_instruction_list_index = 0, instruction_list_index = 0, finished_growing = 0);
    let parameter = runtime.resolve_actor(key, Native).parameters[1].to_ne_bytes();
    let limit = table(runtime, MAX_SPEEDS, parameter[0] as isize);
    session_slot!(runtime, Bang[bang].max_speed = limit);
    let index = 2 * parameter[1] as isize;
    let interval = table(runtime, INTERVALS, index);
    session_slot!(runtime, Bang[bang]; acceleration_interval_timer = interval,
        acceleration_interval_reset = interval);
    let interval = table(runtime, INTERVALS, index + 1);
    session_slot!(runtime, Bang[bang].deceleration_interval_reset = interval);
    runtime.edit_actor(key, Native, |actor| {
        if actor.instruction == 0 {
            actor.pre_instruction = function::NO_MOVEMENT;
        }
        actor.instruction = CORE_IDLING;
    });
}
/// Main AI - bang ($A3:BB25): the AI function in its first parameter, for the
/// electricity, the core or the shell, three consecutive slots.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let bang = current_slot(runtime);
    match session_slot!(runtime, Bang[bang].ai_function) {
        ai::ELECTRICITY => follow_core(runtime, bang + PART, ELECTRICITY_LISTS),
        ai::SHELL => follow_core(runtime, bang - PART, SHELL_LISTS),
        ai::CORE => core(runtime),
        _ => runtime.foreign(|game| game.unreachable()),
    }
}
/// Bang AI - electricity ($A3:BB2B) or shell ($A3:BB4A): the core's growth level in
/// this part's instruction lists, and intangible.
fn follow_core(runtime: &mut impl ActorLease, core: i32, lists: u16) {
    let bang = current_slot(runtime);
    let index = session_slot!(runtime, Bang[core].new_instruction_list_index).wrapping_add(lists);
    session_slot!(runtime, Bang[bang].new_instruction_list_index = index);
    set_instruction_list(runtime);
    let bang = current_slot(runtime);
    session_slot!(runtime, Bang[bang].base.properties, |properties| properties
        | INTANGIBLE);
}
/// Bang AI - core ($A3:BB66): move, carry the electricity and shell along, flash while
/// growing, and once fully grown fire a charged beam at Samus and die with its parts.
fn core(runtime: &mut impl ActorLease) {
    // Execute bang movement function ($A3:BC9E).
    let bang = current_slot(runtime);
    match session_slot!(runtime, Bang[bang].movement_function) {
        function::MOVEMENT_DELAY => delay(runtime),
        function::NO_MOVEMENT => {}
        function::TARGET_SAMUS => target(runtime),
        function::ACCELERATING => move_bang(false, runtime),
        function::DECELERATING => move_bang(true, runtime),
        _ => runtime.foreign(|game| game.unreachable()),
    }
    let x = session_slot!(runtime, Bang[bang].base.x_pos);
    session_slot!(runtime, Bang[bang - PART].base.x_pos = x);
    session_slot!(runtime, Bang[bang + PART].base.x_pos = x);
    let y = session_slot!(runtime, Bang[bang].base.y_pos);
    session_slot!(runtime, Bang[bang - PART].base.y_pos = y);
    session_slot!(runtime, Bang[bang + PART].base.y_pos = y);
    let growing = session_slot!(runtime, Bang[bang].new_instruction_list_index) & 1 != 0;
    let palette = match growing {
        true => GROWING_PALETTE,
        false => session_slot!(runtime, Bang[bang].idle_palette),
    };
    session_slot!(runtime, Bang[bang].base.palette_index = palette);
    set_instruction_list(runtime);
    if session_slot!(runtime, Bang[bang].finished_growing) == 0 {
        return;
    }
    session_slot!(runtime, Bang[bang].finished_growing = 0);
    let index = session_slot!(runtime, Bang[bang].new_instruction_list_index);
    if index != FULLY_GROWN {
        let bang = current_slot(runtime);
        session_slot!(
            runtime,
            Bang[bang].new_instruction_list_index = index.wrapping_add(1)
        );
        return set_instruction_list(runtime);
    }
    session_slot!(runtime, Bang[bang].base.invincibility_timer = 16);
    session_slot!(runtime, Bang[bang].base.properties, |properties| properties
        | INTANGIBLE);
    let direction = api::direction_to_player(runtime).projectile_direction();
    let animation = fire_charged_beam(runtime, direction);
    combat::die(runtime, DeathKind::Normal { animation });
    let bang = current_slot(runtime);
    for part in [bang + PART, bang - PART] {
        session_slot!(runtime, Bang[part].base.properties, |properties| properties
            | DELETED);
    }
}
/// Fire charged beam at Samus ($A3:BBEB) in projectile `direction`, as Samus's own
/// charged beam with the bang's damage; the death animation: a small explosion, or
/// with five projectiles out already, the Samus-contact death.
fn fire_charged_beam(runtime: &mut impl ActorLease, direction: u16) -> u16 {
    runtime.write_slice_le(W::REMOVED_ENEMY_RIGHT_BORDER_COLLISION, direction);
    if runtime.session().projectiles.count.wrapping_sub(5) & SIGN == 0 {
        return 1;
    }
    let mut slot = 0;
    while shot!(runtime, slot, damage) != 0 {
        slot += 2;
    }
    let bang = current_slot(runtime);
    let x = session_slot!(runtime, Bang[bang].base.x_pos);
    shot!(runtime, slot, x = x);
    let y = session_slot!(runtime, Bang[bang].base.y_pos);
    shot!(runtime, slot, y = y);
    shot!(runtime, slot, direction = direction);
    let beam = runtime.session().inventory.beams.active & 15;
    shot!(runtime, slot, kind = beam | CHARGED);
    let count = runtime.session().projectiles.count.wrapping_add(1);
    runtime.session_mut().projectiles.count = count;
    runtime.session_mut().spawn_scratch.coordinates = slot;
    motion::reflect(runtime, slot);
    let damage = session_slot!(runtime, Bang[bang].damage);
    shot!(runtime, slot, damage = damage);
    let beam = shot!(runtime, slot, kind) & 15;
    let sound = table(runtime, CHARGED_SOUNDS, usize::from(beam).cast_signed());
    runtime.queue_sound(0, sound as u8, 6);
    0
}
/// Bang movement function - movement delay ($A3:BCA5). Was `wait`.
pub fn movement_delay<H: LeaseActors + ?Sized>(game: &mut H) {
    delay(&mut game.lease());
}
fn delay(runtime: &mut impl ActorLease) {
    let bang = current_slot(runtime);
    let remaining = session_slot!(runtime, Bang[bang].movement_delay_timer).wrapping_sub(1);
    session_slot!(runtime, Bang[bang].movement_delay_timer = remaining);
    if remaining == 0 {
        session_slot!(
            runtime,
            Bang[bang].movement_delay_timer = MOVEMENT_DELAY_FRAMES
        );
        let key = combat::current(runtime);
        runtime.edit_actor(key, Native, |actor| {
            actor.pre_instruction = function::TARGET_SAMUS
        });
    }
}
fn angle_to_samus(runtime: &impl ActorLease) -> u16 {
    u16::from(
        (api::angle_to_player(runtime, combat::current(runtime)) as u8).wrapping_sub(QUARTER_TURN),
    )
}
/// Bang movement function - target Samus ($A3:BCC5). Was `aim`.
pub fn target_samus<H: LeaseActors + ?Sized>(game: &mut H) {
    target(&mut game.lease());
}
fn target(runtime: &mut impl ActorLease) {
    let angle = angle_to_samus(runtime);
    let bang = current_slot(runtime);
    session_slot!(runtime, Bang[bang].angle = angle);
    let key = combat::current(runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.pre_instruction = function::ACCELERATING
    });
    session_slot!(runtime, Bang[bang]; moving_left = 0, moving_up = 0, acceleration = 0, speed = 0);
}
/// Bang movement function - accelerating ($A3:BD1C) or, with `decelerating`,
/// decelerating ($A3:BD2C): move along the angle (bang X and Y movement,
/// $A3:BDB9), then accelerate ($A3:BD4F) and trigger deceleration if too far
/// off target ($A3:BCF1), or decelerate ($A3:BD89). Was `fly`.
pub fn fly<H: LeaseActors + ?Sized>(decelerating: bool, game: &mut H) {
    move_bang(decelerating, &mut game.lease());
}
fn move_bang(decelerating: bool, runtime: &mut impl ActorLease) {
    for (x, adjustment) in [(true, QUARTER_TURN), (false, 0)] {
        let bang = current_slot(runtime);
        let angle = (session_slot!(runtime, Bang[bang].angle) as u8).wrapping_add(adjustment);
        let sine = table(runtime, SINE, angle as isize);
        if sine & SIGN != 0 {
            if x {
                session_slot!(runtime, Bang[bang].moving_left = 1);
            } else {
                session_slot!(runtime, Bang[bang].moving_up = 1);
            }
        }
        let mut velocity = i32::from((sine as i16).unsigned_abs() >> 8)
            * i32::from(session_slot!(runtime, Bang[bang].speed));
        let reversed = if x {
            session_slot!(runtime, Bang[bang].moving_left)
        } else {
            session_slot!(runtime, Bang[bang].moving_up)
        };
        if reversed != 0 {
            velocity = -velocity;
        }
        let key = combat::current(runtime);
        runtime.edit_actor(key, Native, |actor| {
            if x {
                actor.x.advance(velocity)
            } else {
                actor.y.advance(velocity)
            }
        });
    }
    let bang = current_slot(runtime);
    let remaining = session_slot!(runtime, Bang[bang].acceleration_interval_timer).wrapping_sub(1);
    session_slot!(runtime, Bang[bang].acceleration_interval_timer = remaining);
    if remaining & SIGN != 0 {
        let interval = if decelerating {
            session_slot!(runtime, Bang[bang].deceleration_interval_reset)
        } else {
            session_slot!(runtime, Bang[bang].acceleration_interval_reset)
        };
        session_slot!(runtime, Bang[bang].acceleration_interval_timer = interval);
        let speed = session_slot!(runtime, Bang[bang].speed);
        if decelerating
            || speed.wrapping_sub(session_slot!(runtime, Bang[bang].max_speed)) & SIGN != 0
        {
            let acceleration =
                session_slot!(runtime, Bang[bang].acceleration).wrapping_add(if decelerating {
                    ACCELERATION_STEP.wrapping_neg()
                } else {
                    ACCELERATION_STEP
                });
            session_slot!(runtime, Bang[bang].acceleration = acceleration);
            let next = if decelerating {
                speed.wrapping_sub(acceleration)
            } else {
                speed.wrapping_add(acceleration)
            };
            session_slot!(runtime, Bang[bang].speed = next);
        }
    }
    if decelerating {
        let bang = current_slot(runtime);
        if session_slot!(runtime, Bang[bang].speed) != 0
            || session_slot!(runtime, Bang[bang].acceleration) as i16 <= 0
        {
            let key = combat::current(runtime);
            runtime.edit_actor(key, Native, |actor| {
                actor.pre_instruction = function::MOVEMENT_DELAY
            });
        }
    } else {
        let angle = angle_to_samus(runtime);
        let bang = current_slot(runtime);
        session_slot!(runtime, Bang[bang].angle_to_samus = angle);
        let difference = sign_extend8(angle.wrapping_sub(session_slot!(runtime, Bang[bang].angle)));
        if (difference as i16).unsigned_abs().wrapping_sub(OFF_TARGET) & SIGN == 0 {
            let key = combat::current(runtime);
            runtime.edit_actor(key, Native, |actor| {
                actor.pre_instruction = function::DECELERATING
            });
        }
    }
}
/// Enemy shot - bang ($A3:BEFD): knock the bang away and grow it.
pub fn shot<H: LeaseActors + ?Sized>(game: &mut H) {
    let runtime = &mut game.lease();
    let key = combat::current(runtime);
    let bang = current_slot(runtime);
    if runtime.resolve_actor(key, Native).pre_instruction != function::NO_MOVEMENT {
        let image = runtime.leased_image().expect("ROM must be loaded");
        let index = runtime.session().room.collision_detection_index as isize;
        let direction = shot_word(runtime, ShotMember::Direction, index) & 15;
        let angle = image.table_word(content::offset(SHOT_ANGLES), direction as isize);
        session_slot!(runtime, Bang[bang].angle = angle);
        runtime.edit_actor(key, Native, |actor| {
            actor.pre_instruction = function::DECELERATING
        });
        session_slot!(runtime, Bang[bang]; moving_left = 0, moving_up = 0,
            acceleration = KNOCKED_ACCELERATION, speed = KNOCKED_SPEED);
    }
    let stage = session_slot!(runtime, Bang[bang].new_instruction_list_index);
    if stage != FULLY_GROWN {
        session_slot!(
            runtime,
            Bang[bang].new_instruction_list_index = stage.wrapping_add(1)
        );
        set_instruction_list(runtime);
        let index = runtime.session().room.collision_detection_index as isize;
        let damage = shot_word(runtime, ShotMember::Damage, index);
        runtime.edit_actor(key, Native, |actor| {
            actor.variables[4] = actor.variables[4].wrapping_add(damage)
        });
        combat::mark_shot(runtime, index);
        if session_slot!(runtime, Bang[bang].new_instruction_list_index) == FULLY_GROWN {
            runtime.edit_actor(key, Native, |actor| actor.variables[3] = 1);
        }
    }
}
/// Instruction - enemy finished growing flag = 1 ($A3:BAA8). Was `arm`.
pub fn set_finished_growing<H: LeaseActors + ?Sized>(
    cursor: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    let runtime = &mut game.lease();
    let bang = current_slot(runtime);
    session_slot!(runtime, Bang[bang].finished_growing = 1);
    cursor
}
