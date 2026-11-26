//! The escape timer's word and digits in drawing order.
use crate::{rom::RomAddress, spritemap::Spritemap};

/// The timer uses a fixed word followed by six decimal digits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Figures {
    /// The frame and letters around the digits.
    pub frame: Spritemap,
    /// Figures for zero through nine.
    pub digits: [Spritemap; 10],
}

/// The first entry of a ROM-based indirect HDMA table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndirectHdmaStart {
    /// Address where the hardware reloads the table at the start of the frame.
    pub source: RomAddress,
    /// Encoded line count and repeat flag.
    pub lines: u8,
    /// Address of the scanline data in the configured indirect bank.
    pub indirect_address: u16,
}
