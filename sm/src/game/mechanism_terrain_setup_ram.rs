//! Scoped encoded field access for breakable terrain setup.
#![forbid(unsafe_code)]
use crate::game::mechanism_terrain_setup::{
    self, BlockReaction, Context, ProjectileWord, RejectionProgram, Setup,
};
use crate::game_state::Game;

pub fn install_block(k: u16, a: u16, game: &mut Game) {
    // The level-data byte uses the exact offset, while BTS rounds down to a word.
    let block_high = 0x10003 + usize::from(k);
    game.g_ram[block_high] = (a >> 8) as u8 | (game.g_ram[block_high] & 0x0f);
    game.g_ram[0x16402 + usize::from(k >> 1)] = a as u8;
}

pub struct TerrainFields<'a>(pub &'a mut Game);
impl TerrainFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Context for TerrainFields<'_> {
    type Block = u16;
    fn block(&self, slot: usize) -> u16 {
        self.word(0x1c87 + 2 * slot)
    }
    fn release(&mut self, slot: usize) {
        self.put(0x1c37 + 2 * slot, 0);
    }
    fn level(&self, block: u16) -> u16 {
        self.word(0x10002 + 2 * usize::from(block >> 1))
    }
    fn set_level(&mut self, block: u16, value: u16) {
        self.put(0x10002 + 2 * usize::from(block >> 1), value);
    }
    fn install(&mut self, block: u16, reaction: BlockReaction) {
        install_block(
            block,
            match reaction {
                BlockReaction::Shot => 0xc000,
                BlockReaction::Grapple => 0xe000,
            },
            self.0,
        );
    }
    fn set_saved(&mut self, slot: usize, value: u16) {
        self.put(0x1e17 + 2 * slot, value);
    }
    fn instruction(&self, slot: usize) -> u16 {
        self.word(0x1d27 + 2 * slot)
    }
    fn set_instruction(&mut self, slot: usize, value: u16) {
        self.put(0x1d27 + 2 * slot, value);
    }
    fn reject_program(&mut self, slot: usize, program: RejectionProgram) {
        use crate::game::constants::bank_84::{addr_kPlmInstrList_C91C, addr_kPlmInstrList_C922};
        self.set_instruction(
            slot,
            match program {
                RejectionProgram::PowerBomb => addr_kPlmInstrList_C91C as u16,
                RejectionProgram::SuperMissile => addr_kPlmInstrList_C922 as u16,
            },
        );
    }
    fn set_delay(&mut self, slot: usize, value: u16) {
        self.put(0xde1c + 2 * slot, value);
    }
    fn speed_boost(&self) -> u16 {
        self.word(0xb3e)
    }
    fn pose(&self) -> u16 {
        self.word(0xa1c)
    }
    fn direction(&self) -> u16 {
        self.word(0xb02)
    }
    fn projectile_selection(&self) -> u16 {
        self.word(0xdde)
    }
    fn projectile_word(&self, index: usize, order: ProjectileWord) -> u16 {
        let at = 0xc18 + 2 * index;
        let bytes = [self.0.g_ram[at], self.0.g_ram[at + 1]];
        match order {
            ProjectileWord::Native => u16::from_ne_bytes(bytes),
            ProjectileWord::LittleEndian => u16::from_le_bytes(bytes),
        }
    }
    fn next_behavior(&self, block: u16) -> u8 {
        self.0.g_ram[0x16403 + usize::from(block >> 1)]
    }
    fn set_behavior_word(&mut self, block: u16, value: u16) {
        let at = 0x16402 + usize::from(block >> 1);
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_ne_bytes());
    }
}
pub fn setup(j: u16, kind: Setup, game: &mut Game) -> u8 {
    mechanism_terrain_setup::setup(&mut TerrainFields(game), usize::from(j >> 1), kind)
}
