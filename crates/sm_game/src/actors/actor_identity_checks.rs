//! Verification child of the actual identity module; never a production setter API.
use super::*;
use crate::actors::snapshot::ActorSnapshotPayload;
use crate::room_entry::world::WorldBinding;

fn provenance(part: u16) -> ActorProvenance {
    ActorProvenance {
        authored_species: Some(SpeciesId(7)),
        origin: ActorBirthOrigin::Dynamic {
            template: ActorTemplateId(11),
            part,
        },
    }
}
fn birth(state: &mut ActorIdentityState, actors: &mut Actors, slot: u16) -> InstanceHandle {
    let pending = state
        .prepare_birth(
            slot,
            provenance(slot),
            Some(AppearanceId(20 + u32::from(slot))),
        )
        .unwrap();
    actors.slots[usize::from(slot)].definition = 0xd000;
    state.commit_birth(pending)
}

#[test]
fn discarded_future_handle_rejected_after_same_instance_restore_and_replay() {
    let world = WorldBinding::canonical_legacy();
    let mut state = ActorIdentityState::empty();
    let mut actors = Actors::default();
    let old_a = birth(&mut state, &mut actors, 0);
    let snapshot = ActorSnapshotPayload::capture(&state, &world);
    let actors_a = actors.clone();
    let timeline_a = state.timeline.clone();
    let old_b = birth(&mut state, &mut actors, 1);
    let timeline_b = state.timeline.clone();
    let restored = snapshot.preflight(&state, &world).unwrap();
    actors = actors_a;
    restored.commit(&mut state);
    assert_eq!(
        state.timeline, timeline_a,
        "saved timeline restored exactly"
    );
    let replay_b = birth(&mut state, &mut actors, 1);
    assert_eq!(
        state.timeline, timeline_b,
        "semantic replay matches discarded future"
    );
    assert_eq!(
        state.resolve(old_b, &actors),
        Err(ActorIdentityError::StaleEpoch),
        "discarded_future_handle_rejected"
    );
    assert_eq!(
        state.resolve(replay_b, &actors),
        Ok(ActorKey::from_legacy(64))
    );
    assert_eq!(
        state.resolve(old_a, &actors),
        Err(ActorIdentityError::StaleEpoch)
    );
    assert_ne!(old_b, replay_b, "external handles do not rewind");
}

#[test]
fn compatibility_bytes_and_daff_preserve_occurrence_but_nominal_rebirth_does_not() {
    let mut state = ActorIdentityState::empty();
    let mut actors = Actors::default();
    let handle = birth(&mut state, &mut actors, 0);
    let original = state.observe(0, true, Some(SpeciesId(7)));
    actors.slots[0].definition = 0;
    assert_eq!(
        state.resolve(handle, &actors),
        Err(ActorIdentityError::Vacant)
    );
    let vacant = state.observe(0, false, None);
    assert_eq!(vacant.birth_order, original.birth_order);
    assert_eq!(vacant.appearance, original.appearance);
    actors.slots[0].definition = 0xd000;
    assert_eq!(state.resolve(handle, &actors), Ok(ActorKey::from_legacy(0)));
    actors.slots[0].definition = 0xdaff;
    state.convert_to_placeholder(0);
    assert_eq!(
        state.observe(0, true, Some(SpeciesId(7))),
        original,
        "actor.contract.daff_retains_occurrence"
    );
    state.retire(0);
    assert_eq!(
        state.resolve(handle, &actors),
        Err(ActorIdentityError::Vacant)
    );
    let replacement = birth(&mut state, &mut actors, 0);
    assert_eq!(
        state.resolve(handle, &actors),
        Err(ActorIdentityError::StaleGeneration)
    );
    assert_ne!(replacement, handle);
    assert_eq!(
        state.observe(0, true, Some(SpeciesId(7))).birth_order,
        Some(1)
    );
}

#[test]
fn exceptional_legacy_births_do_not_clamp_or_allocate_nominal_metadata() {
    let state = ActorIdentityState::empty();
    for key in [1, 0x800, 0xe3ea, 0xffff] {
        assert!(
            state
                .prepare_legacy_birth(ActorKey::from_legacy(key), provenance(0), None)
                .unwrap()
                .is_none()
        );
    }
    assert!(
        state
            .prepare_legacy_birth(ActorKey::from_legacy(0x7c0), provenance(31), None)
            .unwrap()
            .is_some()
    );
    assert_eq!(state, ActorIdentityState::empty());
}

#[test]
fn epoch_overflow_preflight_precedes_any_canonical_image_mutation() {
    let mut state = ActorIdentityState::empty();
    state.live_epoch = u64::MAX; // Private verification fixture, not a gameplay API.
    let before = state.clone();
    let mut ram: Box<[u8; 131072]> = vec![0x5a; 131072].into_boxed_slice().try_into().unwrap();
    let result = (|| {
        let token = state.prepare_canonical_transition()?;
        ram.fill(0);
        state.commit_canonical_transition(token, &|at| ram[at]);
        Ok::<_, ActorIdentityError>(())
    })();
    assert_eq!(result, Err(ActorIdentityError::EpochOverflow));
    assert_eq!(state, before);
    assert!(
        ram.iter().all(|&byte| byte == 0x5a),
        "epoch rejection leaves all RAM unchanged"
    );
}

#[test]
fn generation_and_order_overflow_leave_birth_unpublished() {
    let mut state = ActorIdentityState::empty();
    state.timeline.slots[0].generation = u64::MAX;
    let before = state.clone();
    assert!(matches!(
        state.prepare_birth(0, provenance(0), None),
        Err(ActorIdentityError::GenerationOverflow)
    ));
    assert_eq!(state, before);
    state.timeline.slots[0].generation = 0;
    state.timeline.next_birth_order = u64::MAX;
    let before = state.clone();
    assert!(matches!(
        state.prepare_birth(0, provenance(0), None),
        Err(ActorIdentityError::OrderOverflow)
    ));
    assert_eq!(state, before);
}

#[test]
fn snapshot_binding_and_epoch_rejections_preserve_live_identity() {
    let world = WorldBinding::canonical_legacy();
    let other = WorldBinding::canonical_legacy();
    let mut state = ActorIdentityState::empty();
    let snapshot = ActorSnapshotPayload::capture(&state, &world);
    let before = state.clone();
    assert!(matches!(
        snapshot.preflight(&state, &other),
        Err(ActorIdentityError::BindingMismatch)
    ));
    assert_eq!(state, before);
    state.live_epoch = u64::MAX;
    let before = state.clone();
    assert!(matches!(
        snapshot.preflight(&state, &world),
        Err(ActorIdentityError::EpochOverflow)
    ));
    assert_eq!(state, before);
}

#[test]
fn canonical_transition_uses_final_bytes_and_clears_authored_lineage() {
    let mut state = ActorIdentityState::empty();
    let mut actors = Actors::default();
    let old = birth(&mut state, &mut actors, 0);
    let token = state.prepare_canonical_transition().unwrap();
    let mut ram: Box<[u8; 131072]> = vec![0; 131072].into_boxed_slice().try_into().unwrap();
    ram[0xf78 + 31 * 64..0xf7a + 31 * 64].copy_from_slice(&0xdaffu16.to_le_bytes());
    state.commit_canonical_transition(token, &|at| ram[at]);
    assert_eq!(
        state.resolve(old, &actors),
        Err(ActorIdentityError::StaleEpoch)
    );
    assert!(state.timeline.slots[0].occupant.is_none());
    let observed = state.observe(31, true, None);
    assert_eq!(observed.birth_order, Some(0));
    assert_eq!(observed.authored_species, None);
    assert_eq!(observed.appearance, None);
    assert_eq!(state.timeline.next_birth_order, 1);
}
