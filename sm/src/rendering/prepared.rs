//! A prepared indexed-media consumer of the same scanline implementation.
use super::*;

pub const HEIGHT: usize = 224;

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

impl IndexedSource for IndexedImage {
    fn word(&self, index: usize) -> Result<u16, RasterFault> {
        self.words
            .get(index)
            .copied()
            .ok_or(RasterFault::Word(index))
    }
    fn palette(&self, index: u8) -> Result<u16, RasterFault> {
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

impl PreparedSprites {
    fn borrowed(&self) -> SpriteInput<'_> {
        SpriteInput {
            words: &self.words,
            extension: &self.extension,
            tile_words: self.tile_words,
            size: self.size,
            rotation: self.rotation,
            interlace: self.interlace,
            even_frame: self.even_frame,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedMosaic {
    pub native_enabled: u8,
    pub background0_enabled: bool,
    pub size: u8,
    pub start_line: u8,
    pub modulo: [u8; 256],
}

/// One settled line's immutable input; an upstream Scene supplies each line's
/// routing, palette/scroll/effect values rather than advancing them in drawing.
pub struct PreparedInput<'a> {
    pub media: &'a IndexedImage,
    pub sprites: &'a PreparedSprites,
    pub mode: u8,
    pub forced_blank: bool,
    pub backgrounds: [Background; 3],
    pub windows: Windows,
    pub routing: Routing,
    pub affine: Affine,
    pub mosaic: &'a PreparedMosaic,
    pub color: ColorMath,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedLine {
    pub pixels: [Bgra; WIDTH],
    pub sprites: SpriteLine,
}

impl PreparedInput<'_> {
    /// Pure prepared-input execution, with no PPU image or overlay input.
    /// Overflow is supplied explicitly because the native flags are frame-sticky.
    pub fn render(&self, line: u16, overflow: Overflow) -> Result<RenderedLine, RasterFault> {
        if line == 0 || line > 256 {
            return Err(RasterFault::Line(line));
        }
        if self.mode != 1 && self.mode != 7 {
            return Err(RasterFault::Mode(self.mode));
        }
        let mut sprites = SpriteLine::cleared(overflow);
        if !self.forced_blank {
            let (has_tiles, overflow) = evaluate_sprites(
                self.media,
                &self.sprites.borrowed(),
                i32::from(line) - 1,
                &mut sprites.pixels,
                sprites.overflow,
            )?;
            sprites.overflow = overflow;
            sprites.has_tiles = has_tiles;
        }
        let brightness = BrightnessTables::for_level(self.color.brightness);
        let config = LineConfig {
            mode: self.mode,
            forced_blank: self.forced_blank,
            backgrounds: self.backgrounds,
            windows: self.windows,
            routing: self.routing,
            affine: self.affine,
            mosaic: Mosaic {
                native_enabled: self.mosaic.native_enabled,
                background0_enabled: self.mosaic.background0_enabled,
                size: self.mosaic.size,
                start_line: self.mosaic.start_line,
                modulo: &self.mosaic.modulo,
                left: 0,
            },
            color: self.color,
            brightness: brightness.borrowed(),
            has_sprites: sprites.has_tiles,
        };
        let mut pixels = [[0; 4]; WIDTH];
        render_line(self.media, &config, &sprites, i32::from(line), &mut pixels)?;
        Ok(RenderedLine { pixels, sprites })
    }
}

/// Consume exactly224 independently settled line records; no simulation or
/// display-refresh operation occurs here. The output has fixed256x224 dimensions.
pub fn render_frame(
    lines: &[PreparedInput<'_>; HEIGHT],
    initial_overflow: Overflow,
) -> Result<PreparedFrame, RasterFault> {
    let mut pixels = Vec::with_capacity(WIDTH * HEIGHT);
    let mut overflow = initial_overflow;
    for (index, input) in lines.iter().enumerate() {
        let line = input.render((index + 1) as u16, overflow)?;
        overflow = line.sprites.overflow;
        pixels.extend_from_slice(&line.pixels);
    }
    Ok(PreparedFrame { pixels, overflow })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedFrame {
    pixels: Vec<Bgra>,
    pub overflow: Overflow,
}

impl PreparedFrame {
    pub fn pixels(&self) -> &[Bgra] {
        &self.pixels
    }
}
