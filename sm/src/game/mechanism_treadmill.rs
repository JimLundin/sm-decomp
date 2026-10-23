//! Entrance conveyor installation and its conditional push on Samus.
#![forbid(unsafe_code)]
#[derive(Clone, Copy)]
pub enum Direction {
    Right,
    Left,
}
#[derive(Clone, Copy)]
pub enum Surface {
    Air,
    Conveyor,
}
pub trait Terrain {
    type Block: Copy;
    fn block(&self, slot: usize) -> Self::Block;
    fn next_clear(&self, block: Self::Block) -> Self::Block;
    fn row_cell(&self, origin: Self::Block, index: usize) -> Self::Block;
    fn set_level(&mut self, block: Self::Block, surface: Surface);
    fn set_direction(&mut self, block: Self::Block, direction: Direction);
}
pub fn setup(c: &mut impl Terrain, slot: usize) -> u8 {
    let mut block = c.block(slot);
    for _ in 0..56 {
        c.set_level(block, Surface::Air);
        block = c.next_clear(block);
    }
    0
}
pub fn activate(c: &mut impl Terrain, slot: usize, direction: Direction) {
    let origin = c.block(slot);
    for index in 0..56 {
        let block = c.row_cell(origin, index);
        c.set_level(block, Surface::Conveyor);
        c.set_direction(block, direction);
    }
}
pub trait Motion {
    fn area(&self) -> u16;
    fn boss_defeated(&mut self) -> bool;
    fn extra_run_speed(&self) -> u16;
    fn clear_fraction(&mut self);
    fn set_displacement(&mut self, pixels: i16);
    fn normal_speed(&mut self);
}
pub fn carry(c: &mut impl Motion, direction: Direction) {
    if (c.area() != 3 || c.boss_defeated()) && c.extra_run_speed() == 0 {
        c.clear_fraction();
        c.set_displacement(match direction {
            Direction::Right => 2,
            Direction::Left => -2,
        });
    }
    c.normal_speed();
}
