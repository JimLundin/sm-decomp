//! Display state the game sets directly: brightness, the fixed colour and the
//! Mode 7 transform. Compat writes each value to its register at the call
//! site's original point, so values reach the registers in the game's order.

/// Forced blank and master brightness (INIDISP).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Brightness {
    pub forced_blank: bool,
    /// 0 (black) to 15 (full).
    pub level: u8,
}

impl Brightness {
    pub const BLANK_FULL: Self = Self {
        forced_blank: true,
        level: 15,
    };
    pub const FULL: Self = Self {
        forced_blank: false,
        level: 15,
    };

    pub const fn byte(self) -> u8 {
        (self.forced_blank as u8) << 7 | self.level & 0xf
    }
}

/// Sets the selected fixed-colour channels to one intensity (COLDATA).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedColor {
    pub red: bool,
    pub green: bool,
    pub blue: bool,
    /// 0 to 31.
    pub intensity: u8,
}

impl FixedColor {
    const fn channels(red: bool, green: bool, blue: bool, intensity: u8) -> Self {
        Self {
            red,
            green,
            blue,
            intensity,
        }
    }
    /// All three channels at `intensity`.
    pub const fn all(intensity: u8) -> Self {
        Self::channels(true, true, true, intensity)
    }
    pub const fn red(intensity: u8) -> Self {
        Self::channels(true, false, false, intensity)
    }
    pub const fn green(intensity: u8) -> Self {
        Self::channels(false, true, false, intensity)
    }
    pub const fn blue(intensity: u8) -> Self {
        Self::channels(false, false, true, intensity)
    }

    pub const fn byte(self) -> u8 {
        (self.blue as u8) << 7
            | (self.green as u8) << 6
            | (self.red as u8) << 5
            | self.intensity & 0x1f
    }
}

/// The Mode 7 matrix (8.8 fixed point) and centre: M7A..M7D, M7X, M7Y.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mode7Transform {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub d: u16,
    pub center_x: u16,
    pub center_y: u16,
}

impl Mode7Transform {
    /// No rotation or scaling, centred at (`center_x`, `center_y`).
    pub const fn identity(center_x: u16, center_y: u16) -> Self {
        Self {
            a: 0x100,
            b: 0,
            c: 0,
            d: 0x100,
            center_x,
            center_y,
        }
    }

    /// The six values in register order: A, B, C, D, X, Y.
    pub const fn words(self) -> [u16; 6] {
        [self.a, self.b, self.c, self.d, self.center_x, self.center_y]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_match_the_original_constants() {
        assert_eq!(Brightness::BLANK_FULL.byte(), 0x8f);
        assert_eq!(Brightness::FULL.byte(), 0x0f);
        assert_eq!(FixedColor::all(0).byte(), 0xe0);
        assert_eq!(FixedColor::red(0).byte(), 0x20);
        assert_eq!(FixedColor::green(0).byte(), 0x40);
        assert_eq!(FixedColor::blue(0).byte(), 0x80);
        assert_eq!(
            Mode7Transform::identity(3, 4).words(),
            [256, 0, 0, 256, 3, 4]
        );
    }
}
