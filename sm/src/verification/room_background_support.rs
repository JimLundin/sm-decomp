//! Actual device initialization and field-wise observation, synthetic assets only.
use crate::{
    game_state::{Game, WorkRam},
    recording::RecordingBytes,
    rom::RomImage,
};
pub(crate) fn offset(a: u32) -> usize {
    crate::rtl::lorom_offset(a) as usize
}
pub(crate) fn bytes(r: &RomImage, a: u32, v: &[u8]) {
    for (i, b) in v.iter().enumerate() {
        r.write(offset(a) + i, *b);
    }
}
pub(crate) fn rw(r: &RomImage, a: u32, v: u16) {
    bytes(r, a, &v.to_ne_bytes());
}
pub(crate) fn put(g: &mut Game, a: usize, v: u16) {
    g.ram().copy_in(a, &v.to_le_bytes());
}
pub(crate) fn exp(e: &mut [u8], a: usize, v: u16) {
    e[a..a + 2].copy_from_slice(&v.to_le_bytes());
}
pub(crate) fn literal(r: &RomImage, a: u32, v: &[u8]) {
    assert!(!v.is_empty() && v.len() <= 32);
    bytes(r, a, &[(v.len() - 1) as u8]);
    bytes(r, a + 1, v);
    bytes(r, a + 1 + v.len() as u32, &[0xff]);
}
pub(crate) unsafe fn initialized() -> Box<Game> {
    unsafe {
        initialized_with_world(crate::game::room_entry::world::WorldBinding::canonical_legacy())
    }
}
pub(crate) unsafe fn initialized_with_world(
    world: std::sync::Arc<crate::game::room_entry::world::WorldBinding>,
) -> Box<Game> {
    unsafe {
        let mut g = Game::with_world(world);
        g.initialize_backend();
        let image = RomImage::zeroed(0x400000);
        for (a, v) in [(0x80a2f7, 0x4e), (0x80a2f8, 0x18), (0x80a29a, 0x38)] {
            bytes(&image, a, &[v]);
        }
        for i in 0..0x1000 {
            image.write(offset(0x9ab200) + i, (i.wrapping_mul(7) ^ 0xd3) as u8);
        }
        let cart_image = RomImage::zeroed(image.len());
        for i in 0..image.len() {
            cart_image.write(i, image.read(i));
        }
        crate::snes::cart::cart_load((*g.g_snes).cart, 1, cart_image, 8192);
        g.register_native_rom((*(*g.g_snes).cart).rom.clone());
        g.g_sram = (*(*g.g_snes).cart).ram.clone();
        g.execution().reset(true);
        for i in 0..g.ram().len() {
            g.ram()
                .write_byte(i, (i.wrapping_mul(13) ^ (i >> 8) ^ 0x5a) as u8);
        }
        (&mut (*(*g.g_snes).ppu).vram).fill(0x4567);
        g.VERIFY_ACTIVE = true;
        g.g_fail = false;
        g.decompress_src = 0xabcdef;
        g
    }
}
/// Existing typed serializer omits pointers/routing caches. It does not call
/// snes_saveload, whose runningWhichVersion write is an observable side effect.
pub(crate) fn snapshot(g: &mut Game) -> Vec<u8> {
    snapshot_parts(g.ram(), &g.g_fail, &g.decompress_src, |machine| {
        g.save_backend_observation(machine);
    })
}
#[forbid(unsafe_code)]
pub(crate) fn snapshot_parts(
    ram: &WorkRam,
    fail: &bool,
    decompress_src: &u32,
    mut observe: impl FnMut(&mut RecordingBytes),
) -> Vec<u8> {
    let mut out = ram_bytes_parts(ram);
    out.push(u8::from(*fail));
    out.extend_from_slice(&decompress_src.to_le_bytes());
    let mut machine = RecordingBytes::default();
    observe(&mut machine);
    out.extend_from_slice(&(machine.size as u32).to_le_bytes());
    out.extend_from_slice(machine.as_slice());
    out
}

pub(crate) fn record(out: &mut Vec<u8>, tag: u8, payload: &[u8]) {
    out.push(tag);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
}
pub(crate) fn word(out: &mut Vec<(u16, u8)>, a: u16, v: u16) {
    out.push((a, v as u8));
    out.push((a + 1, (v >> 8) as u8));
}
pub(crate) fn copy(out: &mut Vec<(u16, u8)>, dst: u16, src: u32, n: u16) {
    out.push((0x2100, 0x80));
    word(out, 0x2116, dst);
    word(out, 0x4310, 0x1801);
    word(out, 0x4312, src as u16);
    out.push((0x4314, (src >> 16) as u8));
    word(out, 0x4315, n);
    out.extend([(0x2115, 0x80), (0x420b, 2), (0x2100, 15)]);
}
pub(crate) fn direct(out: &mut Vec<(u16, u8)>, dst: u16, src: u16, bankword: u16, n: Option<u16>) {
    word(out, 0x2116, dst);
    word(out, 0x4310, 0x1801);
    word(out, 0x4312, src);
    word(out, 0x4314, bankword);
    if let Some(n) = n {
        word(out, 0x4315, n);
        out.extend([(0x2115, 0x80), (0x420b, 2)]);
    }
}
pub(crate) fn clear_initial(out: &mut Vec<(u16, u8)>, dst: u16, low: u16, high: u16, n: u16) {
    word(out, 0x2116, dst);
    word(out, 0x4310, 0x1808);
    word(out, 0x4312, low);
    word(out, 0x4314, 0x80);
    word(out, 0x4315, n);
    out.extend([(0x2115, 0), (0x420b, 2)]);
    word(out, 0x2116, dst);
    word(out, 0x4310, 0x1908);
    word(out, 0x4312, high);
    word(out, 0x4314, 0x80);
    word(out, 0x4315, n);
    out.extend([(0x2115, 0x80), (0x420b, 2)]);
}
pub(crate) fn vram_bytes(v: &mut [u16], dst: u16, data: &[u8]) {
    assert_eq!(data.len() % 2, 0);
    for (i, b) in data.chunks_exact(2).enumerate() {
        v[(usize::from(dst) + i) & 0x7fff] = u16::from_le_bytes([b[0], b[1]]);
    }
}

pub(crate) fn ram_bytes(game: &Game) -> Vec<u8> {
    ram_bytes_parts(game.ram())
}
pub(crate) fn ram_bytes_parts(ram: &WorkRam) -> Vec<u8> {
    let mut bytes = vec![0; ram.len()];
    ram.copy_out(0, &mut bytes);
    bytes
}
pub(crate) fn ram_span<const N: usize>(game: &Game, at: usize) -> [u8; N] {
    let mut bytes = [0; N];
    game.ram().copy_out(at, &mut bytes);
    bytes
}
pub(crate) fn ram_word(ram: &crate::game_state::WorkRam, at: usize, value: u16) {
    ram.copy_in(at, &value.to_le_bytes());
}

use crate::game::room_entry::{
    content::{AssetRef, AssetSource},
    effects::EntryEffects,
    world::*,
};
// Movies cannot isolate a prepared VRAM stream's self/live/unproduced references.
pub(crate) unsafe fn check_prepared_vram(
    world: &WorldDefinition,
    keys: &CompatibilityKeys,
    assets: &PreparedRoomAssets,
) {
    unsafe {
        use crate::game::room_entry::{
            content::VramDestination,
            effects::{VramDecodeTarget, VramSlot},
        };
        let prepared = crate::room_world_resources::replace_stream(
            assets,
            crate::room_world_fixture::TILES,
            &[0x81, 0, 0, 0x80, 0x10, 0, 0xff],
            world,
            keys,
        );
        let binding =
            WorldBinding::prepared_with_room_assets(world.clone(), keys.clone(), prepared).unwrap();
        let mut game = crate::room_background_support::initialized_with_world(binding);
        (*(*game.g_snes).ppu).vram[0] = 0x4567;
        (*(*game.g_snes).ppu).vram[8] = 0x89ab;
        let cursor = game.decompress_src;
        let mut h = crate::game::session_runtime::Runtime::new(&mut game);
        EntryEffects::device_byte(
            &mut h,
            crate::game::room_entry::effects::DeviceByte::VramIncrement,
            0x80,
        );
        EntryEffects::device_vram_address(&mut h, VramDestination::Slot(VramSlot::MainTiles));
        EntryEffects::decompress_vram(
            &mut h,
            AssetSource::Bound(AssetRef::Prepared(crate::room_world_fixture::TILES)),
            VramDecodeTarget::MainTiles,
        );
        drop(h);
        assert_eq!((*(*game.g_snes).ppu).vram[0], 0x4567);
        assert_eq!((*(*game.g_snes).ppu).vram[1], 0x45ab);
        assert_eq!(game.decompress_src, cursor);
    }
}
// Movies cannot isolate prepared ROM data from cartridge data at bank carry/wrap/size0.
pub(crate) unsafe fn check_prepared_dma(
    world: &WorldDefinition,
    keys: &CompatibilityKeys,
    assets: &PreparedRoomAssets,
) {
    unsafe {
        use crate::game::room_entry::{content::VramDestination, effects::VramSlot};
        let mut world = world.clone();
        let mut keys = keys.clone();
        let id = ResourceId(0x6200_000a);
        world.resources.insert(id, ResourceKind::Background);
        keys.resources
            .insert(id, ResourceKey::Long(LegacyAddress24([0xff, 0xff, 0xc0])));
        world
            .states
            .values_mut()
            .next()
            .unwrap()
            .background
            .push(BackgroundCommand::Transfer {
                source: id,
                destination: BackgroundTarget::Main,
                bytes: 2,
            });
        let mut source = vec![0; 0x400000];
        let (_, raw) = assets.raw_window();
        source[..raw.len()].copy_from_slice(raw);
        source[0x207fff] = 0xa5;
        source[0x208000] = 0xb6;
        let prepared =
            crate::room_world_resources::prepared_assets(&source, &world, &keys).unwrap();
        assert_eq!(
            prepared.rom_byte(0x208000),
            0xb6,
            "room.assets.dma-carry-pre-read-covered"
        );
        let expected = prepared.clone();
        let binding = WorldBinding::prepared_with_room_assets(world, keys, prepared).unwrap();
        let mut game = crate::room_background_support::initialized_with_world(binding);
        (*(*game.g_snes).cart).rom.write(0x207fff, 0x22);
        (*(*game.g_snes).cart).rom.write(0x208000, 0x33);
        for count in [2u16, 0] {
            let cursor = game.decompress_src;
            EntryEffects::copy_vram(
                &mut crate::game::session_runtime::Runtime::new(&mut game),
                VramDestination::Slot(VramSlot::BackgroundFirst),
                AssetSource::Bound(AssetRef::Prepared(id)),
                count,
            );
            let words = if count == 0 { 32768 } else { 1 };
            for index in 0..words {
                let low = 0xffffu16.wrapping_add((index * 2) as u16);
                let high = low.wrapping_add(1);
                let at = |word| crate::rom::lorom_offset(0xc00000 | u32::from(word)) as usize;
                let value =
                    u16::from_le_bytes([expected.rom_byte(at(low)), expected.rom_byte(at(high))]);
                assert_eq!(
                    (*(*game.g_snes).ppu).vram[(0x4000 + index) & 0x7fff],
                    value,
                    "room.assets.dma-source-isolation-{count}-{index}"
                );
            }
            assert_eq!(game.decompress_src, cursor);
        }
    }
}

// No movie supplies independently authored media; share only input records, never expected pixels.
pub(crate) fn raster_fixture() -> Box<crate::types::Ppu> {
    use crate::initial_state::InitialState;
    let mut p = Box::new(crate::types::Ppu::INITIAL);
    p.brightness = 15;
    p.mode = 1;
    p.mosaicSize = 1;
    p.mosaicStartLine = 1;
    for sprite in p.oam.chunks_exact_mut(2) {
        sprite[0] = 0x8000;
    }
    p
}
pub(crate) fn raster_prepared(p: &crate::types::Ppu) -> crate::rendering::prepared::RenderedLine {
    use crate::rendering::{prepared::*, *};
    let media = IndexedImage::new(p.vram.to_vec(), p.cgram).unwrap();
    let sprites = PreparedSprites {
        words: p.oam,
        extension: p.highOam,
        tile_words: [p.objTileAdr1, p.objTileAdr2],
        size: p.objSize,
        rotation: p.objPriority.then_some(p.oamAdr),
        interlace: p.objInterlace,
        even_frame: p.evenFrame,
    };
    let mosaic = PreparedMosaic {
        native_enabled: p.mosaicEnabled,
        background0_enabled: p.bgLayer[0].mosaicEnabled,
        size: p.mosaicSize,
        start_line: p.mosaicStartLine,
        modulo: p.mosaicModulo,
    };
    PreparedInput {
        media: &media,
        sprites: &sprites,
        mode: p.mode,
        forced_blank: p.forcedBlank,
        backgrounds: core::array::from_fn(|i| {
            let b = &p.bgLayer[i];
            Background {
                horizontal_scroll: b.hScroll,
                vertical_scroll: b.vScroll,
                map_words: b.tilemapAdr,
                tile_words: b.tileAdr,
                wider: b.tilemapWider,
                higher: b.tilemapHigher,
            }
        }),
        windows: Windows {
            first: [p.window1left, p.window1right],
            second: [p.window2left, p.window2right],
            native_flags: p.windowsel,
            layers: core::array::from_fn(|i| {
                let w = &p.windowLayer[i];
                Window {
                    first_enabled: w.window1enabled,
                    first_inverted: w.window1inversed,
                    second_enabled: w.window2enabled,
                    second_inverted: w.window2inversed,
                    logic: w.maskLogic,
                }
            }),
        },
        routing: Routing {
            enabled: p.screenEnabled,
            windowed: p.screenWindowed,
        },
        affine: Affine {
            matrix: p.m7matrix,
            horizontal_flip: p.m7xFlip,
            vertical_flip: p.m7yFlip,
            large_field: p.m7largeField,
            character_fill: p.m7charFill,
        },
        mosaic: &mosaic,
        color: ColorMath {
            clip: p.clipMode,
            prevent: p.preventMathMode,
            enabled: p.mathEnabled,
            add_subscreen: p.addSubscreen,
            subtract: p.subtractColor,
            half: p.halfColor,
            fixed: [p.fixedColorR, p.fixedColorG, p.fixedColorB],
            brightness: p.brightness,
        },
    }
    .render(
        1,
        Overflow {
            range: p.rangeOver,
            time: p.timeOver,
        },
    )
    .unwrap()
}
// No movie injects an indexed-media error or checked panic after a completed sprite span.
pub(crate) struct RasterReads {
    pub reads: std::cell::RefCell<Vec<usize>>,
    pub stop: usize,
    pub panic: bool,
}
impl crate::rendering::IndexedSource for RasterReads {
    fn word(&self, index: usize) -> Result<u16, crate::rendering::RasterFault> {
        let mut reads = self.reads.borrow_mut();
        reads.push(index);
        if reads.len() == self.stop {
            if self.panic {
                let empty: Vec<u16> = Vec::new();
                return Ok(empty[index]);
            }
            return Err(crate::rendering::RasterFault::Word(index));
        }
        Ok(if index & 8 == 0 { 0xff } else { 0 })
    }
    fn palette(&self, _: u8) -> Result<u16, crate::rendering::RasterFault> {
        Ok(31)
    }
}
