//! Map-menu window animation and its ordered indirect-HDMA projection.
use super::prelude::*;
use crate::display::transfer::Source;
use crate::display::video::{Command, WindowEdge, WindowHdma};
use crate::menu_window_state::Field;
use crate::scene::hdma_sources::LiveOwner;

/// The map-menu window's indirect HDMA tables, each in its window's buffer.
pub(super) const WINDOW_1_TABLE: LiveOwner = LiveOwner::WindowLeft;
pub(super) const WINDOW_2_TABLE: LiveOwner = LiveOwner::WindowRight;
/// Where in window 1's buffer the open edges' indirect sources lie.
const LEFT_EDGE_OFFSET: usize = 0x32;
const RIGHT_EDGE_OFFSET: usize = 0x36;

/// Work RAM ($7E) addresses the window's HDMA tables name.
mod ram {
    /// The tables' bank-local addresses, as their channels read them.
    pub const WINDOW_1_TABLE: u16 = 0x9e00;
    pub const WINDOW_2_TABLE: u16 = 0x9e10;
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
    game.store_buffer_byte(WINDOW_1_TABLE, LEFT_EDGE_OFFSET, 127);
    game.store_buffer_byte(WINDOW_1_TABLE, RIGHT_EDGE_OFFSET, 129);
    for (table, margin, middle, channel, edge) in [
        (WINDOW_1_TABLE, 34, 50, LEFT_EDGE_CHANNEL, WindowEdge::Left),
        (
            WINDOW_2_TABLE,
            32,
            54,
            RIGHT_EDGE_CHANNEL,
            WindowEdge::Right,
        ),
    ] {
        let page = ram::PAGE;
        game.store_buffer_bytes(
            table,
            0,
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
                    address: table_address(table),
                    bank: ram::BANK,
                },
                indirect_bank: ram::BANK,
            }),
        );
    }
}

/// Return true once the first opening reaches the screen edges.
pub fn open_from_center<H: Host + ?Sized>(game: &mut H) -> bool {
    let (one, two) = (WINDOW_1_TABLE, WINDOW_2_TABLE);
    let outside = game.buffer_byte(one, 0).wrapping_sub(4);
    if (outside as i8) < 0 {
        return true;
    }
    // The first and last line counts of both tables, then the middle two.
    for (table, offset) in [(one, 0), (one, 9), (two, 0), (two, 9)] {
        game.store_buffer_byte(table, offset, outside);
    }
    let inside = game.buffer_byte(one, 3).wrapping_add(4);
    for (table, offset) in [(one, 3), (one, 6), (two, 3), (two, 6)] {
        game.store_buffer_byte(table, offset, inside);
    }
    let left = game.window_state().positions[0].whole.to_le_bytes()[0].wrapping_sub(4);
    game.store_buffer_byte(one, LEFT_EDGE_OFFSET, left);
    let right = game.window_state().positions[1].whole.to_le_bytes()[0].wrapping_add(4);
    game.store_buffer_byte(one, RIGHT_EDGE_OFFSET, right);
    false
}

/// The bank-local address of `table`, as its channel reads it.
fn table_address(table: LiveOwner) -> u16 {
    if table == WINDOW_1_TABLE {
        ram::WINDOW_1_TABLE
    } else {
        ram::WINDOW_2_TABLE
    }
}

fn append<H: Host + ?Sized>(
    table: LiveOwner,
    cursor: u16,
    count: u16,
    address: u16,
    game: &mut H,
) -> u16 {
    let at = usize::from(cursor);
    let split = count & 0x80 != 0;
    game.store_buffer_byte(
        table,
        at,
        if split {
            (count as u8).wrapping_sub(127)
        } else {
            count as u8
        },
    );
    if split {
        game.store_buffer_byte(table, at + 3, 127);
    }
    game.store_buffer_bytes(table, at + 1, &address.to_ne_bytes());
    if split {
        game.store_buffer_bytes(table, at + 4, &address.to_ne_bytes());
    }
    cursor.wrapping_add(if split { 6 } else { 3 })
}

pub fn project<H: Host + ?Sized>(game: &mut H) {
    let top = game.window_state().positions[2].whole;
    let (one, two) = (WINDOW_1_TABLE, WINDOW_2_TABLE);
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
    game.store_buffer_byte(one, usize::from(cursor), 0);
    game.store_buffer_byte(two, usize::from(cursor), 0);
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
