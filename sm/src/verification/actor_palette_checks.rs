//! Per-room palette residency and dynamic selection contracts on actual owner APIs.
use super::{content::*, identity::*, media::*, snapshot::ActorSnapshotPayload};
use crate::{game::room_entry::world::ResourceId, rom::RomImage};
use sm_game::actors::state::Actors;

fn born(identity: &mut ActorIdentityState, actors: &mut Actors) -> InstanceHandle {
    let pending = identity
        .prepare_birth(
            0,
            ActorProvenance {
                authored_species: Some(SpeciesId(7)),
                origin: ActorBirthOrigin::Dynamic {
                    template: ActorTemplateId(11),
                    part: 0,
                },
            },
            Some(AppearanceId(20)),
        )
        .unwrap();
    actors.slots[0].definition = 0xd000;
    identity.commit_birth(pending)
}

#[test]
fn unrelated_rooms_can_reuse_a_row_but_only_its_current_resource_is_resident() {
    let world = super::fixture::binding(false, AppearanceId(20), true);
    let image = RomImage::zeroed(0x200000);
    let mut identity = ActorIdentityState::empty();
    identity.record_palette_residency(9, Some(ResourceId(3)), image.clone());
    identity.record_palette_residency(0, Some(ResourceId(4)), image.clone());
    let media = world.actor_media();
    assert_eq!(
        media.validate_appearance_residency(AppearanceId(20), identity.palette_residency(), &image),
        Ok(())
    );
    assert_eq!(
        media.validate_appearance_residency(AppearanceId(21), identity.palette_residency(), &image),
        Err(ActorContentError::PaletteNotResident(MaterialId(51)))
    );
    identity.clear_actor_palette_residency();
    assert_eq!(
        media.validate_material_residency(MaterialId(52), identity.palette_residency(), &image),
        Ok(()),
        "actor.palette.actor_clear_preserves_bg_receipts"
    );
    assert_eq!(
        media.validate_material_residency(MaterialId(50), identity.palette_residency(), &image),
        Err(ActorContentError::PaletteNotResident(MaterialId(50))),
        "actor.palette.actor_clear_revokes_obj_receipts"
    );
    identity.record_palette_residency(9, Some(ResourceId(4)), image.clone());
    assert_eq!(
        media.validate_appearance_residency(AppearanceId(21), identity.palette_residency(), &image),
        Ok(())
    );
    assert_eq!(
        media.validate_appearance_residency(AppearanceId(20), identity.palette_residency(), &image),
        Err(ActorContentError::PaletteNotResident(MaterialId(50)))
    );
}

#[test]
fn dynamic_appearance_rejects_missing_residency_and_conflicts_before_publication() {
    let world = super::fixture::binding(false, AppearanceId(20), true);
    let image = RomImage::zeroed(0x200000);
    let mut identity = ActorIdentityState::empty();
    let mut actors = Actors::default();
    let handle = born(&mut identity, &mut actors);
    identity.record_palette_residency(9, Some(ResourceId(3)), image.clone());
    let before = identity.clone();
    let before_actors = actors.clone();
    assert_eq!(
        identity.set_appearance(handle, AppearanceId(21), &actors, &world, &image),
        Err(ActorIdentityError::Content(
            ActorContentError::PaletteNotResident(MaterialId(51))
        )),
        "actor.palette.selection_requires_residency"
    );
    assert_eq!(
        identity, before,
        "actor.palette.failed_selection_does_not_publish"
    );
    assert_eq!(
        identity.set_appearance(handle, AppearanceId(22), &actors, &world, &image),
        Err(ActorIdentityError::Content(
            ActorContentError::ConflictingPaletteRow(9)
        ))
    );
    assert_eq!(
        identity, before,
        "actor.palette.conflicting_selection_does_not_publish"
    );
    assert_eq!(actors, before_actors);
    identity.record_palette_residency(9, Some(ResourceId(4)), image.clone());
    identity
        .set_appearance(handle, AppearanceId(21), &actors, &world, &image)
        .unwrap();
    let selected = identity.observe(0, true, Some(SpeciesId(7)));
    assert_eq!(selected.appearance, Some(AppearanceId(21)));
    assert_eq!(selected.handle, Some(handle));
    assert_eq!(selected.birth_order, Some(0));
}

#[test]
fn unknown_overwrite_revokes_and_rebind_rejects_equal_bytes_in_another_allocation() {
    let world = super::fixture::binding(false, AppearanceId(20), false);
    let image = RomImage::zeroed(0x200000);
    let replacement = RomImage::zeroed(0x200000);
    let mut identity = ActorIdentityState::empty();
    identity.record_palette_residency(9, Some(ResourceId(3)), image.clone());
    let check = |state: &ActorIdentityState, image: &RomImage| {
        world.actor_media().validate_material_residency(
            MaterialId(50),
            state.palette_residency(),
            image,
        )
    };
    assert_eq!(check(&identity, &image), Ok(()));
    image.write(crate::rom::lorom_offset(0xb48000) as usize, 0x5a); // A live edit keeps allocation identity.
    assert_eq!(
        check(&identity, &image),
        Ok(()),
        "actor.palette.live_allocation_edit_keeps_receipt"
    );
    replacement.write(crate::rom::lorom_offset(0xb48000) as usize, 0x5a); // Equal bytes are not the same source allocation.
    assert_eq!(
        check(&identity, &replacement),
        Err(ActorContentError::PaletteNotResident(MaterialId(50))),
        "actor.palette.rebound_allocation_rejects_old_receipt"
    );
    identity.record_palette_residency(9, None, image.clone());
    assert_eq!(
        check(&identity, &image),
        Err(ActorContentError::PaletteNotResident(MaterialId(50))),
        "actor.palette.unknown_overwrite_revokes_receipt"
    );
}

#[test]
fn paired_snapshot_restores_receipts_but_canonical_adoption_clears_them() {
    let world = super::fixture::binding(false, AppearanceId(20), false);
    let image = RomImage::zeroed(0x200000);
    let rebound = RomImage::zeroed(0x200000);
    let mut identity = ActorIdentityState::empty();
    identity.record_palette_residency(9, Some(ResourceId(3)), image.clone());
    let snapshot = ActorSnapshotPayload::capture(&identity, &world);
    identity.clear_palette_residency();
    snapshot
        .preflight(&identity, &world)
        .unwrap()
        .commit(&mut identity);
    assert_eq!(
        world.actor_media().validate_material_residency(
            MaterialId(50),
            identity.palette_residency(),
            &image
        ),
        Ok(()),
        "actor.palette.paired_restore_keeps_receipt"
    );
    assert_eq!(
        world.actor_media().validate_material_residency(
            MaterialId(50),
            identity.palette_residency(),
            &rebound
        ),
        Err(ActorContentError::PaletteNotResident(MaterialId(50)))
    );
    let ram: Box<[u8; 131072]> = vec![0; 131072].into_boxed_slice().try_into().unwrap();
    let transition = identity.prepare_canonical_transition().unwrap();
    identity.commit_canonical_transition(transition, &|at| ram[at]);
    assert_eq!(
        world.actor_media().validate_material_residency(
            MaterialId(50),
            identity.palette_residency(),
            &image
        ),
        Err(ActorContentError::PaletteNotResident(MaterialId(50))),
        "actor.palette.canonical_adoption_clears_receipts"
    );
}
