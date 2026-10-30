//! Draygon cannon damage linkage and initial collision shape.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::host::*;
use super::terrain_setup as terrain;
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

pub fn setup(c: &mut impl Host, slot: usize, kind: Setup) -> u8 {
    mechanism!(c, slot, work0 = mechanism!(c, slot, room_argument));
    mechanism!(
        c,
        slot,
        room_argument = if kind == Setup::Unshielded { 3 } else { 0 }
    );
    if kind != Setup::Unshielded {
        let block = mechanism!(c, slot, block);
        terrain::install(c, block, 0xc044);
        let side = match kind {
            Setup::DiagonalRight => Some(Side::Right),
            Setup::DiagonalLeft => Some(Side::Left),
            _ => None,
        };
        if let Some(side) = side {
            terrain::install(
                c,
                adjacent(block, side),
                if side == Side::Right { 0x50ff } else { 0x5001 },
            );
        }
        let bottom = (block).wrapping_add(c.session().room.width.wrapping_mul(2));
        terrain::install(c, bottom, 0xd0ff);
        if let Some(side) = side {
            terrain::install(c, adjacent(bottom, side), 0xd0ff);
        }
    }
    0
}
