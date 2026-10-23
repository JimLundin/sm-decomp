//! Eye-door placement and transition after its persistent open flag is set.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, Global},
    state::Column,
};
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

pub fn begin(c: &mut impl Fields, slot: usize, part: Part) -> u8 {
    if !c.door_opened(c.slot(Column::RoomArgument, slot)) {
        install(
            c.slot(Column::Block, slot),
            if part == Part::Eye {
                Tile::Eye
            } else {
                Tile::Body
            },
            c,
        );
        if part == Part::Eye {
            let row = c.global(Global::RoomWidth).wrapping_mul(2);
            let block = c.slot(Column::Block, slot);
            install((block).wrapping_add(row), Tile::Extension(1), c);
        }
    }
    0
}
pub fn wake(c: &mut impl Fields, slot: usize) {
    if c.door_opened(c.slot(Column::RoomArgument, slot)) {
        c.set_slot(Column::PreAction, slot, 0xd779);
        c.set_slot(Column::Program, slot, c.slot(Column::ReturnProgram, slot));
        c.set_slot(Column::InstructionDelay, slot, 1);
    }
}
pub fn convert(c: &mut impl Fields, slot: usize, side: Side) {
    let old = c.slot(Column::Block, slot);
    let row = c.global(Global::RoomWidth).wrapping_mul(2);
    c.set_slot(Column::Block, slot, (old).wrapping_sub(row));
    let mut block = c.slot(Column::Block, slot);
    install(block, Tile::Blue(side), c);
    for distance in 1..=3 {
        block = (block).wrapping_add(c.global(Global::RoomWidth).wrapping_mul(2));
        install(block, Tile::Extension(distance), c);
    }
}

fn install(block: u16, tile: Tile, c: &mut impl Fields) {
    c.install(
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
