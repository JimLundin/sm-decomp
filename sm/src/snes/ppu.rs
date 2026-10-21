use crate::game_state::Game;
use crate::set_fields;
use ::core::ffi::{c_char, c_long, c_void};
use ::core::mem::{size_of, transmute};

use crate::c_library::{__assert_fail, free, malloc, memcpy, memset};
pub use crate::types::{
    size_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, SaveLoadFunc, Snes, Spc,
    Timer, WindowLayer,
};
crate::type_aliases! {
    pub C2RustUnnamed_0, C2RustUnnamed_1, C2RustUnnamed_2, C2RustUnnamed_3 = u32;
}
pub const kPpuExtraLeftRight: C2RustUnnamed_0 = 0;
pub const kPpuXPixels: C2RustUnnamed_0 = 256;

pub const kWindow2Inversed: C2RustUnnamed_3 = 4;
pub const kWindow2Enabled: C2RustUnnamed_3 = 8;
pub const kWindow1Inversed: C2RustUnnamed_3 = 1;
pub const kWindow1Enabled: C2RustUnnamed_3 = 2;

pub const kPaletteShift: C2RustUnnamed_1 = 8;
pub const kPaletteShift_0: C2RustUnnamed_2 = 6;
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
pub const __ASSERT_FUNCTION: [c_char; 43] = unsafe {
    transmute::<[u8; 43], [c_char; 43]>(*b"void PpuDrawBackgrounds(Ppu *, int, _Bool)\0")
};
#[inline]
unsafe fn IntMin(a: i32, b: i32) -> i32 {
    if a < b {
        a
    } else {
        b
    }
}
#[inline]
unsafe fn IntMax(a: i32, b: i32) -> i32 {
    if a > b {
        a
    } else {
        b
    }
}
static layersPerMode: [[i32; 12]; 10] = [
    [4, 0, 1, 4, 0, 1, 4, 2, 3, 4, 2, 3],
    [4, 0, 1, 4, 0, 1, 4, 2, 4, 2, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 4, 0, 4, 5, 5, 5, 5, 5, 5],
    [4, 4, 4, 0, 4, 5, 5, 5, 5, 5, 5, 5],
    [2, 4, 0, 1, 4, 0, 1, 4, 4, 2, 5, 5],
    [4, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5, 5],
];
static prioritysPerMode: [[i32; 12]; 10] = [
    [3, 1, 1, 2, 0, 0, 1, 1, 1, 0, 0, 0],
    [3, 1, 1, 2, 0, 0, 1, 1, 0, 0, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 0, 0, 5, 5, 5, 5, 5, 5],
    [3, 2, 1, 0, 0, 5, 5, 5, 5, 5, 5, 5],
    [1, 3, 1, 1, 2, 0, 0, 1, 0, 0, 5, 5],
    [3, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5, 5],
];
static layerCountPerMode: [i32; 10] = [12, 10, 8, 8, 8, 8, 6, 5, 10, 7];
static bitDepthsPerMode: [[i32; 4]; 10] = [
    [2, 2, 2, 2],
    [4, 4, 2, 5],
    [4, 4, 5, 5],
    [8, 4, 5, 5],
    [8, 2, 5, 5],
    [4, 2, 5, 5],
    [4, 5, 5, 5],
    [8, 5, 5, 5],
    [4, 4, 2, 5],
    [8, 7, 5, 5],
];
static spriteSizes: [[i32; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [16, 32],
    [16, 32],
];
pub unsafe fn ppu_init(snes: *mut Snes) -> *mut Ppu {
    let ppu = malloc(size_of::<Ppu>() as size_t) as *mut Ppu;
    (*ppu).renderBuffer = core::ptr::null_mut();
    (*ppu).renderPitch = 0;
    (*ppu).snes = snes;
    ppu
}
pub unsafe fn ppu_copy(ppu: *mut Ppu, ppu_src: *mut Ppu) {
    let snes = (*ppu).snes;
    let pitch: size_t = (*ppu).renderPitch as size_t;
    let renderBuffer = (*ppu).renderBuffer;
    memcpy(
        ppu as *mut c_void,
        ppu_src as *const c_void,
        size_of::<Ppu>() as size_t,
    );
    (*ppu).renderBuffer = renderBuffer;
    (*ppu).renderPitch = pitch as u32;
    (*ppu).snes = snes;
}
pub unsafe fn ppu_reset(ppu: *mut Ppu) {
    let snes = (*ppu).snes;
    let pitch = (*ppu).renderPitch;
    let render_buffer = (*ppu).renderBuffer;
    // The first reset initializes a partially constructed allocation.
    core::ptr::write_bytes(ppu, 0, 1);
    (*ppu).renderBuffer = render_buffer;
    (*ppu).renderPitch = pitch;
    (*ppu).snes = snes;
    set_fields! {[(*ppu)], 1; vramIncrement, mosaicSize, mosaicStartLine}
    (*ppu).forcedBlank = true;
}

pub unsafe fn ppu_saveload(
    ppu: *mut Ppu,
    func: Option<SaveLoadFunc>,
    ctx: *mut c_void,
    game: &mut Game,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*ppu).vram as *mut c_void,
        (67313 as size_t).wrapping_sub(8 as size_t),
        game,
    );
}
// These duplicate renderer inputs are outside the legacy serialized PPU range.
pub fn PpuRestoreRenderRouting(ppu: &mut Ppu) {
    ppu.screenEnabled = [0; 2];
    ppu.screenWindowed = [0; 2];
    for (index, layer) in ppu.layer.iter().enumerate() {
        ppu.screenEnabled[0] |= u8::from(layer.mainScreenEnabled) << index;
        ppu.screenEnabled[1] |= u8::from(layer.subScreenEnabled) << index;
        ppu.screenWindowed[0] |= u8::from(layer.mainScreenWindowed) << index;
        ppu.screenWindowed[1] |= u8::from(layer.subScreenWindowed) << index;
    }
    ppu.windowsel = 0;
    for (index, window) in ppu.windowLayer.iter().enumerate() {
        let flags = u32::from(window.window1inversed)
            | (u32::from(window.window1enabled) << 1)
            | (u32::from(window.window2inversed) << 2)
            | (u32::from(window.window2enabled) << 3);
        ppu.windowsel |= flags << (index * 4);
    }
}
pub unsafe fn PpuBeginDrawing(ppu: *mut Ppu, pixels: *mut u8, pitch: size_t, render_flags: u32) {
    (*ppu).renderPitch = pitch as u32;
    (*ppu).renderBuffer = pixels;
}
pub unsafe fn ppu_checkOverscan(ppu: *mut Ppu) -> bool {
    (*ppu).frameOverscan = (*ppu).overscan;
    (*ppu).frameOverscan
}
pub unsafe fn ppu_handleVblank(ppu: *mut Ppu) {
    if !(*ppu).forcedBlank {
        (*ppu).oamAdr = (*ppu).oamAdrWritten;
        (*ppu).oamInHigh = (*ppu).oamInHighWritten;
        (*ppu).oamSecondWrite = false_0 != 0;
    }
    (*ppu).frameInterlace = (*ppu).interlace;
}
#[inline]
fn ClearBackdrop(buf: &mut PpuPixelPrioBufs) {
    buf.data.fill(0x0500);
}
fn update_brightness_tables(brightness: u8, normal: &mut [u8; 63], half: &mut [u8; 64]) {
    for (i, multiplier) in normal[..32].iter_mut().enumerate() {
        let color = i as u16;
        let value = (((color << 3 | color >> 2) * u16::from(brightness)) / 15) as u8;
        *multiplier = value;
        half[i * 2 + 1] = value;
        half[i * 2] = value;
    }
    let maximum = normal[31];
    normal[32..].fill(maximum);
}
pub unsafe fn ppu_runLine(ppu: *mut Ppu, line: i32, game: &mut Game) {
    if line == 0 {
        set_fields! {[(*ppu)]; mosaicStartLine = 1, rangeOver = false_0 != 0,
        timeOver = false_0 != 0, evenFrame = !(*ppu).evenFrame, };
    } else {
        if (*ppu).brightness != (*ppu).lastBrightnessMult {
            let brightness = (*ppu).brightness;
            (*ppu).lastBrightnessMult = brightness;
            update_brightness_tables(
                brightness,
                &mut (*ppu).brightnessMult,
                &mut (*ppu).brightnessMultHalf,
            );
        }
        ClearBackdrop(&mut (*ppu).objBuffer);
        (*ppu).lineHasSprites =
            !(*ppu).forcedBlank && ppu_evaluateSprites(ppu, line - 1i32) as i32 != 0;
        // EXTBG composition and enabled mosaic use the per-pixel path.
        let mosaic = (*ppu).mosaicSize > 1 && (*ppu).bgLayer.iter().any(|bg| bg.mosaicEnabled);
        if game.g_new_ppu && !((*ppu).mode == 7 && (*ppu).m7extBg) && !mosaic {
            PpuDrawWholeLine(ppu, line as u32);
        } else {
            if (*ppu).mode as i32 == 7 {
                ppu_calculateMode7Starts(ppu, line);
            }
            for x in 0i32..256 {
                ppu_render_pixel::<false>(ppu, x, line, (0, 0));
            }
        }
    };
}

// Mode7 scroll/center words are signed 13-bit values; displacements retain
// the original bit-13 sign selection and ten low bits.
fn ppu_mode7_coordinates(
    h_scroll: i16,
    v_scroll: i16,
    x_center: i16,
    y_center: i16,
) -> (i32, i32, i32, i32) {
    let signed13 = |word: i16| i32::from(word.wrapping_shl(3) >> 3);
    let h_scroll = signed13(h_scroll);
    let v_scroll = signed13(v_scroll);
    let x_center = signed13(x_center);
    let y_center = signed13(y_center);
    let clip = |delta: i32| {
        if delta & 0x2000 != 0 {
            delta | !1023
        } else {
            delta & 1023
        }
    };
    (
        x_center,
        y_center,
        clip(h_scroll - x_center),
        clip(v_scroll - y_center),
    )
}
// Sample the modern renderer's fixed-point Mode7 coordinates.
unsafe fn ppu_mode7_sample(
    ppu: *mut Ppu,
    x: u32,
    y: u32,
    outside_value: u32,
    char_fill: bool,
) -> u8 {
    let tile = if x | y > outside_value {
        if !char_fill {
            return 0;
        }
        0
    } else {
        (*ppu).vram[((y >> 11 & 0x7f) * 128 + (x >> 11 & 0x7f)) as usize] & 0xff
    };
    ((*ppu).vram[(u32::from(tile) * 64 + (y >> 8 & 7) * 8 + (x >> 8 & 7)) as usize] >> 8) as u8
}

unsafe fn PpuDrawWholeLine(ppu: *mut Ppu, y: u32) {
    if (*ppu).forcedBlank {
        let dst = (*ppu)
            .renderBuffer
            .offset(y.wrapping_sub(1).wrapping_mul((*ppu).renderPitch) as isize);
        memset(
            dst as *mut c_void,
            0,
            ((256 + i32::from((*ppu).extraLeftRight) * 2) * 4) as size_t,
        );
        return;
    }
    if (*ppu).mode == 1 {
        for layer in 0..3 {
            if (*ppu).mosaicEnabled & (1 << layer) != 0 {
                crate::native_assertion! { '_mosaic; b"0\0", b"src/snes/ppu.c\0", 679 + layer * 5, pointer __ASSERT_FUNCTION.as_ptr() };
            }
        }
    }
    let starts = if (*ppu).mode == 1 {
        (0, 0)
    } else {
        ppu_mode7_start::<true>(ppu, y as i32)
    };
    for x in -i32::from((*ppu).extraLeftCur)..256 + i32::from((*ppu).extraRightCur) {
        ppu_render_pixel::<true>(ppu, x, y as i32, starts);
    }
}

unsafe fn ppu_getPixel<const NATIVE: bool>(
    ppu: *mut Ppu,
    x: i32,
    y: i32,
    sub: bool,
    starts: (u32, u32),
) -> (i32, [i32; 3], bool) {
    if NATIVE {
        return ppu_native_pixel(ppu, x, y, sub, starts);
    }
    let (mut r, mut g, mut b) = (0, 0, 0);
    let mut actMode: i32 = if (*ppu).mode as i32 == 1 && (*ppu).bg3priority as i32 != 0 {
        8
    } else {
        (*ppu).mode as i32
    };
    actMode = if (*ppu).mode as i32 == 7 && (*ppu).m7extBg as i32 != 0 {
        9
    } else {
        actMode
    };
    let mut layer: i32 = 5;
    let mut pixel: i32 = 0;
    let mut i: i32 = 0;
    while i < layerCountPerMode[actMode as usize] {
        let curLayer: i32 = layersPerMode[actMode as usize][i as usize];
        let curPriority: i32 = prioritysPerMode[actMode as usize][i as usize];
        let mut layerActive: bool = false_0 != 0;
        if !sub {
            layerActive = (*ppu).layer[curLayer as usize].mainScreenEnabled as i32 != 0
                && (!(*ppu).layer[curLayer as usize].mainScreenWindowed
                    || !ppu_window_state::<false>(ppu, curLayer, x));
        } else {
            layerActive = (*ppu).layer[curLayer as usize].subScreenEnabled as i32 != 0
                && (!(*ppu).layer[curLayer as usize].subScreenWindowed
                    || !ppu_window_state::<false>(ppu, curLayer, x));
        }
        if layerActive {
            if curLayer < 4 {
                let mut lx: i32 = x;
                let mut ly: i32 = y;
                if (*ppu).bgLayer[curLayer as usize].mosaicEnabled as i32 != 0
                    && (*ppu).mosaicSize as i32 > 1
                {
                    lx -= lx % (*ppu).mosaicSize as i32;
                    ly -= (ly - (*ppu).mosaicStartLine as i32) % (*ppu).mosaicSize as i32;
                }
                if (*ppu).mode as i32 == 7 {
                    pixel = ppu_getPixelForMode7(ppu, lx, curLayer, curPriority != 0);
                } else {
                    lx += (*ppu).bgLayer[curLayer as usize].hScroll as i32;
                    if (*ppu).mode as i32 == 5 || (*ppu).mode as i32 == 6 {
                        lx *= 2;
                        lx += if sub as i32 != 0
                            || (*ppu).bgLayer[curLayer as usize].mosaicEnabled as i32 != 0
                        {
                            0
                        } else {
                            1
                        };
                        if (*ppu).interlace {
                            ly *= 2;
                            ly += if (*ppu).evenFrame as i32 != 0
                                || (*ppu).bgLayer[curLayer as usize].mosaicEnabled as i32 != 0
                            {
                                0
                            } else {
                                1
                            };
                        }
                    }
                    ly += (*ppu).bgLayer[curLayer as usize].vScroll as i32;
                    if (*ppu).mode as i32 == 2 || (*ppu).mode as i32 == 4 || (*ppu).mode as i32 == 6
                    {
                        ppu_handleOPT(ppu, curLayer, &raw mut lx, &raw mut ly);
                    }
                    pixel = ppu_getPixelForBgLayer::<false>(
                        ppu,
                        lx & 0x3ff,
                        ly & 0x3ff,
                        curLayer,
                        curPriority != 0,
                    );
                }
            } else {
                pixel = 0;
                if (*ppu).objBuffer.data[(x + kPpuExtraLeftRight as i32) as usize] as i32 >> 12
                    == curPriority * 4 + 2
                {
                    pixel = (*ppu).objBuffer.data[(x + kPpuExtraLeftRight as i32) as usize] as i32
                        & 0xff;
                }
            }
        }
        if pixel > 0 {
            layer = curLayer;
            break;
        } else {
            i += 1;
        }
    }
    if (*ppu).directColor as i32 != 0
        && layer < 4
        && bitDepthsPerMode[actMode as usize][layer as usize] == 8
    {
        r = (pixel & 0x7) << 2 | (pixel & 0x100) >> 7;
        g = (pixel & 0x38) >> 1 | (pixel & 0x200) >> 8;
        b = (pixel & 0xc0) >> 3 | (pixel & 0x400) >> 8;
    } else {
        let color: u16 = (*ppu).cgram[(pixel & 0xff) as usize];
        r = color as i32 & 0x1f;
        g = color as i32 >> 5 & 0x1f;
        b = color as i32 >> 10 & 0x1f;
    }
    if layer == 4 && pixel < 0xc0 {
        layer = 6;
    }
    (layer, [r, g, b], layer != 5)
}
unsafe fn ppu_handleOPT(ppu: *mut Ppu, layer: i32, lx: *mut i32, ly: *mut i32) {
    let x: i32 = *lx;
    let y: i32 = *ly;
    let mut column: i32 = if (*ppu).mode as i32 == 6 {
        x - (x & 0xf) - ((*ppu).bgLayer[layer as usize].hScroll as i32 * 2 & 0xfff0) >> 4
    } else {
        x - (x & 0x7) - ((*ppu).bgLayer[layer as usize].hScroll as i32 & 0xfff8) >> 3
    };
    if column > 0 {
        let valid: i32 = if layer == 0 { 0x2000 } else { 0x4000 };
        let mut hOffset: u16 = ppu_getOffsetValue(ppu, column - 1i32, 0);
        let mut vOffset: u16 = 0;
        if (*ppu).mode as i32 == 4 {
            if hOffset & 0x8000 != 0 {
                vOffset = hOffset;
                hOffset = 0;
            }
        } else {
            vOffset = ppu_getOffsetValue(ppu, column - 1i32, 1);
        }
        if (*ppu).mode as i32 == 6 {
            if hOffset as i32 & valid != 0 {
                *lx = ((hOffset as i32 & 0x3f8) + column * 8) * 2 | x & 0xf;
            }
        } else if hOffset as i32 & valid != 0 {
            *lx = (hOffset as i32 & 0x3f8) + column * 8 | x & 0x7;
        }
        if vOffset as i32 & valid != 0 {
            *ly = (vOffset as i32 & 0x3ff) + (y - (*ppu).bgLayer[layer as usize].vScroll as i32);
        }
    }
}
unsafe fn ppu_getOffsetValue(ppu: *mut Ppu, col: i32, row: i32) -> u16 {
    let x: i32 = col * 8 + (*ppu).bgLayer[2i32 as usize].hScroll as i32;
    let y: i32 = row * 8 + (*ppu).bgLayer[2i32 as usize].vScroll as i32;
    let tileBits: i32 = if (*ppu).bgLayer[2i32 as usize].bigTiles as i32 != 0 {
        4
    } else {
        3
    };
    let tileHighBit: i32 = if (*ppu).bgLayer[2i32 as usize].bigTiles as i32 != 0 {
        0x200
    } else {
        0x100
    };
    let mut tilemapAdr: u16 = ((*ppu).bgLayer[2i32 as usize].tilemapAdr as i32
        + ((y >> tileBits & 0x1f) << 5 | x >> tileBits & 0x1f))
        as u16;
    if x & tileHighBit != 0 && (*ppu).bgLayer[2i32 as usize].tilemapWider as i32 != 0 {
        tilemapAdr = (tilemapAdr as i32 + 0x400) as u16;
    }
    if y & tileHighBit != 0 && (*ppu).bgLayer[2i32 as usize].tilemapHigher as i32 != 0 {
        tilemapAdr = (tilemapAdr as i32
            + if (*ppu).bgLayer[2i32 as usize].tilemapWider as i32 != 0 {
                0x800
            } else {
                0x400
            }) as u16;
    }
    (*ppu).vram[(tilemapAdr as i32 & 0x7fff) as usize]
}
unsafe fn ppu_getPixelForBgLayer<const NATIVE: bool>(
    ppu: *mut Ppu,
    x: i32,
    y: i32,
    layer: i32,
    priority: bool,
) -> i32 {
    let bg = &(*ppu).bgLayer[layer as usize];
    let big = !NATIVE && bg.bigTiles;
    let wide = big || !NATIVE && ((*ppu).mode == 5 || (*ppu).mode == 6);
    let bits_x = if wide { 4 } else { 3 };
    let bits_y = if big { 4 } else { 3 };
    let mut address = i32::from(bg.tilemapAdr) + ((y >> bits_y & 31) << 5);
    if x & (32 << bits_x) != 0 && bg.tilemapWider {
        address += 0x400;
    }
    if y & (32 << bits_y) != 0 && bg.tilemapHigher {
        address += if bg.tilemapWider { 0x800 } else { 0x400 };
    }
    let column = x >> bits_x & 31;
    let tile = if NATIVE {
        *(*ppu)
            .vram
            .as_ptr()
            .offset(((address & 0x7fff) + column) as isize)
    } else {
        (*ppu).vram[((address + column) & 0x7fff) as usize]
    };
    if !NATIVE && (tile & 0x2000 != 0) != priority {
        return 0;
    }
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
    let mut tile_number = i32::from(tile & 0x3ff);
    if wide && (x & 8 != 0) != (tile & 0x4000 != 0) {
        tile_number += 1;
    }
    if big && (y & 8 != 0) != (tile & 0x8000 != 0) {
        tile_number += 16;
    }
    let depth = if NATIVE {
        if layer == 2 {
            2
        } else {
            4
        }
    } else {
        bitDepthsPerMode[(*ppu).mode as usize][layer as usize]
    };
    let pairs = if depth > 4 {
        4
    } else if depth > 2 {
        2
    } else {
        1
    };
    let address = (i32::from(bg.tileAdr) + (tile_number & 0x3ff) * 4 * depth + row) & 0x7fff;
    let mut pixel = 0;
    for pair in 0..pairs {
        let word = if NATIVE {
            *(*ppu).vram.as_ptr().offset((address + pair * 8) as isize)
        } else {
            (*ppu).vram[((address + pair * 8) & 0x7fff) as usize]
        };
        pixel |=
            ((i32::from(word) >> column & 1) | (i32::from(word) >> (column + 7) & 2)) << (pair * 2);
    }
    if pixel == 0 {
        return 0;
    }
    let palette = i32::from(tile & 0x1c00) >> 10;
    let palette = if !NATIVE && (*ppu).mode == 0 {
        palette + layer * 8
    } else {
        palette
    };
    let color = (1 << (pairs * 2)) * palette + pixel;
    if NATIVE {
        let priorities = [[0x8000, 0xc000], [0x7100, 0xb100], [0x1200, 0xf200]];
        priorities[layer as usize][usize::from(tile & 0x2000 != 0)] + color
    } else {
        color
    }
}
unsafe fn ppu_calculateMode7Starts(ppu: *mut Ppu, y: i32) {
    let (x, y) = ppu_mode7_start::<false>(ppu, y);
    (*ppu).m7startX = x as i32;
    (*ppu).m7startY = y as i32;
}
unsafe fn ppu_getPixelForMode7(ppu: *mut Ppu, x: i32, layer: i32, priority: bool) -> i32 {
    let rx = u32::from((if (*ppu).m7xFlip { 255 - x } else { x }) as u8);
    let pixel = ppu_mode7_sample(
        ppu,
        ((*ppu).m7startX as u32).wrapping_add(((*ppu).m7matrix[0] as u32).wrapping_mul(rx)),
        ((*ppu).m7startY as u32).wrapping_add(((*ppu).m7matrix[2] as u32).wrapping_mul(rx)),
        if (*ppu).m7largeField {
            0x3ffff
        } else {
            u32::MAX
        },
        (*ppu).m7charFill,
    );
    if layer == 1 {
        if (pixel & 0x80 != 0) != priority {
            return 0;
        }
        i32::from(pixel & 0x7f)
    } else {
        i32::from(pixel)
    }
}

unsafe fn ppu_evaluateSprites(ppu: *mut Ppu, line: i32) -> bool {
    let mut index: u8 = (if (*ppu).objPriority as i32 != 0 {
        (*ppu).oamAdr as i32 & 0xfe
    } else {
        0
    }) as u8;
    let mut spritesFound: i32 = 0;
    let mut tilesFound: i32 = 0;
    let mut i: i32 = 0;
    while i < 128 {
        let y: u8 = ((*ppu).oam[index as usize] >> 8) as u8;
        let mut row: u8 = (line - y as i32) as u8;
        let spriteSize: i32 = spriteSizes[(*ppu).objSize as usize]
            [((*ppu).highOam[(index >> 3) as usize] as i32 >> (index as i32 & 7) + 1 & 1) as usize];
        let spriteHeight: i32 = if (*ppu).objInterlace as i32 != 0 {
            spriteSize / 2
        } else {
            spriteSize
        };
        if (row as i32) < spriteHeight {
            let mut x: i32 = (*ppu).oam[index as usize] as i32 & 0xff;
            x |= ((*ppu).highOam[(index >> 3) as usize] as i32 >> (index as i32 & 7) & 1) << 8;
            if x > 255 {
                x -= 512;
            }
            if x > -spriteSize {
                spritesFound += 1;
                if spritesFound > 32 {
                    (*ppu).rangeOver = true_0 != 0;
                    break;
                } else {
                    if (*ppu).objInterlace {
                        row = (row as i32 * 2 + (if (*ppu).evenFrame as i32 != 0 { 0 } else { 1 }))
                            as u8;
                    }
                    let oam1: i32 = (*ppu).oam[(index as i32 + 1) as usize] as i32;
                    let tile: i32 = oam1 & 0xff;
                    let objAdr: i32 = if oam1 & 0x100 != 0 {
                        (*ppu).objTileAdr2 as i32
                    } else {
                        (*ppu).objTileAdr1 as i32
                    };
                    let palette: i32 = (oam1 & 0xe00) >> 9;
                    let hFlipped: bool = oam1 & 0x4000 != 0;
                    if oam1 & 0x8000 != 0 {
                        row = (spriteSize - 1i32 - row as i32) as u8;
                    }
                    let paletteBase: i32 = 0x80 + 16 * ((oam1 & 0xe00) >> 9);
                    let prio: i32 = (((oam1 & 0x3000) >> 12) * 4 + 2) * 16
                        + 4
                        + (if oam1 & 0x800 == 0 { 2 } else { 0 });
                    let z: PpuZbufType = (paletteBase + (prio << 8)) as PpuZbufType;
                    let mut col: i32 = 0;
                    while col < spriteSize {
                        if col + x > -8i32 && col + x < 256 {
                            tilesFound += 1;
                            if tilesFound > 34 {
                                (*ppu).timeOver = true_0 != 0;
                                break;
                            } else {
                                let usedCol: i32 = if oam1 & 0x4000 != 0 {
                                    spriteSize - 1i32 - col
                                } else {
                                    col
                                };
                                let usedTile: i32 = ((oam1 & 0xff) >> 4) + (row as i32 >> 3) << 4
                                    | (oam1 & 0xf) + (usedCol >> 3) & 0xf;
                                let addr = (&raw mut (*ppu).vram as *mut u16).offset(
                                    (objAdr + usedTile * 16 + (row as i32 & 0x7) & 0x7fff) as isize,
                                );
                                let plane: u32 =
                                    (*addr as i32 | (*addr.offset(8) as i32) << 16) as u32;
                                let px_left: i32 =
                                    IntMax(-(col + x + kPpuExtraLeftRight as i32), 0);
                                let px_right: i32 =
                                    IntMin(256 + kPpuExtraLeftRight as i32 - (col + x), 8);
                                let dst = (&raw mut (*ppu).objBuffer.data as *mut PpuZbufType)
                                    .offset(col as isize)
                                    .offset(x as isize)
                                    .offset(px_left as isize)
                                    .offset(kPpuExtraLeftRight as i32 as isize);
                                if px_left < px_right {
                                    draw_sprite_tile_span(
                                        core::slice::from_raw_parts_mut(
                                            dst,
                                            (px_right - px_left) as usize,
                                        ),
                                        plane,
                                        z,
                                        px_left as u32,
                                        hFlipped,
                                    );
                                }
                            }
                        }
                        col += 8;
                    }
                    if tilesFound > 34 {
                        break;
                    }
                }
            }
        }
        index = (index as i32 + 2) as u8;
        i += 1;
    }
    tilesFound != 0
}
unsafe fn ppu_getVramRemap(ppu: *mut Ppu) -> u16 {
    let adr: u16 = (*ppu).vramPointer;
    match (*ppu).vramRemapMode as i32 {
        0 => return adr,
        1 => {
            return (adr as i32 & 0xff00 | (adr as i32 & 0xe0) >> 5 | (adr as i32 & 0x1f) << 3)
                as u16
        }
        2 => {
            return (adr as i32 & 0xfe00 | (adr as i32 & 0x1c0) >> 6 | (adr as i32 & 0x3f) << 3)
                as u16
        }
        3 => {
            return (adr as i32 & 0xfc00 | (adr as i32 & 0x380) >> 7 | (adr as i32 & 0x7f) << 3)
                as u16
        }
        _ => {}
    }
    adr
}
pub unsafe fn ppu_read(ppu: *mut Ppu, adr: u8) -> u8 {
    match adr as i32 {
        4 | 20 | 36 | 5 | 21 | 37 | 6 | 22 | 38 | 8 | 24 | 40 | 9 | 25 | 41 | 10 | 26 | 42 => {
            (*ppu).ppu1openBus
        }
        52 | 53 | 54 => {
            let result: i32 = (*ppu).m7matrix[0i32 as usize] as i32
                * ((*ppu).m7matrix[1i32 as usize] as i32 >> 8);
            (*ppu).ppu1openBus = (result >> 8 * (adr as i32 - 0x34i32) & 0xff) as u8;
            (*ppu).ppu1openBus
        }
        55 => {
            (*ppu).hCount = ((*(*ppu).snes).hPos as i32 / 4) as u16;
            (*ppu).vCount = (*(*ppu).snes).vPos;
            (*ppu).countersLatched = true_0 != 0;
            (*(*ppu).snes).openBus
        }
        56 => {
            let mut ret: u8 = 0;
            if (*ppu).oamInHigh {
                ret = (*ppu).highOam
                    [(((*ppu).oamAdr as i32 & 0xf) << 1 | (*ppu).oamSecondWrite as i32) as usize];
                if (*ppu).oamSecondWrite {
                    (*ppu).oamAdr = (*ppu).oamAdr.wrapping_add(1);
                    if (*ppu).oamAdr as i32 == 0 {
                        (*ppu).oamInHigh = false_0 != 0;
                    }
                }
            } else if !(*ppu).oamSecondWrite {
                ret = ((*ppu).oam[(*ppu).oamAdr as usize] as u8 & 0xff);
            } else {
                crate::update_place! { previous fresh2 = (*ppu).oamAdr => (*ppu).oamAdr.wrapping_add(1); }
                ret = ((*ppu).oam[fresh2 as usize] >> 8) as u8;
                if (*ppu).oamAdr as i32 == 0 {
                    (*ppu).oamInHigh = true_0 != 0;
                }
            }
            (*ppu).oamSecondWrite = !(*ppu).oamSecondWrite;
            (*ppu).ppu1openBus = ret;
            ret
        }
        57 => {
            let val: u16 = (*ppu).vramReadBuffer;
            if !(*ppu).vramIncrementOnHigh {
                (*ppu).vramReadBuffer =
                    (*ppu).vram[(ppu_getVramRemap(ppu) as i32 & 0x7fff) as usize];
                (*ppu).vramPointer =
                    ((*ppu).vramPointer as i32 + (*ppu).vramIncrement as i32) as u16;
            }
            (*ppu).ppu1openBus = (val as u8 & 0xff);
            (val as u8 & 0xff)
        }
        58 => {
            let val_0: u16 = (*ppu).vramReadBuffer;
            if (*ppu).vramIncrementOnHigh {
                (*ppu).vramReadBuffer =
                    (*ppu).vram[(ppu_getVramRemap(ppu) as i32 & 0x7fff) as usize];
                (*ppu).vramPointer =
                    ((*ppu).vramPointer as i32 + (*ppu).vramIncrement as i32) as u16;
            }
            (*ppu).ppu1openBus = (val_0 >> 8) as u8;
            (val_0 >> 8) as u8
        }
        59 => {
            let mut ret_0: u8 = if !(*ppu).cgramSecondWrite {
                ((*ppu).cgram[(*ppu).cgramPointer as usize] as u8 & 0xff)
            } else {
                crate::update_place! { previous fresh3 = (*ppu).cgramPointer => (*ppu).cgramPointer.wrapping_add(1); }
                ((*ppu).cgram[fresh3 as usize] as i32 >> 8 & 0x7f
                    | (*ppu).ppu2openBus as i32 & 0x80) as u8
            };
            (*ppu).cgramSecondWrite = !(*ppu).cgramSecondWrite;
            (*ppu).ppu2openBus = ret_0;
            ret_0
        }
        60 => {
            let mut val_1: u8 = if (*ppu).hCountSecond {
                ((*ppu).hCount as i32 >> 8 & 1 | (*ppu).ppu2openBus as i32 & 0xfe) as u8
            } else {
                ((*ppu).hCount as u8 & 0xff)
            };
            (*ppu).hCountSecond = !(*ppu).hCountSecond;
            (*ppu).ppu2openBus = val_1;
            val_1
        }
        61 => {
            let mut val_2: u8 = if (*ppu).vCountSecond {
                ((*ppu).vCount as i32 >> 8 & 1 | (*ppu).ppu2openBus as i32 & 0xfe) as u8
            } else {
                ((*ppu).vCount as u8 & 0xff)
            };
            (*ppu).vCountSecond = !(*ppu).vCountSecond;
            (*ppu).ppu2openBus = val_2;
            val_2
        }
        62 => {
            let mut val_3: u8 = 0x1;
            val_3 = (val_3 | (*ppu).ppu1openBus & 0x10);
            val_3 = (val_3 as i32 | ((*ppu).rangeOver as i32) << 6) as u8;
            val_3 = (val_3 as i32 | ((*ppu).timeOver as i32) << 7) as u8;
            (*ppu).ppu1openBus = val_3;
            val_3
        }
        63 => {
            let mut val_4: u8 = 0x3;
            val_4 = (val_4 | (*ppu).ppu2openBus & 0x20);
            val_4 = (val_4 as i32 | ((*ppu).countersLatched as i32) << 6) as u8;
            val_4 = (val_4 as i32 | ((*ppu).evenFrame as i32) << 7) as u8;
            set_fields! {[(*ppu)]; countersLatched = false_0 != 0, hCountSecond = false_0 != 0,
            vCountSecond = false_0 != 0, ppu2openBus = val_4, };
            val_4
        }
        _ => (*(*ppu).snes).openBus,
    }
}
pub unsafe fn ppu_write(ppu: *mut Ppu, adr: u8, val: u8) {
    let current_block_164: u64;
    match adr as i32 {
        0 => {
            (*ppu).brightness = (val & 0xf);
            (*ppu).forcedBlank = val & 0x80 != 0;
            current_block_164 = 16754622181974910496;
        }
        1 => {
            (*ppu).objSize = (val >> 5);
            (*ppu).objTileAdr1 = ((val as i32 & 7) << 13) as u16;
            (*ppu).objTileAdr2 =
                ((*ppu).objTileAdr1 as i32 + (((val as i32 & 0x18) + 8) << 9)) as u16;
            current_block_164 = 16754622181974910496;
        }
        2 => {
            set_fields! {[(*ppu)]; oamAdr = val, oamAdrWritten = (*ppu).oamAdr,
            oamInHigh = (*ppu).oamInHighWritten, oamSecondWrite = false_0 != 0, };
            current_block_164 = 16754622181974910496;
        }
        3 => {
            set_fields! {[(*ppu)]; objPriority = val & 0x80 != 0, oamInHigh = val & 1 != 0,
            oamInHighWritten = (*ppu).oamInHigh, oamAdr = (*ppu).oamAdrWritten,
            oamSecondWrite = false_0 != 0, };
            current_block_164 = 16754622181974910496;
        }
        4 => {
            if (*ppu).oamInHigh {
                (*ppu).highOam
                    [(((*ppu).oamAdr as i32 & 0xf) << 1 | (*ppu).oamSecondWrite as i32) as usize] =
                    val;
                if (*ppu).oamSecondWrite {
                    (*ppu).oamAdr = (*ppu).oamAdr.wrapping_add(1);
                    if (*ppu).oamAdr as i32 == 0 {
                        (*ppu).oamInHigh = false_0 != 0;
                    }
                }
            } else if !(*ppu).oamSecondWrite {
                (*ppu).oamBuffer = val;
            } else {
                crate::update_place! { previous fresh4 = (*ppu).oamAdr => (*ppu).oamAdr.wrapping_add(1); }
                (*ppu).oam[fresh4 as usize] = ((val as i32) << 8 | (*ppu).oamBuffer as i32) as u16;
                if (*ppu).oamAdr as i32 == 0 {
                    (*ppu).oamInHigh = true_0 != 0;
                }
            }
            (*ppu).oamSecondWrite = !(*ppu).oamSecondWrite;
            current_block_164 = 16754622181974910496;
        }
        5 => {
            (*ppu).mode = (val & 0x7);
            (*ppu).bg3priority = val & 0x8 != 0;
            (*ppu).bgLayer[0i32 as usize].bigTiles = val & 0x10 != 0;
            (*ppu).bgLayer[1i32 as usize].bigTiles = val & 0x20 != 0;
            (*ppu).bgLayer[2i32 as usize].bigTiles = val & 0x40 != 0;
            (*ppu).bgLayer[3i32 as usize].bigTiles = val & 0x80 != 0;
            current_block_164 = 16754622181974910496;
        }
        6 => {
            (*ppu).bgLayer[0i32 as usize].mosaicEnabled = val & 0x1 != 0;
            (*ppu).bgLayer[1i32 as usize].mosaicEnabled = val & 0x2 != 0;
            (*ppu).bgLayer[2i32 as usize].mosaicEnabled = val & 0x4 != 0;
            (*ppu).bgLayer[3i32 as usize].mosaicEnabled = val & 0x8 != 0;
            (*ppu).mosaicSize = ((val as i32 >> 4) + 1) as u8;
            (*ppu).mosaicStartLine = 0;
            current_block_164 = 16754622181974910496;
        }
        7 | 8 | 9 | 10 => {
            let bg = &mut (*ppu).bgLayer[usize::from(adr - 7)];
            bg.tilemapWider = val & 1 != 0;
            bg.tilemapHigher = val & 2 != 0;
            bg.tilemapAdr = u16::from(val & 0xfc) << 8;
            current_block_164 = 16754622181974910496;
        }
        11 | 12 => {
            let first_bg = usize::from(adr - 11) * 2;
            (*ppu).bgLayer[first_bg].tileAdr = u16::from(val & 0x0f) << 12;
            (*ppu).bgLayer[first_bg + 1].tileAdr = u16::from(val & 0xf0) << 8;
            current_block_164 = 16754622181974910496;
        }
        13 => {
            (*ppu).m7matrix[6] = ((u16::from(val) << 8 | u16::from((*ppu).m7prev)) & 0x1fff) as i16;
            (*ppu).m7prev = val;
            current_block_164 = 6174974146017752131;
        }
        15 | 17 | 19 => current_block_164 = 6174974146017752131,
        14 => {
            (*ppu).m7matrix[7] = ((u16::from(val) << 8 | u16::from((*ppu).m7prev)) & 0x1fff) as i16;
            (*ppu).m7prev = val;
            current_block_164 = 5892776923941496671;
        }
        16 | 18 | 20 => current_block_164 = 5892776923941496671,
        21 => {
            if val & 3 == 0 {
                (*ppu).vramIncrement = 1;
            } else if val & 3 == 1 {
                (*ppu).vramIncrement = 32;
            } else {
                (*ppu).vramIncrement = 128;
            }
            (*ppu).vramRemapMode = ((val as i32 & 0xc) >> 2) as u8;
            (*ppu).vramIncrementOnHigh = val & 0x80 != 0;
            current_block_164 = 16754622181974910496;
        }
        22 => {
            (*ppu).vramPointer = ((*ppu).vramPointer & 0xff00 | val as u16);
            (*ppu).vramReadBuffer = (*ppu).vram[(ppu_getVramRemap(ppu) as i32 & 0x7fff) as usize];
            current_block_164 = 16754622181974910496;
        }
        23 => {
            (*ppu).vramPointer = ((*ppu).vramPointer as i32 & 0xff | (val as i32) << 8) as u16;
            (*ppu).vramReadBuffer = (*ppu).vram[(ppu_getVramRemap(ppu) as i32 & 0x7fff) as usize];
            current_block_164 = 16754622181974910496;
        }
        24 => {
            let vramAdr: u16 = ppu_getVramRemap(ppu);
            (*ppu).vram[(vramAdr as i32 & 0x7fff) as usize] =
                ((*ppu).vram[(vramAdr as i32 & 0x7fff) as usize] & 0xff00 | val as u16);
            if !(*ppu).vramIncrementOnHigh {
                (*ppu).vramPointer =
                    ((*ppu).vramPointer as i32 + (*ppu).vramIncrement as i32) as u16;
            }
            current_block_164 = 16754622181974910496;
        }
        25 => {
            let vramAdr_0: u16 = ppu_getVramRemap(ppu);
            (*ppu).vram[(vramAdr_0 as i32 & 0x7fff) as usize] =
                ((*ppu).vram[(vramAdr_0 as i32 & 0x7fff) as usize] as i32 & 0xff
                    | (val as i32) << 8) as u16;
            if (*ppu).vramIncrementOnHigh {
                (*ppu).vramPointer =
                    ((*ppu).vramPointer as i32 + (*ppu).vramIncrement as i32) as u16;
            }
            current_block_164 = 16754622181974910496;
        }
        26 => {
            set_fields! {[(*ppu)]; m7largeField = val & 0x80 != 0, m7charFill = val & 0x40 != 0,
            m7yFlip = val & 0x02 != 0, m7xFlip = val & 0x01 != 0, };
            current_block_164 = 16754622181974910496;
        }
        27 | 28 | 29 | 30 => {
            (*ppu).m7matrix[usize::from(adr - 0x1b)] =
                (u16::from(val) << 8 | u16::from((*ppu).m7prev)) as i16;
            (*ppu).m7prev = val;
            current_block_164 = 16754622181974910496;
        }
        31 | 32 => {
            (*ppu).m7matrix[usize::from(adr - 0x1b)] =
                ((u16::from(val) << 8 | u16::from((*ppu).m7prev)) & 0x1fff) as i16;
            (*ppu).m7prev = val;
            current_block_164 = 16754622181974910496;
        }
        33 => {
            (*ppu).cgramPointer = val;
            (*ppu).cgramSecondWrite = false_0 != 0;
            current_block_164 = 16754622181974910496;
        }
        34 => {
            if !(*ppu).cgramSecondWrite {
                (*ppu).cgramBuffer = val;
            } else {
                crate::update_place! { previous fresh5 = (*ppu).cgramPointer => (*ppu).cgramPointer.wrapping_add(1); }
                (*ppu).cgram[fresh5 as usize] =
                    ((val as i32) << 8 | (*ppu).cgramBuffer as i32) as u16;
            }
            (*ppu).cgramSecondWrite = !(*ppu).cgramSecondWrite;
            current_block_164 = 16754622181974910496;
        }
        35 | 36 | 37 => {
            if adr as i32 == 0x23 {
                (*ppu).windowsel = (*ppu).windowsel & !(0xffi32) as u32 | val as u32;
            } else if adr as i32 == 0x24 {
                (*ppu).windowsel =
                    (*ppu).windowsel & !(0xff00i32) as u32 | ((val as i32) << 8) as u32;
            } else if adr as i32 == 0x25 {
                (*ppu).windowsel =
                    (*ppu).windowsel & !(0xff0000i32) as u32 | ((val as i32) << 16) as u32;
            }
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32) as usize].window1inversed =
                val & 0x1 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32) as usize].window1enabled =
                val & 0x2 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32) as usize].window2inversed =
                val & 0x4 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32) as usize].window2enabled =
                val & 0x8 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32 + 1i32) as usize].window1inversed =
                val & 0x10 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32 + 1i32) as usize].window1enabled =
                val & 0x20 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32 + 1i32) as usize].window2inversed =
                val & 0x40 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32 + 1i32) as usize].window2enabled =
                val & 0x80 != 0;
            current_block_164 = 16754622181974910496;
        }
        38 => {
            (*ppu).window1left = val;
            current_block_164 = 16754622181974910496;
        }
        39 => {
            (*ppu).window1right = val;
            current_block_164 = 16754622181974910496;
        }
        40 => {
            (*ppu).window2left = val;
            current_block_164 = 16754622181974910496;
        }
        41 => {
            (*ppu).window2right = val;
            current_block_164 = 16754622181974910496;
        }
        42 => {
            (*ppu).windowLayer[0i32 as usize].maskLogic = (val & 0x3);
            (*ppu).windowLayer[1i32 as usize].maskLogic = (val as i32 >> 2 & 0x3) as u8;
            (*ppu).windowLayer[2i32 as usize].maskLogic = (val as i32 >> 4 & 0x3) as u8;
            (*ppu).windowLayer[3i32 as usize].maskLogic = (val as i32 >> 6 & 0x3) as u8;
            current_block_164 = 16754622181974910496;
        }
        43 => {
            (*ppu).windowLayer[4i32 as usize].maskLogic = (val & 0x3);
            (*ppu).windowLayer[5i32 as usize].maskLogic = (val as i32 >> 2 & 0x3) as u8;
            current_block_164 = 16754622181974910496;
        }
        44 => {
            (*ppu).screenEnabled[0] = val;
            (*ppu).layer[0].mainScreenEnabled = val & 0x01 != 0;
            (*ppu).layer[1].mainScreenEnabled = val & 0x02 != 0;
            (*ppu).layer[2].mainScreenEnabled = val & 0x04 != 0;
            (*ppu).layer[3].mainScreenEnabled = val & 0x08 != 0;
            (*ppu).layer[4].mainScreenEnabled = val & 0x10 != 0;
            current_block_164 = 16754622181974910496;
        }
        45 => {
            (*ppu).screenEnabled[1] = val;
            (*ppu).layer[0].subScreenEnabled = val & 0x01 != 0;
            (*ppu).layer[1].subScreenEnabled = val & 0x02 != 0;
            (*ppu).layer[2].subScreenEnabled = val & 0x04 != 0;
            (*ppu).layer[3].subScreenEnabled = val & 0x08 != 0;
            (*ppu).layer[4].subScreenEnabled = val & 0x10 != 0;
            current_block_164 = 16754622181974910496;
        }
        46 => {
            (*ppu).screenWindowed[0] = val;
            (*ppu).layer[0].mainScreenWindowed = val & 0x01 != 0;
            (*ppu).layer[1].mainScreenWindowed = val & 0x02 != 0;
            (*ppu).layer[2].mainScreenWindowed = val & 0x04 != 0;
            (*ppu).layer[3].mainScreenWindowed = val & 0x08 != 0;
            (*ppu).layer[4].mainScreenWindowed = val & 0x10 != 0;
            current_block_164 = 16754622181974910496;
        }
        47 => {
            (*ppu).screenWindowed[1] = val;
            (*ppu).layer[0].subScreenWindowed = val & 0x01 != 0;
            (*ppu).layer[1].subScreenWindowed = val & 0x02 != 0;
            (*ppu).layer[2].subScreenWindowed = val & 0x04 != 0;
            (*ppu).layer[3].subScreenWindowed = val & 0x08 != 0;
            (*ppu).layer[4].subScreenWindowed = val & 0x10 != 0;
            current_block_164 = 16754622181974910496;
        }
        48 => {
            set_fields! {[(*ppu)]; directColor = val & 0x01 != 0, addSubscreen = val & 0x02 != 0,
            preventMathMode = (val & 0x30) >> 4, clipMode = (val & 0xc0) >> 6, };
            current_block_164 = 16754622181974910496;
        }
        49 => {
            (*ppu).subtractColor = val & 0x80 != 0;
            (*ppu).halfColor = val & 0x40 != 0;
            for layer in 0..6 {
                (*ppu).mathEnabled[layer] = val & (1 << layer) != 0;
            }
            current_block_164 = 16754622181974910496;
        }
        50 => {
            if val & 0x80 != 0 {
                (*ppu).fixedColorB = val & 0x1f;
            }
            if val & 0x40 != 0 {
                (*ppu).fixedColorG = val & 0x1f;
            }
            if val & 0x20 != 0 {
                (*ppu).fixedColorR = val & 0x1f;
            }
            current_block_164 = 16754622181974910496;
        }
        51 => {
            set_fields! {[(*ppu)]; interlace = val & 0x01 != 0, objInterlace = val & 0x02 != 0,
            overscan = val & 0x04 != 0, pseudoHires = val & 0x08 != 0,
            m7extBg = val & 0x40 != 0, };
            current_block_164 = 16754622181974910496;
        }
        _ => current_block_164 = 16754622181974910496,
    }
    match current_block_164 {
        5892776923941496671 => {
            let layer = (usize::from(adr) - 0x0e) / 2;
            (*ppu).bgLayer[layer].vScroll =
                ((u16::from(val) << 8) | u16::from((*ppu).scrollPrev)) & 0x3ff;
            (*ppu).scrollPrev = val;
        }
        6174974146017752131 => {
            let layer = (usize::from(adr) - 0x0d) / 2;
            (*ppu).bgLayer[layer].hScroll = ((u16::from(val) << 8)
                | u16::from((*ppu).scrollPrev & 0xf8)
                | u16::from((*ppu).scrollPrev2 & 0x07))
                & 0x3ff;
            (*ppu).scrollPrev = val;
            (*ppu).scrollPrev2 = val;
        }
        _ => {}
    };
}
pub fn PpuGetCurrentRenderScale(ppu: *mut Ppu, render_flags: u32) -> i32 {
    1
}

const SPRITE_BIT_SPREAD: [u32; 256] = {
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

fn draw_sprite_tile_span(dst: &mut [u16], plane: u32, z: u16, px_left: u32, hflip: bool) {
    let mut pixels = SPRITE_BIT_SPREAD[(plane & 0xff) as usize]
        | (SPRITE_BIT_SPREAD[((plane >> 8) & 0xff) as usize] << 1)
        | (SPRITE_BIT_SPREAD[((plane >> 16) & 0xff) as usize] << 2)
        | (SPRITE_BIT_SPREAD[(plane >> 24) as usize] << 3);
    if hflip {
        pixels >>= px_left * 4;
        for dst in dst {
            let pixel = (pixels & 0xf) as u16;
            if pixel != 0 && *dst & 0xff == 0 {
                *dst = z.wrapping_add(pixel);
            }
            pixels >>= 4;
        }
    } else {
        pixels <<= px_left * 4;
        for dst in dst {
            let pixel = (pixels >> 28) as u16;
            if pixel != 0 && *dst & 0xff == 0 {
                *dst = z.wrapping_add(pixel);
            }
            pixels <<= 4;
        }
    }
}

unsafe fn ppu_window_state<const NATIVE: bool>(ppu: *mut Ppu, layer: i32, x: i32) -> bool {
    let window = &(*ppu).windowLayer[layer as usize];
    let flags = if NATIVE {
        (*ppu).windowsel >> (layer * 4)
    } else {
        u32::from(window.window1inversed)
            | u32::from(window.window1enabled) << 1
            | u32::from(window.window2inversed) << 2
            | u32::from(window.window2enabled) << 3
    };
    let first = (x >= i32::from((*ppu).window1left) && x <= i32::from((*ppu).window1right))
        ^ (flags & 1 != 0);
    let second = (x >= i32::from((*ppu).window2left) && x <= i32::from((*ppu).window2right))
        ^ (flags & 4 != 0);
    match flags & 10 {
        2 => first,
        8 => second,
        10 => match if NATIVE { 0 } else { window.maskLogic } {
            0 => first || second,
            1 => first && second,
            2 => first != second,
            3 => first == second,
            _ => false,
        },
        _ => false,
    }
}

unsafe fn ppu_native_layer_enabled(ppu: *mut Ppu, layer: i32, x: i32, sub: bool) -> bool {
    (*ppu).screenEnabled[sub as usize] & (1 << layer) != 0
        && (layer != 2 || (0..256).contains(&x))
        && ((*ppu).screenWindowed[sub as usize] & (1 << layer) == 0
            || !ppu_window_state::<true>(ppu, layer, x))
}

unsafe fn ppu_mode7_start<const NATIVE: bool>(ppu: *mut Ppu, mut y: i32) -> (u32, u32) {
    let (cx, cy, horizontal, vertical) = ppu_mode7_coordinates(
        (*ppu).m7matrix[6],
        (*ppu).m7matrix[7],
        (*ppu).m7matrix[4],
        (*ppu).m7matrix[5],
    );
    if NATIVE {
        if (*ppu).mosaicEnabled & 1 != 0 {
            y = i32::from((*ppu).mosaicModulo[y as usize]);
        }
    } else if (*ppu).bgLayer[0].mosaicEnabled && (*ppu).mosaicSize > 1 {
        y -= (y - i32::from((*ppu).mosaicStartLine)) % i32::from((*ppu).mosaicSize);
    }
    let ry = if (*ppu).m7yFlip { 255 - y } else { y };
    let ry = if NATIVE {
        ry as u32
    } else {
        u32::from(ry as u8)
    };
    let start = |a: i16, b: i16, center: i32| {
        ((i32::from(a) * horizontal & !63) as u32)
            .wrapping_add((b as u32).wrapping_mul(ry) & !63)
            .wrapping_add((i32::from(b) * vertical & !63) as u32)
            .wrapping_add((center << 8) as u32)
    };
    (
        start((*ppu).m7matrix[0], (*ppu).m7matrix[1], cx),
        start((*ppu).m7matrix[2], (*ppu).m7matrix[3], cy),
    )
}

unsafe fn ppu_native_mode7_x(ppu: *mut Ppu, x: i32, sub: bool) -> i32 {
    if (*ppu).mosaicEnabled & 1 == 0 {
        return x;
    }
    let mut left = -i32::from((*ppu).extraLeftCur);
    if (*ppu).screenWindowed[sub as usize] & 1 != 0 {
        for (enabled, first, last) in [
            (2, (*ppu).window1left, (*ppu).window1right),
            (8, (*ppu).window2left, (*ppu).window2right),
        ] {
            if (*ppu).windowsel & enabled != 0 && first <= last {
                for edge in [i32::from(first), i32::from(last) + 1] {
                    if edge <= x {
                        left = left.max(edge);
                    }
                }
            }
        }
    }
    let width = i32::from((*ppu).mosaicSize);
    let first_end = left + width - (left - i32::from((*ppu).mosaicModulo[left as usize]));
    if x < first_end {
        left
    } else {
        first_end + (x - first_end) / width * width
    }
}

unsafe fn ppu_native_pixel(
    ppu: *mut Ppu,
    x: i32,
    y: i32,
    sub: bool,
    starts: (u32, u32),
) -> (i32, [i32; 3], bool) {
    let mut packed = 0x0500;
    let sprite = if (*ppu).lineHasSprites && ppu_native_layer_enabled(ppu, 4, x, sub) {
        i32::from(
            *(*ppu)
                .objBuffer
                .data
                .as_ptr()
                .offset((x + kPpuExtraLeftRight as i32) as isize),
        )
    } else {
        0x0500
    };
    if (*ppu).mode == 1 {
        packed = sprite;
        for layer in 0..3 {
            if ppu_native_layer_enabled(ppu, layer, x, sub) {
                let bg = &(*ppu).bgLayer[layer as usize];
                let pixel = ppu_getPixelForBgLayer::<true>(
                    ppu,
                    x + i32::from(bg.hScroll),
                    y + i32::from(bg.vScroll),
                    layer,
                    false,
                );
                let color_mask = if layer == 2 { 3 } else { 15 };
                if pixel != 0 && pixel & !color_mask > packed {
                    packed = pixel;
                }
            }
        }
    } else {
        if ppu_native_layer_enabled(ppu, 0, x, sub) {
            let mx = ppu_native_mode7_x(ppu, x, sub);
            let rx = if (*ppu).m7xFlip { 255 - mx } else { mx } as u32;
            let pixel = ppu_mode7_sample(
                ppu,
                starts
                    .0
                    .wrapping_add(((*ppu).m7matrix[0] as u32).wrapping_mul(rx)),
                starts
                    .1
                    .wrapping_add(((*ppu).m7matrix[2] as u32).wrapping_mul(rx)),
                if (*ppu).m7largeField {
                    0x3ffff
                } else {
                    u32::MAX
                },
                (*ppu).m7charFill,
            );
            if pixel != 0 {
                packed = 0x5000 + i32::from(pixel);
            }
        }
        packed = packed.max(sprite);
    }
    let index = packed & 255;
    let color = (*ppu).cgram[index as usize];
    (
        (packed >> 8) & 15,
        [
            i32::from(color & 31),
            i32::from((color >> 5) & 31),
            i32::from((color >> 10) & 31),
        ],
        index != 0,
    )
}

unsafe fn ppu_render_pixel<const NATIVE: bool>(ppu: *mut Ppu, x: i32, y: i32, starts: (u32, u32)) {
    let mut output = [0u8; 4];
    if !(*ppu).forcedBlank {
        let (layer, mut color, _) = ppu_getPixel::<NATIVE>(ppu, x, y, false, starts);
        let window = ppu_window_state::<NATIVE>(ppu, 5, x);
        let permits = |mode: u8| !(mode == 3 || mode == 2 && window || mode == 1 && !window);
        if !permits((*ppu).clipMode) {
            color = [0; 3];
        }
        let math =
            layer < 6 && (*ppu).mathEnabled[layer as usize] && permits((*ppu).preventMathMode);
        let mut second = (5, [0; 3], false);
        if math && (*ppu).addSubscreen
            || !NATIVE && ((*ppu).pseudoHires || (*ppu).mode == 5 || (*ppu).mode == 6)
        {
            second = ppu_getPixel::<NATIVE>(ppu, x, y, true, starts);
        }
        let subscreen = (*ppu).addSubscreen && second.2;
        let half = math && (*ppu).halfColor && (second.2 || !(*ppu).addSubscreen);
        let fixed = [(*ppu).fixedColorR, (*ppu).fixedColorG, (*ppu).fixedColorB];
        let fixed_word = u32::from(fixed[0]) | u32::from(fixed[1]) << 5 | u32::from(fixed[2]) << 10;
        for channel in 0..3 {
            let mut value = color[channel];
            if math {
                let operand = if subscreen {
                    second.1[channel]
                } else if NATIVE {
                    ((fixed_word >> (channel * 5)) & 31) as i32
                } else {
                    i32::from(fixed[channel])
                };
                value += if (*ppu).subtractColor {
                    -operand
                } else {
                    operand
                };
            }
            output[2 - channel] = if NATIVE {
                let value = value.max(0) as usize;
                if half {
                    (*ppu).brightnessMultHalf[value]
                } else {
                    (*ppu).brightnessMult[value]
                }
            } else {
                if half {
                    value >>= 1;
                }
                if math {
                    value = value.clamp(0, 31);
                }
                ((value << 3 | value >> 2) * i32::from((*ppu).brightness) / 15) as u8
            };
        }
    }
    let offset = ((y - 1) as u32)
        .wrapping_mul((*ppu).renderPitch)
        .wrapping_add(((x + i32::from((*ppu).extraLeftRight)) * 4) as u32);
    core::ptr::copy_nonoverlapping(
        output.as_ptr(),
        (*ppu).renderBuffer.offset(offset as isize),
        4,
    );
}
