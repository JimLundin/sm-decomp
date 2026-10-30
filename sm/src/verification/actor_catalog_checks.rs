//! Actual constructor/scene contracts; these are not renderer or contact-route tests.
use super::{identity::*, scene::*};
use sm_game::{room_entry::world::*, session::Point};
use sm_game::actors::state::Actors;
use std::sync::Arc;

fn fixture(swap_frames: bool, default_appearance: AppearanceId) -> Arc<WorldBinding> {
    super::fixture::binding(swap_frames, default_appearance, false)
}
fn resolved(world: &WorldBinding, appearance: AppearanceId) -> ResolvedActorVisual {
    let pose = &world.actors().definitions().unwrap().poses[&PoseId(9)];
    world
        .actor_media()
        .resolve(appearance, PoseId(9), pose)
        .unwrap()
}

#[test]
fn media_only_mapping_change_preserves_gameplay_revision_and_collision_definition() {
    let first = fixture(false, AppearanceId(20));
    let changed = fixture(true, AppearanceId(20));
    assert_eq!(
        first.actor_gameplay_revision(),
        changed.actor_gameplay_revision(),
        "actor.contract.media_does_not_change_gameplay_revision"
    );
    assert_ne!(
        first.actor_media_revision(),
        changed.actor_media_revision(),
        "actor.contract.frame_mapping_changes_media_revision"
    );
    let a = first.actors().definitions().unwrap();
    let b = changed.actors().definitions().unwrap();
    assert_eq!(a.poses, b.poses);
    assert_eq!(
        a.collisions, b.collisions,
        "behavior collision definitions remain independent"
    );
    assert_ne!(
        resolved(&first, AppearanceId(20)).form,
        resolved(&changed, AppearanceId(20)).form
    );
}

#[test]
fn default_appearance_changes_media_revision_only() {
    let first = fixture(false, AppearanceId(20));
    let changed = fixture(false, AppearanceId(21));
    assert_eq!(
        first.actor_gameplay_revision(),
        changed.actor_gameplay_revision()
    );
    assert_ne!(
        first.actor_media_revision(),
        changed.actor_media_revision(),
        "actor.contract.default_appearance_changes_media_revision"
    );
}

#[test]
fn same_species_instances_resolve_distinct_frames_without_mutating_behavior() {
    let world = fixture(false, AppearanceId(20));
    let mut identity = ActorIdentityState::empty();
    let mut actors = Actors::default();
    let image = crate::rom::RomImage::zeroed(0x200000);
    identity.record_palette_residency(9, Some(ResourceId(3)), image.clone());
    identity.record_palette_residency(10, Some(ResourceId(4)), image.clone());
    for slot in 0..2 {
        let pending = identity
            .prepare_birth(
                slot,
                ActorProvenance {
                    authored_species: Some(SpeciesId(7)),
                    origin: ActorBirthOrigin::Dynamic {
                        template: ActorTemplateId(3),
                        part: slot,
                    },
                },
                None,
            )
            .unwrap();
        actors.slots[usize::from(slot)].definition = 0xd000;
        actors.slots[usize::from(slot)].radius = Point { x: 8, y: 12 };
        let handle = identity.commit_birth(pending);
        identity
            .set_appearance(
                handle,
                AppearanceId(20 + u32::from(slot)),
                &actors,
                &world,
                &image,
            )
            .unwrap();
    }
    let observations = [
        identity.observe(0, true, Some(SpeciesId(7))),
        identity.observe(1, true, Some(SpeciesId(7))),
    ];
    assert_eq!(
        observations[0].effective_species,
        observations[1].effective_species
    );
    assert_eq!(actors.slots[0].radius, actors.slots[1].radius);
    let before = actors.clone();
    let first = resolved(&world, observations[0].appearance.unwrap());
    let second = resolved(&world, observations[1].appearance.unwrap());
    assert_ne!(
        first.form, second.form,
        "actor.contract.instances_select_distinct_media"
    );
    assert_eq!(
        actors, before,
        "scene resolution cannot mutate actor collision state"
    );
}

#[test]
fn configured_unknown_pose_is_explicitly_unresolved() {
    let world = fixture(false, AppearanceId(20));
    let selected = select_part_visual(
        &world,
        Some(AppearanceId(21)),
        None,
        0,
        0xa0,
        0x9123,
        ActorDrawMode::Extended,
    )
    .unwrap();
    match selected {
        ActorVisualSelection::Unresolved {
            appearance,
            reason: UnresolvedActorVisual::UnknownPose,
            compatibility,
        } => {
            assert_eq!(appearance, AppearanceId(21));
            assert_eq!(compatibility.bank, 0xa0);
            assert_eq!(compatibility.address, 0x9123);
        }
        _ => panic!("actor.contract.configured_unknown_pose_is_unresolved"),
    }
}
