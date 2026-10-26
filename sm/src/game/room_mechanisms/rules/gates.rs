//! Gate triggering, placement and shootblock selection.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::host::*;
use super::terrain_setup as terrain;

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

fn activate(c: &mut impl Host, slot: usize, side: Side) -> u8 {
    let block = neighbor(mechanism!(c, slot, block), side);
    for found in (0..40).rev() {
        if mechanism!(c, found, block) == block {
            if mechanism!(c, found, counter) == 0 {
                let value = mechanism!(c, found, counter).wrapping_add(1);
                mechanism!(c, found, counter = value);
            }
            break;
        }
    }
    mechanism!(c, slot, header = 0);
    1
}

fn trigger(c: &mut impl Host, slot: usize, side: Side, color: Color) -> u8 {
    let projectile = projectile_kind(
        c,
        (c.session().projectiles.selected >> 1) as isize,
        ByteOrder::Native,
    ) & 0xfff;
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
    mechanism!(c, slot, header = 0);
    if side == Side::Left && matches!(color, Color::Yellow | Color::Blue) {
        return trigger(c, slot, Side::Right, color);
    }
    0
}

pub fn setup(c: &mut impl Host, slot: usize, kind: Setup, accumulator: u16) -> u8 {
    match kind {
        Setup::Trigger(side, color) => return trigger(c, slot, side, color),
        Setup::Placement(direction, closed) => {
            if closed {
                c.enemy_projectile(
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
            let mut row = mechanism!(c, slot, block) >> 1;
            for _ in 0..5 {
                let next = behavior_byte(c, usize::from(row) + 1);
                set_behavior_pair(c, usize::from(row), u16::from(next) << 8 | 0x10);
                row = match direction {
                    Direction::Down => row.wrapping_add(c.session().room.width),
                    Direction::Up => row.wrapping_sub(c.session().room.width),
                };
            }
        }
        Setup::Shootblock(direction) => {
            let program = table(c, slot, direction, 0);
            mechanism!(c, slot, program = program);
            for side in [Side::Left, Side::Right] {
                let value = table(c, slot, direction, if side == Side::Left { 16 } else { 32 });
                if value != 0 {
                    let block = neighbor(mechanism!(c, slot, block), side);
                    terrain::install(c, block, value);
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
fn table(c: &impl Host, slot: usize, direction: Direction, offset: usize) -> u16 {
    c.table_word(
        ContentTable::Gate {
            up: direction == Direction::Up,
            byte_offset: offset,
        },
        (mechanism!(c, slot, room_argument) >> 1) as isize,
    )
}
