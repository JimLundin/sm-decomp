//! Decode native audio uploads while retaining the source ROM allocation.
#![forbid(unsafe_code)]
use crate::rom::RomImage;

pub fn transfer(image: RomImage, mut source: usize, mut write: impl FnMut(u16, u8)) {
    let mut byte = || {
        let value = image.read(source);
        source = source.checked_add(1).expect("audio upload offset overflow");
        value
    };
    loop {
        // Retain the native byte order of the original word loads.
        let count = u16::from_ne_bytes([byte(), byte()]);
        if count == 0 {
            return;
        }
        let mut target = u16::from_ne_bytes([byte(), byte()]);
        for _ in 0..count {
            write(target, byte());
            target = target.wrapping_add(1);
        }
    }
}
