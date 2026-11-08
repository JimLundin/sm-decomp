//! The portable video records (ARCHITECTURE D16): the video memory image and the display
//! frame. `game` produces them and `runtime`'s line compositor consumes them; neither side
//! holds the other's types.

/// Words of video RAM.
pub const VRAM_WORDS: usize = 0x8000;
/// Colours in colour RAM.
pub const CGRAM_COLOURS: usize = 256;
/// Bytes of object attribute memory: 512 of the low table, then 32 of the high table.
pub const OAM_BYTES: usize = 544;
/// The most lines a frame shows (an overscan frame).
pub const MAX_LINES: usize = 239;
/// The lines a frame without overscan shows.
pub const NORMAL_LINES: usize = 224;

/// The PPU's three memories as the picture reads them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoMemory {
    /// Video RAM, by word.
    pub vram: Box<[u16; VRAM_WORDS]>,
    /// BGR555 colours.
    pub cgram: Box<[u16; CGRAM_COLOURS]>,
    /// Object attribute memory: four bytes per sprite, then two bits per sprite.
    pub oam: Box<[u8; OAM_BYTES]>,
}

/// A zeroed array built on the heap.
fn zeroed<T: Copy + Default, const N: usize>() -> Box<[T; N]> {
    let boxed = vec![T::default(); N].into_boxed_slice();
    boxed
        .try_into()
        .unwrap_or_else(|_| Box::new([T::default(); N]))
}

impl Default for VideoMemory {
    fn default() -> Self {
        Self {
            vram: zeroed(),
            cgram: zeroed(),
            oam: zeroed(),
        }
    }
}

/// The PPU registers the picture depends on, as the values the game last wrote to them.
/// Write latches are not part of it: a register pair is written within one frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PpuRegisters {
    /// INIDISP: forced blank and master brightness.
    pub inidisp: u8,
    /// OBSEL: sprite sizes and tile bases.
    pub obsel: u8,
    /// BGMODE.
    pub bgmode: u8,
    /// MOSAIC.
    pub mosaic: u8,
    /// BG1SC-BG4SC.
    pub bg_sc: [u8; 4],
    /// BG12NBA.
    pub bg12nba: u8,
    /// BG34NBA.
    pub bg34nba: u8,
    /// `BGnHOFS` and `BGnVOFS` per layer, 10 bits each.
    pub bg_scroll: [[u16; 2]; 4],
    /// M7HOFS and M7VOFS, 13 bits each: BG1's scroll ports as Mode 7 reads them.
    pub m7_scroll: [u16; 2],
    /// M7SEL.
    pub m7sel: u8,
    /// M7A, M7B, M7C, M7D, M7X, M7Y.
    pub m7: [i16; 6],
    /// W12SEL, W34SEL, WOBJSEL.
    pub window_select: [u8; 3],
    /// WH0-WH3.
    pub window_positions: [u8; 4],
    /// WBGLOG.
    pub wbglog: u8,
    /// WOBJLOG.
    pub wobjlog: u8,
    /// TM, TS, TMW, TSW.
    pub screens: [u8; 4],
    /// CGWSEL.
    pub cgwsel: u8,
    /// CGADSUB.
    pub cgadsub: u8,
    /// COLDATA as three 5-bit components: red, green, blue.
    pub fixed_color: [u8; 3],
    /// SETINI.
    pub setini: u8,
}

/// A visible line of the frame, from 1 (the first shown) to 239.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Line(pub u16);

/// One register write inside a frame, placed at the first line that sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LineWrite {
    /// The first line composed with the write in effect.
    pub line: Line,
    /// The beam line the write happened on; MOSAIC restarts its blocks there.
    pub beam_line: u16,
    /// The register's B-bus address low byte ($21xx).
    pub port: u8,
    /// The byte written.
    pub value: u8,
}

/// Everything a frame's picture is composed from besides the video memory: the registers as
/// the frame begins and each change the line program makes while it is shown (D11).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DisplayFrame {
    /// The registers as line 1 begins.
    pub start: PpuRegisters,
    /// In the order written; each write's `line` is at or after the previous one's.
    pub writes: Vec<LineWrite>,
    /// Whether the frame shows 239 lines rather than 224.
    pub overscan: bool,
    /// The PPU's field parity, which interlaced sprites and modes 5 and 6 read.
    pub even_frame: bool,
}
