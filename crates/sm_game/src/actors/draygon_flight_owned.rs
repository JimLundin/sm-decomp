//! Draygon turret/goop behavior and script transitions over owned state.
#![forbid(unsafe_code)]
use super::{
    BUBBLE, BUBBLE_OFFSET, FOAM, FOAM_X, FOAM_Y, GOOP_CHANCE, GOOP_RANGE, GOOP_START_X, GOOP_Y,
    GOOP_Y_AMPLITUDE, GOOPS, OFF_SCREEN_LEFT, OFF_SCREEN_RIGHT, PALETTE_7, RESET_Y, RIGHT_RESET_X,
    TURRET_FRAMES, TURRET_PROJECTILE, TURRET_SPEED,
};
use crate::actors::draygon_host::{
    ARMS, BODY, Draygon, EYE, INTANGIBLE, SIGN, TAIL, abs16, function, instruction_list as list,
};
use crate::actors::session_slots::{SlotAccess, session_slot};
use crate::enemy_script::EnemyScript;

pub trait Host: crate::actors::draygon_grab::owned::Host {
    fn frame(&self) -> u16;
    fn roll_random(&mut self) -> u16;
    fn turret_position(&mut self, index: isize) -> u16;
    fn place_projectile_spawn(&mut self, x: u16, y: u16);
    fn spawn_actor_eproj(&mut self, parameter: u16, key: u16, definition: u16) -> u16;
}

pub fn displace_graphics<S: SlotAccess>(script: EnemyScript, game: &mut S) -> EnemyScript {
    session_slot!(game, Draygon[BODY]; graphics_x_displacement = script.word(0),
        graphics_y_displacement = script.word(2));
    script.offset(2)
}

pub fn facing_animation<S: SlotAccess>(
    facing_key: i32,
    part_key: i32,
    instructions: [u16; 2],
    game: &mut S,
) {
    let instruction =
        instructions[usize::from(session_slot!(game, Draygon[facing_key].facing_right) != 0)];
    session_slot!(game, Draygon[part_key].base; current_instruction = instruction, instruction_timer = 1);
}

pub fn foam_at_mouth<L: crate::actors::draygon_grab::owned::Host>(
    k: u16,
    frame_mask: u16,
    game: &mut L,
) {
    if session_slot!(game, Draygon[k as i32].base.frame_counter) & frame_mask == 0 {
        let dx = if session_slot!(game, Draygon[BODY].facing_right) != 0 {
            FOAM_X
        } else {
            0u16.wrapping_sub(FOAM_X)
        };
        let x = session_slot!(game, Draygon[BODY].base.x_pos).wrapping_add(dx);
        let y = session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(FOAM_Y);
        game.create_sprite(x, y, FOAM, 0);
    }
}

pub fn fire_wall_turret<S: Host>(game: &mut S) -> Option<u16> {
    if game.frame() & TURRET_FRAMES != 0 {
        return None;
    }
    let index = 2 * ((game.roll_random() & 3) + 2);
    if session_slot!(game, Draygon[index as i32].turret_destroyed) == 0 {
        let x = game.turret_position(index as isize);
        let y = game.turret_position(index as isize + 1);
        game.place_projectile_spawn(x, y);
        game.spawn_actor_eproj(TURRET_SPEED, index, TURRET_PROJECTILE);
    }
    Some(index)
}

pub fn initialize_tail_or_arms<S: Host>(arms: bool, game: &mut S) {
    let k = game.session().actors.controls.selected as i32;
    let instruction = if arms {
        list::ARMS_LEFT_IDLE
    } else {
        list::TAIL_LEFT_FAKE_TAIL_WHIP
    };
    session_slot!(game, Draygon[k].base; current_instruction = instruction, palette_index = PALETTE_7);
    if arms {
        session_slot!(game, Draygon[k].base.layer = 2);
    }
}

fn goop_glide<S: Host>(right: bool, game: &mut S) {
    let angle = session_slot!(game, Draygon[BODY].goop_y_angle);
    session_slot!(
        game,
        Draygon[BODY].base.y_pos = game
            .scaled_cosine(angle, GOOP_Y_AMPLITUDE)
            .wrapping_add(GOOP_Y)
    );
    session_slot!(
        game,
        Draygon[BODY].goop_y_angle,
        |old| (old as u8).wrapping_add(1) as u16
    );
    move_horizontally(right, game);
}

pub fn move_horizontally<S: SlotAccess>(right: bool, game: &mut S) {
    let high = session_slot!(game, Draygon[BODY].x_speed);
    let low = session_slot!(game, Draygon[BODY].x_subspeed);
    let speed = (u32::from(high) << 16) | u32::from(low);
    let delta = if right {
        speed
    } else {
        (-(speed as i32)) as u32
    };
    session_slot!(game, Draygon[BODY].base(x_pos, x_subpos) += delta);
}

fn beyond_right(x: u16) -> bool {
    x as i16 >= 0 && x.wrapping_sub(OFF_SCREEN_RIGHT) & SIGN == 0
}

pub fn goop_setup<S: Host>(right: bool, game: &mut S) {
    if right {
        fire_wall_turret(game);
    }
    session_slot!(
        game,
        Draygon[BODY].base.x_pos = if right {
            GOOP_START_X
        } else {
            session_slot!(game, Draygon[BODY].right_reset_x)
        }
    );
    session_slot!(game, Draygon[BODY].base.y_pos = GOOP_Y);
    session_slot!(game, Draygon[BODY]; x_subspeed = 0, x_speed = 1, goop_y_angle = 0);
    session_slot!(game, Draygon[ARMS].base; current_instruction = list::ARMS_RIGHT_NEAR_SWOOP_APEX, instruction_timer = 1);
    session_slot!(
        game,
        Draygon[BODY].function = if right {
            function::GOOP_RIGHT_MOVE_UNTIL_IN_RANGE
        } else {
            function::GOOP_LEFT_MOVE_UNTIL_IN_RANGE
        }
    );
    session_slot!(game, Draygon[BODY].base; current_instruction = if right { list::BODY_RIGHT_RESET } else { list::BODY_LEFT_RESET }, instruction_timer = 1);
    session_slot!(game, Draygon[BODY].facing_right = u16::from(right));
}

pub fn body_bubble<L: crate::actors::draygon_grab::owned::Host>(turret: Option<u16>, game: &mut L) {
    let k = i32::from(turret.unwrap_or_else(|| game.session().actors.controls.selected));
    if session_slot!(game, Draygon[k].base.frame_counter) & 0x7f == 0 {
        let x = session_slot!(game, Draygon[BODY].base.x_pos).wrapping_sub(BUBBLE_OFFSET);
        let y = session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(BUBBLE_OFFSET);
        game.create_sprite(x, y, BUBBLE, 0);
    }
}

pub fn goop_move_until_in_range<L: Host>(right: bool, game: &mut L) {
    let turret = fire_wall_turret(game);
    body_bubble(turret, game);
    let distance =
        abs16(session_slot!(game, Draygon[BODY].base.x_pos).wrapping_sub(game.session().player.x));
    if distance.wrapping_sub(GOOP_RANGE) & SIGN != 0 {
        session_slot!(game, Draygon[BODY]; function = if right { function::GOOP_RIGHT_FIRING } else { function::GOOP_LEFT_FIRING }, goop_counter = GOOPS);
    } else {
        goop_glide(right, game);
    }
}

pub fn goop_firing<S: Host>(right: bool, game: &mut S) {
    if right {
        fire_wall_turret(game);
    }
    if game.session().player.speed_divisor == 0 {
        let mut leave = false;
        if game.session().timing.random_number & GOOP_CHANCE == 0 {
            let remaining = session_slot!(game, Draygon[BODY].goop_counter).wrapping_sub(1);
            session_slot!(game, Draygon[BODY].goop_counter = remaining);
            leave = remaining == 0;
            if !leave {
                session_slot!(game, Draygon[BODY].base; current_instruction = if right { list::BODY_RIGHT_FIRE_GOOP } else { list::BODY_LEFT_FIRE_GOOP }, instruction_timer = 1);
            }
        }
        if !leave {
            goop_glide(right, game);
            // The left pass tests unsigned x_pos < 0, so its left-edge test never
            // succeeds. The left exit below intentionally uses the signed comparison.
            let x = session_slot!(game, Draygon[BODY].base.x_pos);
            if !right || !beyond_right(x) {
                return;
            }
        }
        if game.session().player.speed_divisor == 0 {
            session_slot!(game, Draygon[ARMS].base; current_instruction = if right { list::ARMS_RIGHT_IDLE } else { list::ARMS_LEFT_IDLE }, instruction_timer = 1);
            session_slot!(
                game,
                Draygon[BODY].function = if right {
                    function::GOOP_RIGHT_MOVE_UNTIL_OFF_SCREEN
                } else {
                    function::GOOP_LEFT_MOVE_UNTIL_OFF_SCREEN
                }
            );
            return;
        }
    }
    session_slot!(game, Draygon[BODY].function = function::CHASE_SAMUS);
}

pub fn goop_move_until_off_screen<L: Host>(right: bool, game: &mut L) {
    body_bubble(None, game);
    if !right && game.session().player.speed_divisor != 0 {
        session_slot!(game, Draygon[BODY].function = function::CHASE_SAMUS);
        return;
    }
    goop_glide(right, game);
    let x = session_slot!(game, Draygon[BODY].base.x_pos);
    let finished = if right {
        beyond_right(x)
    } else {
        (x as i16) < 0 && x.wrapping_add(OFF_SCREEN_LEFT) & SIGN != 0
    };
    if finished {
        session_slot!(
            game,
            Draygon[BODY].function = if right {
                function::SWOOP_LEFT_SETUP
            } else {
                function::SWOOP_RIGHT_SETUP
            }
        );
        session_slot!(game, Draygon[BODY].base; x_pos = if right { RIGHT_RESET_X } else { GOOP_START_X }, y_pos = RESET_Y);
        let copied = session_slot!(game, Draygon[BODY].swoop_duration);
        session_slot!(game, Draygon[BODY].function_timer = copied);
    }
}

pub fn set_instruction_lists<S: SlotAccess>(script: EnemyScript, game: &mut S) -> EnemyScript {
    for (k, offset) in [(BODY, 0), (EYE, 2), (TAIL, 4), (ARMS, 6)] {
        session_slot!(
            game,
            Draygon[k].base.current_instruction = script.word(offset)
        );
    }
    for k in [BODY, EYE, TAIL, ARMS] {
        session_slot!(game, Draygon[k].base.instruction_timer = 1);
    }
    script.offset(4)
}

pub fn paralyse_tail_and_arms<S: SlotAccess>(script: EnemyScript, game: &mut S) -> EnemyScript {
    for k in [TAIL, ARMS] {
        session_slot!(game, Draygon[k].base; instruction_timer = 1, current_instruction = list::SLEEP);
    }
    script
}

pub fn set_intangible<S: SlotAccess>(script: EnemyScript, game: &mut S) -> EnemyScript {
    session_slot!(game, Draygon[BODY].base.properties, |old| old | INTANGIBLE);
    script
}
