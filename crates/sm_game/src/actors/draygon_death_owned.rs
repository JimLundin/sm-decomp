//! Draygon reactions, grapple drain and death over canonical actor/player state.
#![forbid(unsafe_code)]
use super::{
    ACCELERATION_STEP, BURIED_Y, DEATH_SPOT_RANGE, DEATH_X, DEATH_Y, DRAYGON_DEAD, ELEVATOR_MUSIC,
    EVIR_FRAMES, FOAM_FRAMES, GRAPPLE_DRAIN, GRAPPLE_FLAG, GRAPPLE_FRAMES, MAX_ACCELERATION,
    QUARTER_TURN,
};
use crate::actors::draygon_flight::owned::facing_animation;
use crate::actors::draygon_host::{
    ARMS, BODY, DELETE, Draygon, EYE, GRAPPLE_DROPPED, SIGN, TAIL, abs16, function,
    instruction_list as list,
};
use crate::actors::host::{Death, Shot};
use crate::actors::queries::angle_from_delta;
use crate::actors::session_slots::session_slot;
use crate::enemy_script::EnemyScript;

pub trait Host: crate::actors::draygon_flight::owned::Host {
    fn apply_hurt_palettes(&mut self);
    fn update_health_palette(&mut self);
    fn shot_reaction(&mut self, shot: Shot);
    fn contact_reaction(&mut self, death: Death);
    fn power_bomb_reaction(&mut self, death: Death);
    fn draygon_spawn_ring(&mut self);
    fn draygon_motion(&mut self, bias: u16);
    fn queue_music_delayed(&mut self, track: u16);
    fn boss_word(&mut self, area: u16) -> u16;
    fn set_boss_word(&mut self, area: u16, value: u16);
    fn scatter_draygon_drop(&mut self, area: u16);
    fn retire_all_sprites(&mut self);
}

pub fn hurt_ai<H: Host>(game: &mut H) {
    game.apply_hurt_palettes();
    hurt_grapple_drain(game);
}
pub fn hurt_grapple_drain<H: Host>(game: &mut H) {
    if game.session().player.grapple_interaction_flags & GRAPPLE_FLAG != 0
        && session_slot!(game, Draygon[BODY].base.frame_counter) & GRAPPLE_FRAMES == 0
    {
        let health = (session_slot!(game, Draygon[BODY].base.health) as i16)
            .wrapping_sub(GRAPPLE_DRAIN)
            .max(0);
        session_slot!(game, Draygon[BODY].base.health = health as u16);
        reaction(game);
    }
}

pub fn shot<L: Host>(game: &mut L) {
    if session_slot!(game, Draygon[BODY].swoop_y_acceleration).wrapping_sub(MAX_ACCELERATION) & SIGN
        != 0
    {
        session_slot!(game, Draygon[BODY].swoop_y_acceleration, |old| old
            .wrapping_add(ACCELERATION_STEP));
    }
    game.shot_reaction(Shot::ReactionOnly);
    reaction(game);
}

pub fn touch<L: Host>(game: &mut L) {
    game.contact_reaction(Death::LeavePending);
    reaction(game);
}

pub fn power_bomb<L: Host>(game: &mut L) {
    game.power_bomb_reaction(Death::LeavePending);
    reaction(game);
}

pub fn reaction<L: Host>(game: &mut L) {
    let k = game.session().actors.controls.selected as i32;
    if session_slot!(game, Draygon[k].base.health) != 0 {
        game.update_health_palette();
        return;
    }
    game.session_mut().player.grapple = GRAPPLE_DROPPED;
    facing_animation(
        BODY,
        BODY,
        [list::BODY_LEFT_DYING, list::BODY_RIGHT_DYING],
        game,
    );
    facing_animation(
        BODY,
        EYE,
        [list::EYE_LEFT_DYING, list::EYE_RIGHT_DYING],
        game,
    );
    session_slot!(game, Draygon[EYE].function = function::NONE);
    session_slot!(
        game,
        Draygon[BODY].function = function::DEATH_DRIFT_TO_DEATH_SPOT
    );
    game.release_samus();
    game.session_mut().player.grapple_interaction_flags = 0;
    let dx = DEATH_X.wrapping_sub(session_slot!(game, Draygon[BODY].base.x_pos));
    let dy = DEATH_Y.wrapping_sub(session_slot!(game, Draygon[BODY].base.y_pos));
    session_slot!(
        game,
        Draygon[BODY].death_angle =
            u16::from(QUARTER_TURN.wrapping_sub(angle_from_delta(dx, dy) as u8))
    );
    let vector = {
        let angle = u16::from(session_slot!(byte game, Draygon[BODY].death_angle[0]));
        game.direction_step(angle, 1)
    };
    session_slot!(game, Draygon[BODY]; death_x_speed = (vector.x >> 16) as u16,
        death_x_subspeed = vector.x as u16, death_y_speed = (vector.y >> 16) as u16,
        death_y_subspeed = vector.y as u16);
}

pub fn drift_to_death_spot<L: Host>(game: &mut L) {
    let k = game.session().actors.controls.selected as i32;
    game.foam_at_mouth(k as u16, FOAM_FRAMES);
    // The angle is from the position rounded to its 4x4 pixel block.
    let dx = (i32::from(DEATH_X >> 2)
        - (i32::from(session_slot!(game, Draygon[BODY].base.x_pos)) >> 2)) as u16;
    let dy = (i32::from(DEATH_Y >> 2)
        - (i32::from(session_slot!(game, Draygon[BODY].base.y_pos)) >> 2)) as u16;
    session_slot!(
        game,
        Draygon[BODY].death_angle =
            u16::from(QUARTER_TURN.wrapping_sub(angle_from_delta(dx, dy) as u8))
    );
    let vector = {
        let angle = u16::from(session_slot!(byte game, Draygon[BODY].death_angle[0]));
        game.direction_step(angle, 1)
    };
    crate::actors::draygon_grab::owned::store_speeds(vector, game);
    let scratch = u16::from(session_slot!(byte game, Draygon[BODY].death_angle[0]));
    game.set_step_scratch(scratch);
    let angle = u16::from(session_slot!(byte game, Draygon[BODY].death_angle[0]));
    game.step_along(angle, vector);
    if abs16(session_slot!(game, Draygon[BODY].base.x_pos).wrapping_sub(DEATH_X))
        .wrapping_sub(DEATH_SPOT_RANGE)
        & SIGN
        == 0
    {
        return;
    }
    if abs16(session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(DEATH_Y))
        .wrapping_sub(DEATH_SPOT_RANGE)
        & SIGN
        == 0
    {
        return;
    }
    game.draygon_spawn_ring();
    session_slot!(
        game,
        Draygon[BODY].function = function::DEATH_WAIT_FOR_EVIRS
    );
    game.queue_music_delayed(ELEVATOR_MUSIC);
    session_slot!(game, Draygon[BODY].function_timer = EVIR_FRAMES);
    session_slot!(game, Draygon[BODY].base; current_instruction = list::SLEEP, instruction_timer = 1);
    let properties = session_slot!(game, Draygon[BODY].base.properties) | DELETE;
    session_slot!(game, Draygon[TAIL].base.properties = properties);
    session_slot!(game, Draygon[ARMS].base.properties = properties);
    facing_animation(BODY, EYE, [list::EYE_LEFT_DEAD, list::EYE_RIGHT_DEAD], game);
    session_slot!(game, Draygon[EYE].function = function::NONE);
}

fn dying_smoke<H: Host + ?Sized>(game: &mut H) {
    if game.frame() & 7 == 0 {
        let random = game.roll_random();
        game.create_sprite((random & 0x7f) + 192, ((random >> 8) & 0x3f) + 400, 21, 0);
    }
}

pub fn spawn_dying_sprite<H: Host>(kind: u16, script: EnemyScript, game: &mut H) -> EnemyScript {
    let random = game.roll_random();
    let [dx, dy] = [random, random >> 8].map(|value| (value & 0x7f).wrapping_sub(64));
    let x = session_slot!(game, Draygon[BODY].base.x_pos).wrapping_add(dx);
    let y = session_slot!(game, Draygon[BODY].base.y_pos).wrapping_add(dy);
    game.create_sprite(x, y, kind, 0);
    script
}

pub fn spawn_goop<H: Host>(right: bool, script: EnemyScript, game: &mut H) -> EnemyScript {
    let dx = if right { 24 } else { 0xffe4 };
    let x = session_slot!(game, Draygon[BODY].base.x_pos).wrapping_add(dx);
    let y = session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(16);
    game.place_projectile_spawn(x, y);
    let angle = (game.roll_random() & 0x3f) + if right { 192 } else { 128 };
    game.session_mut().enemy_projectiles.spawn.secondary = angle;
    let enemy = game.session().actors.controls.selected;
    game.spawn_actor_eproj(2, enemy, 0x8e50);
    script
}

pub fn wait_for_evirs<L: Host>(game: &mut L) {
    dying_smoke(game);
    session_slot!(game, Draygon[BODY].function_timer, |old| old
        .wrapping_sub(1));
    if session_slot!(game, Draygon[BODY].function_timer) != 0 {
        // Moving left, the evirs also take $0E24, whatever was left there ($A5:A01C).
        let bias = game.session().actors.compatibility.scratch[1];
        game.draygon_motion(bias);
    } else {
        session_slot!(
            game,
            Draygon[BODY].function = function::DEATH_BURIED_BY_EVIRS
        );
    }
}

pub fn buried_by_evirs<L: Host>(game: &mut L) {
    dying_smoke(game);
    let bias = game.session().actors.compatibility.scratch[1];
    game.draygon_motion(bias);
    session_slot!(game, Draygon[BODY].base.y_pos, |old| old.wrapping_add(1));
    if session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(BURIED_Y) & SIGN == 0 {
        session_slot!(game, Draygon[BODY].base; current_instruction = list::DELETE, instruction_timer = 1);
        let properties = session_slot!(game, Draygon[BODY].base.properties) | DELETE;
        session_slot!(game, Draygon[BODY].base.properties = properties);
        session_slot!(game, Draygon[EYE].base.properties = properties);
        let drop_index = game.session().room.area;
        let area = game.session().room.area;
        let bits = game.boss_word(area);
        game.set_boss_word(area, bits | DRAYGON_DEAD);
        game.scatter_draygon_drop(drop_index);
        game.retire_all_sprites();
    }
}
