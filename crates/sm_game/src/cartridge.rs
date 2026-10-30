//! The cartridge images game modules read through.
use sm_data::cartridge::RomImage;

pub trait LoadedImage {
    /// The loaded image; panics with "ROM must be loaded" when none is.
    fn loaded_image(&self) -> &RomImage;
}

pub trait NativeImage {
    /// The exact native allocation; panics with "native ROM must be
    /// registered" when none is.
    fn native_image(&self) -> RomImage;
}

/// The native image with the original's invalid-address diagnostic.
pub trait NativeDiagnostics: NativeImage {
    /// The original diagnostic for a native read below $8000 in its bank.
    fn native_rom_diagnostic(&mut self, address: u32);
}

/// The word at signed word `index` from LoROM `address` in the loaded image,
/// as the compat `rom_table_word!` reads it.
#[macro_export]
macro_rules! cartridge_table_word {
    ($game:ident, $address:expr, $index:expr $(,)?) => {
        $crate::cartridge::LoadedImage::loaded_image(&*$game).table_word(
            ::sm_data::cartridge::lorom_offset($address) as usize,
            $index,
        )
    };
}
pub use crate::cartridge_table_word as rom_table_word;
