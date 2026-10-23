//! Compile as a child of verification/player_projectiles.rs before rewriting production.
//! This is an input carrier, not expected behavior or a candidate implementation.
#![forbid(unsafe_code)]
use super::*;
use crate::game::projectile_initialization::Initialization;

pub(crate) const CASES: u32 = 44;

pub(crate) fn capture(id: u32) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let mut g = Game::new();
    let loaded = RomImage::zeroed(0x200000);
    let native = RomImage::zeroed(0x200000);
    let seed = match id {
        0..=9 | 30..=35 => 9,
        10..=15 => 77,
        16..=19 => 79,
        20..=27 => 7,
        28..=29 => 8,
        _ => 73,
    };
    fixture(seed, &mut g, &loaded, &native);
    let rw = |address: u32, value: u16| {
        for image in [&loaded, &native] {
            for (offset, byte) in value.to_le_bytes().into_iter().enumerate() {
                image.write(lorom(address) + offset, byte);
            }
        }
    };
    match id {
        0..=4 => {
            // Echo deliberately uses a negative launch-table selector.
            rw(0x938403 - 68, 0x9000);
        }
        5 | 6 => {
            for at in (0xb64..0xccc).step_by(2) {
                put(&mut g, at, 0x3456);
            }
            put(&mut g, 0xcce, 4);
            put(&mut g, 0xcd2, 3);
        }
        7 => {
            rw(0x939400, 0x822f);
        }
        8 => {
            rw(0x939400, 0x8239);
            rw(0x939402, 0);
        }
        9 => {
            put(&mut g, 0xc54, 0);
        }
        10 => put(&mut g, 0x9c6, 0),
        11 => {
            put(&mut g, 0xcce, 5);
        }
        12 => {
            for at in (0xc18..0xc22).step_by(2) {
                put(&mut g, at, 0x8000);
            }
        }
        13 => {
            put(&mut g, 0x9c6, 0x8000);
        }
        14 => {
            put(&mut g, 0xa1c, 0xffff);
        }
        15 => {
            put(&mut g, 0xa1c, 0xffff);
            put(&mut g, 0xcce, 5);
        }
        16 => {
            put(&mut g, 0xcd2, 5);
        }
        17 => {
            put(&mut g, 0x9d2, 3);
            put(&mut g, 0x9ce, 0);
        }
        18 => {
            put(&mut g, 0x9d2, 3);
            put(&mut g, 0x9ce, 0x8000);
        }
        19 => {
            put(&mut g, 0x9a2, 0);
        }
        20 => put(&mut g, 0xcd0, 1),
        21 => put(&mut g, 0xcd0, 14),
        22 => {
            put(&mut g, 0xcd0, 30);
            native.write(lorom(0x909001), 255);
            native.write(lorom(0x909000), 3);
        }
        23 => {
            put(&mut g, 0xcd0, 30);
            native.write(lorom(0x909001), 254);
            native.write(lorom(0x909002), 1);
            native.write(lorom(0x909000), 3);
        }
        24..=27 => {
            put(&mut g, crate::ram_map::words::hyper_beam_flag, 1);
            for part in 0..3 {
                put(&mut g, 0xcd6 + part * 2, 1);
                put(
                    &mut g,
                    0xcdc + part * 2,
                    [0, 1, 2, 0x8000][(id - 24) as usize],
                );
            }
        }
        28 => put(&mut g, 0xcd0, 1),
        29 => {
            native.write(lorom(0x90c488), 254);
            native.write(lorom(0x90c489), 1);
            native.write(lorom(0x90c487), 5);
        }
        30 => put(&mut g, crate::ram_map::words::time_is_frozen_flag, 1),
        31 => {
            put(&mut g, 0xd658, 0);
            put(&mut g, 0xd67c, 0);
        }
        32 => {
            put(&mut g, 0xd778, 0xff00);
        }
        33 => {
            put(&mut g, 0xd730, 0xff00);
        }
        34 => {
            put(&mut g, crate::ram_map::words::oam_next_ptr, 512);
        }
        35 => {
            put(&mut g, 0xd658, 2);
            put(&mut g, 0xd67c, 1);
        }
        36 => put(&mut g, crate::ram_map::words::time_is_frozen_flag, 1),
        37..=39 => {
            put(&mut g, 0xdfa, 0x1234);
            put(&mut g, 0xa10, 0xabcd);
            put(&mut g, 0xdae, 0x3456);
            put(&mut g, 0xa9a, 9);
        }
        40 => put(&mut g, 0x9d2, 6),
        41 => {
            put(&mut g, 0x9d2, 5);
            put(&mut g, 0x9a2, 0x8000);
        }
        42 => {
            put(&mut g, 0x9d2, 2);
            put(&mut g, 0x9ca, 0);
        }
        43 => {
            put(&mut g, 0x9d2, 0xffff);
        }
        _ => unreachable!(),
    }
    g.loaded_rom = Some(loaded);
    // Candidate installation changes only this binding to register_native_rom.
    g.register_native_rom(native.clone());
    let result = catch_unwind(AssertUnwindSafe(|| match id {
        0..=4 => initialize(
            0,
            [
                Initialization::Projectile,
                Initialization::SuperMissile,
                Initialization::Bomb,
                Initialization::SpecialAttack,
                Initialization::Echo,
            ][id as usize],
            &mut g,
        ),
        5 => reset(&mut g),
        6 => clear(0, &mut g),
        7..=9 => update(&mut g),
        10..=19 | 36 | 40..=43 => prepare(&mut g),
        20..=27 => flare(&mut g),
        28..=29 => grapple_flare(&mut g),
        30..=35 => update_trails(&mut g),
        37 => prepare_entry(0x90e713, &mut g),
        38 => prepare_entry(0x90e8cd, &mut g),
        39 => prepare_entry(0x90e6c9, &mut g),
        _ => unreachable!(),
    }));
    // Prefix records failure category; RAM preserves all defined partial writes.
    let mut bytes = vec![u8::from(result.is_err()), u8::from(g.g_fail)];
    bytes.extend_from_slice(&g.g_ram);
    Some(bytes)
}
