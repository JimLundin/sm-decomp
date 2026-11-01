//! Actual Game/Runtime ownership tests, registered only in the existing DLL verifier.
#![forbid(unsafe_code)]
use crate::{
    game::{
        room_entry::world::WorldBinding,
        session::Session,
        session_runtime::{Runtime, export_session},
    },
    game_state::Game,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
use {
    crate::game::actors::identity::{ActorBirthOrigin, ActorProvenance, InstanceHandle},
    sm_game::actors::{
        content::{ActorTemplateId, SpeciesId},
        media::AppearanceId,
    },
};

/// The fixture's own unwind, told apart from any engine panic by its type.
struct FixtureUnwind;
fn native(game: &Game) -> [u16; 8] {
    [
        game.eproj_spawn_rect.x,
        game.eproj_spawn_rect.y,
        game.eproj_spawn_rect.w,
        game.eproj_spawn_rect.h,
        game.eproj_spawn_r22,
        game.eproj_spawn_pt.x,
        game.eproj_spawn_pt.y,
        game.eproj_spawn_varE24,
    ]
}
fn birth(runtime: &mut Runtime<'_>, slot: u16, appearance: u32) -> InstanceHandle {
    let pending = runtime
        .session()
        .actor_identity
        .prepare_birth(
            slot,
            ActorProvenance {
                authored_species: Some(SpeciesId(7)),
                origin: ActorBirthOrigin::Dynamic {
                    template: ActorTemplateId(11),
                    part: slot,
                },
            },
            Some(AppearanceId(appearance)),
        )
        .unwrap();
    runtime.session_mut().actors.slots[usize::from(slot)].definition = 0xd000;
    runtime.session_mut().actor_identity.commit_birth(pending)
}

pub(crate) fn run_checked() {
    crate::game::actors::initializer_checks::run_checked();
    crate::game::actors::owner_checks::run_checked();
    occupied_game_export_rejected_before_effects();
    mismatched_runtime_returns_original_identity_without_second_panic();
    nested_foreign_unwind_keeps_both_committed_occurrences();
    crate::game::actors::media_checks::run_checked();
    crate::game::actors::contact_checks::run_checked();
    botwoon_maw_publication_preserves_owned_aliases_and_capture_epoch();
    player_lease::run_checked();
    boss_dispatch::spore_aliases_reuse_actual_runtime();
    boss_dispatch::physical_eye_failure_preserves_allocation_prefix();
}

#[path = "boss_dispatch_boundaries.rs"]
mod boss_dispatch;

#[path = "player_lease_checks.rs"]
mod player_lease;

fn occupied_game_export_rejected_before_effects() {
    let mut game = Game::new();
    game.ram().fill(0..game.ram().len(), 0x71);
    game.eproj_spawn_r22 = 0x4567;
    {
        let mut seed = Runtime::new(&mut game);
        birth(&mut seed, 0, 77);
    }
    let before_ram = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<_>>();
    let before_native = native(&game);
    let before_identity = game.actor_identity().clone();
    let mut state = Session::with_world(game.world().clone());
    state.room.entry.frame = 0xbeef;
    state.enemy_projectiles.spawn.variant = 0xabcd;
    assert!(
        catch_unwind(AssertUnwindSafe(|| export_session(state, &mut game))).is_err(),
        "actor.runtime.occupied_export_rejected"
    );
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice()
            == before_ram.as_slice(),
        true,
        "actor.runtime.occupied_export_before_ram"
    );
    assert_eq!(
        native(&game),
        before_native,
        "actor.runtime.occupied_export_before_native"
    );
    assert_eq!(
        game.actor_identity(),
        &before_identity,
        "actor.runtime.occupied_export_identity_intact"
    );
}

fn mismatched_runtime_returns_original_identity_without_second_panic() {
    let mut game = Game::new();
    game.ram().fill(0..game.ram().len(), 0x39);
    {
        let mut seed = Runtime::new(&mut game);
        birth(&mut seed, 1, 88);
    }
    let before_ram = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<_>>();
    let before_native = native(&game);
    let before_identity = game.actor_identity().clone();
    let mut called = false;
    let scope = catch_unwind(AssertUnwindSafe(|| {
        let mut runtime = Runtime::new(&mut game);
        runtime.session_mut().room.entry.frame = 0xbeef;
        runtime.session_mut().enemy_projectiles.spawn.variant = 0xabcd;
        runtime.session_mut().world = WorldBinding::canonical_legacy();
        assert!(
            catch_unwind(AssertUnwindSafe(|| runtime.foreign(|_| called = true))).is_err(),
            "actor.runtime.mismatch_rejected"
        );
        // Scope exit must not panic again or publish the rejected Session.
    }));
    assert!(
        scope.is_ok(),
        "actor.runtime.mismatch_drop_does_not_panic_again"
    );
    assert!(!called, "actor.runtime.mismatch_before_callback");
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice()
            == before_ram.as_slice(),
        true,
        "actor.runtime.mismatch_before_ram"
    );
    assert_eq!(
        native(&game),
        before_native,
        "actor.runtime.mismatch_before_native"
    );
    assert_eq!(
        game.actor_identity(),
        &before_identity,
        "actor.runtime.mismatch_original_identity_recovered"
    );
}

fn nested_foreign_unwind_keeps_both_committed_occurrences() {
    let mut game = Game::new();
    let mut outer_handle = None;
    let mut nested_handle = None;
    let error = catch_unwind(AssertUnwindSafe(|| {
        let mut outer = Runtime::new(&mut game);
        outer_handle = Some(birth(&mut outer, 0, 20));
        outer.foreign(|game| {
            assert!(
                !game.actor_identity_is_with_session(),
                "actor.runtime.foreign_has_sole_identity"
            );
            let mut nested = Runtime::new(game);
            nested_handle = Some(birth(&mut nested, 1, 21));
            nested.foreign(|game| {
                assert_eq!(
                    game.actor_identity()
                        .observe(1, true, Some(SpeciesId(7)))
                        .appearance,
                    Some(AppearanceId(21)),
                    "actor.runtime.nested_published_before_callback"
                );
                std::panic::panic_any(FixtureUnwind);
            });
        });
    }))
    .unwrap_err();
    assert!(
        error.is::<FixtureUnwind>(),
        "actor.runtime.nested_fixture_unwind"
    );
    let restored = Runtime::new(&mut game);
    let state = restored.session();
    assert_eq!(
        state
            .actor_identity
            .resolve(outer_handle.unwrap(), &state.actors)
            .unwrap()
            .byte_index(),
        0
    );
    assert_eq!(
        state
            .actor_identity
            .resolve(nested_handle.unwrap(), &state.actors)
            .unwrap()
            .byte_index(),
        64
    );
    assert_eq!(
        state
            .actor_identity
            .observe(0, true, Some(SpeciesId(7)))
            .birth_order,
        Some(0)
    );
    assert_eq!(
        state
            .actor_identity
            .observe(1, true, Some(SpeciesId(7)))
            .birth_order,
        Some(1),
        "actor.runtime.nested_unwind_keeps_birth_order"
    );
    assert_eq!(
        state
            .actor_identity
            .observe(1, true, Some(SpeciesId(7)))
            .appearance,
        Some(AppearanceId(21)),
        "actor.runtime.nested_unwind_keeps_appearance"
    );
}

// Movies do not reach Maw's full/odd protocol aliases or its captured-gamma lag.
fn botwoon_maw_publication_preserves_owned_aliases_and_capture_epoch() {
    for actor in [0x0f, 0x10, 0x11, 0x7fd6] {
        let mut game = Game::new();
        for (at, value) in [
            (0x801e, 0x8123u16),
            (0x8020, 0x4567),
            (0x803c, 0x3456),
            (0x8818, 0x89ab),
            (0x883e, 0xcdef),
        ] {
            game.ram().copy_in(at, &value.to_ne_bytes());
        }
        game.ram().copy_in(0xe54, &(actor as u16).to_le_bytes());
        game.ram().copy_in(0x911, &0u16.to_le_bytes());
        let x = 0xf78
            + usize::try_from(actor).unwrap()
            + core::mem::offset_of!(crate::types::EnemyData, x_pos);
        game.ram().copy_in(x, &1024u16.to_le_bytes());
        game.ram().copy_in(0x10002, &0x3456u16.to_le_bytes());
        game.ram().copy_in(0xa5a, &0x1357u16.to_le_bytes());
        let mut expected: Vec<_> = (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect();
        expected[0xa5a..0xa5c].copy_from_slice(&0x2468u16.to_le_bytes());
        if actor != 0x7fd6 {
            let at = 0x802c + usize::try_from(actor).unwrap();
            expected[at..at + 2].copy_from_slice(&1u16.to_ne_bytes());
        }
        {
            let mut runtime = Runtime::new(&mut game);
            assert!(crate::game::actors::api::current_outside_screen(&runtime));
            assert_eq!(runtime.inputs().transitions.frame_gamma.to_raw(), 0x1357);
            runtime.session_mut().player.transitions.frame_gamma =
                crate::game::player_transitions::state::FrameGammaMode::from_raw(0x2468);
            runtime.update_maw_visibility(actor);
            assert_eq!(
                runtime.inputs().transitions.frame_gamma.to_raw(),
                0x1357,
                "botwoon.maw.capture_epoch actor={actor:x}"
            );
        }
        assert_eq!(
            (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .collect::<Vec<_>>(),
            expected,
            "botwoon.maw.whole_publication actor={actor:x}"
        );
    }
}
