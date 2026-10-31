//! Actual Runtime publication, full sprite columns, nested entry and unwind.
#![forbid(unsafe_code)]
use crate::{
    game::{
        session_encoding,
        session_runtime::Runtime,
        sprite_effect_host::{Aliases, ControllerInputs, SpriteMember},
        sprite_effect_state::LegacySpriteWordIndex,
    },
    game_state::Game,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn projection(h: &Runtime<'_>, expected: &[u8]) {
    let mut actual = expected.to_vec();
    session_encoding::encode(h.session(), |at, bytes| {
        actual[at..at + bytes.len()].copy_from_slice(bytes)
    });
    assert_eq!(actual, expected, "sprite.guard.whole-union");
    for at in (0xef78..0xf378).chain(0x1844..0x1846) {
        assert_eq!(
            h.byte(at),
            expected[at],
            "sprite.guard.projected-byte-{at:x}"
        );
    }
}
pub(crate) fn run() -> bool {
    catch_unwind(AssertUnwindSafe(check)).is_ok()
}
fn check() {
    let mut game = Game::new();
    for i in 0..game.ram().len() {
        let value = (i.wrapping_mul(37) ^ (i >> 8)) as u8;
        game.ram().write_byte(i, value);
    }
    let mut expected = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<_>>();
    let actor_x = 0xf78 + core::mem::offset_of!(crate::types::EnemyBase, x_pos);
    let actor_y = 0xf78 + core::mem::offset_of!(crate::types::EnemyBase, y_pos);
    let actor_frozen = 0xf78 + core::mem::offset_of!(crate::types::EnemyBase, frozen_timer);
    {
        let mut h = Runtime::new(&mut game);
        projection(&h, &expected);
        // Independently map every typed active/tail field to its expected address.
        for i in 0..32 {
            let s = &mut h.session_mut().sprite_effects.active[i];
            let fields = [
                &mut s.instruction,
                &mut s.timer,
                &mut s.appearance,
                &mut s.x.whole,
                &mut s.x.fraction,
                &mut s.y.whole,
                &mut s.y.fraction,
                &mut s.disable_flags,
            ];
            for (column, field) in fields.into_iter().enumerate() {
                let value = 0x3100 + (column * 64 + i) as u16;
                *field = value;
                put(&mut expected, 0xef78 + column * 128 + i * 2, value);
            }
            let t = &mut h.session_mut().sprite_compatibility;
            let fields = [
                &mut t.instruction[i],
                &mut t.timer[i],
                &mut t.appearance[i],
                &mut t.x[i],
                &mut t.x_fraction[i],
                &mut t.y[i],
                &mut t.y_fraction[i],
                &mut t.disable_flags[i],
            ];
            for (column, field) in fields.into_iter().enumerate() {
                let value = 0x7300 + (column * 64 + i) as u16;
                *field = value;
                put(&mut expected, 0xefb8 + column * 128 + i * 2, value);
            }
        }
        h.session_mut().sprite_effects.selected = 0x1357;
        put(&mut expected, 0x1844, 0x1357);
        h.session_mut().enemy_projectiles.spawn.point.x = 0x1234;
        // Actual broad sprite alias reaches an already adopted owner at F378.
        h.set_sprite_word_le_slice(
            SpriteMember::DisableFlags,
            LegacySpriteWordIndex(64),
            0x4567,
        );
        assert_eq!(
            h.session().room.enemy_processing_stage,
            0x4567,
            "sprite.guard.cross-owner-alias"
        );
        put(&mut expected, 0xf378, 0x4567);
        projection(&h, &expected);
        h.foreign(|g| {
            assert_eq!(
                (0..g.ram().len())
                    .map(|at| g.ram().read_byte(at))
                    .collect::<Vec<_>>()
                    .as_slice(),
                expected.as_slice(),
                "sprite.guard.publish"
            );
            assert_eq!(g.eproj_spawn_pt.x, 0x1234, "sprite.guard.native-publish");
            // Re-enter the actual owner while outer Runtime has relinquished it.
            {
                let mut nested = Runtime::new(g);
                nested.session_mut().sprite_effects.active[0].instruction = 0xbeef;
                nested.session_mut().enemy_projectiles.spawn.point.x = 0x5678;
            }
            put(&mut expected, 0xef78, 0xbeef);
            assert_eq!(
                (0..g.ram().len())
                    .map(|at| g.ram().read_byte(at))
                    .collect::<Vec<_>>()
                    .as_slice(),
                expected.as_slice(),
                "sprite.guard.nested-drop"
            );
            assert_eq!(g.eproj_spawn_pt.x, 0x5678);
            // Foreign writes change every new byte and every pre-existing owner.
            for at in 0..g.ram().len() {
                let value = g.ram().read_byte(at) ^ 0x5a;
                g.ram().write_byte(at, value);
            }
            for byte in &mut expected {
                *byte ^= 0x5a;
            }
            for (at, value) in [
                (actor_x, 0x1122u16),
                (actor_y, 0x3344),
                (actor_frozen, 0x5566),
            ] {
                let _ = &[(); 0x20000][at..at + 2];
                g.ram().copy_in(at, &value.to_ne_bytes());
                expected[at..at + 2].copy_from_slice(&value.to_ne_bytes());
            }
            g.ram().copy_in(
                crate::ram_map::words::DEBUG_TIME_FROZEN_FOR_ENEMIES,
                &(7u16).to_le_bytes(),
            );
            put(
                &mut expected,
                crate::ram_map::words::DEBUG_TIME_FROZEN_FOR_ENEMIES,
                7,
            );
            g.ram().copy_in(
                crate::ram_map::words::CUR_ENEMY_INDEX,
                &(0x80u16).to_le_bytes(),
            );
            put(&mut expected, crate::ram_map::words::CUR_ENEMY_INDEX, 0x80);
        });
        projection(&h, &expected);
        assert_eq!(
            h.session().enemy_projectiles.spawn.point.x,
            0x5678,
            "sprite.guard.native-import"
        );
        assert_eq!(
            (
                h.attachment_x(0),
                h.attachment_y(0),
                h.attachment_frozen_timer(0)
            ),
            (0x1122, 0x3344, 0x5566),
            "sprite.guard.live-actor"
        );
        assert_eq!(
            h.inputs().debug_enemy_freeze,
            7,
            "sprite.guard.refresh-freeze"
        );
        assert_eq!(
            h.inputs().current_enemy,
            0x80,
            "sprite.guard.refresh-actor-index"
        );
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
                for byte in &mut expected {
                    *byte ^= 0xa5;
                }
                g.eproj_spawn_pt.x = 0xdead;
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
            "sprite.guard.owner-relinquished"
        );
    }
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice(),
        expected.as_slice(),
        "sprite.guard.no-stale-drop"
    );
    assert_eq!(
        game.eproj_spawn_pt.x, 0xdead,
        "sprite.guard.no-stale-native-drop"
    );
    // XOR leaves debug freeze nonzero: real entry must preserve the entire pool.
    crate::game::sprite_effect_runtime::update_sprites(&mut game);
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice(),
        expected.as_slice(),
        "sprite.guard.new-frozen-entry"
    );
    let h = Runtime::new(&mut game);
    projection(&h, &expected);
    assert_eq!(
        h.session().enemy_projectiles.spawn.point.x,
        0xdead,
        "sprite.guard.new-entry"
    );
}
/// The fixture's own unwind, told apart from any engine panic by its type.
struct FixtureUnwind;
