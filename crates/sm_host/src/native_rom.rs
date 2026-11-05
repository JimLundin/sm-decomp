//! Raw-pointer view of the shared cartridge image for native and legacy consumers.
#![forbid(unsafe_code)]
use crate::rom::RomImage;

/// Callers name this boundary explicitly; owned code reads through `RomImage`.
pub(crate) trait NativeRomPointer {
    fn as_ptr(&self) -> *const u8;
}

impl NativeRomPointer for RomImage {
    // Legacy consumers only read this allocation; the image is immutable.
    fn as_ptr(&self) -> *const u8 {
        self.bytes().as_ptr()
    }
}
