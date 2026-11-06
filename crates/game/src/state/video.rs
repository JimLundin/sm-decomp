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

/// The PPU register mirrors the NMI commits, and the registers the PPU holds.
///
/// One field per architectural register the game writes. A packed register keeps the PPU's
/// bit layout in its newtype, the byte the game writes, so it goes back out unchanged.
// open: the packed registers split into their parts as the line program (D11) reads them.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    /// Whether the screen is blanked.
    pub forced_blank: bool,
    /// Brightness.
    pub brightness: Brightness,
    /// The sprites' sizes and tile bases (OBSEL).
    pub objects: ObjectSelect,
    /// The background mode and tile sizes (BGMODE).
    pub mode: BgMode,
    /// The mosaic size and layers (MOSAIC).
    pub mosaic: Mosaic,
    /// Each background's tilemap base and size (BG1SC to BG4SC).
    pub tilemaps: [TilemapBase; BACKGROUNDS],
    /// The backgrounds' tile bases, two to a register (BG12NBA, BG34NBA).
    pub tile_bases: [TileBases; BACKGROUNDS / 2],
    /// Backgrounds 1 to 4.
    pub scroll: [Scroll; BACKGROUNDS],
    /// Mode 7's settings and matrix.
    pub mode7: Mode7,
    /// The windows.
    pub windows: Windows,
    /// The main screen's layers.
    pub main_screen: Layers,
    /// The sub screen's layers.
    pub sub_screen: Layers,
    /// The layers the windows mask on the main screen (TMW).
    pub main_window_mask: Layers,
    /// The layers the windows mask on the sub screen (TSW).
    pub sub_window_mask: Layers,
    /// Colour math.
    pub colour_math: ColourMath,
    /// The display settings (SETINI).
    pub settings: DisplaySettings,
}

/// The backgrounds the PPU has.
pub const BACKGROUNDS: usize = 4;

/// OBSEL: the sprite sizes, the tile base and the second table's gap.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjectSelect(pub u8);

/// BGMODE: the mode in the low three bits, BG3's priority, then each background's tile size.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BgMode(pub u8);

impl BgMode {
    /// Whether the mode is mode 7.
    #[must_use]
    pub const fn is_mode7(self) -> bool {
        self.0 & 0x07 == 7
    }
}

/// MOSAIC: the block size and the backgrounds it applies to.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mosaic(pub u8);

/// `BGnSC`: a tilemap's VRAM base and its size in screens.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TilemapBase(pub u8);

/// BG12NBA or BG34NBA: two backgrounds' tile bases, a nibble each.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TileBases(pub u8);

/// SETINI: interlace, overscan, pseudo-hires and external sync.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DisplaySettings(pub u8);

/// Mode 7's registers.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mode7 {
    /// M7SEL: the screen-over behaviour and the flips.
    pub select: Mode7Select,
    /// M7A, M7B, M7C, M7D: the matrix.
    pub matrix: [i16; 4],
    /// M7X, M7Y: the centre.
    pub centre: [i16; 2],
}

/// M7SEL.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mode7Select(pub u8);

/// A window register byte in the PPU's layout: W12SEL, W34SEL or WOBJSEL's enables and
/// inversions, or WBGLOG and WOBJLOG's combining logic.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowSettings(pub u8);

/// The two windows' settings and edges.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Windows {
    /// W12SEL, W34SEL, WOBJSEL.
    pub select: [WindowSettings; 3],
    /// WH0 to WH3: window 1's left and right edge, then window 2's.
    pub edges: [u8; 4],
    /// WBGLOG.
    pub background_logic: WindowSettings,
    /// WOBJLOG.
    pub object_logic: WindowSettings,
}

/// Colour math's registers.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColourMath {
    /// CGWSEL: where colour math applies and what it adds.
    pub select: ColourMathSelect,
    /// CGADSUB: add or subtract, halving, and the layers it applies to.
    pub layers: ColourMathLayers,
    /// COLDATA: the fixed colour.
    pub fixed: FixedColour,
}

/// CGWSEL.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColourMathSelect(pub u8);

/// CGADSUB.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColourMathLayers(pub u8);

/// The fixed colour's three 5-bit components.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FixedColour {
    /// Red.
    pub red: u8,
    /// Green.
    pub green: u8,
    /// Blue.
    pub blue: u8,
}

/// A COLDATA write: an intensity and the components it sets.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColourWrite(pub u8);

/// COLDATA's intensity bits.
const COLOUR_INTENSITY: u8 = 0x1F;
/// COLDATA's component selects: red, green, blue.
const COLOUR_SELECTS: [u8; 3] = [0x20, 0x40, 0x80];

impl ColourWrite {
    /// The value a component takes, `old` when the write doesn't select it.
    const fn component(self, select: u8, old: u8) -> u8 {
        if self.0 & select == 0 {
            old
        } else {
            self.0 & COLOUR_INTENSITY
        }
    }
}

impl FixedColour {
    /// The fixed colour after `write`: each component it selects takes its intensity.
    #[must_use]
    pub const fn with(self, write: ColourWrite) -> Self {
        let [red, green, blue] = COLOUR_SELECTS;
        Self {
            red: write.component(red, self.red),
            green: write.component(green, self.green),
            blue: write.component(blue, self.blue),
        }
    }
}

/// A scroll register's ten bits; the PPU keeps no more of what the game writes.
pub const SCROLL_BITS: u16 = 0x03FF;

/// One DMA channel's registers as the last transfer left them ($43x0 to $43xA).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DmaChannel {
    /// DMAP: the direction, the addressing and the transfer pattern.
    pub control: u8,
    /// BBAD: the B-bus register, $21xx.
    pub destination: u8,
    /// A1T: the A-bus address.
    pub source: u16,
    /// A1B: the A-bus bank.
    pub source_bank: u8,
    /// DAS: the byte count, or the indirect HDMA address.
    pub count: u16,
    /// DASB: the indirect HDMA bank.
    pub indirect_bank: u8,
    /// A2A: the HDMA table's current address.
    pub table: u16,
    /// NLTR: the HDMA line counter.
    pub line_counter: u8,
}

/// The CPU's eight DMA channels.
pub const DMA_CHANNELS: usize = 8;

/// Where a queued VRAM upload's words come from.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum VramSource {
    /// Words from a State buffer, as the NMI's DMA reads them.
    Words(Vec<u16>),
    /// ROM graphics that content doesn't decode yet, by length in words.
    // open: B3's graphics content gives this a `data` reference; until then the upload
    // writes nothing and lockstep shows it.
    Content(u16),
}

impl Default for VramSource {
    fn default() -> Self {
        Self::Words(Vec::new())
    }
}

/// How VRAM's address moves after each word an upload writes.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VramStep {
    /// To the next word: along a tilemap row.
    #[default]
    Row,
    /// 32 words on: down a tilemap column.
    Column,
}

/// A queued VRAM upload.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct VramTransfer {
    /// Where it lands.
    pub to: VramWord,
    /// How the address moves.
    pub step: VramStep,
    /// What it writes.
    pub source: VramSource,
}

/// What the next requested NMI commits.
// open: the HDMA channel table joins with HDMA objects (B6).
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct NmiBatch {
    /// The register mirrors. Their fixed colour is unused: the NMI writes COLDATA from
    /// `fixed_colour`.
    pub registers: Registers,
    /// The three COLDATA writes, in order.
    pub fixed_colour: [ColourWrite; 3],
    /// The OAM buffer.
    pub oam: Oam,
    /// The entries drawn into the OAM buffer so far this frame, from the first.
    pub sprites: SpriteCount,
    /// The palette buffer, which every requested NMI uploads to CGRAM whole.
    pub cgram: [Colour; CGRAM_COLOURS],
    /// The VRAM uploads, in queue order.
    pub vram: Vec<VramTransfer>,
}

impl Default for NmiBatch {
    fn default() -> Self {
        Self {
            registers: Registers::default(),
            fixed_colour: [ColourWrite::default(); 3],
            oam: Oam::default(),
            sprites: SpriteCount::default(),
            cgram: [Colour::default(); CGRAM_COLOURS],
            vram: Vec::new(),
        }
    }
}

/// Video: the memories, what's committed and what's queued.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Video {
    /// The PPU's memories.
    pub memory: VideoMemory,
    /// The registers the PPU holds now.
    pub committed: Registers,
    /// The DMA channels' registers as the last transfer left them.
    // open: the NMI's DMAs and HDMA queue don't set them yet; only the decoder does.
    pub dma: [DmaChannel; DMA_CHANNELS],
    /// The batch for the next requested NMI.
    pub batch: NmiBatch,
    /// Whether the main strand has requested that NMI.
    pub nmi_requested: bool,
}
