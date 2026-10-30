//! Rendering of an extracted prepared message display.
#![forbid(unsafe_code)]
use crate::rendering as r;
use r::{IndexedSource, prepared::IndexedImage};
use sm_data::message::{Display, Fault, Revision, scanline_scroll};
const LINES: usize = 224;
const MAP_BASE: usize = 0x5880;
const MAP_WORDS: usize = 896;

pub struct MessageFrame {
    pub pixels: Vec<r::Bgra>,
    pub overflow: r::Overflow,
    pub revision: Revision,
}
pub fn render(input: Display<'_>) -> Result<MessageFrame, Fault> {
    let Display {
        display,
        map,
        active,
        scroll,
    } = input;
    let media = ComposedMedia {
        base: &display.media,
        map,
        message_palette: active,
    };
    let mut pixels = Vec::with_capacity(r::WIDTH * LINES);
    let mut overflow = display.overflow;
    for (index, line) in display.lines.iter().enumerate() {
        if line.extended_background {
            return Err(Fault::ExtendedBackground);
        }
        if line.mode != 1 && line.mode != 7 {
            return Err(r::RasterFault::Mode(line.mode).into());
        }
        let mut sprites = r::SpriteLine::cleared(overflow);
        if !line.forced_blank {
            let source = &display.sprites;
            let input = r::SpriteInput {
                words: &source.words,
                extension: &source.extension,
                tile_words: source.tile_words,
                size: source.size,
                rotation: source.rotation,
                interlace: source.interlace,
                even_frame: source.even_frame,
            };
            let (has_tiles, flags) =
                r::evaluate_sprites(&media, &input, index as i32, &mut sprites.pixels, overflow)?;
            sprites.has_tiles = has_tiles;
            sprites.overflow = flags;
        }
        let mut backgrounds = line.backgrounds;
        if active {
            backgrounds[2].vertical_scroll = scanline_scroll(scroll, index);
        }
        let brightness = r::BrightnessTables::for_level(line.color.brightness);
        let mosaic = &display.mosaic;
        let config = r::LineConfig {
            mode: line.mode,
            forced_blank: line.forced_blank,
            backgrounds,
            windows: line.windows,
            routing: line.routing,
            affine: line.affine,
            color: line.color,
            brightness: brightness.borrowed(),
            has_sprites: sprites.has_tiles,
            mosaic: r::Mosaic {
                native_enabled: mosaic.native_enabled,
                background0_enabled: mosaic.background0_enabled,
                size: mosaic.size,
                start_line: mosaic.start_line,
                modulo: &mosaic.modulo,
                left: 0,
            },
        };
        let mut row = [[0; 4]; r::WIDTH];
        r::render_line(&media, &config, &sprites, index as i32 + 1, &mut row)?;
        pixels.extend_from_slice(&row);
        overflow = sprites.overflow;
    }
    Ok(MessageFrame {
        pixels,
        overflow,
        revision: display.revision,
    })
}

struct ComposedMedia<'a> {
    base: &'a IndexedImage,
    map: Option<&'a [u16; MAP_WORDS]>,
    message_palette: bool,
}
impl IndexedSource for ComposedMedia<'_> {
    fn word(&self, index: usize) -> Result<u16, r::RasterFault> {
        if let Some(map) = self.map
            && (MAP_BASE..MAP_BASE + MAP_WORDS).contains(&index)
        {
            return Ok(map[index - MAP_BASE]);
        }
        self.base.word(index)
    }
    fn palette(&self, index: u8) -> Result<u16, r::RasterFault> {
        if self.message_palette {
            match index {
                25 => return Ok(0x0bb1),
                26 => return Ok(0x001f),
                _ => {}
            }
        }
        self.base.palette(index)
    }
}
