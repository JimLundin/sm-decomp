use crate::initial_state::InitialState;
use crate::rendering as raster;
use ::core::ffi::c_void;
use ::core::mem::size_of;

use crate::c_library::memcpy;
#[cfg(sm_verify_dll)]
pub(crate) use crate::snes::ppu_lines::native_line_restored;
#[cfg(not(sm_host))]
pub use crate::snes::ppu_lines::{engine_frame, observe_pixels, ppu_run_line};
#[cfg(sm_verify_dll)]
use crate::snes::ppu_lines::{model_check, native_line};
pub use crate::types::{Ppu, PpuPixelPrioBufs, PpuZbufType, Snes};
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
// Registers use upstream. Ppu offsets are asserted in ppu_lines.rs and in
// backend.c. The Rust Snes pointer stays opaque: register calls substitute a C view.
#[cfg(sm_host)]
use crate::ppu_ports::{sm_backend_ppu_read, sm_backend_ppu_write};
#[cfg(not(sm_host))]
#[link(name = "lakesnes_backend", kind = "static", modifiers = "-bundle")]
unsafe extern "C" {
    #[allow(improper_ctypes)]
    fn sm_backend_ppu_read(ppu: *mut Ppu, adr: u8, h_pos: u16, v_pos: u16, bus: u8) -> u8;
    #[allow(improper_ctypes)]
    fn sm_backend_ppu_write(ppu: *mut Ppu, adr: u8, value: u8);
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
        if matches!(adr, 0x18 | 0x19) && SAMUS_VRAM_DMA.get() == Some(ppu as usize) {
            samus_vram_byte(&mut *ppu, adr == 0x19, val);
        } else {
            sm_backend_ppu_write(ppu, adr, val);
        }
        #[cfg(sm_verify_dll)]
        native_line::write(ppu, adr, val);
        #[cfg(sm_verify_dll)]
        if let Some(checked) = checked {
            model_check::after_write(&*ppu, adr, val, checked);
        }
    }
}

thread_local! {
    /// A synchronous Samus DMA interval; never stores video state or a live borrow.
    static SAMUS_VRAM_DMA: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}

/// The original DMA drain retains all bus reads, open-bus writes and channel effects.
/// Only this selected PPU's VRAM byte sink runs the typed implementation.
pub(crate) fn samus_vram_dma(ppu: *mut Ppu, drain: impl FnOnce()) {
    struct Restore(Option<usize>);
    impl Drop for Restore {
        fn drop(&mut self) {
            SAMUS_VRAM_DMA.set(self.0);
        }
    }
    let _restore = Restore(SAMUS_VRAM_DMA.replace(Some(ppu as usize)));
    drain();
}

pub(crate) fn samus_vram_mode(cell: &std::cell::Cell<Ppu>) {
    let mut ppu = cell.get();
    #[cfg(sm_verify_dll)]
    let checked = model_check::before_write(&ppu, 0x15, 0x80);
    let mut ports = ports_of(&ppu);
    ports.set_vram_mode(1, 0, true);
    ppu.vramIncrement = ports.vram_increment;
    ppu.vramRemapMode = ports.vram_remap;
    ppu.vramIncrementOnHigh = ports.vram_increment_on_high;
    cell.set(ppu);
    #[cfg(sm_verify_dll)]
    if let Some(checked) = checked {
        model_check::after_write(&ppu, 0x15, 0x80, checked);
    }
}

pub(crate) fn samus_vram_pointer_byte(cell: &std::cell::Cell<Ppu>, value: u8, high: bool) {
    let mut ppu = cell.get();
    #[cfg(sm_verify_dll)]
    let register = if high { 0x17 } else { 0x16 };
    #[cfg(sm_verify_dll)]
    let checked = model_check::before_write(&ppu, register, value);
    let mut ports = ports_of(&ppu);
    let memory = MemoryView::new(&ppu);
    let mut address = ports.vram_pointer.to_le_bytes();
    address[usize::from(high)] = value;
    ports.set_vram_pointer(&memory, u16::from_le_bytes(address));
    ppu.vramPointer = ports.vram_pointer;
    ppu.vramReadBuffer = ports.vram_read_buffer;
    cell.set(ppu);
    #[cfg(sm_verify_dll)]
    if let Some(checked) = checked {
        model_check::after_write(&ppu, register, value, checked);
    }
}

fn samus_vram_byte(ppu: &mut Ppu, high: bool, value: u8) {
    let mut ports = ports_of(ppu);
    let mut memory = MemoryView::new(ppu);
    ports.write_vram_byte(&mut memory, high, value);
    let Some(Store::Vram(index, word)) = memory.store else {
        unreachable!("a VRAM byte always makes one VRAM store");
    };
    // Publish this byte before the DMA advances its channel, including on failure.
    ppu.vram[usize::from(index)] = word;
    ppu.vramPointer = ports.vram_pointer;
}

/// The emulated PPU's port state in the native renderer's model.
pub(crate) fn ports_of(ppu: &Ppu) -> raster::memory::Ports {
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Store {
    Vram(u16, u16),
    Cgram(u8, u16),
    Oam(u8, u16),
    HighOam(u8, u8),
}

#[cfg(sm_verify_dll)]
impl Store {
    pub(crate) fn reread(self, memory: &impl raster::memory::VideoMemory) -> Self {
        match self {
            Self::Vram(index, _) => Self::Vram(index, memory.vram(index)),
            Self::Cgram(index, _) => Self::Cgram(index, memory.cgram(index)),
            Self::Oam(index, _) => Self::Oam(index, memory.oam(index)),
            Self::HighOam(index, _) => Self::HighOam(index, memory.high_oam(index)),
        }
    }
}

/// A read-only PPU memory view recording one port store. The caller commits it
/// immediately for typed production, or compares it in the verifier.
pub(crate) struct MemoryView<'a> {
    ppu: &'a Ppu,
    /// A port write makes at most one store.
    pub(crate) store: Option<Store>,
}

impl<'a> MemoryView<'a> {
    pub(crate) fn new(ppu: &'a Ppu) -> Self {
        Self { ppu, store: None }
    }
}

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
#[cfg(any(sm_host, sm_verify_dll))]
pub(crate) fn registers_of(ppu: &Ppu) -> raster::registers::Registers {
    use raster::registers::{Layer, Registers};
    Registers {
        mode: ppu.mode,
        bg3_priority: ppu.bg3priority,
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
        mosaic_start_line: ppu.mosaicStartLine,
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
        interlace: ppu.interlace,
        sprite_interlace: ppu.objInterlace,
        overscan: ppu.overscan,
        pseudo_hires: ppu.pseudoHires,
        extended_background: ppu.m7extBg,
        scroll_latch: ppu.scrollPrev,
        horizontal_scroll_latch: ppu.scrollPrev2,
        affine_latch: ppu.m7prev,
    }
}
pub fn ppu_get_current_render_scale(_ppu: *mut Ppu, _render_flags: u32) -> i32 {
    1
}

/// The PPU's memory as the native renderer reads it. VRAM addresses wrap at
/// its end, as on the PPU.
#[cfg(sm_host)]
struct VideoSource<'a>(&'a Ppu);

#[cfg(sm_host)]
impl raster::IndexedSource for VideoSource<'_> {
    fn word(&self, index: usize) -> Result<u16, raster::RasterFault> {
        Ok(self.0.vram[index & 0x7fff])
    }

    fn palette(&self, index: u8) -> Result<u16, raster::RasterFault> {
        Ok(self.0.cgram[usize::from(index)])
    }
}

/// One beam line, drawn by the native renderer (D9) from the PPU's register
/// and memory state into the registered frame buffer. Line 0 starts the frame.
/// This host has no emulated pixel path, so `_new_ppu` selects nothing.
#[cfg(sm_host)]
pub(crate) unsafe fn ppu_run_line(ppu: *mut Ppu, line: i32, _new_ppu: &bool) {
    // SAFETY: the beam's selected PPU is live for the line and only this thread
    // reaches it.
    let ppu = unsafe { &mut *ppu };
    if line == 0 {
        ppu.mosaicStartLine = 1;
        ppu.rangeOver = false;
        ppu.timeOver = false;
        ppu.evenFrame = !ppu.evenFrame;
        return;
    }
    let line = u16::try_from(line).expect("the beam draws lines 1-239");
    let registers = registers_of(ppu);
    let ports = ports_of(ppu);
    let sprites = raster::SpriteInput {
        words: &ppu.oam,
        extension: &ppu.highOam,
        tile_words: ports.sprite_tile_words,
        size: ports.sprite_size,
        rotation: ports.sprite_priority.then_some(ports.oam_address),
        interlace: ppu.objInterlace,
        even_frame: ppu.evenFrame,
    };
    let overflow = raster::Overflow {
        range: ppu.rangeOver,
        time: ppu.timeOver,
    };
    if registers.composes_per_pixel() && !registers.forced_blank {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTS: [AtomicU64; 8] = [const { AtomicU64::new(0) }; 8];
        let r = &registers;
        let reasons = [
            r.mode != 1 && r.mode != 7,
            r.mode == 1 && !r.bg3_priority,
            r.mode == 1 && r.layers[..3].iter().any(|l| l.big_tiles),
            r.mode == 7 && r.extended_background,
            r.mode == 7 && r.direct_color,
            r.pseudo_hires,
            r.mosaic_size > 1 && r.layers.iter().any(|l| l.mosaic),
            r.windows
                .layers
                .iter()
                .any(|w| w.first_enabled && w.second_enabled && w.logic != 0),
        ];
        let names = [
            "mode",
            "bg3-normal",
            "big-tiles",
            "extbg",
            "direct",
            "hires",
            "mosaic",
            "window-logic",
        ];
        for (index, _) in reasons.iter().enumerate().filter(|(_, hit)| **hit) {
            let count = COUNTS[index].fetch_add(1, Ordering::Relaxed) + 1;
            if count == 1 {
                let logic: Vec<u8> = r.windows.layers.iter().map(|w| w.logic).collect();
                eprintln!(
                    "DIAG composed {} #{count} line {line} mode {} bg3 {} main {:02x} sub {:02x} winsel {:06x} logic {logic:?}",
                    names[index],
                    r.mode,
                    r.bg3_priority,
                    r.routing.enabled[0],
                    r.routing.enabled[1],
                    r.windows.native_flags
                );
            } else if count == 10_000 || count == 1_000_000 {
                eprintln!("DIAG {} #{count}", names[index]);
            }
        }
    }
    let mut pixels = [[0; 4]; raster::WIDTH];
    let overflow = raster::scanline::draw_line(
        &VideoSource(ppu),
        &registers,
        &sprites,
        line,
        overflow,
        &mut pixels,
    )
    .unwrap_or_else(|fault| panic!("PPU raster fault: {fault:?}"));
    ppu.rangeOver = overflow.range;
    ppu.timeOver = overflow.time;
    if !ppu.renderBuffer.is_null() {
        let start = usize::from(line - 1) * ppu.renderPitch as usize;
        // SAFETY: the host registered a frame buffer of `renderPitch`-byte rows
        // covering every visible line for this PPU.
        let row = unsafe {
            core::slice::from_raw_parts_mut(ppu.renderBuffer.add(start), raster::WIDTH * 4)
        };
        row.copy_from_slice(pixels.as_flattened());
    }
}

/// A typed producer's port writes begin: until `produced`, they reach only the
/// emulated PPU, not the native one.
pub(crate) fn producing(ppu: *mut Ppu) {
    #[cfg(sm_verify_dll)]
    native_line::producing(ppu);
    #[cfg(not(sm_verify_dll))]
    let _ = ppu;
}

/// A typed producer's port writes are done: the native PPU takes `native`, the
/// producer's typed calls, in their place (see `native_line::produced`).
pub(crate) fn produced(
    ppu: *mut Ppu,
    producer: &'static str,
    native: impl FnOnce(&mut raster::memory::Ports, &mut raster::memory::OwnedMemory),
) {
    #[cfg(sm_verify_dll)]
    native_line::produced(ppu, producer, native);
    #[cfg(not(sm_verify_dll))]
    let _ = (ppu, producer, native);
}
