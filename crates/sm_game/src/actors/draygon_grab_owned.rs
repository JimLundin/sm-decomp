//! Draygon grab, pursuit, spiral and release over canonical actor/player state.
#![forbid(unsafe_code)]
use super::{
    DUST_CLOUD, DUST_CLOUD_Y, EARTHQUAKE_BG1_VERTICAL_3PX, EARTHQUAKE_FRAMES, ENEMY_DEF_BANK,
    ENEMY_DEF_DAMAGE, ENEMY_DEF_HURT_AI_TIME, FLASH_EXTRA, GRAB_COMMAND, GRAB_RANGE,
    GRAPPLE_CONNECTED, HOLD_X, HOLD_Y, HURT_AI, INITIAL_SWOOP_Y_ACCELERATION, MAX_SPIRAL_RADIUS,
    QUARTER_TURN, RISE, SAMUS_RELEASED, SPIRAL_ANGLE, SPIRAL_ANGLE_DELTA, SPIRAL_FOAM_FRAMES,
    SPIRAL_GROWTH, SPIRAL_RISE, SPIRAL_RISE_SUBPIXEL, SPIRAL_TOP, TAIL_WHIP_FRAMES, TAIL_WHIP_LASH,
    TARGET_RANGE, TARGET_X, TARGET_Y,
};
use crate::actors::draygon_flight::owned::facing_animation;
use crate::actors::draygon_host::{
    ARMS, BODY, Draygon, GRAPPLE_DROPPED, INTANGIBLE, SIGN, TAIL, abs16, function,
    instruction_list as list,
};
use crate::actors::queries::angle_from_delta;
use crate::actors::session_slots::session_slot;
use crate::enemy_script::EnemyScript;

pub trait Host:
    crate::actors::access::ActorAccess + crate::actors::session_slots::SessionSlots
{
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord;
    fn fire_wall_turret(&mut self) -> Option<u16>;
    fn foam_at_mouth(&mut self, key: u16, mask: u16);
    fn set_step_scratch(&mut self, value: u16);
    fn direction_step(&mut self, phase: u16, amplitude: u16) -> crate::geometry::Vector32;
    fn step_along(&mut self, phase: u16, vector: crate::geometry::Vector32);
    fn scaled_cosine(&mut self, phase: u16, amplitude: u16) -> u16;
    fn scaled_sine(&mut self, phase: u16, amplitude: u16) -> u16;
    fn command_player(&mut self, command: u16) -> u16;
    fn grab_samus(&mut self, direction: u16);
    fn release_samus(&mut self);
    fn correct_previous_position(&mut self);
    fn suit_damage(&mut self, amount: u16) -> u16;
    fn damage_player(&mut self, amount: u16);
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16;
}

pub(crate) fn store_speeds<S: Host>(vector: crate::geometry::Vector32, game: &mut S) {
    game.session_mut().actors.compatibility.scratch[1] = (vector.x >> 16) as u16;
    game.session_mut().actors.compatibility.scratch[2] = vector.x as u16;
    game.session_mut().actors.compatibility.scratch[3] = (vector.y >> 16) as u16;
    game.session_mut().actors.compatibility.scratch[4] = vector.y as u16;
}

fn pursue<S: Host>(dx: u16, dy: u16, game: &mut S) {
    let angle = QUARTER_TURN.wrapping_sub(angle_from_delta(dx, dy) as u8) as u16;
    game.set_step_scratch(angle);
    let vector = game.direction_step(angle, 2);
    store_speeds(vector, game);
    game.step_along(angle, vector);
}

fn hurt_by_grapple<S: Host>(game: &mut S) -> bool {
    if game.session().player.grapple_interaction_flags & GRAPPLE_CONNECTED == 0 {
        return false;
    }
    let definition = session_slot!(game, Draygon[BODY].base.enemy_ptr);
    let duration = game
        .actor_record(ENEMY_DEF_BANK, definition)
        .byte(ENEMY_DEF_HURT_AI_TIME);
    session_slot!(
        game,
        Draygon[BODY].base.flash_timer = u16::from(duration).wrapping_add(FLASH_EXTRA)
    );
    session_slot!(game, Draygon[BODY].base.ai_handler_bits, |old| old
        | HURT_AI);
    true
}

fn hold_offset<S: Host>(k: i32, game: &mut S) -> u16 {
    if session_slot!(game, Draygon[k].facing_right) == 0 {
        0u16.wrapping_sub(HOLD_X)
    } else {
        HOLD_X
    }
}

pub fn chase_samus<L: Host>(game: &mut L) {
    game.fire_wall_turret();
    if game.session().player.speed_divisor == 0 {
        session_slot!(game, Draygon[BODY].function = function::FLY_STRAIGHT_UP);
        return;
    }
    session_slot!(game, Draygon[BODY].base.properties, |old| old | INTANGIBLE);
    let dx = hold_offset(BODY, game);
    let distance = abs16(
        session_slot!(game, Draygon[BODY].base.x_pos)
            .wrapping_add(dx)
            .wrapping_sub(game.session().player.x),
    );
    if distance.wrapping_sub(GRAB_RANGE) & SIGN != 0
        && abs16(
            session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(game.session().player.y),
        )
        .wrapping_sub(GRAB_RANGE)
            & SIGN
            != 0
    {
        facing_animation(
            BODY,
            ARMS,
            [list::ARMS_LEFT_GRAB, list::ARMS_RIGHT_GRAB],
            game,
        );
        let direction = u16::from(session_slot!(game, Draygon[BODY].facing_right) != 0);
        let repelled = {
            let repelled = game.command_player(GRAB_COMMAND) != 0;
            if !repelled {
                crate::probe::hit("draygon_grab.grab");
                game.grab_samus(direction);
            }
            repelled
        };
        if repelled {
            session_slot!(game, Draygon[BODY].function = function::REPELLED_BY_GRAPPLE);
        } else {
            session_slot!(game, Draygon[BODY]; spiral_x = TARGET_X, spiral_y = TARGET_Y,
                spiral_x_radius = 0, spiral_x_subradius = 0, spiral_angle = SPIRAL_ANGLE,
                spiral_angle_delta = SPIRAL_ANGLE_DELTA,
                function = function::GRABBED_MOVING_TO_TARGET);
        }
    } else {
        let dx = game
            .session()
            .player
            .x
            .wrapping_sub(session_slot!(game, Draygon[BODY].base.x_pos));
        let dy = game
            .session()
            .player
            .y
            .wrapping_sub(session_slot!(game, Draygon[BODY].base.y_pos));
        pursue(dx, dy, game);
    }
}

pub fn repelled_by_grapple<S: Host>(game: &mut S) {
    crate::probe::hit("draygon_grab.repelled_by_grapple");
    game.session_mut().player.grapple = GRAPPLE_DROPPED;
    session_slot!(
        game,
        Draygon[BODY].function = function::FLAIL_TAIL_AND_FLY_STRAIGHT_UP
    );
}

pub fn move_to_target<S: Host>(k: u16, game: &mut S) {
    if hurt_by_grapple(game) {
        return;
    }
    let dx = abs16(session_slot!(game, Draygon[BODY].base.x_pos).wrapping_sub(TARGET_X));
    if dx.wrapping_sub(TARGET_RANGE) & SIGN != 0
        && abs16(session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(TARGET_Y))
            .wrapping_sub(TARGET_RANGE)
            & SIGN
            != 0
    {
        session_slot!(
            game,
            Draygon[BODY].function = function::GRABBED_RISING_SPIRAL
        );
        facing_animation(
            BODY,
            k as i32,
            [list::BODY_LEFT_ROAR, list::BODY_RIGHT_ROAR],
            game,
        );
        session_slot!(game, Draygon[BODY].base.properties, |old| old | INTANGIBLE);
    } else {
        let dx = TARGET_X.wrapping_sub(session_slot!(game, Draygon[BODY].base.x_pos));
        let dy = TARGET_Y.wrapping_sub(session_slot!(game, Draygon[BODY].base.y_pos));
        pursue(dx, dy, game);
        move_samus_with_draygon(k, game);
    }
}

pub fn rising_spiral<L: Host>(k: u16, game: &mut L) {
    if hurt_by_grapple(game) {
        return;
    }
    if game.session().timing.random_number as u8 == 0 {
        session_slot!(game, Draygon[BODY]; tail_whip_timer = TAIL_WHIP_FRAMES,
            function = function::GRABBED_TAIL_WHIP);
        return;
    }
    session_slot!(
        game,
        Draygon[BODY].base.x_pos =
            session_slot!(game, Draygon[BODY].spiral_x).wrapping_add(game.scaled_cosine(
                session_slot!(game, Draygon[BODY].spiral_angle),
                session_slot!(game, Draygon[BODY].spiral_x_radius),
            ))
    );
    session_slot!(
        game,
        Draygon[BODY].base.y_pos =
            session_slot!(game, Draygon[BODY].spiral_y).wrapping_add(game.scaled_sine(
                session_slot!(game, Draygon[BODY].spiral_angle),
                session_slot!(game, Draygon[BODY].spiral_x_radius) >> 2,
            ))
    );
    game.foam_at_mouth(k, SPIRAL_FOAM_FRAMES);
    session_slot!(
        game,
        Draygon[BODY](spiral_x_radius, spiral_x_subradius) += SPIRAL_GROWTH
    );
    let mut finished =
        session_slot!(game, Draygon[BODY].spiral_x_radius).wrapping_sub(MAX_SPIRAL_RADIUS) & SIGN
            == 0;
    if !finished {
        let delta = session_slot!(game, Draygon[BODY].spiral_angle_delta).wrapping_sub(1);
        session_slot!(game, Draygon[BODY].spiral_angle_delta = delta);
        let angle = session_slot!(game, Draygon[BODY].spiral_angle).to_ne_bytes()[0];
        session_slot!(
            game,
            Draygon[BODY].spiral_angle = u16::from(angle.wrapping_add(delta.to_ne_bytes()[1]))
        );
        let subpixel = session_slot!(game, Draygon[BODY].spiral_y_subpixel);
        session_slot!(
            game,
            Draygon[BODY].spiral_y_subpixel = subpixel.wrapping_sub(SPIRAL_RISE_SUBPIXEL)
        );
        let y = (((u32::from(session_slot!(game, Draygon[BODY].spiral_y)) << 16)
            | u32::from(subpixel))
        .wrapping_sub(SPIRAL_RISE)
            >> 16) as u16;
        session_slot!(game, Draygon[BODY].spiral_y = y);
        finished = y.wrapping_sub(SPIRAL_TOP) & SIGN != 0;
    }
    if finished {
        session_slot!(
            game,
            Draygon[BODY].function = function::GRABBED_FINAL_TAIL_WHIPS_START
        );
    } else {
        move_samus_with_draygon(k, game);
    }
}

pub fn tail_whip<S: Host>(k: u16, game: &mut S) {
    move_samus_with_draygon(k, game);
    let remaining = session_slot!(game, Draygon[BODY].tail_whip_timer).wrapping_sub(1);
    session_slot!(game, Draygon[BODY].tail_whip_timer = remaining);
    if remaining == 0 {
        session_slot!(
            game,
            Draygon[BODY].function = function::GRABBED_RISING_SPIRAL
        );
    } else if remaining == TAIL_WHIP_LASH {
        crate::probe::hit("draygon_grab.tail_whip");
        facing_animation(
            BODY,
            TAIL,
            [list::TAIL_LEFT_TAIL_WHIP, list::TAIL_RIGHT_TAIL_WHIP],
            game,
        );
    }
}

pub fn final_tail_whips<S: Host>(k: u16, game: &mut S) {
    crate::probe::hit("draygon_grab.final_tail_whips");
    move_samus_with_draygon(k, game);
    facing_animation(
        k as i32,
        TAIL,
        [
            list::TAIL_LEFT_FINAL_TAIL_WHIPS,
            list::TAIL_RIGHT_FINAL_TAIL_WHIPS,
        ],
        game,
    );
    session_slot!(
        game,
        Draygon[BODY].function = function::GRABBED_FINAL_TAIL_WHIPS_ONGOING
    );
}

pub fn move_samus_with_draygon<S: Host>(k: u16, game: &mut S) {
    let dx = hold_offset(k as i32, game);
    game.session_mut().player.x = session_slot!(game, Draygon[BODY].base.x_pos).wrapping_add(dx);
    game.session_mut().player.y =
        session_slot!(game, Draygon[BODY].base.y_pos).wrapping_add(HOLD_Y);
    game.correct_previous_position();
    if game.session().player.grapple_interaction_flags & SAMUS_RELEASED != 0 {
        game.session_mut().player.grapple_interaction_flags = 0;
        session_slot!(game, Draygon[BODY].function = function::FLY_STRAIGHT_UP);
    }
}

pub fn flail_tail_and_fly_straight_up<L: Host>(game: &mut L) {
    crate::probe::hit("draygon_grab.flail_tail_and_fly_straight_up");
    game.release_samus();
    game.session_mut().player.grapple_interaction_flags = 0;
    session_slot!(game, Draygon[BODY].function = function::FLY_STRAIGHT_UP);
    session_slot!(game, Draygon[BODY].base.properties, |old| old & !INTANGIBLE);
    facing_animation(
        BODY,
        TAIL,
        [list::TAIL_LEFT_TAIL_FLAIL, list::TAIL_RIGHT_TAIL_FLAIL],
        game,
    );
}

pub fn fly_straight_up<S: Host>(game: &mut S) {
    game.fire_wall_turret();
    let y = (session_slot!(game, Draygon[BODY].base.y_pos) as i16).wrapping_sub(RISE);
    session_slot!(game, Draygon[BODY].base.y_pos = y as u16);
    if y < 0 {
        session_slot!(game, Draygon[BODY].base.properties, |old| old & !INTANGIBLE);
        session_slot!(game, Draygon[BODY]; function = function::SWOOP_RIGHT_SETUP, function_timer = 0);
        session_slot!(game, Draygon[BODY].base; x_pos = session_slot!(game, Draygon[BODY].left_reset_x),
            y_pos = session_slot!(game, Draygon[BODY].reset_y));
    }
}

pub fn tail_whip_hit<L: Host>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(
        game,
        Draygon[BODY].swoop_y_acceleration = INITIAL_SWOOP_Y_ACCELERATION
    );
    let definition = session_slot!(game, Draygon[BODY].base.enemy_ptr);
    let amount = game.actor_record(ENEMY_DEF_BANK, definition);
    let amount = game.suit_damage(amount.word(ENEMY_DEF_DAMAGE));
    game.damage_player(amount);
    game.session_mut().feedback.earthquake_timer = EARTHQUAKE_FRAMES;
    game.session_mut().feedback.earthquake_kind = EARTHQUAKE_BG1_VERTICAL_3PX;
    let x = game.session().player.x;
    let y = game.session().player.y.wrapping_add(DUST_CLOUD_Y);
    game.create_sprite(x, y, DUST_CLOUD, 0);
    script
}
