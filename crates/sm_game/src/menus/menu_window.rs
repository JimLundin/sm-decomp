//! Map-menu window animation and its ordered indirect-HDMA projection.
use super::prelude::*;
use crate::ram::checked_word as word;

/// Work RAM ($7E) layout of the map-menu window.
mod ram {
    /// Indirect HDMA tables for window 1 and window 2.
    pub const WINDOW_1_TABLE: usize = 0x9e00;
    pub const WINDOW_2_TABLE: usize = 0x9e10;
    /// Indirect sources: closed (window 2, window 1) and open (left, right) edges.
    pub const CLOSED_2: u16 = 0x9e20;
    pub const CLOSED_1: u16 = 0x9e22;
    pub const LEFT_EDGE: u16 = 0x9e32;
    pub const RIGHT_EDGE: u16 = 0x9e36;
    /// The four 16.16 edge positions (left, right, top, bottom) and velocities.
    pub const EDGE_POSITIONS: usize = 0x9e30;
    pub const EDGE_VELOCITIES: usize = 0x9e40;
    /// The high byte of the indirect table addresses, and their bank.
    pub const PAGE: u8 = 0x9e;
    pub const BANK: u8 = 0x7e;
}

/// HDMA channels 2 and 3, and the indirect transfer mode.
const CHANNEL_2: u16 = 0x4320;
const CHANNEL_3: u16 = 0x4330;
const INDIRECT_ONE_REGISTER: u8 = 0x40;
/// WH0 and WH1: the window 1 left and right position registers' B-bus ports.
const WH0: u8 = 0x26;
const WH1: u8 = 0x27;
/// $81:AA34: each area's eight window velocity words.
const AREA_WINDOW_VELOCITIES: u32 = 0x81_aa34;
/// The last scanline the window reaches.
const LAST_SCANLINE: u8 = 0xe0;

/// Requires a live, exclusively accessed SNES backend.
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    word!(game.expand_sq_topbottom_margin_right_pos = 0);
    word!(game.expand_sq_topbottom_margin_left_pos = u16::MAX);
    game.ram().write_byte(usize::from(ram::LEFT_EDGE), 127);
    game.ram().write_byte(usize::from(ram::RIGHT_EDGE), 129);
    for (table, margin, middle, channel, window) in [
        (ram::WINDOW_1_TABLE, 34, 50, CHANNEL_2, WH0),
        (ram::WINDOW_2_TABLE, 32, 54, CHANNEL_3, WH1),
    ] {
        let page = ram::PAGE;
        game.ram().copy_in(
            table,
            &[
                111, margin, page, 1, middle, page, 1, middle, page, 111, 32, page,
            ],
        );
        // Each table is visible before its channel is configured.
        {
            crate::display::sequence::register_writes! { game;
                byte(channel, INDIRECT_ONE_REGISTER); byte(channel + 1, window);
                byte(channel + 2, table as u8); byte(channel + 3, ram::PAGE);
                byte(channel + 4, ram::BANK); byte(channel + 7, ram::BANK);
                byte(channel + 5, 0); byte(channel + 6, 0);
                byte(channel + 8, 0); byte(channel + 9, 0); byte(channel + 10, 0);
            }
        }
    }
}

/// Return true once the first opening reaches the screen edges.
pub fn open_from_center<H: Host + ?Sized>(game: &mut H) -> bool {
    let (one, two) = (ram::WINDOW_1_TABLE, ram::WINDOW_2_TABLE);
    let outside = game.ram().read_byte(one).wrapping_sub(4);
    if (outside as i8) < 0 {
        return true;
    }
    // The first and last line counts of both tables, then the middle two.
    for at in [one, one + 9, two, two + 9] {
        game.ram().write_byte(at, outside);
    }
    let inside = game.ram().read_byte(one + 3).wrapping_add(4);
    for at in [one + 3, one + 6, two + 3, two + 6] {
        game.ram().write_byte(at, inside);
    }
    game.ram().write_byte(
        usize::from(ram::LEFT_EDGE),
        (word!(game.expand_sq_left_pos) as u8).wrapping_sub(4),
    );
    game.ram().write_byte(
        usize::from(ram::RIGHT_EDGE),
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
    let (one, two) = (ram::WINDOW_1_TABLE, ram::WINDOW_2_TABLE);
    append(one, 0, top, ram::CLOSED_1, game);
    let mut cursor = append(two, 0, top, ram::CLOSED_2, game);
    let mut middle = word!(game.expand_sq_bottom_pos).wrapping_sub(word!(game.expand_sq_top_pos));
    if word!(game.expand_sq_bottom_pos) as u8 == word!(game.expand_sq_top_pos) as u8 {
        middle = 1;
    }
    append(one, cursor, middle, ram::LEFT_EDGE, game);
    cursor = append(two, cursor, middle, ram::RIGHT_EDGE, game);
    let mut bottom =
        (0xff00 | u16::from(LAST_SCANLINE)).wrapping_sub(word!(game.expand_sq_bottom_pos));
    if word!(game.expand_sq_bottom_pos) as u8 == LAST_SCANLINE {
        bottom = 1;
    }
    append(one, cursor, bottom, ram::CLOSED_1, game);
    cursor = append(two, cursor, bottom, ram::CLOSED_2, game);
    game.ram().write_byte(one + usize::from(cursor), 0);
    game.ram().write_byte(two + usize::from(cursor), 0);
}

/// Load native ROM words into the eight little-endian window velocity fields.
pub fn load_velocities<H: Host + ?Sized>(area: u16, game: &mut H) {
    let image = game.native_image();
    let base =
        sm_data::cartridge::lorom_offset(AREA_WINDOW_VELOCITIES) as usize + 16 * usize::from(area);
    for component in 0..8 {
        word!(game, ram::EDGE_VELOCITIES + 2 * component; = image.table_word(base, component as isize));
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
        let position = ram::EDGE_POSITIONS + edge;
        let velocity = ram::EDGE_VELOCITIES + edge;
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
