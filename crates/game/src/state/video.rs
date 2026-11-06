//! Video memory and the PPU registers, in SNES layout (D5).
//!
//! The game reads this memory back and shows its stale contents, so the State holds the
//! image itself; meaning for widescreen and HD art travels in the intent output instead.

use canonical::Canonical;

use super::slots::FlagSet;

/// VRAM's size in 16-bit words: 64 KiB.
pub const VRAM_WORDS: usize = 0x8000;
/// CGRAM's size in colours.
pub const CGRAM_COLOURS: usize = 256;
/// OAM's sprite count.
pub const OAM_SPRITES: usize = 128;
/// OAM's high table, two bits per sprite.
pub const OAM_HIGH_BYTES: usize = OAM_SPRITES / 4;
/// The backgrounds the game scrolls.
pub const SCROLLED_BACKGROUNDS: usize = 3;

/// A count of sprites.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpriteCount(pub u16);

/// A BGR555 colour as CGRAM holds it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Colour(pub u16);

/// A word address within VRAM.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VramWord(pub u16);

/// One OAM low-table entry, in the PPU's byte order.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OamEntry {
    /// X, low eight bits; the ninth is in the high table.
    pub x: u8,
    /// Y.
    pub y: u8,
    /// The tile's low eight bits.
    pub tile: u8,
    /// Name table, palette, priority and flips, as the PPU packs them.
    pub attributes: u8,
}

/// OAM: the low table and the high table of X ninth bits and size bits.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Oam {
    /// One entry per sprite.
    pub low: [OamEntry; OAM_SPRITES],
    /// Two bits per sprite.
    pub high: [u8; OAM_HIGH_BYTES],
}

impl Default for Oam {
    fn default() -> Self {
        Self {
            low: [OamEntry::default(); OAM_SPRITES],
            high: [0; OAM_HIGH_BYTES],
        }
    }
}

/// The PPU's memories.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct VideoMemory {
    /// VRAM, exactly `VRAM_WORDS` words; opening a Session checks it.
    pub vram: Vec<u16>,
    /// CGRAM.
    pub cgram: [Colour; CGRAM_COLOURS],
    /// OAM.
    pub oam: Oam,
}

impl Default for VideoMemory {
    fn default() -> Self {
        Self {
            vram: vec![0; VRAM_WORDS],
            cgram: [Colour::default(); CGRAM_COLOURS],
            oam: Oam::default(),
        }
    }
}

/// A background or the sprite layer.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    /// Background 1.
    Bg1,
    /// Background 2.
    Bg2,
    /// Background 3.
    Bg3,
    /// Background 4.
    Bg4,
    /// Sprites.
    Objects,
}

/// A set of layers.
pub type Layers = FlagSet<Layer>;

/// A background's scroll.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Scroll {
    /// Horizontal.
    pub x: u16,
    /// Vertical.
    pub y: u16,
}

/// The screen's brightness, from black up to full.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Brightness(pub u8);

/// The PPU register mirrors the NMI commits.
// open: the full set is the mirrors the line program reads (D11) and the `registers`
// projection compares; the oracle's register stream for slice 0's rows decides it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    /// Whether the screen is blanked.
    pub forced_blank: bool,
    /// Brightness.
    pub brightness: Brightness,
    /// The main screen's layers.
    pub main_screen: Layers,
    /// The sub screen's layers.
    pub sub_screen: Layers,
    /// Backgrounds 1 to 3.
    pub scroll: [Scroll; SCROLLED_BACKGROUNDS],
}

/// A queued VRAM upload.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct VramTransfer {
    /// Where it lands.
    pub to: VramWord,
    /// What it writes.
    pub words: Vec<u16>,
}

/// What the next requested NMI commits.
// open: CGRAM uploads and the HDMA channel table join with palette FX and HDMA objects.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct NmiBatch {
    /// The register mirrors.
    pub registers: Registers,
    /// The OAM buffer.
    pub oam: Oam,
    /// The entries drawn into the OAM buffer so far this frame, from the first.
    pub sprites: SpriteCount,
    /// The VRAM uploads, in queue order.
    pub vram: Vec<VramTransfer>,
}

/// Video: the memories, what's committed and what's queued.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Video {
    /// The PPU's memories.
    pub memory: VideoMemory,
    /// The registers the PPU holds now.
    pub committed: Registers,
    /// The batch for the next requested NMI.
    pub batch: NmiBatch,
    /// Whether the main strand has requested that NMI.
    pub nmi_requested: bool,
}
