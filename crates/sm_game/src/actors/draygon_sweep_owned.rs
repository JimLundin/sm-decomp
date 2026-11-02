//! Draygon swoops over the live canonical trajectory and captured/fresh actor selectors.
#![forbid(unsafe_code)]
use super::{ARMS_OUT_INDEX, RIGHT_END_X, SWOOP_START_Y, SWOOP_STEP, SWOOP_TABLE_BYTES};
use crate::actors::draygon_host::{
    ARMS, BODY, Draygon, SIGN, abs16, function, instruction_list as list,
};
use crate::actors::session_slots::session_slot;

pub trait Host: crate::actors::draygon_grab::owned::Host {
    fn body_bubble(&mut self, turret: Option<u16>);
    fn move_body_horizontally(&mut self, right: bool);
    fn swoop_table_overflow(&mut self) -> !;
    /// Off-span trajectory byte, relative to the shared boss allocation start.
    fn imported_swoop_byte(&self, relative: usize) -> u8;
    fn set_imported_swoop_byte(&mut self, relative: usize, value: u8);
}

fn trajectory_byte<H: Host>(game: &H, relative: usize) -> u8 {
    if relative < 0x7dc {
        (*game.session().actors.extended.word(3, relative / 2)).to_le_bytes()[relative & 1]
    } else if relative < 0x800 {
        let index = relative - 0x7dc;
        game.session().enemy_projectiles.slots[index / 2]
            .angle
            .to_le_bytes()[index & 1]
    } else {
        game.imported_swoop_byte(relative)
    }
}
fn set_trajectory_byte<H: Host>(game: &mut H, relative: usize, value: u8) {
    let word = if relative < 0x7dc {
        Some(&mut *game.session_mut().actors.extended.word_mut(3, relative / 2))
    } else if relative < 0x800 {
        Some(&mut game.session_mut().enemy_projectiles.slots[(relative - 0x7dc) / 2].angle)
    } else {
        None
    };
    if let Some(word) = word {
        let mut bytes = word.to_le_bytes();
        bytes[relative & 1] = value;
        *word = u16::from_le_bytes(bytes);
    } else {
        game.set_imported_swoop_byte(relative, value);
    }
}

fn write_y_position<S: Host>(index: u16, value: u16, game: &mut S) {
    let relative = 2 + usize::from(index);
    let bytes = value.to_ne_bytes();
    set_trajectory_byte(game, relative, bytes[0]);
    set_trajectory_byte(game, relative + 1, bytes[1]);
}

fn calculate_swoop_y_positions<L: Host>(game: &mut L) {
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
            game.swoop_table_overflow();
        }
    }
    let distance = abs16(
        session_slot!(game, Draygon[BODY].left_reset_x).wrapping_sub(game.session().player.x),
    );
    let speed = (u32::from(distance) << 16)
        .checked_div(u32::from(index >> 2))
        .unwrap_or(0);
    session_slot!(game, Draygon[BODY](x_speed, x_subspeed) = speed);
    write_y_position(index, session_slot!(game, Draygon[BODY].base.y_pos), game);
    session_slot!(game, Draygon[BODY]; function_timer = index, swoop_duration = index);
    // The swoop's speed, last Y position and index stay in $0E22..$0E26.
    game.session_mut().actors.compatibility.scratch[0] = velocity;
    game.session_mut().actors.compatibility.scratch[1] = height;
    game.session_mut().actors.compatibility.scratch[2] = index;
}

pub fn swoop_setup<L: Host>(right: bool, game: &mut L) {
    if right {
        game.fire_wall_turret();
        calculate_swoop_y_positions(game);
    } else {
        // The duration is $0E26's quarter: the last swoop Y position index or
        // chase X subspeed stored there, not the swoop duration meant.
        let distance = abs16(
            session_slot!(game, Draygon[BODY].right_reset_x).wrapping_sub(game.session().player.x),
        );
        let speed = (u32::from(distance) << 16)
            .checked_div(u32::from(
                game.session().actors.compatibility.scratch[2] >> 2,
            ))
            .unwrap_or(0);
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
        let copied = session_slot!(game, Draygon[BODY].swoop_duration);
        session_slot!(game, Draygon[BODY].function_timer = copied);
    }
}

pub fn swoop_apex<S: Host>(right: bool, game: &mut S) {
    game.fire_wall_turret();
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

pub fn swoop<L: Host>(right: bool, ascending: bool, game: &mut L) {
    let turret = game.fire_wall_turret();
    if !right || !ascending {
        game.body_bubble(turret);
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
    let relative = 2 + usize::from(index);
    let height = u16::from_ne_bytes([
        trajectory_byte(game, relative),
        trajectory_byte(game, relative + 1),
    ]);
    session_slot!(
        game,
        Draygon[game.session().actors.controls.selected as i32]
            .base
            .y_pos = height
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
            game.move_body_horizontally(right);
        }
    } else if next != session_slot!(game, Draygon[BODY].swoop_duration) {
        game.move_body_horizontally(right);
    } else if right {
        session_slot!(game, Draygon[BODY].function_timer = 0);
        session_slot!(
            game,
            Draygon[BODY].function = if game.session().timing.random_number & 1 != 0 {
                function::SWOOP_LEFT_SETUP
            } else {
                function::GOOP_LEFT_SETUP
            }
        );
    } else if game.session().timing.random_number & 1 != 0 {
        session_slot!(game, Draygon[BODY]; function = function::SWOOP_RIGHT_SETUP, function_timer = 0);
        session_slot!(game, Draygon[BODY].base; x_pos = session_slot!(game, Draygon[BODY].left_reset_x),
            y_pos = session_slot!(game, Draygon[BODY].reset_y));
    } else {
        session_slot!(game, Draygon[BODY].function = function::GOOP_RIGHT_SETUP);
        let copied = session_slot!(game, Draygon[BODY].left_reset_x);
        session_slot!(game, Draygon[BODY].base.x_pos = copied);
    }
}
