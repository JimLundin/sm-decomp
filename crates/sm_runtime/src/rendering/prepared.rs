//! A prepared indexed-media consumer of the same scanline implementation.
use super::*;

pub use sm_data::raster::{IndexedImage, PreparedMosaic, PreparedSprites};

impl IndexedSource for IndexedImage {
    fn word(&self, index: usize) -> Result<u16, RasterFault> {
        IndexedImage::word(self, index)
    }
    fn palette(&self, index: u8) -> Result<u16, RasterFault> {
        IndexedImage::palette(self, index)
    }
}

fn sprite_input(sprites: &PreparedSprites) -> SpriteInput<'_> {
    SpriteInput {
        words: &sprites.words,
        extension: &sprites.extension,
        tile_words: sprites.tile_words,
        size: sprites.size,
        rotation: sprites.rotation,
        interlace: sprites.interlace,
        even_frame: sprites.even_frame,
    }
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
                &sprite_input(self.sprites),
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
