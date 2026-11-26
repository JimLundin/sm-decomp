//! Standard graphics for the Landing Site receiving loader.

use crate::dma::BusAddress;

/// Transfer assets and their source identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graphics {
    /// Standard BG3 graphics, in the order of VRAM words.
    pub background: Vec<u16>,
    /// Increasing ROM portion of the standard sprite transfer.
    pub sprites: Vec<u8>,
    /// Source of the first sprite byte.
    pub sprite_source: BusAddress,
    /// Length of the full sprite transfer, including the later mapped-RAM portion.
    pub sprite_length: u16,
}
