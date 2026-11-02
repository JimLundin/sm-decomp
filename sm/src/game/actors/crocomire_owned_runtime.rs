//! Crocomire's finite compatibility resources/effects on the existing actor owner.
//! Included as a child of session_runtime; it never admits another Session.
#![forbid(unsafe_code)]
use super::Runtime;
use sm_data::crocomire::{
    BRIDGE_DEBRIS_ADDRESS, MELT_TILES, SCROLL_SPRITEMAP_ADDRESS, SPIKE_WALL_DESTINATION_ADDRESS,
    SPIKE_WALL_TILE_ADDRESS,
};
use sm_game::actors::crocomire_reactions::{CrocomireHost, Host as ResourceHost};

impl CrocomireHost for Runtime<'_> {
    fn crocomire_word(&self, address: u32, index: isize) -> u16 {
        match address {
            SCROLL_SPRITEMAP_ADDRESS
            | BRIDGE_DEBRIS_ADDRESS
            | SPIKE_WALL_TILE_ADDRESS
            | SPIKE_WALL_DESTINATION_ADDRESS => self.game.crocomire_table_word(address, index),
            _ => self.loaded_image().expect("ROM must be loaded").table_word(
                sm_data::cartridge::lorom_offset(address).unsigned_abs(),
                index,
            ),
        }
    }
    fn native_crocomire_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        sm_game::cartridge::native_record(bank, address, self)
    }
    fn melt_tilemap(&mut self, table: u32) -> Option<[u16; MELT_TILES]> {
        self.game.crocomire_melt_tilemap(table)
    }
    fn low_mirror_byte(&self, at: usize) -> u8 {
        self.byte(at)
    }
    fn bg2_screen(&self) -> u8 {
        self.byte(0x59)
    }
    fn bg12_characters(&self) -> u8 {
        self.byte(0x52)
    }
    fn scroll_hdma_word(&mut self, address: usize, value: u16, native: bool) {
        let bytes = if native {
            value.to_ne_bytes()
        } else {
            value.to_le_bytes()
        };
        for (lane, byte) in bytes.into_iter().enumerate() {
            // The original byte publication refreshes observations after each store.
            self.set_byte(address + lane, byte);
        }
    }
    fn melting_pixel_byte(&self, at: usize) -> u8 {
        self.byte(at)
    }
    fn set_melting_pixel_byte(&mut self, at: usize, value: u8) {
        self.set_byte(at, value);
    }
    fn free_scroll_hdma(&mut self, object: u16) {
        let index = usize::from(object >> 1);
        if index < sm_game::hdma::OBJECTS {
            sm_game::hdma_word!(self, index, Channels => 0);
        } else {
            let at = 0x18b4 + 2 * index;
            self.set_byte(at, 0);
            self.set_byte(at + 1, 0);
        }
    }
    fn disable_minimap(&mut self) {
        sm_game::player_update::minimap::disable_for_boss(self);
    }
    fn guard<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        self.owned_call(action)
    }
}
