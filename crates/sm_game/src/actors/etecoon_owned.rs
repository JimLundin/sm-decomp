//! Etecoon behavior over the current owned actor and player state.
#![forbid(unsafe_code)]
use super::{
    Etecoon, FLEX_SOUND, HOP_SOUND, HOP_SPEED, LEFT_RUN_UP_X, LOOK_DISTANCE, RIGHT_RUN_UP_X,
    RUN_LEFT_SPEED, RUN_RIGHT_SPEED, SOUND_SAMUS_X, TOP_HOPS, TUNNEL_JUMP_SPEED, TUNNEL_LEDGE_X,
    TUNNEL_X, WAIT_FOR_SAMUS_X, WALL_JUMP_SOUND, function, instruction_list as list,
};
use crate::actors::api::PlayerDirection;
use crate::actors::host::{Axis, Slopes, below};
use crate::actors::session_slots::session_slot;
use crate::constants::enemy_properties::K_ENEMY_PROPS_DISABLE_SAMUS_COLL;

/// Synchronous movement and failure operations; all state stays in Session.
pub trait Host:
    crate::actors::access::ActorAccess
    + crate::actors::session_slots::SessionSlots
    + crate::actors::host::ActorMotion
    + crate::effects::SoundEffects
{
    fn probe_horizontal(&mut self, key: u16, amount: i32) -> bool;
    fn invalid_function(&mut self, address: u16) -> !;
}

fn timer_expired<L: Host>(game: &mut L, k: u16) -> bool {
    let me = k as i32;
    let last = session_slot!(game, Etecoon[me].timer) == 1;
    session_slot!(game, Etecoon[me].timer, |old| old.wrapping_sub(1));
    let negative = (session_slot!(game, Etecoon[me].timer) & 0x8000) != 0;
    last || negative
}

fn sound_near_samus<L: Host>(game: &mut L, sound: u8) {
    if !below(game.session().player.x, SOUND_SAMUS_X) {
        game.queue_sound(1, sound, 6);
    }
}

fn set_hop<L: Host>(game: &mut L, me: i32) {
    session_slot!(game, Etecoon[me]; y_speed = HOP_SPEED, y_subspeed = 0);
}

fn set_run<L: Host>(game: &mut L, me: i32, speed: u16) {
    session_slot!(game, Etecoon[me]; x_speed = speed, x_subspeed = 0);
}

fn freeze_if_quake<L: Host>(game: &mut L, k: u16) {
    if game.session().feedback.earthquake_timer != 0 {
        let me = k as i32;
        session_slot!(
            game,
            Etecoon[me].route = (session_slot!(byte game, Etecoon[me].route[0]) as u16) | 0x8000
        );
        session_slot!(game, Etecoon[me].base.instruction_timer, |old| old
            .wrapping_add(128));
    }
}

fn move_horizontally<L: Host>(game: &mut L, k: u16) -> bool {
    let me = k as i32;
    let amount = (((session_slot!(game, Etecoon[me].x_speed) as u32) << 16)
        | session_slot!(game, Etecoon[me].x_subspeed) as u32) as i32;
    game.move_horizontal(k, amount, Slopes::Ignore)
}

fn move_vertically<L: Host>(game: &mut L, k: u16) -> bool {
    let me = k as i32;
    let amount = (((session_slot!(game, Etecoon[me].y_speed) as u32) << 16)
        | session_slot!(game, Etecoon[me].y_subspeed) as u32) as i32;
    if below(session_slot!(game, Etecoon[me].y_speed), 5) {
        let gravity = (u32::from(game.session().player.contact.y_acceleration.whole) << 16)
            | u32::from(game.session().player.contact.y_acceleration.fraction);
        session_slot!(game, Etecoon[me](y_speed, y_subspeed) += gravity);
    }
    game.move_vertical(k, amount)
}

pub fn initialize<L: Host>(game: &mut L) {
    let me = game.session().actors.controls.selected as i32;
    session_slot!(game, Etecoon[me].base.properties, |old| old
        | (K_ENEMY_PROPS_DISABLE_SAMUS_COLL as u16));
    session_slot!(
        game,
        Etecoon[me].base.spritemap_pointer = list::NOTHING_SPRITEMAP
    );
    session_slot!(game, Etecoon[me].base; instruction_timer = 1, timer = 0,
        current_instruction = list::INITIAL);
    session_slot!(game, Etecoon[me]; function = function::INITIAL, timer = 0xffff);
}

pub fn main_ai<L: Host>(game: &mut L) {
    crate::probe::hit("etecoon.main_ai");
    let me = game.session().actors.controls.selected as i32;
    if session_slot!(byte game, Etecoon[me].route[1]) != 0 {
        session_slot!(game, Etecoon[me].route, |old| old.wrapping_sub(256));
    } else {
        let address = session_slot!(game, Etecoon[me].function);
        run_function(game, address);
    }
}

fn run_function<L: Host>(game: &mut L, address: u16) {
    let k = game.session().actors.controls.selected;
    match address {
        function::INITIAL => initial(game, k),
        function::HOPPING_BOTTOM => hopping_bottom(game, k),
        function::RUNNING_LEFT => running_left(game, k),
        function::RUNNING_RIGHT => running_right(game, k),
        function::JUMPING => jumping(game, k),
        function::WALL_JUMP => wall_jump(game, k),
        function::RUN_TO_LEFT_RUN_UP => run_to_left_run_up_point(game, k),
        function::RUN_TO_RIGHT_RUN_UP => run_to_right_run_up_point(game, k),
        function::RUNNING_FOR_TUNNEL_JUMP => running_for_tunnel_jump(game, k),
        function::TUNNEL_JUMP => tunnel_jump(game, k),
        function::RUNNING_THROUGH_TUNNEL => running_through_tunnel(game, k),
        function::FALLING_FROM_TUNNEL_LEDGE => falling_from_tunnel_ledge(game, k),
        function::HOPPING_TOP => hopping_top(game, k),
        function::START_HOP_TOP => start_hop_top(game, k),
        function::HOP_UNTIL_SAMUS_NEAR => hop_until_samus_near(game, k),
        function::RUNNING_FOR_FAILED_TUNNEL_JUMP => running_for_failed_tunnel_jump(game, k),
        function::FAILED_TUNNEL_JUMP => failed_tunnel_jump(game, k),
        function::START_HOP_BOTTOM => start_hop_bottom(game, k),
        function::LOOK_AT_SAMUS => look_at_samus(game, k),
        function::LANDED_FROM_JUMP => landed_from_jump(game, k),
        _ => game.invalid_function(address),
    }
}

pub fn initial<L: Host>(game: &mut L, k: u16) {
    if game.session().room.entry.enemy_interlock == 0 {
        let me = k as i32;
        if (session_slot!(game, Etecoon[me].timer) & 0x8000) == 0 {
            if timer_expired(game, k) {
                session_slot!(
                    game,
                    Etecoon[me].base.current_instruction = list::HOPPING_FACING_LEFT
                );
                session_slot!(game, Etecoon[me]; function = function::START_HOP_BOTTOM, timer = 11);
            }
        } else if game.player_within(k, Axis::Y, 0x80) {
            if (session_slot!(game, Etecoon[me].route) & 3) == 0 {
                game.queue_sound(1, FLEX_SOUND, 15);
            }
            session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::FLEXING);
            session_slot!(game, Etecoon[me].timer = 256);
        }
    }
}

pub fn start_hop_bottom<L: Host>(game: &mut L, k: u16) {
    if !timer_expired(game, k) {
        return;
    }
    let me = k as i32;
    set_hop(game, me);
    session_slot!(game, Etecoon[me].base; current_instruction = (session_slot!(game, Etecoon[me].base.current_instruction)).wrapping_add(2), instruction_timer = 1);
    session_slot!(game, Etecoon[me].function = function::HOPPING_BOTTOM);
    sound_near_samus(game, HOP_SOUND);
}

pub fn hopping_bottom<L: Host>(game: &mut L, k: u16) {
    if move_vertically(game, k) {
        let me = k as i32;
        if (session_slot!(game, Etecoon[me].y_speed) & 0x8000) == 0 {
            if game.player_within(k, Axis::Y, 0x40) && game.player_within(k, Axis::X, LOOK_DISTANCE)
            {
                let samus_right_or_level = matches!(
                    crate::actors::queries::direction_to_player(game),
                    PlayerDirection::Up
                        | PlayerDirection::UpRight
                        | PlayerDirection::Right
                        | PlayerDirection::DownRight
                        | PlayerDirection::Down
                );
                if samus_right_or_level {
                    crate::probe::hit("etecoon.look_right_run_left");
                    session_slot!(
                        game,
                        Etecoon[me].base.current_instruction = list::LOOK_RIGHT_AND_RUN_LEFT
                    );
                    session_slot!(game, Etecoon[me].facing = 0);
                } else {
                    session_slot!(
                        game,
                        Etecoon[me].base.current_instruction = list::LOOK_LEFT_AND_RUN_RIGHT
                    );
                    session_slot!(game, Etecoon[me].facing = 1);
                }
                session_slot!(game, Etecoon[me].timer = 32);
                session_slot!(game, Etecoon[me].base.instruction_timer = 1);
                session_slot!(game, Etecoon[me].function = function::LOOK_AT_SAMUS);
            } else {
                session_slot!(game, Etecoon[me]; timer = 11, function = function::START_HOP_BOTTOM);
                session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::HOPPING_FACING_LEFT);
            }
        } else {
            hit_ceiling(game, me);
        }
    }
}

fn hit_ceiling<L: Host>(game: &mut L, me: i32) {
    session_slot!(game, Etecoon[me]; y_speed = 0, y_subspeed = 0);
    session_slot!(game, Etecoon[me].base; instruction_timer = 3, current_instruction = list::HIT_CEILING);
}

pub fn look_at_samus<L: Host>(game: &mut L, k: u16) {
    if !timer_expired(game, k) {
        return;
    }
    let me = k as i32;
    session_slot!(game, Etecoon[me].base; current_instruction = (session_slot!(game, Etecoon[me].base.current_instruction)).wrapping_add(2), instruction_timer = 1);
    if session_slot!(game, Etecoon[me].facing) != 0 {
        set_run(game, me, RUN_RIGHT_SPEED);
        session_slot!(game, Etecoon[me].function = function::RUNNING_RIGHT);
    } else {
        set_run(game, me, RUN_LEFT_SPEED);
        session_slot!(game, Etecoon[me].function = function::RUNNING_LEFT);
    }
    set_hop(game, me);
}

pub fn running_left<L: Host>(game: &mut L, k: u16) {
    if move_horizontally(game, k) {
        let me = k as i32;
        set_run(game, me, RUN_RIGHT_SPEED);
        session_slot!(game, Etecoon[me].function = function::RUNNING_RIGHT);
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::RUNNING_RIGHT);
        session_slot!(game, Etecoon[me].facing = 1);
    }
}

pub fn running_right<L: Host>(game: &mut L, k: u16) {
    if game.probe_horizontal(k, (32u32 << 16) as i32) {
        let me = k as i32;
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::JUMPING_RIGHT);
        session_slot!(game, Etecoon[me].function = function::JUMPING);
    } else {
        move_horizontally(game, k);
    }
}

pub fn jumping<L: Host>(game: &mut L, k: u16) {
    crate::probe::hit("etecoon.jumping");
    freeze_if_quake(game, k);
    if move_horizontally(game, k) {
        let me = k as i32;
        if session_slot!(game, Etecoon[me].facing) != 0 {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::WALL_JUMP_LEFT_ELIGIBLE
            );
            session_slot!(game, Etecoon[me].facing = 0);
        } else {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::WALL_JUMP_RIGHT_ELIGIBLE
            );
            session_slot!(game, Etecoon[me].facing = 1);
        }
        session_slot!(game, Etecoon[me].base.instruction_timer = 1);
        session_slot!(game, Etecoon[me]; function = function::WALL_JUMP, timer = 8);
        sound_near_samus(game, WALL_JUMP_SOUND);
    } else if move_vertically(game, k) {
        let me = k as i32;
        if session_slot!(game, Etecoon[me].facing) != 0 {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::HOPPING_FACING_LEFT
            );
        } else {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::HOPPING_FACING_RIGHT
            );
        }
        session_slot!(game, Etecoon[me].base.instruction_timer = 1);
        session_slot!(game, Etecoon[me]; timer = 11, function = function::LANDED_FROM_JUMP, y_speed = HOP_SPEED, y_subspeed = 0);
    }
}

pub fn wall_jump<L: Host>(game: &mut L, k: u16) {
    freeze_if_quake(game, k);
    if timer_expired(game, k) {
        let me = k as i32;
        if session_slot!(game, Etecoon[me].facing) != 0 {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::WALL_JUMP_RIGHT
            );
            set_run(game, me, RUN_RIGHT_SPEED);
        } else {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::WALL_JUMP_LEFT
            );
            set_run(game, me, RUN_LEFT_SPEED);
        }
        session_slot!(game, Etecoon[me].base.instruction_timer = 1);
        session_slot!(game, Etecoon[me].function = function::JUMPING);
        set_hop(game, me);
    }
}

pub fn landed_from_jump<L: Host>(game: &mut L, k: u16) {
    if !timer_expired(game, k) {
        return;
    }
    let me = k as i32;
    set_hop(game, me);
    let routes: [fn(&mut L, i32); 3] = [
        run_to_left_run_up,
        run_to_right_run_up,
        hop_at_centre_run_up,
    ];
    routes[session_slot!(byte game, Etecoon[me].route[0]) as usize](game, me);
    session_slot!(game, Etecoon[me].base.instruction_timer = 1);
}

fn run_to_left_run_up<L: Host>(game: &mut L, me: i32) {
    session_slot!(game, Etecoon[me].function = function::RUN_TO_LEFT_RUN_UP);
    session_slot!(
        game,
        Etecoon[me].base.current_instruction = list::RUNNING_LEFT
    );
    set_run(game, me, RUN_LEFT_SPEED);
}

fn run_to_right_run_up<L: Host>(game: &mut L, me: i32) {
    session_slot!(game, Etecoon[me].function = function::RUN_TO_RIGHT_RUN_UP);
    session_slot!(
        game,
        Etecoon[me].base.current_instruction = list::RUNNING_RIGHT
    );
    set_run(game, me, RUN_RIGHT_SPEED);
}

fn hop_at_centre_run_up<L: Host>(game: &mut L, me: i32) {
    session_slot!(game, Etecoon[me].function = function::HOPPING_TOP);
    session_slot!(game, Etecoon[me].base.current_instruction, |old| old
        .wrapping_add(2));
    set_run(game, me, RUN_RIGHT_SPEED);
}

fn start_top_hops<L: Host>(game: &mut L, me: i32) {
    session_slot!(game, Etecoon[me]; timer = 11, function = function::START_HOP_TOP);
    session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::HOPPING_FACING_LEFT);
}

pub fn run_to_left_run_up_point<L: Host>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    let me = k as i32;
    if below(session_slot!(game, Etecoon[me].base.x_pos), LEFT_RUN_UP_X) {
        start_top_hops(game, me);
    }
}

pub fn run_to_right_run_up_point<L: Host>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    let me = k as i32;
    if !below(session_slot!(game, Etecoon[me].base.x_pos), RIGHT_RUN_UP_X) {
        start_top_hops(game, me);
    }
}

pub fn running_for_tunnel_jump<L: Host>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    let me = k as i32;
    if !below(session_slot!(game, Etecoon[me].base.x_pos), RIGHT_RUN_UP_X) {
        session_slot!(game, Etecoon[me]; function = function::TUNNEL_JUMP,
            y_speed = TUNNEL_JUMP_SPEED, y_subspeed = 0);
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::JUMPING_RIGHT);
    }
}

pub fn tunnel_jump<L: Host>(game: &mut L, k: u16) {
    crate::probe::hit("etecoon.tunnel_jump");
    move_horizontally(game, k);
    move_vertically(game, k);
    let me = k as i32;
    if !below(session_slot!(game, Etecoon[me].base.x_pos), TUNNEL_X) {
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::RUNNING_RIGHT);
        session_slot!(
            game,
            Etecoon[me].function = function::RUNNING_THROUGH_TUNNEL
        );
    }
}

pub fn running_through_tunnel<L: Host>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    let me = k as i32;
    if !below(session_slot!(game, Etecoon[me].base.x_pos), TUNNEL_LEDGE_X) {
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::JUMPING_RIGHT);
        session_slot!(
            game,
            Etecoon[me].function = function::FALLING_FROM_TUNNEL_LEDGE
        );
        session_slot!(game, Etecoon[me]; y_speed = 0xffff, y_subspeed = 0);
    }
}

pub fn falling_from_tunnel_ledge<L: Host>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    if move_vertically(game, k) {
        let me = k as i32;
        session_slot!(game, Etecoon[me].timer = 11);
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::HOPPING_FACING_LEFT);
        session_slot!(game, Etecoon[me].function = function::START_HOP_TOP);
    }
}

pub fn hopping_top<L: Host>(game: &mut L, k: u16) {
    freeze_if_quake(game, k);
    if move_vertically(game, k) {
        let me = k as i32;
        if (session_slot!(game, Etecoon[me].y_speed) & 0x8000) == 0 {
            session_slot!(game, Etecoon[me].timer = 11);
            session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::HOPPING_FACING_LEFT);
            if (session_slot!(game, Etecoon[me].route) & 2) != 0
                && below(
                    session_slot!(game, Etecoon[me].base.x_pos),
                    WAIT_FOR_SAMUS_X,
                )
            {
                session_slot!(game, Etecoon[me].function = function::HOP_UNTIL_SAMUS_NEAR);
            } else {
                session_slot!(game, Etecoon[me].function = function::START_HOP_TOP);
            }
        } else {
            hit_ceiling(game, me);
        }
    }
}

pub fn start_hop_top<L: Host>(game: &mut L, k: u16) {
    freeze_if_quake(game, k);
    if timer_expired(game, k) {
        let me = k as i32;
        let hops: u16 = (session_slot!(game, Etecoon[me].facing)).wrapping_add(256);
        session_slot!(game, Etecoon[me].facing = hops);
        if below(hops & 0xff00, TOP_HOPS << 8) || {
            session_slot!(
                game,
                Etecoon[me].facing = session_slot!(byte game, Etecoon[me].facing[0]) as u16
            );
            (session_slot!(game, Etecoon[me].route) & 2) != 0
        } {
            session_slot!(game, Etecoon[me].function = function::HOPPING_TOP);
            session_slot!(game, Etecoon[me].base.current_instruction, |old| old
                .wrapping_add(2));
        } else {
            session_slot!(
                game,
                Etecoon[me].function = function::RUNNING_FOR_FAILED_TUNNEL_JUMP
            );
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::RUNNING_RIGHT
            );
            set_run(game, me, RUN_RIGHT_SPEED);
        }
        set_hop(game, me);
        session_slot!(game, Etecoon[me].base.instruction_timer = 1);
        sound_near_samus(game, HOP_SOUND);
    }
}

pub fn hop_until_samus_near<L: Host>(game: &mut L, k: u16) {
    freeze_if_quake(game, k);
    if timer_expired(game, k) {
        let me = k as i32;
        if game.player_within(k, Axis::Y, 0x40) && game.player_within(k, Axis::X, 0x30) {
            session_slot!(
                game,
                Etecoon[me].base.current_instruction = list::RUNNING_RIGHT
            );
            session_slot!(
                game,
                Etecoon[me].function = function::RUNNING_FOR_TUNNEL_JUMP
            );
        } else {
            set_hop(game, me);
            session_slot!(game, Etecoon[me].base.current_instruction, |old| old
                .wrapping_add(2));
            session_slot!(game, Etecoon[me].function = function::HOPPING_TOP);
            sound_near_samus(game, HOP_SOUND);
        }
        session_slot!(game, Etecoon[me].base.instruction_timer = 1);
    }
}

pub fn running_for_failed_tunnel_jump<L: Host>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    let me = k as i32;
    if !below(session_slot!(game, Etecoon[me].base.x_pos), RIGHT_RUN_UP_X) {
        session_slot!(game, Etecoon[me].function = function::FAILED_TUNNEL_JUMP);
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::JUMPING_RIGHT);
    }
}

pub fn failed_tunnel_jump<L: Host>(game: &mut L, k: u16) {
    move_horizontally(game, k);
    if move_vertically(game, k) {
        let me = k as i32;
        session_slot!(game, Etecoon[me]; x_speed = RUN_LEFT_SPEED, x_subspeed = 0, function = function::RUNNING_LEFT, y_speed = HOP_SPEED, y_subspeed = 0);
        session_slot!(game, Etecoon[me].base; instruction_timer = 1, current_instruction = list::RUNNING_LEFT);
    }
}
