//! Compile as a child of verification/player_projectiles.rs before rewriting production.
//! This is an input carrier, not expected behavior or a candidate implementation.
#![forbid(unsafe_code)]
use super::*;
use crate::game::projectile_initialization::Initialization;

pub(crate) const CASES: u32 = 102;

fn reaction_fixture(id: u32, g: &mut Game, loaded: &RomImage) -> (Vec<u8>, u8) {
    let (block, bts, header, outcome) = [
        (0x4000, 1, 0x8800, 0),
        (0x7000, 1, 0x8820, 0),
        (0xc000, 1, 0x8800, 1),
        (0xf000, 1, 0x8820, 1),
        (0x4000, 0x81, 0, 0),
        (0x7000, 0x81, 0, 0),
        (0xf000, 0x81, 0, 1),
        (0xc000, 0x81, 0x8840, 1),
        (0, 1, 0, 0),
        (0, 0x81, 0, 0),
    ][(id - 92) as usize];
    let bomb = id >= 100;
    // Only Loaded contains the authored definitions; Native remains independently zeroed.
    for (address, value) in [
        (0x949ea8, 0x8800u16),
        (0x94a014, 0x8820),
        (0x949fc8, 0x9000),
        (0x949002, 0x8840),
        (0x848800, 0xb3cf),
        (0x848802, 0x9100),
        (0x848820, 0xb3cf),
        (0x848822, 0x9120),
        (0x848840, 0xb3cf),
        (0x848842, 0x9140),
    ] {
        rom_word(loaded, address, value);
    }
    for (at, value) in [
        (0x79f, 1),
        (0x7a5, 8),
        (0x7a7, 8),
        (0x7a9, 2),
        (0x7ab, 2),
        (0x7b9, 128),
        (0xdc4, 0x7777),
        (0x1c27, 16),
        (if bomb { 0xb64 } else { 0xd08 }, 32),
        (if bomb { 0xb78 } else { 0xd0c }, if bomb { 32 } else { 16 }),
        (0xc18, if bomb { 0x500 } else { 0 }),
    ] {
        put(g, at, value);
    }
    let single_cell = [(10, block)];
    let cells: &[(usize, u16)] = if bomb {
        &[(18, 0x4000), (10, 0x7000), (19, 0xc000), (17, 0xf000)]
    } else {
        &single_cell
    };
    for &(cell, level) in cells {
        put(g, 0x10002 + 2 * cell, level);
        g.ram().write_byte(0x16402 + cell, bts);
    }
    for raw in (if bomb { 72 } else { 78 }..=78).step_by(2) {
        for base in [
            0x1c87, 0x1cd7, 0x1d27, 0x1d77, 0x1dc7, 0x1e17, 0xde1c, 0xde6c, 0xdebc, 0xdf0c,
        ] {
            put(g, base + raw, 0xaaaa);
        }
    }
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    expected[0xdc4..0xdc6].copy_from_slice(&(if bomb { 26u16 } else { 10 }).to_le_bytes());
    if bomb {
        expected[0xc18..0xc1a].copy_from_slice(&0x501u16.to_le_bytes());
    }
    let single_allocation = [(78, header, header + 0x900, 20)];
    // Independent literal allocations: center/north/east/west, then no-op south.
    let allocations: &[(usize, u16, u16, u16)] = match id {
        100 => &[
            (78, 0x8800, 0x9100, 36),
            (76, 0x8820, 0x9120, 20),
            (74, 0x8800, 0x9100, 38),
            (72, 0x8820, 0x9120, 34),
        ],
        101 => &[(78, 0x8840, 0x9140, 38)],
        _ if header != 0 => &single_allocation,
        _ => &[],
    };
    for &(raw, header, program, block) in allocations {
        expected[0x1c27..0x1c29].copy_from_slice(&(raw as u16).to_le_bytes());
        for (base, value) in [
            (0x1c37, header),
            (0x1c87, block),
            (0x1cd7, 0x853d),
            (0x1d27, program),
            (0x1d77, 0),
            (0x1dc7, 0),
            (0xde1c, 1),
            (0xde6c, 0x8da0),
            (0xdf0c, 0),
        ] {
            expected[base + raw..base + raw + 2].copy_from_slice(&value.to_le_bytes());
        }
    }
    (expected, outcome)
}

pub(crate) fn capture(id: u32, grapple: impl FnOnce(&mut Game) -> u8) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let mut g = Game::new();
    let loaded = RomImage::zeroed(0x200000);
    let native = RomImage::zeroed(0x200000);
    let seed = match id {
        0..=9 | 30..=35 | 44..=91 => 9,
        10..=15 => 77,
        16..=19 => 79,
        20..=27 => 7,
        28..=29 => 8,
        _ => 73,
    };
    let expected = if id >= 92 {
        Some(reaction_fixture(id, &mut g, &loaded))
    } else {
        fixture(seed, &mut g, &loaded, &native);
        None
    };
    let rw = |address: u32, value: u16| {
        rom_word(&loaded, address, value);
        rom_word(&native, address, value);
    };
    let mut limit = None;
    if (82..=91).contains(&id) {
        // Actual numeric collision callers; no initializer substitutes for a scan.
        g.ram().fill(0xb64..0xcd4, 0);
        for (at, value) in [
            (0x7a5, 8),
            (0x7a7, 8),
            (0x7a9, 2),
            (0x7ab, 2),
            (0x7b9, 128),
            (0xb64, 32),
            (0xb78, 32),
            (0xb8c, 0xf000),
            (0xba0, 0xf000),
            (0xbb4, 2),
            (0xbc8, 2),
            (0xbdc, 0x180),
            (0xbf0, 0x180),
            (0xc04, 2),
            (0xc18, 0x8000),
            (0xc2c, 20),
            (0xc40, 0x9400),
            (0xc54, 1),
            (0xc68, 0xb169),
            (0xcce, 1),
        ] {
            put(&mut g, at, value);
        }
        put(&mut g, crate::ram_map::words::projectile_index, 0);
        put(&mut g, crate::ram_map::words::cur_block_index, 0x7777);
        match id {
            82..=84 => {
                // Parent raw0 links child raw2; child actually moves into terrain.
                for (at, value) in [
                    (0xb8c, 0),
                    (0xba0, 0),
                    (0xc18, 0x8200),
                    (0xc7c, 0x102),
                    (0xb66, 32),
                    (0xb7a, 32),
                    (0xbb6, 2),
                    (0xbca, 2),
                    (0xc06, 2),
                    (0xc1a, 0x8200),
                    (0xc2e, 20),
                    (0xc42, 0x9400),
                    (0xc56, 1),
                    (0xc6a, 0xb075),
                    (0xcce, 2),
                ] {
                    put(&mut g, at, value);
                }
                if id == 83 {
                    put(&mut g, 0xb78, 60);
                    put(&mut g, 0xbf0, 0xf400);
                    put(&mut g, 0xbf2, 0xfa00);
                    put(&mut g, 0xc04, 0);
                    put(&mut g, 0xc06, 0);
                    put(&mut g, 0x10002 + 18 * 2, 0x8000);
                } else {
                    put(&mut g, 0xbdc, 0xc00);
                    put(&mut g, 0xbde, 0x600);
                    put(
                        &mut g,
                        0x10002 + 19 * 2,
                        if id == 84 { 0xc000 } else { 0x8000 },
                    );
                    if id == 84 {
                        // Parent scan succeeds; child fails in reaction table word.
                        limit = Some(lorom(0x949ea6) + 1);
                    }
                }
            }
            85 | 89 => {
                put(&mut g, 0xbc8, 9);
                for cell in [10, 18] {
                    put(
                        &mut g,
                        0x10002 + cell * 2,
                        if id == 85 { 0x8000 } else { 0x1000 },
                    );
                    g.ram().write_byte(0x16402 + cell, 4); // Fully occupied square slope.
                }
                if id == 85 {
                    put(&mut g, 0xc18, 0x8001);
                }
            }
            86 => {
                put(&mut g, 0xbb4, 9);
                put(&mut g, 0xc18, 0x8001);
                for cell in [17, 18] {
                    put(&mut g, 0x10002 + cell * 2, 0x1000);
                    g.ram().write_byte(0x16402 + cell, 4);
                }
            }
            87 => {
                put(&mut g, 0xc18, 0x8001);
                put(&mut g, 0xb78, 0x8000); // Opposite stored high byte rejects after X move.
            }
            88 => {
                put(&mut g, 0xc18, 0x8001);
                put(&mut g, 0xb78, 511); // Y moves to513 before its window rejection.
            }
            90 => {
                put(&mut g, 0xb64, 31);
                put(&mut g, 0xb78, 40);
                put(&mut g, 0xbb4, 9);
                put(&mut g, 0xc04, 4);
                put(&mut g, 0x10002 + 17 * 2, 0x1000);
                g.ram().write_byte(0x16402 + 17, 7); // Height8: first cell ends the two-column scan.
            }
            91 => {
                put(&mut g, 0xb78, 256);
                put(&mut g, 0xbc8, 129); // Span17 rejects after the fractional X move.
            }
            _ => unreachable!(),
        }
    }
    if (44..=81).contains(&id) {
        // Synthetic animation/trail definitions, deliberately distinct by frame,
        // style, direction and side. These are inputs, never expected algorithms.
        g.ram().fill(0xd658..0xd7c0, 0);
        put(&mut g, crate::ram_map::words::projectile_index, 0);
        put(&mut g, 0xb64, 200); // Both signed-offset trail halves remain visible.
        rw(0x9393f8, 3);
        rw(0x9393fe, 1);
        rw(0x939400, 7);
        rw(0x939402, 0xa010);
        rw(0x939404, 0x0905);
        rw(0x939406, 2);
        for style in 0..64 {
            rw(0x90b5bb + style * 2, 0x8100 + style as u16 * 8);
            rw(0x90b609 + style * 2, 0x8400 + style as u16 * 8);
        }
        // The style tables overlap historically. Restore the used style0 rows.
        rw(0x90b5bb, 0x8100);
        rw(0x90b609, 0x8400);
        for cursor in [0x8100, 0x8400, 0x8200, 0x8500, 0x8208, 0x8508] {
            rw(0x900000 + cursor, 2);
            rw(0x900002 + cursor, cursor as u16 ^ 0x7777);
        }
        for (table, first, second, bias) in [
            (0x9ba4b3, 0xa800u16, 0xb800u16, 0u8),
            (0x9ba4cb, 0xac00, 0xc800, 16),
            (0x9ba4e3, 0xb000, 0xd800, 32),
        ] {
            for kind in 0..12u16 {
                let row = first + kind * 32;
                rw(table + u32::from(kind) * 2, row);
                for direction in 0..10u16 {
                    let offsets = second + kind * 128 + direction * 12;
                    rw(0x9b0000 + u32::from(row + direction * 2), offsets);
                    for marker in 0..3u16 {
                        let at = 0x9b0000 + u32::from(offsets + marker * 4);
                        let delta = bias + kind as u8 + direction as u8 + marker as u8;
                        rw(at, u16::from_le_bytes([0x80 + delta, 9 + delta]));
                        rw(at + 2, u16::from_le_bytes([7 + delta, 0xfd - delta]));
                    }
                }
            }
        }
        match id {
            44 => {
                rw(0x939400, 0x8239);
                rw(0x939402, 0x9420);
                rw(0x939420, 7);
                rw(0x939422, 0xa000);
                rw(0x939424, 0x0905);
            }
            45 | 79 => rw(0x939400, 0),
            46..=48 => limit = Some(lorom(0x939400) + [2, 4, 5][(id - 46) as usize]),
            49 => {
                put(&mut g, crate::ram_map::words::projectile_index, 0xffff);
                put(&mut g, 0xc40 + 0xfffe, 0x9400);
                put(&mut g, 0xc54 + 0xfffe, 1);
            }
            50 => put(&mut g, crate::ram_map::words::projectile_index, 1),
            51 => {}
            52 => put(&mut g, 0xc54, 2),
            53 => rw(0x939400, 0x8239),
            54 => put(&mut g, 0xc18, 0x300),
            55 => {
                for pair in 0..18 {
                    put(&mut g, 0xd658 + pair * 2, 2);
                }
            }
            56 => put(&mut g, 0xd67c + 34, 7),
            57 => put(&mut g, 0xd658 + 34, 2),
            58 => {
                put(&mut g, crate::ram_map::words::projectile_index, 2);
                for (at, value) in [
                    (0xb66, 200),
                    (0xb7a, 150),
                    (0xc1a, 0x11),
                    (0xc06, 3),
                    (0xc42, 0x9400),
                    (0xc56, 1),
                ] {
                    put(&mut g, at, value);
                }
            }
            59 => {
                for (at, value) in [
                    (0x93f, 0x8000),
                    (0x78, 0x100),
                    (0x7a, 0x80),
                    (0x7c, 0x40),
                    (0x80, 64),
                    (0x82, 64),
                    (0x911, 10),
                    (0x915, 20),
                ] {
                    put(&mut g, at, value);
                }
            }
            60 | 61 => {
                put(&mut g, 0xc90, 1);
                put(&mut g, crate::ram_map::words::room_width_in_blocks, 32);
                put(&mut g, crate::ram_map::words::room_height_in_blocks, 32);
            }
            62 => limit = Some(lorom(0x9bb820) + 2),
            63 => limit = Some(lorom(0x90b609)),
            64 => limit = Some(lorom(0x939400) + 1),
            65..=73 => {
                let right = id >= 68 && id <= 70;
                let offset = if right { 36 } else { 0 };
                put(&mut g, 0xd658 + offset, 1);
                put(&mut g, 0xd6a0 + offset, 0x8100);
                put(&mut g, 0xd6e8 + offset, 0x4567);
                put(&mut g, 0xd730, 90);
                put(&mut g, 0xd754, 91);
                put(&mut g, 0xd778, 80);
                put(&mut g, 0xd79c, 81);
                if id <= 70 {
                    rw(0x908100, [0xb525, 0xb587, 0xb5b3][((id - 65) % 3) as usize]);
                    rw(0x908102, 2);
                    rw(0x908104, 0x2777);
                } else if id == 71 {
                    rw(0x908100, 0);
                    limit = Some(lorom(0x908100) + 2);
                } else if id == 72 {
                    put(&mut g, 0xd6a0, 0xfffe);
                    rw(0x90fffe, 0xb587);
                    rw(0x900000, 2);
                    rw(0x900002, 0x2777);
                } else {
                    limit = Some(lorom(0x908100) + 2);
                }
            }
            74 => {
                put(&mut g, 0xb64, 0xfffe);
                put(&mut g, 0xb78, 1);
            }
            75..=78 => put(
                &mut g,
                0xc18,
                [0x100, 0x200, 0x20, 0x10][(id - 75) as usize],
            ),
            80 | 81 => {
                put(&mut g, crate::ram_map::words::projectile_index, 20);
                put(&mut g, 0xc40 + 20, 0x9400);
                put(&mut g, 0xc54 + 20, 1);
                if id == 81 {
                    limit = Some(lorom(0x939400) + 5);
                }
            }
            _ => unreachable!(),
        }
    }
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
        44..=101 => {}
        _ => unreachable!(),
    }
    g.loaded_rom = Some(if let Some(length) = limit {
        let short = RomImage::zeroed(length);
        for at in 0..length {
            short.write(at, loaded.read(at));
        }
        short
    } else {
        loaded
    });
    // Candidate installation changes only this binding to register_native_rom.
    g.register_native_rom(native.clone());
    let outcome = expected.as_ref().map_or(0, |(_, outcome)| *outcome);
    let result = catch_unwind(AssertUnwindSafe(|| match id {
        92..=99 => assert_eq!(grapple(&mut g), outcome, "grapple.bridge.result.{id}"),
        100..=101 => call_motion(0x90b099, 0, &mut g),
        82..=91 => call_motion(
            match id {
                82 | 84 => 0x90b06a,
                83 => 0x90b047,
                85 | 87 => 0x90b151,
                86 | 88 => 0x90b13b,
                89 | 91 => 0x90af60,
                90 => 0x90af4a,
                _ => unreachable!(),
            },
            0,
            &mut g,
        ),
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
        44..=50 | 80 | 81 => animation::advance(&mut Runtime::new(&mut g)),
        60 | 61 => {
            call_motion(if id == 60 { 0x90aef3 } else { 0x90dc9c }, 0, &mut g);
            update_trails(&mut g);
        }
        65..=73 => update_trails(&mut g),
        51..=64 | 74..=79 => {
            presentation::emit(&mut Runtime::new(&mut g), 0);
            if id == 51 || id == 59 {
                update_trails(&mut g);
            }
            if id == 51 {
                update_trails(&mut g);
            } // Observe a later lifetime/draw step.
        }
        _ => unreachable!(),
    }));
    if let Some((expected, _)) = expected {
        assert!(result.is_ok() && !g.g_fail, "reaction.diagnostic.{id}");
        assert_eq!(
            &(0..g.ram().len())
                .map(|at| g.ram().read_byte(at))
                .collect::<Vec<u8>>()[..],
            &expected,
            "reaction.full_ram.{id}"
        );
    }
    // Prefix records failure category; RAM preserves all defined partial writes.
    let mut bytes = vec![u8::from(result.is_err()), u8::from(g.g_fail)];
    bytes.extend((0..g.ram().len()).map(|at| g.ram().read_byte(at)));
    Some(bytes)
}
