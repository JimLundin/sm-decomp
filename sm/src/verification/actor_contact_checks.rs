//! Actual Runtime contact assertions for stage aliases and appearance-independent geometry.
#![forbid(unsafe_code)]
use crate::{
    game::{room_entry::world::*, session_runtime::Runtime},
    game_state::Game,
    rom::{RomImage, lorom_offset},
    types::EnemyDef,
};
use core::mem::offset_of;
use std::{collections::BTreeMap, sync::Arc};
use {
    crate::game::actors::identity::{ActorBirthOrigin, ActorProvenance},
    sm_game::actors::{
        access::{AccessMode, ActorAccess, ActorCollections, ActorKey},
        content::*,
        media::*,
    },
};

fn native(image: &RomImage, address: u32, offset: usize, value: u16) {
    let at = lorom_offset(address) as usize + offset;
    for (lane, byte) in value.to_ne_bytes().into_iter().enumerate() {
        image.write(at + lane, byte);
    }
}
pub(crate) fn run_checked() {
    direct_touch_body_publishes_stage_before_aliased_map();
    same_behavior_pose_two_appearances_have_identical_actual_shot_contact();
}
fn direct_touch_body_publishes_stage_before_aliased_map() {
    let image = RomImage::zeroed(0x200000);
    native(&image, 0xa0d000, offset_of!(EnemyDef, touch_ai), 0x804c);
    let mut game = Game::new();
    game.register_native_rom(image.clone());
    game.register_rom(image);
    let mut runtime = Runtime::new(&mut game);
    let key = ActorKey::from_legacy(0xe3ea);
    runtime.edit_actor(key, AccessMode::Native, |actor| {
        actor.definition = 0xd000;
        actor.bank = 0xa0;
        actor.spritemap = 0;
    });
    runtime.session_mut().actors.controls.selected = 0xe3ea;
    runtime.session_mut().room.enemy_processing_stage = 0;
    runtime.session_mut().player.contact.contact_attack = 1;
    runtime.session_mut().player.invincibility = 77;
    assert_eq!(
        runtime.resolve_actor(key, AccessMode::Native).spritemap,
        0,
        "actor.contact.alias_fixture_map_starts_zero"
    );
    // Existing thin verification delegate executes the actual private touch_body.
    crate::game::actors::contacts::movement_verification_body(&mut runtime);
    assert_eq!(
        runtime.session().player.invincibility,
        0,
        "actor.contact.f7_stage_before_aliased_map"
    );
    assert_eq!(
        runtime.session().room.enemy_processing_stage,
        9,
        "actor.contact.alias_stage_is_nine"
    );
}
fn collision_world() -> Arc<WorldBinding> {
    let base = sm_game::actors::fixture::binding(false, AppearanceId(20), false);
    let definition = base.definition().unwrap().clone();
    let keys = CompatibilityKeys {
        starts: BTreeMap::new(),
        rooms: BTreeMap::new(),
        states: BTreeMap::new(),
        exits: BTreeMap::new(),
        door_lists: BTreeMap::new(),
        resources: definition
            .resources
            .keys()
            .map(|&id| (id, base.resource_key(id)))
            .collect(),
    };
    let actors = base.actors().definitions().unwrap().clone();
    let mut actor_keys = base.actors().keys().unwrap().clone();
    // Actual retained no-op hitbox shot dispatch, with no species memory dereference.
    actor_keys.callbacks.insert(ActorCallbackId(61), 0xa2804c);
    let mut media = base.actor_media().definitions().unwrap().clone();
    let FrameDefinition::Sprites(pieces) = media.frames.get_mut(&FrameId(31)).unwrap() else {
        unreachable!("shared fixture sprite frame")
    };
    pieces[0].offset.x = 64;
    WorldBinding::prepared_with_actors(definition, keys, actors, actor_keys, media).unwrap()
}
fn stage_materials(runtime: &mut Runtime<'_>, image: &RomImage) {
    for (row, resource, address) in [
        (9u16, ResourceId(3), 0xb48000u32),
        (10, ResourceId(4), 0xb49000),
    ] {
        let capture = ActorLiveRecord {
            image: image.clone(),
            base: lorom_offset(address) as usize,
        };
        assert_eq!(
            sm_game::actors::media::palette_resource_for_capture(
                &capture,
                &runtime.session().world
            ),
            Some(resource)
        );
        let colors = core::array::from_fn(|lane| image.read(capture.base + lane));
        runtime
            .session_mut()
            .actor_identity
            .record_palette_residency(row, None, image.clone());
        runtime.copy_palette_row((row - 8) as u8, colors);
        runtime
            .session_mut()
            .actor_identity
            .record_palette_residency(row, Some(resource), image.clone());
    }
}
fn same_behavior_pose_two_appearances_have_identical_actual_shot_contact() {
    let world = collision_world();
    let image = RomImage::zeroed(0x200000);
    native(&image, 0xa0d000, offset_of!(EnemyDef, shot_ai), 0x8040);
    native(&image, 0xa0d000, offset_of!(EnemyDef, touch_ai), 0x804c);
    // The imported part/hitbox pointers stay bounded even though gameplay geometry is prepared.
    native(&image, 0xa0c000, 0, 1);
    native(&image, 0xa0c000, 2, 0);
    native(&image, 0xa0c000, 4, 0);
    native(&image, 0xa0c000, 6, 0xe000);
    native(&image, 0xa0c000, 8, 0xe100);
    native(&image, 0xa0e100, 0, 1);
    let mut game = Game::with_world(world.clone());
    game.register_native_rom(image.clone());
    game.register_rom(image.clone());
    let mut runtime = Runtime::new(&mut game);
    stage_materials(&mut runtime, &image);
    for (slot, appearance) in [(0u16, AppearanceId(20)), (1, AppearanceId(21))] {
        assert_eq!(
            world.actor_media().validate_appearance_residency(
                appearance,
                runtime.session().actor_identity.palette_residency(),
                &image
            ),
            Ok(())
        );
        // Initialize authored instances using the same checked birth publication as population.
        // This fixture tests contacts, not allocation or initializer behavior.
        let birth = runtime
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
                Some(appearance),
            )
            .unwrap();
        runtime.edit_actor(
            ActorKey::from_legacy(slot * 64),
            AccessMode::Native,
            |actor| {
                actor.definition = 0xd000;
                actor.bank = 0xa0;
                actor.spritemap = 0xc000;
                actor.extra_properties = 4;
                actor.x.whole = 100;
                actor.y.whole = 100;
                actor.radius.x = 8;
                actor.radius.y = 12;
            },
        );
        runtime.session_mut().actor_identity.commit_birth(birth);
    }
    for (slot, appearance) in [(0u16, AppearanceId(20)), (1, AppearanceId(21))] {
        runtime.session_mut().actors.controls.selected = slot * 64;
        runtime.session_mut().room.collision_detection_index = 0x55;
        runtime.session_mut().projectiles.count = 1;
        let shot = &mut runtime.session_mut().projectiles.shots[0];
        shot.kind = 0x100;
        shot.direction = 0;
        shot.x = 100;
        shot.y = 100;
        shot.x_radius = 2;
        shot.y_radius = 2;
        let observed = runtime
            .session()
            .actor_identity
            .observe(slot, true, Some(SpeciesId(7)));
        assert_eq!(observed.authored_species, Some(SpeciesId(7)));
        assert_eq!(observed.appearance, Some(appearance));
        let pose = runtime.capture_actor_pose(ActorKey::from_legacy(slot * 64));
        assert_eq!(pose.position, Some(PoseId(9)));
        // Full actual ordered shot/bomb/touch producer, including actual no-op hitbox callback.
        crate::game::actors::contacts::process_contacts(&mut runtime);
        assert_eq!(
            (
                runtime.session().projectiles.shots[0].direction & 0x10,
                runtime.session().room.collision_detection_index
            ),
            (0x10, 0),
            "actor.contact.f8_same_pose_collision_ignores_appearance slot={slot}"
        );
    }
}
