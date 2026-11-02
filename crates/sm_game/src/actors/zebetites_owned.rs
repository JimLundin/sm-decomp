//! Zebetite lifecycle over owned actors and room state.
#![forbid(unsafe_code)]
use super::{
    DESTROYED_EVENTS, HIT_SOUND, MAX_HEALTH, NEXT_ZEBETITES, PARTNER, ZEBETITES, Zebetite, state,
};
use crate::actors::host::{Death, Shot};
use crate::constants::enemy_properties::K_ENEMY_PROPS_DELETED;
use crate::session_slot;

pub trait Host:
    crate::actors::access::ActorAccess
    + crate::actors::session_slots::SessionSlots
    + crate::effects::SoundEffects
{
    fn placement(&self, destroyed: isize) -> sm_data::zebetites::Placement;
    fn health_lists(&self, stage: isize) -> sm_data::zebetites::HealthLists;
    fn event_happened(&mut self, event: u16) -> u16;
    fn set_event(&mut self, event: u16);
    fn clear_event(&mut self, event: u16);
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16;
    fn glow(&mut self);
    fn normal_death(&mut self, animation: u16);
    fn invalid_function(&mut self, state: u16) -> !;
    fn react_to_contact(&mut self, death: Death);
    fn react_to_shot(&mut self, shot: Shot);
}

fn current<L: Host + ?Sized>(game: &L) -> i32 {
    i32::from(game.session().actors.controls.selected)
}

pub fn initialize<L: Host + ?Sized>(game: &mut L) {
    let me = current(game);
    session_slot!(game, Zebetite[me].base; properties = (session_slot!(game, Zebetite[me].base.properties)) | (0xa000u16),
        instruction_timer = 1, timer = 0, palette_index = 1024, vram_tiles_index = 128, );
    session_slot!(game, Zebetite[me].glow_frame = 0);
    let mut first_state = state::SPAWN_PARTNER;
    if session_slot!(game, Zebetite[me].second_of_pair) != 0 {
        first_state = state::WAIT_FOR_DOOR;
    }
    session_slot!(game, Zebetite[me].state = first_state);
    let mut destroyed: u16 = 0;
    for event in DESTROYED_EVENTS.into_iter().rev() {
        destroyed = ((destroyed as i32) << 1 | game.event_happened(event) as i32) as u16;
    }
    session_slot!(game, Zebetite[me].destroyed = destroyed);
    if (destroyed as i32 - ZEBETITES as i32) & 0x8000 != 0 {
        let placement = game.placement(destroyed as isize);
        session_slot!(game, Zebetite[me].pair = placement.multi_part);
        session_slot!(game, Zebetite[me].base.y_height = placement.height);
        session_slot!(
            game,
            Zebetite[me].base.current_instruction = placement.instruction_list
        );
        session_slot!(game, Zebetite[me].base.x_pos = placement.x_position);
        let y = if session_slot!(game, Zebetite[me].second_of_pair) != 0 {
            placement.y_position_alternate
        } else {
            placement.y_position
        };
        session_slot!(game, Zebetite[me].base.y_pos = y);
    } else {
        session_slot!(
            game,
            Zebetite[me].base.properties = (session_slot!(game, Zebetite[me].base.properties))
                | (K_ENEMY_PROPS_DELETED as u16)
        );
    }
}

pub fn main_ai<L: Host + ?Sized>(game: &mut L) {
    if game.session().feedback.earthquake_timer == 0 {
        session_slot!(game, Zebetite[current(game)].base.shake_timer = 0);
    }
    let me = current(game);
    let state = session_slot!(game, Zebetite[me].state);
    let k = game.session().actors.controls.selected;
    match state {
        state::SPAWN_PARTNER => spawn_partner(game, k),
        state::WAIT_FOR_DOOR => wait_for_door(game, k),
        state::ACTIVE => run(game),
        _ => game.invalid_function(state),
    }
}

fn spawn_partner<L: Host + ?Sized>(game: &mut L, k: u16) {
    let me = k as i32;
    if (session_slot!(game, Zebetite[me].pair) & 0x8000) != 0 {
        let partner = game.spawn_population(0xa6, PARTNER);
        session_slot!(game, Zebetite[partner as i32].partner = k);
        session_slot!(game, Zebetite[k as i32].partner = partner);
    }
    session_slot!(game, Zebetite[k as i32].state = state::WAIT_FOR_DOOR);
    wait_for_door(game, k);
}

fn wait_for_door<L: Host + ?Sized>(game: &mut L, k: u16) {
    if game.session().room.entry.elevator_interlock == 0 {
        session_slot!(game, Zebetite[k as i32].state = state::ACTIVE);
        run(game);
    }
}

fn run<L: Host + ?Sized>(game: &mut L) {
    game.glow();
    show_health_stage(game);
    let me = current(game);
    let health: u16 = session_slot!(game, Zebetite[me].base.health);
    if health != 0 {
        let mut health: i16 = (health as i16).wrapping_add(1);
        if (health as i32 - MAX_HEALTH as i32) & 0x8000 == 0 {
            health = MAX_HEALTH;
        }
        session_slot!(game, Zebetite[me].base.health = health as u16);
    } else if session_slot!(game, Zebetite[me].second_of_pair) != 0 || {
        count_destroyed(game);
        (session_slot!(game, Zebetite[me].destroyed) as i32 - ZEBETITES as i32) & 0x8000 == 0
    } {
        die(game);
    } else {
        die(game);
        game.spawn_population(0xa6, NEXT_ZEBETITES);
    }
}

fn die<L: Host + ?Sized>(game: &mut L) {
    game.normal_death(0);
}

fn count_destroyed<L: Host + ?Sized>(game: &mut L) {
    let me = current(game);
    let destroyed: i16 = ((session_slot!(game, Zebetite[me].destroyed)) as i16).wrapping_add(1);
    session_slot!(game, Zebetite[me].destroyed = destroyed as u16);
    for (bit, event) in DESTROYED_EVENTS.into_iter().enumerate() {
        if (destroyed as i32 >> bit & 1) as u8 != 0 {
            game.set_event(event);
        } else {
            game.clear_event(event);
        }
    }
}

fn show_health_stage<L: Host + ?Sized>(game: &mut L) {
    let mut stage: u16 = 0;
    let me = current(game);
    let health: u16 = session_slot!(game, Zebetite[me].base.health);
    for threshold in [800, 600, 400, 200] {
        if (health as i32 - threshold) & 0x8000 != 0 {
            stage += 2;
        } else {
            break;
        }
    }
    let lists = game.health_lists((stage as i32 >> 1) as isize);
    let mut animation: u16 = lists.big;
    if (session_slot!(game, Zebetite[me].pair) & 0x8000) != 0 {
        animation = lists.small_pair;
    }
    session_slot!(game, Zebetite[me].base; current_instruction = animation, instruction_timer = 1);
    session_slot!(game, Zebetite[me].base.timer = 0);
}

pub fn touch<L: Host + ?Sized>(game: &mut L) {
    game.react_to_contact(Death::LeavePending);
}

pub fn shot<L: Host + ?Sized>(game: &mut L) {
    game.queue_sound(2, HIT_SOUND, 6);
    game.react_to_shot(Shot::ReactionOnly);
    let me = current(game);
    let partner: i32 = session_slot!(game, Zebetite[me].partner) as i32;
    session_slot!(
        game,
        Zebetite[partner].base.health = session_slot!(game, Zebetite[me].base.health)
    );
    session_slot!(
        game,
        Zebetite[partner].base.flash_timer = session_slot!(game, Zebetite[me].base.flash_timer)
    );
}
