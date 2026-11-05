//! The emulated PPU's line renderer: the oracle's video and the verifier's
//! line checks. The engine's own frames are drawn by `sm_runtime::rendering`
//! (D9); this path stays with the emulator on the oracle side.
use crate::c_library::memset;
use crate::rendering as raster;
#[cfg(sm_verify_dll)]
use crate::snes::ppu::{MemoryView, Store, ports_of, registers_of};
use crate::snes::ppu::{Ppu, PpuPixelPrioBufs};
use ::core::ffi::{c_char, c_void};
use ::core::mem::transmute;

pub const PPU_EXTRA_LEFT_RIGHT: u32 = 0;

pub const __ASSERT_FUNCTION: [c_char; 43] = unsafe {
    transmute::<[u8; 43], [c_char; 43]>(*b"void PpuDrawBackgrounds(Ppu *, int, _Bool)\0")
};

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

pub unsafe fn ppu_run_line(ppu: *mut Ppu, line: i32, new_ppu: &bool) {
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
            let draw = pixels_observed();
            if *new_ppu && !((*ppu).mode == 7 && (*ppu).m7extBg) && !mosaic {
                ppu_draw_whole_line(ppu, line as u32, draw);
                if draw {
                    #[cfg(sm_verify_dll)]
                    cross_render::check(ppu, line);
                    #[cfg(sm_verify_dll)]
                    native_line::check(ppu, line);
                }
            } else {
                if (*ppu).mode as i32 == 7 {
                    ppu_calculate_mode7_starts(ppu, line);
                }
                if draw {
                    for x in 0i32..256 {
                        ppu_render_pixel(ppu, x, line);
                    }
                }
            }
        };
    }
}

/// The native line. Without `draw` it keeps the line's assertions and start
/// faults and writes no pixels.
unsafe fn ppu_draw_whole_line(ppu: *mut Ppu, y: u32, draw: bool) {
    unsafe {
        if (*ppu).forcedBlank {
            if !draw {
                return;
            }
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
        // No PPU field changes within a line, so one configuration serves
        // every pixel; only the separate render buffer is written.
        let source = NativeRasterSource(ppu);
        let config = ppu_line_config(ppu);
        let starts = if (*ppu).mode == 1 {
            (0, 0)
        } else {
            ppu_raster_result(raster::affine_start(
                &config.affine,
                &config.mosaic,
                y as i32,
                raster::Quantization::Native,
            ))
        };
        if !draw {
            return;
        }
        let row = y.wrapping_sub(1).wrapping_mul((*ppu).renderPitch);
        let left_right = i32::from((*ppu).extraLeftRight);
        for x in -i32::from((*ppu).extraLeftCur)..256 + i32::from((*ppu).extraRightCur) {
            let output = ppu_raster_result(raster::render_pixel(
                &source, &config, &source, x, y as i32, starts,
            ));
            let offset = row.wrapping_add(((x + left_right) * 4) as u32);
            core::ptr::copy_nonoverlapping(
                output.as_ptr(),
                (*ppu).renderBuffer.offset(offset as isize),
                4,
            );
        }
    }
}

/// The reference backend's pixel; the native path is `raster::render_pixel`.
unsafe fn ppu_get_pixel(ppu: *mut Ppu, x: i32, y: i32, sub: bool) -> raster::LayerPixel {
    unsafe {
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

#[link(name = "lakesnes_backend", kind = "static", modifiers = "-bundle")]
unsafe extern "C" {
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

thread_local! {
    /// Whether anything reads this thread's rendered frames.
    static PIXELS_OBSERVED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Say whether anything reads the rendered frames; on by default. With it off,
/// each line keeps its register, brightness, sprite, Mode 7 start and fault
/// effects and composes no pixels, so the render buffers keep what the host
/// last put there.
pub fn observe_pixels(observed: bool) {
    PIXELS_OBSERVED.set(observed);
}

/// The verifier's line checks compare rendered pixels, so they keep them.
fn pixels_observed() -> bool {
    #[cfg(sm_verify_dll)]
    if cross_render::interval() != 0 || native_line::enabled() {
        return true;
    }
    PIXELS_OBSERVED.get()
}

/// Run `frame` as the engine's frame. Under `SM_NATIVE_LINES` the lines it draws
/// are the native renderer's (D9: the replay's video output); an oracle frame's
/// stay the emulated PPU's, which the native lines are compared with.
pub fn engine_frame<R>(frame: impl FnOnce() -> R) -> R {
    #[cfg(sm_verify_dll)]
    {
        struct Restore(bool);
        impl Drop for Restore {
            fn drop(&mut self) {
                native_line::set_engine_frame(self.0);
            }
        }
        let _restore = Restore(native_line::set_engine_frame(true));
        frame()
    }
    #[cfg(not(sm_verify_dll))]
    frame()
}

/// The verifier's per-access check of the native register and port models
/// against the emulated PPU: before an access each model is rebuilt from the
/// emulated state and applies it, and afterwards both must agree.
/// `SM_PPU_MODEL_CHECK=N` checks the accesses of every Nth frame (0 or unset
/// disables it), since a check on every access slows DMA-heavy replays.
#[cfg(sm_verify_dll)]
pub(crate) mod model_check {
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

    pub(crate) fn begin_frame() {
        FRAMES.fetch_add(1, Ordering::Relaxed);
    }

    fn sampled() -> bool {
        let every = interval();
        every != 0 && FRAMES.load(Ordering::Relaxed) % every == 0
    }

    pub(crate) struct Write {
        registers: raster::registers::Registers,
        ports: raster::memory::Ports,
        store: Option<Store>,
    }

    pub(crate) fn before_write(ppu: &Ppu, adr: u8, val: u8) -> Option<Write> {
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

    pub(crate) fn after_write(ppu: &Ppu, adr: u8, val: u8, expected: Write) {
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

    pub(crate) struct Read {
        ports: raster::memory::Ports,
        value: Option<u8>,
    }

    pub(crate) fn before_read(ppu: &Ppu, adr: u8) -> Option<Read> {
        if !sampled() {
            return None;
        }
        let mut ports = ports_of(ppu);
        let value = ports.read(&MemoryView::new(ppu), adr, ppu.ppu2openBus);
        Some(Read { ports, value })
    }

    pub(crate) fn after_read(ppu: &Ppu, adr: u8, value: u8, expected: Read) {
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

/// The reference backend's pixel at `x`, written to the render buffer.
unsafe fn ppu_render_pixel(ppu: *mut Ppu, x: i32, y: i32) {
    unsafe {
        let mut output = [0u8; 4];
        if !(*ppu).forcedBlank {
            let main = ppu_get_pixel(ppu, x, y, false);
            let policy = raster::Quantization::Reference;
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
                || (*ppu).pseudoHires
                || (*ppu).mode == 5
                || (*ppu).mode == 6
            {
                second = ppu_get_pixel(ppu, x, y, true);
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
/// accesses (`Registers`, `Ports`, `OwnedMemory`) and the frame count, rendering
/// each line with no emulated PPU or beam state. With
/// `SM_NATIVE_LINES` set, each natively drawn line is drawn again from it and
/// compared with the emulated PPU's line, and on the engine's frame
/// (`engine_frame`) the native line replaces it (the replay's video output, D9); `SM_NATIVE_LINES_LOG` receives the
/// differing lines and each resync. Harness snapshot restores resync it
/// explicitly; any other frame whose start finds the emulated PPU's state
/// differing from the native one is logged as a resync, since it means some
/// path loaded PPU state without its ports.
///
/// Step 4's typed producers feed it through typed calls in place of their port
/// writes, which still reach the emulated PPU; after each producer the two
/// PPUs' ports and memory must agree, and a differing producer is logged
/// (frame, `typed`, producer, part) and the native ports and memory resynced.
#[cfg(sm_verify_dll)]
pub(crate) mod native_line {
    use super::{Ppu, ports_of, raster, registers_of};
    use raster::memory::{OwnedMemory, Ports, VideoMemory};
    use raster::registers::Registers;
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use std::io::Write as _;
    use std::sync::OnceLock;

    fn memory_of(ppu: &Ppu) -> OwnedMemory {
        OwnedMemory {
            vram: Box::new(ppu.vram),
            cgram: ppu.cgram,
            oam: ppu.oam,
            high_oam: ppu.highOam,
        }
    }

    struct NativePpu {
        registers: Registers,
        ports: Ports,
        memory: OwnedMemory,
        overflow: raster::Overflow,
        even_frame: bool,
        frame: u32,
    }

    impl NativePpu {
        fn of(ppu: &Ppu, frame: u32) -> Self {
            Self {
                registers: registers_of(ppu),
                ports: ports_of(ppu),
                memory: memory_of(ppu),
                overflow: raster::Overflow {
                    range: false,
                    time: false,
                },
                even_frame: ppu.evenFrame,
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
            } else if self.even_frame != ppu.evenFrame {
                Some("frame parity")
            } else if ppu.mosaicEnabled != 0 || ppu.mosaicModulo != [0; 256] {
                Some("mosaic cache")
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
        /// The PPU whose port writes a typed producer is making, if any.
        static PRODUCING: Cell<Option<usize>> = const { Cell::new(None) };
        /// Whether the lines drawn now are the engine's frame (`engine_frame`).
        static ENGINE_FRAME: Cell<bool> = const { Cell::new(false) };
    }

    /// Mark the lines drawn from here on as the engine's frame, or not; the
    /// previous mark.
    pub(crate) fn set_engine_frame(engine: bool) -> bool {
        ENGINE_FRAME.replace(engine)
    }

    /// Port accesses the native PPU takes: none of a typed producer's.
    fn feeds(ppu: *mut Ppu) -> bool {
        enabled() && PRODUCING.get() != Some(ppu as usize)
    }

    pub(crate) fn producing(ppu: *mut Ppu) {
        if enabled() {
            PRODUCING.set(Some(ppu as usize));
        }
    }

    pub(crate) fn produced(
        ppu: *mut Ppu,
        producer: &'static str,
        native: impl FnOnce(&mut Ports, &mut OwnedMemory),
    ) {
        if !enabled() {
            return;
        }
        PRODUCING.set(None);
        let ppu = unsafe { &*ppu };
        STATES.with_borrow_mut(|states| {
            let Some(state) = states.get_mut(&(ppu as *const Ppu as usize)) else {
                return;
            };
            native(&mut state.ports, &mut state.memory);
            let part = if state.ports != ports_of(ppu) {
                "ports"
            } else if *state.memory.vram != ppu.vram {
                "vram"
            } else if state.memory.cgram != ppu.cgram {
                "cgram"
            } else if state.memory.oam != ppu.oam || state.memory.high_oam != ppu.highOam {
                "oam"
            } else {
                return;
            };
            log(format_args!(
                "{},typed,{producer},{part}
",
                state.frame
            ));
            state.ports = ports_of(ppu);
            state.memory = memory_of(ppu);
        });
    }

    pub(crate) fn enabled() -> bool {
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

    pub(crate) unsafe fn begin_frame(ppu: *mut Ppu) {
        if !enabled() {
            return;
        }
        let ppu = unsafe { &*ppu };
        STATES.with_borrow_mut(|states| {
            let key = ppu as *const Ppu as usize;
            // A PPU seen for the first time starts from the frame it is at.
            let Some(state) = states.get_mut(&key) else {
                states.insert(key, NativePpu::of(ppu, 1));
                return;
            };
            state.frame += 1;
            state.overflow = raster::Overflow {
                range: false,
                time: false,
            };
            state.even_frame = !state.even_frame;
            state.registers.begin_frame();
            if let Some(part) = state.divergence(ppu) {
                log(format_args!("{},resync,{part}\n", state.frame));
                *state = NativePpu::of(ppu, state.frame);
            }
        });
    }

    /// The harness loaded the emulated PPU from a snapshot without its ports.
    pub(crate) unsafe fn restored(ppu: *mut Ppu) {
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

    pub(crate) unsafe fn write(ppu: *mut Ppu, address: u8, value: u8) {
        if !feeds(ppu) {
            return;
        }
        STATES.with_borrow_mut(|states| {
            if let Some(state) = states.get_mut(&(ppu as usize)) {
                state.registers.write(address, value);
                state.ports.write(&mut state.memory, address, value);
            }
        });
    }

    pub(crate) unsafe fn read(ppu: *mut Ppu, address: u8) {
        if !feeds(ppu) {
            return;
        }
        let open_bus = unsafe { (*ppu).ppu2openBus };
        STATES.with_borrow_mut(|states| {
            if let Some(state) = states.get_mut(&(ppu as usize)) {
                state.ports.read(&state.memory, address, open_bus);
            }
        });
    }

    pub(crate) unsafe fn check(ppu: *mut Ppu, line: i32) {
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
                    interlace: state.registers.sprite_interlace,
                    even_frame: state.even_frame,
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
                // The emulated PPU's native mosaic byte and modulo are renderer
                // caches nothing writes; lines with mosaic take the per-pixel path.
                mosaic: raster::Mosaic {
                    native_enabled: 0,
                    background0_enabled: state.registers.layers[0].mosaic,
                    size: state.registers.mosaic_size,
                    start_line: state.registers.mosaic_start_line,
                    modulo: &[0; 256],
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
            // SAFETY: the host registered a render buffer of `renderPitch`-byte
            // rows for the frame's PPU, and the emulated PPU has finished the line.
            let row =
                unsafe { core::slice::from_raw_parts_mut(ppu.renderBuffer.add(start), 256 * 4) };
            let mut differing = (0..256).filter(|&x| pixels[x][..] != row[x * 4..x * 4 + 4]);
            if let Some(first) = differing.next() {
                let count = 1 + differing.count();
                log(format_args!(
                    "{},{line},{},{count},{first}\n",
                    state.frame, registers.mode
                ));
            }
            // D9: the engine's line is the native renderer's; the emulated PPU's
            // was drawn only for this comparison. An oracle frame keeps its own.
            if ENGINE_FRAME.get() {
                row.copy_from_slice(pixels.as_flattened());
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

    pub(crate) fn interval() -> u32 {
        static INTERVAL: OnceLock<u32> = OnceLock::new();
        *INTERVAL.get_or_init(|| {
            std::env::var("SM_CROSS_RENDER")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0)
        })
    }

    pub(crate) unsafe fn check(ppu: *mut Ppu, line: i32) {
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
                ppu_render_pixel(ppu, x, line);
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
