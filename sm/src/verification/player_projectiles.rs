//! Synthetic integration checks against frozen old-runtime observations.
//! Expected deltas were captured from libsm-runtime.rlib, independently executing
//! the old bank entries; no candidate algorithm produced the expected results.
#![forbid(unsafe_code)]
use super::*;
use crate::{game_state::Game, rom::RomImage};
use std::panic::{AssertUnwindSafe, catch_unwind};
fn lorom(a: u32) -> usize {
    crate::rtl::lorom_offset(a) as usize
}
fn put(g: &mut Game, a: usize, v: u16) {
    g.g_ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
}
fn fixture(id: u32, g: &mut Game, loaded: &RomImage, native: &RomImage) {
    let rw = |address: u32, value: u16| {
        let a = lorom(address);
        for (i, b) in value.to_le_bytes().into_iter().enumerate() {
            loaded.write(a + i, b);
            native.write(a + i, b);
        }
    };
    match id {
        0..=2 => {
            for a in (0xb64..0xccc).step_by(2) {
                put(g, a, 0x1234);
            }
            put(g, 0xcce, 4);
            put(g, 0xcd2, 3);
        }
        3 | 4 => {
            put(g, 0xaf6, 100);
            put(g, 0xafa, 100);
            if id == 4 {
                put(g, 0x911, 500);
            }
        }
        5 => {
            put(g, 0xaf6, 100);
            put(g, 0xafa, 100);
            put(g, 0xb64, 104);
            put(g, 0xb78, 96);
        }
        6 => {
            put(g, 0xaf6, 100);
            put(g, 0xafa, 100);
            put(g, 0xb6a, 96);
            put(g, 0xb7e, 96);
            put(g, 0xc82, 1);
            put(g, 0xcce, 1);
        }
        7 => {
            put(g, 0xcd0, 15);
            for i in 0..3 {
                rw(0x90c481 + 2 * i, 0x9000);
            }
            native.write(lorom(0x909001), 7);
            loaded.write(lorom(0x909001), 99);
            native.write(lorom(0x91b62c), 255);
        }
        8 => {
            put(g, 0xcd0, 2);
            put(g, 0xd1c, 256);
            native.write(lorom(0x90c488), 7);
            loaded.write(lorom(0x90c488), 99);
        }
        9..=72 => {
            let family = ((id - 9) / 8) as usize;
            let kind = [0u16, 1, 0x100, 0x200, 0x300, 0x500, 0x700, 0x800][family];
            for base in [0x9383c1, 0x9383d9, 0x9383f1, 0x938413, 0x93842b] {
                for n in 0..16 {
                    rw(base + 2 * n, 0x9000);
                }
            }
            rw(0x939000, 20);
            for n in 0..10 {
                rw(0x939002 + 2 * n, 0x9400);
            }
            for (a, v) in [
                (0x939400, 1),
                (0x939402, 0xa000),
                (0x939404, 0x0303),
                (0x939406, 0),
                (0x939408, 0x8239),
                (0x93940a, 0x9400),
                (0x93a000, 1),
                (0x93a002, 0),
                (0x93a004, 0x7700),
                (0x93a006, 0x22),
            ] {
                rw(a, v);
            }
            for a in [0x93867b, 0x93867f, 0x938693, 0x938683] {
                rw(a, 0x9400);
            }
            for (a, v) in [
                (0xb64, 100),
                (0xb78, 100),
                (0xbb4, 3),
                (0xbc8, 3),
                (0xc04, 2),
                (0xc18, kind),
                (0xc2c, 20),
                (0xc40, 0x9400),
                (0xc54, 1),
                (0xc68, 0xb169),
                (0xc7c, 240),
                (0xc90, 4),
                (0xcb8, 0xa000),
                (0xcce, 1),
                (0xcd2, 1),
                (0xccc, 30),
            ] {
                put(g, a, v);
            }
            for side in 0..2 {
                let script = 0x8100 + side * 0x20;
                rw(0x900000 + script, 2);
                rw(0x900002 + script, 0x2777 + side as u16);
                put(g, 0xd658 + side as usize * 36, 1);
                put(g, 0xd6a0 + side as usize * 36, script as u16);
                put(g, 0xd730 + side as usize * 36, 90);
                put(g, 0xd778 + side as usize * 36, 80);
            }
        }
        73..=88 => {
            fixture(9, g, loaded, native);
            g.g_ram[0xb64..0xce2].fill(0);
            g.g_ram[0xd658..0xd7c0].fill(0);
            for (a, v) in [
                (0x7a5, 16),
                (0x7a9, 8),
                (0x7ab, 8),
                (0x7b9, 1024),
                (0xa42, 0xe695),
                (0xa60, 0xe90e),
                (0xa1c, 1),
                (0xaf6, 47),
                (0xafa, 63),
                (0xb00, 16),
                (0x9b2, 0x40),
                (0x9b6, 0x80),
                (0x9ba, 0x20),
                (0x9b8, 0x10),
                (0xd32, 0xc4f0),
                (0x998, 8),
                (0x9c2, 99),
                (0x8b, 0x40),
                (0x8f, 0x40),
                (0x9c6, 10),
                (0x9ca, 10),
                (0x9ce, 10),
                (0x195e, 0xffff),
                (0x1962, 0xffff),
            ] {
                put(g, a, v);
            }
            for image in [loaded, native] {
                image.write(lorom(0x91b629) + 8 + 3, 2);
                image.write(lorom(0x91b629) + 8 + 6, 16);
            }
            let mode = (id - 73) % 8;
            match mode {
                0 => {}
                1 => put(g, 0x9a6, 1),
                2 => put(g, 0x9a6, 2),
                3 => {
                    put(g, 0x9a6, 0x1004);
                    put(g, 0xcd0, 120);
                    put(g, 0x8b, 0);
                }
                4 => put(g, 0x9d2, 1),
                5 => put(g, 0x9d2, 2),
                6 | 7 => {
                    g.g_ram[0xa1f] = 4;
                    put(g, 0x9a2, 0x1000);
                    if mode == 7 {
                        put(g, 0x9d2, 3);
                    }
                }
                _ => unreachable!(),
            }
        }
        89..=98 => {
            if id <= 96 {
                fixture(73, g, loaded, native);
                put(g, 0x9a6, 0x1000 | [1, 2, 4, 8][((id - 89) % 4) as usize]);
                put(g, 0x9d2, 3);
                put(g, 0xcd0, 120);
            } else {
                fixture(79, g, loaded, native);
                put(g, 0xcd0, 120);
            }
        }
        _ => unreachable!(),
    }
}
const OLD_DELTAS: &[&[(usize, u8)]] = &[
    &[
        (0xb64, 0),
        (0xb65, 0),
        (0xb78, 0),
        (0xb79, 0),
        (0xb8c, 0),
        (0xb8d, 0),
        (0xba0, 0),
        (0xba1, 0),
        (0xbb4, 0),
        (0xbb5, 0),
        (0xbc8, 0),
        (0xbc9, 0),
        (0xbdc, 0),
        (0xbdd, 0),
        (0xbf0, 0),
        (0xbf1, 0),
        (0xc04, 0),
        (0xc05, 0),
        (0xc18, 0),
        (0xc19, 0),
        (0xc2c, 0),
        (0xc2d, 0),
        (0xc40, 0),
        (0xc41, 0),
        (0xc54, 0),
        (0xc55, 0),
        (0xc68, 105),
        (0xc69, 177),
        (0xc7c, 0),
        (0xc7d, 0),
        (0xcb8, 0),
        (0xcb9, 0),
        (0xcce, 3),
    ],
    &[
        (0xb76, 0),
        (0xb77, 0),
        (0xb8a, 0),
        (0xb8b, 0),
        (0xb9e, 0),
        (0xb9f, 0),
        (0xbb2, 0),
        (0xbb3, 0),
        (0xbc6, 0),
        (0xbc7, 0),
        (0xbda, 0),
        (0xbdb, 0),
        (0xbee, 0),
        (0xbef, 0),
        (0xc02, 0),
        (0xc03, 0),
        (0xc16, 0),
        (0xc17, 0),
        (0xc2a, 0),
        (0xc2b, 0),
        (0xc3e, 0),
        (0xc3f, 0),
        (0xc52, 0),
        (0xc53, 0),
        (0xc66, 0),
        (0xc67, 0),
        (0xc7a, 105),
        (0xc7b, 177),
        (0xc8e, 0),
        (0xc8f, 0),
        (0xcca, 0),
        (0xccb, 0),
        (0xcd2, 2),
    ],
    &[(0xcce, 3), (0x10c66, 105), (0x10c67, 177)],
    &[
        (0xaae, 100),
        (0xab6, 92),
        (0xb64, 100),
        (0xb78, 92),
        (0xbdc, 8),
    ],
    &[(0xc68, 105), (0xc69, 177)],
    &[
        (0xa68, 180),
        (0xacc, 1),
        (0xb64, 228),
        (0xb78, 228),
        (0xb79, 255),
        (0xc7c, 1),
    ],
    &[
        (0xa58, 75),
        (0xa59, 240),
        (0xb6a, 0),
        (0xb7e, 0),
        (0xc6e, 105),
        (0xc6f, 177),
        (0xc82, 0),
        (0xcce, 0),
    ],
    &[(0xcd6, 1), (0xcdc, 7)],
    &[(0xcd6, 1), (0xcdc, 7)],
];
fn native_entries() {
    for id in 0..9 {
        let mut g = Game::new();
        let loaded = RomImage::zeroed(0x200000);
        let native = RomImage::zeroed(0x200000);
        fixture(id, &mut g, &loaded, &native);
        let mut expected = g.g_ram.to_vec();
        for &(a, b) in OLD_DELTAS[id as usize] {
            expected[a] = b;
        }
        let actual = capture_case(id).unwrap();
        let differences: Vec<_> = actual
            .iter()
            .zip(&expected)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .take(12)
            .collect();
        assert!(
            differences.is_empty(),
            "old native case {id}: {differences:?}"
        );
    }
}
// Independently written physical layout, intentionally not encoded::owned_words.
fn physical_words() -> Vec<usize> {
    let mut a: Vec<_> = (0xb64..0xccc)
        .step_by(2)
        .chain((0xd658..0xd7c0).step_by(2))
        .chain((0xdaa..0xdba).step_by(2))
        .collect();
    a.extend([
        0xcce, 0xcd2, 0xdde, 0xccc, 0xcd4, 0xb60, 0xcd0, 0xdc2, 0xb18, 0xb62, 0xdc0, 0xb40, 0xb5e,
        0x18ac, 0x9d2, 0xa04, 0xaaa, 0xcd6, 0xcd8, 0xcda, 0xcdc, 0xcde, 0xce0,
    ]);
    assert_eq!(a.len(), 391);
    let mut sorted = a.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 391);
    a
}
// Read every owned value directly, so an accidentally external lane cannot pass.
fn typed_image(state: &crate::game::session::Session, bytes: &[u8]) {
    let expect = |at: usize, v: u16| {
        assert_eq!(
            v,
            u16::from_le_bytes([bytes[at], bytes[at + 1]]),
            "typed word {at:x}"
        )
    };
    for (i, s) in state.projectiles.shots.iter().enumerate() {
        for (column, value) in [
            s.x,
            s.y,
            s.x_fraction,
            s.y_fraction,
            s.x_radius,
            s.y_radius,
            s.x_speed,
            s.y_speed,
            s.direction,
            s.kind,
            s.damage,
            s.cursor,
            s.remaining,
            s.motion,
            s.variable,
            s.trail_time,
            s.auxiliary,
            s.sprite,
        ]
        .into_iter()
        .enumerate()
        {
            expect(0xb64 + 20 * column + 2 * i, value);
        }
    }
    for (i, p) in state.projectiles.trails.iter().enumerate() {
        for (side, s) in [&p.left, &p.right].into_iter().enumerate() {
            for (column, value) in [s.remaining, s.cursor, s.tile, s.x, s.y]
                .into_iter()
                .enumerate()
            {
                expect(0xd658 + 36 * (2 * column + side) + 2 * i, value);
            }
        }
    }
    for (i, &v) in state.projectiles.launch.displacements.iter().enumerate() {
        expect(0xdaa + 4 * i, ((v as u32) >> 16) as u16);
        expect(0xdac + 4 * i, v as u16);
    }
    let c = &state.projectiles.controls;
    for (address, value) in [
        (0xcce, state.projectiles.count),
        (0xcd2, state.projectiles.bomb_count),
        (0xdde, state.projectiles.selected),
        (0xccc, c.cooldown),
        (0xcd4, c.spread_charge),
        (0xb60, c.orbit_step),
        (0xcd0, c.charge),
        (0xdc2, c.previous_charge),
        (0xb18, c.glow),
        (0xb62, c.charge_palette),
        (0xdc0, c.resume_sound),
        (0xb40, c.echo_sound),
        (0xb5e, c.pending_aim),
        (0x18ac, c.protection),
        (0x9d2, c.selected_weapon),
        (0xa04, c.automatic_weapon),
        (0xaaa, c.weapon_changed),
        (0xcd6, c.flare_frames[0]),
        (0xcd8, c.flare_frames[1]),
        (0xcda, c.flare_frames[2]),
        (0xcdc, c.flare_timers[0]),
        (0xcde, c.flare_timers[1]),
        (0xce0, c.flare_timers[2]),
    ] {
        expect(address, value);
    }
}

fn lane_projection() {
    use super::access::Store;
    let mut g = Game::new();
    for (a, b) in g.g_ram.iter_mut().enumerate() {
        *b = (a.wrapping_mul(37) ^ a / 17) as u8;
    }
    let mut expected = g.g_ram.to_vec();
    let addresses = physical_words();
    let mut owner = Runtime::new(&mut g);
    typed_image(owner.session(), &expected);
    for &a in &addresses {
        for lane in 0..2 {
            assert_eq!(owner.byte(a + lane), expected[a + lane]);
            let value = expected[a + lane] ^ 0xa5;
            owner.foreign(|g| g.g_ram[a + lane] = value);
            expected[a + lane] = value;
        }
    }
    typed_image(owner.session(), &expected);
    for &a in &addresses {
        assert_eq!(
            u16::from_le_bytes([owner.byte(a), owner.byte(a + 1)]),
            u16::from_le_bytes([expected[a], expected[a + 1]])
        );
    }
    for a in [0xda9, 0xd7bf] {
        owner.store(a, 0x5ac3, Store::SliceLe);
        expected[a..a + 2].copy_from_slice(&0x5ac3u16.to_le_bytes());
    }
    owner.store(0xda8, 0x9182, Store::PrefixLe);
    expected[0xda8..0xdaa].copy_from_slice(&0x9182u16.to_le_bytes());
    for (slot, address) in [(0x1f6, 0xdaa), (0x22a, 0xdde), (0xcaa4, 0xd658)] {
        shot!(owner, slot, x_radius = 0x1234);
        assert_eq!(
            u16::from_le_bytes([owner.byte(address), owner.byte(address + 1)]),
            0x1234
        );
        expected[address..address + 2].copy_from_slice(&0x1234u16.to_le_bytes());
    }
    assert_eq!(
        (owner.session().projectiles.launch.displacements[0] as u32) >> 16,
        0x1234
    );
    assert_eq!(owner.session().projectiles.selected, 0x1234);
    assert_eq!(owner.session().projectiles.trails[0].left.remaining, 0x1234);
    // The old negative-array verification case addresses the same physical word.
    owner.store(0xb62, 0x3456, Store::PrefixLe);
    assert_eq!(owner.session().projectiles.controls.charge_palette, 0x3456);
    expected[0xb62..0xb64].copy_from_slice(&0x3456u16.to_le_bytes());
    drop(owner);
    assert_eq!(g.g_ram.as_slice(), expected);
}
fn publication() {
    use super::access::Store;
    let mut g = Game::new();
    let mut owner = Runtime::new(&mut g);
    assert!(owner.loaded_image().is_none());
    let order = std::cell::Cell::new(0);
    shot!(
        owner,
        {
            assert_eq!(order.get(), 1);
            order.set(2);
            0
        },
        x = {
            assert_eq!(order.get(), 0);
            order.set(1);
            0x2468
        }
    );
    assert_eq!(order.get(), 2);
    assert_eq!(owner.session().projectiles.shots[0].x, 0x2468);
    drop(owner);
    let end = g.g_ram.len() - 1;
    let failed = catch_unwind(AssertUnwindSafe(|| {
        let mut owner = Runtime::new(&mut g);
        owner.session_mut().projectiles.shots[0].x = 0x1234;
        owner.store(end, 0xaabb, Store::PrefixLe);
    }));
    assert!(failed.is_err());
    assert_eq!(&g.g_ram[0xb64..0xb66], &[0x34, 0x12]);
    assert_eq!(g.g_ram[end], 0xbb);
    let failed = catch_unwind(AssertUnwindSafe(|| {
        Runtime::new(&mut g).store(end, 0xccdd, Store::SliceLe);
    }));
    assert!(failed.is_err());
    assert_eq!(g.g_ram[end], 0xbb);
    put(&mut g, 0x1c37, 0x4321);
    let mut owner = Runtime::new(&mut g);
    owner.session_mut().projectiles.selected = 19;
    let result = owner.foreign(|g| {
        assert_eq!(&g.g_ram[0xdde..0xde0], &[19, 0]);
        crate::game::room_mechanisms::clear(g);
        assert_eq!(&g.g_ram[0x1c37..0x1c39], &[0, 0]);
        assert_eq!(&g.g_ram[0xdde..0xde0], &[19, 0]);
        put(g, 0xdde, 31);
        put(g, 0xb64, 47);
        17
    });
    assert_eq!(result, 17);
    assert_eq!(owner.session().projectiles.selected, 31);
    assert_eq!(owner.session().projectiles.shots[0].x, 47);
    drop(owner);
    assert_eq!(&g.g_ram[0x1c37..0x1c39], &[0, 0]);
    let failed = catch_unwind(AssertUnwindSafe(|| {
        Runtime::new(&mut g).foreign(|g| {
            put(g, 0xdde, 55);
            panic!("callback unwind")
        });
    }));
    assert!(failed.is_err());
    assert_eq!(&g.g_ram[0xdde..0xde0], &[55, 0]);
    let failed = catch_unwind(AssertUnwindSafe(|| {
        let mut owner = Runtime::new(&mut g);
        owner.session_mut().projectiles.selected = 73;
        owner.session_mut().inventory.ammunition[0].current = 11;
        panic!("core unwind");
    }));
    assert!(failed.is_err());
    assert_eq!(&g.g_ram[0xdde..0xde0], &[73, 0]);
    assert_eq!(&g.g_ram[0x9c6..0x9c8], &[11, 0]);
}

pub(crate) fn run() -> u32 {
    lane_projection();
    publication();
    native_entries();
    matrix_entries();
    106
}

// Independently captured old-runtime FNV-1a digests of all 131072 RAM bytes.
// Raw synthetic observations and SHA-256 provenance accompany the validation.
const OLD_MATRIX: [u64; 95] = [
    0x17cbe2c4d3c4f5c6, // 9
    0x25a819d46086040d, // 10
    0xda1ae6d210b40e95, // 11
    0x9ac81c2dc4a539ae, // 12
    0x5384205c18521799, // 13
    0x33618154b7990915, // 14
    0xae801238ad642af9, // 15
    0x57de32db2b7bbfb2, // 16
    0x021952c358183173, // 17
    0x3dd54b549b366f48, // 18
    0x6a54b66aee9623a0, // 19
    0x5b09498b73b6c56b, // 20
    0x8dff558200dbfb94, // 21
    0xdb955055cde60842, // 22
    0xae801238ad642af9, // 23
    0xbeffa0c46cbf178f, // 24
    0x137dfb3cfc6968fd, // 25
    0x1dd07cf510204862, // 26
    0x202e9481abf74fd7, // 27
    0x7b46ce341e787c42, // 28
    0x7b46ce341e787c42, // 29
    0x3eebcde88e56c2e2, // 30
    0xae801238ad642af9, // 31
    0xd7de0dcf6b529509, // 32
    0xcd86a36620710e18, // 33
    0x53c9bb18a94d110f, // 34
    0x3ba67021ac0f4216, // 35
    0xc4292959e4eee208, // 36
    0xc4292959e4eee208, // 37
    0xfea0406681a5e52f, // 38
    0xae801238ad642af9, // 39
    0xabccf5de03d03988, // 40
    0xf097dda10fb96cdf, // 41
    0xef34f94ed8e696d4, // 42
    0x4c9cc41d1a5d94c5, // 43
    0xc4292959e4eee208, // 44
    0x92c9ae5cf12b0974, // 45
    0x589a5a96e9331be0, // 46
    0xae801238ad642af9, // 47
    0x4369d6305885639f, // 48
    0x190ff46207bde9c1, // 49
    0xdb7a8bd53806d676, // 50
    0x2ad86d20c414e9d7, // 51
    0x7b46ce341e787c42, // 52
    0x92c9ae5cf12b0974, // 53
    0xa658de5dc28ff91e, // 54
    0xae801238ad642af9, // 55
    0x5881cfbf8b3474f5, // 56
    0x6f4e69cb50605f63, // 57
    0x9701281990850178, // 58
    0xbf678327915c9171, // 59
    0xc4292959e4eee208, // 60
    0x92c9ae5cf12b0974, // 61
    0x713725e66ff5f7fc, // 62
    0xae801238ad642af9, // 63
    0x4c4814bc21dd513b, // 64
    0xe9086618c13e545e, // 65
    0xe2582622fd837855, // 66
    0x81075a31aed24b38, // 67
    0x7b46ce341e787c42, // 68
    0x92c9ae5cf12b0974, // 69
    0x27ecbd3c36cbb98d, // 70
    0xae801238ad642af9, // 71
    0xefe40c8aee41e90a, // 72
    0x38f2a988cdd7c320, // 73
    0x1d12e6aea90899cf, // 74
    0x372b77cb400905eb, // 75
    0x6be88c101ffcd581, // 76
    0x6c606bf6224a3a77, // 77
    0x176ea5e524b927bb, // 78
    0x57a9086c0960d8e7, // 79
    0x8ea1bab60daac6c4, // 80
    0x4c223d6955fdcfac, // 81
    0xf21e2d88fe442dba, // 82
    0x705b7347f3549a9f, // 83
    0x8f5e227c79e447e2, // 84
    0x2964557872a638a3, // 85
    0xfc13bffae290d2f1, // 86
    0xcb871cb6004e575b, // 87
    0x7c6863285ed69791, // 88
    0x4f57ac894c5691e6, // 89
    0xee0e86a02e6c883e, // 90
    0xf38a9d35120bc5fe, // 91
    0xf6073cf421887140, // 92
    0x2873f3776577a013, // 93
    0x96bd6b3d17f64187, // 94
    0xdb777fd3c1d413dd, // 95
    0x866e977721924d37, // 96
    0x7277f208bf284b94, // 97
    0x71094db59cd1af95, // 98
    0x083dddefe38b766c, // 99
    0x55d31a36e3024753, // 100
    0xb5d3c64f3d5a66e2, // 101
    0x97e262c796faa2aa, // 102
    0xf19e469e8b9410c3, // 103
];
fn matrix_entries() {
    for id in 9..104 {
        let bytes = capture_case(id).unwrap();
        let digest = bytes.iter().fold(0xcbf29ce484222325u64, |h, &b| {
            (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
        });
        assert_eq!(
            digest,
            OLD_MATRIX[(id - 9) as usize],
            "old native matrix case {id}"
        );
    }
}
pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    use crate::game::projectile_initialization::Initialization;
    if !(0..104).contains(&id) {
        return None;
    }
    let mut g = Game::new();
    let loaded = RomImage::zeroed(0x200000);
    let native = RomImage::zeroed(0x200000);
    let seed = match id {
        99 | 102 => 79,
        100 => 80,
        101 => 97,
        103 => 77,
        _ => id,
    };
    fixture(seed, &mut g, &loaded, &native);
    g.loaded_rom = Some(loaded);
    g.register_native_rom(native.clone());
    // Real bus dependencies reached by Power Bomb's HDMA allocation.
    let hardware = if id == 100 {
        use crate::initial_state::InitialState;
        let mut snes = Box::new(crate::types::Snes::INITIAL);
        let mut dma = Box::new(crate::types::Dma::INITIAL);
        let mut cart = Box::new(crate::types::Cart {
            snes: &mut *snes,
            type_0: 0,
            rom: Default::default(),
            romSize: 0,
            ram: Default::default(),
            ramSize: 0,
        });
        snes.dma = &mut *dma;
        snes.cart = &mut *cart;
        snes.ram = g.g_ram.as_mut_ptr();
        dma.snes = &mut *snes;
        cart.snes = &mut *snes;
        g.g_snes = &mut *snes;
        Some((snes, dma, cart))
    } else {
        None
    };
    if id <= 8 {
        match id {
            0 => clear(0, &mut g),
            1 => clear(19, &mut g),
            2 => clear(0xffff, &mut g),
            3 => call_motion(0x90d4d2, 0, &mut g),
            4 => call_motion(0x90d4d2, 1, &mut g),
            5 => call_motion(0x90efd3, 0, &mut g),
            6 => call_motion(0x90efd3, 6, &mut g),
            7 => flare(&mut g),
            8 => grapple_flare(&mut g),
            _ => unreachable!(),
        }
    } else if id >= 99 {
        prepare(&mut g);
        match id {
            99 | 100 => put(&mut g, 0xc86, 1),
            102 => put(&mut g, 0xc86, 16),
            101 | 103 => {
                for a in (0x10002..0x10402).step_by(2) {
                    put(&mut g, a, 0x8000);
                }
            }
            _ => unreachable!(),
        }
        update(&mut g);
        if let Some((_, dma, _)) = &hardware {
            assert_eq!(dma.channel[2].bAdr, 0x28);
            assert_eq!(dma.channel[3].bAdr, 0x29);
            assert!(dma.channel[2].indirect && dma.channel[3].indirect);
            assert_eq!(dma.channel[2].mode, 0);
            assert_eq!(dma.channel[3].mode, 0);
        }
    } else if id <= 72 {
        match (id - 9) % 8 {
            0 => initialize(0, Initialization::Projectile, &mut g),
            1 => {
                initialize(0, Initialization::Projectile, &mut g);
                update(&mut g);
            }
            2 => {
                initialize(0, Initialization::Projectile, &mut g);
                draw_projectiles(&mut g);
                draw_explosions(&mut g);
            }
            3 => impact(0, &mut g),
            4 => retire(0, &mut g),
            5 => reflect(0, &mut g),
            6 => reset(&mut g),
            7 => update_trails(&mut g),
            _ => unreachable!(),
        }
    } else {
        prepare(&mut g);
        // Prove these cases reached their intended producers before retirement.
        let read = |a: usize| u16::from_le_bytes([g.g_ram[a], g.g_ram[a + 1]]);
        let (slot, motion, count) = if id <= 88 {
            let mode = ((id - 73) % 8) as usize;
            (
                [0, 0, 0, 0, 0, 0, 5, 5][mode],
                [
                    0xaef3, 0xb0e4, 0xaef3, 0xaef3, 0xaf68, 0xafe5, 0xb099, 0xb0ae,
                ][mode],
                [1, 1, 1, 1, 1, 2, 1, 1][mode],
            )
        } else if id <= 96 {
            (
                0,
                [0xda08, 0xcf09, 0xdb06, 0xd793][((id - 89) % 4) as usize],
                4,
            )
        } else {
            (5, 0xd8f7, 5)
        };
        assert_eq!(read(0xc68 + 2 * slot), motion, "producer motion {id}");
        assert_eq!(
            read(if slot == 5 { 0xcd2 } else { 0xcce }),
            count,
            "producer count {id}"
        );
        assert_ne!(read(0xc40 + 2 * slot), 0, "producer cursor {id}");
        if (id <= 88 && id >= 81) || (id >= 93 && id <= 96) || id == 98 {
            draw_projectiles(&mut g);
            draw_explosions(&mut g);
            retire((2 * slot) as u16, &mut g);
        }
    }
    assert!(!g.g_fail, "native diagnostic {id}");
    Some(g.g_ram.to_vec())
}

#[path = "player_projectiles_expanded.rs"]
pub(crate) mod expanded;

#[path = "player_projectiles_presentation_reset.rs"]
pub(crate) mod presentation_reset;
