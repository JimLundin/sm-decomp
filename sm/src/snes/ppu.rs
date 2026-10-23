use crate::game_state::Game;
use crate::initial_state::InitialState;
use ::core::ffi::{c_char, c_void};
use ::core::mem::{size_of, transmute};

use crate::c_library::{__assert_fail, memcpy, memset};
pub use crate::types::{Ppu, PpuPixelPrioBufs, PpuZbufType, Snes, size_t};
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
    if a < b { a } else { b }
}
#[inline]
unsafe fn IntMax(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}
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
#[forbid(unsafe_code)]
pub fn ppu_init(snes: *mut Snes) -> *mut Ppu {
    Box::into_raw(Box::new(Ppu {
        snes,
        ..Ppu::INITIAL
    }))
}
pub unsafe fn ppu_copy(ppu: *mut Ppu, ppu_src: *mut Ppu) {
    unsafe {
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
}
pub unsafe fn ppu_reset(ppu: *mut Ppu) {
    unsafe {
        let snes = (*ppu).snes;
        let pitch = (*ppu).renderPitch;
        let render_buffer = (*ppu).renderBuffer;
        // Reset also clears saved padding and renderer caches.
        *ppu = Ppu {
            snes,
            renderBuffer: render_buffer,
            renderPitch: pitch,
            ..Ppu::INITIAL
        };
        set_fields! {[(*ppu)], 1; vramIncrement, mosaicSize, mosaicStartLine}
        (*ppu).forcedBlank = true;
    }
}

pub unsafe fn ppu_saveload(ppu: *mut Ppu, io: &mut crate::snapshot_io::SnapshotIo<'_>) {
    unsafe {
        crate::rtl::snapshot_result(io.state(&mut *ppu));
    }
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
    unsafe {
        (*ppu).renderPitch = pitch as u32;
        (*ppu).renderBuffer = pixels;
    }
}
pub unsafe fn ppu_checkOverscan(ppu: *mut Ppu) -> bool {
    unsafe {
        (*ppu).frameOverscan = (*ppu).overscan;
        (*ppu).frameOverscan
    }
}
pub unsafe fn ppu_handleVblank(ppu: *mut Ppu) {
    unsafe {
        if !(*ppu).forcedBlank {
            (*ppu).oamAdr = (*ppu).oamAdrWritten;
            (*ppu).oamInHigh = (*ppu).oamInHighWritten;
            (*ppu).oamSecondWrite = false_0 != 0;
        }
        (*ppu).frameInterlace = (*ppu).interlace;
    }
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
    unsafe {
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
    unsafe {
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
}

unsafe fn PpuDrawWholeLine(ppu: *mut Ppu, y: u32) {
    unsafe {
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
}

unsafe fn ppu_getPixel<const NATIVE: bool>(
    ppu: *mut Ppu,
    x: i32,
    y: i32,
    sub: bool,
    starts: (u32, u32),
) -> (i32, [i32; 3], bool) {
    unsafe {
        if NATIVE {
            return ppu_native_pixel(ppu, x, y, sub, starts);
        }
        // The pixel loop supplies x in 0..256. Keep invalid-mode failures in Rust.
        assert!((*ppu).mode < 10);
        let sprite = (*ppu).objBuffer.data[x as usize];
        let mut color = [0; 3];
        let layer = sm_backend_ppu_pixel(ppu, x, y, sub, sprite, color.as_mut_ptr());
        (layer, color, layer != 5)
    }
}
unsafe fn ppu_getPixelForBgLayer(ppu: *mut Ppu, x: i32, y: i32, layer: i32) -> i32 {
    unsafe {
        let bg = &(*ppu).bgLayer[layer as usize];
        let mut address = i32::from(bg.tilemapAdr) + ((y >> 3 & 31) << 5);
        if x & 256 != 0 && bg.tilemapWider {
            address += 0x400;
        }
        if y & 256 != 0 && bg.tilemapHigher {
            address += if bg.tilemapWider { 0x800 } else { 0x400 };
        }
        // Native addressing wraps the base before adding the column/plane offsets.
        let tile = *(*ppu)
            .vram
            .as_ptr()
            .offset(((address & 0x7fff) + (x >> 3 & 31)) as isize);
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
        let address = (i32::from(bg.tileAdr) + i32::from(tile & 0x3ff) * 4 * depth + row) & 0x7fff;
        let mut pixel = 0;
        for pair in 0..depth / 2 {
            let word = *(*ppu).vram.as_ptr().offset((address + pair * 8) as isize);
            pixel |= ((i32::from(word) >> column & 1) | (i32::from(word) >> (column + 7) & 2))
                << (pair * 2);
        }
        if pixel == 0 {
            return 0;
        }
        let color = (1 << depth) * (i32::from(tile & 0x1c00) >> 10) + pixel;
        let priorities = [[0x8000, 0xc000], [0x7100, 0xb100], [0x1200, 0xf200]];
        priorities[layer as usize][usize::from(tile & 0x2000 != 0)] + color
    }
}
unsafe fn ppu_calculateMode7Starts(ppu: *mut Ppu, y: i32) {
    unsafe {
        let (x, y) = ppu_mode7_start::<false>(ppu, y);
        (*ppu).m7startX = x as i32;
        (*ppu).m7startY = y as i32;
    }
}
unsafe fn ppu_evaluateSprites(ppu: *mut Ppu, line: i32) -> bool {
    unsafe {
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
            let spriteSize: i32 =
                spriteSizes[(*ppu).objSize as usize][((*ppu).highOam[(index >> 3) as usize] as i32
                    >> (index as i32 & 7) + 1
                    & 1) as usize];
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
                            row = (row as i32 * 2
                                + (if (*ppu).evenFrame as i32 != 0 { 0 } else { 1 }))
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
                                    let usedTile: i32 = ((oam1 & 0xff) >> 4) + (row as i32 >> 3)
                                        << 4
                                        | (oam1 & 0xf) + (usedCol >> 3) & 0xf;
                                    let addr = (&raw mut (*ppu).vram as *mut u16).offset(
                                        (objAdr + usedTile * 16 + (row as i32 & 0x7) & 0x7fff)
                                            as isize,
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
}
// Registers and legacy pixel selection use upstream; native rendering stays local.
#[link(name = "lakesnes_backend", kind = "static", modifiers = "-bundle")]
unsafe extern "C" {
    fn sm_backend_ppu_read(ppu: *mut Ppu, adr: u8, h_pos: u16, v_pos: u16, bus: u8) -> u8;
    fn sm_backend_ppu_write(ppu: *mut Ppu, adr: u8, value: u8);
    fn sm_backend_ppu_pixel(
        ppu: *mut Ppu,
        x: i32,
        y: i32,
        sub: bool,
        sprite: u16,
        color: *mut i32,
    ) -> i32;
}

pub unsafe fn ppu_read(ppu: *mut Ppu, adr: u8) -> u8 {
    unsafe {
        // Only open-bus reads and counter latching access the host SNES pointer.
        let (h_pos, v_pos, bus) = match adr {
            0x37 => (
                (*(*ppu).snes).hPos,
                (*(*ppu).snes).vPos,
                (*(*ppu).snes).openBus,
            ),
            4
            | 20
            | 36
            | 5
            | 21
            | 37
            | 6
            | 22
            | 38
            | 8
            | 24
            | 40
            | 9
            | 25
            | 41
            | 10
            | 26
            | 42
            | 0x34..=0x36
            | 0x38..=0x3f => (0, 0, 0),
            _ => (0, 0, (*(*ppu).snes).openBus),
        };
        sm_backend_ppu_read(ppu, adr, h_pos, v_pos, bus)
    }
}

pub unsafe fn ppu_write(ppu: *mut Ppu, adr: u8, val: u8) {
    unsafe {
        // The native renderer caches complete register bytes, including unused bits.
        match adr {
            0x23..=0x25 => {
                let shift = u32::from(adr - 0x23) * 8;
                (*ppu).windowsel =
                    ((*ppu).windowsel & !(0xff << shift)) | (u32::from(val) << shift);
            }
            0x2c..=0x2d => (*ppu).screenEnabled[usize::from(adr - 0x2c)] = val,
            0x2e..=0x2f => (*ppu).screenWindowed[usize::from(adr - 0x2e)] = val,
            _ => {}
        }
        sm_backend_ppu_write(ppu, adr, val);
    }
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
    unsafe {
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
}

unsafe fn ppu_native_layer_enabled(ppu: *mut Ppu, layer: i32, x: i32, sub: bool) -> bool {
    unsafe {
        (*ppu).screenEnabled[sub as usize] & (1 << layer) != 0
            && (layer != 2 || (0..256).contains(&x))
            && ((*ppu).screenWindowed[sub as usize] & (1 << layer) == 0
                || !ppu_window_state::<true>(ppu, layer, x))
    }
}

unsafe fn ppu_mode7_start<const NATIVE: bool>(ppu: *mut Ppu, mut y: i32) -> (u32, u32) {
    unsafe {
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
}

unsafe fn ppu_native_mode7_x(ppu: *mut Ppu, x: i32, sub: bool) -> i32 {
    unsafe {
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
}

unsafe fn ppu_native_pixel(
    ppu: *mut Ppu,
    x: i32,
    y: i32,
    sub: bool,
    starts: (u32, u32),
) -> (i32, [i32; 3], bool) {
    unsafe {
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
                    let pixel = ppu_getPixelForBgLayer(
                        ppu,
                        x + i32::from(bg.hScroll),
                        y + i32::from(bg.vScroll),
                        layer,
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
}

unsafe fn ppu_render_pixel<const NATIVE: bool>(ppu: *mut Ppu, x: i32, y: i32, starts: (u32, u32)) {
    unsafe {
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
            let fixed_word =
                u32::from(fixed[0]) | u32::from(fixed[1]) << 5 | u32::from(fixed[2]) << 10;
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
}

const _: () = {
    assert!(core::mem::offset_of!(Ppu, vram) == 8);
    assert!(core::mem::offset_of!(Ppu, bgLayer) == 67142);
    assert!(core::mem::offset_of!(Ppu, windowLayer) == 67244);
    assert!(core::mem::offset_of!(Ppu, pixelbuffer_placeholder) == 67313);
};
