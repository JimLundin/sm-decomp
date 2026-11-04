//! Compat endpoints for `sm_game::word_transfers`' ROM reads.
#![forbid(unsafe_code)]

impl sm_game::word_transfers::LoadedRom for crate::game::session_runtime::Runtime<'_> {
    fn loaded_rom_image(&self) -> crate::rom::RomImage {
        sm_game::cartridge::LoadedImage::loaded_image(self).clone()
    }
}

// Callers retain any legacy address diagnostics before capturing the image.
