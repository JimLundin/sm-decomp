//! The per-pixel layer composition for the lines the native line path does not
//! draw: background mosaic and Mode 7 EXTBG. Each pixel walks its mode's layer
//! order front to back and takes the first opaque layer, with LakeSnes's layer
//! orders, tile decoding, Mode 7 sampling and direct colour (MIT, revision
//! 28b8fbc15ad15afff003165ff4f2a8adb1c93d18), the oracle's renderer for them.
use super::registers::Registers;
use super::{
    Affine, Bgra, Brightness, IndexedSource, LayerPixel, Mosaic, Quantization, RasterFault,
    SpritePixels, WIDTH, affine_start, needs_subscreen, shade_pixel, window_contains,
};

/// One entry of a layer order: a plane and the priority it is drawn at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Plane {
    Background(usize),
    Sprites,
}

/// A mode's front-to-back layer order and its backgrounds' bit depths.
struct Layout {
    order: &'static [(Plane, u8)],
    depths: [Option<u8>; 4],
}

const fn bg(layer: usize, priority: u8) -> (Plane, u8) {
    (Plane::Background(layer), priority)
}

const fn obj(priority: u8) -> (Plane, u8) {
    (Plane::Sprites, priority)
}

const MODE_0: Layout = Layout {
    order: &[
        obj(3),
        bg(0, 1),
        bg(1, 1),
        obj(2),
        bg(0, 0),
        bg(1, 0),
        obj(1),
        bg(2, 1),
        bg(3, 1),
        obj(0),
        bg(2, 0),
        bg(3, 0),
    ],
    depths: [Some(2), Some(2), Some(2), Some(2)],
};

const MODE_1: Layout = Layout {
    order: &[
        obj(3),
        bg(0, 1),
        bg(1, 1),
        obj(2),
        bg(0, 0),
        bg(1, 0),
        obj(1),
        bg(2, 1),
        obj(0),
        bg(2, 0),
    ],
    depths: [Some(4), Some(4), Some(2), None],
};

/// Mode 1 with BGMODE's BG3 priority bit: BG3's high tiles are in front.
const MODE_1_BG3_FRONT: Layout = Layout {
    order: &[
        bg(2, 1),
        obj(3),
        bg(0, 1),
        bg(1, 1),
        obj(2),
        bg(0, 0),
        bg(1, 0),
        obj(1),
        obj(0),
        bg(2, 0),
    ],
    depths: [Some(4), Some(4), Some(2), None],
};

const TWO_BACKGROUNDS: &[(Plane, u8)] = &[
    obj(3),
    bg(0, 1),
    obj(2),
    bg(1, 1),
    obj(1),
    bg(0, 0),
    obj(0),
    bg(1, 0),
];

const MODE_2: Layout = Layout {
    order: TWO_BACKGROUNDS,
    depths: [Some(4), Some(4), None, None],
};

const MODE_3: Layout = Layout {
    order: TWO_BACKGROUNDS,
    depths: [Some(8), Some(4), None, None],
};

const MODE_4: Layout = Layout {
    order: TWO_BACKGROUNDS,
    depths: [Some(8), Some(2), None, None],
};

const MODE_5: Layout = Layout {
    order: TWO_BACKGROUNDS,
    depths: [Some(4), Some(2), None, None],
};

const MODE_6: Layout = Layout {
    order: &[obj(3), bg(0, 1), obj(2), obj(1), bg(0, 0), obj(0)],
    depths: [Some(4), None, None, None],
};

const MODE_7: Layout = Layout {
    order: &[obj(3), obj(2), obj(1), bg(0, 0), obj(0)],
    depths: [Some(8), None, None, None],
};

/// Mode 7 with EXTBG: BG2 is the Mode 7 map's low seven bits, bit 7 its priority.
const MODE_7_EXTBG: Layout = Layout {
    order: &[obj(3), obj(2), bg(1, 1), obj(1), bg(0, 0), obj(0), bg(1, 0)],
    depths: [Some(8), Some(7), None, None],
};

impl Registers {
    fn layout(&self) -> &'static Layout {
        match self.mode {
            0 => &MODE_0,
            1 if self.bg3_priority => &MODE_1_BG3_FRONT,
            1 => &MODE_1,
            2 => &MODE_2,
            3 => &MODE_3,
            4 => &MODE_4,
            5 => &MODE_5,
            6 => &MODE_6,
            _ if self.extended_background => &MODE_7_EXTBG,
            _ => &MODE_7,
        }
    }

    /// Whether a background mosaic applies on this line.
    pub fn mosaic_applies(&self) -> bool {
        self.mosaic_size > 1 && self.layers.iter().any(|layer| layer.mosaic)
    }

    /// Whether a layer's two windows combine with AND, XOR or XNOR, which the
    /// native line path does not model (it combines them with OR).
    fn window_logic_applies(&self) -> bool {
        self.windows
            .layers
            .iter()
            .any(|window| window.first_enabled && window.second_enabled && window.logic != 0)
    }

    /// The native line path draws Mode 1 with BG3's high tiles in front and 8x8
    /// tiles, and Mode 7 with palette colour, both at 256 pixels and with OR
    /// windows. Every other line is composed per pixel: mosaic, EXTBG, the other
    /// modes, BG3 at normal priority, 16x16 tiles, direct colour, pseudo-hires
    /// and the other window logics.
    pub fn composes_per_pixel(&self) -> bool {
        let native = match self.mode {
            1 => self.bg3_priority && self.layers[..3].iter().all(|layer| !layer.big_tiles),
            7 => !self.extended_background && !self.direct_color,
            _ => false,
        };
        !native || self.pseudo_hires || self.mosaic_applies() || self.window_logic_applies()
    }
}

/// One line's composition inputs, fixed for the whole line.
pub struct Composer<'a, S> {
    source: &'a S,
    registers: &'a Registers,
    even_frame: bool,
    affine_start: (i32, i32),
}

impl<'a, S: IndexedSource> Composer<'a, S> {
    /// The Mode 7 start is taken at the line's start, after sprite evaluation.
    pub fn new(
        source: &'a S,
        registers: &'a Registers,
        even_frame: bool,
        line: i32,
    ) -> Result<Self, RasterFault> {
        let affine_start = if registers.mode == 7 {
            let mosaic = Mosaic {
                native_enabled: 0,
                background0_enabled: registers.layers[0].mosaic,
                size: registers.mosaic_size,
                start_line: registers.mosaic_start_line,
                modulo: &[0; 256],
                left: 0,
            };
            let (x, y) = affine_start(&registers.affine, &mosaic, line, Quantization::Reference)?;
            (x as i32, y as i32)
        } else {
            (0, 0)
        };
        Ok(Self {
            source,
            registers,
            even_frame,
            affine_start,
        })
    }

    /// A VRAM word; every index this path forms is already in VRAM.
    fn vram(&self, index: i32) -> Result<u16, RasterFault> {
        let index = usize::try_from(index).map_err(|_| RasterFault::Word(usize::MAX))?;
        self.source.word(index)
    }

    fn window(&self, layer: usize, x: i32) -> bool {
        window_contains(&self.registers.windows, layer, x, Quantization::Reference)
    }

    /// The first opaque layer at `x` on the main or sub screen.
    fn pixel(
        &self,
        sprites: &impl SpritePixels,
        x: i32,
        y: i32,
        sub: bool,
    ) -> Result<LayerPixel, RasterFault> {
        let registers = self.registers;
        let layout = registers.layout();
        let sprite = sprites.pixel(x)?;
        // The sprite word packs `(priority * 4 + 2) << 12` over its colour index.
        let tag = sprite >> 12;
        let sprite_priority = (tag & 3 == 2).then_some(tag >> 2);
        let screen = usize::from(sub);
        let mut found = None;
        for &(plane, priority) in layout.order {
            let index = match plane {
                Plane::Background(layer) => layer,
                Plane::Sprites => 4,
            };
            let enabled = registers.routing.enabled[screen] & (1 << index) != 0;
            let windowed = registers.routing.windowed[screen] & (1 << index) != 0;
            if !enabled || (windowed && self.window(index, x)) {
                continue;
            }
            let pixel = match plane {
                Plane::Background(layer) => self.background(layer, x, y, sub, priority != 0)?,
                Plane::Sprites if sprite_priority == Some(u16::from(priority)) => {
                    i32::from(sprite & 0xff)
                }
                Plane::Sprites => 0,
            };
            if pixel > 0 {
                found = Some((index, pixel));
                break;
            }
        }
        let (layer, pixel) = found.unwrap_or((5, 0));
        let direct = registers.direct_color && layer < 4 && layout.depths[layer] == Some(8);
        let color = if direct {
            [
                ((pixel & 7) << 2) | ((pixel & 0x100) >> 7),
                ((pixel & 0x38) >> 1) | ((pixel & 0x200) >> 8),
                ((pixel & 0xc0) >> 3) | ((pixel & 0x400) >> 8),
            ]
        } else {
            let word = i32::from(self.source.palette((pixel & 0xff) as u8)?);
            [word & 0x1f, (word >> 5) & 0x1f, (word >> 10) & 0x1f]
        };
        // Sprites from palettes 0-3 never take colour math: layer 6.
        let layer = if layer == 4 && pixel < 0xc0 { 6 } else { layer };
        Ok(LayerPixel {
            layer: layer as i32,
            color,
            non_backdrop: layer != 5,
        })
    }

    fn background(
        &self,
        layer: usize,
        x: i32,
        y: i32,
        sub: bool,
        priority: bool,
    ) -> Result<i32, RasterFault> {
        let registers = self.registers;
        let settings = &registers.layers[layer];
        let (mut lx, mut ly) = (x, y);
        if settings.mosaic && registers.mosaic_size > 1 {
            let size = i32::from(registers.mosaic_size);
            lx -= lx % size;
            ly -= (ly - i32::from(registers.mosaic_start_line)) % size;
        }
        if registers.mode == 7 {
            return self.affine(lx, layer, priority);
        }
        let background = &settings.background;
        lx += i32::from(background.horizontal_scroll);
        if matches!(registers.mode, 5 | 6) {
            lx = lx * 2 + i32::from(!(sub || settings.mosaic));
            if registers.interlace {
                ly = ly * 2 + i32::from(!(self.even_frame || settings.mosaic));
            }
        }
        ly += i32::from(background.vertical_scroll);
        if matches!(registers.mode, 2 | 4 | 6) {
            (lx, ly) = self.offset_per_tile(layer, lx, ly)?;
        }
        self.tile_pixel(layer, lx & 0x3ff, ly & 0x3ff, priority)
    }

    fn map_word(&self, layer: usize, x: i32, y: i32, wide: bool) -> Result<u16, RasterFault> {
        let settings = &self.registers.layers[layer];
        let background = &settings.background;
        let (bits_x, high_x) = if wide { (4, 0x200) } else { (3, 0x100) };
        let (bits_y, high_y) = if settings.big_tiles {
            (4, 0x200)
        } else {
            (3, 0x100)
        };
        let cell = ((((y >> bits_y) & 0x1f) << 5) | ((x >> bits_x) & 0x1f)) as u16;
        let mut map = background.map_words.wrapping_add(cell);
        if x & high_x != 0 && background.wider {
            map = map.wrapping_add(0x400);
        }
        if y & high_y != 0 && background.higher {
            map = map.wrapping_add(if background.wider { 0x800 } else { 0x400 });
        }
        self.vram(i32::from(map & 0x7fff))
    }

    /// Modes 2, 4 and 6 move each column by BG3's offset-per-tile map.
    fn offset_per_tile(&self, layer: usize, x: i32, y: i32) -> Result<(i32, i32), RasterFault> {
        let registers = self.registers;
        let scroll = i32::from(registers.layers[layer].background.horizontal_scroll);
        let column = if registers.mode == 6 {
            ((x - (x & 0xf)) - ((scroll * 2) & 0xfff0)) >> 4
        } else {
            ((x - (x & 7)) - (scroll & 0xfff8)) >> 3
        };
        if column <= 0 {
            return Ok((x, y));
        }
        let offsets = &registers.layers[2];
        let offset = |row: i32| {
            let ox = (column - 1) * 8 + i32::from(offsets.background.horizontal_scroll);
            let oy = row * 8 + i32::from(offsets.background.vertical_scroll);
            self.map_word(2, ox, oy, offsets.big_tiles)
        };
        let valid = if layer == 0 { 0x2000 } else { 0x4000 };
        let mut horizontal = offset(0)?;
        let mut vertical = 0;
        if registers.mode == 4 {
            if horizontal & 0x8000 != 0 {
                vertical = horizontal;
                horizontal = 0;
            }
        } else {
            vertical = offset(1)?;
        }
        let (mut lx, mut ly) = (x, y);
        if horizontal & valid != 0 {
            let moved = i32::from(horizontal & 0x3f8) + column * 8;
            lx = if registers.mode == 6 {
                (moved * 2) | (x & 0xf)
            } else {
                moved | (x & 7)
            };
        }
        if vertical & valid != 0 {
            let scroll = i32::from(registers.layers[layer].background.vertical_scroll);
            ly = i32::from(vertical & 0x3ff) + (y - scroll);
        }
        Ok((lx, ly))
    }

    fn tile_pixel(&self, layer: usize, x: i32, y: i32, priority: bool) -> Result<i32, RasterFault> {
        let registers = self.registers;
        let settings = &registers.layers[layer];
        let wide = settings.big_tiles || matches!(registers.mode, 5 | 6);
        let tile = i32::from(self.map_word(layer, x, y, wide)?);
        if (tile & 0x2000 != 0) != priority {
            return Ok(0);
        }
        let mut palette = (tile & 0x1c00) >> 10;
        let row = if tile & 0x8000 != 0 {
            7 - (y & 7)
        } else {
            y & 7
        };
        let column = if tile & 0x4000 != 0 {
            x & 7
        } else {
            7 - (x & 7)
        };
        let mut number = tile & 0x3ff;
        if wide && ((x & 8 != 0) != (tile & 0x4000 != 0)) {
            number += 1;
        }
        if settings.big_tiles && ((y & 8 != 0) != (tile & 0x8000 != 0)) {
            number += 0x10;
        }
        let depth = registers.layout().depths[layer].ok_or(RasterFault::Layer(layer as i32))?;
        let depth = i32::from(depth);
        if registers.mode == 0 {
            palette += 8 * layer as i32;
        }
        let base = i32::from(settings.background.tile_words) + (number & 0x3ff) * 4 * depth + row;
        let mut pixel = 0;
        let planes = match depth {
            ..=2 => 1,
            3..=4 => 2,
            _ => 4,
        };
        for pair in 0..planes {
            let word = i32::from(self.vram((base + pair * 8) & 0x7fff)?);
            let bits = ((word >> column) & 1) | (((word >> (8 + column)) & 1) << 1);
            pixel |= bits << (pair * 2);
        }
        if pixel == 0 {
            return Ok(0);
        }
        let colours = 1 << (planes * 2);
        Ok(colours * palette + pixel)
    }

    /// The Mode 7 map at screen column `x`; EXTBG's BG2 keeps bit 7 as its priority.
    fn affine(&self, x: i32, layer: usize, priority: bool) -> Result<i32, RasterFault> {
        let affine: &Affine = &self.registers.affine;
        let rx = i32::from(if affine.horizontal_flip {
            (255 - x) as u8
        } else {
            x as u8
        });
        let (start_x, start_y) = self.affine_start;
        let mut column = start_x.wrapping_add(i32::from(affine.matrix[0]).wrapping_mul(rx)) >> 8;
        let mut row = start_y.wrapping_add(i32::from(affine.matrix[2]).wrapping_mul(rx)) >> 8;
        let outside =
            affine.large_field && !((0..1024).contains(&column) && (0..1024).contains(&row));
        column &= 0x3ff;
        row &= 0x3ff;
        let tile = if outside {
            0
        } else {
            self.vram((row >> 3) * 128 + (column >> 3))? & 0xff
        };
        let pixel = if outside && !affine.character_fill {
            0
        } else {
            let index = i32::from(tile) * 64 + (row & 7) * 8 + (column & 7);
            i32::from(self.vram(index)? >> 8)
        };
        if layer == 1 {
            if (pixel & 0x80 != 0) != priority {
                return Ok(0);
            }
            return Ok(pixel & 0x7f);
        }
        Ok(pixel)
    }

    /// Compose and shade one line into `output`.
    pub fn draw(
        &self,
        sprites: &impl SpritePixels,
        brightness: Brightness<'_>,
        y: i32,
        output: &mut [Bgra; WIDTH],
    ) -> Result<(), RasterFault> {
        let registers = self.registers;
        let policy = Quantization::Reference;
        for (x, pixel) in (0i32..).zip(output.iter_mut()) {
            let main = self.pixel(sprites, x, y, false)?;
            let window = self.window(5, x);
            let hires = registers.pseudo_hires || matches!(registers.mode, 5 | 6);
            let second = if needs_subscreen(&registers.color, main, window, false)? || hires {
                self.pixel(sprites, x, y, true)?
            } else {
                LayerPixel {
                    layer: 5,
                    color: [0; 3],
                    non_backdrop: false,
                }
            };
            *pixel = shade_pixel(&registers.color, brightness, main, window, second, policy)?;
        }
        Ok(())
    }
}
