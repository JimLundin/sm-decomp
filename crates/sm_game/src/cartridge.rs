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
