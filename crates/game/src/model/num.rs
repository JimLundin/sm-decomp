//! The numbers motion counts in: a pixel word and a subpixel word, and the rules for
//! comparing, adding and holding them.

use canonical::Canonical;

use data::movement::Speed;

/// A coordinate, speed or displacement: a pixel word and a subpixel word, wrapping mod 2^32.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fixed(pub u32);

impl Fixed {
    pub(crate) const ZERO: Self = Self(0);
    /// The value of `pixel` and `sub`.
    #[must_use]
    pub const fn new(pixel: u16, sub: u16) -> Self {
        Self(Speed::new(pixel, sub).0)
    }
    /// The pixel word.
    #[must_use]
    pub const fn pixel(self) -> u16 {
        Speed(self.0).pixels()
    }
    /// The subpixel word.
    #[must_use]
    pub const fn sub(self) -> u16 {
        Speed(self.0).subpixels()
    }
    /// Its pixel word's sign bit is set: every "negative" of the spec.
    pub(crate) const fn negative(self) -> bool {
        self.pixel().cast_signed().is_negative()
    }
}

impl From<Speed> for Fixed {
    fn from(speed: Speed) -> Self {
        Self(speed.0)
    }
}
