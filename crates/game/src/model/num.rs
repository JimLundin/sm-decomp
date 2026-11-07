//! The numbers motion counts in: a pixel word and a subpixel word, and the rules for
//! comparing, adding and holding them.

use canonical::Canonical;

use crate::session::SpeedCap;
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
    pub(crate) const fn plus(self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }
    pub(crate) const fn minus(self, other: Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }
    /// The value with its subpixel word replaced.
    pub(crate) const fn with_sub(self, sub: u16) -> Self {
        Self::new(self.pixel(), sub)
    }
    /// Its pixel word's sign bit is set: every "negative" of the spec.
    pub(crate) const fn negative(self) -> bool {
        self.pixel().cast_signed().is_negative()
    }
    /// `a` is below `b` when their difference mod 2^16 is negative; every "below" of the spec.
    pub(crate) const fn below(a: u16, b: u16) -> bool {
        a.wrapping_sub(b).cast_signed().is_negative()
    }
    // SpeedCap: the base speed held to `max` after its add.
    pub(crate) const fn capped(self, max: Self, rule: SpeedCap) -> Self {
        let (pixel, sub, max_pixel, max_sub) = (self.pixel(), self.sub(), max.pixel(), max.sub());
        let keep = match rule {
            SpeedCap::Wordwise => {
                Self::below(pixel, max_pixel)
                    || (pixel == max_pixel && (sub == max_sub || Self::below(sub, max_sub)))
            }
            SpeedCap::Exact => self.0 <= max.0,
        };
        if keep { self } else { max }
    }
    // DisplacementClamp: at most `max` whole pixels either way; the subpixel is kept.
    pub(crate) const fn clamped(self, max: u16) -> Self {
        let pixel = self.pixel();
        let leftward = pixel.cast_signed().is_negative();
        let pixel = if !leftward && !Self::below(pixel, max.wrapping_add(1)) {
            max
        } else if leftward && Self::below(pixel, max.wrapping_neg()) {
            max.wrapping_neg()
        } else {
            pixel
        };
        Self::new(pixel, self.sub())
    }
}

impl From<Speed> for Fixed {
    fn from(speed: Speed) -> Self {
        Self(speed.0)
    }
}
