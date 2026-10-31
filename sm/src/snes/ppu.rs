use crate::initial_state::InitialState;
use crate::rendering as raster;
use ::core::ffi::{c_char, c_void};
use ::core::mem::{size_of, transmute};

use crate::c_library::{memcpy, memset};
pub use crate::types::{Ppu, PpuPixelPrioBufs, PpuZbufType, Snes};
pub const PPU_EXTRA_LEFT_RIGHT: u32 = 0;

pub const __ASSERT_FUNCTION: [c_char; 43] = unsafe {
    transmute::<[u8; 43], [c_char; 43]>(*b"void PpuDrawBackgrounds(Ppu *, int, _Bool)\0")
};
#[inline]
pub unsafe fn ppu_copy(ppu: *mut Ppu, ppu_src: *mut Ppu) {
    unsafe {
        let snes = (*ppu).snes;
        let pitch: usize = (*ppu).renderPitch as usize;
        let buffer = (*ppu).renderBuffer;
        memcpy(
            ppu as *mut c_void,
            ppu_src as *const c_void,
            size_of::<Ppu>() as usize,
        );
        (*ppu).renderBuffer = buffer;
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
pub fn ppu_restore_render_routing(ppu: &mut Ppu) {
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
pub unsafe fn ppu_begin_drawing(ppu: *mut Ppu, pixels: *mut u8, pitch: usize, _render_flags: u32) {
    unsafe {
        (*ppu).renderPitch = pitch as u32;
        (*ppu).renderBuffer = pixels;
    }
}
pub unsafe fn ppu_check_overscan(ppu: *mut Ppu) -> bool {
    unsafe {
        (*ppu).frameOverscan = (*ppu).overscan;
        (*ppu).frameOverscan
    }
}
pub unsafe fn ppu_handle_vblank(ppu: *mut Ppu) {
    unsafe {
        if !(*ppu).forcedBlank {
            (*ppu).oamAdr = (*ppu).oamAdrWritten;
            (*ppu).oamInHigh = (*ppu).oamInHighWritten;
            (*ppu).oamSecondWrite = false;
        }
        (*ppu).frameInterlace = (*ppu).interlace;
    }
}
#[inline]
fn clear_backdrop(buf: &mut PpuPixelPrioBufs) {
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
                .offset((x + PPU_EXTRA_LEFT_RIGHT as i32) as isize))
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
            set_fields! {[(*ppu)]; mosaicStartLine = 1, rangeOver = false,
            timeOver = false, evenFrame = !(*ppu).evenFrame, };
            #[cfg(sm_verify_dll)]
            model_check::begin_frame();
            #[cfg(sm_verify_dll)]
            native_line::begin_frame(ppu);
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
            clear_backdrop(&mut (*ppu).objBuffer);
            (*ppu).lineHasSprites =
                !(*ppu).forcedBlank && ppu_evaluate_sprites(ppu, line - 1i32) as i32 != 0;
            // EXTBG composition and enabled mosaic use the per-pixel path.
            let mosaic = (*ppu).mosaicSize > 1 && (*ppu).bgLayer.iter().any(|bg| bg.mosaicEnabled);
            if *new_ppu && !((*ppu).mode == 7 && (*ppu).m7extBg) && !mosaic {
                ppu_draw_whole_line(ppu, line as u32);
                #[cfg(sm_verify_dll)]
                cross_render::check(ppu, line);
                #[cfg(sm_verify_dll)]
                native_line::check(ppu, line);
            } else {
                if (*ppu).mode as i32 == 7 {
                    ppu_calculate_mode7_starts(ppu, line);
                }
                for x in 0i32..256 {
                    ppu_render_pixel::<false>(ppu, x, line, (0, 0));
                }
            }
        };
    }
}

unsafe fn ppu_draw_whole_line(ppu: *mut Ppu, y: u32) {
    unsafe {
        if (*ppu).forcedBlank {
            let dst = (*ppu)
                .renderBuffer
                .offset(y.wrapping_sub(1).wrapping_mul((*ppu).renderPitch) as isize);
            memset(
                dst as *mut c_void,
                0,
                ((256 + i32::from((*ppu).extraLeftRight) * 2) * 4) as usize,
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

unsafe fn ppu_get_pixel<const NATIVE: bool>(
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

unsafe fn ppu_calculate_mode7_starts(ppu: *mut Ppu, y: i32) {
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

unsafe fn ppu_evaluate_sprites(ppu: *mut Ppu, line: i32) -> bool {
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
        #[cfg(sm_verify_dll)]
        let checked = model_check::before_read(&*ppu, adr);
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
        #[cfg(sm_verify_dll)]
        native_line::read(ppu, adr);
        let value = sm_backend_ppu_read(ppu, adr, h_pos, v_pos, bus);
        #[cfg(sm_verify_dll)]
        if let Some(checked) = checked {
            model_check::after_read(&*ppu, adr, value, checked);
        }
        value
    }
}

pub unsafe fn ppu_write(ppu: *mut Ppu, adr: u8, val: u8) {
    unsafe {
        #[cfg(sm_verify_dll)]
        let checked = model_check::before_write(&*ppu, adr, val);
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
        #[cfg(sm_verify_dll)]
        native_line::write(ppu, adr, val);
        #[cfg(sm_verify_dll)]
        if let Some(checked) = checked {
            model_check::after_write(&*ppu, adr, val, checked);
        }
    }
}

/// The verifier's per-access check of the native register and port models
/// against the emulated PPU: before an access each model is rebuilt from the
/// emulated state and applies it, and afterwards both must agree.
/// `SM_PPU_MODEL_CHECK=N` checks the accesses of every Nth frame (0 or unset
/// disables it), since a check on every access slows DMA-heavy replays.
#[cfg(sm_verify_dll)]
mod model_check {
    use super::{MemoryView, Ppu, Store, ports_of, raster, registers_of};
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU32, Ordering};

    static FRAMES: AtomicU32 = AtomicU32::new(0);

    fn interval() -> u32 {
        static INTERVAL: OnceLock<u32> = OnceLock::new();
        *INTERVAL.get_or_init(|| {
            std::env::var("SM_PPU_MODEL_CHECK")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0)
        })
    }

    pub(super) fn begin_frame() {
        FRAMES.fetch_add(1, Ordering::Relaxed);
    }

    fn sampled() -> bool {
        let every = interval();
        every != 0 && FRAMES.load(Ordering::Relaxed) % every == 0
    }

    pub(super) struct Write {
        registers: raster::registers::Registers,
        ports: raster::memory::Ports,
        store: Option<Store>,
    }

    pub(super) fn before_write(ppu: &Ppu, adr: u8, val: u8) -> Option<Write> {
        if !sampled() {
            return None;
        }
        let mut registers = registers_of(ppu);
        registers.write(adr, val);
        let mut ports = ports_of(ppu);
        let mut view = MemoryView::new(ppu);
        ports.write(&mut view, adr, val);
        Some(Write {
            registers,
            ports,
            store: view.store,
        })
    }

    pub(super) fn after_write(ppu: &Ppu, adr: u8, val: u8, expected: Write) {
        assert_eq!(
            expected.registers,
            registers_of(ppu),
            "native PPU register model differs after ${adr:02X} = ${val:02X}"
        );
        assert_eq!(
            expected.ports,
            ports_of(ppu),
            "native PPU port model differs after ${adr:02X} = ${val:02X}"
        );
        if let Some(store) = expected.store {
            assert_eq!(
                store,
                store.reread(&MemoryView::new(ppu)),
                "native PPU memory store differs after ${adr:02X} = ${val:02X}"
            );
        }
    }

    pub(super) struct Read {
        ports: raster::memory::Ports,
        value: Option<u8>,
    }

    pub(super) fn before_read(ppu: &Ppu, adr: u8) -> Option<Read> {
        if !sampled() {
            return None;
        }
        let mut ports = ports_of(ppu);
        let value = ports.read(&MemoryView::new(ppu), adr, ppu.ppu2openBus);
        Some(Read { ports, value })
    }

    pub(super) fn after_read(ppu: &Ppu, adr: u8, value: u8, expected: Read) {
        assert!(
            expected.value.is_none_or(|expected| expected == value),
            "native PPU port model read differs at ${adr:02X}"
        );
        assert_eq!(
            expected.ports,
            ports_of(ppu),
            "native PPU port model differs after reading ${adr:02X}"
        );
    }
}

/// The emulated PPU's port state in the native renderer's model.
#[cfg(sm_verify_dll)]
fn ports_of(ppu: &Ppu) -> raster::memory::Ports {
    raster::memory::Ports {
        sprite_size: ppu.objSize,
        sprite_tile_words: [ppu.objTileAdr1, ppu.objTileAdr2],
        sprite_priority: ppu.objPriority,
        oam_address: ppu.oamAdr,
        oam_address_written: ppu.oamAdrWritten,
        oam_high: ppu.oamInHigh,
        oam_high_written: ppu.oamInHighWritten,
        oam_second: ppu.oamSecondWrite,
        oam_buffer: ppu.oamBuffer,
        vram_pointer: ppu.vramPointer,
        vram_increment: ppu.vramIncrement,
        vram_remap: ppu.vramRemapMode,
        vram_increment_on_high: ppu.vramIncrementOnHigh,
        vram_read_buffer: ppu.vramReadBuffer,
        cgram_pointer: ppu.cgramPointer,
        cgram_second: ppu.cgramSecondWrite,
        cgram_buffer: ppu.cgramBuffer,
    }
}

/// One memory store the port model made, to compare with the emulated PPU's.
#[cfg(sm_verify_dll)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Store {
    Vram(u16, u16),
    Cgram(u8, u16),
    Oam(u8, u16),
    HighOam(u8, u8),
}

#[cfg(sm_verify_dll)]
impl Store {
    fn reread(self, memory: &impl raster::memory::VideoMemory) -> Self {
        match self {
            Self::Vram(index, _) => Self::Vram(index, memory.vram(index)),
            Self::Cgram(index, _) => Self::Cgram(index, memory.cgram(index)),
            Self::Oam(index, _) => Self::Oam(index, memory.oam(index)),
            Self::HighOam(index, _) => Self::HighOam(index, memory.high_oam(index)),
        }
    }
}

/// The emulated PPU's memory, read in place; the port model's store is recorded
/// rather than applied, so the emulated write stays the only one.
#[cfg(sm_verify_dll)]
struct MemoryView<'a> {
    ppu: &'a Ppu,
    /// A port write makes at most one store.
    store: Option<Store>,
}

#[cfg(sm_verify_dll)]
impl<'a> MemoryView<'a> {
    fn new(ppu: &'a Ppu) -> Self {
        Self { ppu, store: None }
    }
}

#[cfg(sm_verify_dll)]
impl raster::memory::VideoMemory for MemoryView<'_> {
    fn vram(&self, index: u16) -> u16 {
        self.ppu.vram[usize::from(index & 0x7fff)]
    }
    fn set_vram(&mut self, index: u16, value: u16) {
        self.store = Some(Store::Vram(index & 0x7fff, value));
    }
    fn cgram(&self, index: u8) -> u16 {
        self.ppu.cgram[usize::from(index)]
    }
    fn set_cgram(&mut self, index: u8, value: u16) {
        self.store = Some(Store::Cgram(index, value));
    }
    fn oam(&self, index: u8) -> u16 {
        self.ppu.oam[usize::from(index)]
    }
    fn set_oam(&mut self, index: u8, value: u16) {
        self.store = Some(Store::Oam(index, value));
    }
    fn high_oam(&self, index: u8) -> u8 {
        self.ppu.highOam[usize::from(index & 31)]
    }
    fn set_high_oam(&mut self, index: u8, value: u8) {
        self.store = Some(Store::HighOam(index & 31, value));
    }
}

/// The emulated PPU's line registers in the native renderer's model.
#[cfg(sm_verify_dll)]
fn registers_of(ppu: &Ppu) -> raster::registers::Registers {
    use raster::registers::{Layer, Registers};
    Registers {
        mode: ppu.mode,
        forced_blank: ppu.forcedBlank,
        layers: core::array::from_fn(|index| {
            let bg = &ppu.bgLayer[index];
            Layer {
                background: raster::Background {
                    horizontal_scroll: bg.hScroll,
                    vertical_scroll: bg.vScroll,
                    map_words: bg.tilemapAdr,
                    tile_words: bg.tileAdr,
                    wider: bg.tilemapWider,
                    higher: bg.tilemapHigher,
                },
                big_tiles: bg.bigTiles,
                mosaic: bg.mosaicEnabled,
            }
        }),
        mosaic_size: ppu.mosaicSize,
        windows: raster::Windows {
            first: [ppu.window1left, ppu.window1right],
            second: [ppu.window2left, ppu.window2right],
            native_flags: ppu.windowsel,
            layers: core::array::from_fn(|index| {
                let window = &ppu.windowLayer[index];
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
            enabled: ppu.screenEnabled,
            windowed: ppu.screenWindowed,
        },
        affine: raster::Affine {
            matrix: ppu.m7matrix,
            horizontal_flip: ppu.m7xFlip,
            vertical_flip: ppu.m7yFlip,
            large_field: ppu.m7largeField,
            character_fill: ppu.m7charFill,
        },
        color: raster::ColorMath {
            clip: ppu.clipMode,
            prevent: ppu.preventMathMode,
            enabled: ppu.mathEnabled,
            add_subscreen: ppu.addSubscreen,
            subtract: ppu.subtractColor,
            half: ppu.halfColor,
            fixed: [ppu.fixedColorR, ppu.fixedColorG, ppu.fixedColorB],
            brightness: ppu.brightness,
        },
        direct_color: ppu.directColor,
        scroll_latch: ppu.scrollPrev,
        horizontal_scroll_latch: ppu.scrollPrev2,
        affine_latch: ppu.m7prev,
    }
}
pub fn ppu_get_current_render_scale(_ppu: *mut Ppu, _render_flags: u32) -> i32 {
    1
}

unsafe fn ppu_render_pixel<const NATIVE: bool>(ppu: *mut Ppu, x: i32, y: i32, starts: (u32, u32)) {
    unsafe {
        let mut output = [0u8; 4];
        if !(*ppu).forcedBlank {
            let main = ppu_get_pixel::<NATIVE>(ppu, x, y, false, starts);
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
                second = ppu_get_pixel::<NATIVE>(ppu, x, y, true, starts);
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

/// Resync the native line state after a harness snapshot restore.
#[cfg(sm_verify_dll)]
pub(crate) unsafe fn native_line_restored(ppu: *mut Ppu) {
    unsafe { native_line::restored(ppu) }
}

/// Step 3 of the native renderer: a PPU state built only from register and port
/// accesses (`Registers`, `Ports`, `OwnedMemory`), rendering each line with no C
/// PPU state but the beam-timed mosaic start line and modulo. With
/// `SM_NATIVE_LINES` set, each natively drawn line is drawn again from it and
/// compared with the emulated PPU's line; `SM_NATIVE_LINES_LOG` receives the
/// differing lines and each resync. Harness snapshot restores resync it
/// explicitly; any other frame whose start finds the emulated PPU's state
/// differing from the native one is logged as a resync, since it means some
/// path loaded PPU state without its ports.
#[cfg(sm_verify_dll)]
mod native_line {
    use super::{Ppu, ports_of, raster, registers_of};
    use raster::memory::{OwnedMemory, Ports, VideoMemory};
    use raster::registers::Registers;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::io::Write as _;
    use std::sync::OnceLock;

    struct NativePpu {
        registers: Registers,
        ports: Ports,
        memory: OwnedMemory,
        overflow: raster::Overflow,
        frame: u32,
    }

    impl NativePpu {
        fn of(ppu: &Ppu, frame: u32) -> Self {
            Self {
                registers: registers_of(ppu),
                ports: ports_of(ppu),
                memory: OwnedMemory {
                    vram: Box::new(ppu.vram),
                    cgram: ppu.cgram,
                    oam: ppu.oam,
                    high_oam: ppu.highOam,
                },
                overflow: raster::Overflow {
                    range: false,
                    time: false,
                },
                frame,
            }
        }

        /// The first part of the state the emulated PPU no longer shares, if any.
        fn divergence(&self, ppu: &Ppu) -> Option<&'static str> {
            if self.registers != registers_of(ppu) {
                Some("registers")
            } else if self.ports != ports_of(ppu) {
                Some("ports")
            } else if *self.memory.vram != ppu.vram {
                Some("vram")
            } else if self.memory.cgram != ppu.cgram {
                Some("cgram")
            } else if self.memory.oam != ppu.oam || self.memory.high_oam != ppu.highOam {
                Some("oam")
            } else {
                None
            }
        }
    }

    /// The native renderer's memory, with the emulated PPU's alias words past
    /// VRAM (its port fields, then CGRAM) for out-of-range tile reads.
    struct Source<'a>(&'a NativePpu, &'a Ppu);

    impl raster::IndexedSource for Source<'_> {
        fn word(&self, index: usize) -> Result<u16, raster::RasterFault> {
            let (native, ppu) = (self.0, self.1);
            let ports = &native.ports;
            Ok(match index {
                0..=0x7fff => native.memory.vram[index],
                0x8000 => ports.vram_pointer,
                0x8001 => u16::from_ne_bytes([
                    u8::from(ports.vram_increment_on_high),
                    ppu.snapshot_padding0[0],
                ]),
                0x8002 => ports.vram_increment,
                0x8003 => u16::from_ne_bytes([ports.vram_remap, ppu.snapshot_padding1[0]]),
                0x8004 => ports.vram_read_buffer,
                0x8005..=0x801e => native.memory.cgram[index - 0x8005],
                _ => return Err(raster::RasterFault::Word(index)),
            })
        }
        fn palette(&self, index: u8) -> Result<u16, raster::RasterFault> {
            Ok(self.0.memory.cgram(index))
        }
    }

    thread_local! {
        static STATES: RefCell<HashMap<usize, NativePpu>> = RefCell::new(HashMap::new());
    }

    fn enabled() -> bool {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var_os("SM_NATIVE_LINES").is_some())
    }

    fn log(row: std::fmt::Arguments<'_>) {
        if let Some(path) = std::env::var_os("SM_NATIVE_LINES_LOG") {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap();
            file.write_fmt(row).unwrap();
        }
    }

    pub(super) unsafe fn begin_frame(ppu: *mut Ppu) {
        if !enabled() {
            return;
        }
        let ppu = unsafe { &*ppu };
        STATES.with_borrow_mut(|states| {
            let state = states
                .entry(ppu as *const Ppu as usize)
                .or_insert_with(|| NativePpu::of(ppu, 0));
            state.frame += 1;
            state.overflow = raster::Overflow {
                range: false,
                time: false,
            };
            if let Some(part) = state.divergence(ppu) {
                log(format_args!("{},resync,{part}\n", state.frame));
                *state = NativePpu::of(ppu, state.frame);
            }
        });
    }

    /// The harness loaded the emulated PPU from a snapshot without its ports.
    pub(super) unsafe fn restored(ppu: *mut Ppu) {
        if !enabled() {
            return;
        }
        let ppu = unsafe { &*ppu };
        STATES.with_borrow_mut(|states| {
            if let Some(state) = states.get_mut(&(ppu as *const Ppu as usize)) {
                let frame = state.frame;
                *state = NativePpu::of(ppu, frame);
            }
        });
    }

    pub(super) unsafe fn write(ppu: *mut Ppu, address: u8, value: u8) {
        if !enabled() {
            return;
        }
        STATES.with_borrow_mut(|states| {
            if let Some(state) = states.get_mut(&(ppu as usize)) {
                state.registers.write(address, value);
                state.ports.write(&mut state.memory, address, value);
            }
        });
    }

    pub(super) unsafe fn read(ppu: *mut Ppu, address: u8) {
        if !enabled() {
            return;
        }
        let open_bus = unsafe { (*ppu).ppu2openBus };
        STATES.with_borrow_mut(|states| {
            if let Some(state) = states.get_mut(&(ppu as usize)) {
                state.ports.read(&state.memory, address, open_bus);
            }
        });
    }

    pub(super) unsafe fn check(ppu: *mut Ppu, line: i32) {
        if !enabled() {
            return;
        }
        let ppu = unsafe { &*ppu };
        STATES.with_borrow_mut(|states| {
            let Some(state) = states.get_mut(&(ppu as *const Ppu as usize)) else {
                return;
            };
            let registers = state.registers.line();
            let mut sprites = raster::SpriteLine::cleared(state.overflow);
            if !registers.forced_blank {
                let input = raster::SpriteInput {
                    words: &state.memory.oam,
                    extension: &state.memory.high_oam,
                    tile_words: state.ports.sprite_tile_words,
                    size: state.ports.sprite_size,
                    rotation: state
                        .ports
                        .sprite_priority
                        .then_some(state.ports.oam_address),
                    interlace: ppu.objInterlace,
                    even_frame: ppu.evenFrame,
                };
                let (has_tiles, overflow) = raster::evaluate_sprites(
                    &Source(state, ppu),
                    &input,
                    line - 1,
                    &mut sprites.pixels,
                    sprites.overflow,
                )
                .unwrap_or_else(|fault| panic!("native line {line} sprites: {fault:?}"));
                sprites.has_tiles = has_tiles;
                sprites.overflow = overflow;
            }
            let brightness = raster::BrightnessTables::for_level(registers.color.brightness);
            let config = raster::LineConfig {
                mode: registers.mode,
                forced_blank: registers.forced_blank,
                backgrounds: registers.backgrounds,
                windows: registers.windows,
                routing: registers.routing,
                affine: registers.affine,
                mosaic: raster::Mosaic {
                    native_enabled: ppu.mosaicEnabled,
                    background0_enabled: state.registers.layers[0].mosaic,
                    size: state.registers.mosaic_size,
                    start_line: ppu.mosaicStartLine,
                    modulo: &ppu.mosaicModulo,
                    left: 0,
                },
                color: registers.color,
                brightness: brightness.borrowed(),
                has_sprites: sprites.has_tiles,
            };
            let mut pixels = [[0; 4]; raster::WIDTH];
            raster::render_line(&Source(state, ppu), &config, &sprites, line, &mut pixels)
                .unwrap_or_else(|fault| panic!("native line {line}: {fault:?}"));
            state.overflow = sprites.overflow;
            let start = ((line - 1) as u32).wrapping_mul(ppu.renderPitch) as usize
                + usize::from(ppu.extraLeftRight) * 4;
            let row = unsafe { core::slice::from_raw_parts(ppu.renderBuffer.add(start), 256 * 4) };
            let mut differing = (0..256).filter(|&x| pixels[x][..] != row[x * 4..x * 4 + 4]);
            if let Some(first) = differing.next() {
                let count = 1 + differing.count();
                log(format_args!(
                    "{},{line},{},{count},{first}\n",
                    state.frame, registers.mode
                ));
            }
        });
    }
}

/// An independent check of the native line rasterizer: on sampled frames each
/// natively drawn line is drawn again by LakeSnes's per-pixel layer selection
/// (`sm_backend_ppu_pixel`), and differing pixels are logged. Colour math,
/// windows and brightness are shared, so this covers layer and priority
/// selection, Mode 7 sampling and sprite composition, not the shading.
///
/// `SM_CROSS_RENDER=N` samples every Nth frame (0 or unset disables it) and
/// `SM_CROSS_RENDER_LOG` names the CSV of differing lines.
#[cfg(sm_verify_dll)]
mod cross_render {
    use super::{Ppu, ppu_calculate_mode7_starts, ppu_render_pixel};
    use std::io::Write as _;
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU32, Ordering};

    static FRAMES: AtomicU32 = AtomicU32::new(0);

    fn interval() -> u32 {
        static INTERVAL: OnceLock<u32> = OnceLock::new();
        *INTERVAL.get_or_init(|| {
            std::env::var("SM_CROSS_RENDER")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0)
        })
    }

    pub(super) unsafe fn check(ppu: *mut Ppu, line: i32) {
        let every = interval();
        if every == 0 {
            return;
        }
        if line == 1 {
            FRAMES.fetch_add(1, Ordering::Relaxed);
        }
        let frame = FRAMES.load(Ordering::Relaxed);
        if frame % every != 0 || unsafe { (*ppu).forcedBlank } {
            return;
        }
        unsafe {
            let start = ((line - 1) as u32).wrapping_mul((*ppu).renderPitch) as usize
                + usize::from((*ppu).extraLeftRight) * 4;
            let row = core::slice::from_raw_parts_mut((*ppu).renderBuffer.add(start), 256 * 4);
            let native = row.to_vec();
            if (*ppu).mode == 7 {
                ppu_calculate_mode7_starts(ppu, line);
            }
            for x in 0..256 {
                ppu_render_pixel::<false>(ppu, x, line, (0, 0));
            }
            let row = core::slice::from_raw_parts_mut((*ppu).renderBuffer.add(start), 256 * 4);
            let differing: Vec<usize> = (0..256)
                .filter(|&x| native[x * 4..x * 4 + 4] != row[x * 4..x * 4 + 4])
                .collect();
            if let (Some(&first), Some(path)) =
                (differing.first(), std::env::var_os("SM_CROSS_RENDER_LOG"))
            {
                let mut log = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .unwrap();
                writeln!(
                    log,
                    "{frame},{line},{},{},{first},{:02x?},{:02x?}",
                    (*ppu).mode,
                    differing.len(),
                    &native[first * 4..first * 4 + 4],
                    &row[first * 4..first * 4 + 4],
                )
                .unwrap();
            }
            row.copy_from_slice(&native);
        }
    }
}

const _: () = {
    assert!(core::mem::offset_of!(Ppu, vram) == 8);
    assert!(core::mem::offset_of!(Ppu, bgLayer) == 67142);
    assert!(core::mem::offset_of!(Ppu, windowLayer) == 67244);
    assert!(core::mem::offset_of!(Ppu, pixelbuffer_placeholder) == 67313);
};
