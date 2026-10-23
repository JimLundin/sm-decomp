//! Checked item setup encoding, including native saved terrain and missing-target overlap.
#![forbid(unsafe_code)]
use crate::game::mechanism_item_setup::{self, Container, Context, Graphic, Setup};
use crate::game_state::Game;
pub struct ItemFields<'a>(pub &'a mut Game);
impl ItemFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Context for ItemFields<'_> {
    type Block = u16;
    type SavedBlock = u16;
    fn set_graphic(&mut self, slot: usize, graphic: Graphic) {
        self.put(
            0xdf0c + 2 * slot,
            match graphic {
                Graphic::EnergyTank => 8,
                Graphic::MissileTank => 10,
                Graphic::SuperMissileTank => 12,
                Graphic::PowerBombTank => 14,
            },
        );
    }
    fn block(&self, slot: usize) -> u16 {
        self.word(0x1c87 + 2 * slot)
    }
    fn clear_block(&mut self, slot: usize) {
        self.put(0x1c87 + 2 * slot, 0);
    }
    fn install(&mut self, block: u16, container: Container) {
        crate::game::sm_84::WriteLevelDataBlockTypeAndBts(
            block,
            match container {
                Container::Visible => 0x45,
                Container::ShotBlock => 0xc045,
            },
            self.0,
        );
    }
    fn level(&self, block: u16) -> u16 {
        let at = 0x10002 + usize::from(block & !1);
        u16::from_ne_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn save_level(&mut self, slot: usize, level: u16) {
        self.put(0x1e17 + 2 * slot, level);
    }
    fn loaded_count(&self) -> u16 {
        self.word(0xd91a)
    }
    fn set_loaded_count(&mut self, count: u16) {
        self.put(0xd91a, count);
    }
    fn frozen(&self) -> bool {
        self.word(0xa78) != 0
    }
    fn release(&mut self, slot: usize) {
        self.put(0x1c37 + 2 * slot, 0);
    }
    fn signal_target(&mut self, target: Option<usize>) {
        // The original signed scan ends at -2; timers[-1] aliases the preceding program word.
        self.put(target.map_or(0x1d75, |slot| 0x1d77 + 2 * slot), 255);
    }
}
pub fn setup(j: u16, kind: Setup, game: &mut Game) -> u8 {
    mechanism_item_setup::setup(&mut ItemFields(game), usize::from(j >> 1), kind)
}
