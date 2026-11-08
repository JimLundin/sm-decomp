//! Samus's speeds: a speed or an acceleration, horizontal or vertical, as the game keeps it.

use canonical::Canonical;

/// A speed or acceleration, horizontal or vertical: pixels a frame with a 1/65536 fraction,
/// as the original keeps it in a pixel word and a subpixel word.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Speed(pub u32);

impl Speed {
    /// Still.
    pub const ZERO: Self = Self(0);

    /// The speed of `pixels` and `subpixels` a frame.
    #[must_use]
    pub const fn new(pixels: u16, subpixels: u16) -> Self {
        let [pixels_high, pixels_low] = pixels.to_be_bytes();
        let [subpixels_high, subpixels_low] = subpixels.to_be_bytes();
        Self(u32::from_be_bytes([
            pixels_high,
            pixels_low,
            subpixels_high,
            subpixels_low,
        ]))
    }

    /// Whole pixels a frame: the original's speed word.
    #[must_use]
    pub const fn pixels(self) -> u16 {
        let [high, low, _, _] = self.0.to_be_bytes();
        u16::from_be_bytes([high, low])
    }

    /// The fraction, in 1/65536ths of a pixel: the original's subspeed word.
    #[must_use]
    pub const fn subpixels(self) -> u16 {
        let [_, _, high, low] = self.0.to_be_bytes();
        u16::from_be_bytes([high, low])
    }
}

#[cfg(test)]
mod tests;
