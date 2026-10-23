//! Owned ROM cursor for enemy-projectile instructions and explicit stop/jump results.
#![forbid(unsafe_code)]
use crate::rom::RomImage;

// The original executor subtracts 0x8000 from bank 86's mapped window.
const BANK_BASE: usize = 0x28000;

pub struct EprojScript {
    image: RomImage,
    position: usize,
}

impl EprojScript {
    pub fn new(image: RomImage, address: u16) -> Self {
        Self {
            image,
            position: BANK_BASE + usize::from(address),
        }
    }

    pub fn seek(&mut self, position: usize) {
        self.position = position;
    }
    pub fn bank_address(&self) -> u16 {
        self.position.wrapping_sub(BANK_BASE) as u16
    }

    pub fn byte(&self, offset: isize) -> u8 {
        self.image.read(
            self.position
                .checked_add_signed(offset)
                .expect("instruction offset overflow"),
        )
    }

    pub fn word(&self, offset: isize) -> u16 {
        u16::from_ne_bytes([self.byte(offset), self.byte(offset + 1)])
    }

    pub fn long(&self) -> u32 {
        u32::from(self.word(0)) | (u32::from(self.byte(2)) << 16)
    }

    // Sequential and relative movement remain linear in the captured allocation.
    pub fn offset(&self, bytes: isize) -> Option<usize> {
        Some(
            self.position
                .checked_add_signed(bytes)
                .expect("instruction offset overflow"),
        )
    }

    // A numeric jump to zero stops; linear movement to bank address zero does not.
    pub fn jump(&self, address: u16) -> Option<usize> {
        (address != 0).then_some(BANK_BASE + usize::from(address))
    }
}
