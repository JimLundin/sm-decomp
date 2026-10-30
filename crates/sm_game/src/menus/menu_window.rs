//! Map-menu window animation and its ordered indirect-HDMA projection.
use super::prelude::*;
use crate::ram::checked_word as word;

/// Requires a live, exclusively accessed SNES backend.
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    word!(game.expand_sq_topbottom_margin_right_pos = 0);
    word!(game.expand_sq_topbottom_margin_left_pos = u16::MAX);
    game.ram().write_byte(0x9e32, 127);
    game.ram().write_byte(0x9e36, 129);
    for (table, margin, middle, channel, window) in [
        (0x9e00, 34, 50, 0x4320, 0x26),
        (0x9e10, 32, 54, 0x4330, 0x27),
    ] {
        game.ram().copy_in(
            table,
            &[
                111, margin, 0x9e, 1, middle, 0x9e, 1, middle, 0x9e, 111, 32, 0x9e,
            ],
        );
        // Each table is visible before its channel is configured.
        {
            crate::display::sequence::register_writes! { game;
                byte(channel, 0x40); byte(channel + 1, window);
                byte(channel + 2, table as u8); byte(channel + 3, 0x9e);
                byte(channel + 4, 0x7e); byte(channel + 7, 0x7e);
                byte(channel + 5, 0); byte(channel + 6, 0);
                byte(channel + 8, 0); byte(channel + 9, 0); byte(channel + 10, 0);
            }
        }
    }
}

/// Return true once the first opening reaches the screen edges.
pub fn open_from_center<H: Host + ?Sized>(game: &mut H) -> bool {
    let outside = game.ram().read_byte(0x9e00).wrapping_sub(4);
    if (outside as i8) < 0 {
        return true;
    }
    for at in [0x9e00, 0x9e09, 0x9e10, 0x9e19] {
        game.ram().write_byte(at, outside);
    }
    let inside = game.ram().read_byte(0x9e03).wrapping_add(4);
    for at in [0x9e03, 0x9e06, 0x9e13, 0x9e16] {
        game.ram().write_byte(at, inside);
    }
    game.ram().write_byte(
        0x9e32,
        (word!(game.expand_sq_left_pos) as u8).wrapping_sub(4),
    );
    game.ram().write_byte(
        0x9e36,
        (word!(game.expand_sq_right_pos) as u8).wrapping_add(4),
    );
    false
}

fn append<H: Host + ?Sized>(
    table: usize,
    cursor: u16,
    count: u16,
    address: u16,
    game: &mut H,
) -> u16 {
    let at = table + usize::from(cursor);
    let split = count & 0x80 != 0;
    game.ram().write_byte(
        at,
        if split {
            (count as u8).wrapping_sub(127)
        } else {
            count as u8
        },
    );
    if split {
        game.ram().write_byte(at + 3, 127);
    }
    game.ram().copy_in(at + 1, &address.to_ne_bytes());
    if split {
        game.ram().copy_in(at + 4, &address.to_ne_bytes());
    }
    cursor.wrapping_add(if split { 6 } else { 3 })
}

pub fn project<H: Host + ?Sized>(game: &mut H) {
    let top = word!(game.expand_sq_top_pos);
    append(0x9e00, 0, top, 0x9e22, game);
    let mut cursor = append(0x9e10, 0, top, 0x9e20, game);
    let mut middle = word!(game.expand_sq_bottom_pos).wrapping_sub(word!(game.expand_sq_top_pos));
    if word!(game.expand_sq_bottom_pos) as u8 == word!(game.expand_sq_top_pos) as u8 {
        middle = 1;
    }
    append(0x9e00, cursor, middle, 0x9e32, game);
    cursor = append(0x9e10, cursor, middle, 0x9e36, game);
    let mut bottom = 0xffe0u16.wrapping_sub(word!(game.expand_sq_bottom_pos));
    if word!(game.expand_sq_bottom_pos) as u8 == 0xe0 {
        bottom = 1;
    }
    append(0x9e00, cursor, bottom, 0x9e22, game);
    cursor = append(0x9e10, cursor, bottom, 0x9e20, game);
    game.ram().write_byte(0x9e00 + usize::from(cursor), 0);
    game.ram().write_byte(0x9e10 + usize::from(cursor), 0);
}

/// Load native ROM words into the eight little-endian window velocity fields.
pub fn load_velocities<H: Host + ?Sized>(area: u16, game: &mut H) {
    let image = game.native_image();
    let base = sm_data::cartridge::lorom_offset(0x81aa34) as usize + 16 * usize::from(area);
    for component in 0..8 {
        word!(game, 0x9e40 + 2 * component; = image.table_word(base, component as isize));
    }
}

pub enum Motion {
    Expand,
    Contract,
}

pub fn advance<H: Host + ?Sized>(motion: Motion, game: &mut H) {
    for (edge, boundary, limit, minimum) in [
        (0, 1, 1, true),
        (4, 256, 255, false),
        (8, 1, 1, true),
        (12, 224, 224, false),
    ] {
        let position = 0x9e30 + edge;
        let velocity = 0x9e40 + edge;
        let delta =
            (u32::from(word!(game, velocity + 2;)) << 16) | u32::from(word!(game, velocity;));
        let delta = match motion {
            Motion::Expand => delta,
            // Retain the signed negation's overflow behavior before any position write.
            Motion::Contract => (-(delta as i32)) as u32,
        };
        let low = u16::from_ne_bytes([
            game.ram().read_byte(position),
            game.ram().read_byte(position + 1),
        ]);
        let high = u16::from_ne_bytes([
            game.ram().read_byte(position + 2),
            game.ram().read_byte(position + 3),
        ]);
        let sum = (u32::from(low) | (u32::from(high) << 16)).wrapping_add(delta);
        game.ram().copy_in(position, &(sum as u16).to_ne_bytes());
        game.ram()
            .copy_in(position + 2, &((sum >> 16) as u16).to_ne_bytes());
        if matches!(motion, Motion::Expand) {
            let below = (word!(game, position + 2;).wrapping_sub(boundary) as i16) < 0;
            if below == minimum {
                word!(game, position + 2; = limit);
            }
        }
    }
    project(game);
}
