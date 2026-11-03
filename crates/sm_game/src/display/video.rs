//! Typed video memory uploads. A producer brackets its register writes with
//! `producing` and `produced`, giving the uploads those writes make as typed
//! values, so a host whose video memory takes typed calls can apply them in
//! place of the register writes. A producer that no longer names registers
//! hands the host a `Command` instead, which the host applies at once.

use super::control::FixedColor;
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

/// One of window 1's edges, the B-bus target of a window HDMA channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowEdge {
    Left,
    Right,
}

/// An indirect HDMA channel that sets one window 1 edge each line: `table`
/// lists line counts and pointers into `indirect_bank`. Setting it up clears
/// the channel's transfer cursors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowHdma {
    /// The HDMA channel, 0 to 7.
    pub channel: u8,
    pub edge: WindowEdge,
    pub table: Source,
    pub indirect_bank: u8,
}

/// A display change a producer makes now, at its original point. The host
/// applies it before returning; the emulated backend's encoder performs the
/// original's register writes for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Upload to VRAM now as `Upload::Tiles`, setting the VRAM port before
    /// the transfer channel.
    TilesNow {
        address: u16,
        increment: u16,
        source: Source,
        bytes: u16,
    },
    /// The fixed colour's selected channels.
    FixedColor(FixedColor),
    WindowHdma(WindowHdma),
}

/// A host that takes a producer's uploads and display commands as typed calls.
pub trait Video {
    /// The producer's register writes begin.
    fn producing(&mut self);
    /// The producer's register writes are done; `uploads` are what they did.
    fn produced(&mut self, producer: &'static str, uploads: &[Upload]);
    /// Apply `producer`'s `command` now.
    fn apply(&mut self, producer: &'static str, command: Command);
}
