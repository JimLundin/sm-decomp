//! The elevator platform, which carries Samus between rooms (bank
//! $A3:94D6-$A3:95F5); its status lives with the room entry state.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess};
use crate::actors::combat;
use crate::actors::host::{ActorLease, BossEffects as _, LeaseActors};
use crate::enemy_projectiles::host::Effects as _;
use crate::session_access::SessionAccess as _;

/// Elevator status values.
pub mod status {
    pub const INACTIVE: u16 = 0;
    pub const LEAVING_ROOM: u16 = 1;
    pub const DOOR_TRANSITION: u16 = 2;
    pub const ENTERING_ROOM: u16 = 3;
}

/// The elevator's target Y position is its first `variables` word; its first
/// parameter, doubled at initialisation, is nonzero for an upward elevator.
const TARGET_Y: usize = 0;
/// $A3:94D6: the elevator's instruction list; $A3:94E2: the controller
/// inputs that start it, down then up; $A3:804D: no spritemap.
const INSTRUCTION_LIST: u16 = 0x94d6;
const CONTROLLER_INPUTS: u32 = 0xa3_94e2;
const NO_SPRITEMAP: u16 = 0x804d;
/// Samus stands 1Ah px above the elevator, which moves 1.8000h px a frame.
const SAMUS_HEIGHT: u16 = 0x1a;
const SPEED: i32 = 0x18000;
const UP: u16 = 0x8000;
/// The elevator (Bh, library 3), spin jump end (32h, library 1) and silence
/// (25h, library 3).
const ELEVATOR_SOUND: u16 = 0xb;
const SPIN_JUMP_END: u16 = 0x32;
const SILENCE: u16 = 0x25;
/// Samus commands 7: set Samus up for the elevator, and Bh: unlock Samus from
/// facing forward.
const SET_UP_FOR_ELEVATOR: u16 = 7;
const UNLOCK_FACING_FORWARD: u16 = 0xb;

/// Place Samus on top of elevator.
fn place_samus(runtime: &mut impl ActorLease) {
    let key = combat::current(runtime);
    runtime.session_mut().player.y = runtime
        .resolve_actor(key, Native)
        .y
        .whole
        .wrapping_sub(SAMUS_HEIGHT);
    runtime.session_mut().player.y_fraction = 0;
    runtime.session_mut().player.x = runtime.resolve_actor(key, Native).x.whole;
    runtime.session_mut().player.contact.y_speed.whole = 0;
    runtime.session_mut().player.contact.y_speed.fraction = 0;
}
/// Initialisation AI - elevator ($A3:94E6).
pub fn initialize<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.spritemap = NO_SPRITEMAP;
        actor.restart_animation(INSTRUCTION_LIST);
        actor.parameters[0] = actor.parameters[0].wrapping_mul(2);
        actor.variables[TARGET_Y] = actor.y.whole;
    });
    if runtime.session().room.entry.elevator_status != status::DOOR_TRANSITION {
        runtime.session_mut().room.entry.elevator_status = status::INACTIVE;
        runtime.session_mut().room.entry.elevator_flags = 0;
    }
    let entry = &runtime.session().room.entry;
    if entry.elevator_status != 0 || entry.elevator_flags != 0 {
        runtime.edit_actor(key, Native, |actor| actor.y.whole = actor.parameters[1]);
        place_samus(&mut runtime);
    }
}
/// Main AI / grapple AI / frozen AI - elevator ($A3:952A): run the elevator
/// AI for its status. Was `advance`.
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let entry = &runtime.session().room.entry;
    if entry.elevator_interlock != 0 || (entry.elevator_status == 0 && entry.elevator_flags == 0) {
        return;
    }
    // Retain the former four-entry dispatch's bounds panic for unknown states.
    let status = [
        status::INACTIVE,
        status::LEAVING_ROOM,
        status::DOOR_TRANSITION,
        status::ENTERING_ROOM,
    ][usize::from(entry.elevator_status)];
    if status == status::INACTIVE {
        // Elevator AI - elevator status = 0: inactive ($A3:9548).
        let image = runtime.leased_image().expect("ROM must be loaded");
        let key = combat::current(&runtime);
        let index = runtime.resolve_actor(key, Native).parameters[0] >> 1;
        let buttons = image.table_word(
            sm_data::cartridge::lorom_offset(CONTROLLER_INPUTS) as usize,
            index as isize,
        );
        if buttons & runtime.inputs().pressed != 0 {
            runtime.queue_sound(3, 6, ELEVATOR_SOUND);
            runtime.queue_sound(1, 6, SPIN_JUMP_END);
            runtime.foreign(|game| game.command_player(SET_UP_FOR_ELEVATOR));
            runtime.foreign(|game| game.reset_player_projectiles());
            place_samus(&mut runtime);
            let entry = &mut runtime.session_mut().room.entry;
            entry.elevator_status = entry.elevator_status.wrapping_add(1);
        } else {
            runtime.session_mut().room.entry.elevator_flags = 0;
        }
        return;
    }
    // Elevator AI - elevator status = 2: door transition ($A3:95B9).
    if status == status::DOOR_TRANSITION {
        let entry = &mut runtime.session_mut().room.entry;
        entry.elevator_status = entry.elevator_status.wrapping_add(1);
    }
    // Elevator AI - elevator status = 1: leaving room ($A3:9579), and 3:
    // entering room ($A3:95BC).
    let leaving = status == status::LEAVING_ROOM;
    let key = combat::current(&runtime);
    let upward = runtime.resolve_actor(key, Native).parameters[0] != 0;
    if leaving {
        runtime.session_mut().room.entry.elevator_direction = if upward { UP } else { 0 };
    }
    let arrived = runtime.edit_actor(key, Native, |actor| {
        actor
            .y
            .advance(if upward == leaving { -SPEED } else { SPEED });
        !leaving && (actor.y.whole < actor.variables[TARGET_Y]) != upward
    });
    if arrived {
        runtime.session_mut().room.entry.elevator_status = status::INACTIVE;
        runtime.session_mut().room.entry.elevator_flags = 0;
        runtime.queue_sound(3, 6, SILENCE);
        runtime.edit_actor(key, Native, |actor| {
            actor.y.whole = actor.variables[TARGET_Y]
        });
        runtime.foreign(|game| game.command_player(UNLOCK_FACING_FORWARD));
    }
    place_samus(&mut runtime);
}
