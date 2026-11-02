//! Finite CPU arithmetic and the shared PPU affine write/read rule.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arithmetic {
    pub multiplicand: u8,
    pub product: u16,
    pub dividend: u16,
    pub quotient: u16,
}
impl Arithmetic {
    /// Actual SNES hard-reset values. Supplied starts never call this implicitly.
    pub const fn reset() -> Self {
        Self {
            multiplicand: 0xff,
            product: 0xfe01,
            dividend: 0xffff,
            quotient: 0x101,
        }
    }
    pub fn multiply(&mut self, multiplier: u8) {
        self.product = multiply(self.multiplicand, multiplier);
    }
    pub fn divide(&mut self, divisor: u8) {
        if divisor == 0 {
            self.quotient = u16::MAX;
            self.product = self.dividend;
        } else {
            self.quotient = self.dividend / u16::from(divisor);
            self.product = self.dividend % u16::from(divisor);
        }
    }
}

pub fn multiply(multiplicand: u8, multiplier: u8) -> u16 {
    u16::from(multiplicand) * u16::from(multiplier)
}

/// Every affine and BG1 scroll write uses the same physical previous byte.
pub fn affine_word(previous: &mut u8, value: u8) -> i16 {
    let word = i16::from_le_bytes([*previous, value]);
    *previous = value;
    word
}

/// MPYL/MPYM/MPYH are a signed sixteen by signed eight-bit readback.
pub fn affine_product(matrix: &[i16; 8]) -> [u8; 3] {
    let product = i32::from(matrix[0]) * i32::from(matrix[1] >> 8);
    let [low, middle, high, _] = product.to_le_bytes();
    [low, middle, high]
}

pub fn tile_address(word: u16, remap: u8) -> u16 {
    match remap {
        0 => word,
        1 => (word & 0xff00) | ((word & 0x00e0) >> 5) | ((word & 0x001f) << 3),
        2 => (word & 0xfe00) | ((word & 0x01c0) >> 6) | ((word & 0x003f) << 3),
        3 => (word & 0xfc00) | ((word & 0x0380) >> 7) | ((word & 0x007f) << 3),
        _ => word,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Affine {
    pub matrix: [i16; 8],
    pub horizontal_flip: bool,
    pub vertical_flip: bool,
    pub large_field: bool,
    pub character_fill: bool,
}
impl Affine {
    pub fn write(&mut self, previous: &mut u8, index: usize, value: u8) {
        crate::table::check_index(index, 8);
        let word = affine_word(previous, value);
        self.matrix[index] = if index < 4 { word } else { word & 0x1fff };
    }
    pub fn select(&mut self, value: u8) {
        self.horizontal_flip = value & 1 != 0;
        self.vertical_flip = value & 2 != 0;
        self.character_fill = value & 0x40 != 0;
        self.large_field = value & 0x80 != 0;
    }
}
