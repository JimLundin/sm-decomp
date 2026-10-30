//! Elevator travel shares one movement path and the room-entry state owner.
#![forbid(unsafe_code)]
use super::access::{AccessMode::Native, ActorAccess};
use super::combat;
use crate::game::session_runtime::{Runtime, player_command};
use crate::game_state::Game;
use sm_game::enemy_projectiles::host::Effects;

fn synchronize_player(runtime: &mut Runtime<'_>) {
    let key = combat::current(runtime);
    runtime.session_mut().player.y = runtime.resolve_actor(key, Native).y.whole.wrapping_sub(26);
    runtime.session_mut().player.y_fraction = 0;
    runtime.session_mut().player.x = runtime.resolve_actor(key, Native).x.whole;
    runtime.session_mut().player.contact.y_speed.whole = 0;
    runtime.session_mut().player.contact.y_speed.fraction = 0;
}
pub(crate) fn initialize(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let key = combat::current(&runtime);
    runtime.edit_actor(key, Native, |actor| {
        actor.spritemap = 0x804d;
        actor.restart_animation(0x94d6);
        actor.parameters[0] = actor.parameters[0].wrapping_mul(2);
        actor.variables[0] = actor.y.whole;
    });
    if runtime.session().room.entry.elevator_status != 2 {
        runtime.session_mut().room.entry.elevator_status = 0;
        runtime.session_mut().room.entry.elevator_flags = 0;
    }
    let entry = &runtime.session().room.entry;
    if entry.elevator_status != 0 || entry.elevator_flags != 0 {
        runtime.edit_actor(key, Native, |actor| actor.y.whole = actor.parameters[1]);
        synchronize_player(&mut runtime);
    }
}
pub(crate) fn advance(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let entry = &runtime.session().room.entry;
    if entry.elevator_interlock != 0 || (entry.elevator_status == 0 && entry.elevator_flags == 0) {
        return;
    }
    // Retain the former four-entry dispatch's bounds panic for unknown states.
    let phase = [0, 1, 2, 3][usize::from(entry.elevator_status)];
    if phase == 0 {
        let image = runtime.loaded_image().expect("ROM must be loaded");
        let key = combat::current(&runtime);
        let index = runtime.resolve_actor(key, Native).parameters[0] >> 1;
        let buttons = image.table_word(crate::rom::lorom_offset(0xa394e2) as usize, index as isize);
        if buttons & runtime.inputs().pressed != 0 {
            runtime.queue_sound(3, 6, 0xb);
            runtime.queue_sound(1, 6, 0x32);
            runtime.foreign(|game| player_command(7, game));
            runtime.foreign(crate::game::player_projectiles::reset);
            synchronize_player(&mut runtime);
            let entry = &mut runtime.session_mut().room.entry;
            entry.elevator_status = entry.elevator_status.wrapping_add(1);
        } else {
            runtime.session_mut().room.entry.elevator_flags = 0;
        }
        return;
    }
    if phase == 2 {
        let entry = &mut runtime.session_mut().room.entry;
        entry.elevator_status = entry.elevator_status.wrapping_add(1);
    }
    let departing = phase == 1;
    let key = combat::current(&runtime);
    let upward = runtime.resolve_actor(key, Native).parameters[0] != 0;
    if departing {
        runtime.session_mut().room.entry.elevator_direction = if upward { 0x8000 } else { 0 };
    }
    let arrived = runtime.edit_actor(key, Native, |actor| {
        actor.y.advance(if upward == departing {
            -0x18000
        } else {
            0x18000
        });
        !departing && (actor.y.whole < actor.variables[0]) != upward
    });
    if arrived {
        runtime.session_mut().room.entry.elevator_status = 0;
        runtime.session_mut().room.entry.elevator_flags = 0;
        runtime.queue_sound(3, 6, 0x25);
        runtime.edit_actor(key, Native, |actor| actor.y.whole = actor.variables[0]);
        runtime.foreign(|game| player_command(0xb, game));
    }
    synchronize_player(&mut runtime);
}
