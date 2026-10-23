//! Checked quota fields; persistent progress and callback identities remain in RAM.
#![forbid(unsafe_code)]
use crate::game::mechanism_metroid_quota::{self, Context, Room};
use crate::game_state::Game;
pub struct QuotaFields<'a>(pub &'a mut Game);
impl QuotaFields<'_> {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0.g_ram[at], self.0.g_ram[at + 1]])
    }
}
impl Context for QuotaFields<'_> {
    type Callback = u16;
    fn selected_callback(&self, slot: usize) -> u16 {
        let rom = self
            .0
            .loaded_rom
            .as_ref()
            .expect("ROM must be loaded")
            .clone();
        let argument = self.word(0x1dc7 + 2 * slot);
        rom.table_word(0x25b28, (argument >> 1) as isize)
    }
    fn watch(&mut self, slot: usize, callback: u16) {
        let at = 0x1cd7 + 2 * slot;
        self.0.g_ram[at..at + 2].copy_from_slice(&callback.to_le_bytes());
    }
    fn deaths(&self) -> u16 {
        self.word(0xe50)
    }
    fn required(&self) -> u16 {
        self.word(0xe52)
    }
    fn cleared(&mut self, room: Room) {
        crate::game::progress_flags::SetEventHappened(
            match room {
                Room::First => 0x10,
                Room::Second => 0x11,
                Room::Third => 0x12,
                Room::Fourth => 0x13,
            },
            self.0,
        );
    }
}
pub fn begin(j: u16, game: &mut Game) -> u8 {
    mechanism_metroid_quota::begin(&mut QuotaFields(game), usize::from(j >> 1))
}
pub fn poll(room: Room, game: &mut Game) {
    mechanism_metroid_quota::poll(&mut QuotaFields(game), room);
}
