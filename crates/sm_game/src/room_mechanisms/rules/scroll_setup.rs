//! Extension placement and scroll-mechanism activation.
use super::super::access::*;
use super::super::host::*;
use super::terrain_setup as terrain;
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

pub fn setup(c: &mut impl Host, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::Place(tile) => {
            terrain::install(
                c,
                mechanism!(c, slot, block),
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
                Tile::Extension(_) => mechanism!(c, slot, header = 0),
                Tile::Scroll(_) => {
                    mechanism!(c, slot, work0 = 0);
                    {
                        let index = slot;
                        let value = mechanism!(c, index, program).wrapping_add(4);
                        mechanism!(c, index, program = value);
                    };
                }
            }
        }
        Setup::Touch => {
            let block = mechanism!(c, slot, block);
            mechanism!(c, slot, block = 0);
            let Some(found) = (0..40)
                .rev()
                .find(|&found| mechanism!(c, found, block) == block)
            else {
                // Preserve the missing-scroll-target nonreturn after clearing the trigger block.
                {
                    c.diagnostic(Diagnostic::MissingScroll);
                    loop {
                        core::hint::spin_loop();
                    }
                }
            };
            if mechanism!(c, found, work0) & 0x8000 == 0 {
                mechanism!(c, found, work0 = 0x8000);
                {
                    let index = found;
                    let value = mechanism!(c, index, program).wrapping_add(1);
                    mechanism!(c, index, program = value);
                };
                {
                    let index = found;
                    let value = mechanism!(c, index, program).wrapping_add(1);
                    mechanism!(c, index, program = value);
                };
                mechanism!(c, found, instruction_delay = 1);
            }
        }
    }
    0
}
