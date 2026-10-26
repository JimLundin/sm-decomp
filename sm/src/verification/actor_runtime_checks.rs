//! Actual Game/Runtime ownership tests, registered only in the existing DLL verifier.
#![forbid(unsafe_code)]
use super::{
    content::{ActorTemplateId, SpeciesId},
    identity::{ActorBirthOrigin, ActorProvenance, InstanceHandle},
    media::AppearanceId,
};
use crate::{
    game::{
        room_entry::world::WorldBinding,
        session::Session,
        session_runtime::{Runtime, export_session},
    },
    game_state::Game,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn panic_text(error: Box<dyn core::any::Any + Send>) -> String {
    if let Some(text) = error.downcast_ref::<&str>() {
        (*text).to_owned()
    } else if let Some(text) = error.downcast_ref::<String>() {
        text.clone()
    } else {
        "unexpected non-text panic".to_owned()
    }
}
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
    super::initializer_checks::run_checked();
    super::owner_checks::run_checked();
    occupied_game_export_rejected_before_effects();
    mismatched_runtime_returns_original_identity_without_second_panic();
    nested_foreign_unwind_keeps_both_committed_occurrences();
    super::media_checks::run_checked();
    super::contact_checks::run_checked();
}

fn occupied_game_export_rejected_before_effects() {
    let mut game = Game::new();
    game.g_ram.fill(0x71);
    game.eproj_spawn_r22 = 0x4567;
    {
        let mut seed = Runtime::new(&mut game);
        birth(&mut seed, 0, 77);
    }
    let before_ram = game.g_ram.to_vec();
    let before_native = native(&game);
    let before_identity = game.actor_identity().clone();
    let mut state = Session::with_world(game.world().clone());
    state.room.entry.frame = 0xbeef;
    state.enemy_projectiles.spawn.variant = 0xabcd;
    let error = catch_unwind(AssertUnwindSafe(|| export_session(state, &mut game))).unwrap_err();
    assert_eq!(
        panic_text(error),
        "Game already owns actor identity",
        "actor.runtime.occupied_export_named_rejection"
    );
    assert_eq!(
        game.g_ram.as_slice() == before_ram.as_slice(),
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
    game.g_ram.fill(0x39);
    {
        let mut seed = Runtime::new(&mut game);
        birth(&mut seed, 1, 88);
    }
    let before_ram = game.g_ram.to_vec();
    let before_native = native(&game);
    let before_identity = game.actor_identity().clone();
    let mut called = false;
    let scope = catch_unwind(AssertUnwindSafe(|| {
        let mut runtime = Runtime::new(&mut game);
        runtime.session_mut().room.entry.frame = 0xbeef;
        runtime.session_mut().enemy_projectiles.spawn.variant = 0xabcd;
        runtime.session_mut().world = WorldBinding::canonical_legacy();
        let error =
            catch_unwind(AssertUnwindSafe(|| runtime.foreign(|_| called = true))).unwrap_err();
        assert_eq!(
            panic_text(error),
            "Session belongs to a different world binding"
        );
        // Scope exit must not panic again or publish the rejected Session.
    }));
    assert!(
        scope.is_ok(),
        "actor.runtime.mismatch_drop_does_not_panic_again"
    );
    assert!(!called, "actor.runtime.mismatch_before_callback");
    assert_eq!(
        game.g_ram.as_slice() == before_ram.as_slice(),
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
                panic!("actor runtime nested fixture unwind");
            });
        });
    }))
    .unwrap_err();
    assert_eq!(panic_text(error), "actor runtime nested fixture unwind");
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
