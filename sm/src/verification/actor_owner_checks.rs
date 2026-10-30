//! Actual Actor/eproj angle ownership across Runtime foreign handoffs.
//! New ownership invariant, not a simulated codec or predecessor capture.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::actors::access::{AccessMode, ActorAccess, ActorKey};

const ANGLES: usize = 0x97dc;
const COUNT: usize = 18;

fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn get(bytes: &crate::game_state::WorkRam, at: usize) -> u16 {
    u16::from_le_bytes([bytes.read_byte(at), bytes.read_byte(at + 1)])
}
fn key(slot: usize) -> ActorKey {
    // Actor.health is at +14. This source-derived address aliases each existing
    // eproj angle, including the last word97FE, without a production map helper.
    ActorKey::from_legacy((ANGLES + 2 * slot - 0x0f78 - 0x14) as u16)
}
fn check_ram(game: &Game, expected: &[u8], phase: &str, angle_base: u16) {
    for slot in 0..COUNT {
        assert_eq!(
            get(game.ram(), ANGLES + 2 * slot),
            angle_base + slot as u16,
            "actor.owner.angle_roundtrip.{phase}.angle slot={slot}"
        );
    }
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice()
            == expected,
        true,
        "actor.owner.angle_roundtrip.{phase}.whole_ram"
    );
}

/// Called by the existing checked actor Runtime suite, itself reached by the
/// existing sm_verify_room_runtime_guard DLL export. No new runner is needed.
pub(crate) fn run_checked() {
    let mut game = Game::new();
    for slot in 0..COUNT {
        game.ram()
            .copy_in(ANGLES + 2 * slot, &(0x1000 + slot as u16).to_le_bytes());
    }
    game.ram().write_byte(ANGLES - 1, 0xa7);
    game.ram().write_byte(ANGLES + COUNT * 2, 0xb9);
    let mut expected = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<_>>();
    {
        let mut runtime = Runtime::new(&mut game);
        runtime.edit_actor(
            ActorKey::from_legacy(0),
            AccessMode::LittleEndian,
            |actor| {
                actor.health = 0x6543;
            },
        );
        put(&mut expected, 0x0f78 + 0x14, 0x6543);

        for slot in 0..COUNT {
            let from_actor = 0x2100 + slot as u16;
            runtime.edit_actor(key(slot), AccessMode::LittleEndian, |actor| {
                actor.health = from_actor;
            });
            assert_eq!(
                runtime.session().enemy_projectiles.slots[slot].angle,
                from_actor,
                "actor.owner.angle_roundtrip.actor_to_eproj.angle slot={slot}"
            );

            // This is the existing normal eproj owned-state operation, not an
            // exceptional alias accessor or fixture-maintained second owner.
            let from_eproj = 0x3200 + slot as u16;
            runtime.session_mut().enemy_projectiles.slots[slot].angle = from_eproj;
            assert_eq!(
                runtime
                    .resolve_actor(key(slot), AccessMode::LittleEndian)
                    .health,
                from_eproj,
                "actor.owner.angle_roundtrip.eproj_to_actor.angle slot={slot}"
            );
            put(&mut expected, ANGLES + 2 * slot, from_eproj);
        }

        runtime.foreign(|game| {
            // F5 must fail this exact angle assertion if actor export restores
            // the import-time duplicate1000 over the current eproj value3200.
            check_ram(game, &expected, "foreign_entry", 0x3200);
            {
                let mut nested = Runtime::new(game);
                for slot in 0..COUNT {
                    let from_eproj = 0x4300 + slot as u16;
                    nested.session_mut().enemy_projectiles.slots[slot].angle = from_eproj;
                    assert_eq!(
                        nested
                            .resolve_actor(key(slot), AccessMode::LittleEndian)
                            .health,
                        from_eproj,
                        "actor.owner.angle_roundtrip.nested_eproj_to_actor.angle slot={slot}"
                    );
                    let from_actor = 0x5400 + slot as u16;
                    nested.edit_actor(key(slot), AccessMode::LittleEndian, |actor| {
                        actor.health = from_actor;
                    });
                    assert_eq!(
                        nested.session().enemy_projectiles.slots[slot].angle,
                        from_actor,
                        "actor.owner.angle_roundtrip.nested_actor_to_eproj.angle slot={slot}"
                    );
                    put(&mut expected, ANGLES + 2 * slot, from_actor);
                }
            }
            check_ram(game, &expected, "nested_export", 0x5400);
        });

        for slot in 0..COUNT {
            assert_eq!(
                runtime.session().enemy_projectiles.slots[slot].angle,
                0x5400 + slot as u16,
                "actor.owner.angle_roundtrip.reimport_eproj.angle slot={slot}"
            );
            assert_eq!(
                runtime
                    .resolve_actor(key(slot), AccessMode::LittleEndian)
                    .health,
                0x5400 + slot as u16,
                "actor.owner.angle_roundtrip.reimport_actor.angle slot={slot}"
            );
            runtime.session_mut().enemy_projectiles.slots[slot].angle = 0x6500 + slot as u16;
            put(&mut expected, ANGLES + 2 * slot, 0x6500 + slot as u16);
        }
        runtime.foreign(|game| check_ram(game, &expected, "second_foreign", 0x6500));
    }
    check_ram(&game, &expected, "final_drop", 0x6500);
}
