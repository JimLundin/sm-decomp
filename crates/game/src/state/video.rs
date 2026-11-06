//! Video memory and the PPU registers, in SNES layout (D5).
//!
//! The game reads this memory back and shows its stale contents, so the State holds the
//! image itself; meaning for widescreen and HD art travels in the intent output instead.

use canonical::Canonical;
pub use data::dma::BusAddress;
pub use data::pose::tiles::{GraphicsRef, TileDefinitionId};

use super::slots::{Flag, FlagSet};

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

/// One of a background's eight palettes.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BgPalette(pub u8);

/// A background tile's number, ten bits.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TileNumber(pub u16);

/// One background tilemap cell: the tile and how it's drawn. The PPU's word is
/// [`TilemapEntry::word`], and [`TilemapEntry::from_word`] gives every word back exactly.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TilemapEntry {
    /// The tile.
    pub tile: TileNumber,
    /// Its palette.
    pub palette: BgPalette,
    /// Whether it draws above the background's low-priority tiles.
    pub priority: bool,
    /// Mirrored left to right.
    pub flip_x: bool,
    /// Mirrored top to bottom.
    pub flip_y: bool,
}

/// A tilemap word's tile number bits.
const TILE_BITS: u16 = 0x03FF;
/// A tilemap word's palette field: its shift and width.
const PALETTE_SHIFT: u16 = 10;
const PALETTE_BITS: u16 = 0x07;
const PRIORITY_BIT: u16 = 0x2000;
const FLIP_X_BIT: u16 = 0x4000;
const FLIP_Y_BIT: u16 = 0x8000;

/// `bit` when `set`, else nothing.
const fn bit_if(set: bool, bit: u16) -> u16 {
    if set { bit } else { 0 }
}

impl TilemapEntry {
    /// The entry the PPU reads from `word`.
    #[must_use]
    pub const fn from_word(word: u16) -> Self {
        let [palette, _] = ((word >> PALETTE_SHIFT) & PALETTE_BITS).to_le_bytes();
        Self {
            tile: TileNumber(word & TILE_BITS),
            palette: BgPalette(palette),
            priority: word & PRIORITY_BIT != 0,
            flip_x: word & FLIP_X_BIT != 0,
            flip_y: word & FLIP_Y_BIT != 0,
        }
    }

    /// The word the PPU reads.
    #[must_use]
    pub const fn word(self) -> u16 {
        let palette = u16::from_le_bytes([self.palette.0, 0]) & PALETTE_BITS;
        (self.tile.0 & TILE_BITS)
            | (palette << PALETTE_SHIFT)
            | bit_if(self.priority, PRIORITY_BIT)
            | bit_if(self.flip_x, FLIP_X_BIT)
            | bit_if(self.flip_y, FLIP_Y_BIT)
    }

    /// The same tile in `palette`.
    #[must_use]
    pub const fn in_palette(self, palette: BgPalette) -> Self {
        Self { palette, ..self }
    }
}

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

impl Flag for Layer {
    /// The layer's bit in TM, TS, TMW and TSW.
    fn bit(self) -> u32 {
        let place = match self {
            Self::Bg1 => 0,
            Self::Bg2 => 1,
            Self::Bg3 => 2,
            Self::Bg4 => 3,
            Self::Objects => 4,
        };
        1 << place
    }
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

/// The registers the HUD interrupt writes for the playfield below the HUD.
///
/// Bank `$80` log, end HUD drawing `$80:96A9`: the game's own copies, which the NMI's register
/// commit refreshes and the message box overrides while it is up.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Playfield {
    /// TM below the HUD; the commit copies the TM mirror here (`$80:9257`).
    pub main_screen: Layers,
    /// BG3SC below the HUD, which the HUD's own tilemap replaces above it.
    pub bg3_tilemap: TilemapBase,
    /// CGWSEL and CGADSUB below the HUD.
    pub colour_math: PlayfieldColourMath,
}

/// The playfield's colour math registers, CGWSEL and CGADSUB.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayfieldColourMath {
    /// CGWSEL.
    pub select: ColourMathSelect,
    /// CGADSUB.
    pub layers: ColourMathLayers,
}

/// A scroll register's ten bits; the PPU keeps no more of what the game writes.
pub const SCROLL_BITS: u16 = 0x03FF;

/// Which way a DMA moves its bytes (DMAP bit 7).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DmaDirection {
    /// A bus to B bus: memory to a PPU register.
    #[default]
    ToRegister,
    /// B bus to A bus: a PPU register to memory.
    FromRegister,
}

/// How a DMA steps its A-bus address (DMAP bits 3-4). Both fixed encodings are kept: the
/// hardware treats them alike and the registers hold whichever was written.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DmaAddressStep {
    /// Up by one a byte.
    #[default]
    Increment,
    /// Stays put (encoding 1).
    Fixed,
    /// Down by one a byte.
    Decrement,
    /// Stays put (encoding 3).
    FixedAgain,
}

/// Which B-bus registers a DMA writes and in what order (DMAP bits 0-2). Encodings 5 to 7
/// repeat 1 to 3 on the hardware and are kept as written.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DmaPattern {
    /// One register a byte.
    #[default]
    OneRegister,
    /// Two consecutive registers, a byte each.
    TwoRegisters,
    /// One register written twice.
    WriteTwice,
    /// Two consecutive registers, each written twice.
    TwoRegistersTwice,
    /// Four consecutive registers.
    FourRegisters,
    /// Encoding 5, which acts as [`Self::TwoRegisters`].
    TwoRegistersMirror,
    /// Encoding 6, which acts as [`Self::WriteTwice`].
    WriteTwiceMirror,
    /// Encoding 7, which acts as [`Self::TwoRegistersTwice`].
    TwoRegistersTwiceMirror,
}

/// A DMA channel's control register, DMAP ($43x0).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DmaControl {
    /// Bit 7.
    pub direction: DmaDirection,
    /// Bit 6: HDMA reads its data through an indirect address.
    pub indirect: bool,
    /// Bit 5, which the hardware ignores; the registers keep what was written.
    pub unused: bool,
    /// Bits 3-4.
    pub step: DmaAddressStep,
    /// Bits 0-2.
    pub pattern: DmaPattern,
}

impl DmaControl {
    /// Memory to one register, a byte at a time.
    pub const TO_ONE_REGISTER: Self = Self::to_register(DmaPattern::OneRegister);
    /// Memory to a register pair, a word at a time.
    pub const TO_TWO_REGISTERS: Self = Self::to_register(DmaPattern::TwoRegisters);
    /// A register pair to memory, a word at a time.
    pub const FROM_TWO_REGISTERS: Self = Self {
        direction: DmaDirection::FromRegister,
        ..Self::TO_TWO_REGISTERS
    };
    /// Indirect HDMA writing one register twice a line.
    pub const HDMA_INDIRECT_WRITE_TWICE: Self = Self {
        indirect: true,
        ..Self::to_register(DmaPattern::WriteTwice)
    };

    const fn to_register(pattern: DmaPattern) -> Self {
        Self {
            direction: DmaDirection::ToRegister,
            indirect: false,
            unused: false,
            step: DmaAddressStep::Increment,
            pattern,
        }
    }

    /// The control register's bits as the registers hold them: every byte is a control.
    #[must_use]
    pub const fn from_byte(byte: u8) -> Self {
        Self {
            direction: if byte & 0x80 == 0 {
                DmaDirection::ToRegister
            } else {
                DmaDirection::FromRegister
            },
            indirect: byte & 0x40 != 0,
            unused: byte & 0x20 != 0,
            step: match (byte >> 3) & 0b11 {
                0 => DmaAddressStep::Increment,
                1 => DmaAddressStep::Fixed,
                2 => DmaAddressStep::Decrement,
                _ => DmaAddressStep::FixedAgain,
            },
            pattern: match byte & 0b111 {
                0 => DmaPattern::OneRegister,
                1 => DmaPattern::TwoRegisters,
                2 => DmaPattern::WriteTwice,
                3 => DmaPattern::TwoRegistersTwice,
                4 => DmaPattern::FourRegisters,
                5 => DmaPattern::TwoRegistersMirror,
                6 => DmaPattern::WriteTwiceMirror,
                _ => DmaPattern::TwoRegistersTwiceMirror,
            },
        }
    }

    /// The byte the register holds.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        let direction = match self.direction {
            DmaDirection::ToRegister => 0,
            DmaDirection::FromRegister => 0x80,
        };
        let step = match self.step {
            DmaAddressStep::Increment => 0,
            DmaAddressStep::Fixed => 1,
            DmaAddressStep::Decrement => 2,
            DmaAddressStep::FixedAgain => 3,
        };
        let pattern = match self.pattern {
            DmaPattern::OneRegister => 0,
            DmaPattern::TwoRegisters => 1,
            DmaPattern::WriteTwice => 2,
            DmaPattern::TwoRegistersTwice => 3,
            DmaPattern::FourRegisters => 4,
            DmaPattern::TwoRegistersMirror => 5,
            DmaPattern::WriteTwiceMirror => 6,
            DmaPattern::TwoRegistersTwiceMirror => 7,
        };
        let indirect = if self.indirect { 0x40 } else { 0 };
        let unused = if self.unused { 0x20 } else { 0 };
        direction | indirect | unused | (step << 3) | pattern
    }
}

impl From<u8> for DmaControl {
    fn from(byte: u8) -> Self {
        Self::from_byte(byte)
    }
}

impl From<DmaControl> for u8 {
    fn from(control: DmaControl) -> Self {
        control.to_byte()
    }
}

/// A DMA channel's B-bus register, BBAD ($43x1): the PPU register the transfer addresses.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BBusRegister {
    /// OAMDATA, $2104.
    OamData,
    /// BG3VOFS, $2112, which HDMA writes twice a line.
    Bg3VerticalScroll,
    /// VMDATAL, $2118: VRAM word writes.
    VramData,
    /// CGDATA, $2122.
    ColourData,
    /// VMDATALREAD, $2139: VRAM word reads.
    VramDataRead,
    /// A register the engine doesn't use. A channel nothing programmed holds $FF (the
    /// register $21FF), and the registers keep whatever was written.
    Other(u8),
}

impl BBusRegister {
    /// The register BBAD names: every byte is a register.
    #[must_use]
    pub const fn from_byte(byte: u8) -> Self {
        match byte {
            0x04 => Self::OamData,
            0x12 => Self::Bg3VerticalScroll,
            0x18 => Self::VramData,
            0x22 => Self::ColourData,
            0x39 => Self::VramDataRead,
            other => Self::Other(other),
        }
    }

    /// The byte BBAD holds, the register's offset from $2100.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::OamData => 0x04,
            Self::Bg3VerticalScroll => 0x12,
            Self::VramData => 0x18,
            Self::ColourData => 0x22,
            Self::VramDataRead => 0x39,
            Self::Other(byte) => byte,
        }
    }
}

impl Default for BBusRegister {
    fn default() -> Self {
        Self::from_byte(0)
    }
}

impl From<u8> for BBusRegister {
    fn from(byte: u8) -> Self {
        Self::from_byte(byte)
    }
}

impl From<BBusRegister> for u8 {
    fn from(register: BBusRegister) -> Self {
        register.to_byte()
    }
}

/// One DMA channel's registers as the last transfer left them ($43x0 to $43xA).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DmaChannel {
    /// DMAP: the direction, the addressing and the transfer pattern.
    pub control: DmaControl,
    /// BBAD: the B-bus register, $21xx.
    pub destination: BBusRegister,
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
    Words {
        /// Where the original's DMA reads them, which the channel's registers keep.
        from: BusAddress,
        /// The words.
        words: Vec<u16>,
    },
    /// ROM graphics, which the content's graphics resolve.
    Content(RomWords),
}

/// Words of ROM graphics: where they start and how many.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RomWords {
    /// The first word.
    pub from: GraphicsRef,
    /// How many.
    pub words: u16,
}

/// Samus's tile uploads the next requested NMI makes: the tiles definition of each half her
/// drawing queued this frame (bank `$92` log, `$92:8000`; bank `$80` log, `$80:9376`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SamusTileUploads {
    /// The top half's.
    pub top: Option<TileDefinitionId>,
    /// The bottom half's.
    pub bottom: Option<TileDefinitionId>,
}

/// A level tilemap upload's part: where it lands and how many bytes it writes.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TilemapPart {
    /// Its first word in VRAM.
    pub to: VramWord,
    /// Its length in bytes.
    pub bytes: u16,
}

/// How many times the main strand built an upload since the NMI last made it: the NMI
/// makes it while the low byte is nonzero, then clears it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UploadRequests(pub u16);

impl UploadRequests {
    /// One more build.
    #[must_use]
    pub const fn requested(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    /// Whether the NMI has an upload to make: the flag's low byte is nonzero.
    #[must_use]
    pub const fn pending(self) -> bool {
        self.0 & 0x00FF != 0
    }

    /// The requests after the NMI made the upload: the low byte clears (`STZ` with an 8-bit
    /// accumulator), the high byte stays.
    #[must_use]
    pub const fn served(self) -> Self {
        Self(self.0 & 0xFF00)
    }
}

/// The tilemap words a column of 16 blocks draws: two 8x8 columns, top to bottom.
pub const COLUMN_WORDS: usize = 32;
/// The tilemap words a row of 17 blocks draws: two 8x8 rows, left to right.
pub const ROW_WORDS: usize = 34;

/// A block column's tilemap upload (bank `$80` log, `$80:A9DE`): its two 8x8 columns go
/// down the tilemap from the column's row, the part past the tilemap's bottom wrapping to its
/// top.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColumnUpload {
    /// The left 8x8 column.
    pub left: [TilemapEntry; COLUMN_WORDS],
    /// The right one.
    pub right: [TilemapEntry; COLUMN_WORDS],
    /// The part to the tilemap's bottom.
    pub first: TilemapPart,
    /// The part wrapped to its top.
    pub wrapped: TilemapPart,
    /// Builds not yet made.
    pub requests: UploadRequests,
}

/// A block row's tilemap upload (bank `$80` log, `$80:AB78`): its two 8x8 rows go along the
/// tilemap from the row's column, the part past the screen's right edge wrapping to the other
/// screen.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowUpload {
    /// The top 8x8 row.
    pub top: [TilemapEntry; ROW_WORDS],
    /// The bottom one.
    pub bottom: [TilemapEntry; ROW_WORDS],
    /// The part to the screen's right edge.
    pub first: TilemapPart,
    /// The part wrapped to the other screen.
    pub wrapped: TilemapPart,
    /// Builds not yet made.
    pub requests: UploadRequests,
}

impl Default for RowUpload {
    fn default() -> Self {
        Self {
            top: [TilemapEntry::default(); ROW_WORDS],
            bottom: [TilemapEntry::default(); ROW_WORDS],
            first: TilemapPart::default(),
            wrapped: TilemapPart::default(),
            requests: UploadRequests::default(),
        }
    }
}

/// One background's level uploads.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayerUploads {
    /// The block column scrolling across last built.
    pub column: ColumnUpload,
    /// The block row scrolling down or up last built.
    pub row: RowUpload,
}

/// The level tilemap uploads the BG graphics update builds as the camera scrolls (bank `$80`
/// log, `$80:A3DF`), which the NMI's scrolling DMAs make (`$80:8CD8`, `$80:8DAC`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LevelUploads {
    /// BG1's, from layer 1.
    pub bg1: LayerUploads,
    /// BG2's, from layer 2's level data.
    pub bg2: LayerUploads,
}

impl Default for VramSource {
    fn default() -> Self {
        Self::Words {
            from: BusAddress::default(),
            words: Vec::new(),
        }
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

impl VramStep {
    /// The words the address moves after each word written.
    #[must_use]
    pub const fn words(self) -> usize {
        match self {
            Self::Row => 1,
            Self::Column => 32,
        }
    }
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
    /// Samus's tile uploads.
    pub samus_tiles: SamusTileUploads,
    /// The level tilemap uploads scrolling builds.
    pub level: LevelUploads,
    /// The playfield's colour math the commit hands the HUD interrupt (`$80:9277`).
    pub playfield_colour_math: PlayfieldColourMath,
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
            samus_tiles: SamusTileUploads::default(),
            level: LevelUploads::default(),
            playfield_colour_math: PlayfieldColourMath::default(),
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
    /// What the HUD interrupt writes for the playfield.
    pub playfield: Playfield,
    /// The DMA channels' registers as the last transfer left them.
    // open: the animated tiles (`$80:9416`, channel 0), the scrolling DMAs (`$80:8CD8`,
    // `$80:8DAC`, channel 1) and the HDMA queue join with their transfers.
    pub dma: [DmaChannel; DMA_CHANNELS],
    /// The batch for the next requested NMI.
    pub batch: NmiBatch,
    /// Whether the main strand has requested that NMI.
    pub nmi_requested: bool,
    /// What the message box set aside to put back when it closes.
    pub message_box: MessageBoxSaved,
}

/// Words of BG3 tilemap the message box covers and backs up.
pub const MESSAGE_BOX_BACKUP_WORDS: usize = 0x380;

/// What the message box saves of the playfield: the BG3 tilemap rows it covers and BG3SC,
/// which it points at its own tilemap while it is up.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct MessageBoxSaved {
    /// The tilemap rows' words, as VRAM held them when the box opened.
    pub tilemap: Vec<u16>,
    /// The playfield's BG3SC when the box opened.
    pub bg3_tilemap: TilemapBase,
}

impl Default for MessageBoxSaved {
    fn default() -> Self {
        Self {
            tilemap: vec![0; MESSAGE_BOX_BACKUP_WORDS],
            bg3_tilemap: TilemapBase::default(),
        }
    }
}
