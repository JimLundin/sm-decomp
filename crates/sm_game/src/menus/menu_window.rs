//! Map-menu window animation and its ordered indirect-HDMA projection.
use super::prelude::*;
use crate::display::transfer::Source;
use crate::display::video::{Command, WindowEdge, WindowHdma};
use crate::menu_window_state::Field;

/// Work RAM ($7E) layout of the map-menu window.
pub(super) mod ram {
    /// Indirect HDMA tables for window 1 and window 2.
    pub const WINDOW_1_TABLE: usize = 0x9e00;
    pub const WINDOW_2_TABLE: usize = 0x9e10;
    /// Indirect sources: closed (window 2, window 1) and open (left, right) edges.
    pub const CLOSED_2: u16 = 0x9e20;
    pub const CLOSED_1: u16 = 0x9e22;
    pub const LEFT_EDGE: u16 = 0x9e32;
    pub const RIGHT_EDGE: u16 = 0x9e36;
    /// The high byte of the indirect table addresses, and their bank.
    pub const PAGE: u8 = 0x9e;
    pub const BANK: u8 = 0x7e;
}

/// The window 1 edges' HDMA channels.
const LEFT_EDGE_CHANNEL: u8 = 2;
const RIGHT_EDGE_CHANNEL: u8 = 3;
/// $81:AA34: each area's eight window velocity words.
const AREA_WINDOW_VELOCITIES: u32 = 0x81_aa34;
/// The last scanline the window reaches.
const LAST_SCANLINE: u8 = 0xe0;

/// Requires a live, exclusively accessed SNES backend.
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    let mut window = game.window_state();
    window.right_margin = 0;
    window.left_margin = u16::MAX;
    game.publish_window(window, &[Field::RightMargin, Field::LeftMargin]);
    game.ram().write_byte(usize::from(ram::LEFT_EDGE), 127);
    game.ram().write_byte(usize::from(ram::RIGHT_EDGE), 129);
    for (table, margin, middle, channel, edge) in [
        (
            ram::WINDOW_1_TABLE,
            34,
            50,
            LEFT_EDGE_CHANNEL,
            WindowEdge::Left,
        ),
        (
            ram::WINDOW_2_TABLE,
            32,
            54,
            RIGHT_EDGE_CHANNEL,
            WindowEdge::Right,
        ),
    ] {
        let page = ram::PAGE;
        game.ram().copy_in(
            table,
            &[
                111, margin, page, 1, middle, page, 1, middle, page, 111, 32, page,
            ],
        );
        // Each table is visible before its channel is configured.
        game.apply(
            "map window",
            Command::WindowHdma(WindowHdma {
                channel,
                edge,
                table: Source {
                    address: u16::from_le_bytes([table.to_le_bytes()[0], ram::PAGE]),
                    bank: ram::BANK,
                },
                indirect_bank: ram::BANK,
            }),
        );
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
        game.window_state().positions[0].whole.to_le_bytes()[0].wrapping_sub(4),
    );
    game.ram().write_byte(
        usize::from(ram::RIGHT_EDGE),
        game.window_state().positions[1].whole.to_le_bytes()[0].wrapping_add(4),
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
    let top = game.window_state().positions[2].whole;
    let (one, two) = (ram::WINDOW_1_TABLE, ram::WINDOW_2_TABLE);
    append(one, 0, top, ram::CLOSED_1, game);
    let mut cursor = append(two, 0, top, ram::CLOSED_2, game);
    let mut middle = game.window_state().positions[3]
        .whole
        .wrapping_sub(game.window_state().positions[2].whole);
    if game.window_state().positions[3].whole as u8 == game.window_state().positions[2].whole as u8
    {
        middle = 1;
    }
    append(one, cursor, middle, ram::LEFT_EDGE, game);
    cursor = append(two, cursor, middle, ram::RIGHT_EDGE, game);
    let mut bottom =
        (0xff00 | u16::from(LAST_SCANLINE)).wrapping_sub(game.window_state().positions[3].whole);
    if game.window_state().positions[3].whole as u8 == LAST_SCANLINE {
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
    let mut window = game.window_state();
    for (component, field) in [
        Field::VelocityFraction(0),
        Field::VelocityWhole(0),
        Field::VelocityFraction(1),
        Field::VelocityWhole(1),
        Field::VelocityFraction(2),
        Field::VelocityWhole(2),
        Field::VelocityFraction(3),
        Field::VelocityWhole(3),
    ]
    .into_iter()
    .enumerate()
    {
        *field.get_mut(&mut window) = image.table_word(base, isize::try_from(component).unwrap());
        // The prior velocity words survive a later native-ROM bounds failure.
        game.publish_window(window, &[field]);
    }
}

pub enum Motion {
    Expand,
    Contract,
}

pub fn advance<H: Host + ?Sized>(motion: Motion, game: &mut H) {
    for (edge, _) in game.window_state().positions.into_iter().enumerate() {
        let mut window = game.window_state();
        let contract = matches!(motion, Motion::Contract);
        window.advance_edge(edge, contract);
        game.publish_window(window, &[Field::Fraction(edge), Field::Whole(edge)]);
        window = game.window_state();
        if !contract && window.clamp_edge(edge) {
            game.publish_window(window, &[Field::Whole(edge)]);
        }
    }
    project(game);
}
