//! Brinstar plants capture and hold the player through damage and recovery.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Floor,
    Ceiling,
}
pub trait Context {
    type Block;
    fn radius(&self) -> u16;
    fn player_x(&self) -> u16;
    fn player_y(&self) -> u16;
    fn block(&self, slot: usize) -> Self::Block;
    fn deactivate(&mut self, block: Self::Block);
    fn save_x(&mut self, slot: usize, x: u16);
    fn save_y(&mut self, slot: usize, y: u16);
    fn saved_x(&self, slot: usize) -> u16;
    fn saved_y(&self, slot: usize) -> u16;
    fn set_player_x(&mut self, x: u16);
    fn set_player_y(&mut self, y: u16);
    fn protection(&self) -> u16;
    fn set_protection(&mut self, frames: u16);
    fn damage(&self) -> u16;
    fn set_damage(&mut self, damage: u16);
    fn release(&mut self, slot: usize);
}
pub fn begin(c: &mut impl Context, slot: usize, surface: Surface) -> u8 {
    let aligned = match surface {
        Surface::Floor => c.radius().wrapping_add(c.player_y()) & 15 == 0,
        Surface::Ceiling => c.player_y().wrapping_sub(c.radius()) & 15 == 0,
    };
    if aligned {
        c.deactivate(c.block(slot));
        c.save_x(slot, c.player_x());
        let y = match surface {
            Surface::Floor => c.player_y().wrapping_sub(1),
            Surface::Ceiling => c.player_y().wrapping_add(1),
        };
        c.save_y(slot, y);
    } else {
        c.release(slot);
    }
    0
}
pub fn hold(c: &mut impl Context, slot: usize) {
    c.set_player_x(c.saved_x(slot));
    c.set_player_y(c.saved_y(slot));
    c.set_protection(c.protection() | 0x10);
}
pub fn hurt(c: &mut impl Context) {
    c.set_damage(c.damage().wrapping_add(2));
}
pub fn protect(c: &mut impl Context) {
    c.set_protection(48);
}
