//! Camera limits and scroll-cell rules; live callers retain RAM authority.
#![forbid(unsafe_code)]
use crate::game::room_entry::content::Axis;
use crate::game_state::Game;
use crate::ram_map::{word, words as W};

// Multiplication uses the width's low byte. A column may include a full-word
// row stride; it is added afterward, with the original word wrapping.
fn camera_scroll_value(column: u16, row: u8, game: &Game) -> u8 {
    let index =
        (u16::from(row) * u16::from(word!(game.room_width_in_scrolls) as u8)).wrapping_add(column);
    game.g_ram[0xcd20 + usize::from(index)]
}

pub(super) fn auto_x(game: &mut Game) {
    crate::verify_probe::hit("camera.auto_x");
    if word!(game.time_is_frozen_flag) != 0 {
        return;
    }
    let previous = word!(game.layer1_x_pos);
    if previous & 0x8000 != 0 {
        word!(game.layer1_x_pos = 0);
    }
    let limit = word!(game.room_width_in_scrolls)
        .wrapping_sub(1)
        .swap_bytes();
    if limit < word!(game.layer1_x_pos) {
        word!(game.layer1_x_pos = limit);
    }
    let position = word!(game.layer1_x_pos);
    let row = (word!(game.layer1_y_pos).wrapping_add(128) >> 8) as u8;
    let column = position >> 8;
    let next = if camera_scroll_value(column, row, game) != 0 {
        if camera_scroll_value(column + 1, row, game) != 0 {
            return;
        }
        let boundary = position & 0xff00;
        let next = previous
            .wrapping_sub(word!(game.absolute_moved_last_frame_x))
            .wrapping_sub(2);
        if (next.wrapping_sub(boundary) as i16) < 0 {
            boundary
        } else if camera_scroll_value(next >> 8, row, game) != 0 {
            next
        } else {
            (next & 0xff00).wrapping_add(256)
        }
    } else {
        let boundary = (position & 0xff00).wrapping_add(256);
        let next = previous
            .wrapping_add(word!(game.absolute_moved_last_frame_x))
            .wrapping_add(2);
        if next >= boundary {
            boundary
        } else if camera_scroll_value(
            u16::from((next >> 8) as u8).wrapping_add(1) & 0xff,
            row,
            game,
        ) != 0
        {
            next
        } else {
            next & 0xff00
        }
    };
    word!(game.layer1_x_pos = next);
}
pub(super) fn trigger_right(game: &mut Game) {
    crate::verify_probe::hit("camera.trigger_right");
    let previous = word!(game.layer1_x_pos);
    if (word!(game.ideal_layer1_xpos).wrapping_sub(word!(game.layer1_x_pos)) as i16) < 0 {
        word! {game; layer1_x_pos = word!(game.ideal_layer1_xpos); layer1_x_subpos = 0; }
    }
    let limit = word!(game.room_width_in_scrolls)
        .wrapping_sub(1)
        .swap_bytes();
    let position = word!(game.layer1_x_pos);
    if limit < position {
        word!(game.layer1_x_pos = limit);
    } else if camera_scroll_value(
        (position >> 8) + 1,
        (word!(game.layer1_y_pos).wrapping_add(128) >> 8) as u8,
        game,
    ) == 0
    {
        let boundary = position & 0xff00;
        word!(
            game.layer1_x_pos =
                retreat_to_boundary(previous, word!(game.absolute_moved_last_frame_x), boundary)
        );
    }
}
pub(super) fn trigger_left(game: &mut Game) {
    crate::verify_probe::hit("camera.trigger_left");
    let previous = word!(game.layer1_x_pos);
    if (word!(game.layer1_x_pos).wrapping_sub(word!(game.ideal_layer1_xpos)) as i16) < 0 {
        word! {game; layer1_x_pos = word!(game.ideal_layer1_xpos); layer1_x_subpos = 0; }
    }
    let position = word!(game.layer1_x_pos);
    if position & 0x8000 != 0 {
        word!(game.layer1_x_pos = 0);
    } else if camera_scroll_value(
        position >> 8,
        (word!(game.layer1_y_pos).wrapping_add(128) >> 8) as u8,
        game,
    ) == 0
    {
        let boundary = (position & 0xff00).wrapping_add(256);
        let next = previous
            .wrapping_add(word!(game.absolute_moved_last_frame_x))
            .wrapping_add(2);
        word!(game.layer1_x_pos = next.min(boundary));
    }
}
pub(super) fn auto_y(game: &mut Game) {
    crate::verify_probe::hit("camera.auto_y");
    if word!(game.time_is_frozen_flag) != 0 {
        return;
    }
    let column = word!(game.layer1_x_pos).wrapping_add(128) >> 8;
    let padding = if camera_scroll_value(column, (word!(game.layer1_y_pos) >> 8) as u8, game) == 1 {
        0u16
    } else {
        31
    };
    let previous = word!(game.layer1_y_pos);
    if previous & 0x8000 != 0 {
        word!(game.layer1_y_pos = 0);
    }
    let limit = word!(game.room_height_in_scrolls)
        .wrapping_sub(1)
        .swap_bytes()
        .wrapping_add(padding);
    if limit < word!(game.layer1_y_pos) {
        word!(game.layer1_y_pos = limit);
    }
    let position = word!(game.layer1_y_pos);
    let row = (position >> 8) as u8;
    if camera_scroll_value(column, row, game) == 0 {
        let boundary = (position & 0xff00).wrapping_add(256);
        let next = previous
            .wrapping_add(word!(game.absolute_moved_last_frame_y))
            .wrapping_add(2);
        let next = if next >= boundary {
            boundary
        } else if camera_scroll_value(column, ((next >> 8) as u8).wrapping_add(1), game) != 0 {
            next
        } else {
            next & 0xff00
        };
        word!(game.layer1_y_pos = next);
    } else if camera_scroll_value(
        column.wrapping_add(word!(game.room_width_in_scrolls)),
        row,
        game,
    ) == 0
    {
        let boundary = padding.wrapping_add(position & 0xff00);
        if boundary < position {
            let next = previous
                .wrapping_sub(word!(game.absolute_moved_last_frame_y))
                .wrapping_sub(2);
            let next = if (next.wrapping_sub(boundary) as i16) < 0 {
                boundary
            } else if camera_scroll_value(column, (next >> 8) as u8, game) != 0 {
                next
            } else {
                (next & 0xff00).wrapping_add(256)
            };
            word!(game.layer1_y_pos = next);
        }
    }
}
pub(super) fn trigger_down(game: &mut Game) {
    crate::verify_probe::hit("camera.trigger_down");
    let previous = word!(game.layer1_y_pos);
    let column = word!(game.layer1_x_pos).wrapping_add(128) >> 8;
    let row = (word!(game.layer1_y_pos) >> 8) as u8;
    let padding = if camera_scroll_value(column, row, game) == 1 {
        0u16
    } else {
        31
    };
    if (word!(game.ideal_layer1_ypos).wrapping_sub(word!(game.layer1_y_pos)) as i16) < 0 {
        word! {game; layer1_y_pos = word!(game.ideal_layer1_ypos); layer1_y_subpos = 0; }
    }
    let position = word!(game.layer1_y_pos);
    let mut boundary = word!(game.room_height_in_scrolls)
        .wrapping_sub(1)
        .swap_bytes()
        .wrapping_add(padding);
    if boundary < position
        || camera_scroll_value(
            column.wrapping_add(word!(game.room_width_in_scrolls)),
            row,
            game,
        ) == 0
            && {
                boundary = padding.wrapping_add(position & 0xff00);
                boundary < position
            }
    {
        word!(
            game.layer1_y_pos =
                retreat_to_boundary(previous, word!(game.absolute_moved_last_frame_y), boundary)
        );
    }
}
pub(super) fn trigger_up(game: &mut Game) {
    crate::verify_probe::hit("camera.trigger_up");
    let previous = word!(game.layer1_y_pos);
    if (word!(game.layer1_y_pos).wrapping_sub(word!(game.ideal_layer1_ypos)) as i16) < 0 {
        word! {game; layer1_y_pos = word!(game.ideal_layer1_ypos); layer1_y_subpos = 0; }
    }
    let position = word!(game.layer1_y_pos);
    if position & 0x8000 != 0 {
        word!(game.layer1_y_pos = 0);
    } else if camera_scroll_value(
        word!(game.layer1_x_pos).wrapping_add(128) >> 8,
        (position >> 8) as u8,
        game,
    ) == 0
    {
        let boundary = (position & 0xff00).wrapping_add(256);
        let next = previous
            .wrapping_add(word!(game.absolute_moved_last_frame_y))
            .wrapping_add(2);
        word!(game.layer1_y_pos = next.min(boundary));
    }
}
fn retreat_to_boundary(previous: u16, distance: u16, boundary: u16) -> u16 {
    let next = previous.wrapping_sub(distance).wrapping_sub(2);
    if (next.wrapping_sub(boundary) as i16) < 0 {
        boundary
    } else {
        next
    }
}

pub(crate) fn layer2_position(axis: Axis, game: &mut Game) -> u8 {
    let (control, source, destination) = match axis {
        Axis::X => (0x91b, W::layer1_x_pos, W::layer2_x_pos),
        Axis::Y => (0x91c, W::layer1_y_pos, W::layer2_y_pos),
    };
    let scale = game.g_ram[control];
    if scale == 1 {
        return 1;
    }
    let position = word!(@read game, source);
    let position = if scale == 0 {
        position
    } else {
        (u32::from(position) * u32::from(scale & 0xfe) >> 8) as u16
    };
    word!(@write game, destination, position);
    0
}

pub(crate) fn calculate_scrolls(game: &mut Game) {
    {
        word!(game.reg_BG1HOFS = (word!(game.bg1_x_offset)).wrapping_add(word!(game.layer1_x_pos)));
        word!(game.reg_BG1VOFS = (word!(game.bg1_y_offset)).wrapping_add(word!(game.layer1_y_pos)));
        word!(game.reg_BG2HOFS = (word!(game.bg2_x_scroll)).wrapping_add(word!(game.layer2_x_pos)));
        word!(game.reg_BG2VOFS = (word!(game.bg2_y_scroll)).wrapping_add(word!(game.layer2_y_pos)));
    }
}

pub(crate) fn calculate_blocks(game: &mut Game) {
    {
        crate::verify_probe::hit("camera.blocks");
        word! {game; bg1_x_block = word!(game.reg_BG1HOFS) >> 4;
        bg2_x_block = word!(game.reg_BG2HOFS) >> 4;
        layer1_x_block = ((word!(game.layer1_x_pos) as i16) >> 4) as u16;
        layer2_x_block = ((word!(game.layer2_x_pos) as i16) >> 4) as u16;
        bg1_y_block = word!(game.reg_BG1VOFS) >> 4; bg2_y_block = word!(game.reg_BG2VOFS) >> 4;
        layer1_y_block = ((word!(game.layer1_y_pos) as i16) >> 4) as u16;
        layer2_y_block = ((word!(game.layer2_y_pos) as i16) >> 4) as u16; }
    }
}

pub(crate) fn debug_position(game: &mut Game) {
    {
        if (word!(game.joypad2_new_keys) & 0x40) != 0 {
            word!(game.debug_saveload_scrollpos_toggle += 1);
        }
        if word!(game.debug_saveload_scrollpos_toggle) & 1 != 0 {
            word!(game.layer1_x_pos = word!(game.debug_saved_xscroll));
            word!(game.layer1_y_pos = word!(game.debug_saved_yscroll));
        } else {
            word!(game.debug_saved_xscroll = word!(game.layer1_x_pos));
            word!(game.debug_saved_yscroll = word!(game.layer1_y_pos));
        };
    }
}
