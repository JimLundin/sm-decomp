//! Synthetic predecessor sprite-pool fixtures; no cartridge-derived bytes.
#![forbid(unsafe_code)]
use crate::{
    game_state::{Game, WorkRam},
    ram_map::words as w,
    rom::RomImage,
};
#[derive(Clone, Copy)]
pub(crate) enum Operation {
    PopulationClear,
    RetireAll,
    Spawn,
    SpawnTwice,
    Update,
    UpdateTwice,
    Draw,
    MetroidInit,
    MetroidHurt,
    MetroidMain,
    MetroidFrozen,
    MawAppearance,
    DraygonMotion(u16),
    DraygonStart,
    DraygonFour,
}
pub(crate) const CASES: u32 = 117;
pub(crate) fn put(g: &mut Game, at: usize, value: u16) {
    put_parts(g.ram(), at, value);
}
pub(crate) fn put_parts(ram: &WorkRam, at: usize, value: u16) {
    let _ = &[(); 0x20000][at..at + 2];
    ram.copy_in(at, &value.to_le_bytes());
}
pub(crate) fn word(image: &RomImage, address: u32, value: u16) {
    let at = crate::rtl::lorom_offset(address) as usize;
    for (i, b) in value.to_ne_bytes().into_iter().enumerate() {
        image.write(at + i, b);
    }
}
fn slot(g: &mut Game, column: usize, index: usize, value: u16) {
    put(g, column + 2 * index, value);
}
fn seed(id: u32, g: &mut Game) -> Operation {
    g.ram().fill(0..g.ram().len(), 0);
    for index in 0..512 {
        put(g, 0xef78 + index * 2, 0x5000 ^ (index as u16));
    }
    for index in 0..32 {
        slot(g, 0xef78, index, 0);
    }
    put(g, 0x1844, 0xbeef);
    put(g, 0xf378, 0xcafe);
    let image = RomImage::zeroed(0x200000);
    for selector in 0..64 {
        word(&image, 0xb4bda8 + 2 * selector, 0x9000);
    }
    for address in [0x9000u16, 0x9100, 0x9200] {
        word(&image, 0xb40000 | u32::from(address), 7);
        word(&image, 0xb40000 | u32::from(address + 2), 0xa000);
    }
    // One actual backend spritemap record. OAM bytes expose submission order.
    word(&image, 0xb4a000, 1);
    word(&image, 0xb4a002, 0);
    image.write(crate::rtl::lorom_offset(0xb4a004) as usize, 0);
    word(&image, 0xb4a005, 3);
    g.loaded_rom = Some(image.clone());
    g.register_native_rom(image.clone());
    match id {
        0 => {
            slot(g, 0xef78, 0, 0xa55a);
            Operation::PopulationClear
        }
        1 => {
            for i in 0..32 {
                slot(g, 0xef78, i, 0x9000 + i as u16);
            }
            Operation::RetireAll
        }
        2..=7 => {
            if id != 2 && id != 7 {
                for i in 0..32 {
                    slot(g, 0xef78, i, 0x9000);
                }
            }
            match id {
                4 => slot(g, 0xef78, 0, 0),
                5 => {
                    slot(g, 0xef78, 3, 0);
                    slot(g, 0xef78, 17, 0);
                }
                6 => slot(g, 0xef78, 31, 0),
                _ => (),
            }
            if id == 7 {
                Operation::SpawnTwice
            } else {
                Operation::Spawn
            }
        }
        8..=11 => {
            if id == 8 {
                g.loaded_rom = None;
            } else if id == 9 {
                g.loaded_rom = Some(RomImage::zeroed(
                    crate::rtl::lorom_offset(0xb4bda8) as usize + 1,
                ));
            } else {
                let short = RomImage::zeroed(
                    crate::rtl::lorom_offset(0xb4f000) as usize + (id - 10) as usize,
                );
                word(&short, 0xb4bda8, 0xf000);
                g.loaded_rom = Some(short);
            }
            Operation::Spawn
        }
        12..=15 => {
            slot(g, 0xef78, 31, 0x9000);
            slot(g, 0xeff8, 31, 1);
            put(
                g,
                if id < 14 {
                    w::time_is_frozen_flag
                } else {
                    w::debug_time_frozen_for_enemies
                },
                if id & 1 == 0 { 1 } else { 0x8000 },
            );
            g.loaded_rom = None;
            Operation::Update
        }
        16..=39 => {
            let n = (id - 16) as usize;
            slot(g, 0xef78, 31, 0x8ffc);
            slot(g, 0xeff8, 31, [0, 1, 2, 0x7fff, 0xbcf0, 0xbd07][n % 6]);
            slot(g, 0xf2f8, 31, [0, 1, 2, 0x8000][n / 6]);
            Operation::Update
        }
        40..=47 => {
            let n = id - 40;
            slot(g, 0xef78, 31, 0x9000);
            slot(g, 0xeff8, 31, 0xbd12);
            slot(g, 0xf2f8, 31, 0);
            word(&image, 0xb49002, if n >= 4 { 0x9100 } else { 0x9200 });
            word(
                &image,
                if n >= 4 { 0xb49100 } else { 0xb49200 },
                [7, 0xbcf0, 0xbd07, 1][(n % 4) as usize],
            );
            if n == 7 {
                word(&image, 0xb49104, 0xbd07);
            }
            if n >= 4 {
                Operation::UpdateTwice
            } else {
                Operation::Update
            }
        }
        48..=71 => {
            let n = (id - 48) as usize;
            slot(g, 0xef78, 31, 0x9000);
            slot(
                g,
                0xf0f8,
                31,
                [
                    0xffef, 0xfff0, 0xffff, 0, 255, 256, 271, 272, 0x7fff, 0x8000, 0xfff0, 0xffff,
                ][n % 12],
            );
            slot(g, 0xf1f8, 31, if n < 12 { 20 } else { 260 });
            slot(g, 0xf078, 31, 0xfbe7);
            Operation::Draw
        }
        72 => {
            for (i, x) in [(0, 31), (5, 47), (31, 63)] {
                slot(g, 0xef78, i, 0x9000);
                slot(g, 0xf0f8, i, x);
                slot(g, 0xf1f8, i, 30);
                slot(g, 0xf078, i, 0xe00 + i as u16);
                slot(g, 0xf2f8, i, 1);
            }
            Operation::Draw
        }
        73..=75 => {
            slot(g, 0xef78, 31, 0x9000);
            slot(g, 0xf0f8, 31, 30);
            slot(g, 0xf1f8, 31, if id == 73 { 0xffff } else { 272 });
            if id == 75 {
                g.loaded_rom = None;
            }
            Operation::Draw
        }
        76 => {
            slot(g, 0xef78, 31, 0x1000);
            slot(g, 0xf0f8, 31, 30);
            slot(g, 0xf1f8, 31, 30);
            Operation::Draw
        }
        77 => {
            word(&image, 0xb4bda8, 0x1000);
            Operation::Spawn
        }
        78 => {
            slot(g, 0xef78, 31, 0x0ffc);
            slot(g, 0xeff8, 31, 1);
            slot(g, 0xf2f8, 31, 0);
            Operation::Update
        }
        79 => {
            slot(g, 0xef78, 31, 0x1000);
            slot(g, 0xeff8, 31, 0xbd12);
            slot(g, 0xf2f8, 31, 0);
            word(&image, 0xb49002, 0x1200);
            Operation::Update
        }
        80..=110 => super::sprite_controller_cases::seed(id, g, &image),
        111..=116 => super::sprite_effect_observer::seed(id, g, &image),
        _ => unreachable!(),
    }
}
pub(crate) fn capture_case(id: u32, run: fn(Operation, &mut Game) -> u16) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let mut game = Game::new();
    let operation = seed(id, &mut game);
    super::sprite_effect_observer::begin(id);
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(operation, &mut game)));
    let events = super::sprite_effect_observer::finish();
    assert_eq!(
        result.is_err(),
        (8..=11).contains(&id),
        "sprite.pool.{id}.panic"
    );
    assert_eq!(
        game.g_fail,
        (76..=79).contains(&id) || matches!(id, 111..=113 | 115),
        "sprite.pool.{id}.diagnostic"
    );
    let mut bytes = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<_>>();
    bytes.extend_from_slice(&result.unwrap_or(0).to_le_bytes());
    bytes.push(u8::from((8..=11).contains(&id)));
    bytes.push(u8::from(game.g_fail));
    bytes.extend(events);
    Some(bytes)
}
