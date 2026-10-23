//! Extension placement and scroll-mechanism activation.
#![forbid(unsafe_code)]
use super::super::{fields::Fields, state::Column};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Right,
    Left,
    Down,
    Up,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Solidity {
    Air,
    Solid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Extension(Direction),
    Scroll(Solidity),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Place(Tile),
    Touch,
}

pub fn setup(c: &mut impl Fields, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::Place(tile) => {
            c.install(
                c.slot(Column::Block, slot),
                match tile {
                    Tile::Extension(Direction::Right) => 0x50ff,
                    Tile::Extension(Direction::Left) => 0x5001,
                    Tile::Extension(Direction::Down) => 0xd0ff,
                    Tile::Extension(Direction::Up) => 0xd001,
                    Tile::Scroll(Solidity::Air) => 0x3046,
                    Tile::Scroll(Solidity::Solid) => 0xb046,
                },
            );
            match tile {
                Tile::Extension(_) => c.release(slot),
                Tile::Scroll(_) => {
                    c.set_slot(Column::Work0, slot, 0);
                    c.advance(slot, 4);
                }
            }
        }
        Setup::Touch => {
            let block = c.slot(Column::Block, slot);
            c.set_slot(Column::Block, slot, 0);
            let Some(found) = (0..40)
                .rev()
                .find(|&found| c.slot(Column::Block, found) == block)
            else {
                // Preserve the missing-scroll-target nonreturn after clearing the trigger block.
                c.missing_scroll()
            };
            if c.slot(Column::Work0, found) & 0x8000 == 0 {
                c.set_slot(Column::Work0, found, 0x8000);
                c.advance(found, 1);
                c.advance(found, 1);
                c.set_slot(Column::InstructionDelay, found, 1);
            }
        }
    }
    0
}
