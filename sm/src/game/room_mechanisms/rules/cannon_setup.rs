//! Draygon cannon damage linkage and initial collision shape.
#![forbid(unsafe_code)]
use super::super::{fields::Fields, state::Column};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Right,
    Left,
    DiagonalRight,
    DiagonalLeft,
    Unshielded,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Right,
    Left,
}
fn adjacent(block: u16, side: Side) -> u16 {
    match side {
        Side::Right => block.wrapping_add(2),
        Side::Left => block.wrapping_sub(2),
    }
}

pub fn setup(c: &mut impl Fields, slot: usize, kind: Setup) -> u8 {
    c.set_slot(Column::Work0, slot, c.slot(Column::RoomArgument, slot));
    c.set_slot(
        Column::RoomArgument,
        slot,
        if kind == Setup::Unshielded { 3 } else { 0 },
    );
    if kind != Setup::Unshielded {
        let block = c.slot(Column::Block, slot);
        c.install(block, 0xc044);
        let side = match kind {
            Setup::DiagonalRight => Some(Side::Right),
            Setup::DiagonalLeft => Some(Side::Left),
            _ => None,
        };
        if let Some(side) = side {
            c.install(
                adjacent(block, side),
                if side == Side::Right { 0x50ff } else { 0x5001 },
            );
        }
        let bottom = c.below(block);
        c.install(bottom, 0xd0ff);
        if let Some(side) = side {
            c.install(adjacent(bottom, side), 0xd0ff);
        }
    }
    0
}
