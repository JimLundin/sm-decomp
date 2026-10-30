//! Eye-door placement and transition after its persistent open flag is set.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::host::*;
use super::terrain_setup as terrain;
use crate::progress as flags;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Eye,
    Body,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Right,
    Left,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Eye,
    Body,
    Blue(Side),
    Extension(u8),
}

pub fn begin(c: &mut impl Host, slot: usize, part: Part) -> u8 {
    let argument = mechanism!(c, slot, room_argument);
    if !flags::door_opened(c, argument) {
        install(
            mechanism!(c, slot, block),
            if part == Part::Eye {
                Tile::Eye
            } else {
                Tile::Body
            },
            c,
        );
        if part == Part::Eye {
            let row = c.session().room.width.wrapping_mul(2);
            let block = mechanism!(c, slot, block);
            install((block).wrapping_add(row), Tile::Extension(1), c);
        }
    }
    0
}
pub fn wake(c: &mut impl Host, slot: usize) {
    let argument = mechanism!(c, slot, room_argument);
    if flags::door_opened(c, argument) {
        mechanism!(c, slot, pre_action = 0xd779);
        mechanism!(c, slot, program = mechanism!(c, slot, return_program));
        mechanism!(c, slot, instruction_delay = 1);
    }
}
pub fn convert(c: &mut impl Host, slot: usize, side: Side) {
    let old = mechanism!(c, slot, block);
    let row = c.session().room.width.wrapping_mul(2);
    mechanism!(c, slot, block = (old).wrapping_sub(row));
    let mut block = mechanism!(c, slot, block);
    install(block, Tile::Blue(side), c);
    for distance in 1..=3 {
        block = (block).wrapping_add(c.session().room.width.wrapping_mul(2));
        install(block, Tile::Extension(distance), c);
    }
}

fn install(block: u16, tile: Tile, c: &mut impl Host) {
    terrain::install(
        c,
        block,
        match tile {
            Tile::Eye => 0xc044,
            Tile::Body => 0xa000,
            Tile::Blue(Side::Right) => 0xc041,
            Tile::Blue(Side::Left) => 0xc040,
            Tile::Extension(distance) => 0xd000 | u16::from(0u8.wrapping_sub(distance)),
        },
    );
}
