//! Door camera setup and stepping; RAM remains authoritative between callbacks.
use super::Host;
use super::{add_native_motion, calculate_blocks, calculate_scrolls, layer2_position, update_map};
use crate::camera::{set_bg1_offset, set_layer1};
use crate::ram::words as fields;
use crate::ram_words as words;
use crate::room_entry::content::Axis;

// Door steps retain native split-word and scrolling tilemap publication order.
pub fn begin<H: Host + ?Sized>(game: &mut H) {
    set_layer1(game, 0, game.session().room.entry.arrival_x);
    set_layer1(game, 1, game.session().room.entry.arrival_y);
    let direction = game.session().room.entry.direction & 3;
    let backwards = matches!(direction, 0 | 2);
    let block_delta = if backwards { u16::MAX } else { 1 };
    // The dispatch leaves A = direction * 2 when the X calculation starts.
    let x = layer2_accumulator(Axis::X, direction * 2, game);
    match direction {
        0 | 1 => {
            let offset = if backwards { 0xff00 } else { 256 };
            game.session_mut().room.camera.layer2[0] = x.wrapping_add(offset);
            layer2_position(Axis::Y, game);
            set_layer1(
                game,
                0,
                game.session().room.camera.layer1[0].wrapping_add(offset),
            );
        }
        2 => {
            let y = layer2_accumulator(Axis::Y, x, game);
            game.session_mut().room.camera.layer2[1] = y.wrapping_sub(224);
            set_layer1(
                game,
                1,
                game.session().room.camera.layer1[1].wrapping_sub(224),
            );
        }
        _ => {
            let previous_y = game.session().room.camera.layer1[1];
            let lowered = previous_y.wrapping_add(31);
            set_layer1(game, 1, lowered);
            let y = layer2_accumulator(Axis::Y, lowered, game);
            game.session_mut().room.camera.layer2[1] = y.wrapping_add(224);
            set_layer1(game, 1, previous_y.wrapping_add(256));
        }
    }
    set_bg1_offset(
        game,
        0,
        crate::menus::prelude::display_scroll!(game.Bg1H)
            .wrapping_sub(game.session().room.camera.layer1[0]),
    );
    game.session_mut().room.camera.bg1_offset[1] =
        crate::menus::prelude::display_scroll!(game.Bg1V)
            .wrapping_sub(game.session().room.camera.layer1[1]);
    game.session_mut().room.camera.bg2_scroll[0] =
        crate::menus::prelude::display_scroll!(game.Bg2H)
            .wrapping_sub(game.session().room.camera.layer1[0]);
    game.session_mut().room.camera.bg2_scroll[1] =
        crate::menus::prelude::display_scroll!(game.Bg2V)
            .wrapping_sub(game.session().room.camera.layer1[1]);
    if direction == 3 {
        game.session_mut().room.entry.arrival_y =
            game.session().room.entry.arrival_y.wrapping_add(32);
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
            set_layer1(
                game,
                1,
                game.session().room.camera.layer1[1].wrapping_sub(1),
            );
            up(game);
        }
    }
}

/// The accumulator the original's layer-2 calculation ($80:A2F9/$A33A) returns: the new
/// position, or for a layer locked to the screen (scroll value 1) the locked value in the low
/// byte under the high byte of `entry`, the accumulator it was called with.
fn layer2_accumulator<H: Host + ?Sized>(axis: Axis, entry: u16, game: &mut H) -> u16 {
    let index = match axis {
        Axis::X => 0,
        Axis::Y => 1,
    };
    if layer2_position(axis, game) == 0 {
        game.session().room.camera.layer2[index]
    } else {
        (entry & 0xff00) | 1
    }
}

fn prepare_blocks<H: Host + ?Sized>(vertical: bool, delta: u16, game: &mut H) {
    calculate_blocks(game);
    game.session_mut().room.camera.previous_layer1_block[0] =
        game.session().room.camera.layer1_block[0];
    game.session_mut().room.camera.previous_layer2_block[0] =
        game.session().room.camera.layer2_block[0];
    game.session_mut().room.camera.previous_layer1_block[1] =
        game.session().room.camera.layer1_block[1];
    game.session_mut().room.camera.previous_layer2_block[1] =
        game.session().room.camera.layer2_block[1];
    if vertical {
        game.session_mut().room.camera.previous_layer1_block[1] =
            game.session().room.camera.previous_layer1_block[1].wrapping_add(delta);
        game.session_mut().room.camera.previous_layer2_block[1] =
            game.session().room.camera.previous_layer2_block[1].wrapping_add(delta);
    } else {
        game.session_mut().room.camera.previous_layer1_block[0] =
            game.session().room.camera.previous_layer1_block[0].wrapping_add(delta);
        game.session_mut().room.camera.previous_layer2_block[0] =
            game.session().room.camera.previous_layer2_block[0].wrapping_add(delta);
    }
}

pub fn fix_up<H: Host + ?Sized>(game: &mut H) {
    game.session_mut().room.entry.frame = 0;
    prepare_blocks(true, 1, game);
    up(game);
}

pub fn horizontal<H: Host + ?Sized>(left: bool, game: &mut H) -> u8 {
    let frame = game.session().room.entry.frame;
    let speed = words!(
        game.SAMUS_DOOR_TRANSITION_SPEED,
        SAMUS_DOOR_TRANSITION_SUBSPEED
    );
    let speed = if left { -(speed as i32) as u32 } else { speed };
    add_native_motion(fields::SAMUS_X_POS, fields::SAMUS_X_SUBPOS, speed, game);
    game.set_previous_player_x(game.player_x());
    let delta = if left { 0xfffc } else { 4 };
    set_layer1(
        game,
        0,
        game.session().room.camera.layer1[0].wrapping_add(delta),
    );
    game.session_mut().room.camera.layer2[0] =
        game.session().room.camera.layer2[0].wrapping_add(delta);
    update_map(game);
    game.session_mut().room.entry.frame = frame.wrapping_add(1);
    if frame == 63 && !left {
        update_map(game);
    }
    (frame == 63) as u8
}

fn first_vertical_frame<H: Host + ?Sized>(offset: u16, block_delta: u16, game: &mut H) {
    let bg1 = crate::menus::prelude::display_scroll!(game.Bg1V);
    let bg2 = crate::menus::prelude::display_scroll!(game.Bg2V);
    let layer1 = game.session().room.camera.layer1[1];
    set_layer1(
        game,
        1,
        game.session().room.camera.layer1[1].wrapping_sub(offset),
    );
    let layer2 = game.session().room.camera.layer2[1];
    game.session_mut().room.camera.layer2[1] =
        game.session().room.camera.layer2[1].wrapping_sub(offset);
    prepare_blocks(true, block_delta, game);
    update_map(game);
    game.session_mut().room.camera.layer2[1] = layer2;
    set_layer1(game, 1, layer1);
    crate::menus::prelude::display_scroll!(game.Bg2V = bg2);
    crate::menus::prelude::display_scroll!(game.Bg1V = bg1);
}

pub fn down<H: Host + ?Sized>(game: &mut H) -> u8 {
    let frame = game.session().room.entry.frame;
    if frame == 0 {
        first_vertical_frame(15, u16::MAX, game);
    } else if frame < 0x39 {
        add_native_motion(
            fields::SAMUS_Y_POS,
            fields::SAMUS_Y_SUBPOS,
            words!(
                game.SAMUS_DOOR_TRANSITION_SPEED,
                SAMUS_DOOR_TRANSITION_SUBSPEED
            ),
            game,
        );
        game.set_previous_player_y(game.player_y());
        set_layer1(
            game,
            1,
            game.session().room.camera.layer1[1].wrapping_add(4),
        );
        game.session_mut().room.camera.layer2[1] =
            game.session().room.camera.layer2[1].wrapping_add(4);
        update_map(game);
    }
    game.session_mut().room.entry.frame = frame.wrapping_add(1);
    if frame.wrapping_add(1) < 0x39 {
        return 0;
    }
    update_map(game);
    1
}

pub fn up<H: Host + ?Sized>(game: &mut H) -> u8 {
    let frame = game.session().room.entry.frame;
    if frame == 0 {
        first_vertical_frame(16, 1, game);
    } else {
        add_native_motion(
            fields::SAMUS_Y_POS,
            fields::SAMUS_Y_SUBPOS,
            -(words!(
                i32,
                game.SAMUS_DOOR_TRANSITION_SPEED,
                SAMUS_DOOR_TRANSITION_SUBSPEED
            )) as u32,
            game,
        );
        game.set_previous_player_y(game.player_y());
        set_layer1(
            game,
            1,
            game.session().room.camera.layer1[1].wrapping_sub(4),
        );
        game.session_mut().room.camera.layer2[1] =
            game.session().room.camera.layer2[1].wrapping_sub(4);
        if game.session().room.entry.frame >= 5 {
            update_map(game);
        } else {
            calculate_scrolls(game);
        }
    }
    game.session_mut().room.entry.frame = frame.wrapping_add(1);
    (frame == 56) as u8
}
