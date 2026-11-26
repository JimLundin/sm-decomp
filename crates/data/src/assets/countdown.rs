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

/// A programmed indirect channel and its registers after a visible frame runs its table to
/// completion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndirectHdmaProgram {
    /// Contents of DMAP and BBAD in the channel's spawn descriptor.
    pub control: u8,
    /// The offset of the target register from the B-bus base.
    pub destination: u8,
    /// The bank the indirect addresses refer to.
    pub data_bank: u8,
    /// The first header, loaded at the start of the next visible frame.
    pub opening: IndirectHdmaStart,
    /// DAS after the terminal header read.
    pub terminal_address: u16,
    /// The hardware cursor after the terminal fetch.
    pub terminal_cursor: u16,
}

/// The beam reopens the same three channels when the shaft reaches the interrupted vblank.
#[must_use]
pub const fn shaft_opening(
    [haze, mode, scroll]: [IndirectHdmaProgram; 3],
) -> [Option<IndirectHdmaStart>; 8] {
    [
        None,
        None,
        Some(haze.opening),
        Some(mode.opening),
        None,
        None,
        None,
        Some(scroll.opening),
    ]
}
