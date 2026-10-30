//! Compat endpoints for `sm_game::word_transfers!`.
#![forbid(unsafe_code)]
use crate::game_state::Game;
pub(crate) use sm_game::word_transfers::WordEndpoint;

impl sm_game::word_transfers::LoadedRom for Game {
    fn loaded_rom_image(&self) -> crate::rom::RomImage {
        self.loaded_rom
            .as_ref()
            .expect("ROM must be loaded")
            .clone()
    }
}

impl sm_game::word_transfers::LoadedRom for crate::game::session_runtime::Runtime<'_> {
    fn loaded_rom_image(&self) -> crate::rom::RomImage {
        sm_game::cartridge::LoadedImage::loaded_image(self).clone()
    }
}

// Callers retain any legacy address diagnostics before capturing the image.
pub(crate) fn rom_source(address: u32, game: &mut Game) -> WordEndpoint<'static> {
    WordEndpoint::Rom(
        game.loaded_rom
            .as_ref()
            .expect("ROM must be loaded")
            .clone(),
        crate::rtl::lorom_offset(address),
    )
}
