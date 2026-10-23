//! Entrance conveyor installation and its conditional push on Samus.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, Global, WordOrder},
    state::Column,
};
#[derive(Clone, Copy)]
pub enum Direction {
    Right,
    Left,
}

pub fn setup(c: &mut impl Fields, slot: usize) -> u8 {
    let mut block = usize::from(c.slot(Column::Block, slot));
    for _ in 0..56 {
        c.set_wide_level(block, 0xff, WordOrder::Little);
        block = usize::from((block as u16).wrapping_add(2));
    }
    0
}
pub fn activate(c: &mut impl Fields, slot: usize, direction: Direction) {
    let origin = usize::from(c.slot(Column::Block, slot));
    for index in 0..56 {
        let block = origin + 2 * index;
        c.set_wide_level(block, 0x30ff, WordOrder::Native);
        c.set_wide_behavior(
            block,
            match direction {
                Direction::Right => 8,
                Direction::Left => 9,
            },
        );
    }
}

pub fn carry(c: &mut impl Fields, direction: Direction) {
    if (c.global(Global::Area) != 3 || c.boss(1) & 1 != 0)
        && c.global(Global::ConveyorGateSpeed) == 0
    {
        c.set_global(Global::ExternalXFraction, 0);
        c.set_global(
            Global::ExternalX,
            (match direction {
                Direction::Right => 2,
                Direction::Left => -2,
            }) as u16,
        );
    }
    c.set_global(Global::MovementSpeedTable, 0x9f49u16.wrapping_add(12));
}
