//! Gate triggering, placement and shootblock selection.
#![forbid(unsafe_code)]

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

pub trait Context {
    type Block: Copy + PartialEq;
    type Row: Copy;
    type Program;
    fn block(&self, slot: usize) -> Self::Block;
    fn neighbor(&self, block: Self::Block, side: Side) -> Self::Block;
    fn timer(&self, slot: usize) -> u16;
    fn set_timer(&mut self, slot: usize, value: u16);
    fn release(&mut self, slot: usize);
    fn projectile_type(&self) -> u16;
    fn rejection_sound(&mut self);
    fn spawn_closed_gate(&mut self, direction: Direction, accumulator: u16);
    fn row(&self, block: Self::Block) -> Self::Row;
    fn next_behavior(&self, row: Self::Row) -> u8;
    fn set_behavior_word(&mut self, row: Self::Row, value: u16);
    fn advance_row(&self, row: Self::Row, direction: Direction) -> Self::Row;
    fn program(&self, slot: usize, direction: Direction) -> Self::Program;
    fn set_program(&mut self, slot: usize, program: Self::Program);
    fn optional_block(&self, slot: usize, direction: Direction, side: Side) -> u16;
    fn install_block(&mut self, block: Self::Block, value: u16);
}

fn activate(c: &mut impl Context, slot: usize, side: Side) -> u8 {
    let block = c.neighbor(c.block(slot), side);
    for found in (0..40).rev() {
        if c.block(found) == block {
            if c.timer(found) == 0 {
                let value = c.timer(found).wrapping_add(1);
                c.set_timer(found, value);
            }
            break;
        }
    }
    c.release(slot);
    1
}

fn trigger(c: &mut impl Context, slot: usize, side: Side, color: Color) -> u8 {
    let projectile = c.projectile_type() & 0xfff;
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
        c.rejection_sound();
    }
    c.release(slot);
    if side == Side::Left && matches!(color, Color::Yellow | Color::Blue) {
        return trigger(c, slot, Side::Right, color);
    }
    0
}

pub fn setup(c: &mut impl Context, slot: usize, kind: Setup, accumulator: u16) -> u8 {
    match kind {
        Setup::Trigger(side, color) => return trigger(c, slot, side, color),
        Setup::Placement(direction, closed) => {
            if closed {
                c.spawn_closed_gate(
                    direction,
                    if direction == Direction::Down {
                        accumulator
                    } else {
                        0
                    },
                );
            }
            let mut row = c.row(c.block(slot));
            for _ in 0..5 {
                let next = c.next_behavior(row);
                c.set_behavior_word(row, u16::from(next) << 8 | 0x10);
                row = c.advance_row(row, direction);
            }
        }
        Setup::Shootblock(direction) => {
            let program = c.program(slot, direction);
            c.set_program(slot, program);
            for side in [Side::Left, Side::Right] {
                let value = c.optional_block(slot, direction, side);
                if value != 0 {
                    let block = c.neighbor(c.block(slot), side);
                    c.install_block(block, value);
                }
            }
        }
    }
    0
}
