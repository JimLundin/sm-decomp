//! Door camera setup and stepping; RAM remains authoritative between callbacks.
#![forbid(unsafe_code)]
use super::Host;
use super::{add_native_motion, calculate_blocks, calculate_scrolls, layer2_position, update_map};
use crate::ram::{word, words as fields};
use crate::ram_words as words;
use crate::room_entry::content::Axis;

// Door steps retain native split-word and scrolling tilemap publication order.
pub fn begin<H: Host + crate::runtime_access::RuntimeAccess + ?Sized>(game: &mut H) {
    word!(game.layer1_x_pos = word!(game.door_destination_x_pos));
    word!(game.layer1_y_pos = word!(game.door_destination_y_pos));
    let direction = {
        use crate::session_access::SessionAccess;
        game.runtime().session().room.entry.direction & 3
    };
    let backwards = matches!(direction, 0 | 2);
    let block_delta = if backwards { u16::MAX } else { 1 };
    layer2_position(Axis::X, game);
    match direction {
        0 | 1 => {
            let offset = if backwards { 0xff00 } else { 256 };
            word!(game.layer2_x_pos += offset);
            layer2_position(Axis::Y, game);
            word!(game.layer1_x_pos += offset);
        }
        2 => {
            layer2_position(Axis::Y, game);
            word!(game.layer2_y_pos -= 224);
            word!(game.layer1_y_pos -= 224);
        }
        _ => {
            let previous_y = word!(game.layer1_y_pos);
            word!(game.layer1_y_pos += 31);
            layer2_position(Axis::Y, game);
            word!(game.layer2_y_pos += 224);
            word!(game.layer1_y_pos = previous_y.wrapping_add(256));
        }
    }
    word!(game.bg1_x_offset = word!(game.reg_BG1HOFS).wrapping_sub(word!(game.layer1_x_pos)));
    word!(game.bg1_y_offset = word!(game.reg_BG1VOFS).wrapping_sub(word!(game.layer1_y_pos)));
    word!(game.bg2_x_scroll = word!(game.reg_BG2HOFS).wrapping_sub(word!(game.layer1_x_pos)));
    word!(game.bg2_y_scroll = word!(game.reg_BG2VOFS).wrapping_sub(word!(game.layer1_y_pos)));
    if direction == 3 {
        word!(game.door_destination_y_pos += 32);
    }
    prepare_blocks(direction >= 2, block_delta, game);
    match direction {
        0 | 1 => {
            horizontal(direction == 1, game);
        }
        2 => {
            down(game);
        }
        _ => {
            word!(game.layer1_y_pos -= 1);
            up(game);
        }
    }
}

fn prepare_blocks<H: Host + ?Sized>(vertical: bool, delta: u16, game: &mut H) {
    {
        calculate_blocks(game);
    }
    word!(game.previous_layer1_x_block = word!(game.layer1_x_block));
    word!(game.previous_layer2_x_block = word!(game.layer2_x_block));
    word!(game.previous_layer1_y_block = word!(game.layer1_y_block));
    word!(game.previous_layer2_y_block = word!(game.layer2_y_block));
    if vertical {
        word!(game.previous_layer1_y_block += delta);
        word!(game.previous_layer2_y_block += delta);
    } else {
        word!(game.previous_layer1_x_block += delta);
        word!(game.previous_layer2_x_block += delta);
    }
}

pub fn fix_up<H: Host + ?Sized>(game: &mut H) {
    word!(game.door_transition_frame_counter = 0);
    prepare_blocks(true, 1, game);
    up(game);
}

pub fn horizontal<H: Host + ?Sized>(left: bool, game: &mut H) -> u8 {
    let frame = word!(game.door_transition_frame_counter);
    let speed = words!(
        game.samus_door_transition_speed,
        samus_door_transition_subspeed
    );
    let speed = if left { -(speed as i32) as u32 } else { speed };
    add_native_motion(fields::samus_x_pos, fields::samus_x_subpos, speed, game);
    game.set_previous_player_x(game.player_x());
    let delta = if left { 0xfffc } else { 4 };
    word!(game.layer1_x_pos += delta);
    word!(game.layer2_x_pos += delta);
    update_map(game);
    word!(game.door_transition_frame_counter = frame.wrapping_add(1));
    if frame == 63 && !left {
        update_map(game);
    }
    (frame == 63) as u8
}

fn first_vertical_frame<H: Host + ?Sized>(offset: u16, block_delta: u16, game: &mut H) {
    let bg1 = word!(game.reg_BG1VOFS);
    let bg2 = word!(game.reg_BG2VOFS);
    let layer1 = word!(game.layer1_y_pos);
    word!(game.layer1_y_pos -= offset);
    let layer2 = word!(game.layer2_y_pos);
    word!(game.layer2_y_pos -= offset);
    prepare_blocks(true, block_delta, game);
    update_map(game);
    word! { game; layer2_y_pos = layer2; layer1_y_pos = layer1;
    reg_BG2VOFS = bg2; reg_BG1VOFS = bg1; }
}

pub fn down<H: Host + ?Sized>(game: &mut H) -> u8 {
    let frame = word!(game.door_transition_frame_counter);
    if frame == 0 {
        first_vertical_frame(15, u16::MAX, game);
    } else if frame < 0x39 {
        add_native_motion(
            fields::samus_y_pos,
            fields::samus_y_subpos,
            words!(
                game.samus_door_transition_speed,
                samus_door_transition_subspeed
            ),
            game,
        );
        game.set_previous_player_y(game.player_y());
        word!(game.layer1_y_pos += 4);
        word!(game.layer2_y_pos += 4);
        update_map(game);
    }
    word!(game.door_transition_frame_counter = frame.wrapping_add(1));
    if frame.wrapping_add(1) < 0x39 {
        return 0;
    }
    update_map(game);
    1
}

pub fn up<H: Host + ?Sized>(game: &mut H) -> u8 {
    let frame = word!(game.door_transition_frame_counter);
    if frame == 0 {
        first_vertical_frame(16, 1, game);
    } else {
        add_native_motion(
            fields::samus_y_pos,
            fields::samus_y_subpos,
            -(words!(
                i32,
                game.samus_door_transition_speed,
                samus_door_transition_subspeed
            )) as u32,
            game,
        );
        game.set_previous_player_y(game.player_y());
        word!(game.layer1_y_pos -= 4);
        word!(game.layer2_y_pos -= 4);
        if word!(game.door_transition_frame_counter) >= 5 {
            update_map(game);
        } else {
            calculate_scrolls(game);
        }
    }
    word!(game.door_transition_frame_counter = frame.wrapping_add(1));
    (frame == 56) as u8
}
