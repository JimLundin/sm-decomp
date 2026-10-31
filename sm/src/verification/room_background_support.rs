//! Actual device initialization and field-wise observation, synthetic assets only.
use crate::{
    game_state::{Game, WorkRam},
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
pub(crate) unsafe fn initialized_with_world(
    world: std::sync::Arc<sm_game::room_entry::world::WorldBinding>,
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
        let cart_image = crate::snes::cart::CartRom::from(
            (0..image.len()).map(|i| image.read(i)).collect::<Vec<_>>(),
        );
        crate::snes::cart::cart_load((*g.g_snes).cart, 1, cart_image, 8192);
        // The engine reads its own copy, as snes_init registers it.
        g.register_native_rom(image);
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

use sm_game::room_entry::{
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
        use sm_game::room_entry::{
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
            sm_game::room_entry::effects::DeviceByte::VramIncrement,
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
        use sm_game::room_entry::{content::VramDestination, effects::VramSlot};
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
