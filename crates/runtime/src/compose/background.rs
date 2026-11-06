//! A background's dots on one line: tiled modes 0-6 and the Mode 7 map.

use super::setup::{Affine, Background, Bg, Depth, LineSetup, Mode, Outside};
use super::{Dot, WIDTH};
use data::video::VRAM_WORDS;

/// What every background on a line reads.
pub struct LineContext<'a> {
    pub setup: &'a LineSetup,
    pub vram: &'a [u16; VRAM_WORDS],
    pub line: u16,
    pub mosaic_start: u16,
    pub even_frame: bool,
}

impl LineContext<'_> {
    /// A VRAM word; addresses wrap at 32K words.
    fn word(&self, address: u16) -> u16 {
        self.vram
            .get(usize::from(address & 0x7fff))
            .copied()
            .unwrap_or_default()
    }

    fn background(&self, layer: Bg) -> Background {
        self.setup
            .backgrounds
            .get(layer.index())
            .copied()
            .unwrap_or_default()
    }
}

/// The background's dots on the line, or `None` if the mode has no such background.
pub fn draw(context: &LineContext<'_>, layer: Bg) -> Option<[Dot; WIDTH]> {
    let setup = context.setup;
    let depth = setup.mode.depths().get(layer.index()).copied().flatten()?;
    let settings = context.background(layer);
    let mosaic = settings.mosaic && setup.mosaic_size > 1;
    let size = u16::from(setup.mosaic_size);
    let y = if mosaic {
        let into = context
            .line
            .wrapping_sub(context.mosaic_start)
            .cast_signed();
        let into = into.rem_euclid(size.cast_signed()).cast_unsigned();
        context.line.wrapping_sub(into)
    } else {
        context.line
    };
    let mut dots = [Dot::default(); WIDTH];
    if let Mode::Seven { .. } = setup.mode {
        let starts = affine_starts(&setup.affine, y);
        for (x, dot) in (0_u8..=u8::MAX).zip(dots.iter_mut()) {
            let x = if mosaic {
                x.wrapping_sub(x.checked_rem(setup.mosaic_size).unwrap_or_default())
            } else {
                x
            };
            let index = affine_dot(context, &setup.affine, starts, x);
            *dot = match layer {
                Bg::Two => Dot {
                    index: index & 0x7f,
                    priority: u8::from(index & 0x80 != 0),
                },
                Bg::One | Bg::Three | Bg::Four => Dot { index, priority: 0 },
            };
        }
        return Some(dots);
    }
    let hires = setup.mode.hires();
    // Dots of one tile row share its fetch.
    let mut cached: Option<((u16, u16), Sliver)> = None;
    for (x, dot) in (0_u16..).zip(dots.iter_mut()) {
        let mut lx = if mosaic {
            x.wrapping_sub(x.checked_rem(size).unwrap_or_default())
        } else {
            x
        };
        let mut ly = y;
        lx = lx.wrapping_add(settings.horizontal);
        if hires {
            lx = lx.wrapping_mul(2).wrapping_add(u16::from(!settings.mosaic));
            if setup.interlace {
                ly = ly
                    .wrapping_mul(2)
                    .wrapping_add(u16::from(!(context.even_frame || settings.mosaic)));
            }
        }
        ly = ly.wrapping_add(settings.vertical);
        if setup.mode.offset_per_tile() {
            (lx, ly) = offset_per_tile(context, &settings, lx, ly, layer);
        }
        let (lx, ly) = (lx & 0x3ff, ly & 0x3ff);
        let key = (lx >> 3, ly);
        let sliver = match cached {
            Some((at, sliver)) if at == key => sliver,
            _ => {
                let sliver = Sliver::fetch(context, &settings, depth, layer, lx, ly);
                cached = Some((key, sliver));
                sliver
            }
        };
        *dot = sliver.dot(lx);
    }
    Some(dots)
}

/// The tile map word at a background position.
fn map_word(context: &LineContext<'_>, settings: &Background, x: u16, y: u16, wide: bool) -> u16 {
    let (shift_x, high_x) = if wide { (4, 0x200) } else { (3, 0x100) };
    let (shift_y, high_y) = if settings.big_tiles {
        (4, 0x200)
    } else {
        (3, 0x100)
    };
    let cell = (((y >> shift_y) & 0x1f) << 5) | ((x >> shift_x) & 0x1f);
    let mut address = settings.map.wrapping_add(cell);
    if x & high_x != 0 && settings.size.wider {
        address = address.wrapping_add(0x400);
    }
    if y & high_y != 0 && settings.size.higher {
        address = address.wrapping_add(if settings.size.wider { 0x800 } else { 0x400 });
    }
    context.word(address)
}

/// Modes 2, 4 and 6 move each column by BG3's offset-per-tile map.
fn offset_per_tile(
    context: &LineContext<'_>,
    settings: &Background,
    x: u16,
    y: u16,
    layer: Bg,
) -> (u16, u16) {
    let mode = context.setup.mode;
    let (x_signed, scroll) = (i32::from(x), i32::from(settings.horizontal));
    let column = if mode == Mode::Six {
        (x_signed
            .wrapping_sub(x_signed & 0xf)
            .wrapping_sub(scroll.wrapping_mul(2) & 0xfff0))
            >> 4
    } else {
        (x_signed
            .wrapping_sub(x_signed & 7)
            .wrapping_sub(scroll & 0xfff8))
            >> 3
    };
    let Ok(column) = u16::try_from(column) else {
        return (x, y);
    };
    if column == 0 {
        return (x, y);
    }
    let offsets = context.background(Bg::Three);
    let offset = |row: u16| {
        let ox = column
            .wrapping_sub(1)
            .wrapping_mul(8)
            .wrapping_add(offsets.horizontal);
        let oy = row.wrapping_mul(8).wrapping_add(offsets.vertical);
        map_word(context, &offsets, ox, oy, offsets.big_tiles)
    };
    let valid = if layer == Bg::One { 0x2000 } else { 0x4000 };
    let mut horizontal = offset(0);
    let mut vertical = 0;
    if mode == Mode::Four {
        if horizontal & 0x8000 != 0 {
            vertical = horizontal;
            horizontal = 0;
        }
    } else {
        vertical = offset(1);
    }
    let (mut lx, mut ly) = (x, y);
    if horizontal & valid != 0 {
        let moved = (horizontal & 0x3f8).wrapping_add(column.wrapping_mul(8));
        lx = if mode == Mode::Six {
            moved.wrapping_mul(2) | (x & 0xf)
        } else {
            moved | (x & 7)
        };
    }
    if vertical & valid != 0 {
        ly = (vertical & 0x3ff).wrapping_add(y.wrapping_sub(settings.vertical));
    }
    (lx, ly)
}

/// One tile row of a tiled background: its bit planes and how a dot reads them.
#[derive(Clone, Copy, Debug)]
struct Sliver {
    planes: [u16; 4],
    pairs: u16,
    colour_base: u16,
    priority: u8,
    horizontal_flip: bool,
}

impl Sliver {
    /// The tile row under a background position.
    fn fetch(
        context: &LineContext<'_>,
        settings: &Background,
        depth: Depth,
        layer: Bg,
        x: u16,
        y: u16,
    ) -> Self {
        let mode = context.setup.mode;
        let wide = settings.big_tiles || mode.hires();
        let tile = map_word(context, settings, x, y, wide);
        let vertical_flip = tile & 0x8000 != 0;
        let horizontal_flip = tile & 0x4000 != 0;
        let row = if vertical_flip { 7 ^ (y & 7) } else { y & 7 };
        let mut number = tile & 0x3ff;
        if wide && ((x & 8 != 0) != horizontal_flip) {
            number = number.wrapping_add(1);
        }
        if settings.big_tiles && ((y & 8 != 0) != vertical_flip) {
            number = number.wrapping_add(0x10);
        }
        let mut palette = (tile & 0x1c00) >> 10;
        if mode == Mode::Zero {
            palette = palette
                .wrapping_add(8_u16.wrapping_mul(u16::try_from(layer.index()).unwrap_or_default()));
        }
        let pairs = depth.plane_pairs();
        let base = settings
            .tiles
            .wrapping_add((number & 0x3ff).wrapping_mul(8).wrapping_mul(pairs))
            .wrapping_add(row);
        let mut planes = [0; 4];
        for (pair, plane) in (0..pairs).zip(planes.iter_mut()) {
            *plane = context.word(base.wrapping_add(pair.wrapping_mul(8)));
        }
        Self {
            planes,
            pairs,
            colour_base: (1_u16 << pairs.wrapping_mul(2)).wrapping_mul(palette),
            priority: u8::from(tile & 0x2000 != 0),
            horizontal_flip,
        }
    }

    /// The dot at column `x & 7` of the row.
    fn dot(&self, x: u16) -> Dot {
        let column = if self.horizontal_flip {
            x & 7
        } else {
            7 ^ (x & 7)
        };
        let mut pixel: u16 = 0;
        for (pair, word) in (0..self.pairs).zip(self.planes) {
            let bits = ((word >> column) & 1) | (((word >> column.wrapping_add(8)) & 1) << 1);
            pixel |= bits << pair.wrapping_mul(2);
        }
        let index = if pixel == 0 {
            0
        } else {
            self.colour_base.wrapping_add(pixel)
        };
        Dot {
            index,
            priority: self.priority,
        }
    }
}

/// The Mode 7 start of the line: the transform applied to the line's first dot, in 1/256
/// map dots.
fn affine_starts(affine: &Affine, y: u16) -> [i32; 2] {
    let [centre_x, centre_y] = affine.centre;
    let [scroll_x, scroll_y] = affine.scroll;
    let clip = |delta: i32| {
        if delta & 0x2000 != 0 {
            delta | !1023
        } else {
            delta & 1023
        }
    };
    let horizontal = clip(scroll_x.wrapping_sub(centre_x));
    let vertical = clip(scroll_y.wrapping_sub(centre_y));
    let y = u8::try_from(y & 0xff).unwrap_or_default();
    let ry = i32::from(if affine.flip.vertical { !y } else { y });
    let [m7a, m7b, m7c, m7d] = affine.matrix.map(i32::from);
    let start = |first: i32, second: i32, centre: i32| {
        (first.wrapping_mul(horizontal) & !63)
            .wrapping_add(second.wrapping_mul(ry) & !63)
            .wrapping_add(second.wrapping_mul(vertical) & !63)
            .wrapping_add(centre.wrapping_shl(8))
    };
    [start(m7a, m7b, centre_x), start(m7c, m7d, centre_y)]
}

/// The Mode 7 map's colour index at screen dot `x`.
fn affine_dot(
    context: &LineContext<'_>,
    affine: &Affine,
    [start_x, start_y]: [i32; 2],
    x: u8,
) -> u16 {
    let rx = i32::from(if affine.flip.horizontal { !x } else { x });
    let [m7a, _, m7c, _] = affine.matrix.map(i32::from);
    let column = start_x.wrapping_add(m7a.wrapping_mul(rx)) >> 8;
    let row = start_y.wrapping_add(m7c.wrapping_mul(rx)) >> 8;
    let inside = (0..1024).contains(&column) && (0..1024).contains(&row);
    let column = u16::try_from(column & 0x3ff).unwrap_or_default();
    let row = u16::try_from(row & 0x3ff).unwrap_or_default();
    let tile = match (inside, affine.outside) {
        (true, _) | (false, Outside::Wrap) => {
            context.word(((row >> 3) << 7) | (column >> 3)) & 0xff
        }
        (false, Outside::Transparent) => return 0,
        (false, Outside::TileZero) => 0,
    };
    context.word((tile << 6) | ((row & 7) << 3) | (column & 7)) >> 8
}
