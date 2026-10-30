//! Candidate-only actual spawn/initializer publication and nested-unwind checks.
//! No predecessor carrier bytes or assertions are changed by this module.
#![forbid(unsafe_code)]
use crate::{
    game::{
        room_entry::world::{CompatibilityKeys, ResourceId, WorldBinding},
        session::Point,
        session_runtime::Runtime,
    },
    game_state::Game,
    rom::{RomImage, lorom_offset},
    types::EnemyDef,
};
use core::mem::offset_of;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
};
use {
    crate::game::actors::{
        api::ActorSpawnRequest,
        identity::{ActorBirthOrigin, ActorObservation},
    },
    sm_game::actors::{
        content::{
            ActorProgramDefinition, ActorProgramId, ActorTemplateId, PopulationEntry,
            PopulationEntryId, SpeciesId,
        },
        media::AppearanceId,
    },
};

const UNWIND: &str = "actor initializer fixture deliberate nested dispatch unwind";
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Single,
    NestedReturn,
    NestedUnwind,
}
struct Capture {
    mode: Mode,
    observed: Vec<ActorObservation>,
}
thread_local! {
    static ACTIVE: RefCell<Option<Capture>> = const { RefCell::new(None) };
}
struct ActiveGuard;
impl ActiveGuard {
    fn begin(mode: Mode) -> Self {
        ACTIVE.with(|cell| {
            assert!(
                cell.borrow().is_none(),
                "actor.initializer.fixture_not_reentrant"
            );
            *cell.borrow_mut() = Some(Capture {
                mode,
                observed: Vec::new(),
            });
        });
        Self
    }
    fn finish(self) -> Capture {
        ACTIVE.with(|cell| {
            cell.borrow_mut()
                .take()
                .expect("active initializer fixture")
        })
    }
}
impl Drop for ActiveGuard {
    fn drop(&mut self) {
        ACTIVE.with(|cell| {
            cell.borrow_mut().take();
        });
    }
}
fn panic_text(error: Box<dyn core::any::Any + Send>) -> String {
    if let Some(text) = error.downcast_ref::<&str>() {
        (*text).to_owned()
    } else if let Some(text) = error.downcast_ref::<String>() {
        text.clone()
    } else {
        "unexpected non-text panic".to_owned()
    }
}

/// Called only inside the existing real A2804C dispatch hook. When dormant it
/// checks one TLS cell and returns: no Runtime, foreign transition or APAN event.
pub(super) fn dispatch_noop(game: &mut Game) {
    let active = ACTIVE.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|capture| (capture.mode, capture.observed.len()))
    });
    let Some((mode, ordinal)) = active else {
        return;
    };
    assert!(ordinal < 2, "actor.initializer.exact_dispatch_count");
    assert!(
        !game.actor_identity_is_with_session(),
        "actor.initializer.game_owns_payload"
    );
    let key = game.cur_enemy_index();
    assert_eq!(
        key,
        (ordinal as u16) * 64,
        "actor.initializer.actual_selected_slot"
    );
    let observation = game
        .actor_identity()
        .observe(ordinal as u16, true, Some(SpeciesId(7)));
    // This assertion must precede handle unwrap/resolution: F2 is killed by
    // missing publication, never by an unrelated Option/borrow panic.
    assert!(
        observation.handle.is_some(),
        "actor.initializer.first_callback_occurrence_committed"
    );
    assert_eq!(
        observation.birth_order,
        Some(ordinal as u64),
        "actor.initializer.first_callback_birth_order"
    );
    assert_eq!(
        observation.authored_species,
        Some(SpeciesId(7)),
        "actor.initializer.first_callback_species"
    );
    assert_eq!(
        observation.appearance,
        Some(AppearanceId(20 + ordinal as u32)),
        "actor.initializer.first_callback_appearance"
    );
    assert_eq!(
        game.actor_identity()
            .initializer_fixture_origin(ordinal as u16),
        Some(ActorBirthOrigin::Dynamic {
            template: ActorTemplateId(11 + ordinal as u32),
            part: 0
        }),
        "actor.initializer.first_callback_authored_occurrence"
    );
    assert_eq!(
        u16::from_ne_bytes({
            let mut bytes = [0; 2];
            game.ram().copy_out(0xf78 + usize::from(key), &mut bytes);
            bytes
        }),
        0xd000,
        "actor.initializer.definition_already_written"
    );
    ACTIVE.with(|cell| {
        cell.borrow_mut()
            .as_mut()
            .unwrap()
            .observed
            .push(observation)
    });
    // Release the observer borrow before the actual nested Runtime/spawn entry.
    if ordinal == 0 && mode != Mode::Single {
        let handle = crate::game::actors::population::spawn(
            &mut Runtime::new(game),
            ActorSpawnRequest {
                template: ActorTemplateId(12),
                appearance: Some(AppearanceId(21)),
            },
        )
        .expect("nested actual spawn succeeds")
        .expect("nested slot available");
        let expected = ACTIVE.with(|cell| cell.borrow().as_ref().unwrap().observed[1].handle);
        assert_eq!(
            Some(handle),
            expected,
            "actor.initializer.nested_return_matches_observed_handle"
        );
        assert_eq!(
            game.cur_enemy_index(),
            0,
            "actor.initializer.nested_restores_outer_selector"
        );
    } else if ordinal == 1 && mode == Mode::NestedUnwind {
        // Deliberate injected unwind at a real initializer dispatch, NOT a
        // natural record-bounds failure or predecessor fidelity claim.
        panic!("{UNWIND}");
    }
}

fn binding() -> Arc<WorldBinding> {
    let base = sm_game::actors::fixture::binding(false, AppearanceId(20), false);
    let world = base.definition().unwrap().clone();
    let keys = CompatibilityKeys {
        starts: BTreeMap::new(),
        rooms: BTreeMap::new(),
        states: BTreeMap::new(),
        exits: BTreeMap::new(),
        door_lists: BTreeMap::new(),
        resources: world
            .resources
            .keys()
            .map(|&id| (id, base.resource_key(id)))
            .collect(),
    };
    let mut actors = base.actors().definitions().unwrap().clone();
    let mut actor_keys = base.actors().keys().unwrap().clone();
    let media = base.actor_media().definitions().unwrap().clone();
    actors.programs.insert(
        ActorProgramId(70),
        ActorProgramDefinition::CompatibleEnemyScript,
    );
    actor_keys
        .programs
        .insert(ActorProgramId(70), (0xa2, 0x9000));
    for (template, entry, address, appearance) in [(11, 100, 0x8000, 20), (12, 101, 0x8040, 21)] {
        actors.templates.insert(
            ActorTemplateId(template),
            PopulationEntry {
                id: PopulationEntryId(entry),
                species: SpeciesId(7),
                appearance: Some(AppearanceId(appearance)),
                position: Point {
                    x: 40 + template as u16,
                    y: 80,
                },
                initial_program: ActorProgramId(70),
                properties: 0,
                extra_properties: 0,
                parameters: [0, 0],
            },
        );
        actor_keys
            .templates
            .insert(ActorTemplateId(template), (0xa1, address));
    }
    WorldBinding::prepared_with_actors(world, keys, actors, actor_keys, media).unwrap()
}
fn word(image: &RomImage, address: u32, value: u16) {
    let at = lorom_offset(address) as usize;
    for (lane, byte) in value.to_ne_bytes().into_iter().enumerate() {
        image.write(at + lane, byte);
    }
}
fn game() -> Box<Game> {
    let mut game = Game::with_world(binding());
    let image = RomImage::zeroed(0x200000);
    // All source bytes initialized, full owned allocation, harmless actual init.
    for (field, value) in [
        (offset_of!(EnemyDef, bank), 0x04a2),
        (offset_of!(EnemyDef, ai_init), 0x804c),
        (offset_of!(EnemyDef, num_parts), 1),
        (offset_of!(EnemyDef, name_ptr), 0),
    ] {
        word(&image, 0xa0d000 + field as u32, value);
    }
    for address in [0xa18000, 0xa18040] {
        for (lane, value) in [0xd000, 51, 80, 0x9000, 0, 0, 0, 0].into_iter().enumerate() {
            word(&image, address + lane as u32 * 2, value);
        }
    }
    game.register_rom(image.clone());
    game.register_native_rom(image.clone());
    game.ram().copy_in(
        crate::ram_map::words::CUR_ENEMY_INDEX,
        &0x140u16.to_le_bytes(),
    );
    {
        let mut runtime = Runtime::new(&mut game);
        use sm_game::actors::access::ActorCollections;
        // Verification resource setup uses the actual palette-copy capability;
        // actors are born ONLY by the actual spawn below, never a seed helper.
        for (row, resource, address) in [
            (9u16, ResourceId(3), 0xb48000),
            (10, ResourceId(4), 0xb49000),
        ] {
            let captured = sm_game::actors::content::ActorLiveRecord {
                image: image.clone(),
                base: lorom_offset(address) as usize,
            };
            let mapped = sm_game::actors::media::palette_resource_for_capture(
                &captured,
                &runtime.session().world,
            );
            assert_eq!(
                mapped,
                Some(resource),
                "actor.initializer.fixture_palette_mapping"
            );
            let colors = core::array::from_fn(|lane| image.read(captured.base + lane));
            runtime.copy_palette_row((row - 8) as u8, colors);
            runtime
                .session_mut()
                .actor_identity
                .record_palette_residency(row, mapped, image.clone());
        }
        assert_eq!(
            runtime
                .session()
                .actor_identity
                .observe(0, false, None)
                .handle,
            None
        );
        assert_eq!(
            runtime
                .session()
                .actor_identity
                .observe(1, false, None)
                .handle,
            None
        );
    }
    game
}

fn assert_preserved(game: &mut Game, observed: &[ActorObservation], unwind: bool) {
    assert!(
        !game.actor_identity_is_with_session(),
        "actor.initializer.after_scope_game_owns_payload"
    );
    let runtime = Runtime::new(game);
    for (ordinal, before) in observed.iter().enumerate() {
        let now =
            runtime
                .session()
                .actor_identity
                .observe(ordinal as u16, true, Some(SpeciesId(7)));
        let label = if unwind && ordinal == 1 {
            "actor.initializer.after_unwind_nested_occurrence"
        } else {
            "actor.initializer.after_scope_occurrence"
        };
        assert_eq!(now.handle, before.handle, "{label}");
        assert!(now.handle.is_some(), "{label}");
        assert_eq!(
            now.birth_order, before.birth_order,
            "actor.initializer.after_scope_birth_order"
        );
        assert_eq!(
            now.appearance, before.appearance,
            "actor.initializer.after_scope_appearance"
        );
        assert_eq!(
            runtime
                .session()
                .actor_identity
                .initializer_fixture_origin(ordinal as u16),
            Some(ActorBirthOrigin::Dynamic {
                template: ActorTemplateId(11 + ordinal as u32),
                part: 0
            }),
            "actor.initializer.after_scope_authored_occurrence"
        );
        let resolved = runtime
            .session()
            .actor_identity
            .resolve(before.handle.unwrap(), &runtime.session().actors);
        assert_eq!(
            resolved.map(|key| key.byte_index()),
            Ok(ordinal as u16 * 64),
            "actor.initializer.after_scope_handle_resolves"
        );
        assert_eq!(
            runtime.session().actors.slots[ordinal].definition,
            0xd000,
            "actor.initializer.after_scope_definition"
        );
    }
}

fn run(mode: Mode) {
    let mut game = game();
    let active = ActiveGuard::begin(mode);
    let result = catch_unwind(AssertUnwindSafe(|| {
        crate::game::actors::population::spawn(
            &mut Runtime::new(&mut game),
            ActorSpawnRequest {
                template: ActorTemplateId(11),
                appearance: Some(AppearanceId(20)),
            },
        )
    }));
    let capture = active.finish(); // Clears TLS for expected and unexpected unwinds.
    let returned = if mode == Mode::NestedUnwind {
        assert!(
            result.is_err(),
            "actor.initializer.deliberate_unwind_reached"
        );
        assert_eq!(
            panic_text(result.unwrap_err()),
            UNWIND,
            "actor.initializer.exact_deliberate_unwind"
        );
        None
    } else {
        // Preserve the first callback's exact F2 assertion, not a coverage panic.
        Some(match result {
            Ok(result) => result,
            Err(error) => std::panic::resume_unwind(error),
        })
    };
    assert_eq!(
        capture.observed.len(),
        if mode == Mode::Single { 1 } else { 2 },
        "actor.initializer.exact_callback_coverage"
    );
    if let Some(returned) = returned {
        assert_eq!(
            returned,
            Ok(capture.observed[0].handle),
            "actor.initializer.outer_return_handle"
        );
        assert_eq!(
            game.cur_enemy_index(),
            0x140,
            "actor.initializer.outer_restores_selector"
        );
    }
    assert_preserved(&mut game, &capture.observed, mode == Mode::NestedUnwind);
}
pub(crate) fn run_checked() {
    run(Mode::Single);
    run(Mode::NestedReturn);
    run(Mode::NestedUnwind);
}
