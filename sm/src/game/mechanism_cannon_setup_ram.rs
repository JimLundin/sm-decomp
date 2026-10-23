//! Checked encoding for cannon damage links and initial collision shapes.
#![forbid(unsafe_code)]
use crate::game::mechanism_cannon_setup::{self, Context, Setup, Side, Tile};
use crate::game_state::Game;
pub struct CannonFields<'a>(pub &'a mut Game);
impl CannonFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Context for CannonFields<'_> {
    type Block = u16;
    type DamageLink = u16;
    fn damage_link(&self, slot: usize) -> u16 {
        self.word(0x1dc7 + 2 * slot)
    }
    fn save_damage_link(&mut self, slot: usize, link: u16) {
        self.put(0x1e17 + 2 * slot, link);
    }
    fn set_hits(&mut self, slot: usize, hits: u16) {
        self.put(0x1dc7 + 2 * slot, hits);
    }
    fn block(&self, slot: usize) -> u16 {
        self.word(0x1c87 + 2 * slot)
    }
    fn adjacent(&self, block: u16, side: Side) -> u16 {
        match side {
            Side::Right => block.wrapping_add(2),
            Side::Left => block.wrapping_sub(2),
        }
    }
    fn below(&self, block: u16) -> u16 {
        block.wrapping_add(self.word(0x7a5).wrapping_mul(2))
    }
    fn install(&mut self, block: u16, tile: Tile) {
        crate::game::sm_84::WriteLevelDataBlockTypeAndBts(
            block,
            match tile {
                Tile::Shield => 0xc044,
                Tile::Horizontal(Side::Right) => 0x50ff,
                Tile::Horizontal(Side::Left) => 0x5001,
                Tile::Below => 0xd0ff,
            },
            self.0,
        );
    }
}
pub fn setup(j: u16, kind: Setup, game: &mut Game) -> u8 {
    mechanism_cannon_setup::setup(&mut CannonFields(game), usize::from(j >> 1), kind)
}
