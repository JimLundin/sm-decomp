//! Checked fields while periodic damage and health remain RAM-authoritative.
#![forbid(unsafe_code)]
use crate::game::samus_periodic_damage::{self, Context, InvalidDamage};
use crate::game_state::Game;
use crate::ram_map::words as ram;

pub struct DamageFields<'a>(pub &'a mut Game);
impl DamageFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
    fn put(&mut self, at: usize, value: u16) {
        self.0.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
impl Context for DamageFields<'_> {
    fn frozen(&self) -> bool {
        self.word(ram::time_is_frozen_flag) != 0
    }
    fn pending_damage(&self) -> u32 {
        (u32::from(self.pending_whole()) << 16)
            | u32::from(self.word(ram::samus_periodic_subdamage))
    }
    fn pending_whole(&self) -> u16 {
        self.word(ram::samus_periodic_damage)
    }
    fn set_pending_whole(&mut self, value: u16) {
        self.put(ram::samus_periodic_damage, value);
    }
    fn equipment(&self) -> u16 {
        self.word(ram::equipped_items)
    }
    fn health(&self) -> u32 {
        // The former AddToHiLo reads native low then high, unlike pending damage.
        let low = u16::from_ne_bytes([
            self.0.g_ram[ram::samus_subunit_health],
            self.0.g_ram[ram::samus_subunit_health + 1],
        ]);
        let high = u16::from_ne_bytes([
            self.0.g_ram[ram::samus_health],
            self.0.g_ram[ram::samus_health + 1],
        ]);
        u32::from(low) | (u32::from(high) << 16)
    }
    fn set_health(&mut self, value: u32) {
        self.0.g_ram[ram::samus_subunit_health..ram::samus_subunit_health + 2]
            .copy_from_slice(&(value as u16).to_ne_bytes());
        self.0.g_ram[ram::samus_health..ram::samus_health + 2]
            .copy_from_slice(&((value >> 16) as u16).to_ne_bytes());
    }
    fn health_is_negative(&self) -> bool {
        // The later sign test returns to the ordinary little-endian field view.
        self.word(ram::samus_health) & 0x8000 != 0
    }
    fn clear_health(&mut self) {
        self.put(ram::samus_subunit_health, 0);
        self.put(ram::samus_health, 0);
    }
    fn clear_damage(&mut self) {
        self.put(ram::samus_periodic_subdamage, 0);
        self.put(ram::samus_periodic_damage, 0);
    }
}
pub fn grapple_contact(game: &mut Game) -> u8 {
    samus_periodic_damage::grapple_contact(&mut DamageFields(game))
}
pub fn apply(game: &mut Game) -> Result<(), InvalidDamage> {
    samus_periodic_damage::apply(&mut DamageFields(game))
}
