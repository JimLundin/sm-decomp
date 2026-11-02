//! Dachora state machine over owned actors and player physics.
#![forbid(unsafe_code)]
use super::{
    BLINK_FRAMES, BLINK_SOUND, BUMP_SOUND, CHARGE_FRAMES, CHARGE_SOUND, Dachora, ECHO_INTERVAL,
    ECHO_LIFETIME, ECHOES, Echo, FALL_TOP_SPEED, FASTER_RUN_STEP, LAND_SOUND, LEFT_END_X,
    RIGHT_END_X, RUN_ACCELERATION, SHINE_SOUND, SHINESPARK_SOUND, SHINESPARK_TOP_SPEED, TOP_SPEED,
    WAKE_DISTANCE, function, instruction_list as list,
};
use crate::actors::host::{Axis, Slopes, below};
use crate::actors::session_slots::session_slot;
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_DISABLE_SAMUS_COLL, K_ENEMY_PROPS_INVISIBLE,
};

/// Palette effects execute synchronously at the original call sites.
pub trait Host:
    crate::actors::access::ActorAccess
    + crate::actors::session_slots::SessionSlots
    + crate::actors::host::ActorMotion
    + crate::effects::SoundEffects
{
    fn frame(&self) -> u16;
    fn load_palette(&mut self, source: u16, key: u16);
    fn load_shine_palette(&mut self, key: u16);
    fn dachora_palette(&mut self, charging: bool, index: isize) -> u16;
    fn invalid_function(&mut self, address: u16) -> !;
}

fn gravity<L: Host>(game: &L) -> u32 {
    (u32::from(game.session().player.contact.y_acceleration.whole) << 16)
        | u32::from(game.session().player.contact.y_acceleration.fraction)
}

pub fn initialize<L: Host>(game: &mut L) {
    let me = game.session().actors.controls.selected as i32;
    session_slot!(game, Dachora[me].base.properties, |old| old
        | (K_ENEMY_PROPS_DISABLE_SAMUS_COLL as u16));
    session_slot!(
        game,
        Dachora[me].base.spritemap_pointer = list::NOTHING_SPRITEMAP
    );
    session_slot!(game, Dachora[me].base; instruction_timer = 1, timer = 0);
    let kind: i16 = session_slot!(game, Dachora[me].facing_right) as i16;
    if kind < 0 {
        if (kind as u16 & 1) != 0 {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::ECHO_FACING_RIGHT
            );
        } else {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::ECHO_FACING_LEFT
            );
        }
        session_slot!(game, Dachora[me].function = function::ECHO);
    } else {
        if kind != 0 {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::IDLING_FACING_RIGHT
            );
        } else {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::IDLING_FACING_LEFT
            );
        }
        session_slot!(game, Dachora[me].function = function::WAIT_FOR_SAMUS);
    }
}

pub fn main_ai<L: Host>(game: &mut L) {
    let me = game.session().actors.controls.selected as i32;
    let k = game.session().actors.controls.selected;
    let address = session_slot!(game, Dachora[me].function);
    match address {
        function::WAIT_FOR_SAMUS => wait_for_samus(game, k),
        function::START_RUNNING => start_running(game, k),
        function::RUNNING_LEFT => running_left(game, k),
        function::RUNNING_RIGHT => running_right(game, k),
        function::ACTIVATE_SHINESPARK => activate_shinespark(game, k),
        function::SHINESPARKING => shinesparking(game, k),
        function::FALLING => falling(game, k),
        function::ECHO => echo(game, k),
        _ => game.invalid_function(address),
    }
}

pub fn wait_for_samus<L: Host>(game: &mut L, k: u16) {
    game.move_vertical(k, (1u32 << 16) as i32);
    if game.player_within(k, Axis::Y, 0x40) && game.player_within(k, Axis::X, WAKE_DISTANCE) {
        let me = k as i32;
        if session_slot!(game, Dachora[me].facing_right) != 0 {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::BLINKING_FACING_RIGHT
            );
        } else {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::BLINKING_FACING_LEFT
            );
        }
        session_slot!(game, Dachora[me].base.instruction_timer = 1);
        session_slot!(game, Dachora[me]; function = function::START_RUNNING, speed = BLINK_FRAMES);
        game.queue_sound(1, BLINK_SOUND, 15);
    }
}

pub fn start_running<L: Host>(game: &mut L, k: u16) {
    let me = k as i32;
    let left = session_slot!(game, Dachora[me].speed);
    let copied = session_slot!(game, Dachora[me].speed).wrapping_sub(1);
    session_slot!(game, Dachora[me].speed = copied);
    if left == 1 {
        if session_slot!(game, Dachora[me].facing_right) != 0 {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::RUNNING_RIGHT
            );
            session_slot!(game, Dachora[me].function = function::RUNNING_RIGHT);
        } else {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::RUNNING_LEFT
            );
            session_slot!(game, Dachora[me].function = function::RUNNING_LEFT);
        }
        session_slot!(game, Dachora[me].base.instruction_timer = 1);
        session_slot!(game, Dachora[me].shine = 1);
    }
}

pub fn running_left<L: Host>(game: &mut L, k: u16) {
    let amount: i32 = -accelerate(game, k);
    let me = k as i32;
    if game.move_horizontal(k, amount, Slopes::Ignore) || {
        game.snap_to_slopes(k);
        below(session_slot!(game, Dachora[me].base.x_pos), LEFT_END_X)
    } {
        session_slot!(
            game,
            Dachora[me].base.current_instruction = list::RUNNING_RIGHT
        );
        session_slot!(game, Dachora[me].function = function::RUNNING_RIGHT);
        session_slot!(game, Dachora[me]; shine = 1, facing_right = 1);
        session_slot!(game, Dachora[me].base.instruction_timer = 1);
        session_slot!(game, Dachora[me]; speed = 0, subspeed = 0);
        game.load_palette(list::PALETTE, k);
    }
}

pub fn running_right<L: Host>(game: &mut L, k: u16) {
    let amount: i32 = accelerate(game, k);
    let me = k as i32;
    if game.move_horizontal(k, amount, Slopes::Ignore) {
        game.queue_sound(1, BUMP_SOUND, 15);
        session_slot!(
            game,
            Dachora[me].base.current_instruction = list::RUNNING_LEFT
        );
        session_slot!(game, Dachora[me].function = function::RUNNING_LEFT);
        session_slot!(game, Dachora[me]; facing_right = 0, speed = 0);
        game.load_palette(list::PALETTE, k);
    } else {
        game.snap_to_slopes(k);
        if !below(session_slot!(game, Dachora[me].base.x_pos), RIGHT_END_X) {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::CHARGE_SHINESPARK_FACING_RIGHT
            );
            session_slot!(game, Dachora[me]; function = function::ACTIVATE_SHINESPARK,
                speed = CHARGE_FRAMES);
            session_slot!(game, Dachora[me].base.y_pos, |old| old.wrapping_add(8));
            game.queue_sound(1, CHARGE_SOUND, 6);
        } else {
            return;
        }
    }
    session_slot!(game, Dachora[me]; shine = 0, subspeed = 0);
    session_slot!(game, Dachora[me].base.instruction_timer = 1);
}

pub fn activate_shinespark<L: Host>(game: &mut L, k: u16) {
    game.load_shine_palette(k);
    let me = k as i32;
    let left = session_slot!(game, Dachora[me].speed);
    let copied = session_slot!(game, Dachora[me].speed).wrapping_sub(1);
    session_slot!(game, Dachora[me].speed = copied);
    if left == 1 {
        session_slot!(game, Dachora[me].base; current_instruction = (session_slot!(game, Dachora[me].base.current_instruction)).wrapping_add(2), instruction_timer = 1);
        session_slot!(game, Dachora[me].function = function::SHINESPARKING);
        let first = me + ECHOES[0];
        session_slot!(game, Echo[first]; spawn_timer = 0, lifetime = 0);
        for echo in &ECHOES[1..] {
            session_slot!(game, Echo[me + echo].lifetime = 0);
        }
        session_slot!(game, Dachora[me]; acceleration = 0, subacceleration = 0);
        session_slot!(game, Dachora[me].base.y_pos, |old| old.wrapping_sub(8));
        game.queue_sound(1, SHINESPARK_SOUND, 6);
        let echo_list = if session_slot!(game, Dachora[me].facing_right) != 0 {
            list::ECHO_FACING_RIGHT
        } else {
            list::ECHO_FACING_LEFT
        };
        for echo in ECHOES {
            session_slot!(game, Echo[me + echo].base.current_instruction = echo_list);
        }
        for echo in ECHOES {
            session_slot!(game, Echo[me + echo].base.instruction_timer = 1);
        }
    }
}

pub fn shinesparking<L: Host>(game: &mut L, k: u16) {
    game.load_shine_palette(k);
    update_echoes(game, k);
    let me = k as i32;
    let pull = gravity(game);
    session_slot!(game, Dachora[me](acceleration, subacceleration) += pull);
    let acceleration = ((session_slot!(game, Dachora[me].acceleration) as u32) << 16)
        | session_slot!(game, Dachora[me].subacceleration) as u32;
    session_slot!(game, Dachora[me](speed, subspeed) += acceleration);
    let mut speed: u16 = session_slot!(game, Dachora[me].speed);
    if !below(speed, SHINESPARK_TOP_SPEED) {
        speed = SHINESPARK_TOP_SPEED;
    }
    let amount = (((speed as u32) << 16) | session_slot!(game, Dachora[me].subspeed) as u32) as i32;
    if game.move_vertical(k, -amount) {
        if session_slot!(game, Dachora[me].facing_right) != 0 {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::FALLING_FACING_LEFT
            );
            session_slot!(game, Dachora[me].facing_right = 0);
        } else {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::FALLING_FACING_RIGHT
            );
            session_slot!(game, Dachora[me].facing_right = 1);
        }
        session_slot!(game, Dachora[me].function = function::FALLING);
        session_slot!(game, Dachora[me].base.instruction_timer = 1);
        session_slot!(game, Dachora[me]; speed = 0, subspeed = 0, shine = 0);
        game.load_palette(list::PALETTE, k);
        game.queue_sound(1, LAND_SOUND, 6);
    }
}

pub fn falling<L: Host>(game: &mut L, k: u16) {
    let me = k as i32;
    let pull = gravity(game);
    session_slot!(game, Dachora[me](speed, subspeed) += pull);
    let mut amount = (((session_slot!(game, Dachora[me].speed) as u32) << 16)
        | session_slot!(game, Dachora[me].subspeed) as u32) as i32;
    if !below((amount >> 16) as u16, FALL_TOP_SPEED) {
        amount = (FALL_TOP_SPEED as i32) << 16;
    }
    if game.move_vertical(k, amount) {
        if session_slot!(game, Dachora[me].facing_right) != 0 {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::RUNNING_RIGHT
            );
            session_slot!(game, Dachora[me].function = function::RUNNING_RIGHT);
        } else {
            session_slot!(
                game,
                Dachora[me].base.current_instruction = list::RUNNING_LEFT
            );
            session_slot!(game, Dachora[me].function = function::RUNNING_LEFT);
        }
        session_slot!(game, Dachora[me].base.instruction_timer = 1);
        session_slot!(game, Dachora[me]; speed = 0, subspeed = 0);
    }
}

pub fn echo<L: Host>(game: &mut L, k: u16) {
    let me = k as i32;
    let lifetime: u16 = session_slot!(game, Echo[me].lifetime);
    if lifetime != 0 {
        session_slot!(game, Echo[me].lifetime = lifetime.wrapping_sub(1));
        let odd_frame = (game.frame() & 1) != 0;
        // Odd echo slots show on odd frames, the others on even frames.
        let keep_hidden = if (k & 0x40) != 0 {
            !odd_frame
        } else {
            odd_frame
        };
        if !keep_hidden {
            session_slot!(game, Echo[me].base.properties, |old| (old as i32
                & !(K_ENEMY_PROPS_INVISIBLE as i32))
                as u16);
            return;
        }
    }
    session_slot!(game, Echo[me].base.properties, |old| old
        | (K_ENEMY_PROPS_INVISIBLE as u16));
}

fn accelerate<L: Host>(game: &mut L, k: u16) -> i32 {
    crate::probe::hit("dachora.accelerate");
    let me = k as i32;
    if !below(session_slot!(game, Dachora[me].speed), TOP_SPEED) {
        if session_slot!(game, Dachora[me].shine) == 1 {
            game.queue_sound(1, SHINE_SOUND, 6);
        }
        let shine: u16 = (session_slot!(game, Dachora[me].shine)).wrapping_sub(1);
        session_slot!(game, Dachora[me].shine = shine);
        if shine as u8 == 0 {
            let palette = game.dachora_palette(
                false,
                session_slot!(byte game, Dachora[me].shine[1]) as isize,
            );
            game.load_palette(palette, k);
            let next: u16 = (session_slot!(game, Dachora[me].shine)).wrapping_add(0x110);
            session_slot!(game, Dachora[me].shine = next);
            if !below(next, 0x410) {
                session_slot!(game, Dachora[me].shine = 0x310);
            }
        }
    }
    game.move_vertical(k, (1u32 << 16) as i32);
    if !below(session_slot!(game, Dachora[me].speed), TOP_SPEED)
        && (session_slot!(game, Dachora[me].subspeed) as i16) >= 0
    {
        session_slot!(game, Dachora[me]; speed = TOP_SPEED, subspeed = 0);
        return ((TOP_SPEED as u32) << 16) as i32;
    }
    session_slot!(game, Dachora[me](speed, subspeed) += RUN_ACCELERATION);
    if (session_slot!(game, Dachora[me].speed) == 4 || session_slot!(game, Dachora[me].speed) == 8)
        && session_slot!(game, Dachora[me].subspeed) == 0
    {
        session_slot!(game, Dachora[me].base.current_instruction, |old| old
            .wrapping_add(FASTER_RUN_STEP));
    }
    (((session_slot!(game, Dachora[me].speed) as u32) << 16)
        | session_slot!(game, Dachora[me].subspeed) as u32) as i32
}

fn update_echoes<L: Host>(game: &mut L, k: u16) {
    let me = k as i32;
    let first = me + ECHOES[0];
    let timer: u16 = session_slot!(game, Echo[first].spawn_timer);
    if timer != 0 {
        session_slot!(game, Echo[first].spawn_timer = timer.wrapping_sub(1));
        return;
    }
    session_slot!(game, Echo[first].spawn_timer = ECHO_INTERVAL);
    for echo in ECHOES {
        let slot = me + echo;
        if session_slot!(game, Echo[slot].lifetime) == 0 {
            let copied = session_slot!(game, Dachora[me].base.x_pos);
            session_slot!(game, Echo[slot].base.x_pos = copied);
            let copied = session_slot!(game, Dachora[me].base.y_pos);
            session_slot!(game, Echo[slot].base.y_pos = copied);
            session_slot!(game, Echo[slot].lifetime = ECHO_LIFETIME);
            return;
        }
    }
}
