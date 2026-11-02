//! Explicit device image transfer at the retained external-writer boundary.
//! The caller chooses the actual physical PPU/SNES and owns quiescence/rejoin.
#![forbid(unsafe_code)]
use crate::types::{Ppu, Snes};
use sm_game::scene::hardware::DeviceImage;

pub fn capture(snes: &Snes, ppu: &Ppu) -> DeviceImage {
    let effective = effective(ppu);
    DeviceImage {
        effective,
        arithmetic: sm_data::hardware::Arithmetic {
            multiplicand: snes.multiplyA,
            product: snes.multiplyResult,
            dividend: snes.divideA,
            quotient: snes.divideResult,
        },
        affine: sm_data::hardware::Affine {
            matrix: ppu.m7matrix,
            horizontal_flip: ppu.m7xFlip,
            vertical_flip: ppu.m7yFlip,
            large_field: ppu.m7largeField,
            character_fill: ppu.m7charFill,
        },
        affine_previous: ppu.m7prev,
        ppu_open_bus: [ppu.ppu1openBus, ppu.ppu2openBus],
        mode_byte: ppu.mode
            | (u8::from(ppu.bg3priority) << 3)
            | ppu.bgLayer.iter().enumerate().fold(0, |bits, (index, bg)| {
                bits | (u8::from(bg.bigTiles) << (index + 4))
            }),
        scroll: core::array::from_fn(|index| {
            let bg = &ppu.bgLayer[index / 2];
            if index & 1 == 0 {
                bg.hScroll
            } else {
                bg.vScroll
            }
        }),
        window_selection: core::array::from_fn(|index| {
            let nibble = |window: &crate::types::WindowLayer| {
                u8::from(window.window1inversed)
                    | (u8::from(window.window1enabled) << 1)
                    | (u8::from(window.window2inversed) << 2)
                    | (u8::from(window.window2enabled) << 3)
            };
            nibble(&ppu.windowLayer[index * 2]) | (nibble(&ppu.windowLayer[index * 2 + 1]) << 4)
        }),
        scroll_previous: ppu.scrollPrev,
        scroll_previous_second: ppu.scrollPrev2,
    }
}
pub fn restore(image: DeviceImage, snes: &mut Snes, ppu: &mut Ppu) {
    restore_effective(image.effective, ppu);
    snes.multiplyA = image.arithmetic.multiplicand;
    snes.multiplyResult = image.arithmetic.product;
    snes.divideA = image.arithmetic.dividend;
    snes.divideResult = image.arithmetic.quotient;
    ppu.m7matrix = image.affine.matrix;
    ppu.m7xFlip = image.affine.horizontal_flip;
    ppu.m7yFlip = image.affine.vertical_flip;
    ppu.m7largeField = image.affine.large_field;
    ppu.m7charFill = image.affine.character_fill;
    ppu.m7prev = image.affine_previous;
    [ppu.ppu1openBus, ppu.ppu2openBus] = image.ppu_open_bus;
    ppu.mode = image.mode_byte & 7;
    ppu.bg3priority = image.mode_byte & 8 != 0;
    for (index, bg) in ppu.bgLayer.iter_mut().enumerate() {
        bg.bigTiles = image.mode_byte & (0x10 << index) != 0;
        bg.hScroll = image.scroll[index * 2];
        bg.vScroll = image.scroll[index * 2 + 1];
    }
    for (index, window) in ppu.windowLayer.iter_mut().enumerate() {
        let flags = image.window_selection[index / 2] >> ((index & 1) * 4);
        window.window1inversed = flags & 1 != 0;
        window.window1enabled = flags & 2 != 0;
        window.window2inversed = flags & 4 != 0;
        window.window2enabled = flags & 8 != 0;
    }
    // Rebuild only this register's existing cache. Routing is a different owner.
    ppu.windowsel = (ppu.windowsel & 0xff00_0000)
        | u32::from_le_bytes([
            image.window_selection[0],
            image.window_selection[1],
            image.window_selection[2],
            0,
        ]);
    ppu.scrollPrev = image.scroll_previous;
    ppu.scrollPrev2 = image.scroll_previous_second;
    ppu.windowsel = image.effective.raster_inputs.window_native_flags;
}

fn effective(ppu: &Ppu) -> sm_game::scene::state::EffectiveDisplayDescription {
    use sm_game::scene::state::{EffectiveDisplayDescription, RasterInputs};
    let other = [0, 1, 3];
    EffectiveDisplayDescription {
        affine: sm_data::hardware::Affine {
            matrix: ppu.m7matrix,
            horizontal_flip: ppu.m7xFlip,
            vertical_flip: ppu.m7yFlip,
            large_field: ppu.m7largeField,
            character_fill: ppu.m7charFill,
        },
        mode_byte: ppu.mode
            | (u8::from(ppu.bg3priority) << 3)
            | ppu.bgLayer.iter().enumerate().fold(0, |bits, (index, bg)| {
                bits | (u8::from(bg.bigTiles) << (index + 4))
            }),
        scroll: core::array::from_fn(|index| {
            let bg = &ppu.bgLayer[index / 2];
            if index & 1 == 0 {
                bg.hScroll
            } else {
                bg.vScroll
            }
        }),
        window_selection: core::array::from_fn(|index| {
            let nibble = |window: &crate::types::WindowLayer| {
                u8::from(window.window1inversed)
                    | (u8::from(window.window1enabled) << 1)
                    | (u8::from(window.window2inversed) << 2)
                    | (u8::from(window.window2enabled) << 3)
            };
            nibble(&ppu.windowLayer[index * 2]) | (nibble(&ppu.windowLayer[index * 2 + 1]) << 4)
        }),
        background3_map_words: ppu.bgLayer[2].tilemapAdr,
        background3_map_wider: ppu.bgLayer[2].tilemapWider,
        background3_map_higher: ppu.bgLayer[2].tilemapHigher,
        color_clip: ppu.clipMode,
        color_prevent_math: ppu.preventMathMode,
        color_add_subscreen: ppu.addSubscreen,
        direct_color: ppu.directColor,
        color_math_enabled: ppu.mathEnabled,
        color_subtract: ppu.subtractColor,
        color_half: ppu.halfColor,
        main_layers: ppu.screenEnabled[0],
        forced_blank: ppu.forcedBlank,
        raster_inputs: RasterInputs {
            map_words: other.map(|index| ppu.bgLayer[index].tilemapAdr),
            map_wider: other.map(|index| ppu.bgLayer[index].tilemapWider),
            map_higher: other.map(|index| ppu.bgLayer[index].tilemapHigher),
            tile_words: core::array::from_fn(|index| ppu.bgLayer[index].tileAdr),
            object_tile_words: [ppu.objTileAdr1, ppu.objTileAdr2],
            object_size: ppu.objSize,
            object_priority: ppu.objPriority,
            object_interlaced: ppu.objInterlace,
            extended_background: ppu.m7extBg,
            pseudo_hires: ppu.pseudoHires,
            window_positions: [
                ppu.window1left,
                ppu.window1right,
                ppu.window2left,
                ppu.window2right,
            ],
            window_logic: core::array::from_fn(|index| ppu.windowLayer[index].maskLogic),
            window_native_flags: ppu.windowsel,
            sub_layers: ppu.screenEnabled[1],
            screen_windowed: ppu.screenWindowed,
            fixed_color: [ppu.fixedColorR, ppu.fixedColorG, ppu.fixedColorB],
            brightness: ppu.brightness,
            mosaic_enabled: core::array::from_fn(|index| ppu.bgLayer[index].mosaicEnabled),
            mosaic_native_enabled: ppu.mosaicEnabled,
            mosaic_size: ppu.mosaicSize,
            mosaic_modulo: ppu.mosaicModulo,
        },
    }
}
fn restore_effective(effective: sm_game::scene::state::EffectiveDisplayDescription, ppu: &mut Ppu) {
    let inputs = effective.raster_inputs;
    for (slot, index) in [0, 1, 3].into_iter().enumerate() {
        ppu.bgLayer[index].tilemapAdr = inputs.map_words[slot];
        ppu.bgLayer[index].tilemapWider = inputs.map_wider[slot];
        ppu.bgLayer[index].tilemapHigher = inputs.map_higher[slot];
    }
    ppu.bgLayer[2].tilemapAdr = effective.background3_map_words;
    ppu.bgLayer[2].tilemapWider = effective.background3_map_wider;
    ppu.bgLayer[2].tilemapHigher = effective.background3_map_higher;
    for (index, bg) in ppu.bgLayer.iter_mut().enumerate() {
        bg.tileAdr = inputs.tile_words[index];
        bg.mosaicEnabled = inputs.mosaic_enabled[index];
    }
    [ppu.objTileAdr1, ppu.objTileAdr2] = inputs.object_tile_words;
    ppu.objSize = inputs.object_size;
    ppu.objPriority = inputs.object_priority;
    ppu.objInterlace = inputs.object_interlaced;
    ppu.m7extBg = inputs.extended_background;
    ppu.pseudoHires = inputs.pseudo_hires;
    [
        ppu.window1left,
        ppu.window1right,
        ppu.window2left,
        ppu.window2right,
    ] = inputs.window_positions;
    for (index, window) in ppu.windowLayer.iter_mut().enumerate() {
        window.maskLogic = inputs.window_logic[index];
    }
    ppu.windowsel = inputs.window_native_flags;
    ppu.screenEnabled = [effective.main_layers, inputs.sub_layers];
    ppu.screenWindowed = inputs.screen_windowed;
    for (index, layer) in ppu.layer.iter_mut().enumerate() {
        layer.mainScreenEnabled = effective.main_layers & (1 << index) != 0;
        layer.subScreenEnabled = inputs.sub_layers & (1 << index) != 0;
        layer.mainScreenWindowed = inputs.screen_windowed[0] & (1 << index) != 0;
        layer.subScreenWindowed = inputs.screen_windowed[1] & (1 << index) != 0;
    }
    ppu.clipMode = effective.color_clip;
    ppu.preventMathMode = effective.color_prevent_math;
    ppu.addSubscreen = effective.color_add_subscreen;
    ppu.directColor = effective.direct_color;
    ppu.mathEnabled = effective.color_math_enabled;
    ppu.subtractColor = effective.color_subtract;
    ppu.halfColor = effective.color_half;
    ppu.forcedBlank = effective.forced_blank;
    [ppu.fixedColorR, ppu.fixedColorG, ppu.fixedColorB] = inputs.fixed_color;
    ppu.brightness = inputs.brightness;
    ppu.mosaicSize = inputs.mosaic_size;
    ppu.mosaicEnabled = inputs.mosaic_native_enabled;
    ppu.mosaicModulo = inputs.mosaic_modulo;
}

pub fn capture_video(snes: &Snes, ppu: &Ppu) -> sm_game::scene::hardware::PhysicalVideoImage {
    use sm_game::scene::{hardware::PhysicalVideoImage, state::DisplayLatches};
    let latches = DisplayLatches {
        tile_pointer: ppu.vramPointer,
        tile_increment: ppu.vramIncrement,
        tile_remap: ppu.vramRemapMode,
        tile_read_buffer: ppu.vramReadBuffer,
        tile_increment_on_high: ppu.vramIncrementOnHigh,
        tile_padding: [ppu.snapshot_padding0[0], ppu.snapshot_padding1[0]],
        palette_pointer: ppu.cgramPointer,
        palette_second: ppu.cgramSecondWrite,
        palette_buffer: ppu.cgramBuffer,
        object_written_address: u16::from(ppu.oamAdrWritten),
        object_written_high_table: ppu.oamInHighWritten,
        object_address: u16::from(ppu.oamAdr),
        object_high_table: ppu.oamInHigh,
        object_second: ppu.oamSecondWrite,
        object_buffer: ppu.oamBuffer,
        scroll_previous: ppu.scrollPrev,
        scroll_previous_second: ppu.scrollPrev2,
        affine_previous: ppu.m7prev,
        latched_horizontal: ppu.hCount,
        latched_vertical: ppu.vCount,
        horizontal_second: ppu.hCountSecond,
        vertical_second: ppu.vCountSecond,
        counter_latched: ppu.countersLatched,
        open_bus: snes.openBus,
        ppu_open_bus: [ppu.ppu1openBus, ppu.ppu2openBus],
    };
    let mut words = Vec::with_capacity(0x801f);
    words.extend_from_slice(&ppu.vram);
    words.extend_from_slice(&[
        ppu.vramPointer,
        u16::from_ne_bytes([u8::from(ppu.vramIncrementOnHigh), ppu.snapshot_padding0[0]]),
        ppu.vramIncrement,
        u16::from_ne_bytes([ppu.vramRemapMode, ppu.snapshot_padding1[0]]),
        ppu.vramReadBuffer,
    ]);
    words.extend_from_slice(&ppu.cgram[..26]);
    let mut objects = [0; 544];
    for (bytes, word) in objects[..512].chunks_exact_mut(2).zip(ppu.oam) {
        bytes.copy_from_slice(&word.to_le_bytes());
    }
    objects[512..].copy_from_slice(&ppu.highOam);
    PhysicalVideoImage {
        effective: effective(ppu),
        latches,
        raster: raster(ppu),
        words: std::sync::Arc::from(words),
        colors: ppu.cgram,
        objects,
        caches: render_caches(ppu),
    }
}
fn render_caches(ppu: &Ppu) -> sm_game::scene::hardware::PhysicalRenderCaches {
    sm_game::scene::hardware::PhysicalRenderCaches {
        affine_start: [ppu.m7startX, ppu.m7startY],
        object_pixels: ppu.objPixelBufferXX,
        object_priorities: ppu.objPriorityBufferXX,
        backgrounds: core::array::from_fn(|index| ppu.bgBuffers[index].data),
        objects: ppu.objBuffer.data,
        brightness_last: ppu.lastBrightnessMult,
        brightness_normal: ppu.brightnessMult,
        brightness_half: ppu.brightnessMultHalf,
        extra_left: ppu.extraLeftCur,
        extra_right: ppu.extraRightCur,
        extra_left_right: ppu.extraLeftRight,
    }
}
fn raster(ppu: &Ppu) -> sm_game::scene::hardware::PhysicalRaster {
    sm_game::scene::hardware::PhysicalRaster {
        range_overflow: ppu.rangeOver,
        time_overflow: ppu.timeOver,
        has_objects: ppu.lineHasSprites,
        even_frame: ppu.evenFrame,
        interlaced: ppu.interlace,
        frame_interlaced: ppu.frameInterlace,
        overscan: ppu.overscan,
        frame_overscan: ppu.frameOverscan,
        mosaic_origin: ppu.mosaicStartLine,
    }
}
pub fn restore_video(
    image: &sm_game::scene::hardware::PhysicalVideoImage,
    snes: &mut Snes,
    ppu: &mut Ppu,
) -> Result<(), sm_game::scene::state::DisplayError> {
    use sm_game::scene::state::DisplayError;
    // Validate representation before the first physical store.
    let words = image.words.get(..0x8000).ok_or(DisplayError::Incomplete)?;
    let aliases = [
        image.latches.tile_pointer,
        u16::from_ne_bytes([
            u8::from(image.latches.tile_increment_on_high),
            image.latches.tile_padding[0],
        ]),
        image.latches.tile_increment,
        u16::from_ne_bytes([image.latches.tile_remap, image.latches.tile_padding[1]]),
        image.latches.tile_read_buffer,
    ];
    if image.words.get(0x8000..0x8005) != Some(aliases.as_slice())
        || image.words.get(0x8005..0x801f) != Some(&image.colors[..26])
    {
        return Err(DisplayError::Source);
    }
    let address = u8::try_from(image.latches.object_address).map_err(|_| DisplayError::Geometry)?;
    let written =
        u8::try_from(image.latches.object_written_address).map_err(|_| DisplayError::Geometry)?;
    let latches = image.latches;
    let device = DeviceImage {
        arithmetic: sm_data::hardware::Arithmetic {
            multiplicand: snes.multiplyA,
            product: snes.multiplyResult,
            dividend: snes.divideA,
            quotient: snes.divideResult,
        },
        effective: image.effective,
        affine: image.effective.affine,
        affine_previous: latches.affine_previous,
        ppu_open_bus: latches.ppu_open_bus,
        mode_byte: image.effective.mode_byte,
        scroll: image.effective.scroll,
        window_selection: image.effective.window_selection,
        scroll_previous: latches.scroll_previous,
        scroll_previous_second: latches.scroll_previous_second,
    };
    restore(device, snes, ppu);
    ppu.vram.copy_from_slice(words);
    ppu.cgram = image.colors;
    for (word, bytes) in ppu.oam.iter_mut().zip(image.objects[..512].chunks_exact(2)) {
        *word = u16::from_le_bytes([bytes[0], bytes[1]]);
    }
    ppu.highOam.copy_from_slice(&image.objects[512..]);
    ppu.vramPointer = latches.tile_pointer;
    ppu.vramIncrement = latches.tile_increment;
    ppu.vramRemapMode = latches.tile_remap;
    ppu.vramReadBuffer = latches.tile_read_buffer;
    ppu.vramIncrementOnHigh = latches.tile_increment_on_high;
    ppu.snapshot_padding0[0] = latches.tile_padding[0];
    ppu.snapshot_padding1[0] = latches.tile_padding[1];
    ppu.cgramPointer = latches.palette_pointer;
    ppu.cgramSecondWrite = latches.palette_second;
    ppu.cgramBuffer = latches.palette_buffer;
    ppu.oamAdr = address;
    ppu.oamAdrWritten = written;
    ppu.oamInHigh = latches.object_high_table;
    ppu.oamInHighWritten = latches.object_written_high_table;
    ppu.oamSecondWrite = latches.object_second;
    ppu.oamBuffer = latches.object_buffer;
    ppu.hCount = latches.latched_horizontal;
    ppu.vCount = latches.latched_vertical;
    ppu.hCountSecond = latches.horizontal_second;
    ppu.vCountSecond = latches.vertical_second;
    ppu.countersLatched = latches.counter_latched;
    snes.openBus = latches.open_bus;
    let status = image.raster;
    ppu.rangeOver = status.range_overflow;
    ppu.timeOver = status.time_overflow;
    ppu.lineHasSprites = status.has_objects;
    ppu.evenFrame = status.even_frame;
    ppu.interlace = status.interlaced;
    ppu.frameInterlace = status.frame_interlaced;
    ppu.overscan = status.overscan;
    ppu.frameOverscan = status.frame_overscan;
    ppu.mosaicStartLine = status.mosaic_origin;
    let caches = &image.caches;
    [ppu.m7startX, ppu.m7startY] = caches.affine_start;
    ppu.objPixelBufferXX = caches.object_pixels;
    ppu.objPriorityBufferXX = caches.object_priorities;
    for (index, buffer) in ppu.bgBuffers.iter_mut().enumerate() {
        buffer.data = caches.backgrounds[index];
    }
    ppu.objBuffer.data = caches.objects;
    ppu.lastBrightnessMult = caches.brightness_last;
    ppu.brightnessMult = caches.brightness_normal;
    ppu.brightnessMultHalf = caches.brightness_half;
    ppu.extraLeftCur = caches.extra_left;
    ppu.extraRightCur = caches.extra_right;
    ppu.extraLeftRight = caches.extra_left_right;
    Ok(())
}

pub use sm_game::scene::hardware::{
    PhysicalRasterHistory as NativeRasterHistory, PhysicalRasterRow as NativeRasterRow,
};
struct RasterCollection {
    keys: [usize; 2],
    rows: [Vec<NativeRasterRow>; 2],
    active: bool,
}
thread_local! {
    static RASTER: std::cell::RefCell<Option<RasterCollection>> = const { std::cell::RefCell::new(None) };
}
pub(crate) fn begin_raster(keys: [usize; 2]) -> Result<(), sm_game::scene::state::DisplayError> {
    RASTER.with_borrow_mut(|collection| {
        if collection.is_some() {
            return Err(sm_game::scene::state::DisplayError::Phase);
        }
        *collection = Some(RasterCollection {
            keys,
            rows: [Vec::new(), Vec::new()],
            active: true,
        });
        Ok(())
    })
}
pub(crate) fn finish_raster(
    key: usize,
    terminal: sm_game::scene::hardware::PhysicalVideoImage,
) -> Result<NativeRasterHistory, sm_game::scene::state::DisplayError> {
    RASTER.with_borrow_mut(|collection| {
        let current = collection
            .as_mut()
            .ok_or(sm_game::scene::state::DisplayError::Phase)?;
        let index = current
            .keys
            .iter()
            .position(|old| *old == key)
            .ok_or(sm_game::scene::state::DisplayError::Identity)?;
        let rows = std::mem::take(&mut current.rows[index]);
        *collection = None;
        Ok(NativeRasterHistory { rows, terminal })
    })
}
pub(crate) fn cancel_raster() {
    // Keep an unwinding operation's prefix for explicit selection after catch_unwind.
    RASTER.with_borrow_mut(|collection| {
        if let Some(current) = collection.as_mut() {
            current.active = false;
        }
    });
}
pub(crate) fn raster_before(snes: &Snes, ppu: &Ppu, row: i32) {
    let key = core::ptr::from_ref(ppu) as usize;
    RASTER.with_borrow_mut(|collection| {
        let Some(current) = collection.as_mut() else {
            return;
        };
        if !current.active {
            return;
        }
        let Some(index) = current.keys.iter().position(|old| *old == key) else {
            return;
        };
        if row == 0 {
            current.rows[index].clear();
            return;
        }
        let row = u16::try_from(row).expect("native raster row is nonnegative");
        current.rows[index].push(NativeRasterRow {
            row,
            before: capture_video(snes, ppu),
            after: None,
            after_caches: None,
        });
    });
}
pub(crate) fn raster_after(ppu: &Ppu) {
    let key = core::ptr::from_ref(ppu) as usize;
    RASTER.with_borrow_mut(|collection| {
        let Some(current) = collection.as_mut() else {
            return;
        };
        if !current.active {
            return;
        }
        let Some(index) = current.keys.iter().position(|old| *old == key) else {
            return;
        };
        if let Some(row) = current.rows[index].last_mut() {
            row.after = Some(raster(ppu));
            row.after_caches = Some(render_caches(ppu));
        }
    });
}
