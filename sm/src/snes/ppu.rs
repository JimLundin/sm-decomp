use crate::game_state::Game;
use ::core::ffi::{c_char, c_long, c_void};
use ::core::mem::{size_of, transmute};

use crate::c_library::{__assert_fail, free, malloc, memcpy, memset};
pub use crate::types::{
    size_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, SaveLoadFunc, Snes, Spc,
    Timer, WindowLayer,
};
pub type C2RustUnnamed_0 = u32;
pub const kPpuExtraLeftRight: C2RustUnnamed_0 = 0;
pub const kPpuXPixels: C2RustUnnamed_0 = 256;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PpuWindows {
    pub edges: [i16; 6],
    pub nr: u8,
    pub bits: u8,
}
impl PpuWindows {
    const EMPTY: Self = PpuWindows {
        edges: [0; 6],
        nr: 0,
        bits: 0,
    };
}

pub const kWindow2Inversed: C2RustUnnamed_3 = 4;
pub const kWindow2Enabled: C2RustUnnamed_3 = 8;
pub const kWindow1Inversed: C2RustUnnamed_3 = 1;
pub const kWindow1Enabled: C2RustUnnamed_3 = 2;

pub const kPaletteShift: C2RustUnnamed_1 = 8;
pub type C2RustUnnamed_1 = u32;
pub const kPaletteShift_0: C2RustUnnamed_2 = 6;
pub type C2RustUnnamed_2 = u32;

pub type C2RustUnnamed_3 = u32;
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
    let ppu: *mut Ppu = malloc(size_of::<Ppu>() as size_t) as *mut Ppu;
    (*ppu).renderBuffer = core::ptr::null_mut();
    (*ppu).renderPitch = 0;
    (*ppu).snes = snes;
    ppu
}
pub unsafe fn ppu_free(ppu: *mut Ppu) {
    free(ppu as *mut c_void);
}
pub unsafe fn ppu_copy(ppu: *mut Ppu, ppu_src: *mut Ppu) {
    let snes: *mut Snes = (*ppu).snes;
    let pitch: size_t = (*ppu).renderPitch as size_t;
    let renderBuffer: *mut u8 = (*ppu).renderBuffer;
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
    (*ppu).vramIncrement = 1;
    (*ppu).mosaicSize = 1;
    (*ppu).mosaicStartLine = 1;
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
    if line == 0i32 {
        (*ppu).mosaicStartLine = 1u8;
        (*ppu).rangeOver = false_0 != 0;
        (*ppu).timeOver = false_0 != 0;
        (*ppu).evenFrame = !(*ppu).evenFrame;
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
            if (*ppu).mode as i32 == 7i32 {
                ppu_calculateMode7Starts(ppu, line);
            }
            let mut x: i32 = 0i32;
            while x < 256i32 {
                ppu_handlePixel(ppu, x, line);
                x += 1;
            }
        }
    };
}
unsafe fn ppu_windows_for_layer(ppu: *mut Ppu, sub: usize, layer: u32) -> PpuWindows {
    let mut win = PpuWindows::EMPTY;
    if (*ppu).screenWindowed[sub] & (1 << layer) != 0 {
        PpuWindows_Calc(&raw mut win, ppu, layer);
    } else {
        PpuWindows_Clear(&raw mut win, ppu, layer);
    }
    win
}
unsafe fn PpuWindows_Clear(win: *mut PpuWindows, ppu: *mut Ppu, layer: u32) {
    (*win).edges[0i32 as usize] = -if layer != 2u32 {
        (*ppu).extraLeftCur as i32
    } else {
        0i32
    } as i16;
    (*win).edges[1i32 as usize] = (256i32
        + (if layer != 2u32 {
            (*ppu).extraRightCur as i32
        } else {
            0i32
        })) as i16;
    (*win).nr = 1u8;
    (*win).bits = 0u8;
}
// Each window covers consecutive intervals between its two boundary edges.
fn ppu_window_bits(edges: &[i16; 6], left: &u8, right: &u8) -> u8 {
    let mut start: u32 = 0;
    while edges[start as usize] != *left as i16 {
        start = start.wrapping_add(1);
    }
    let mut end = start;
    while edges[end as usize] != *right as i16 + 1 {
        end = end.wrapping_add(1);
    }
    (((1i32 << end.wrapping_sub(start)) - 1) << start) as u8
}
unsafe fn PpuWindows_Calc(win: *mut PpuWindows, ppu: *mut Ppu, layer: u32) {
    let winflags: u32 = (*ppu).windowsel >> layer.wrapping_mul(4u32);
    let mut nr: u32 = 1u32;
    let window_right: i32 = 256i32
        + (if layer != 2u32 {
            (*ppu).extraRightCur as i32
        } else {
            0i32
        });
    (*win).edges[0i32 as usize] = -if layer != 2u32 {
        (*ppu).extraLeftCur as i32
    } else {
        0i32
    } as i16;
    (*win).edges[1i32 as usize] = window_right as i16;
    let mut i: u32 = 0;
    let mut j: u32 = 0;
    let mut t: i32 = 0;
    let w1_ena: bool = winflags & kWindow1Enabled as i32 as u32 != 0
        && (*ppu).window1left as i32 <= (*ppu).window1right as i32;
    if w1_ena {
        if (*ppu).window1left as i32 > (*win).edges[0i32 as usize] as i32 {
            (*win).edges[nr as usize] = (*ppu).window1left as i16;
            nr = nr.wrapping_add(1);
            (*win).edges[nr as usize] = window_right as i16;
        }
        if ((*ppu).window1right as i32 + 1i32) < window_right {
            (*win).edges[nr as usize] = ((*ppu).window1right as i32 + 1i32) as i16;
            nr = nr.wrapping_add(1);
            (*win).edges[nr as usize] = window_right as i16;
        }
    }
    let w2_ena: bool = winflags & kWindow2Enabled as i32 as u32 != 0
        && (*ppu).window2left as i32 <= (*ppu).window2right as i32;
    if w2_ena {
        i = 0u32;
        while i <= nr && {
            t = (*ppu).window2left as i32;
            t != (*win).edges[i as usize] as i32
        } {
            if t < (*win).edges[i as usize] as i32 {
                let fresh0 = nr;
                nr = nr.wrapping_add(1);
                j = fresh0;
                while j >= i {
                    (*win).edges[j.wrapping_add(1u32) as usize] = (*win).edges[j as usize];
                    j = j.wrapping_sub(1);
                }
                (*win).edges[i as usize] = t as i16;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        while i <= nr && {
            t = (*ppu).window2right as i32 + 1i32;
            t != (*win).edges[i as usize] as i32
        } {
            if t < (*win).edges[i as usize] as i32 {
                let fresh1 = nr;
                nr = nr.wrapping_add(1);
                j = fresh1;
                while j >= i {
                    (*win).edges[j.wrapping_add(1u32) as usize] = (*win).edges[j as usize];
                    j = j.wrapping_sub(1);
                }
                (*win).edges[i as usize] = t as i16;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
    }
    (*win).nr = nr as u8;
    let mut w1_bits: u8 = 0u8;
    let mut w2_bits: u8 = 0u8;
    if w1_ena {
        w1_bits = ppu_window_bits(&(*win).edges, &(*ppu).window1left, &(*ppu).window1right);
    }
    if winflags & (kWindow1Enabled as i32 | kWindow1Inversed as i32) as u32
        == (kWindow1Enabled as i32 | kWindow1Inversed as i32) as u32
    {
        w1_bits = !(w1_bits as i32) as u8;
    }
    if w2_ena {
        w2_bits = ppu_window_bits(&(*win).edges, &(*ppu).window2left, &(*ppu).window2right);
    }
    if winflags & (kWindow2Enabled as i32 | kWindow2Inversed as i32) as u32
        == (kWindow2Enabled as i32 | kWindow2Inversed as i32) as u32
    {
        w2_bits = !(w2_bits as i32) as u8;
    }
    (*win).bits = (w1_bits as i32 | w2_bits as i32) as u8;
}
// Select the tilemap halves and normal/flipped tile rows for both bit depths.
unsafe fn ppu_background_row(
    ppu: *mut Ppu,
    bglayer: *mut BgLayer,
    mut y: u32,
    layer: u32,
) -> ([*const u16; 2], i32, i32) {
    y = y.wrapping_add((*bglayer).vScroll as u32);
    let mut sc_offs: i32 =
        ((*bglayer).tilemapAdr as u32).wrapping_add((y >> 3i32 & 0x1fu32) << 5i32) as i32;
    if y & 0x100u32 != 0 && (*bglayer).tilemapHigher as i32 != 0 {
        sc_offs += if (*bglayer).tilemapWider as i32 != 0 {
            0x800i32
        } else {
            0x400i32
        };
    }
    let tps: [*const u16; 2] = [
        (&raw mut (*ppu).vram as *mut u16).offset((sc_offs & 0x7fffi32) as isize) as *const u16,
        (&raw mut (*ppu).vram as *mut u16).offset(
            (sc_offs
                + (if (*bglayer).tilemapWider as i32 != 0 {
                    0x400i32
                } else {
                    0i32
                })
                & 0x7fffi32) as isize,
        ) as *const u16,
    ];
    let tileadr: i32 = (*ppu).bgLayer[layer as usize].tileAdr as i32;
    let tileadr1: i32 = ((tileadr + 7i32) as u32).wrapping_sub(y & 0x7u32) as i32;
    let tileadr0: i32 = (tileadr as u32).wrapping_add(y & 0x7u32) as i32;
    (tps, tileadr0, tileadr1)
}
// Gather one bit from each packed plane byte into the palette index.
fn ppu_planar_pixel<const PLANES: u32>(bits: u32, shift: u32) -> i32 {
    let pixel = (bits >> shift & 1) | (bits >> (shift + 7) & 2);
    if PLANES == 4 {
        (pixel | (bits >> (shift + 14) & 4) | (bits >> (shift + 21) & 8)) as i32
    } else {
        pixel as i32
    }
}
unsafe fn ppu_load_tile_row<const PLANES: u32>(
    ppu: *mut Ppu,
    tile: u32,
    tileadr0: i32,
    tileadr1: i32,
    zhi: PpuZbufType,
    zlo: PpuZbufType,
) -> (PpuZbufType, u32) {
    let row = if tile & 0x8000 != 0 {
        tileadr1
    } else {
        tileadr0
    };
    let priority = if tile & 0x2000 != 0 { zhi } else { zlo };
    let address = (row as u32).wrapping_add((tile & 0x3ff).wrapping_mul(PLANES * 4)) & 0x7fff;
    let words = (&raw mut (*ppu).vram as *mut u16).offset(address as isize);
    let mut bits = *words as u32;
    if PLANES == 4 {
        bits |= (*words.offset(8) as u32) << 16;
    }
    (priority, bits)
}
// Store the eight pixels in screen order, retaining transparent pixels and
// destinations with equal or higher priority.
unsafe fn ppu_draw_full_tile<const PLANES: u32>(
    dst: *mut PpuZbufType,
    bits: u32,
    priority: PpuZbufType,
    hflip: bool,
) -> i32 {
    let mut pixel = 0;
    for offset in 0..8 {
        let shift = if hflip { offset } else { 7 - offset };
        pixel = ppu_planar_pixel::<PLANES>(bits, shift);
        let visible = if PLANES == 4 {
            let mask = if hflip {
                0x01010101 << offset
            } else {
                0x80808080 >> offset
            };
            bits & mask != 0
        } else {
            pixel != 0
        };
        if visible && priority > *dst.offset(offset as isize) {
            *dst.offset(offset as isize) = (i32::from(priority) + pixel) as PpuZbufType;
        }
    }
    pixel
}
// Partial rows arrive with their first pixel shifted to the end selected
// by hflip. Keep stores and bit advancement in screen order.
unsafe fn ppu_draw_partial_tile<const PLANES: u32>(
    mut dst: *mut PpuZbufType,
    mut bits: u32,
    priority: PpuZbufType,
    mut count: u32,
    hflip: bool,
) -> (u32, *mut PpuZbufType, i32) {
    let mut pixel;
    loop {
        pixel = ppu_planar_pixel::<PLANES>(bits, if hflip { 0 } else { 7 });
        let visible = if PLANES == 4 {
            bits & if hflip { 0x01010101 } else { 0x80808080 } != 0
        } else {
            pixel != 0
        };
        if visible && priority > *dst {
            *dst = (i32::from(priority) + pixel) as PpuZbufType;
        }
        if hflip {
            bits >>= 1;
        } else {
            bits <<= 1;
        }
        dst = dst.offset(1);
        count = count.wrapping_sub(1);
        if count == 0 {
            break;
        }
    }
    (bits, dst, pixel)
}
unsafe fn ppu_draw_planar_background<const PLANES: u32, const PALETTE_SHIFT: u32>(
    ppu: *mut Ppu,
    y: u32,
    sub: bool,
    layer: u32,
    zhi: PpuZbufType,
    zlo: PpuZbufType,
) {
    if (*ppu).screenEnabled[sub as usize] as i32 & (1i32) << layer == 0 {
        return;
    }
    let win = ppu_windows_for_layer(ppu, sub as usize, layer);
    let bglayer: *mut BgLayer = (&raw mut (*ppu).bgLayer as *mut BgLayer).offset(layer as isize);
    let (tps, tileadr0, tileadr1) = ppu_background_row(ppu, bglayer, y, layer);
    let mut pixel: i32 = 0;
    let mut windex: size_t = 0 as size_t;
    while windex < win.nr as size_t {
        if (win.bits as i32 & (1i32) << windex == 0) {
            let mut x: u32 = (win.edges[windex as usize] as i32 + (*bglayer).hScroll as i32) as u32;
            let mut w: u32 = (win.edges[windex.wrapping_add(1 as size_t) as usize] as i32
                - win.edges[windex as usize] as i32) as u32;
            let mut dstz: *mut PpuZbufType = (&raw mut (*(&raw mut (*ppu).bgBuffers
                as *mut PpuPixelPrioBufs)
                .offset(sub as isize))
            .data as *mut PpuZbufType)
                .offset(win.edges[windex as usize] as i32 as isize)
                .offset(kPpuExtraLeftRight as i32 as isize);
            let mut tp: *const u16 =
                tps[(x >> 8i32 & 1u32) as usize].offset((x >> 3i32 & 0x1fu32) as isize);
            let mut tp_last: *const u16 = tps[(x >> 8i32 & 1u32) as usize].offset(31);
            let mut tp_next: *const u16 = tps[(x >> 8i32 & 1u32 ^ 1u32) as usize];
            if x & 7u32 != 0 {
                let mut curw: i32 = IntMin((8u32).wrapping_sub(x & 7u32) as i32, w as i32);
                w = w.wrapping_sub(curw as u32);
                let tile: u32 = *tp as u32;
                if tp != tp_last {
                    tp = tp.offset(1);
                } else {
                    tp = tp_next;
                    tp_next = tp_last.offset(-(31i32 as isize));
                    tp_last = tp.offset(31);
                }
                let (mut z, mut bits) =
                    ppu_load_tile_row::<PLANES>(ppu, tile, tileadr0, tileadr1, zhi, zlo);
                if bits != 0 {
                    z = (z as u32).wrapping_add((tile & 0x1c00u32) >> PALETTE_SHIFT as i32)
                        as PpuZbufType as PpuZbufType;
                    if tile & 0x4000u32 != 0 {
                        bits >>= x & 7u32;
                        x = x.wrapping_add(curw as u32);
                        (bits, dstz, pixel) =
                            ppu_draw_partial_tile::<PLANES>(dstz, bits, z, curw as u32, true);
                        curw = 0;
                    } else {
                        bits <<= x & 7u32;
                        x = x.wrapping_add(curw as u32);
                        (bits, dstz, pixel) =
                            ppu_draw_partial_tile::<PLANES>(dstz, bits, z, curw as u32, false);
                        curw = 0;
                    }
                } else {
                    dstz = dstz.offset(curw as isize);
                }
            }
            while w >= 8u32 {
                let tile_0: u32 = *tp as u32;
                if tp != tp_last {
                    tp = tp.offset(1);
                } else {
                    tp = tp_next;
                    tp_next = tp_last.offset(-(31i32 as isize));
                    tp_last = tp.offset(31);
                }
                let (mut z_0, bits_0) =
                    ppu_load_tile_row::<PLANES>(ppu, tile_0, tileadr0, tileadr1, zhi, zlo);
                if bits_0 != 0 {
                    z_0 = (z_0 as u32).wrapping_add((tile_0 & 0x1c00u32) >> PALETTE_SHIFT as i32)
                        as PpuZbufType as PpuZbufType;
                    pixel = ppu_draw_full_tile::<PLANES>(dstz, bits_0, z_0, tile_0 & 0x4000 != 0);
                }
                dstz = dstz.offset(8);
                w = w.wrapping_sub(8u32);
            }
            if w != 0 {
                let tile_1: u32 = *tp as u32;
                let (mut z_1, mut bits_1) =
                    ppu_load_tile_row::<PLANES>(ppu, tile_1, tileadr0, tileadr1, zhi, zlo);
                if bits_1 != 0 {
                    z_1 = (z_1 as u32).wrapping_add((tile_1 & 0x1c00u32) >> PALETTE_SHIFT as i32)
                        as PpuZbufType as PpuZbufType;
                    if tile_1 & 0x4000u32 != 0 {
                        (bits_1, dstz, pixel) =
                            ppu_draw_partial_tile::<PLANES>(dstz, bits_1, z_1, w as u32, true);
                        w = 0;
                    } else {
                        (bits_1, dstz, pixel) =
                            ppu_draw_partial_tile::<PLANES>(dstz, bits_1, z_1, w as u32, false);
                        w = 0;
                    }
                }
            }
        }
        windex = windex.wrapping_add(1);
    }
}
unsafe fn PpuDrawBackground_4bpp(
    ppu: *mut Ppu,
    y: u32,
    sub: bool,
    layer: u32,
    zhi: PpuZbufType,
    zlo: PpuZbufType,
) {
    ppu_draw_planar_background::<4, kPaletteShift_0>(ppu, y, sub, layer, zhi, zlo);
}
unsafe fn PpuDrawBackground_2bpp(
    ppu: *mut Ppu,
    y: u32,
    sub: bool,
    layer: u32,
    zhi: PpuZbufType,
    zlo: PpuZbufType,
) {
    ppu_draw_planar_background::<2, kPaletteShift>(ppu, y, sub, layer, zhi, zlo);
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
unsafe fn PpuDrawBackground_mode7(ppu: *mut Ppu, mut y: u32, sub: bool, z: PpuZbufType) {
    let layer: i32 = 0i32;
    if (*ppu).screenEnabled[sub as usize] as i32 & (1i32) << layer == 0 {
        return;
    }
    let win = ppu_windows_for_layer(ppu, sub as usize, layer as u32);
    let (xCenter, yCenter, clippedH, clippedV) = ppu_mode7_coordinates(
        (*ppu).m7matrix[6],
        (*ppu).m7matrix[7],
        (*ppu).m7matrix[4],
        (*ppu).m7matrix[5],
    );
    let mosaic_enabled: bool = (*ppu).mosaicEnabled as i32 & (1i32) << 0i32 != 0;
    if mosaic_enabled {
        y = (*ppu).mosaicModulo[y as usize] as u32;
    }
    let ry: u32 = if (*ppu).m7yFlip as i32 != 0 {
        (255u32).wrapping_sub(y)
    } else {
        y
    };
    let m7startX: u32 = (((*ppu).m7matrix[0i32 as usize] as i32 * clippedH & !(63i32)) as u32)
        .wrapping_add(((*ppu).m7matrix[1i32 as usize] as u32).wrapping_mul(ry) & !(63i32) as u32)
        .wrapping_add(((*ppu).m7matrix[1i32 as usize] as i32 * clippedV & !(63i32)) as u32)
        .wrapping_add((xCenter << 8i32) as u32);
    let m7startY: u32 = (((*ppu).m7matrix[2i32 as usize] as i32 * clippedH & !(63i32)) as u32)
        .wrapping_add(((*ppu).m7matrix[3i32 as usize] as u32).wrapping_mul(ry) & !(63i32) as u32)
        .wrapping_add(((*ppu).m7matrix[3i32 as usize] as i32 * clippedV & !(63i32)) as u32)
        .wrapping_add((yCenter << 8i32) as u32);
    let mut windex: size_t = 0 as size_t;
    while windex < win.nr as size_t {
        if (win.bits as i32 & (1i32) << windex == 0) {
            let x: i32 = win.edges[windex as usize] as i32;
            let x2: i32 = win.edges[windex.wrapping_add(1 as size_t) as usize] as i32;
            let mut dstz: *mut PpuZbufType = (&raw mut (*(&raw mut (*ppu).bgBuffers
                as *mut PpuPixelPrioBufs)
                .offset(sub as isize))
            .data as *mut PpuZbufType)
                .offset(x as isize)
                .offset(kPpuExtraLeftRight as i32 as isize);
            let dstz_end: *mut PpuZbufType = (&raw mut (*(&raw mut (*ppu).bgBuffers
                as *mut PpuPixelPrioBufs)
                .offset(sub as isize))
            .data as *mut PpuZbufType)
                .offset(x2 as isize)
                .offset(kPpuExtraLeftRight as i32 as isize);
            let rx: u32 = (if (*ppu).m7xFlip as i32 != 0 {
                255i32 - x
            } else {
                x
            }) as u32;
            let mut xpos: u32 =
                m7startX.wrapping_add(((*ppu).m7matrix[0i32 as usize] as u32).wrapping_mul(rx));
            let mut ypos: u32 =
                m7startY.wrapping_add(((*ppu).m7matrix[2i32 as usize] as u32).wrapping_mul(rx));
            let dx: u32 = (if (*ppu).m7xFlip as i32 != 0 {
                -((*ppu).m7matrix[0i32 as usize] as i32)
            } else {
                (*ppu).m7matrix[0i32 as usize] as i32
            }) as u32;
            let dy: u32 = (if (*ppu).m7xFlip as i32 != 0 {
                -((*ppu).m7matrix[2i32 as usize] as i32)
            } else {
                (*ppu).m7matrix[2i32 as usize] as i32
            }) as u32;
            let outside_value: u32 = if (*ppu).m7largeField as i32 != 0 {
                0x3ffffu32
            } else {
                0xffffffffu32
            };
            let char_fill: bool = (*ppu).m7charFill;
            if mosaic_enabled {
                let mut w: i32 =
                    (*ppu).mosaicSize as i32 - (x - (*ppu).mosaicModulo[x as usize] as i32);
                loop {
                    w = IntMin(w, dstz_end.offset_from(dstz) as c_long as i32);
                    let pixel = ppu_mode7_sample(ppu, xpos, ypos, outside_value, char_fill);
                    if pixel != 0 {
                        let mut i: i32 = 0i32;
                        loop {
                            *dstz.offset(i as isize) = (pixel as i32 + z as i32) as PpuZbufType;
                            i += 1;
                            if (i == w) {
                                break;
                            }
                        }
                    }
                    xpos = xpos.wrapping_add(dx.wrapping_mul(w as u32));
                    ypos = ypos.wrapping_add(dy.wrapping_mul(w as u32));
                    dstz = dstz.offset(w as isize);
                    w = (*ppu).mosaicSize as i32;
                    if (dstz_end.offset_from(dstz) as c_long == 0 as c_long) {
                        break;
                    }
                }
            } else {
                loop {
                    let pixel_0 = ppu_mode7_sample(ppu, xpos, ypos, outside_value, char_fill);
                    if pixel_0 != 0 {
                        *dstz.offset(0) = (pixel_0 as i32 + z as i32) as PpuZbufType;
                    }
                    xpos = xpos.wrapping_add(dx);
                    ypos = ypos.wrapping_add(dy);
                    dstz = dstz.offset(1);
                    if (dstz == dstz_end) {
                        break;
                    }
                }
            }
        }
        windex = windex.wrapping_add(1);
    }
}
unsafe fn PpuDrawSprites(ppu: *mut Ppu, y: u32, sub: u32, clear_backdrop: bool) {
    let layer: i32 = 4i32;
    if (*ppu).screenEnabled[sub as usize] as i32 & (1i32) << layer == 0 {
        return;
    }
    let win = ppu_windows_for_layer(ppu, sub as usize, layer as u32);
    let mut windex: size_t = 0 as size_t;
    while windex < win.nr as size_t {
        if (win.bits as i32 & (1i32) << windex == 0) {
            let left: i32 = win.edges[windex as usize] as i32;
            let mut width: i32 = win.edges[windex.wrapping_add(1 as size_t) as usize] as i32 - left;
            let mut src: *mut PpuZbufType = (&raw mut (*ppu).objBuffer.data as *mut PpuZbufType)
                .offset(left as isize)
                .offset(kPpuExtraLeftRight as i32 as isize);
            let mut dst: *mut PpuZbufType = (&raw mut (*(&raw mut (*ppu).bgBuffers
                as *mut PpuPixelPrioBufs)
                .offset(sub as isize))
            .data as *mut PpuZbufType)
                .offset(left as isize)
                .offset(kPpuExtraLeftRight as i32 as isize);
            if clear_backdrop {
                memcpy(
                    dst as *mut c_void,
                    src as *const c_void,
                    (width as size_t).wrapping_mul(size_of::<u16>() as size_t),
                );
            } else {
                loop {
                    if *src.offset(0) as i32 > *dst.offset(0) as i32 {
                        *dst.offset(0) = *src.offset(0);
                    }
                    src = src.offset(1);
                    dst = dst.offset(1);
                    width -= 1;
                    if (width == 0) {
                        break;
                    }
                }
            }
        }
        windex = windex.wrapping_add(1);
    }
}
unsafe fn PpuDrawBackgrounds(ppu: *mut Ppu, y: i32, sub: bool) {
    if (*ppu).mode as i32 == 1i32 {
        if (*ppu).lineHasSprites {
            PpuDrawSprites(ppu, y as u32, sub as u32, true_0 != 0);
        }
        if (*ppu).mosaicEnabled as i32 & (1i32) << 0i32 != 0 {
            '_c2rust_label: {
                __assert_fail(
                    b"0\0" as *const u8 as *const c_char,
                    b"src/snes/ppu.c\0" as *const u8 as *const c_char,
                    679u32,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            };
        } else {
            PpuDrawBackground_4bpp(
                ppu,
                y as u32,
                sub,
                0u32,
                0xc000 as PpuZbufType,
                0x8000 as PpuZbufType,
            );
        }
        if (*ppu).mosaicEnabled as i32 & (1i32) << 1i32 != 0 {
            '_c2rust_label_0: {
                __assert_fail(
                    b"0\0" as *const u8 as *const c_char,
                    b"src/snes/ppu.c\0" as *const u8 as *const c_char,
                    684u32,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            };
        } else {
            PpuDrawBackground_4bpp(
                ppu,
                y as u32,
                sub,
                1u32,
                0xb100 as PpuZbufType,
                0x7100 as PpuZbufType,
            );
        }
        if (*ppu).mosaicEnabled as i32 & (1i32) << 2i32 != 0 {
            '_c2rust_label_1: {
                __assert_fail(
                    b"0\0" as *const u8 as *const c_char,
                    b"src/snes/ppu.c\0" as *const u8 as *const c_char,
                    689u32,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            };
        } else {
            PpuDrawBackground_2bpp(
                ppu,
                y as u32,
                sub,
                2u32,
                0xf200 as PpuZbufType,
                0x1200 as PpuZbufType,
            );
        }
    } else {
        PpuDrawBackground_mode7(ppu, y as u32, sub, 0x5000 as PpuZbufType);
        if (*ppu).lineHasSprites {
            PpuDrawSprites(ppu, y as u32, sub as u32, false_0 != 0);
        }
    };
}
unsafe fn PpuDrawWholeLine(ppu: *mut Ppu, y: u32) {
    if (*ppu).forcedBlank {
        let dst: *mut u8 = (*ppu)
            .renderBuffer
            .offset((y).wrapping_sub(1u32).wrapping_mul((*ppu).renderPitch) as isize);
        let n: size_t = (size_of::<u32>() as size_t)
            .wrapping_mul((256i32 + (*ppu).extraLeftRight as i32 * 2i32) as size_t);
        memset(dst as *mut c_void, 0i32, n);
        return;
    }
    ClearBackdrop(&mut (*ppu).bgBuffers[0]);
    PpuDrawBackgrounds(ppu, y as i32, false_0 != 0);
    let mut math_enabled: u32 = 0u32;
    let mut i: i32 = 0i32;
    while i < 6i32 {
        math_enabled |= (((*ppu).mathEnabled[i as usize] as i32) << i) as u32;
        i += 1;
    }
    let mut rendered_subscreen: bool = false_0 != 0;
    if (*ppu).preventMathMode as i32 != 3i32 && (*ppu).addSubscreen as i32 != 0 && math_enabled != 0
    {
        ClearBackdrop(&mut (*ppu).bgBuffers[1]);
        if (*ppu).screenEnabled[1i32 as usize] as i32 != 0i32 {
            PpuDrawBackgrounds(ppu, y as i32, true_0 != 0);
            rendered_subscreen = true_0 != 0;
        }
    }
    let mut cwin: PpuWindows = PpuWindows::EMPTY;
    PpuWindows_Calc(&raw mut cwin, ppu, 5u32);
    static kCwBitsMod: [u8; 8] = [0, 0xff, 0xff, 0, 0xff, 0, 0xff, 0];
    let mut cw_clip_math: u32 = (cwin.bits as i32 & kCwBitsMod[(*ppu).clipMode as usize] as i32
        ^ kCwBitsMod[((*ppu).clipMode as i32 + 4i32) as usize] as i32
        | (cwin.bits as i32 & kCwBitsMod[(*ppu).preventMathMode as usize] as i32
            ^ kCwBitsMod[((*ppu).preventMathMode as i32 + 4i32) as usize] as i32)
            << 8i32) as u32;
    let mut dst_0: *mut u32 = (*ppu)
        .renderBuffer
        .offset((y).wrapping_sub(1u32).wrapping_mul((*ppu).renderPitch) as isize)
        as *mut u32;
    let dst_org: *mut u32 = dst_0;
    dst_0 = dst_0.offset(((*ppu).extraLeftRight as i32 - (*ppu).extraLeftCur as i32) as isize);
    let mut windex: u32 = 0u32;
    loop {
        let left: u32 = (cwin.edges[windex as usize] as i32 + kPpuExtraLeftRight as i32) as u32;
        let right: u32 = (cwin.edges[windex.wrapping_add(1u32) as usize] as i32
            + kPpuExtraLeftRight as i32) as u32;
        let clip_color_mask: u32 = (if cw_clip_math & 1u32 != 0 {
            0x1fi32
        } else {
            0i32
        }) as u32;
        let mut math_enabled_cur: u32 = if cw_clip_math & 0x100u32 != 0 {
            math_enabled
        } else {
            0u32
        };
        let fixed_color: u32 = ((*ppu).fixedColorR as i32
            | ((*ppu).fixedColorG as i32) << 5i32
            | ((*ppu).fixedColorB as i32) << 10i32) as u32;
        if math_enabled_cur == 0u32
            || fixed_color == 0u32 && !(*ppu).halfColor && !rendered_subscreen
        {
            // The original do-loop emits one pixel even for an empty span.
            let count = if right > left {
                (right - left) as usize
            } else {
                1
            };
            draw_palette_span(
                core::slice::from_raw_parts_mut(dst_0, count),
                core::slice::from_raw_parts(
                    (*ppu).bgBuffers[0].data.as_ptr().add(left as usize),
                    count,
                ),
                &(*ppu).cgram,
                &(*ppu).brightnessMult,
                clip_color_mask as u16,
            );
            dst_0 = dst_0.add(count);
        } else {
            let count = if right > left {
                (right - left) as usize
            } else {
                1
            };
            let math_brightness: &[u8; 63] = if (*ppu).halfColor {
                (&(&(*ppu).brightnessMultHalf)[..63]).try_into().unwrap()
            } else {
                &(*ppu).brightnessMult
            };
            math_enabled_cur |=
                (u32::from((*ppu).addSubscreen) << 8) | (u32::from((*ppu).subtractColor) << 9);
            draw_color_math_span(
                core::slice::from_raw_parts_mut(dst_0, count),
                core::slice::from_raw_parts(
                    (*ppu).bgBuffers[0].data.as_ptr().add(left as usize),
                    count,
                ),
                core::slice::from_raw_parts(
                    (*ppu).bgBuffers[1].data.as_ptr().add(left as usize),
                    count,
                ),
                &(*ppu).cgram,
                &(*ppu).brightnessMult,
                math_brightness,
                clip_color_mask != 0,
                math_enabled_cur,
                fixed_color,
            );
            dst_0 = dst_0.add(count);
        }
        cw_clip_math >>= 1i32;
        windex = windex.wrapping_add(1);
        if (windex >= cwin.nr as u32) {
            break;
        }
    }
}
unsafe fn ppu_handlePixel(ppu: *mut Ppu, x: i32, y: i32) {
    let mut r: i32 = 0i32;
    let mut r2: i32 = 0i32;
    let mut g: i32 = 0i32;
    let mut g2: i32 = 0i32;
    let mut b: i32 = 0i32;
    let mut b2: i32 = 0i32;
    if !(*ppu).forcedBlank {
        let mainLayer: i32 =
            ppu_getPixel(ppu, x, y, false_0 != 0, &raw mut r, &raw mut g, &raw mut b);
        let colorWindowState: bool = ppu_getWindowState(ppu, 5i32, x);
        if (*ppu).clipMode as i32 == 3i32
            || (*ppu).clipMode as i32 == 2i32 && colorWindowState as i32 != 0
            || (*ppu).clipMode as i32 == 1i32 && !colorWindowState
        {
            r = 0i32;
            g = 0i32;
            b = 0i32;
        }
        let mut secondLayer: i32 = 5i32;
        let mathEnabled: bool = mainLayer < 6i32
            && (*ppu).mathEnabled[mainLayer as usize] as i32 != 0
            && !((*ppu).preventMathMode as i32 == 3i32
                || (*ppu).preventMathMode as i32 == 2i32 && colorWindowState as i32 != 0
                || (*ppu).preventMathMode as i32 == 1i32 && !colorWindowState);
        if mathEnabled as i32 != 0 && (*ppu).addSubscreen as i32 != 0
            || (*ppu).pseudoHires as i32 != 0
            || (*ppu).mode as i32 == 5i32
            || (*ppu).mode as i32 == 6i32
        {
            secondLayer = ppu_getPixel(
                ppu,
                x,
                y,
                true_0 != 0,
                &raw mut r2,
                &raw mut g2,
                &raw mut b2,
            );
        }
        if mathEnabled {
            if (*ppu).subtractColor {
                r -= if (*ppu).addSubscreen as i32 != 0 && secondLayer != 5i32 {
                    r2
                } else {
                    (*ppu).fixedColorR as i32
                };
                g -= if (*ppu).addSubscreen as i32 != 0 && secondLayer != 5i32 {
                    g2
                } else {
                    (*ppu).fixedColorG as i32
                };
                b -= if (*ppu).addSubscreen as i32 != 0 && secondLayer != 5i32 {
                    b2
                } else {
                    (*ppu).fixedColorB as i32
                };
            } else {
                r += if (*ppu).addSubscreen as i32 != 0 && secondLayer != 5i32 {
                    r2
                } else {
                    (*ppu).fixedColorR as i32
                };
                g += if (*ppu).addSubscreen as i32 != 0 && secondLayer != 5i32 {
                    g2
                } else {
                    (*ppu).fixedColorG as i32
                };
                b += if (*ppu).addSubscreen as i32 != 0 && secondLayer != 5i32 {
                    b2
                } else {
                    (*ppu).fixedColorB as i32
                };
            }
            if (*ppu).halfColor as i32 != 0 && (secondLayer != 5i32 || !(*ppu).addSubscreen) {
                r >>= 1i32;
                g >>= 1i32;
                b >>= 1i32;
            }
            if r > 31i32 {
                r = 31i32;
            }
            if g > 31i32 {
                g = 31i32;
            }
            if b > 31i32 {
                b = 31i32;
            }
            if r < 0i32 {
                r = 0i32;
            }
            if g < 0i32 {
                g = 0i32;
            }
            if b < 0i32 {
                b = 0i32;
            }
        }
        if !((*ppu).pseudoHires as i32 != 0
            || (*ppu).mode as i32 == 5i32
            || (*ppu).mode as i32 == 6i32)
        {
            r2 = r;
            g2 = g;
            b2 = b;
        }
    }
    let row: i32 = y - 1i32;
    let pixelBuffer: *mut u8 = (*ppu).renderBuffer.offset(
        (row as u32)
            .wrapping_mul((*ppu).renderPitch)
            .wrapping_add(((x + (*ppu).extraLeftRight as i32) * 4i32) as u32) as isize,
    );
    *pixelBuffer.offset(0) = ((b << 3i32 | b >> 2i32) * (*ppu).brightness as i32 / 15i32) as u8;
    *pixelBuffer.offset(1) = ((g << 3i32 | g >> 2i32) * (*ppu).brightness as i32 / 15i32) as u8;
    *pixelBuffer.offset(2) = ((r << 3i32 | r >> 2i32) * (*ppu).brightness as i32 / 15i32) as u8;
    *pixelBuffer.offset(3) = 0u8;
}
unsafe fn ppu_getPixel(
    ppu: *mut Ppu,
    x: i32,
    y: i32,
    sub: bool,
    r: *mut i32,
    g: *mut i32,
    b: *mut i32,
) -> i32 {
    let mut actMode: i32 = if (*ppu).mode as i32 == 1i32 && (*ppu).bg3priority as i32 != 0 {
        8i32
    } else {
        (*ppu).mode as i32
    };
    actMode = if (*ppu).mode as i32 == 7i32 && (*ppu).m7extBg as i32 != 0 {
        9i32
    } else {
        actMode
    };
    let mut layer: i32 = 5i32;
    let mut pixel: i32 = 0i32;
    let mut i: i32 = 0i32;
    while i < layerCountPerMode[actMode as usize] {
        let curLayer: i32 = layersPerMode[actMode as usize][i as usize];
        let curPriority: i32 = prioritysPerMode[actMode as usize][i as usize];
        let mut layerActive: bool = false_0 != 0;
        if !sub {
            layerActive = (*ppu).layer[curLayer as usize].mainScreenEnabled as i32 != 0
                && (!(*ppu).layer[curLayer as usize].mainScreenWindowed
                    || !ppu_getWindowState(ppu, curLayer, x));
        } else {
            layerActive = (*ppu).layer[curLayer as usize].subScreenEnabled as i32 != 0
                && (!(*ppu).layer[curLayer as usize].subScreenWindowed
                    || !ppu_getWindowState(ppu, curLayer, x));
        }
        if layerActive {
            if curLayer < 4i32 {
                let mut lx: i32 = x;
                let mut ly: i32 = y;
                if (*ppu).bgLayer[curLayer as usize].mosaicEnabled as i32 != 0
                    && (*ppu).mosaicSize as i32 > 1i32
                {
                    lx -= lx % (*ppu).mosaicSize as i32;
                    ly -= (ly - (*ppu).mosaicStartLine as i32) % (*ppu).mosaicSize as i32;
                }
                if (*ppu).mode as i32 == 7i32 {
                    pixel = ppu_getPixelForMode7(ppu, lx, curLayer, curPriority != 0);
                } else {
                    lx += (*ppu).bgLayer[curLayer as usize].hScroll as i32;
                    if (*ppu).mode as i32 == 5i32 || (*ppu).mode as i32 == 6i32 {
                        lx *= 2i32;
                        lx += if sub as i32 != 0
                            || (*ppu).bgLayer[curLayer as usize].mosaicEnabled as i32 != 0
                        {
                            0i32
                        } else {
                            1i32
                        };
                        if (*ppu).interlace {
                            ly *= 2i32;
                            ly += if (*ppu).evenFrame as i32 != 0
                                || (*ppu).bgLayer[curLayer as usize].mosaicEnabled as i32 != 0
                            {
                                0i32
                            } else {
                                1i32
                            };
                        }
                    }
                    ly += (*ppu).bgLayer[curLayer as usize].vScroll as i32;
                    if (*ppu).mode as i32 == 2i32
                        || (*ppu).mode as i32 == 4i32
                        || (*ppu).mode as i32 == 6i32
                    {
                        ppu_handleOPT(ppu, curLayer, &raw mut lx, &raw mut ly);
                    }
                    pixel = ppu_getPixelForBgLayer(
                        ppu,
                        lx & 0x3ffi32,
                        ly & 0x3ffi32,
                        curLayer,
                        curPriority != 0,
                    );
                }
            } else {
                pixel = 0i32;
                if (*ppu).objBuffer.data[(x + kPpuExtraLeftRight as i32) as usize] as i32 >> 12i32
                    == curPriority * 4i32 + 2i32
                {
                    pixel = (*ppu).objBuffer.data[(x + kPpuExtraLeftRight as i32) as usize] as i32
                        & 0xffi32;
                }
            }
        }
        if pixel > 0i32 {
            layer = curLayer;
            break;
        } else {
            i += 1;
        }
    }
    if (*ppu).directColor as i32 != 0
        && layer < 4i32
        && bitDepthsPerMode[actMode as usize][layer as usize] == 8i32
    {
        *r = (pixel & 0x7i32) << 2i32 | (pixel & 0x100i32) >> 7i32;
        *g = (pixel & 0x38i32) >> 1i32 | (pixel & 0x200i32) >> 8i32;
        *b = (pixel & 0xc0i32) >> 3i32 | (pixel & 0x400i32) >> 8i32;
    } else {
        let color: u16 = (*ppu).cgram[(pixel & 0xffi32) as usize];
        *r = color as i32 & 0x1fi32;
        *g = color as i32 >> 5i32 & 0x1fi32;
        *b = color as i32 >> 10i32 & 0x1fi32;
    }
    if layer == 4i32 && pixel < 0xc0i32 {
        layer = 6i32;
    }
    layer
}
unsafe fn ppu_handleOPT(ppu: *mut Ppu, layer: i32, lx: *mut i32, ly: *mut i32) {
    let x: i32 = *lx;
    let y: i32 = *ly;
    let mut column: i32 = 0i32;
    if (*ppu).mode as i32 == 6i32 {
        column =
            x - (x & 0xfi32) - ((*ppu).bgLayer[layer as usize].hScroll as i32 * 2i32 & 0xfff0i32)
                >> 4i32;
    } else {
        column =
            x - (x & 0x7i32) - ((*ppu).bgLayer[layer as usize].hScroll as i32 & 0xfff8i32) >> 3i32;
    }
    if column > 0i32 {
        let valid: i32 = if layer == 0i32 { 0x2000i32 } else { 0x4000i32 };
        let mut hOffset: u16 = ppu_getOffsetValue(ppu, column - 1i32, 0i32);
        let mut vOffset: u16 = 0u16;
        if (*ppu).mode as i32 == 4i32 {
            if hOffset as i32 & 0x8000i32 != 0 {
                vOffset = hOffset;
                hOffset = 0u16;
            }
        } else {
            vOffset = ppu_getOffsetValue(ppu, column - 1i32, 1i32);
        }
        if (*ppu).mode as i32 == 6i32 {
            if hOffset as i32 & valid != 0 {
                *lx = ((hOffset as i32 & 0x3f8i32) + column * 8i32) * 2i32 | x & 0xfi32;
            }
        } else if hOffset as i32 & valid != 0 {
            *lx = (hOffset as i32 & 0x3f8i32) + column * 8i32 | x & 0x7i32;
        }
        if vOffset as i32 & valid != 0 {
            *ly = (vOffset as i32 & 0x3ffi32) + (y - (*ppu).bgLayer[layer as usize].vScroll as i32);
        }
    }
}
unsafe fn ppu_getOffsetValue(ppu: *mut Ppu, col: i32, row: i32) -> u16 {
    let x: i32 = col * 8i32 + (*ppu).bgLayer[2i32 as usize].hScroll as i32;
    let y: i32 = row * 8i32 + (*ppu).bgLayer[2i32 as usize].vScroll as i32;
    let tileBits: i32 = if (*ppu).bgLayer[2i32 as usize].bigTiles as i32 != 0 {
        4i32
    } else {
        3i32
    };
    let tileHighBit: i32 = if (*ppu).bgLayer[2i32 as usize].bigTiles as i32 != 0 {
        0x200i32
    } else {
        0x100i32
    };
    let mut tilemapAdr: u16 = ((*ppu).bgLayer[2i32 as usize].tilemapAdr as i32
        + ((y >> tileBits & 0x1fi32) << 5i32 | x >> tileBits & 0x1fi32))
        as u16;
    if x & tileHighBit != 0 && (*ppu).bgLayer[2i32 as usize].tilemapWider as i32 != 0 {
        tilemapAdr = (tilemapAdr as i32 + 0x400i32) as u16;
    }
    if y & tileHighBit != 0 && (*ppu).bgLayer[2i32 as usize].tilemapHigher as i32 != 0 {
        tilemapAdr = (tilemapAdr as i32
            + if (*ppu).bgLayer[2i32 as usize].tilemapWider as i32 != 0 {
                0x800i32
            } else {
                0x400i32
            }) as u16;
    }
    (*ppu).vram[(tilemapAdr as i32 & 0x7fffi32) as usize]
}
unsafe fn ppu_getPixelForBgLayer(ppu: *mut Ppu, x: i32, y: i32, layer: i32, priority: bool) -> i32 {
    let wideTiles: bool = (*ppu).bgLayer[layer as usize].bigTiles as i32 != 0
        || (*ppu).mode as i32 == 5i32
        || (*ppu).mode as i32 == 6i32;
    let tileBitsX: i32 = if wideTiles as i32 != 0 { 4i32 } else { 3i32 };
    let tileHighBitX: i32 = if wideTiles as i32 != 0 {
        0x200i32
    } else {
        0x100i32
    };
    let tileBitsY: i32 = if (*ppu).bgLayer[layer as usize].bigTiles as i32 != 0 {
        4i32
    } else {
        3i32
    };
    let tileHighBitY: i32 = if (*ppu).bgLayer[layer as usize].bigTiles as i32 != 0 {
        0x200i32
    } else {
        0x100i32
    };
    let mut tilemapAdr: u16 = ((*ppu).bgLayer[layer as usize].tilemapAdr as i32
        + ((y >> tileBitsY & 0x1fi32) << 5i32 | x >> tileBitsX & 0x1fi32))
        as u16;
    if x & tileHighBitX != 0 && (*ppu).bgLayer[layer as usize].tilemapWider as i32 != 0 {
        tilemapAdr = (tilemapAdr as i32 + 0x400i32) as u16;
    }
    if y & tileHighBitY != 0 && (*ppu).bgLayer[layer as usize].tilemapHigher as i32 != 0 {
        tilemapAdr = (tilemapAdr as i32
            + if (*ppu).bgLayer[layer as usize].tilemapWider as i32 != 0 {
                0x800i32
            } else {
                0x400i32
            }) as u16;
    }
    let tile: u16 = (*ppu).vram[(tilemapAdr as i32 & 0x7fffi32) as usize];
    if (tile as i32 & 0x2000i32 != 0) as i32 != priority as i32 {
        return 0i32;
    }
    let mut paletteNum: i32 = (tile as i32 & 0x1c00i32) >> 10i32;
    let row: i32 = if tile as i32 & 0x8000i32 != 0 {
        7i32 - (y & 0x7i32)
    } else {
        y & 0x7i32
    };
    let col: i32 = if tile as i32 & 0x4000i32 != 0 {
        x & 0x7i32
    } else {
        7i32 - (x & 0x7i32)
    };
    let mut tileNum: i32 = tile as i32 & 0x3ffi32;
    if wideTiles && (x & 8i32 != 0) as i32 ^ (tile as i32 & 0x4000i32 != 0) as i32 != 0 {
        tileNum += 1i32;
    }
    if (*ppu).bgLayer[layer as usize].bigTiles
        && (y & 8i32 != 0) as i32 ^ (tile as i32 & 0x8000i32 != 0) as i32 != 0
    {
        tileNum += 0x10i32;
    }
    let bitDepth: i32 = bitDepthsPerMode[(*ppu).mode as usize][layer as usize];
    if (*ppu).mode as i32 == 0i32 {
        paletteNum += 8i32 * layer;
    }
    let mut paletteSize: i32 = 4i32;
    let plane1: u16 = (*ppu).vram[((*ppu).bgLayer[layer as usize].tileAdr as i32
        + (tileNum & 0x3ffi32) * 4i32 * bitDepth
        + row
        & 0x7fffi32) as usize];
    let mut pixel: i32 = plane1 as i32 >> col & 1i32;
    pixel |= (plane1 as i32 >> 8i32 + col & 1i32) << 1i32;
    if bitDepth > 2i32 {
        paletteSize = 16i32;
        let plane2: u16 = (*ppu).vram[((*ppu).bgLayer[layer as usize].tileAdr as i32
            + (tileNum & 0x3ffi32) * 4i32 * bitDepth
            + 8i32
            + row
            & 0x7fffi32) as usize];
        pixel |= (plane2 as i32 >> col & 1i32) << 2i32;
        pixel |= (plane2 as i32 >> 8i32 + col & 1i32) << 3i32;
    }
    if bitDepth > 4i32 {
        paletteSize = 256i32;
        let plane3: u16 = (*ppu).vram[((*ppu).bgLayer[layer as usize].tileAdr as i32
            + (tileNum & 0x3ffi32) * 4i32 * bitDepth
            + 16i32
            + row
            & 0x7fffi32) as usize];
        pixel |= (plane3 as i32 >> col & 1i32) << 4i32;
        pixel |= (plane3 as i32 >> 8i32 + col & 1i32) << 5i32;
        let plane4: u16 = (*ppu).vram[((*ppu).bgLayer[layer as usize].tileAdr as i32
            + (tileNum & 0x3ffi32) * 4i32 * bitDepth
            + 24i32
            + row
            & 0x7fffi32) as usize];
        pixel |= (plane4 as i32 >> col & 1i32) << 6i32;
        pixel |= (plane4 as i32 >> 8i32 + col & 1i32) << 7i32;
    }
    if pixel == 0i32 {
        0i32
    } else {
        paletteSize * paletteNum + pixel
    }
}
unsafe fn ppu_calculateMode7Starts(ppu: *mut Ppu, mut y: i32) {
    let (xCenter, yCenter, clippedH, clippedV) = ppu_mode7_coordinates(
        (*ppu).m7matrix[6],
        (*ppu).m7matrix[7],
        (*ppu).m7matrix[4],
        (*ppu).m7matrix[5],
    );
    if (*ppu).bgLayer[0i32 as usize].mosaicEnabled as i32 != 0 && (*ppu).mosaicSize as i32 > 1i32 {
        y -= (y - (*ppu).mosaicStartLine as i32) % (*ppu).mosaicSize as i32;
    }
    let ry: u8 = (if (*ppu).m7yFlip as i32 != 0 {
        255i32 - y
    } else {
        y
    }) as u8;
    (*ppu).m7startX = (((*ppu).m7matrix[0i32 as usize] as i32 * clippedH & !(63i32))
        + ((*ppu).m7matrix[1i32 as usize] as i32 * ry as i32 & !(63i32))
        + ((*ppu).m7matrix[1i32 as usize] as i32 * clippedV & !(63i32))
        + (xCenter << 8i32));
    (*ppu).m7startY = (((*ppu).m7matrix[2i32 as usize] as i32 * clippedH & !(63i32))
        + ((*ppu).m7matrix[3i32 as usize] as i32 * ry as i32 & !(63i32))
        + ((*ppu).m7matrix[3i32 as usize] as i32 * clippedV & !(63i32))
        + (yCenter << 8i32));
}
unsafe fn ppu_getPixelForMode7(ppu: *mut Ppu, x: i32, layer: i32, priority: bool) -> i32 {
    let rx: u8 = (if (*ppu).m7xFlip as i32 != 0 {
        255i32 - x
    } else {
        x
    }) as u8;
    let mut xPos: i32 = (*ppu).m7startX + (*ppu).m7matrix[0i32 as usize] as i32 * rx as i32 >> 8i32;
    let mut yPos: i32 = (*ppu).m7startY + (*ppu).m7matrix[2i32 as usize] as i32 * rx as i32 >> 8i32;
    let mut outsideMap: bool = xPos < 0i32 || xPos >= 1024i32 || yPos < 0i32 || yPos >= 1024i32;
    xPos &= 0x3ffi32;
    yPos &= 0x3ffi32;
    if !(*ppu).m7largeField {
        outsideMap = false_0 != 0;
    }
    let tile: u8 = (if outsideMap as i32 != 0 {
        0i32
    } else {
        (*ppu).vram[((yPos >> 3i32) * 128i32 + (xPos >> 3i32)) as usize] as i32 & 0xffi32
    }) as u8;
    let pixel: u8 = (if outsideMap as i32 != 0 && !(*ppu).m7charFill {
        0i32
    } else {
        (*ppu).vram[(tile as i32 * 64i32 + (yPos & 7i32) * 8i32 + (xPos & 7i32)) as usize] as i32
            >> 8i32
    }) as u8;
    if layer == 1i32 {
        if (pixel as i32 & 0x80i32 != 0) as i32 != priority as i32 {
            return 0i32;
        }
        return pixel as i32 & 0x7fi32;
    }
    pixel as i32
}
unsafe fn ppu_getWindowState(ppu: *mut Ppu, layer: i32, x: i32) -> bool {
    if !(*ppu).windowLayer[layer as usize].window1enabled
        && !(*ppu).windowLayer[layer as usize].window2enabled
    {
        return false_0 != 0;
    }
    if (*ppu).windowLayer[layer as usize].window1enabled as i32 != 0
        && !(*ppu).windowLayer[layer as usize].window2enabled
    {
        let test: bool = x >= (*ppu).window1left as i32 && x <= (*ppu).window1right as i32;
        return if (*ppu).windowLayer[layer as usize].window1inversed as i32 != 0 {
            !test as i32
        } else {
            test as i32
        } != 0;
    }
    if !(*ppu).windowLayer[layer as usize].window1enabled
        && (*ppu).windowLayer[layer as usize].window2enabled as i32 != 0
    {
        let test_0: bool = x >= (*ppu).window2left as i32 && x <= (*ppu).window2right as i32;
        return if (*ppu).windowLayer[layer as usize].window2inversed as i32 != 0 {
            !test_0 as i32
        } else {
            test_0 as i32
        } != 0;
    }
    let mut test1: bool = x >= (*ppu).window1left as i32 && x <= (*ppu).window1right as i32;
    let mut test2: bool = x >= (*ppu).window2left as i32 && x <= (*ppu).window2right as i32;
    if (*ppu).windowLayer[layer as usize].window1inversed {
        test1 = !test1;
    }
    if (*ppu).windowLayer[layer as usize].window2inversed {
        test2 = !test2;
    }
    match (*ppu).windowLayer[layer as usize].maskLogic as i32 {
        0 => return test1 as i32 != 0 || test2 as i32 != 0,
        1 => return test1 as i32 != 0 && test2 as i32 != 0,
        2 => return test1 as i32 != test2 as i32,
        3 => return test1 as i32 == test2 as i32,
        _ => {}
    }
    false_0 != 0
}
unsafe fn ppu_evaluateSprites(ppu: *mut Ppu, line: i32) -> bool {
    let mut index: u8 = (if (*ppu).objPriority as i32 != 0 {
        (*ppu).oamAdr as i32 & 0xfei32
    } else {
        0i32
    }) as u8;
    let mut spritesFound: i32 = 0i32;
    let mut tilesFound: i32 = 0i32;
    let mut i: i32 = 0i32;
    while i < 128i32 {
        let y: u8 = ((*ppu).oam[index as usize] as i32 >> 8i32) as u8;
        let mut row: u8 = (line - y as i32) as u8;
        let spriteSize: i32 = spriteSizes[(*ppu).objSize as usize][((*ppu).highOam
            [(index as i32 >> 3i32) as usize]
            as i32
            >> (index as i32 & 7i32) + 1i32
            & 1i32) as usize];
        let spriteHeight: i32 = if (*ppu).objInterlace as i32 != 0 {
            spriteSize / 2i32
        } else {
            spriteSize
        };
        if (row as i32) < spriteHeight {
            let mut x: i32 = (*ppu).oam[index as usize] as i32 & 0xffi32;
            x |= ((*ppu).highOam[(index as i32 >> 3i32) as usize] as i32 >> (index as i32 & 7i32)
                & 1i32)
                << 8i32;
            if x > 255i32 {
                x -= 512i32;
            }
            if x > -spriteSize {
                spritesFound += 1;
                if spritesFound > 32i32 {
                    (*ppu).rangeOver = true_0 != 0;
                    break;
                } else {
                    if (*ppu).objInterlace {
                        row = (row as i32 * 2i32
                            + (if (*ppu).evenFrame as i32 != 0 {
                                0i32
                            } else {
                                1i32
                            })) as u8;
                    }
                    let oam1: i32 = (*ppu).oam[(index as i32 + 1i32) as usize] as i32;
                    let tile: i32 = oam1 & 0xffi32;
                    let objAdr: i32 = if oam1 & 0x100i32 != 0 {
                        (*ppu).objTileAdr2 as i32
                    } else {
                        (*ppu).objTileAdr1 as i32
                    };
                    let palette: i32 = (oam1 & 0xe00i32) >> 9i32;
                    let hFlipped: bool = oam1 & 0x4000i32 != 0;
                    if oam1 & 0x8000i32 != 0 {
                        row = (spriteSize - 1i32 - row as i32) as u8;
                    }
                    let paletteBase: i32 = 0x80i32 + 16i32 * ((oam1 & 0xe00i32) >> 9i32);
                    let prio: i32 = (((oam1 & 0x3000i32) >> 12i32) * 4i32 + 2i32) * 16i32
                        + 4i32
                        + (if oam1 & 0x800i32 == 0i32 { 2i32 } else { 0i32 });
                    let z: PpuZbufType = (paletteBase + (prio << 8i32)) as PpuZbufType;
                    let mut col: i32 = 0i32;
                    while col < spriteSize {
                        if col + x > -8i32 && col + x < 256i32 {
                            tilesFound += 1;
                            if tilesFound > 34i32 {
                                (*ppu).timeOver = true_0 != 0;
                                break;
                            } else {
                                let usedCol: i32 = if oam1 & 0x4000i32 != 0 {
                                    spriteSize - 1i32 - col
                                } else {
                                    col
                                };
                                let usedTile: i32 =
                                    ((oam1 & 0xffi32) >> 4i32) + (row as i32 >> 3i32) << 4i32
                                        | (oam1 & 0xfi32) + (usedCol >> 3i32) & 0xfi32;
                                let addr: *mut u16 = (&raw mut (*ppu).vram as *mut u16).offset(
                                    (objAdr + usedTile * 16i32 + (row as i32 & 0x7i32) & 0x7fffi32)
                                        as isize,
                                );
                                let plane: u32 = (*addr.offset(0) as i32
                                    | (*addr.offset(8) as i32) << 16i32)
                                    as u32;
                                let px_left: i32 =
                                    IntMax(-(col + x + kPpuExtraLeftRight as i32), 0i32);
                                let px_right: i32 =
                                    IntMin(256i32 + kPpuExtraLeftRight as i32 - (col + x), 8i32);
                                let dst: *mut PpuZbufType = (&raw mut (*ppu).objBuffer.data
                                    as *mut PpuZbufType)
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
                        col += 8i32;
                    }
                    if tilesFound > 34i32 {
                        break;
                    }
                }
            }
        }
        index = (index as i32 + 2i32) as u8;
        i += 1;
    }
    tilesFound != 0i32
}
unsafe fn ppu_getVramRemap(ppu: *mut Ppu) -> u16 {
    let adr: u16 = (*ppu).vramPointer;
    match (*ppu).vramRemapMode as i32 {
        0 => return adr,
        1 => {
            return (adr as i32 & 0xff00i32
                | (adr as i32 & 0xe0i32) >> 5i32
                | (adr as i32 & 0x1fi32) << 3i32) as u16;
        }
        2 => {
            return (adr as i32 & 0xfe00i32
                | (adr as i32 & 0x1c0i32) >> 6i32
                | (adr as i32 & 0x3fi32) << 3i32) as u16;
        }
        3 => {
            return (adr as i32 & 0xfc00i32
                | (adr as i32 & 0x380i32) >> 7i32
                | (adr as i32 & 0x7fi32) << 3i32) as u16;
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
                * ((*ppu).m7matrix[1i32 as usize] as i32 >> 8i32);
            (*ppu).ppu1openBus = (result >> 8i32 * (adr as i32 - 0x34i32) & 0xffi32) as u8;
            (*ppu).ppu1openBus
        }
        55 => {
            (*ppu).hCount = ((*(*ppu).snes).hPos as i32 / 4i32) as u16;
            (*ppu).vCount = (*(*ppu).snes).vPos;
            (*ppu).countersLatched = true_0 != 0;
            (*(*ppu).snes).openBus
        }
        56 => {
            let mut ret: u8 = 0u8;
            if (*ppu).oamInHigh {
                ret = (*ppu).highOam[(((*ppu).oamAdr as i32 & 0xfi32) << 1i32
                    | (*ppu).oamSecondWrite as i32) as usize];
                if (*ppu).oamSecondWrite {
                    (*ppu).oamAdr = (*ppu).oamAdr.wrapping_add(1);
                    if (*ppu).oamAdr as i32 == 0i32 {
                        (*ppu).oamInHigh = false_0 != 0;
                    }
                }
            } else if !(*ppu).oamSecondWrite {
                ret = ((*ppu).oam[(*ppu).oamAdr as usize] as i32 & 0xffi32) as u8;
            } else {
                let fresh2 = (*ppu).oamAdr;
                (*ppu).oamAdr = (*ppu).oamAdr.wrapping_add(1);
                ret = ((*ppu).oam[fresh2 as usize] as i32 >> 8i32) as u8;
                if (*ppu).oamAdr as i32 == 0i32 {
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
                    (*ppu).vram[(ppu_getVramRemap(ppu) as i32 & 0x7fffi32) as usize];
                (*ppu).vramPointer =
                    ((*ppu).vramPointer as i32 + (*ppu).vramIncrement as i32) as u16;
            }
            (*ppu).ppu1openBus = (val as i32 & 0xffi32) as u8;
            (val as i32 & 0xffi32) as u8
        }
        58 => {
            let val_0: u16 = (*ppu).vramReadBuffer;
            if (*ppu).vramIncrementOnHigh {
                (*ppu).vramReadBuffer =
                    (*ppu).vram[(ppu_getVramRemap(ppu) as i32 & 0x7fffi32) as usize];
                (*ppu).vramPointer =
                    ((*ppu).vramPointer as i32 + (*ppu).vramIncrement as i32) as u16;
            }
            (*ppu).ppu1openBus = (val_0 as i32 >> 8i32) as u8;
            (val_0 as i32 >> 8i32) as u8
        }
        59 => {
            let mut ret_0: u8 = 0u8;
            if !(*ppu).cgramSecondWrite {
                ret_0 = ((*ppu).cgram[(*ppu).cgramPointer as usize] as i32 & 0xffi32) as u8;
            } else {
                let fresh3 = (*ppu).cgramPointer;
                (*ppu).cgramPointer = (*ppu).cgramPointer.wrapping_add(1);
                ret_0 = ((*ppu).cgram[fresh3 as usize] as i32 >> 8i32 & 0x7fi32
                    | (*ppu).ppu2openBus as i32 & 0x80i32) as u8;
            }
            (*ppu).cgramSecondWrite = !(*ppu).cgramSecondWrite;
            (*ppu).ppu2openBus = ret_0;
            ret_0
        }
        60 => {
            let mut val_1: u8 = 0u8;
            if (*ppu).hCountSecond {
                val_1 = ((*ppu).hCount as i32 >> 8i32 & 1i32 | (*ppu).ppu2openBus as i32 & 0xfei32)
                    as u8;
            } else {
                val_1 = ((*ppu).hCount as i32 & 0xffi32) as u8;
            }
            (*ppu).hCountSecond = !(*ppu).hCountSecond;
            (*ppu).ppu2openBus = val_1;
            val_1
        }
        61 => {
            let mut val_2: u8 = 0u8;
            if (*ppu).vCountSecond {
                val_2 = ((*ppu).vCount as i32 >> 8i32 & 1i32 | (*ppu).ppu2openBus as i32 & 0xfei32)
                    as u8;
            } else {
                val_2 = ((*ppu).vCount as i32 & 0xffi32) as u8;
            }
            (*ppu).vCountSecond = !(*ppu).vCountSecond;
            (*ppu).ppu2openBus = val_2;
            val_2
        }
        62 => {
            let mut val_3: u8 = 0x1u8;
            val_3 = (val_3 as i32 | (*ppu).ppu1openBus as i32 & 0x10i32) as u8;
            val_3 = (val_3 as i32 | ((*ppu).rangeOver as i32) << 6i32) as u8;
            val_3 = (val_3 as i32 | ((*ppu).timeOver as i32) << 7i32) as u8;
            (*ppu).ppu1openBus = val_3;
            val_3
        }
        63 => {
            let mut val_4: u8 = 0x3u8;
            val_4 = (val_4 as i32 | (*ppu).ppu2openBus as i32 & 0x20i32) as u8;
            val_4 = (val_4 as i32 | ((*ppu).countersLatched as i32) << 6i32) as u8;
            val_4 = (val_4 as i32 | ((*ppu).evenFrame as i32) << 7i32) as u8;
            (*ppu).countersLatched = false_0 != 0;
            (*ppu).hCountSecond = false_0 != 0;
            (*ppu).vCountSecond = false_0 != 0;
            (*ppu).ppu2openBus = val_4;
            val_4
        }
        _ => (*(*ppu).snes).openBus,
    }
}
pub unsafe fn ppu_write(ppu: *mut Ppu, adr: u8, val: u8) {
    let current_block_164: u64;
    match adr as i32 {
        0 => {
            (*ppu).brightness = (val as i32 & 0xfi32) as u8;
            (*ppu).forcedBlank = val as i32 & 0x80i32 != 0;
            current_block_164 = 16754622181974910496;
        }
        1 => {
            (*ppu).objSize = (val as i32 >> 5i32) as u8;
            (*ppu).objTileAdr1 = ((val as i32 & 7i32) << 13i32) as u16;
            (*ppu).objTileAdr2 =
                ((*ppu).objTileAdr1 as i32 + (((val as i32 & 0x18i32) + 8i32) << 9i32)) as u16;
            current_block_164 = 16754622181974910496;
        }
        2 => {
            (*ppu).oamAdr = val;
            (*ppu).oamAdrWritten = (*ppu).oamAdr;
            (*ppu).oamInHigh = (*ppu).oamInHighWritten;
            (*ppu).oamSecondWrite = false_0 != 0;
            current_block_164 = 16754622181974910496;
        }
        3 => {
            (*ppu).objPriority = val as i32 & 0x80i32 != 0;
            (*ppu).oamInHigh = val as i32 & 1i32 != 0;
            (*ppu).oamInHighWritten = (*ppu).oamInHigh;
            (*ppu).oamAdr = (*ppu).oamAdrWritten;
            (*ppu).oamSecondWrite = false_0 != 0;
            current_block_164 = 16754622181974910496;
        }
        4 => {
            if (*ppu).oamInHigh {
                (*ppu).highOam[(((*ppu).oamAdr as i32 & 0xfi32) << 1i32
                    | (*ppu).oamSecondWrite as i32) as usize] = val;
                if (*ppu).oamSecondWrite {
                    (*ppu).oamAdr = (*ppu).oamAdr.wrapping_add(1);
                    if (*ppu).oamAdr as i32 == 0i32 {
                        (*ppu).oamInHigh = false_0 != 0;
                    }
                }
            } else if !(*ppu).oamSecondWrite {
                (*ppu).oamBuffer = val;
            } else {
                let fresh4 = (*ppu).oamAdr;
                (*ppu).oamAdr = (*ppu).oamAdr.wrapping_add(1);
                (*ppu).oam[fresh4 as usize] =
                    ((val as i32) << 8i32 | (*ppu).oamBuffer as i32) as u16;
                if (*ppu).oamAdr as i32 == 0i32 {
                    (*ppu).oamInHigh = true_0 != 0;
                }
            }
            (*ppu).oamSecondWrite = !(*ppu).oamSecondWrite;
            current_block_164 = 16754622181974910496;
        }
        5 => {
            (*ppu).mode = (val as i32 & 0x7i32) as u8;
            (*ppu).bg3priority = val as i32 & 0x8i32 != 0;
            (*ppu).bgLayer[0i32 as usize].bigTiles = val as i32 & 0x10i32 != 0;
            (*ppu).bgLayer[1i32 as usize].bigTiles = val as i32 & 0x20i32 != 0;
            (*ppu).bgLayer[2i32 as usize].bigTiles = val as i32 & 0x40i32 != 0;
            (*ppu).bgLayer[3i32 as usize].bigTiles = val as i32 & 0x80i32 != 0;
            current_block_164 = 16754622181974910496;
        }
        6 => {
            (*ppu).bgLayer[0i32 as usize].mosaicEnabled = val as i32 & 0x1i32 != 0;
            (*ppu).bgLayer[1i32 as usize].mosaicEnabled = val as i32 & 0x2i32 != 0;
            (*ppu).bgLayer[2i32 as usize].mosaicEnabled = val as i32 & 0x4i32 != 0;
            (*ppu).bgLayer[3i32 as usize].mosaicEnabled = val as i32 & 0x8i32 != 0;
            (*ppu).mosaicSize = ((val as i32 >> 4i32) + 1i32) as u8;
            (*ppu).mosaicStartLine = 0u8;
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
        15 | 17 | 19 => {
            current_block_164 = 6174974146017752131;
        }
        14 => {
            (*ppu).m7matrix[7] = ((u16::from(val) << 8 | u16::from((*ppu).m7prev)) & 0x1fff) as i16;
            (*ppu).m7prev = val;
            current_block_164 = 5892776923941496671;
        }
        16 | 18 | 20 => {
            current_block_164 = 5892776923941496671;
        }
        21 => {
            if val as i32 & 3i32 == 0i32 {
                (*ppu).vramIncrement = 1u16;
            } else if val as i32 & 3i32 == 1i32 {
                (*ppu).vramIncrement = 32u16;
            } else {
                (*ppu).vramIncrement = 128u16;
            }
            (*ppu).vramRemapMode = ((val as i32 & 0xci32) >> 2i32) as u8;
            (*ppu).vramIncrementOnHigh = val as i32 & 0x80i32 != 0;
            current_block_164 = 16754622181974910496;
        }
        22 => {
            (*ppu).vramPointer = ((*ppu).vramPointer as i32 & 0xff00i32 | val as i32) as u16;
            (*ppu).vramReadBuffer =
                (*ppu).vram[(ppu_getVramRemap(ppu) as i32 & 0x7fffi32) as usize];
            current_block_164 = 16754622181974910496;
        }
        23 => {
            (*ppu).vramPointer =
                ((*ppu).vramPointer as i32 & 0xffi32 | (val as i32) << 8i32) as u16;
            (*ppu).vramReadBuffer =
                (*ppu).vram[(ppu_getVramRemap(ppu) as i32 & 0x7fffi32) as usize];
            current_block_164 = 16754622181974910496;
        }
        24 => {
            let vramAdr: u16 = ppu_getVramRemap(ppu);
            (*ppu).vram[(vramAdr as i32 & 0x7fffi32) as usize] = ((*ppu).vram
                [(vramAdr as i32 & 0x7fffi32) as usize]
                as i32
                & 0xff00i32
                | val as i32)
                as u16;
            if !(*ppu).vramIncrementOnHigh {
                (*ppu).vramPointer =
                    ((*ppu).vramPointer as i32 + (*ppu).vramIncrement as i32) as u16;
            }
            current_block_164 = 16754622181974910496;
        }
        25 => {
            let vramAdr_0: u16 = ppu_getVramRemap(ppu);
            (*ppu).vram[(vramAdr_0 as i32 & 0x7fffi32) as usize] =
                ((*ppu).vram[(vramAdr_0 as i32 & 0x7fffi32) as usize] as i32 & 0xffi32
                    | (val as i32) << 8i32) as u16;
            if (*ppu).vramIncrementOnHigh {
                (*ppu).vramPointer =
                    ((*ppu).vramPointer as i32 + (*ppu).vramIncrement as i32) as u16;
            }
            current_block_164 = 16754622181974910496;
        }
        26 => {
            (*ppu).m7largeField = val & 0x80 != 0;
            (*ppu).m7charFill = val & 0x40 != 0;
            (*ppu).m7yFlip = val & 0x02 != 0;
            (*ppu).m7xFlip = val & 0x01 != 0;
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
                let fresh5 = (*ppu).cgramPointer;
                (*ppu).cgramPointer = (*ppu).cgramPointer.wrapping_add(1);
                (*ppu).cgram[fresh5 as usize] =
                    ((val as i32) << 8i32 | (*ppu).cgramBuffer as i32) as u16;
            }
            (*ppu).cgramSecondWrite = !(*ppu).cgramSecondWrite;
            current_block_164 = 16754622181974910496;
        }
        35 | 36 | 37 => {
            if adr as i32 == 0x23i32 {
                (*ppu).windowsel = (*ppu).windowsel & !(0xffi32) as u32 | val as u32;
            } else if adr as i32 == 0x24i32 {
                (*ppu).windowsel =
                    (*ppu).windowsel & !(0xff00i32) as u32 | ((val as i32) << 8i32) as u32;
            } else if adr as i32 == 0x25i32 {
                (*ppu).windowsel =
                    (*ppu).windowsel & !(0xff0000i32) as u32 | ((val as i32) << 16i32) as u32;
            }
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32) as usize].window1inversed =
                val as i32 & 0x1i32 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32) as usize].window1enabled =
                val as i32 & 0x2i32 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32) as usize].window2inversed =
                val as i32 & 0x4i32 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32) as usize].window2enabled =
                val as i32 & 0x8i32 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32 + 1i32) as usize].window1inversed =
                val as i32 & 0x10i32 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32 + 1i32) as usize].window1enabled =
                val as i32 & 0x20i32 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32 + 1i32) as usize].window2inversed =
                val as i32 & 0x40i32 != 0;
            (*ppu).windowLayer[((adr as i32 - 0x23i32) * 2i32 + 1i32) as usize].window2enabled =
                val as i32 & 0x80i32 != 0;
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
            (*ppu).windowLayer[0i32 as usize].maskLogic = (val as i32 & 0x3i32) as u8;
            (*ppu).windowLayer[1i32 as usize].maskLogic = (val as i32 >> 2i32 & 0x3i32) as u8;
            (*ppu).windowLayer[2i32 as usize].maskLogic = (val as i32 >> 4i32 & 0x3i32) as u8;
            (*ppu).windowLayer[3i32 as usize].maskLogic = (val as i32 >> 6i32 & 0x3i32) as u8;
            current_block_164 = 16754622181974910496;
        }
        43 => {
            (*ppu).windowLayer[4i32 as usize].maskLogic = (val as i32 & 0x3i32) as u8;
            (*ppu).windowLayer[5i32 as usize].maskLogic = (val as i32 >> 2i32 & 0x3i32) as u8;
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
            (*ppu).directColor = val & 0x01 != 0;
            (*ppu).addSubscreen = val & 0x02 != 0;
            (*ppu).preventMathMode = (val & 0x30) >> 4;
            (*ppu).clipMode = (val & 0xc0) >> 6;
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
            (*ppu).interlace = val & 0x01 != 0;
            (*ppu).objInterlace = val & 0x02 != 0;
            (*ppu).overscan = val & 0x04 != 0;
            (*ppu).pseudoHires = val & 0x08 != 0;
            (*ppu).m7extBg = val & 0x40 != 0;
            current_block_164 = 16754622181974910496;
        }
        _ => {
            current_block_164 = 16754622181974910496;
        }
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
    1i32
}

fn draw_palette_span(
    dst: &mut [u32],
    pixels: &[u16],
    palette: &[u16; 256],
    brightness: &[u8; 63],
    mask: u16,
) {
    for (dst, &pixel) in dst.iter_mut().zip(pixels) {
        let color = palette[usize::from(pixel & 0xff)];
        *dst = u32::from(brightness[usize::from(color & mask)]) << 16
            | u32::from(brightness[usize::from((color >> 5) & mask)]) << 8
            | u32::from(brightness[usize::from((color >> 10) & mask)]);
    }
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

fn draw_color_math_span(
    dst: &mut [u32],
    main: &[u16],
    sub: &[u16],
    palette: &[u16; 256],
    brightness: &[u8; 63],
    math_brightness: &[u8; 63],
    color_visible: bool,
    flags: u32,
    fixed: u32,
) {
    let mask: u16 = if color_visible { 31 } else { 0 };
    for (index, (dst, &pixel)) in dst.iter_mut().zip(main).enumerate() {
        let color = palette[usize::from(pixel & 0xff)];
        let layer = (pixel >> 8) & 0xf;
        let mut r = u32::from(color & mask);
        let mut g = u32::from((color >> 5) & mask);
        let mut b = u32::from((color >> 10) & mask);
        let mut color_map: &[u8; 63] = brightness;
        if flags & (1 << layer) != 0 {
            let color2 = if flags & 0x100 != 0 {
                let sub_pixel = sub[index] & 0xff;
                if sub_pixel != 0 {
                    color_map = math_brightness;
                    u32::from(palette[usize::from(sub_pixel)])
                } else {
                    fixed
                }
            } else {
                color_map = math_brightness;
                fixed
            };
            let r2 = color2 & 0x1f;
            let g2 = (color2 >> 5) & 0x1f;
            let b2 = (color2 >> 10) & 0x1f;
            if flags & 0x200 != 0 {
                r = r.saturating_sub(r2);
                g = g.saturating_sub(g2);
                b = b.saturating_sub(b2);
            } else {
                r += r2;
                g += g2;
                b += b2;
            }
        }
        *dst = u32::from(color_map[b as usize])
            | u32::from(color_map[g as usize]) << 8
            | u32::from(color_map[r as usize]) << 16;
    }
}
