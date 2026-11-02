//! Camera limits and scroll-cell rules over canonical room/player state.
use super::{CameraHost, Host, set_layer1};

use crate::room_entry::content::Axis;

// Multiplication uses the width's low byte. A column may include a full-word
// row stride; it is added afterward, with the original word wrapping.
fn camera_scroll_value<H: CameraHost + ?Sized>(column: u16, row: u8, game: &H) -> u8 {
    let index =
        (u16::from(row) * u16::from(game.session().room.scroll_width as u8)).wrapping_add(column);
    game.camera_scroll_byte(usize::from(index))
}

pub fn auto_x<H: CameraHost + ?Sized>(game: &mut H) {
    crate::probe::hit("camera.auto_x");
    if game.session().timing.time_frozen != 0 {
        return;
    }
    let previous = game.session().room.camera.layer1[0];
    if previous & 0x8000 != 0 {
        set_layer1(game, 0, 0);
    }
    let limit = game
        .session()
        .room
        .scroll_width
        .wrapping_sub(1)
        .swap_bytes();
    if limit < game.session().room.camera.layer1[0] {
        set_layer1(game, 0, limit);
    }
    let position = game.session().room.camera.layer1[0];
    let row = (game.session().room.camera.layer1[1].wrapping_add(128) >> 8) as u8;
    let column = position >> 8;
    let next = if camera_scroll_value(column, row, game) != 0 {
        if camera_scroll_value(column + 1, row, game) != 0 {
            return;
        }
        let boundary = position & 0xff00;
        let next = previous
            .wrapping_sub(game.session().player.camera_distance[0].whole)
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
            .wrapping_add(game.session().player.camera_distance[0].whole)
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
    set_layer1(game, 0, next);
}
pub fn trigger_right<H: CameraHost + ?Sized>(game: &mut H) {
    crate::probe::hit("camera.trigger_right");
    let previous = game.session().room.camera.layer1[0];
    if (game.session().room.camera.ideal_layer1[0]
        .wrapping_sub(game.session().room.camera.layer1[0]) as i16)
        < 0
    {
        set_layer1(game, 0, game.session().room.camera.ideal_layer1[0]);
        game.session_mut().room.camera.layer1_fraction[0] = 0;
    }
    let limit = game
        .session()
        .room
        .scroll_width
        .wrapping_sub(1)
        .swap_bytes();
    let position = game.session().room.camera.layer1[0];
    if limit < position {
        set_layer1(game, 0, limit);
    } else if camera_scroll_value(
        (position >> 8) + 1,
        (game.session().room.camera.layer1[1].wrapping_add(128) >> 8) as u8,
        game,
    ) == 0
    {
        let boundary = position & 0xff00;
        set_layer1(
            game,
            0,
            retreat_to_boundary(
                previous,
                game.session().player.camera_distance[0].whole,
                boundary,
            ),
        );
    }
}
pub fn trigger_left<H: CameraHost + ?Sized>(game: &mut H) {
    crate::probe::hit("camera.trigger_left");
    let previous = game.session().room.camera.layer1[0];
    if (game.session().room.camera.layer1[0]
        .wrapping_sub(game.session().room.camera.ideal_layer1[0]) as i16)
        < 0
    {
        set_layer1(game, 0, game.session().room.camera.ideal_layer1[0]);
        game.session_mut().room.camera.layer1_fraction[0] = 0;
    }
    let position = game.session().room.camera.layer1[0];
    if position & 0x8000 != 0 {
        set_layer1(game, 0, 0);
    } else if camera_scroll_value(
        position >> 8,
        (game.session().room.camera.layer1[1].wrapping_add(128) >> 8) as u8,
        game,
    ) == 0
    {
        let boundary = (position & 0xff00).wrapping_add(256);
        let next = previous
            .wrapping_add(game.session().player.camera_distance[0].whole)
            .wrapping_add(2);
        set_layer1(game, 0, next.min(boundary));
    }
}
pub fn auto_y<H: CameraHost + ?Sized>(game: &mut H) {
    crate::probe::hit("camera.auto_y");
    if game.session().timing.time_frozen != 0 {
        return;
    }
    let column = game.session().room.camera.layer1[0].wrapping_add(128) >> 8;
    let padding = if camera_scroll_value(
        column,
        (game.session().room.camera.layer1[1] >> 8) as u8,
        game,
    ) == 1
    {
        0u16
    } else {
        31
    };
    let previous = game.session().room.camera.layer1[1];
    if previous & 0x8000 != 0 {
        set_layer1(game, 1, 0);
    }
    let limit = game
        .session()
        .room
        .scroll_height
        .wrapping_sub(1)
        .swap_bytes()
        .wrapping_add(padding);
    if limit < game.session().room.camera.layer1[1] {
        set_layer1(game, 1, limit);
    }
    let position = game.session().room.camera.layer1[1];
    let row = (position >> 8) as u8;
    if camera_scroll_value(column, row, game) == 0 {
        let boundary = (position & 0xff00).wrapping_add(256);
        let next = previous
            .wrapping_add(game.session().player.camera_distance[1].whole)
            .wrapping_add(2);
        let next = if next >= boundary {
            boundary
        } else if camera_scroll_value(column, ((next >> 8) as u8).wrapping_add(1), game) != 0 {
            next
        } else {
            next & 0xff00
        };
        set_layer1(game, 1, next);
    } else if camera_scroll_value(
        column.wrapping_add(game.session().room.scroll_width),
        row,
        game,
    ) == 0
    {
        let boundary = padding.wrapping_add(position & 0xff00);
        if boundary < position {
            let next = previous
                .wrapping_sub(game.session().player.camera_distance[1].whole)
                .wrapping_sub(2);
            let next = if (next.wrapping_sub(boundary) as i16) < 0 {
                boundary
            } else if camera_scroll_value(column, (next >> 8) as u8, game) != 0 {
                next
            } else {
                (next & 0xff00).wrapping_add(256)
            };
            set_layer1(game, 1, next);
        }
    }
}
pub fn trigger_down<H: CameraHost + ?Sized>(game: &mut H) {
    crate::probe::hit("camera.trigger_down");
    let previous = game.session().room.camera.layer1[1];
    let column = game.session().room.camera.layer1[0].wrapping_add(128) >> 8;
    let row = (game.session().room.camera.layer1[1] >> 8) as u8;
    let padding = if camera_scroll_value(column, row, game) == 1 {
        0u16
    } else {
        31
    };
    if (game.session().room.camera.ideal_layer1[1]
        .wrapping_sub(game.session().room.camera.layer1[1]) as i16)
        < 0
    {
        set_layer1(game, 1, game.session().room.camera.ideal_layer1[1]);
        game.session_mut().room.camera.layer1_fraction[1] = 0;
    }
    let position = game.session().room.camera.layer1[1];
    let mut boundary = game
        .session()
        .room
        .scroll_height
        .wrapping_sub(1)
        .swap_bytes()
        .wrapping_add(padding);
    if boundary < position
        || camera_scroll_value(
            column.wrapping_add(game.session().room.scroll_width),
            row,
            game,
        ) == 0
            && {
                boundary = padding.wrapping_add(position & 0xff00);
                boundary < position
            }
    {
        set_layer1(
            game,
            1,
            retreat_to_boundary(
                previous,
                game.session().player.camera_distance[1].whole,
                boundary,
            ),
        );
    }
}
pub fn trigger_up<H: CameraHost + ?Sized>(game: &mut H) {
    crate::probe::hit("camera.trigger_up");
    let previous = game.session().room.camera.layer1[1];
    if (game.session().room.camera.layer1[1]
        .wrapping_sub(game.session().room.camera.ideal_layer1[1]) as i16)
        < 0
    {
        set_layer1(game, 1, game.session().room.camera.ideal_layer1[1]);
        game.session_mut().room.camera.layer1_fraction[1] = 0;
    }
    let position = game.session().room.camera.layer1[1];
    if position & 0x8000 != 0 {
        set_layer1(game, 1, 0);
    } else if camera_scroll_value(
        game.session().room.camera.layer1[0].wrapping_add(128) >> 8,
        (position >> 8) as u8,
        game,
    ) == 0
    {
        let boundary = (position & 0xff00).wrapping_add(256);
        let next = previous
            .wrapping_add(game.session().player.camera_distance[1].whole)
            .wrapping_add(2);
        set_layer1(game, 1, next.min(boundary));
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

pub fn layer2_position<H: CameraHost + ?Sized>(axis: Axis, game: &mut H) -> u8 {
    let index = match axis {
        Axis::X => 0,
        Axis::Y => 1,
    };
    let entry = &game.session().room.entry;
    let scale = [entry.layer2_x, entry.layer2_y][index];
    if scale == 1 {
        return 1;
    }
    let position = game.session().room.camera.layer1[index];
    let position = if scale == 0 {
        position
    } else {
        ((u32::from(position) * u32::from(scale & 0xfe)) >> 8) as u16
    };
    game.session_mut().room.camera.layer2[index] = position;
    0
}

pub fn calculate_scrolls<H: Host + ?Sized>(game: &mut H) {
    crate::menus::prelude::display_scroll!(
        game.Bg1H = (game.session().room.camera.bg1_offset[0])
            .wrapping_add(game.session().room.camera.layer1[0])
    );
    crate::menus::prelude::display_scroll!(
        game.Bg1V = (game.session().room.camera.bg1_offset[1])
            .wrapping_add(game.session().room.camera.layer1[1])
    );
    crate::menus::prelude::display_scroll!(
        game.Bg2H = (game.session().room.camera.bg2_scroll[0])
            .wrapping_add(game.session().room.camera.layer2[0])
    );
    crate::menus::prelude::display_scroll!(
        game.Bg2V = (game.session().room.camera.bg2_scroll[1])
            .wrapping_add(game.session().room.camera.layer2[1])
    );
}

pub fn calculate_blocks<H: Host + ?Sized>(game: &mut H) {
    crate::probe::hit("camera.blocks");
    game.session_mut().room.camera.bg1_block[0] =
        crate::menus::prelude::display_scroll!(game.Bg1H) >> 4;
    game.session_mut().room.camera.bg2_block[0] =
        crate::menus::prelude::display_scroll!(game.Bg2H) >> 4;
    game.session_mut().room.camera.layer1_block[0] =
        ((game.session().room.camera.layer1[0] as i16) >> 4) as u16;
    game.session_mut().room.camera.layer2_block[0] =
        ((game.session().room.camera.layer2[0] as i16) >> 4) as u16;
    game.session_mut().room.camera.bg1_block[1] =
        crate::menus::prelude::display_scroll!(game.Bg1V) >> 4;
    game.session_mut().room.camera.bg2_block[1] =
        crate::menus::prelude::display_scroll!(game.Bg2V) >> 4;
    game.session_mut().room.camera.layer1_block[1] =
        ((game.session().room.camera.layer1[1] as i16) >> 4) as u16;
    game.session_mut().room.camera.layer2_block[1] =
        ((game.session().room.camera.layer2[1] as i16) >> 4) as u16;
}

pub fn debug_position<H: Host + ?Sized>(game: &mut H) {
    if (crate::controller_state::Word::Joypad2NewKeys.get(&game.session().control) & 0x40) != 0 {
        let toggle = &mut game.session_mut().game_loop.debug.camera_toggle;
        *toggle = toggle.wrapping_add(1);
    }
    if game.session().game_loop.debug.camera_toggle & 1 != 0 {
        set_layer1(game, 0, game.session().game_loop.debug.camera[0]);
        set_layer1(game, 1, game.session().game_loop.debug.camera[1]);
    } else {
        game.session_mut().game_loop.debug.camera[0] = game.session().room.camera.layer1[0];
        game.session_mut().game_loop.debug.camera[1] = game.session().room.camera.layer1[1];
    };
}
