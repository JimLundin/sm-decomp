//! Synthetic resource backing for the authored fixture on actual hardware.
//! Install only after real SnesInit in an isolated child. No cartridge data emitted.
#![forbid(unsafe_code)]
use crate::game::room_entry::{
    content::{AssetRef, AssetSource, SharedAsset},
    effects::{EntryEffects, InstallBuffer},
    world::*,
};
use crate::{game_state::Game, rom::RomImage};
use std::sync::Arc;

fn write(image: &RomImage, address: u32, bytes: &[u8]) {
    let base = crate::rtl::lorom_offset(address) as usize;
    for (offset, &byte) in bytes.iter().enumerate() {
        image.write(base + offset, byte);
    }
}
fn fill_commands(count: usize, value: u8, out: &mut Vec<u8>) {
    let mut left = count;
    while left != 0 {
        let length = left.min(1024);
        let encoded = length - 1;
        out.extend_from_slice(&[0xe4 | ((encoded >> 8) as u8), encoded as u8, value]);
        left -= length;
    }
}
fn level(tile: u16) -> Vec<u8> {
    // 6x6 screens:18432 level bytes,9216 BTS,18432 layer2 plus2-byte size.
    // First tile distinguishes actual selected/install paths after decompression.
    let [low, high] = tile.to_le_bytes();
    let mut bytes = vec![3, 0, 0x48, low, high]; // Four literal bytes.
    fill_commands(46078, 0, &mut bytes);
    bytes.push(0xff);
    bytes
}
pub(crate) fn install(game: &mut Game) {
    assert!(!game.g_snes.is_null(), "room.world.hardware-initialized");
    let loaded = game.loaded_rom.clone().expect("real SnesInit loaded ROM");
    let native = game.capture_native_rom();
    for image in [&loaded, &native] {
        for (address, tile) in [(0xc08000, 0x0101), (0xc08100, 0x0202), (0xc08200, 0x0303)] {
            write(image, address, &level(tile));
        }
        for (address, count) in [(0xc08300, 6144), (0xc08400, 4096), (0xc08500, 512)] {
            let mut bytes = Vec::new();
            fill_commands(count, 0, &mut bytes);
            bytes.push(0xff);
            write(image, address, &bytes);
        }
        if game
            .world()
            .prepared_table(&AssetSource::Shared(SharedAsset::CreTileTable))
            .is_some()
        {
            write(image, 0xb9a09d, &table_stream(2048, 0));
            write(image, 0xc08302, &[0x55]);
            write(image, 0xb9a09f, &[0x55]);
        }
        if game.world().prepared_room_assets().is_some() {
            // Complete packs remain authoritative after the mutable source changes.
            for address in [0xc08000, 0xc08100, 0xc08200, 0xc08400, 0xc08500, 0xb98000] {
                write(image, address, &[0xff]);
            }
        }
        // Both retained enemy-list consumers stop onFFFF; quota follows population.
        write(image, 0xa19000, &[0xff, 0xff, 0]);
        write(image, 0xb49000, &[0xff, 0xff]);
    }
}

fn table_stream(count: usize, value: u8) -> Vec<u8> {
    let mut stream = Vec::new();
    fill_commands(count, value, &mut stream);
    stream.push(0xff);
    stream
}
fn table_source(table: &[u8]) -> Vec<u8> {
    let at = crate::rom::lorom_offset(0xc08300) as usize;
    let mut source = vec![0xff; at + table.len()];
    source[at..].copy_from_slice(table);
    let cre = table_stream(2048, 0);
    let at = crate::rom::lorom_offset(0xb9a09d) as usize;
    source[at..at + cre.len()].copy_from_slice(&cre);
    source
}
pub(crate) fn tables(
    world: &WorldDefinition,
    keys: &CompatibilityKeys,
    value: u8,
) -> PreparedTileTables {
    PreparedTileTables::import(&table_source(&table_stream(6144, value)), world, keys).unwrap()
}
pub(crate) fn binding_with_tables(swapped: bool) -> Arc<WorldBinding> {
    let (mut world, keys) = crate::room_world_fixture::definition(swapped);
    for room in world.rooms.values_mut() {
        room.cre = 2;
    }
    let tables = tables(&world, &keys, 0);
    let bytes = tables.to_bytes();
    let loaded = PreparedTileTables::load(&bytes, &world, &keys).unwrap();
    assert_eq!(loaded.to_bytes(), bytes, "room.tables.canonical-pack");
    WorldBinding::prepared_with_tables(world, keys, loaded).unwrap()
}
// Movies cannot construct native packs or malformed catalog/command variants.
fn check_pack() {
    let (world, keys) = crate::room_world_fixture::definition(false);
    let tables = tables(&world, &keys, 0);
    let pack = tables.to_bytes();
    let cre_length = u32::from_le_bytes(pack[53..57].try_into().unwrap()) as usize;
    for kind in 0..8 {
        let mut bad = pack.clone();
        match kind {
            0 => {
                bad.pop();
            }
            1 => bad.push(0),
            2 => bad[4] = 2,
            3 => bad[44..48].copy_from_slice(&1u32.to_le_bytes()),
            4 => bad[44..48].copy_from_slice(&1026u32.to_le_bytes()),
            5 => bad[57 + cre_length..62 + cre_length].copy_from_slice(&[0; 5]),
            6 => bad[53..57].copy_from_slice(&262145u32.to_le_bytes()),
            _ => bad[57] = 0xff,
        }
        assert!(
            PreparedTileTables::load(&bad, &world, &keys).is_err(),
            "room.tables.pack-reject-{kind}"
        );
    }
    let mut different = world.clone();
    different.rooms.values_mut().next().unwrap().map_x += 1;
    assert!(PreparedTileTables::load(&pack, &different, &keys).is_err());
    assert!(WorldBinding::prepared_with_tables(different, keys.clone(), tables.clone()).is_err());
    assert!(PreparedTileTables::import(&[], &world, &keys).is_err());
    assert!(
        PreparedTileTables::import(&table_source(&table_stream(0x15801, 0)), &world, &keys)
            .is_err(),
        "room.tables.synthetic-terminator-rejected"
    );
    assert!(
        PreparedTileTables::import(&table_source(&[0, 0xff]), &world, &keys).is_err(),
        "room.tables.literal-ff-needs-real-terminator"
    );
}
// Movies cannot install these authored live-overlap, self-reference and unproduced-prefix streams.
fn check_references() {
    let (world, keys) = crate::room_world_fixture::definition(false);
    for (stream, expected) in [
        (
            vec![0, 0x11, 0x82, 0, 0, 0x80, 0x10, 0, 0xff],
            vec![0x11, 0x11, 0x11, 0x11, 0x77],
        ),
        (vec![0xc0, 0, 0xff], vec![0x42]),
        (vec![0, 0xff, 0xff], vec![0xff]),
    ] {
        let tables = PreparedTileTables::import(&table_source(&stream), &world, &keys).unwrap();
        let binding =
            WorldBinding::prepared_with_tables(world.clone(), keys.clone(), tables).unwrap();
        let mut sample = Game::with_world(binding);
        sample.decompress_src = 0xdeadbeef;
        sample.ram().write_byte(0xa800, 0x42);
        sample.ram().write_byte(0xa810, 0x77);
        EntryEffects::decompress(
            &mut crate::game::session_runtime::Runtime::new(&mut sample),
            AssetSource::Bound(AssetRef::Prepared(crate::room_world_fixture::TILE_TABLE)),
            InstallBuffer::TileTableAfterCre,
        );
        assert_eq!(
            (0..expected.len())
                .map(|i| sample.ram().read_byte(0xa800 + i))
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            sample.decompress_src, 0xdeadbeef,
            "room.tables.legacy-cursor-history"
        );
    }
}
// Movies cannot bind immutable tables before later shared ROM fixture mutations.
pub(crate) fn check_tables(g: &mut Game) {
    check_pack();
    check_references();
    let world = g.world().clone();
    let cre = world
        .prepared_table(&AssetSource::Shared(SharedAsset::CreTileTable))
        .unwrap();
    let cursor = g.decompress_src;
    let mut runtime = crate::game::session_runtime::Runtime::new(g);
    EntryEffects::decompress(
        &mut runtime,
        AssetSource::Shared(SharedAsset::CreTileTable),
        InstallBuffer::TileTablePrimary,
    );
    EntryEffects::decompress(
        &mut runtime,
        AssetSource::Bound(AssetRef::Prepared(crate::room_world_fixture::TILE_TABLE)),
        InstallBuffer::TileTableAfterCre,
    );
    drop(runtime);
    assert_eq!(g.decompress_src, cursor, "room.tables.both-install-cursor");
    assert!(Arc::ptr_eq(g.world(), &world));
    assert!(Arc::ptr_eq(
        &cre,
        &g.world()
            .prepared_table(&AssetSource::Shared(SharedAsset::CreTileTable))
            .unwrap()
    ));
    assert_eq!(
        (g.ram().read_byte(0xa000), g.ram().read_byte(0xa800)),
        (0, 0),
        "room.tables.immutable-after-ROM-mutation"
    );
}

/// Overlay only the existing authored fixture into caller-owned independent ROM bytes.
/// The original raw windows remain exact; no shared RomImage or fixture copy is used.
pub(crate) fn prepared_assets(
    source: &[u8],
    world: &WorldDefinition,
    keys: &CompatibilityKeys,
) -> Result<PreparedRoomAssets, PreparedRoomError> {
    let mut source = source.to_vec();
    let mut put = |address, bytes: &[u8]| -> Result<(), PreparedRoomError> {
        let at = crate::rom::lorom_offset(address) as usize;
        source
            .get_mut(at..at + bytes.len())
            .ok_or(PreparedRoomError::MissingPayload)?
            .copy_from_slice(bytes);
        Ok(())
    };
    for (address, tile) in [(0xc08000, 0x0101), (0xc08100, 0x0202), (0xc08200, 0x0303)] {
        put(address, &level(tile))?;
    }
    for (address, count) in [
        (0xc08300, 6144),
        (0xc08400, 4096),
        (0xc08500, 512),
        (0xb9a09d, 2048),
        (0xb98000, 8192),
    ] {
        put(address, &table_stream(count, 0))?;
    }
    PreparedRoomAssets::import(&source, world, keys)
}
// Movies cannot construct checksum-valid malformed immutable asset packs.
fn check_asset_pack(
    world: &WorldDefinition,
    keys: &CompatibilityKeys,
    assets: &PreparedRoomAssets,
) {
    let bytes = assets.to_bytes();
    assert_eq!(
        PreparedRoomAssets::load(&bytes, world, keys)
            .unwrap()
            .to_bytes(),
        bytes
    );
    let mut bad = bytes.clone();
    let table_length = u32::from_le_bytes(bad[48..52].try_into().unwrap()) as usize;
    let first = 52 + table_length + 4;
    let stream_length = u32::from_le_bytes(bad[first + 5..first + 9].try_into().unwrap()) as usize;
    assert!(stream_length > 1);
    bad[first + 9] = 0xff; // Real early terminator leaves trailing encoded payload.
    let end = bad.len() - 32;
    let digest = PreparedRoomAssets::checksum(&bad[..end]);
    bad[end..].copy_from_slice(&digest);
    assert!(
        matches!(
            PreparedRoomAssets::load(&bad, world, keys),
            Err(PreparedRoomError::Malformed)
        ),
        "room.assets.checksum-valid-structural-reject"
    );
}
// Movies cannot produce a self-referenced live header with only2 decoded bytes.
fn check_live_level(
    world: &WorldDefinition,
    keys: &CompatibilityKeys,
    assets: &PreparedRoomAssets,
) {
    let mut bytes = assets.to_bytes();
    let table_length = u32::from_le_bytes(bytes[48..52].try_into().unwrap()) as usize;
    let mut at = 52 + table_length + 4;
    let count = u32::from_le_bytes(bytes[at - 4..at].try_into().unwrap());
    for _ in 0..count {
        let id = u32::from_le_bytes(bytes[at + 1..at + 5].try_into().unwrap());
        let length = u32::from_le_bytes(bytes[at + 5..at + 9].try_into().unwrap()) as usize;
        if bytes[at] == 1 && id == crate::room_world_fixture::LEVEL_A.0 {
            bytes[at + 5..at + 9].copy_from_slice(&4u32.to_le_bytes());
            bytes.splice(at + 9..at + 9 + length, [0x81, 0, 0, 0xff]);
            break;
        }
        at += 9 + length;
    }
    let end = bytes.len() - 32;
    let digest = PreparedRoomAssets::checksum(&bytes[..end]);
    bytes[end..].copy_from_slice(&digest);
    let altered = PreparedRoomAssets::load(&bytes, world, keys).unwrap();
    let binding =
        WorldBinding::prepared_with_room_assets(world.clone(), keys.clone(), altered).unwrap();
    for size in [2u16, 4, 0x6a00] {
        let mut game = Game::with_world(binding.clone());
        game.decompress_src = 0xdeadbeef;
        game.ram().copy_in(0x10000, &size.to_le_bytes());
        for i in 0..0x20000 {
            if i >= 0x10002 {
                game.ram().write_byte(i, 0x5a);
            }
        }
        if size == 0x6a00 {
            for i in 0x19f02..0x20000 {
                game.ram().write_byte(i, 0xa5);
            }
            game.ram().write_byte(0x1f700, 0xcc);
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            EntryEffects::clear_buffer(
                &mut crate::game::session_runtime::Runtime::new(&mut game),
                crate::game::room_entry::effects::BufferClear::InitialLevel,
            );
            EntryEffects::decode_level_and_copy_planes(
                &mut crate::game::session_runtime::Runtime::new(&mut game),
                AssetSource::Bound(AssetRef::Prepared(crate::room_world_fixture::LEVEL_A)),
            );
        }));
        assert_eq!(game.decompress_src, 0xdeadbeef);
        assert_eq!(
            u16::from_le_bytes([game.ram().read_byte(0x10000), game.ram().read_byte(0x10001)]),
            size
        );
        assert_eq!(result.is_ok(), size != 0x6a00);
        if size == 0x6a00 {
            assert_eq!(game.ram().read_byte(0x19602), 0xa5);
            assert_eq!(game.ram().read_byte(0x1f6ff), 0xa5);
            assert_eq!(game.ram().read_byte(0x1f700), 0xcc);
            assert_eq!(game.ram().read_byte(0x16402), 0x5a);
        } else {
            assert_eq!(
                game.ram().read_byte(0x19602),
                if size == 2 { 0x80 } else { 0 }
            );
            assert_eq!(game.ram().read_byte(0x19602 + usize::from(size)), 0x5a);
            assert_eq!(game.ram().read_byte(0x16402 + usize::from(size / 2)), 0x5a);
        }
    }
}
// Movies cannot select malformed external packs or inspect immutable-catalog provenance.
pub(crate) fn check_assets(game: &mut Game) {
    let world = game.world().clone();
    let assets = world.prepared_room_assets().unwrap();
    let definition = world.definition().unwrap();
    check_asset_pack(definition, world.compatibility_keys(), &assets);
    check_live_level(definition, world.compatibility_keys(), &assets);
    check_fx_assets(definition, world.compatibility_keys(), &assets);
    assert!(Arc::ptr_eq(game.world(), &world));
}

/// Existing external producer: the outer importer supplies normalized independent bytes.
pub fn verify_room_pack_bytes(
    source: &[u8],
    alternate: bool,
    changed_payload: bool,
) -> Result<Vec<u8>, String> {
    let (mut world, keys) =
        crate::room_world_fixture::definition_with_application_starts(alternate);
    for room in world.rooms.values_mut() {
        room.cre = 2;
    }
    let mut assets = prepared_assets(source, &world, &keys).map_err(|error| error.to_string())?;
    if changed_payload {
        let mut bytes = assets.to_bytes();
        let tables = u32::from_le_bytes(bytes[48..52].try_into().unwrap()) as usize;
        let mut at = 52 + tables + 4;
        let count = u32::from_le_bytes(bytes[at - 4..at].try_into().unwrap());
        for _ in 0..count {
            let id = u32::from_le_bytes(bytes[at + 1..at + 5].try_into().unwrap());
            let length = u32::from_le_bytes(bytes[at + 5..at + 9].try_into().unwrap()) as usize;
            if bytes[at] == 1 && id == crate::room_world_fixture::PALETTE.0 {
                bytes[at + 11] ^= 1;
                break;
            }
            at += 9 + length;
        }
        let end = bytes.len() - 32;
        let hash = PreparedRoomAssets::checksum(&bytes[..end]);
        bytes[end..].copy_from_slice(&hash);
        assets =
            PreparedRoomAssets::load(&bytes, &world, &keys).map_err(|error| error.to_string())?;
    }
    crate::room_pack::to_bytes(&world, &keys, &assets).map_err(|error| error.to_string())
}
// Movies do not select all8 odd/wrapped FX slots or replace the backing ROM after import.
fn check_fx_assets(world: &WorldDefinition, keys: &CompatibilityKeys, assets: &PreparedRoomAssets) {
    let mut world = world.clone();
    let mut keys = keys.clone();
    let id = ResourceId(0x6200_0009);
    let base = 0xffb1u16;
    world.resources.insert(id, ResourceKind::Fx);
    keys.resources.insert(
        id,
        ResourceKey::Word {
            bank: 0x83,
            offset: base,
        },
    );
    world.states.values_mut().next().unwrap().fx = Some(id);
    let mut source = vec![0; 0x400000];
    let (_, raw) = assets.raw_window();
    source[..raw.len()].copy_from_slice(raw);
    for index in 0..8u16 {
        let at =
            crate::rom::lorom_offset(0x830000 | u32::from(base.wrapping_add(index * 16))) as usize;
        source[at..at + 16].fill(0);
        source[at..at + 2]
            .copy_from_slice(&if index == 7 { 0u16 } else { 0x1000 + index }.to_ne_bytes());
        source[at + 2..at + 4].copy_from_slice(&(0x2000 + index).to_ne_bytes());
        source[at + 4..at + 6].copy_from_slice(&(0x3000 + index).to_ne_bytes());
        source[at + 8] = index as u8;
    }
    let prepared = prepared_assets(&source, &world, &keys).unwrap();
    let binding = WorldBinding::prepared_with_room_assets(world, keys, prepared).unwrap();
    source.fill(0xff);
    let mut game = Game::with_world(binding);
    game.decompress_src = 0xdeadbeef;
    game.ram().copy_in(
        crate::ram_map::words::room_layer3_asm_ptr,
        &base.to_le_bytes(),
    );
    for index in 0..16u16 {
        game.g_fail = false;
        crate::game::room_state::load_fx_entry(index, &mut game);
        let h = crate::game::session_runtime::Runtime::new(&mut game);
        assert_eq!(h.session().room.liquid_height, 0x2000 + (index & 7));
        assert_eq!(h.session().room.liquid_target, 0x3000 + (index & 7));
        drop(h);
        assert_eq!(
            game.g_fail,
            (index & 7) >= 5,
            "room.assets.fx-low-window-diagnostic"
        );
    }
    game.g_fail = false;
    game.ram().copy_in(
        crate::ram_map::words::door_def_ptr,
        &0x1003u16.to_le_bytes(),
    );
    crate::game::room_state::load_fx_header(&mut game);
    assert_eq!(
        crate::game::session_runtime::Runtime::new(&mut game)
            .session()
            .room
            .liquid_height,
        0x2003
    );
    assert_eq!(game.decompress_src, 0xdeadbeef);
}
// Movies cannot replace an immutable encoded resource without altering its world/keys.
pub(crate) fn replace_stream(
    assets: &PreparedRoomAssets,
    id: ResourceId,
    stream: &[u8],
    world: &WorldDefinition,
    keys: &CompatibilityKeys,
) -> PreparedRoomAssets {
    let mut bytes = assets.to_bytes();
    let tables = u32::from_le_bytes(bytes[48..52].try_into().unwrap()) as usize;
    let mut at = 52 + tables + 4;
    let count = u32::from_le_bytes(bytes[at - 4..at].try_into().unwrap());
    let mut found = false;
    for _ in 0..count {
        let length = u32::from_le_bytes(bytes[at + 5..at + 9].try_into().unwrap()) as usize;
        if bytes[at] == 1 && u32::from_le_bytes(bytes[at + 1..at + 5].try_into().unwrap()) == id.0 {
            bytes[at + 5..at + 9].copy_from_slice(&(stream.len() as u32).to_le_bytes());
            bytes.splice(at + 9..at + 9 + length, stream.iter().copied());
            found = true;
            break;
        }
        at += 9 + length;
    }
    assert!(found);
    let end = bytes.len() - 32;
    let hash = PreparedRoomAssets::checksum(&bytes[..end]);
    bytes[end..].copy_from_slice(&hash);
    PreparedRoomAssets::load(&bytes, world, keys).unwrap()
}
