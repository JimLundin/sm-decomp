//! Typed video memory uploads. A producer brackets its register writes with
//! `producing` and `produced`, giving the uploads those writes make as typed
//! values, so a host whose video memory takes typed calls can apply them in
//! place of the register writes.

use super::transfer::Source;

/// One upload. Objects and palettes come from work RAM, `source` being the
/// offset there; tiles come from anywhere on the CPU bus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Upload {
    /// OAM from the OAMADD word `address`: the low table, then the high table.
    Objects {
        address: u16,
        source: usize,
        bytes: u16,
    },
    /// CGRAM from palette entry `index`.
    Palette {
        index: u8,
        source: usize,
        bytes: u16,
    },
    /// VRAM from word `address`, its bytes alternating low and high and the
    /// address stepping by `increment` words after each high byte. A `bytes`
    /// of zero transfers 64 KiB, as the DMA does.
    Tiles {
        address: u16,
        increment: u16,
        source: Source,
        bytes: u16,
    },
}

/// A host that takes a producer's uploads as typed calls.
pub trait Video {
    /// The producer's register writes begin.
    fn producing(&mut self);
    /// The producer's register writes are done; `uploads` are what they did.
    fn produced(&mut self, producer: &'static str, uploads: &[Upload]);
}
