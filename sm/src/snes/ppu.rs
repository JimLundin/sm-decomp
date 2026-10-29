use crate::initial_state::InitialState;
use crate::rendering as raster;
use ::core::ffi::{c_char, c_void};
use ::core::mem::{size_of, transmute};

use crate::c_library::{memcpy, memset};
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
pub unsafe fn PpuBeginDrawing(ppu: *mut Ppu, pixels: *mut u8, pitch: size_t, _render_flags: u32) {
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

// This private source lives only inside the unsafe native scanline interval.
// Word reads stay in VRAM, its 31 native alias words, or CGRAM; signed sprite
// reads retain their original selected allocation address.
struct NativeRasterSource(*mut Ppu);

impl raster::IndexedSource for NativeRasterSource {
    fn word(&self, index: usize) -> Result<u16, raster::RasterFault> {
        unsafe {
            match index {
                0..=0x7fff => Ok((*self.0).vram[index]),
                0x8000 => Ok((*self.0).vramPointer),
                0x8001 => Ok(u16::from_ne_bytes([
                    u8::from((*self.0).vramIncrementOnHigh),
                    (*self.0).snapshot_padding0[0],
                ])),
                0x8002 => Ok((*self.0).vramIncrement),
                0x8003 => Ok(u16::from_ne_bytes([
                    (*self.0).vramRemapMode,
                    (*self.0).snapshot_padding1[0],
                ])),
                0x8004 => Ok((*self.0).vramReadBuffer),
                0x8005..=0x801e => Ok((*self.0).cgram[index - 0x8005]),
                _ => Err(raster::RasterFault::Word(index)),
            }
        }
    }

    fn palette(&self, index: u8) -> Result<u16, raster::RasterFault> {
        unsafe { Ok((*self.0).cgram[usize::from(index)]) }
    }
}

impl raster::SpritePixels for NativeRasterSource {
    fn pixel(&self, x: i32) -> Result<u16, raster::RasterFault> {
        // native_pixel calls this only after the shared OBJ/window predicate.
        unsafe {
            Ok(*(*self.0)
                .objBuffer
                .data
                .as_ptr()
                .offset((x + kPpuExtraLeftRight as i32) as isize))
        }
    }
}

fn ppu_raster_result<T>(result: Result<T, raster::RasterFault>) -> T {
    result.unwrap_or_else(|fault| panic!("PPU raster fault: {fault:?}"))
}

// Borrow only the named cache/modulo arrays for one read phase. In particular,
// this does not form a shared Ppu borrow beside native aliases or pixel writes.
unsafe fn ppu_line_config<'a>(ppu: *mut Ppu) -> raster::LineConfig<'a> {
    unsafe {
        raster::LineConfig {
            mode: (*ppu).mode,
            forced_blank: (*ppu).forcedBlank,
            backgrounds: core::array::from_fn(|index| {
                let bg = &(*ppu).bgLayer[index];
                raster::Background {
                    horizontal_scroll: bg.hScroll,
                    vertical_scroll: bg.vScroll,
                    map_words: bg.tilemapAdr,
                    tile_words: bg.tileAdr,
                    wider: bg.tilemapWider,
                    higher: bg.tilemapHigher,
                }
            }),
            windows: raster::Windows {
                first: [(*ppu).window1left, (*ppu).window1right],
                second: [(*ppu).window2left, (*ppu).window2right],
                native_flags: (*ppu).windowsel,
                layers: core::array::from_fn(|index| {
                    let window = &(*ppu).windowLayer[index];
                    raster::Window {
                        first_enabled: window.window1enabled,
                        first_inverted: window.window1inversed,
                        second_enabled: window.window2enabled,
                        second_inverted: window.window2inversed,
                        logic: window.maskLogic,
                    }
                }),
            },
            routing: raster::Routing {
                enabled: (*ppu).screenEnabled,
                windowed: (*ppu).screenWindowed,
            },
            affine: raster::Affine {
                matrix: (*ppu).m7matrix,
                horizontal_flip: (*ppu).m7xFlip,
                vertical_flip: (*ppu).m7yFlip,
                large_field: (*ppu).m7largeField,
                character_fill: (*ppu).m7charFill,
            },
            mosaic: raster::Mosaic {
                native_enabled: (*ppu).mosaicEnabled,
                background0_enabled: (*ppu).bgLayer[0].mosaicEnabled,
                size: (*ppu).mosaicSize,
                start_line: (*ppu).mosaicStartLine,
                modulo: &(*ppu).mosaicModulo,
                left: -i32::from((*ppu).extraLeftCur),
            },
            color: raster::ColorMath {
                clip: (*ppu).clipMode,
                prevent: (*ppu).preventMathMode,
                enabled: (*ppu).mathEnabled,
                add_subscreen: (*ppu).addSubscreen,
                subtract: (*ppu).subtractColor,
                half: (*ppu).halfColor,
                fixed: [(*ppu).fixedColorR, (*ppu).fixedColorG, (*ppu).fixedColorB],
                brightness: (*ppu).brightness,
            },
            brightness: raster::Brightness {
                normal: &(*ppu).brightnessMult,
                half: &(*ppu).brightnessMultHalf,
            },
            has_sprites: (*ppu).lineHasSprites,
        }
    }
}

pub(crate) unsafe fn ppu_run_line(ppu: *mut Ppu, line: i32, new_ppu: &bool) {
    unsafe {
        if line == 0 {
            set_fields! {[(*ppu)]; mosaicStartLine = 1, rangeOver = false_0 != 0,
            timeOver = false_0 != 0, evenFrame = !(*ppu).evenFrame, };
        } else {
            if (*ppu).brightness != (*ppu).lastBrightnessMult {
                let brightness = (*ppu).brightness;
                (*ppu).lastBrightnessMult = brightness;
                raster::update_brightness(
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
            if *new_ppu && !((*ppu).mode == 7 && (*ppu).m7extBg) && !mosaic {
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
            let config = ppu_line_config(ppu);
            ppu_raster_result(raster::affine_start(
                &config.affine,
                &config.mosaic,
                y as i32,
                raster::Quantization::Native,
            ))
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
) -> raster::LayerPixel {
    unsafe {
        if NATIVE {
            let source = NativeRasterSource(ppu);
            let config = ppu_line_config(ppu);
            return ppu_raster_result(raster::native_pixel(
                &source, &config, &source, x, y, sub, starts,
            ));
        }
        // Keep the reference backend query and its invalid-mode failure point.
        assert!((*ppu).mode < 10);
        let sprite = (*ppu).objBuffer.data[x as usize];
        let mut color = [0; 3];
        let layer = sm_backend_ppu_pixel(ppu, x, y, sub, sprite, color.as_mut_ptr());
        raster::LayerPixel {
            layer,
            color,
            non_backdrop: layer != 5,
        }
    }
}

unsafe fn ppu_calculateMode7Starts(ppu: *mut Ppu, y: i32) {
    unsafe {
        let (x, y) = {
            let config = ppu_line_config(ppu);
            ppu_raster_result(raster::affine_start(
                &config.affine,
                &config.mosaic,
                y,
                raster::Quantization::Reference,
            ))
        };
        (*ppu).m7startX = x as i32;
        (*ppu).m7startY = y as i32;
    }
}

unsafe fn ppu_evaluateSprites(ppu: *mut Ppu, line: i32) -> bool {
    unsafe {
        let overflow = raster::Overflow {
            range: (*ppu).rangeOver,
            time: (*ppu).timeOver,
        };
        let result = {
            let input = raster::SpriteInput {
                words: &(*ppu).oam,
                extension: &(*ppu).highOam,
                tile_words: [(*ppu).objTileAdr1, (*ppu).objTileAdr2],
                size: (*ppu).objSize,
                rotation: (*ppu).objPriority.then(|| (*ppu).oamAdr),
                interlace: (*ppu).objInterlace,
                even_frame: (*ppu).evenFrame,
            };
            // Only this output field is mutably borrowed. The lazy source reads
            // VRAM/its tail/CGRAM; immutable OAM fields are also disjoint.
            raster::evaluate_sprites(
                &NativeRasterSource(ppu),
                &input,
                line,
                &mut (*ppu).objBuffer.data,
                overflow,
            )
        };
        let (has_tiles, overflow) = ppu_raster_result(result);
        // A trip flag exits the evaluator immediately, with no later failure.
        (*ppu).rangeOver = overflow.range;
        (*ppu).timeOver = overflow.time;
        has_tiles
    }
}

// Registers and legacy pixel selection use upstream; native rendering stays local.
// Ppu offsets are asserted below and in backend.c. The Rust Snes pointer stays
// opaque: register calls substitute a C view, and pixel lookup does not follow it.
#[link(name = "lakesnes_backend", kind = "static", modifiers = "-bundle")]
unsafe extern "C" {
    #[allow(improper_ctypes)]
    fn sm_backend_ppu_read(ppu: *mut Ppu, adr: u8, h_pos: u16, v_pos: u16, bus: u8) -> u8;
    #[allow(improper_ctypes)]
    fn sm_backend_ppu_write(ppu: *mut Ppu, adr: u8, value: u8);
    #[allow(improper_ctypes)]
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
pub fn PpuGetCurrentRenderScale(_ppu: *mut Ppu, _render_flags: u32) -> i32 {
    1
}

unsafe fn ppu_render_pixel<const NATIVE: bool>(ppu: *mut Ppu, x: i32, y: i32, starts: (u32, u32)) {
    unsafe {
        let mut output = [0u8; 4];
        if !(*ppu).forcedBlank {
            let main = ppu_getPixel::<NATIVE>(ppu, x, y, false, starts);
            let policy = if NATIVE {
                raster::Quantization::Native
            } else {
                raster::Quantization::Reference
            };
            let (window, mut math) = {
                let config = ppu_line_config(ppu);
                (
                    raster::window_contains(&config.windows, 5, x, policy),
                    config.color,
                )
            };
            let mut second = raster::LayerPixel {
                layer: 5,
                color: [0; 3],
                non_backdrop: false,
            };
            if ppu_raster_result(raster::needs_subscreen(&math, main, window, false))
                || !NATIVE && ((*ppu).pseudoHires || (*ppu).mode == 5 || (*ppu).mode == 6)
            {
                second = ppu_getPixel::<NATIVE>(ppu, x, y, true, starts);
            }
            // These inputs were read after the optional backend sub query.
            math.add_subscreen = (*ppu).addSubscreen;
            math.half = (*ppu).halfColor;
            math.fixed = [(*ppu).fixedColorR, (*ppu).fixedColorG, (*ppu).fixedColorB];
            math.subtract = (*ppu).subtractColor;
            math.brightness = (*ppu).brightness;
            output = {
                let tables = raster::Brightness {
                    normal: &(*ppu).brightnessMult,
                    half: &(*ppu).brightnessMultHalf,
                };
                ppu_raster_result(raster::shade_pixel(
                    &math, tables, main, window, second, policy,
                ))
            };
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
