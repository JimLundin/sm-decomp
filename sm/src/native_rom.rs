//! Raw-pointer view of the shared cartridge image for native and legacy consumers.
#![forbid(unsafe_code)]
use crate::rom::RomImage;

/// Callers name this boundary explicitly; owned code reads through `RomImage`.
pub(crate) trait NativeRomPointer {
    fn as_mut_ptr(&self) -> *mut u8;
}

impl NativeRomPointer for RomImage {
    // Legacy cartridge and patch consumers borrow this allocation. Cell keeps
    // its bytes mutable while shared owners exist; access is serialized by Game.
    fn as_mut_ptr(&self) -> *mut u8 {
        self.cells().as_ptr().cast::<u8>().cast_mut()
    }
}
