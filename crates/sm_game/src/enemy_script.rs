//! Owned enemy-script positions and explicit interpreter control results.
use sm_data::cartridge::RomImage;
use sm_data::program_image::ProgramImage;
use std::rc::Rc;

/// What a script cursor reads: the prepared program image, or the live
/// cartridge image where none serves it.
#[derive(Clone)]
pub enum ScriptImage {
    Prepared(Rc<ProgramImage>),
    Live(RomImage),
}

impl ScriptImage {
    /// `prepared` while it serves `image`, otherwise `image` itself.
    pub fn for_image(prepared: Option<&Rc<ProgramImage>>, image: RomImage) -> Self {
        match prepared {
            Some(prepared) if prepared.serves(&image) => Self::Prepared(prepared.clone()),
            _ => Self::Live(image),
        }
    }
    pub fn read(&self, offset: usize) -> u8 {
        match self {
            Self::Prepared(prepared) => prepared.read(offset),
            Self::Live(image) => image.read(offset),
        }
    }
    /// The cartridge image the script belongs to.
    pub fn source(&self) -> &RomImage {
        match self {
            Self::Prepared(prepared) => prepared.source(),
            Self::Live(image) => image,
        }
    }
}

impl From<RomImage> for ScriptImage {
    fn from(image: RomImage) -> Self {
        Self::Live(image)
    }
}

#[derive(Clone)]
pub enum EnemyScript {
    // Sequential movement retains this allocation across callbacks.
    At { image: ScriptImage, position: usize },
    // Numeric targets resolve against the interpreter's captured bank and image.
    Jump(u16),
    Stop,
}

pub fn bank_base(bank: u8) -> isize {
    sm_data::cartridge::lorom_offset((u32::from(bank) << 16) | 0x8000) - 0x8000
}

impl EnemyScript {
    pub fn at(image: impl Into<ScriptImage>, position: usize) -> Self {
        Self::At {
            image: image.into(),
            position,
        }
    }
    pub fn jump(address: u16) -> Self {
        // A zero jump stops; a linear position at bank address zero does not.
        if address == 0 {
            Self::Stop
        } else {
            Self::Jump(address)
        }
    }
    fn cursor(&self) -> (&ScriptImage, usize) {
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
    pub fn bank_address(&self, base: usize) -> u16 {
        self.cursor().1.wrapping_sub(base) as u16
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
    pub fn resolve(self, image: &ScriptImage, base: isize) -> Option<Self> {
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
