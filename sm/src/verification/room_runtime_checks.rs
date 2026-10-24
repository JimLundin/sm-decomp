//! Actual Game/Runtime binding and room projection guards. No replacement owner.
#![forbid(unsafe_code)]
use crate::{
    game::{
        room_entry::{
            content::{AssetRef, DoorList},
            world::{Identity, WorldBinding},
        },
        session::Session,
        session_encoding,
        session_runtime::Runtime,
    },
    game_state::{Game, GameOwner},
    room_world_fixture as fixture,
};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
};

const RANGES: [(usize, usize); 13] = [
    (0x78d, 0x78f),
    (0x791, 0x79f),
    (0x7a1, 0x7a5),
    (0x7ad, 0x7b7),
    (0x7bb, 0x7d3),
    (0x7df, 0x7e1),
    (0x7e9, 0x7eb),
    (0x91b, 0x91d),
    (0x925, 0x933),
    (0x99c, 0x99e),
    (0xe16, 0xe1a),
    (0xa9, 0xab),
    (0x612, 0x614),
];
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn native(g: &Game) -> [u16; 8] {
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
fn set_native(g: &mut Game, v: [u16; 8]) {
    g.eproj_spawn_rect.x = v[0];
    g.eproj_spawn_rect.y = v[1];
    g.eproj_spawn_rect.w = v[2];
    g.eproj_spawn_rect.h = v[3];
    g.eproj_spawn_r22 = v[4];
    g.eproj_spawn_pt.x = v[5];
    g.eproj_spawn_pt.y = v[6];
    g.eproj_spawn_varE24 = v[7];
}
fn state_native(s: &Session) -> [u16; 8] {
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
fn projection(h: &Runtime<'_>, expected: &[u8], world: &Arc<WorldBinding>) {
    assert!(
        Arc::ptr_eq(&h.session().world, world),
        "room.guard.pinned-world"
    );
    let mut bytes = expected.to_vec();
    session_encoding::encode(h.session(), &mut bytes);
    assert_eq!(bytes, expected, "room.guard.whole-union");
    assert_eq!(RANGES.iter().map(|(a, b)| b - a).sum::<usize>(), 84);
    for (begin, end) in RANGES {
        for at in begin..end {
            assert_eq!(h.byte(at), expected[at], "room.guard.projected-byte-{at:x}");
        }
    }
}
pub(crate) fn run() -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        constructors();
        round_trip(fixture::binding(false));
        mismatch();
    }))
    .is_ok()
}
fn constructors() {
    let canonical = Game::new();
    assert!(canonical.supports_legacy_persistence());
    assert_eq!(
        canonical.g_runmode,
        crate::cpu_infra::RM_BOTH as u8,
        "room.guard.canonical-mode"
    );
    let owner = GameOwner::new();
    owner.with(|g| {
        assert!(g.supports_legacy_persistence());
        assert_eq!(
            g.g_runmode,
            crate::cpu_infra::RM_BOTH as u8,
            "room.guard.canonical-owner-mode"
        );
    });
    let world = fixture::binding(false);
    let prepared = Game::with_world(world.clone());
    assert_eq!(
        prepared.g_runmode,
        crate::cpu_infra::RM_MINE as u8,
        "room.guard.prepared-game-mode"
    );
    let owner = GameOwner::with_world(world.clone());
    owner.with(|g| {
        assert_eq!(
            g.g_runmode,
            crate::cpu_infra::RM_MINE as u8,
            "room.guard.prepared-mode"
        );
        assert!(
            !g.supports_legacy_persistence(),
            "room.guard.prepared-policy"
        );
        assert!(
            Arc::ptr_eq(g.world(), &world),
            "room.guard.owner-constructor"
        );
        let h = Runtime::new(g);
        assert!(
            Arc::ptr_eq(&h.session().world, &world),
            "room.guard.constructor-import"
        );
    });
}
fn round_trip(world: Arc<WorldBinding>) {
    let mut game = Game::with_world(world.clone());
    for (i, b) in game.g_ram.iter_mut().enumerate() {
        *b = (i.wrapping_mul(37) ^ (i >> 8)) as u8;
    }
    let original_native = [
        0x1201, 0x2302, 0x3403, 0x4504, 0x5605, 0x6706, 0x7807, 0x8908,
    ];
    let callback_native = [
        0x9121, 0xa232, 0xb343, 0xc454, 0xd565, 0xe676, 0xf787, 0x1898,
    ];
    set_native(&mut game, original_native);
    let mut expected = game.g_ram.to_vec();
    {
        let mut h = Runtime::new(&mut game);
        projection(&h, &expected, &world);
        // Every new scalar is populated through its actual typed field.
        let e = &mut h.session_mut().room.entry;
        for (at, field) in [
            (0x791, &mut e.direction),
            (0x793, &mut e.elevator_properties),
            (0x795, &mut e.elevator_interlock),
            (0x797, &mut e.enemy_interlock),
            (0x799, &mut e.elevator_direction),
            (0x79d, &mut e.room_index),
            (0x7a1, &mut e.map_x),
            (0x7a3, &mut e.map_y),
            (0x7ad, &mut e.scroll_up),
            (0x7af, &mut e.scroll_down),
            (0x7b1, &mut e.previous_cre),
            (0x7b3, &mut e.cre),
            (0x7c9, &mut e.music_track),
            (0x7cb, &mut e.music_data),
            (0x7cd, &mut e.fx_key),
            (0x7cf, &mut e.enemy_population_key),
            (0x7d1, &mut e.enemy_tilesets_key),
            (0x7df, &mut e.main_code_key),
            (0x7e9, &mut e.scrolling_hook),
            (0x925, &mut e.frame),
            (0x927, &mut e.arrival_x),
            (0x929, &mut e.arrival_y),
            (0x92b, &mut e.speed_fraction),
            (0x92d, &mut e.speed_whole),
            (0x92f, &mut e.elevator_delay),
            (0x931, &mut e.completion),
            (0x99c, &mut e.transition_selector),
            (0xe16, &mut e.elevator_flags),
            (0xe18, &mut e.elevator_status),
            (0xa9, &mut e.irq_policy),
        ] {
            let value = (at as u16) ^ 0xa56b;
            *field = value;
            put(&mut expected, at, value);
        }
        e.exit = Identity::LegacyUnresolved(0x1234);
        put(&mut expected, 0x78d, 0x1234);
        e.room = Identity::LegacyUnresolved(0x2345);
        put(&mut expected, 0x79b, 0x2345);
        e.state = Identity::LegacyUnresolved(0x3456);
        put(&mut expected, 0x7bb, 0x3456);
        e.doors = DoorList::Legacy(0x4567);
        put(&mut expected, 0x7b5, 0x4567);
        for (at, asset, bytes) in [
            (0x7bd, &mut e.level, [0x12, 0x34, 0x56]),
            (0x7c0, &mut e.tile_table, [0x23, 0x45, 0x67]),
            (0x7c3, &mut e.tiles, [0x34, 0x56, 0x78]),
            (0x7c6, &mut e.palette, [0x45, 0x67, 0x89]),
        ] {
            *asset = AssetRef::Legacy(bytes);
            expected[at..at + 3].copy_from_slice(&bytes);
        }
        e.layer2_x = 0x93;
        e.layer2_y = 0xa4;
        expected[0x91b] = 0x93;
        expected[0x91c] = 0xa4;
        h.session_mut().continuations.primary = 0xfe;
        h.session_mut().continuations.secondary = 0x81;
        expected[0x612] = 0xfe;
        expected[0x613] = 0x81;
        projection(&h, &expected, &world);
        // Actual byte projection must resolve intermediate unknown keys losslessly.
        h.session_mut().room.entry.room = Identity::Known(fixture::ROOM_A);
        put(&mut expected, 0x79b, fixture::ROOM_A_KEY);
        h.set_byte(0x79b, 1);
        expected[0x79b] = 1;
        assert_eq!(
            h.session().room.entry.room,
            Identity::LegacyUnresolved((fixture::ROOM_A_KEY & 0xff00) | 1),
            "room.guard.partial-key-unresolved"
        );
        h.set_byte(0x79b, fixture::ROOM_A_KEY as u8);
        expected[0x79b] = fixture::ROOM_A_KEY as u8;
        assert_eq!(
            h.session().room.entry.room,
            Identity::Known(fixture::ROOM_A),
            "room.guard.partial-key-known"
        );
        h.foreign(|g| {
            assert!(Arc::ptr_eq(g.world(), &world));
            assert_eq!(
                g.g_ram.as_slice(),
                expected.as_slice(),
                "room.guard.publish-all"
            );
            assert_eq!(native(g), original_native);
            {
                let mut nested = Runtime::new(g);
                nested.session_mut().continuations.primary = 2;
                nested.session_mut().continuations.secondary = 1;
                nested.session_mut().room.entry.room = Identity::Known(fixture::ROOM_B);
            }
            expected[0x612] = 2;
            expected[0x613] = 1;
            put(&mut expected, 0x79b, fixture::ROOM_B_KEY);
            assert_eq!(
                g.g_ram.as_slice(),
                expected.as_slice(),
                "room.guard.nested-publication"
            );
            for b in &mut g.g_ram {
                *b ^= 0x5a;
            }
            for b in &mut expected {
                *b ^= 0x5a;
            }
            set_native(g, callback_native);
        });
        projection(&h, &expected, &world);
        assert_eq!(
            state_native(h.session()),
            callback_native,
            "room.guard.native-import"
        );
        let error = catch_unwind(AssertUnwindSafe(|| {
            h.foreign(|g| {
                assert_eq!(g.g_ram.as_slice(), expected.as_slice());
                for b in &mut g.g_ram {
                    *b ^= 0xa5;
                }
                for b in &mut expected {
                    *b ^= 0xa5;
                }
                set_native(g, original_native);
                panic!("intentional room foreign unwind");
            })
        }))
        .expect_err("foreign must panic");
        assert_eq!(
            error.downcast_ref::<&str>().copied(),
            Some("intentional room foreign unwind")
        );
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let _ = h.session();
            }))
            .is_err(),
            "room.guard.owner-relinquished"
        );
    }
    assert_eq!(
        game.g_ram.as_slice(),
        expected.as_slice(),
        "room.guard.no-stale-drop"
    );
    assert_eq!(
        native(&game),
        original_native,
        "room.guard.no-stale-native-drop"
    );
    assert!(Arc::ptr_eq(game.world(), &world));
    let h = Runtime::new(&mut game);
    projection(&h, &expected, &world);
}
fn mismatch() {
    let first = fixture::binding(false);
    let other = fixture::binding(true);
    assert_ne!(
        first.revision(),
        other.revision(),
        "room.guard.fixture-distinct-revisions"
    );
    let mut game = Game::with_world(first);
    game.g_ram.fill(0x71);
    let before = game.g_ram.to_vec();
    let native_before = native(&game);
    let mut called = false;
    {
        let mut h = Runtime::new(&mut game);
        h.session_mut().room.entry.frame = 0xbeef;
        h.session_mut().inventory.health.current = 0x1234;
        h.session_mut().enemy_projectiles.spawn.variant = 0xbeef;
        h.session_mut().world = other;
        assert!(
            catch_unwind(AssertUnwindSafe(|| h.foreign(|_| called = true))).is_err(),
            "room.guard.mismatch-rejected"
        );
    }
    assert!(!called, "room.guard.mismatch-before-callback");
    assert_eq!(
        game.g_ram.as_slice(),
        before.as_slice(),
        "room.guard.mismatch-before-writes"
    );
    assert_eq!(
        native(&game),
        native_before,
        "room.guard.mismatch-native-unchanged"
    );
}
