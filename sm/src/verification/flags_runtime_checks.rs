//! Actual shared Runtime guard for flag ownership and foreign lifecycle.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::{
    game::{session::Session, session_encoding, session_runtime::Runtime},
    game_state::Game,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn native_game(g: &Game) -> [u16; 8] {
    [
        g.eproj_spawn_rect.x,
        g.eproj_spawn_rect.y,
        g.eproj_spawn_rect.w,
        g.eproj_spawn_rect.h,
        g.eproj_spawn_r22,
        g.eproj_spawn_pt.x,
        g.eproj_spawn_pt.y,
        g.eproj_spawn_varE24,
    ]
}
fn set_native_game(g: &mut Game, v: [u16; 8]) {
    g.eproj_spawn_rect.x = v[0];
    g.eproj_spawn_rect.y = v[1];
    g.eproj_spawn_rect.w = v[2];
    g.eproj_spawn_rect.h = v[3];
    g.eproj_spawn_r22 = v[4];
    g.eproj_spawn_pt.x = v[5];
    g.eproj_spawn_pt.y = v[6];
    g.eproj_spawn_varE24 = v[7];
}
fn native_state(s: &Session) -> [u16; 8] {
    let n = &s.enemy_projectiles.spawn;
    [
        n.rectangle.origin.x,
        n.rectangle.origin.y,
        n.rectangle.extent.x,
        n.rectangle.extent.y,
        n.variant,
        n.point.x,
        n.point.y,
        n.drop_definition,
    ]
}
fn set_native_state(s: &mut Session, v: [u16; 8]) {
    let n = &mut s.enemy_projectiles.spawn;
    n.rectangle.origin.x = v[0];
    n.rectangle.origin.y = v[1];
    n.rectangle.extent.x = v[2];
    n.rectangle.extent.y = v[3];
    n.variant = v[4];
    n.point.x = v[5];
    n.point.y = v[6];
    n.drop_definition = v[7];
}
fn projection(h: &Runtime<'_>, expected: &[u8]) {
    let mut actual = expected.to_vec();
    session_encoding::encode(h.session(), |at, bytes| {
        actual[at..at + bytes.len()].copy_from_slice(bytes)
    });
    assert_eq!(actual, expected, "flags.guard.whole-union");
    for at in (0xd820..0xd8f0).chain(0x5e7..0x5e9) {
        assert_eq!(
            h.byte(at),
            expected[at],
            "flags.guard.projected-byte-{at:x}"
        );
    }
}
pub(crate) fn run() -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        check();
        reset_entry();
        load_entry(63);
        load_entry(65);
    }))
    .is_ok()
}
// Only the maintained native entry has an unsafe signature; no synthetic RNG.
fn next_random(g: &mut Game) -> u16 {
    g.next_random()
}
fn new_save_file(g: &mut Game) {
    sm_game::menus::save_slots::initialize(g)
}
fn load_from_sram(slot: u16, g: &mut Game) -> u8 {
    sm_game::menus::save_slots::load(slot, g)
}
/// The fixture's own unwind, told apart from any engine panic by its type.
struct FixtureUnwind;
#[forbid(unsafe_code)]
fn load_entry(case: u32) {
    let mut game = Game::new();
    game.ram().fill(0..game.ram().len(), 0x69);
    game.g_sram = sm_game::save_file::SaveRam::zeroed(8192);
    let mut image = vec![0; 0x200000];
    game.VERIFY_ACTIVE = true;
    let slot = crate::flags_cases::seed_load(case, &mut game, &mut image);
    crate::flags_cases::install(&mut game, &image);
    assert_eq!(slot, 0);
    let payload = game.g_sram.bytes(0x10..0x10 + 0x65c);
    let flags = &payload[0x60..0x130]; // D820..D8EF within D7C0 payload.
    let native = [
        0x1021, 0x2132, 0x3243, 0x4354, 0x5465, 0x6576, 0x7687, 0x8798,
    ];
    let mut after = Vec::new();
    {
        let mut h = Runtime::new(&mut game);
        h.session_mut().gameplay_flags.events.fill(0x17);
        h.session_mut().gameplay_flags.bosses_by_area.fill(0x28);
        h.session_mut().flag_compatibility.bitmask = 0xabcd;
        set_native_state(h.session_mut(), native);
        h.foreign(|g| {
            assert_eq!(
                (0xd820..0xd828)
                    .map(|at| g.ram().read_byte(at))
                    .collect::<Vec<_>>()
                    .as_slice(),
                &[0x17; 8],
                "flags.guard.load-publish"
            );
            assert_eq!(native_game(g), native);
            let status = load_from_sram(slot, g);
            assert_eq!(
                status,
                if case == 63 { 0 } else { 1 },
                "flags.guard.load-status"
            );
            assert_eq!(
                (0xd820..0xd8f0)
                    .map(|at| g.ram().read_byte(at))
                    .collect::<Vec<_>>()
                    .as_slice(),
                flags,
                "flags.guard.load-copied-flags"
            );
            assert_eq!(
                (0x5e7..0x5e9)
                    .map(|at| g.ram().read_byte(at))
                    .collect::<Vec<_>>()
                    .as_slice(),
                &[0xcd, 0xab],
                "flags.guard.load-scratch"
            );
            assert_eq!(
                session_encoding::read(&|at| g.ram().read_byte(at), 0x79f),
                if case == 63 { 2 } else { 0 },
                "flags.guard.load-area"
            );
            if case == 65 {
                assert!(
                    g.g_sram.bytes(0x10..0x10 + 0x65c).iter().all(|&b| b == 0),
                    "flags.guard.failed-load-clears-sram"
                );
            }
            assert_eq!(native_game(g), native, "flags.guard.load-native-preserved");
            after = (0..g.ram().len())
                .map(|at| g.ram().read_byte(at))
                .collect::<Vec<_>>();
        });
        projection(&h, &after);
        assert_eq!(
            h.session().gameplay_flags.events.as_slice(),
            &flags[..8],
            "flags.guard.load-import"
        );
        assert_eq!(native_state(h.session()), native);
    }
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice(),
        after.as_slice(),
        "flags.guard.load-no-stale-drop"
    );
    let h = Runtime::new(&mut game);
    projection(&h, &after);
    assert_eq!(
        native_state(h.session()),
        native,
        "flags.guard.load-new-entry"
    );
}
#[forbid(unsafe_code)]
fn reset_entry() {
    let mut game = Game::new();
    game.ram().fill(0..game.ram().len(), 0x69);
    let mut after = Vec::new();
    {
        let mut h = Runtime::new(&mut game);
        h.session_mut().gameplay_flags.events.fill(0x91);
        h.session_mut().gameplay_flags.bosses_by_area.fill(0x82);
        h.session_mut().gameplay_flags.used_room_objects.fill(0x73);
        h.session_mut()
            .gameplay_flags
            .collected_locations
            .fill(0x64);
        h.session_mut().gameplay_flags.opened_doors.fill(0x55);
        h.session_mut().flag_compatibility.bitmask = 0xabcd;
        h.foreign(|g| {
            assert_eq!(
                (0xd830..0xd870)
                    .map(|at| g.ram().read_byte(at))
                    .collect::<Vec<_>>()
                    .as_slice(),
                &[0x73; 64]
            );
            new_save_file(g);
            assert_eq!(
                (0xd820..0xd828)
                    .map(|at| g.ram().read_byte(at))
                    .collect::<Vec<_>>()
                    .as_slice(),
                &[0x91; 8],
                "flags.guard.reset-events-survive"
            );
            assert_eq!(
                (0xd828..0xd830)
                    .map(|at| g.ram().read_byte(at))
                    .collect::<Vec<_>>()
                    .as_slice(),
                &[0x82; 8],
                "flags.guard.reset-bosses-survive"
            );
            assert_eq!(
                (0xd830..0xd8f0)
                    .map(|at| g.ram().read_byte(at))
                    .collect::<Vec<_>>()
                    .as_slice(),
                &[0; 192],
                "flags.guard.reset-collections"
            );
            assert_eq!(
                (0x5e7..0x5e9)
                    .map(|at| g.ram().read_byte(at))
                    .collect::<Vec<_>>()
                    .as_slice(),
                &[0xcd, 0xab],
                "flags.guard.reset-scratch-survives"
            );
            after = (0..g.ram().len())
                .map(|at| g.ram().read_byte(at))
                .collect::<Vec<_>>();
        });
        projection(&h, &after);
        assert_eq!(h.session().gameplay_flags.opened_doors, [0; 64]);
    }
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice(),
        after.as_slice(),
        "flags.guard.reset-no-stale-drop"
    );
    let h = Runtime::new(&mut game);
    projection(&h, &after);
}
#[forbid(unsafe_code)]
fn check() {
    let mut game = Game::new();
    for i in 0..game.ram().len() {
        let value = (i.wrapping_mul(37) ^ (i >> 8)) as u8;
        game.ram().write_byte(i, value);
    }
    let mut expected = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<_>>();
    let initial = [
        0x1201, 0x2302, 0x3403, 0x4504, 0x5605, 0x6706, 0x7807, 0x8908,
    ];
    let changed = [
        0x9121, 0xa232, 0xb343, 0xc454, 0xd565, 0xe676, 0xf787, 0x1898,
    ];
    let final_native = [
        0x2931, 0x3a42, 0x4b53, 0x5c64, 0x6d75, 0x7e86, 0x8f97, 0x90a8,
    ];
    {
        let mut h = Runtime::new(&mut game);
        projection(&h, &expected);
        let f = &mut h.session_mut().gameplay_flags;
        for (base, field) in [
            (0xd820, &mut f.events[..]),
            (0xd828, &mut f.bosses_by_area[..]),
            (0xd830, &mut f.used_room_objects[..]),
            (0xd870, &mut f.collected_locations[..]),
            (0xd8b0, &mut f.opened_doors[..]),
        ] {
            for (i, b) in field.iter_mut().enumerate() {
                *b = (base as u8).wrapping_add((i as u8).wrapping_mul(19)) ^ 0xa7;
                expected[base + i] = *b;
            }
        }
        h.session_mut().flag_compatibility.bitmask = 0xabcd;
        put(&mut expected, 0x5e7, 0xabcd);
        h.session_mut().room.area = 7;
        put(&mut expected, 0x79f, 7);
        set_native_state(h.session_mut(), initial);
        crate::game::progress_flags::mark_boss_defeated(&mut h, 0x1234);
        expected[0xd82f] |= 0x34;
        expected[0x5e7] = 0x34;
        assert_eq!(
            h.session().flag_compatibility.bitmask,
            0xab34,
            "flags.guard.boss-low-scratch"
        );
        assert_eq!(h.session().room.area, 7, "flags.guard.shared-area");
        crate::game::progress_flags::set_event(&mut h, 64);
        expected[0xd828] |= 1;
        put(&mut expected, 0x5e7, 1);
        assert_eq!(
            h.session().flag_compatibility.bitmask,
            1,
            "flags.guard.event-full-scratch"
        );
        projection(&h, &expected);
        h.foreign(|g| {
            assert_eq!(
                (0..g.ram().len())
                    .map(|at| g.ram().read_byte(at))
                    .collect::<Vec<_>>()
                    .as_slice(),
                expected.as_slice(),
                "flags.guard.publish-all"
            );
            assert_eq!(native_game(g), initial, "flags.guard.native-publish");
            {
                let mut nested = Runtime::new(g);
                nested.session_mut().gameplay_flags.events[0] = 0x6b;
                nested.session_mut().flag_compatibility.bitmask = 0x9132;
                nested.session_mut().room.area = 3;
                set_native_state(nested.session_mut(), changed);
            }
            expected[0xd820] = 0x6b;
            put(&mut expected, 0x5e7, 0x9132);
            put(&mut expected, 0x79f, 3);
            assert_eq!(
                (0..g.ram().len())
                    .map(|at| g.ram().read_byte(at))
                    .collect::<Vec<_>>()
                    .as_slice(),
                expected.as_slice(),
                "flags.guard.nested-publication"
            );
            assert_eq!(native_game(g), changed);
            for at in 0..g.ram().len() {
                let value = g.ram().read_byte(at) ^ 0x5a;
                g.ram().write_byte(at, value);
            }
            for b in &mut expected {
                *b ^= 0x5a;
            }
            // Scratch and RNG are separate: both must be refreshed after callback.
            g.ram().copy_in(0x5e7, &(0x5634u16).to_le_bytes());
            put(&mut expected, 0x5e7, 0x5634);
            g.ram().copy_in(0x5e5, &(0xab89u16).to_le_bytes());
            put(&mut expected, 0x5e5, 0xab89);
        });
        projection(&h, &expected);
        assert_eq!(
            native_state(h.session()),
            changed,
            "flags.guard.native-import"
        );
        assert_eq!(
            h.session().flag_compatibility.bitmask,
            0x5634,
            "flags.guard.callback-scratch"
        );
        assert_eq!(h.inputs().random, 0xab89, "flags.guard.callback-random");
        // Scratch must survive the actual NextRandom foreign trip.
        // Independent arithmetic: 89*5=02AD, AB*5(low)=57, high=5A,
        // then add17 =>5ABE (no carry beyond the high byte).
        let random = h.foreign(next_random);
        assert_eq!(random, 0x5abe, "flags.guard.real-rng-return");
        put(&mut expected, 0x5e5, 0x5abe);
        assert_eq!(h.inputs().random, 0x5abe, "flags.guard.real-rng-import");
        assert_eq!(
            h.session().flag_compatibility.bitmask,
            0x5634,
            "flags.guard.real-rng-scratch"
        );
        // LE scalar spans the odd scratch address; low-byte writes preserve05E8.
        h.set_byte(0x5e7, 0x81);
        expected[0x5e7] = 0x81;
        assert_eq!(
            h.session().flag_compatibility.bitmask,
            0x5681,
            "flags.guard.scratch-low-byte"
        );
        h.write_slice_le(0x5e7, 0x0080);
        put(&mut expected, 0x5e7, 0x0080);
        assert_eq!(
            h.session().flag_compatibility.bitmask,
            0x0080,
            "flags.guard.scratch-le-word"
        );
        projection(&h, &expected);
        let error = catch_unwind(AssertUnwindSafe(|| {
            h.foreign(|g| {
                assert_eq!(
                    (0..g.ram().len())
                        .map(|at| g.ram().read_byte(at))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    expected.as_slice()
                );
                for at in 0..g.ram().len() {
                    let value = g.ram().read_byte(at) ^ 0xa5;
                    g.ram().write_byte(at, value);
                }
                for b in &mut expected {
                    *b ^= 0xa5;
                }
                set_native_game(g, final_native);
                std::panic::panic_any(FixtureUnwind);
            })
        }))
        .expect_err("foreign callback must panic");
        assert!(error.is::<FixtureUnwind>());
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let _ = h.session();
            }))
            .is_err(),
            "flags.guard.owner-relinquished"
        );
    }
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice(),
        expected.as_slice(),
        "flags.guard.no-stale-drop"
    );
    assert_eq!(
        native_game(&game),
        final_native,
        "flags.guard.no-stale-native-drop"
    );
    {
        let h = Runtime::new(&mut game);
        projection(&h, &expected);
        assert_eq!(
            native_state(h.session()),
            final_native,
            "flags.guard.new-entry"
        );
    }
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice(),
        expected.as_slice(),
        "flags.guard.new-entry-drop"
    );
}
