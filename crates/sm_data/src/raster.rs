//! Shared raster input limits and failure values.

pub const MAX_INDEXED_WORDS: usize = 32768 + 31;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RasterFault {
    Word(usize),
    SpritePixel(i32),
    SpriteSize(u8),
    BrightnessIndex(usize),
    MosaicIndex(i32),
    MosaicSize,
    Line(u16),
    Mode(u8),
    MediaLimit,
    Layer(i32),
    EnabledMosaic(usize),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Background {
    pub horizontal_scroll: u16,
    pub vertical_scroll: u16,
    pub map_words: u16,
    pub tile_words: u16,
    pub wider: bool,
    pub higher: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub first_enabled: bool,
    pub first_inverted: bool,
    pub second_enabled: bool,
    pub second_inverted: bool,
    pub logic: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Windows {
    pub first: [u8; 2],
    pub second: [u8; 2],
    pub native_flags: u32,
    pub layers: [Window; 6],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Routing {
    pub enabled: [u8; 2],
    pub windowed: [u8; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Affine {
    /// a,b,c,d, center-x,center-y, horizontal-scroll,vertical-scroll.
    pub matrix: [i16; 8],
    pub horizontal_flip: bool,
    pub vertical_flip: bool,
    pub large_field: bool,
    pub character_fill: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorMath {
    pub clip: u8,
    pub prevent: u8,
    pub enabled: [bool; 6],
    pub add_subscreen: bool,
    pub subtract: bool,
    pub half: bool,
    pub fixed: [u8; 3],
    pub brightness: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Overflow {
    pub range: bool,
    pub time: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexedImage {
    words: Vec<u16>,
    palette: [u16; 256],
}

impl IndexedImage {
    /// The supplied tail is explicit content, including any required aliases.
    /// Missing reads fail at the query; validation never fabricates zero words.
    pub fn new(words: Vec<u16>, palette: [u16; 256]) -> Result<Self, RasterFault> {
        if words.len() > MAX_INDEXED_WORDS {
            return Err(RasterFault::MediaLimit);
        }
        Ok(Self { words, palette })
    }
}

impl IndexedImage {
    pub fn word(&self, index: usize) -> Result<u16, RasterFault> {
        self.words
            .get(index)
            .copied()
            .ok_or(RasterFault::Word(index))
    }
    pub fn palette(&self, index: u8) -> Result<u16, RasterFault> {
        Ok(self.palette[usize::from(index)])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedSprites {
    pub words: [u16; 256],
    pub extension: [u8; 32],
    pub tile_words: [u16; 2],
    pub size: u8,
    pub rotation: Option<u8>,
    pub interlace: bool,
    pub even_frame: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedMosaic {
    pub native_enabled: u8,
    pub background0_enabled: bool,
    pub size: u8,
    pub start_line: u8,
    pub modulo: [u8; 256],
}
