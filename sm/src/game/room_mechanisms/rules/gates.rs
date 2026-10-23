//! Gate triggering, placement and shootblock selection.
#![forbid(unsafe_code)]
use super::super::{
    fields::{ContentTable, Fields, Global, SoundChannel},
    state::Column,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Green,
    Red,
    Yellow,
    Blue,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Trigger(Side, Color),
    Placement(Direction, bool),
    Shootblock(Direction),
}

fn activate(c: &mut impl Fields, slot: usize, side: Side) -> u8 {
    let block = neighbor(c.slot(Column::Block, slot), side);
    for found in (0..40).rev() {
        if c.slot(Column::Block, found) == block {
            if c.slot(Column::Counter, found) == 0 {
                let value = c.slot(Column::Counter, found).wrapping_add(1);
                c.set_slot(Column::Counter, found, value);
            }
            break;
        }
    }
    c.release(slot);
    1
}

fn trigger(c: &mut impl Fields, slot: usize, side: Side, color: Color) -> u8 {
    let projectile =
        c.projectile_word((c.global(Global::ProjectileSelection) >> 1) as isize) & 0xfff;
    let accepted = match color {
        Color::Green => projectile == 0x200,
        Color::Red => matches!(projectile, 0x100 | 0x200),
        Color::Yellow if side == Side::Left => projectile == 0x300,
        Color::Yellow | Color::Blue => projectile != 0x300,
    };
    if accepted {
        return activate(
            c,
            slot,
            if side == Side::Left {
                Side::Right
            } else {
                Side::Left
            },
        );
    }
    if color != Color::Blue {
        c.sound(SoundChannel::Two, 6, 0x57);
    }
    c.release(slot);
    if side == Side::Left && matches!(color, Color::Yellow | Color::Blue) {
        return trigger(c, slot, Side::Right, color);
    }
    0
}

pub fn setup(c: &mut impl Fields, slot: usize, kind: Setup, accumulator: u16) -> u8 {
    match kind {
        Setup::Trigger(side, color) => return trigger(c, slot, side, color),
        Setup::Placement(direction, closed) => {
            if closed {
                c.eproj(
                    if direction == Direction::Down {
                        0xe659
                    } else {
                        0xe675
                    },
                    if direction == Direction::Down {
                        accumulator
                    } else {
                        0
                    },
                );
            }
            let mut row = c.slot(Column::Block, slot) >> 1;
            for _ in 0..5 {
                let next = c.behavior_at(usize::from(row), 1);
                c.set_behavior_at(usize::from(row), u16::from(next) << 8 | 0x10);
                row = match direction {
                    Direction::Down => row.wrapping_add(c.global(Global::RoomWidth)),
                    Direction::Up => row.wrapping_sub(c.global(Global::RoomWidth)),
                };
            }
        }
        Setup::Shootblock(direction) => {
            let program = table(c, slot, direction, 0);
            c.set_slot(Column::Program, slot, program);
            for side in [Side::Left, Side::Right] {
                let value = table(c, slot, direction, if side == Side::Left { 16 } else { 32 });
                if value != 0 {
                    let block = neighbor(c.slot(Column::Block, slot), side);
                    c.install(block, value);
                }
            }
        }
    }
    0
}

fn neighbor(block: u16, side: Side) -> u16 {
    match side {
        Side::Left => block.wrapping_sub(2),
        Side::Right => block.wrapping_add(2),
    }
}
fn table(c: &impl Fields, slot: usize, direction: Direction, offset: usize) -> u16 {
    c.content_word(
        ContentTable::Gate {
            up: direction == Direction::Up,
            offset,
        },
        (c.slot(Column::RoomArgument, slot) >> 1) as isize,
    )
}
