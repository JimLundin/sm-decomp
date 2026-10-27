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
