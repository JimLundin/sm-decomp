//! Owned enemy-script positions and explicit interpreter control results.
#![forbid(unsafe_code)]
use crate::rom::RomImage;

#[derive(Clone)]
pub enum EnemyScript {
    // Sequential movement retains this allocation across callbacks.
    At { image: RomImage, position: usize },
    // Numeric targets resolve against the interpreter's captured bank and image.
    Jump(u16),
    Stop,
}

pub(crate) fn bank_base(bank: u8) -> isize {
    crate::rom::lorom_offset((u32::from(bank) << 16) | 0x8000) - 0x8000
}

impl EnemyScript {
    pub fn at(image: RomImage, position: usize) -> Self {
        Self::At { image, position }
    }
    pub fn jump(address: u16) -> Self {
        // A zero jump stops; a linear position at bank address zero does not.
        if address == 0 {
            Self::Stop
        } else {
            Self::Jump(address)
        }
    }
    fn cursor(&self) -> (&RomImage, usize) {
        match self {
            Self::At { image, position } => (image, *position),
            _ => panic!("script control result is not a readable position"),
        }
    }
    pub fn byte(&self, offset: isize) -> u8 {
        let (image, position) = self.cursor();
        image.read(
            position
                .checked_add_signed(offset)
                .expect("enemy script offset overflow"),
        )
    }
    pub fn word(&self, offset: isize) -> u16 {
        // Original script words use native byte order, including unaligned data.
        u16::from_ne_bytes([self.byte(offset), self.byte(offset + 1)])
    }
    pub fn load24(&self, offset: isize) -> u32 {
        // Load24 reads the adjacent fourth byte before masking the native word.
        u32::from_ne_bytes(core::array::from_fn(|i| self.byte(offset + i as isize))) & 0xffffff
    }
    pub fn advance(&self, bytes: isize) -> Self {
        let (image, position) = self.cursor();
        Self::at(
            image.clone(),
            position
                .checked_add_signed(bytes)
                .expect("enemy script offset overflow"),
        )
    }
    pub fn offset(&self, words: isize) -> Self {
        self.advance(words.checked_mul(2).expect("enemy script offset overflow"))
    }
    pub fn offset_from(&self, base: isize) -> isize {
        isize::try_from(self.cursor().1).expect("enemy script offset overflow") - base
    }
    pub fn resolve(self, image: &RomImage, base: isize) -> Option<Self> {
        match self {
            Self::Stop => None,
            Self::Jump(address) => Some(Self::at(
                image.clone(),
                usize::try_from(base + address as isize).expect("enemy script jump outside ROM"),
            )),
            cursor => Some(cursor),
        }
    }
}
