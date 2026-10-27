//! Bounded immutable inputs and caller-owned results for the shared raster core.
pub const WIDTH: usize = 256;
pub const BACKDROP: u16 = 0x0500;
pub const MAX_INDEXED_WORDS: usize = 32768 + 31;
pub type Bgra = [u8; 4];

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

/// Indexes are word positions, not byte offsets. A caller supplies native aliases
/// explicitly: wrapping the base does not wrap a later column or plane read.
pub trait IndexedSource {
    fn word(&self, index: usize) -> Result<u16, RasterFault>;
    fn palette(&self, index: u8) -> Result<u16, RasterFault>;
}

/// The native caller may have signed-x aliases outside the ordinary sprite line.
pub trait SpritePixels {
    fn pixel(&self, x: i32) -> Result<u16, RasterFault>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
pub enum Quantization {
    Native,
    Reference,
}

impl Windows {
    pub fn contains(&self, layer: usize, x: i32, policy: Quantization) -> bool {
        let window = self.layers[layer];
        let flags = match policy {
            Quantization::Native => self.native_flags >> (layer * 4),
            Quantization::Reference => {
                u32::from(window.first_inverted)
                    | u32::from(window.first_enabled) << 1
                    | u32::from(window.second_inverted) << 2
                    | u32::from(window.second_enabled) << 3
            }
        };
        let first =
            (x >= i32::from(self.first[0]) && x <= i32::from(self.first[1])) ^ (flags & 1 != 0);
        let second =
            (x >= i32::from(self.second[0]) && x <= i32::from(self.second[1])) ^ (flags & 4 != 0);
        match flags & 10 {
            2 => first,
            8 => second,
            10 => match policy {
                Quantization::Native => first || second,
                Quantization::Reference => match window.logic {
                    0 => first || second,
                    1 => first && second,
                    2 => first != second,
                    3 => first == second,
                    _ => false,
                },
            },
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Routing {
    pub enabled: [u8; 2],
    pub windowed: [u8; 2],
}

impl Routing {
    pub(super) fn enabled(&self, windows: &Windows, layer: usize, x: i32, sub: bool) -> bool {
        self.enabled[usize::from(sub)] & (1 << layer) != 0
            && (layer != 2 || (0..256).contains(&x))
            && (self.windowed[usize::from(sub)] & (1 << layer) == 0
                || !windows.contains(layer, x, Quantization::Native))
    }
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

#[derive(Clone, Copy)]
pub struct Mosaic<'a> {
    pub native_enabled: u8,
    pub background0_enabled: bool,
    pub size: u8,
    pub start_line: u8,
    pub modulo: &'a [u8; 256],
    pub left: i32,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrightnessTables {
    pub normal: [u8; 63],
    pub half: [u8; 64],
}

#[derive(Clone, Copy)]
pub struct Brightness<'a> {
    pub normal: &'a [u8; 63],
    pub half: &'a [u8; 64],
}

impl BrightnessTables {
    pub fn for_level(brightness: u8) -> Self {
        let mut tables = Self {
            normal: [0; 63],
            half: [0; 64],
        };
        update_brightness(brightness, &mut tables.normal, &mut tables.half);
        tables
    }

    pub fn borrowed(&self) -> Brightness<'_> {
        Brightness {
            normal: &self.normal,
            half: &self.half,
        }
    }
}

pub fn update_brightness(brightness: u8, normal: &mut [u8; 63], half: &mut [u8; 64]) {
    for (index, multiplier) in normal[..32].iter_mut().enumerate() {
        let color = index as u16;
        let value = (((color << 3 | color >> 2) * u16::from(brightness)) / 15) as u8;
        *multiplier = value;
        half[index * 2] = value;
        half[index * 2 + 1] = value;
    }
    let maximum = normal[31];
    normal[32..].fill(maximum);
}

#[derive(Clone, Copy)]
pub struct LineConfig<'a> {
    pub mode: u8,
    pub forced_blank: bool,
    pub backgrounds: [Background; 3],
    pub windows: Windows,
    pub routing: Routing,
    pub affine: Affine,
    pub mosaic: Mosaic<'a>,
    pub color: ColorMath,
    pub brightness: Brightness<'a>,
    pub has_sprites: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerPixel {
    pub layer: i32,
    pub color: [i32; 3],
    pub non_backdrop: bool,
}

#[derive(Clone, Copy)]
pub struct SpriteInput<'a> {
    pub words: &'a [u16; 256],
    pub extension: &'a [u8; 32],
    pub tile_words: [u16; 2],
    pub size: u8,
    pub rotation: Option<u8>,
    pub interlace: bool,
    pub even_frame: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Overflow {
    pub range: bool,
    pub time: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteLine {
    pub pixels: [u16; WIDTH],
    pub overflow: Overflow,
    pub has_tiles: bool,
}

impl SpriteLine {
    pub fn cleared(overflow: Overflow) -> Self {
        Self {
            pixels: [BACKDROP; WIDTH],
            overflow,
            has_tiles: false,
        }
    }
}

impl SpritePixels for SpriteLine {
    fn pixel(&self, x: i32) -> Result<u16, RasterFault> {
        usize::try_from(x)
            .ok()
            .and_then(|index| self.pixels.get(index))
            .copied()
            .ok_or(RasterFault::SpritePixel(x))
    }
}
