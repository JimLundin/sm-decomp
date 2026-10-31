//! Authored physical-byte contracts for map seams and lazy content failures.
//! These run the owned rules, never the unaligned predecessor Rust paths.
#![forbid(unsafe_code)]
use super::Runtime;
use crate::{game::player_update, game_state::Game, ram_map::words as w, rom::RomImage};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(crate) const COUNT: usize = 4;
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn rom_bytes(image: &mut [u8], address: u32, bytes: &[u8]) {
    let at = crate::rom::lorom_offset(address) as usize;
    image[at..at + bytes.len()].copy_from_slice(bytes);
}
/// Seeds RAM and the loaded image; the caller authors the native bytes, then
/// registers them with `install`.
fn seed(g: &mut Game, x: u16, map_x: u16, native_length: usize) -> Vec<u8> {
    g.ram().fill(0..g.ram().len(), 0);
    g.g_fail = false;
    for index in 0..96 {
        g.ram()
            .copy_in(0xc608 + 2 * index, &(0x5000 + index as u16).to_le_bytes());
    }
    for (at, value) in [
        (w::SAMUS_X_POS, x),
        (w::SAMUS_Y_POS, 256),
        (w::ROOM_WIDTH_IN_BLOCKS, 64),
        (w::ROOM_HEIGHT_IN_BLOCKS, 32),
        (w::ROOM_X_COORDINATE_ON_MAP, map_x),
        (w::ROOM_Y_COORDINATE_ON_MAP, 0),
        (w::HAS_AREA_MAP, 1),
    ] {
        g.ram().copy_in(at, &(value as u16).to_le_bytes());
    }
    g.ram().write_byte(0x5b5, 8);
    let mut loaded = vec![0; 0x200000];
    let mut native = vec![0; native_length];
    rom_bytes(&mut loaded, 0x829717, &0x9000u16.to_le_bytes());
    rom_bytes(&mut native, 0x82964a, &[0, 0x80, 0x90, 0]);
    g.register_rom(RomImage::from(loaded));
    native
}
fn install(g: &mut Game, native: Vec<u8>) {
    g.register_native_rom(RomImage::from(native));
}
fn check_half_crossing(g: &mut Game, reverse: bool) {
    let mut native = seed(g, 128, if reverse { 32 } else { 31 }, 0x200000);
    let (columns, expected_tiles) = if reverse {
        // Columns30,31 are in the previous half;32,33,34 are in the current half.
        for offset in [7, 11, 15] {
            rom_bytes(&mut native, 0x829000 + offset, &[3]);
        }
        for offset in [132, 136, 140] {
            rom_bytes(&mut native, 0x829000 + offset, &[0xe0]);
        }
        for (offset, byte) in [(7, 1), (11, 2), (15, 3)] {
            g.ram().write_byte(0x7f7 + offset, byte);
        }
        (
            [62usize, 63, 1056, 1057, 1058],
            [
                [0x6c01, 0x6802, 0x6c03, 0x6c04, 0x6c05],
                [0x6806, 0x6c07, 0x6808, 0x6c09, 0x6c0a],
                [0x680b, 0x680c, 0x6c0d, 0x6c0e, 0x6c0f],
            ],
        )
    } else {
        // Columns29,30,31 precede the source-row wrap;32,33 follow it.
        for offset in [7, 11, 15] {
            rom_bytes(&mut native, 0x829000 + offset, &[7]);
        }
        for offset in [132, 136, 140] {
            rom_bytes(&mut native, 0x829000 + offset, &[0xc0]);
        }
        for (offset, byte) in [(132, 0x40), (136, 0x80), (140, 0xc0)] {
            g.ram().write_byte(0x7f7 + offset, byte);
        }
        (
            [61usize, 62, 63, 1056, 1057],
            [
                [0x6c01, 0x6c02, 0x6c03, 0x6c04, 0x6805],
                [0x6c06, 0x6c07, 0x6808, 0x6809, 0x6c0a],
                [0x6c0b, 0x6c0c, 0x6c0d, 0x680e, 0x680f],
            ],
        )
    };
    for row in 0..3 {
        for column in 0..5 {
            let index = columns[column] + 32 * row;
            rom_bytes(
                &mut native,
                0x908000 + 2 * index as u32,
                &(0x4001u16 + (row * 5 + column) as u16).to_le_bytes(),
            );
        }
    }
    install(g, native);
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    if reverse {
        expected[0x87f] = 0x80;
    } else {
        expected[0x802] = 1;
    }
    for row in 0..3 {
        for column in 0..5 {
            put(
                &mut expected,
                0xc608 + 2 * ([26, 58, 90][row] + column),
                expected_tiles[row][column],
            );
        }
    }
    {
        let mut runtime = Runtime::new(g);
        player_update::minimap::update(&mut runtime);
    }
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "map half crossing and source-row wrap reverse={reverse}"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
}
fn check_lazy_tiles(g: &mut Game, fail_visited: bool) {
    // Only center tile66 needs content. Later row2/unvisited tile reads would
    // exceed this allocation; an eager row/tile copy is observably wrong.
    let tile_base = crate::rom::lorom_offset(0x908000) as usize;
    let length = tile_base + if fail_visited { 133 } else { 134 };
    let mut native = seed(g, 512, 0, length);
    native[tile_base + 132] = 6;
    if !fail_visited {
        native[tile_base + 133] = 0x40;
    }
    install(g, native);
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    expected[0x7ff] = 0x20;
    for column in 0..5 {
        for row in 0..3 {
            if !fail_visited || column * 3 + row < 8 {
                put(
                    &mut expected,
                    0xc608 + 2 * ([26, 58, 90][row] + column),
                    0x2c1f,
                );
            }
        }
    }
    if !fail_visited {
        put(&mut expected, 0xc608 + 2 * 60, 0x6806);
    }
    let matched_source = Arc::new(AtomicBool::new(false));
    let previous_hook = if fail_visited {
        let previous = Arc::new(std::panic::take_hook());
        let forward = previous.clone();
        let observed = matched_source.clone();
        std::panic::set_hook(Box::new(move |info| {
            if info
                .location()
                .is_some_and(|at| at.file().ends_with("cartridge.rs"))
            {
                observed.store(true, Ordering::Relaxed);
            } else {
                forward(info);
            }
        }));
        Some(previous)
    } else {
        None
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut runtime = Runtime::new(g);
        player_update::minimap::update(&mut runtime);
    }));
    if let Some(previous) = previous_hook {
        drop(std::panic::take_hook());
        std::panic::set_hook(
            Arc::try_unwrap(previous)
                .ok()
                .expect("released scoped map panic hook"),
        );
    }
    if fail_visited {
        assert!(
            result.is_err(),
            "visited content must fail on its missing second byte"
        );
        assert!(
            matched_source.load(Ordering::Relaxed),
            "visited failure must originate in the cartridge image read"
        );
    } else if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "lazy map tiles and mapped HUD failure prefix fail={fail_visited}"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
}
pub(crate) fn run(g: &mut Game) -> usize {
    assert!(!g.actor_identity_is_with_session());
    let ram = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let fail = g.g_fail;
    let loaded = g.loaded_rom.clone();
    let native = g.capture_native_rom_binding();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        check_half_crossing(g, false);
        check_half_crossing(g, true);
        check_lazy_tiles(g, false);
        check_lazy_tiles(g, true);
        COUNT
    }));
    g.ram().copy_in(0, &ram);
    g.g_fail = fail;
    g.loaded_rom = loaded;
    g.restore_native_rom_binding(native);
    match result {
        Ok(count) => count,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
