//! Ridley's ordinary fight object strips.

use crate::dma::BusAddress;

/// Two tiles in one row of Ridley's ribs.
pub const STRIP_WORDS: usize = 32;

/// A strip and its DMA source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Strip {
    /// The source retained in the DMA registers after the upload.
    pub source: BusAddress,
    /// The complete planar tile words in upload order.
    pub words: [u16; STRIP_WORDS],
}

/// The two strips for one look, top then bottom.
pub type Strips = [Strip; 2];

/// Object graphics for the three ribs looks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Art {
    /// The closed look already present in the room's original sheet.
    pub closed: Strips,
    /// The partly open look shared by the opening and closing beats.
    pub partly_open: Strips,
    /// The wide look.
    pub wide: Strips,
}
