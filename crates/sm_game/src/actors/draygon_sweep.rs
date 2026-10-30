//! Draygon's swoops: the swoop Y positions, setups, descents, apexes and
//! ascents, mirrored for each side (bank $A5:87F4-$A5:8B09).
//! The native table remains authoritative and may alias other compatibility state.
#![forbid(unsafe_code)]
use crate::actors::boss_lease::BossState;
use crate::actors::draygon_host::Lease;
use crate::actors::host::BossEffects as _;
use crate::actors::session_slots::session_slot;

use crate::actors::draygon_flight::{body_bubble, fire_wall_turret, move_horizontally};
use crate::actors::draygon_host::{
    ARMS, BODY, Draygon, SIGN, abs16, function, instruction_list as list,
};
use crate::ram::word;

/// $7E:9002: the swoop Y positions, one every 4 bytes, up to 800h bytes.
const SWOOP_Y_POSITIONS: usize = 0x9002;
const SWOOP_STEP: u16 = 4;
const SWOOP_TABLE_BYTES: u16 = 0x800;
/// Swoops start descending from Y 180h.
const SWOOP_START_Y: u16 = 0x180;
/// The arms reach out at swoop index 68h.
const ARMS_OUT_INDEX: u16 = 0x68;
/// The right swoop ends at X 2A0h.
const RIGHT_END_X: i32 = 0x2a0;

fn write_y_position<S: BossState>(index: u16, value: u16, game: &mut S) {
    let at = SWOOP_Y_POSITIONS + usize::from(index);
    let bytes = value.to_ne_bytes();
    game.set_slot_byte(at, bytes[0]);
    game.set_slot_byte(at + 1, bytes[1]);
}
/// Calculate Draygon swoop Y positions ($A5:8817): fall with the swoop Y
/// acceleration to the reset Y, then set the X speed that reaches Samus.
fn calculate_swoop_y_positions<L: Lease>(game: &mut L) {
    let (mut height, mut velocity, mut index) = (SWOOP_START_Y, 0u16, 0u16);
    loop {
        velocity = velocity.wrapping_add(session_slot!(game, Draygon[BODY].swoop_y_acceleration));
        let next = height.wrapping_sub(u16::from(velocity.to_ne_bytes()[1]));
        if (next as i16).wrapping_sub(session_slot!(game, Draygon[BODY].reset_y) as i16) < 0 {
            break;
        }
        height = next;
        write_y_position(index, next, game);
        index = index.wrapping_add(SWOOP_STEP);
        if index.wrapping_sub(SWOOP_TABLE_BYTES) & SIGN == 0 {
            game.foreign(|game| game.unreachable());
        }
    }
    let distance =
        abs16(session_slot!(game, Draygon[BODY].left_reset_x).wrapping_sub(game.player_x()));
    let speed = (u32::from(distance) << 16)
        .checked_div(u32::from(index >> 2))
        .unwrap_or(0);
    session_slot!(game, Draygon[BODY](x_speed, x_subspeed) = speed);
    write_y_position(index, session_slot!(game, Draygon[BODY].base.y_pos), game);
    session_slot!(game, Draygon[BODY]; function_timer = index, swoop_duration = index);
}
/// Draygon body function - swoop right ($A5:87F4) or left ($A5:89B3) - setup.
/// Was `begin`.
pub fn swoop_setup<L: Lease>(right: bool, game: &mut L) {
    if right {
        fire_wall_turret(game);
        calculate_swoop_y_positions(game);
    } else {
        // The bank retains the original diagnostic before this zero-divisor setup.
        let distance =
            abs16(session_slot!(game, Draygon[BODY].right_reset_x).wrapping_sub(game.player_x()));
        let speed = (u32::from(distance) << 16).checked_div(0).unwrap_or(0);
        session_slot!(game, Draygon[BODY](x_speed, x_subspeed) = speed);
    }
    session_slot!(
        game,
        Draygon[BODY].function = if right {
            function::SWOOP_RIGHT_DESCENDING
        } else {
            function::SWOOP_LEFT_DESCENDING
        }
    );
    session_slot!(game, Draygon[BODY].base; current_instruction = if right { list::BODY_RIGHT_RESET } else { list::BODY_LEFT_RESET }, instruction_timer = 1);
    session_slot!(game, Draygon[BODY].facing_right = u16::from(right));
    if !right {
        session_slot!(
            game,
            Draygon[BODY].function_timer = session_slot!(game, Draygon[BODY].swoop_duration)
        );
    }
}
/// Draygon body function - swoop right ($A5:8922) or left ($A5:8A50) - apex:
/// set the X speed that reaches the far side in the swoop's duration. Was `turn`.
pub fn swoop_apex<S: BossState>(right: bool, game: &mut S) {
    fire_wall_turret(game);
    let count = u32::from(session_slot!(game, Draygon[BODY].swoop_duration) >> 2);
    let x = session_slot!(game, Draygon[BODY].base.x_pos) as i32;
    let distance = if right {
        RIGHT_END_X - x
    } else {
        x - session_slot!(game, Draygon[BODY].left_reset_x) as i32
    };
    let speed = ((distance << 16) as u32).checked_div(count).unwrap_or(0);
    if right {
        session_slot!(game, Draygon[BODY](x_speed, x_subspeed) = speed);
    } else {
        // The left apex writes high then low; other speed setters write low first.
        session_slot!(game, Draygon[BODY]; x_speed = (speed >> 16) as u16, x_subspeed = speed as u16);
    }
    session_slot!(
        game,
        Draygon[BODY].function = if right {
            function::SWOOP_RIGHT_ASCENDING
        } else {
            function::SWOOP_LEFT_ASCENDING
        }
    );
}
/// Draygon body function - swoop right or left - descending ($A5:88B1,
/// $A5:8A00) or, with `ascending`, ascending ($A5:8951, $A5:8A90): follow the
/// swoop Y positions, then pick the next attack. Was `advance`.
pub fn swoop<L: Lease>(right: bool, ascending: bool, game: &mut L) {
    fire_wall_turret(game);
    if !right || !ascending {
        body_bubble(game);
    }
    let index = session_slot!(game, Draygon[BODY].function_timer);
    if index == ARMS_OUT_INDEX {
        let instruction = match (right, ascending) {
            (true, false) => list::ARMS_RIGHT_NEAR_SWOOP_APEX,
            (true, true) => list::ARMS_RIGHT_IDLE,
            (false, false) => list::ARMS_LEFT_NEAR_SWOOP_APEX,
            (false, true) => list::ARMS_LEFT_IDLE,
        };
        session_slot!(game, Draygon[ARMS].base; current_instruction = instruction, instruction_timer = 1);
    }
    let at = SWOOP_Y_POSITIONS + usize::from(index);
    let height = u16::from_ne_bytes([game.slot_byte(at), game.slot_byte(at + 1)]);
    session_slot!(
        game,
        Draygon[word!(game.cur_enemy_index) as i32].base.y_pos = height
    );
    // A broad selected slot can alias the body's index: reread after its Y store.
    let index = session_slot!(game, Draygon[BODY].function_timer);
    let next = if ascending {
        index.wrapping_add(SWOOP_STEP)
    } else {
        index.wrapping_sub(SWOOP_STEP)
    };
    session_slot!(game, Draygon[BODY].function_timer = next);
    if !ascending {
        if next == 0 {
            session_slot!(
                game,
                Draygon[BODY].function = if right {
                    function::SWOOP_RIGHT_APEX
                } else {
                    function::SWOOP_LEFT_APEX
                }
            );
        } else {
            move_horizontally(right, game);
        }
    } else if next != session_slot!(game, Draygon[BODY].swoop_duration) {
        move_horizontally(right, game);
    } else if right {
        session_slot!(game, Draygon[BODY].function_timer = 0);
        session_slot!(
            game,
            Draygon[BODY].function = if word!(game.random_number) & 1 != 0 {
                function::SWOOP_LEFT_SETUP
            } else {
                function::GOOP_LEFT_SETUP
            }
        );
    } else if word!(game.random_number) & 1 != 0 {
        session_slot!(game, Draygon[BODY]; function = function::SWOOP_RIGHT_SETUP, function_timer = 0);
        session_slot!(game, Draygon[BODY].base; x_pos = session_slot!(game, Draygon[BODY].left_reset_x),
            y_pos = session_slot!(game, Draygon[BODY].reset_y));
    } else {
        session_slot!(game, Draygon[BODY].function = function::GOOP_RIGHT_SETUP);
        session_slot!(
            game,
            Draygon[BODY].base.x_pos = session_slot!(game, Draygon[BODY].left_reset_x)
        );
    }
}
