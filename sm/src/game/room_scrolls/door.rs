//! Door camera setup and stepping; RAM remains authoritative between callbacks.
use super::{calculate_blocks, calculate_scrolls, layer2_position, update_map};
use crate::game::room_entry::content::Axis;
use crate::game_state::Game;
use crate::ram_map::word;
use crate::rtl::words;

// Callers retain the native split-word and scrolling tilemap safety contracts.
pub(crate) unsafe fn begin(game: &mut Game) {
    unsafe {
        word!(game.layer1_x_pos = word!(game.door_destination_x_pos));
        word!(game.layer1_y_pos = word!(game.door_destination_y_pos));
        let direction = word!(game.door_direction) & 3;
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
}

unsafe fn prepare_blocks(vertical: bool, delta: u16, game: &mut Game) {
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

pub(crate) unsafe fn fix_up(game: &mut Game) {
    unsafe {
        word!(game.door_transition_frame_counter = 0);
        prepare_blocks(true, 1, game);
        up(game);
    }
}

pub(crate) unsafe fn horizontal(left: bool, game: &mut Game) -> u8 {
    unsafe {
        let frame = word!(game.door_transition_frame_counter);
        let speed = words!(
            game.samus_door_transition_speed,
            samus_door_transition_subspeed
        );
        let speed = if left { -(speed as i32) as u32 } else { speed };
        words!(game.samus_x_pos, samus_x_subpos += speed);
        word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
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
}

unsafe fn first_vertical_frame(offset: u16, block_delta: u16, game: &mut Game) {
    unsafe {
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
}

pub(crate) unsafe fn down(game: &mut Game) -> u8 {
    unsafe {
        let frame = word!(game.door_transition_frame_counter);
        if frame == 0 {
            first_vertical_frame(15, u16::MAX, game);
        } else if frame < 0x39 {
            words!(
                game.samus_y_pos,
                samus_y_subpos += words!(
                    game.samus_door_transition_speed,
                    samus_door_transition_subspeed
                )
            );
            word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
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
}

pub(crate) unsafe fn up(game: &mut Game) -> u8 {
    unsafe {
        let frame = word!(game.door_transition_frame_counter);
        if frame == 0 {
            first_vertical_frame(16, 1, game);
        } else {
            words!(
                game.samus_y_pos,
                samus_y_subpos += -(words!(
                    i32,
                    game.samus_door_transition_speed,
                    samus_door_transition_subspeed
                )) as u32
            );
            word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
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
}
