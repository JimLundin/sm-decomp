//! Frozen synthetic inputs for defined old Samus palette functions only.
//! The selector2 incompatible function-pointer call is deliberately never invoked.
#![forbid(unsafe_code)]
use crate::{game_state::Game, rom::RomImage};
#[derive(Clone, Copy)]
pub(crate) enum Operation {
    Advance,
    Misc,
    Visor,
    ScrewSpeed,
    Shine,
    Shinespark,
    Crystal,
    Xray,
    CopyCurrent,
    CopyTarget,
    Copy10,
    Copy6,
    LoadCurrent,
    LoadTarget,
    Cancel,
    Beam,
}
pub(crate) const CASES: u32 = 224;
fn put(game: &mut Game, at: usize, value: u16) {
    game.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn word(image: &RomImage, address: u32, value: u16) {
    let at = crate::rtl::lorom_offset(address) as usize;
    for (i, byte) in value.to_ne_bytes().into_iter().enumerate() {
        image.write(at + i, byte);
    }
}
fn seed(id: u32, game: &mut Game) -> Operation {
    game.g_ram.fill(0);
    game.g_ram[0x1ffff] = 1;
    for at in (0xc000..0xc400).step_by(2) {
        put(game, at, 0x7777);
    }
    let image = RomImage::zeroed(0x100000);
    // Every color has a distinct value, including all fixed suit sources.
    for at in (0x9b8000..0x9bfffe).step_by(2) {
        word(&image, at, (at as u16).rotate_left(3) ^ 0x1357);
    }
    for table in [0x91d727, 0x91d998, 0x91d99e, 0x91dc28] {
        for i in 0..16 {
            word(&image, table + 2 * i, 0x9000 + 32 * i as u16);
        }
    }
    // Indirect animation tables use one native bank91 row, distinct from colors.
    for table in [0x91da4a, 0x91daa9, 0x91db10, 0x91db75, 0x91d7d5, 0x91d7ff] {
        for i in 0..3 {
            word(&image, table + 2 * i, 0x9100 + 32 * i as u16);
        }
    }
    for i in 0..48 {
        word(&image, 0x919100 + 2 * i, 0x9000 + 32 * (i % 16) as u16);
    }
    for i in 0..10 {
        word(&image, 0x91dc00 + 4 * i, 0x9400 + 32 * i as u16);
        word(&image, 0x91dc02 + 4 * i, 3 + i as u16);
    }
    for i in 0..16 {
        word(&image, 0x90c3c9 + 2 * i, 0x9600 + 32 * i as u16);
    }
    for at in (0x909600..0x909820).step_by(2) {
        word(&image, at, (at as u16).rotate_left(5) ^ 0x2468);
    }
    game.loaded_rom = Some(image.clone());
    game.register_native_rom(image);
    for (at, value) in [
        (0x998, 8),
        (0xa44, 0xe913),
        (0xd32, 0xc4f0),
        (0x9b2, 0x40),
        (0x195e, 0xffff),
        (0x1962, 0xffff),
    ] {
        put(game, at, value);
    }
    let n = (id % 24) as usize;
    match id {
        0..=15 => {
            let n = id as usize;
            put(game, 0x1982, [0, 40, 42, 41][n / 4]);
            put(game, 0xa72, [0x601, 0x600, 0xa01, 0x202][n % 4]);
            if id == 15 {
                put(game, 0xacc, 8);
            }
            Operation::Visor
        }
        16..=39 => {
            let n = (id - 16) as usize;
            put(
                game,
                0xa48,
                [0, 1, 2, 6, 7, 38, 39, 58, 59, 60, 0x803a, 0xffff][n % 12],
            );
            if n >= 12 {
                put(game, 0x1f51, 1);
            }
            Operation::Misc
        }
        40..=55 => {
            let n = (id - 40) as usize;
            put(game, 0xa4a, [1, 2, 0x8000, 0xffff][n / 4]);
            put(game, 0xad0, [0, 1, 2, 0x8000][n % 4]);
            put(game, 0xace, 6);
            put(game, 0xb62, 9);
            Operation::Misc
        }
        56..=79 => {
            put(game, 0xa74, 4);
            put(game, 0xa1f, [3, 0x14, 0][n % 3]);
            put(game, 0x9a2, if n & 1 == 0 { 8 } else { 0 });
            put(game, 0xa96, [0, 2, 3, 26, 27, 28][n % 6]);
            put(game, 0xb3e, 0x400);
            put(game, 0xad0, [0, 1, 2, 0x8000][n % 4]);
            put(game, 0xace, [0, 6, 10][n % 3]);
            Operation::ScrewSpeed
        }
        80..=103 | 104..=127 => {
            let n = if id < 104 {
                (id - 80) as usize
            } else {
                (id - 104) as usize
            };
            put(
                game,
                0xa68,
                [0, 1, 2, 170, 0x8000, 0x8001, 0xffff, 169][n % 8],
            );
            put(game, 0xace, [0, 6, 10][n / 8]);
            put(game, 0xacc, if id < 104 { 1 } else { 6 });
            if id < 104 {
                Operation::Shine
            } else {
                Operation::Shinespark
            }
        }
        128..=151 => {
            let n = (id - 128) as usize;
            put(game, 0xa68, [0xffff, 0, 1, 2][n % 4]);
            put(game, 0xdf2, [0, 1, 2][n / 4 % 3]);
            put(game, 0xace, if n >= 12 { 10 } else { 0 });
            put(game, 0xad0, if n >= 12 { 36 } else { 0 });
            put(game, 0xacc, 7);
            Operation::Crystal
        }
        152..=175 => {
            let n = (id - 152) as usize;
            put(game, 0xa80, [0xffff, 0, 1][n % 3]);
            put(game, 0xa7a, if n & 1 == 0 { 1 } else { 2 });
            put(game, 0xad0, [0, 1, 2, 0x8000][n / 6]);
            put(game, 0xace, [0, 4, 6, 10][n / 6]);
            put(game, 0xacc, 8);
            Operation::Xray
        }
        176..=187 => {
            let n = (id - 176) as usize;
            put(game, 0x9a2, [0, 1, 0x20, 0x21][n % 4]);
            match n / 4 {
                0 => Operation::LoadCurrent,
                1 => Operation::LoadTarget,
                _ => Operation::Cancel,
            }
        }
        188..=195 => {
            let n = id - 188;
            put(game, 0xb3c, if n & 1 == 0 { 1 } else { 0 });
            put(game, 0xb3e, 0x400);
            put(game, 0xace, 6);
            put(game, 0xad0, 4);
            put(game, 0xaae, if n & 2 == 0 { 0 } else { 0xffff });
            game.g_ram[0xa1e] = if n & 4 == 0 { 4 } else { 8 };
            Operation::Cancel
        }
        196 => Operation::CopyCurrent,
        197 => Operation::CopyTarget,
        198 => Operation::Copy10,
        199 => Operation::Copy6,
        200..=215 => {
            put(game, 0x9a6, (id - 200) as u16);
            Operation::Beam
        }
        216..=223 => {
            let n = (id - 216) as usize;
            put(game, 0xacc, [0, 1, 6, 7, 8, 9, 10, 0xffff][n]);
            put(game, 0xa68, 2);
            put(game, 0xdf2, 2);
            put(game, 0xad0, 2);
            if n == 7 {
                put(game, 0xa4a, 0x8000);
            }
            Operation::Advance
        }
        _ => unreachable!(),
    }
}
pub(crate) fn capture_case(id: u32, run: fn(Operation, &mut Game) -> u16) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let mut game = Game::new();
    let operation = seed(id, &mut game);
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(operation, &mut game)));
    assert!(
        result.is_ok(),
        "visual.samus.defined.{id}: unexpected panic"
    );
    assert!(
        !game.g_fail,
        "visual.samus.defined.{id}: unexpected diagnostic"
    );
    let mut bytes = game.g_ram.to_vec();
    bytes.extend_from_slice(&result.unwrap().to_le_bytes());
    Some(bytes)
}
