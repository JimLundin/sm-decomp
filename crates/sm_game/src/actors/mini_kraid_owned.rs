//! Mini Kraid behavior over the current owned actor/player state.
#![forbid(unsafe_code)]
use super::{
    DEATH_ANIMATION, FACING_LEFT, FACING_RIGHT, MiniKraid, ROAR_SOUND, SPIKE_SOUND,
    SPIT_VELOCITIES, animation, projectile,
};
use crate::actors::host::{Death, Shot, Slopes};
use crate::constants::enemy_properties::K_ENEMY_PROPS_DISABLE_SAMUS_COLL;
use crate::enemy_projectiles::drops::DropSource;
use crate::enemy_script::EnemyScript;
use crate::session_slot;

pub trait Host:
    crate::actors::access::ActorAccess
    + crate::actors::session_slots::SessionSlots
    + crate::actors::host::ActorRuntime
    + crate::effects::SoundEffects
{
    fn spawn_actor_eproj(&mut self, parameter: u16, key: u16, definition: u16) -> u16;
    fn normal_death(&mut self, animation: u16);
    fn scatter_enemy_drop(&mut self, source: DropSource, key: u16);
}

fn current<L: Host + ?Sized>(game: &L) -> i32 {
    i32::from(game.session().actors.controls.selected)
}

fn face_samus<L: Host + ?Sized>(game: &mut L, me: i32) {
    session_slot!(game, MiniKraid[me].facing = FACING_LEFT);
    if ((session_slot!(game, MiniKraid[me].base.x_pos)) as i16)
        .wrapping_sub((game.session().player.x) as i16)
        < 0
    {
        session_slot!(game, MiniKraid[me].facing = FACING_RIGHT);
    }
}

pub fn initialize<L: Host + ?Sized>(game: &mut L) {
    let mut steps: u16 = ((game.session().timing.random_number as i32 & 3) + 2) as u16;
    let me = current(game);
    session_slot!(game, MiniKraid[me]; steps_until_turn = steps, cycles_until_spit = steps);
    steps = steps.wrapping_add(64);
    session_slot!(game, MiniKraid[me].spike_timer = steps);
    steps = steps.wrapping_add(32);
    session_slot!(game, MiniKraid[me]; spike_timer_1 = steps, spike_timer_2 = steps.wrapping_sub(48),
        spike_cycle = 0);
    session_slot!(
        game,
        MiniKraid[me].base.properties = (session_slot!(game, MiniKraid[me].base.properties))
            | (K_ENEMY_PROPS_DISABLE_SAMUS_COLL as u16)
    );
    session_slot!(game, MiniKraid[me].base; instruction_timer = 1, timer = 0);
    session_slot!(game, MiniKraid[me]; walk_speed = FACING_LEFT, facing = FACING_LEFT);
    let mut walk = animation::WALK_FORWARD_LEFT;
    if ((session_slot!(game, MiniKraid[me].base.x_pos)) as i16)
        .wrapping_sub((game.session().player.x) as i16)
        < 0
    {
        session_slot!(game, MiniKraid[me]; walk_speed = FACING_RIGHT, facing = FACING_RIGHT);
        walk = animation::WALK_FORWARD_RIGHT;
    }
    session_slot!(game, MiniKraid[me].base.current_instruction = walk);
}

pub fn main_ai<L: Host + ?Sized>(game: &mut L) {
    let me = current(game);
    let spike = session_slot!(game, MiniKraid[me].spike_cycle);
    let mut next = spike.wrapping_add(2);
    if (spike as i32 - 4i32) & 0x8000 == 0 {
        next = 0;
    }
    session_slot!(game, MiniKraid[me].spike_cycle = next);
    fire_spikes_when_ready(game, game.session().actors.controls.selected, spike);
}

fn fire_spikes_when_ready<L: Host + ?Sized>(game: &mut L, k: u16, spike: u16) {
    let timer: i32 = k as i32 + spike as i32;
    let left = session_slot!(game, MiniKraid[timer].spike_timer);
    if left != 0 {
        session_slot!(game, MiniKraid[timer].spike_timer = left.wrapping_sub(1));
    } else {
        session_slot!(
            game,
            MiniKraid[timer].spike_timer =
                ((game.session().timing.random_number as i32 & 0x3f) + 16) as u16
        );
        let me = k as i32;
        session_slot!(game, MiniKraid[me].last_spike = spike);
        let mut spikes = projectile::SPIKES_LEFT;
        let facing: i16 = session_slot!(game, MiniKraid[me].facing) as i16;
        if facing >= 0 {
            spikes = projectile::SPIKES_RIGHT;
        }
        game.spawn_actor_eproj(facing as u16, k, spikes);
        if u16::from(game.current_outside_screen()) == 0 {
            game.queue_sound(1, SPIKE_SOUND, 6);
        }
    }
}

fn spit<L: Host + ?Sized>(game: &mut L, row: u16, offset: u16) {
    let k: u16 = game.session().actors.controls.selected;
    let me = current(game);
    session_slot!(game, MiniKraid[me].spit_x_offset = offset);
    let index: i32 = row as i32 >> 1;
    session_slot!(
        game,
        MiniKraid[me].spit_x_velocity = SPIT_VELOCITIES[index as usize]
    );
    let y_velocity: u16 = SPIT_VELOCITIES[(index + 1) as usize];
    session_slot!(game, MiniKraid[me].spit_y_velocity = y_velocity);
    game.spawn_actor_eproj(
        y_velocity,
        game.session().actors.controls.selected,
        projectile::SPIT,
    );
    session_slot!(
        game,
        MiniKraid[me].spit_x_velocity = SPIT_VELOCITIES[(index + 2) as usize]
    );
    let y_velocity: u16 = SPIT_VELOCITIES[(index + 3) as usize];
    session_slot!(game, MiniKraid[me].spit_y_velocity = y_velocity);
    game.spawn_actor_eproj(y_velocity, k, projectile::SPIT);
}

pub fn step<L: Host + ?Sized>(game: &mut L) {
    let me = current(game);
    if session_slot!(game, MiniKraid[me].cycles_until_spit) != 0 {
        session_slot!(game, MiniKraid[me].cycles_until_spit, |old| old
            .wrapping_sub(1));
    }
    let steps = session_slot!(game, MiniKraid[me].steps_until_turn);
    session_slot!(game, MiniKraid[me].steps_until_turn, |old| old
        .wrapping_sub(1));
    let turn_around: bool = if steps == 1 {
        session_slot!(
            game,
            MiniKraid[me].steps_until_turn =
                ((game.session().timing.random_number as i32 & 3) + 7) as u16
        );
        true
    } else {
        let actor = game.session().actors.controls.selected;
        let amount = (session_slot!(game, MiniKraid[me].walk_speed) as i16 as i32) << 16;
        game.move_horizontal(actor, amount, Slopes::Ignore)
    };
    if turn_around {
        session_slot!(
            game,
            MiniKraid[me].walk_speed =
                -(session_slot!(game, MiniKraid[me].walk_speed) as i32) as u16
        );
    }
    face_samus(game, me);
}

pub fn choose_animation<L: Host + ?Sized>(game: &mut L) -> EnemyScript {
    let me = current(game);
    if session_slot!(game, MiniKraid[me].cycles_until_spit) != 0 {
        if (session_slot!(game, MiniKraid[me].facing) & 0x8000) != 0 {
            if (session_slot!(game, MiniKraid[current(game)].walk_speed) & 0x8000) == 0 {
                return EnemyScript::jump(animation::WALK_BACKWARD_LEFT);
            }
            EnemyScript::jump(animation::WALK_FORWARD_LEFT)
        } else {
            if (session_slot!(game, MiniKraid[me].walk_speed) & 0x8000) != 0 {
                return EnemyScript::jump(animation::WALK_BACKWARD_RIGHT);
            }
            EnemyScript::jump(animation::WALK_FORWARD_RIGHT)
        }
    } else {
        session_slot!(
            game,
            MiniKraid[me].cycles_until_spit =
                ((game.session().timing.random_number as i32 & 3) + 3) as u16
        );
        if (session_slot!(game, MiniKraid[me].facing) & 0x8000) != 0 {
            return EnemyScript::jump(animation::SPIT_LEFT);
        }
        EnemyScript::jump(animation::SPIT_RIGHT)
    }
}

pub fn roar<L: Host + ?Sized>(game: &mut L) {
    if u16::from(game.current_outside_screen()) == 0 {
        game.queue_sound(1, ROAR_SOUND, 6);
    }
}

pub fn spit_left<L: Host + ?Sized>(game: &mut L) {
    spit(game, 0, FACING_LEFT);
}

pub fn spit_right<L: Host + ?Sized>(game: &mut L) {
    spit(game, 8, FACING_RIGHT);
}

fn mark_drop_origin<L: Host + ?Sized>(game: &mut L) {
    let me = current(game);
    game.session_mut().enemy_projectiles.drop_origin.x =
        session_slot!(game, MiniKraid[me].base.x_pos);
    game.session_mut().enemy_projectiles.drop_origin.y =
        session_slot!(game, MiniKraid[me].base.y_pos);
}

pub fn touch<L: Host + ?Sized>(game: &mut L) {
    mark_drop_origin(game);
    game.contact_reaction(Death::LeavePending);
    die_when_out_of_health(game);
}

pub fn shot<L: Host + ?Sized>(game: &mut L) {
    mark_drop_origin(game);
    game.shot_reaction(Shot::ReactionOnly);
    die_when_out_of_health(game);
}

fn die_when_out_of_health<L: Host + ?Sized>(game: &mut L) {
    if session_slot!(
        game,
        MiniKraid[game.session().actors.controls.selected as i32]
            .base
            .health
    ) == 0
    {
        game.normal_death(DEATH_ANIMATION);
        let index = game.session().actors.controls.selected;
        game.scatter_enemy_drop(DropSource::MiniKraid, index);
    }
}
