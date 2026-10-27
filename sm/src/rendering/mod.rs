//! Shared owned scanline computation; acquisition and publication belong to callers.
#![forbid(unsafe_code)]

pub mod prepared;

mod records;
pub use records::*;

const SPRITE_SIZES: [[i32; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [16, 32],
    [16, 32],
];

const BIT_SPREAD: [u32; 256] = {
    let mut table = [0; 256];
    let mut byte = 0;
    while byte < table.len() {
        let mut bit = 0;
        while bit < 8 {
            table[byte] |= (((byte >> bit) & 1) as u32) << (bit * 4);
            bit += 1;
        }
        byte += 1;
    }
    table
};

pub fn sprite_span(dst: &mut [u16], plane: u32, z: u16, left: u32, flipped: bool) {
    let mut pixels = BIT_SPREAD[(plane & 255) as usize]
        | BIT_SPREAD[((plane >> 8) & 255) as usize] << 1
        | BIT_SPREAD[((plane >> 16) & 255) as usize] << 2
        | BIT_SPREAD[((plane >> 24) & 255) as usize] << 3;
    if flipped {
        pixels >>= left * 4;
        for dst in dst {
            let pixel = (pixels & 15) as u16;
            if pixel != 0 && *dst & 255 == 0 {
                *dst = z.wrapping_add(pixel);
            }
            pixels >>= 4;
        }
    } else {
        pixels <<= left * 4;
        for dst in dst {
            let pixel = (pixels >> 28) as u16;
            if pixel != 0 && *dst & 255 == 0 {
                *dst = z.wrapping_add(pixel);
            }
            pixels <<= 4;
        }
    }
}

/// The caller clears its narrow workspace first; writes survive error or unwinding.
pub fn evaluate_sprites(
    source: &impl IndexedSource,
    input: &SpriteInput<'_>,
    line: i32,
    pixels: &mut [u16; WIDTH],
    mut overflow: Overflow,
) -> Result<(bool, Overflow), RasterFault> {
    let mut index = input.rotation.map_or(0, |index| index & 0xfe);
    let mut sprites = 0;
    let mut tiles = 0;
    for _ in 0..128 {
        let y = (input.words[usize::from(index)] >> 8) as u8;
        let mut row = (line - i32::from(y)) as u8;
        let sizes = SPRITE_SIZES
            .get(usize::from(input.size))
            .ok_or(RasterFault::SpriteSize(input.size))?;
        let extension = input.extension[usize::from(index >> 3)];
        let size = sizes[usize::from((extension >> ((index & 7) + 1)) & 1)];
        let height = if input.interlace { size / 2 } else { size };
        if i32::from(row) < height {
            let mut x = i32::from(input.words[usize::from(index)] & 255)
                | (i32::from(input.extension[usize::from(index >> 3)] >> (index & 7) & 1) << 8);
            if x > 255 {
                x -= 512;
            }
            if x > -size {
                sprites += 1;
                if sprites > 32 {
                    overflow.range = true;
                    break;
                }
                if input.interlace {
                    row = (i32::from(row) * 2 + i32::from(!input.even_frame)) as u8;
                }
                let attribute = i32::from(input.words[usize::from(index) + 1]);
                let base = i32::from(input.tile_words[usize::from(attribute & 0x100 != 0)]);
                let flipped = attribute & 0x4000 != 0;
                if attribute & 0x8000 != 0 {
                    row = (size - 1 - i32::from(row)) as u8;
                }
                let palette = 0x80 + 16 * ((attribute & 0xe00) >> 9);
                let priority = (((attribute & 0x3000) >> 12) * 4 + 2) * 16
                    + 4
                    + if attribute & 0x800 == 0 { 2 } else { 0 };
                let z = (palette + (priority << 8)) as u16;
                let mut column = 0;
                while column < size {
                    if column + x > -8 && column + x < 256 {
                        tiles += 1;
                        if tiles > 34 {
                            overflow.time = true;
                            break;
                        }
                        let used_column = if flipped { size - 1 - column } else { column };
                        let tile = (((attribute & 255) >> 4) + (i32::from(row) >> 3)) << 4
                            | (((attribute & 15) + (used_column >> 3)) & 15);
                        let address = (base + tile * 16 + (i32::from(row) & 7)) & 0x7fff;
                        let low = source.word(address as usize)?;
                        let high = source.word(address as usize + 8)?;
                        let plane = u32::from(low) | u32::from(high) << 16;
                        let left = (-(column + x)).max(0);
                        let right = (256 - (column + x)).min(8);
                        if left < right {
                            let start = (column + x + left) as usize;
                            sprite_span(
                                &mut pixels[start..start + (right - left) as usize],
                                plane,
                                z,
                                left as u32,
                                flipped,
                            );
                        }
                    }
                    column += 8;
                }
                if tiles > 34 {
                    break;
                }
            }
        }
        index = index.wrapping_add(2);
    }
    Ok((tiles != 0, overflow))
}

pub fn background_pixel(
    source: &impl IndexedSource,
    bg: &Background,
    x: i32,
    y: i32,
    layer: usize,
) -> Result<i32, RasterFault> {
    let mut address = i32::from(bg.map_words) + ((y >> 3 & 31) << 5);
    if x & 256 != 0 && bg.wider {
        address += 0x400;
    }
    if y & 256 != 0 && bg.higher {
        address += if bg.wider { 0x800 } else { 0x400 };
    }
    let tile = source.word(((address & 0x7fff) + (x >> 3 & 31)) as usize)?;
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
    let depth = if layer == 2 { 2 } else { 4 };
    let address = (i32::from(bg.tile_words) + i32::from(tile & 0x3ff) * 4 * depth + row) & 0x7fff;
    let mut pixel = 0;
    for pair in 0..depth / 2 {
        let word = i32::from(source.word((address + pair * 8) as usize)?);
        pixel |= ((word >> column & 1) | (word >> (column + 7) & 2)) << (pair * 2);
    }
    if pixel == 0 {
        return Ok(0);
    }
    let color = (1 << depth) * (i32::from(tile & 0x1c00) >> 10) + pixel;
    let priorities = [[0x8000, 0xc000], [0x7100, 0xb100], [0x1200, 0xf200]];
    Ok(priorities[layer][usize::from(tile & 0x2000 != 0)] + color)
}

pub fn affine_coordinates(matrix: &[i16; 8]) -> (i32, i32, i32, i32) {
    let signed13 = |word: i16| i32::from(word.wrapping_shl(3) >> 3);
    let horizontal = signed13(matrix[6]);
    let vertical = signed13(matrix[7]);
    let center_x = signed13(matrix[4]);
    let center_y = signed13(matrix[5]);
    let clip = |delta: i32| {
        if delta & 0x2000 != 0 {
            delta | !1023
        } else {
            delta & 1023
        }
    };
    (
        center_x,
        center_y,
        clip(horizontal - center_x),
        clip(vertical - center_y),
    )
}

pub fn affine_start(
    affine: &Affine,
    mosaic: &Mosaic<'_>,
    mut y: i32,
    policy: Quantization,
) -> Result<(u32, u32), RasterFault> {
    let (cx, cy, horizontal, vertical) = affine_coordinates(&affine.matrix);
    match policy {
        Quantization::Native if mosaic.native_enabled & 1 != 0 => {
            y = i32::from(
                *usize::try_from(y)
                    .ok()
                    .and_then(|y| mosaic.modulo.get(y))
                    .ok_or(RasterFault::MosaicIndex(y))?,
            );
        }
        Quantization::Reference if mosaic.background0_enabled && mosaic.size > 1 => {
            y -= (y - i32::from(mosaic.start_line)) % i32::from(mosaic.size);
        }
        _ => {}
    }
    let ry = if affine.vertical_flip { 255 - y } else { y };
    let ry = match policy {
        Quantization::Native => ry as u32,
        Quantization::Reference => u32::from(ry as u8),
    };
    let start = |a: i16, b: i16, center: i32| {
        ((i32::from(a) * horizontal & !63) as u32)
            .wrapping_add((b as u32).wrapping_mul(ry) & !63)
            .wrapping_add((i32::from(b) * vertical & !63) as u32)
            .wrapping_add((center << 8) as u32)
    };
    Ok((
        start(affine.matrix[0], affine.matrix[1], cx),
        start(affine.matrix[2], affine.matrix[3], cy),
    ))
}

pub fn affine_sample(
    source: &impl IndexedSource,
    x: u32,
    y: u32,
    outside: u32,
    character_fill: bool,
) -> Result<u8, RasterFault> {
    let tile = if x | y > outside {
        if !character_fill {
            return Ok(0);
        }
        0
    } else {
        source.word(((y >> 11 & 127) * 128 + (x >> 11 & 127)) as usize)? & 255
    };
    Ok(
        (source.word((u32::from(tile) * 64 + (y >> 8 & 7) * 8 + (x >> 8 & 7)) as usize)? >> 8)
            as u8,
    )
}

fn affine_x(config: &LineConfig<'_>, x: i32, sub: bool) -> Result<i32, RasterFault> {
    if config.mosaic.native_enabled & 1 == 0 {
        return Ok(x);
    }
    let mut left = config.mosaic.left;
    if config.routing.windowed[usize::from(sub)] & 1 != 0 {
        for (enabled, bounds) in [(2, config.windows.first), (8, config.windows.second)] {
            if config.windows.native_flags & enabled != 0 && bounds[0] <= bounds[1] {
                for edge in [i32::from(bounds[0]), i32::from(bounds[1]) + 1] {
                    if edge <= x {
                        left = left.max(edge);
                    }
                }
            }
        }
    }
    let width = i32::from(config.mosaic.size);
    let modulo = i32::from(
        *usize::try_from(left)
            .ok()
            .and_then(|left| config.mosaic.modulo.get(left))
            .ok_or(RasterFault::MosaicIndex(left))?,
    );
    let first_end = left + width - (left - modulo);
    if x < first_end {
        return Ok(left);
    }
    if width == 0 {
        return Err(RasterFault::MosaicSize);
    }
    Ok(first_end + (x - first_end) / width * width)
}

/// C can use this pure gate before acquiring a signed-x native sprite alias.
pub fn sprite_enabled(config: &LineConfig<'_>, x: i32, sub: bool) -> bool {
    config.has_sprites && config.routing.enabled(&config.windows, 4, x, sub)
}

pub fn native_pixel(
    source: &impl IndexedSource,
    config: &LineConfig<'_>,
    sprites: &impl SpritePixels,
    x: i32,
    y: i32,
    sub: bool,
    starts: (u32, u32),
) -> Result<LayerPixel, RasterFault> {
    let sprite = if sprite_enabled(config, x, sub) {
        i32::from(sprites.pixel(x)?)
    } else {
        i32::from(BACKDROP)
    };
    let mut packed = i32::from(BACKDROP);
    if config.mode == 1 {
        packed = sprite;
        for layer in 0..3 {
            if config.routing.enabled(&config.windows, layer, x, sub) {
                let bg = config.backgrounds[layer];
                let pixel = background_pixel(
                    source,
                    &bg,
                    x + i32::from(bg.horizontal_scroll),
                    y + i32::from(bg.vertical_scroll),
                    layer,
                )?;
                let mask = if layer == 2 { 3 } else { 15 };
                if pixel != 0 && pixel & !mask > packed {
                    packed = pixel;
                }
            }
        }
    } else {
        if config.routing.enabled(&config.windows, 0, x, sub) {
            let mx = affine_x(config, x, sub)?;
            let rx = if config.affine.horizontal_flip {
                255 - mx
            } else {
                mx
            } as u32;
            let pixel = affine_sample(
                source,
                starts
                    .0
                    .wrapping_add((config.affine.matrix[0] as u32).wrapping_mul(rx)),
                starts
                    .1
                    .wrapping_add((config.affine.matrix[2] as u32).wrapping_mul(rx)),
                if config.affine.large_field {
                    0x3ffff
                } else {
                    u32::MAX
                },
                config.affine.character_fill,
            )?;
            if pixel != 0 {
                packed = 0x5000 + i32::from(pixel);
            }
        }
        packed = packed.max(sprite);
    }
    let index = (packed & 255) as u8;
    let color = source.palette(index)?;
    Ok(LayerPixel {
        layer: packed >> 8 & 15,
        color: [
            i32::from(color & 31),
            i32::from(color >> 5 & 31),
            i32::from(color >> 10 & 31),
        ],
        non_backdrop: index != 0,
    })
}

fn permits(mode: u8, window: bool) -> bool {
    !(mode == 3 || mode == 2 && window || mode == 1 && !window)
}

fn math_enabled(math: &ColorMath, layer: i32, window: bool) -> Result<bool, RasterFault> {
    if layer >= 6 {
        return Ok(false);
    }
    let enabled = *math
        .enabled
        .get(layer as usize)
        .ok_or(RasterFault::Layer(layer))?;
    Ok(enabled && permits(math.prevent, window))
}

/// Determine this after the main sample/window, before any subscreen query.
pub fn needs_subscreen(
    math: &ColorMath,
    main: LayerPixel,
    window: bool,
    reference_hires: bool,
) -> Result<bool, RasterFault> {
    let enabled = math_enabled(math, main.layer, window)?;
    Ok(enabled && math.add_subscreen || reference_hires)
}

pub fn shade_pixel(
    math: &ColorMath,
    tables: Brightness<'_>,
    main: LayerPixel,
    window: bool,
    second: LayerPixel,
    policy: Quantization,
) -> Result<Bgra, RasterFault> {
    let mut color = main.color;
    if !permits(math.clip, window) {
        color = [0; 3];
    }
    let enabled = math_enabled(math, main.layer, window)?;
    let subscreen = math.add_subscreen && second.non_backdrop;
    let half = enabled && math.half && (second.non_backdrop || !math.add_subscreen);
    let fixed =
        u32::from(math.fixed[0]) | u32::from(math.fixed[1]) << 5 | u32::from(math.fixed[2]) << 10;
    let mut output = [0; 4];
    for channel in 0..3 {
        let mut value = color[channel];
        if enabled {
            let operand = if subscreen {
                second.color[channel]
            } else if policy == Quantization::Native {
                ((fixed >> (channel * 5)) & 31) as i32
            } else {
                i32::from(math.fixed[channel])
            };
            value += if math.subtract { -operand } else { operand };
        }
        output[2 - channel] = match policy {
            Quantization::Native => {
                let index = value.max(0) as usize;
                if half {
                    tables.half.get(index)
                } else {
                    tables.normal.get(index)
                }
                .copied()
                .ok_or(RasterFault::BrightnessIndex(index))?
            }
            Quantization::Reference => {
                if half {
                    value >>= 1;
                }
                if enabled {
                    value = value.clamp(0, 31);
                }
                ((value << 3 | value >> 2) * i32::from(math.brightness) / 15) as u8
            }
        };
    }
    Ok(output)
}

pub fn render_pixel(
    source: &impl IndexedSource,
    config: &LineConfig<'_>,
    sprites: &impl SpritePixels,
    x: i32,
    y: i32,
    starts: (u32, u32),
) -> Result<Bgra, RasterFault> {
    if config.forced_blank {
        return Ok([0; 4]);
    }
    let main = native_pixel(source, config, sprites, x, y, false, starts)?;
    let window = config.windows.contains(5, x, Quantization::Native);
    let second = if needs_subscreen(&config.color, main, window, false)? {
        native_pixel(source, config, sprites, x, y, true, starts)?
    } else {
        LayerPixel {
            layer: 5,
            color: [0; 3],
            non_backdrop: false,
        }
    };
    shade_pixel(
        &config.color,
        config.brightness,
        main,
        window,
        second,
        Quantization::Native,
    )
}

/// Call after sprite evaluation and before the first native pixel publication.
/// C preserves the original assertion handler for EnabledMosaic failures.
pub fn native_starts(config: &LineConfig<'_>, y: i32) -> Result<(u32, u32), RasterFault> {
    if config.forced_blank {
        return Ok((0, 0));
    }
    if config.mode == 1 {
        for layer in 0..3 {
            if config.mosaic.native_enabled & (1 << layer) != 0 {
                return Err(RasterFault::EnabledMosaic(layer));
            }
        }
        Ok((0, 0))
    } else {
        affine_start(&config.affine, &config.mosaic, y, Quantization::Native)
    }
}

/// Pixels already written to output remain available if a later read fails.
pub fn render_line(
    source: &impl IndexedSource,
    config: &LineConfig<'_>,
    sprites: &impl SpritePixels,
    y: i32,
    output: &mut [Bgra; WIDTH],
) -> Result<(), RasterFault> {
    if config.forced_blank {
        output.fill([0; 4]);
        return Ok(());
    }
    let starts = native_starts(config, y)?;
    for (x, pixel) in output.iter_mut().enumerate() {
        *pixel = render_pixel(source, config, sprites, x as i32, y, starts)?;
    }
    Ok(())
}
