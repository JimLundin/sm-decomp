//! Movies cannot author malformed identity payloads or inspect detached handle epochs.
use super::*;
use crate::actors::snapshot::CheckpointImages;
use crate::{rom::RomImage, room_entry::world::WorldBinding};
use crate::{scene::component::DisplayStartMetadata, session::Session};
use std::sync::Arc;

/// Movies cannot observe the installed live epoch after a gated binary restore.
pub fn assert_installed_epoch(prior: &ActorIdentityState, current: &ActorIdentityState) {
    assert_eq!(current.live_epoch, prior.live_epoch.checked_add(1).unwrap());
    assert!(!current.poisoned);
}

/// No movie supplies independent lineage counters or a detached epoch replacement.
pub fn assert_owned_checkpoint(session: &Session, loaded: &RomImage, native: &RomImage) {
    // Preserve the original canonical-legacy binding witness independently of prepared rooms.
    {
        let mut legacy = session.clone();
        legacy.world = WorldBinding::canonical_legacy();
        let session = &legacy;
        assert!(session.world.definition().is_none());
        assert_checkpoint_binding(session, loaded, native);
    }
    assert!(session.world.definition().is_some());
    assert_checkpoint_binding(session, loaded, native);
}

fn assert_checkpoint_binding(session: &Session, loaded: &RomImage, native: &RomImage) {
    let metadata = session.control.metadata().unwrap();
    let current = &session.actor_identity;
    let original = current.clone();
    let loaded_before: Vec<_> = (0..loaded.len()).map(|at| loaded.read(at)).collect();
    let native_before: Vec<_> = (0..native.len()).map(|at| native.read(at)).collect();
    assert!(!loaded.same_allocation(native));
    assert_eq!(
        loaded_before, native_before,
        "equal content, distinct fixture allocations"
    );
    let bytes = current
        .encode_owned_checkpoint(
            &session.actors,
            &session.world,
            metadata,
            CheckpointImages {
                loaded: Some(loaded),
                native: Some(native),
                sources: &[],
            },
        )
        .unwrap();
    let decoded = ActorIdentityState::decode_owned_checkpoint(
        &bytes,
        current,
        &session.actors,
        &session.world,
        metadata,
        CheckpointImages {
            loaded: Some(loaded),
            native: Some(native),
            sources: &[],
        },
    )
    .unwrap();
    assert_eq!(decoded.timeline, current.timeline);
    assert_eq!(decoded.palette_residency, current.palette_residency);
    assert_eq!(decoded.live_epoch, current.live_epoch + 1);
    assert_eq!(
        decoded
            .encode_owned_checkpoint(
                &session.actors,
                &session.world,
                metadata,
                CheckpointImages {
                    loaded: Some(loaded),
                    native: Some(native),
                    sources: &[]
                }
            )
            .unwrap(),
        bytes
    );
    // Both canonical-legacy and prepared-room lifecycles retain legacy actor/palette catalogs.
    assert!(session.world.actors().definitions().is_none());
    assert!(session.world.actor_media().definitions().is_none());
    assert_eq!(current.palette_residency.checkpoint_rows().count(), 0);
    prepared_checkpoint_cases::run(session, loaded, native);
    authored_timeline(session, loaded, native);
    authority_failures(session, loaded, native);
    assert_eq!(current, &original);
    assert_eq!(
        (0..loaded.len())
            .map(|at| loaded.read(at))
            .collect::<Vec<_>>(),
        loaded_before
    );
    assert_eq!(
        (0..native.len())
            .map(|at| native.read(at))
            .collect::<Vec<_>>(),
        native_before
    );
}

/// No movie exposes vacant generations, retained raw-zero occurrences or codec tags.
fn authored_timeline(session: &Session, loaded: &RomImage, native: &RomImage) {
    let metadata = session.control.metadata().unwrap();
    let mut expected = session.actor_identity.clone();
    let mut actors = session.actors.clone();
    assert!(
        expected
            .timeline
            .slots
            .iter()
            .all(|slot| slot.occupant.is_none())
    );
    expected.timeline.next_birth_order = 19;
    expected.timeline.slots[0] = SlotIdentity {
        generation: 7,
        occupant: Some(ActorOccupant {
            birth_order: 11,
            provenance: ActorProvenance {
                authored_species: None,
                origin: ActorBirthOrigin::CanonicalImport,
            },
            appearance: None,
        }),
    };
    expected.timeline.slots[31] = SlotIdentity {
        generation: 23,
        occupant: None,
    };
    actors.slots[0].definition = 0x1234;
    actors.slots[1].definition = 0x2345; // Explicit compatibility-live tag2 without an occurrence.
    let old = expected.observe(0, true, None).handle.unwrap();
    let bytes = expected
        .encode_owned_checkpoint(
            &actors,
            &session.world,
            metadata,
            CheckpointImages {
                loaded: Some(loaded),
                native: Some(native),
                sources: &[],
            },
        )
        .unwrap();
    let decoded = ActorIdentityState::decode_owned_checkpoint(
        &bytes,
        &expected,
        &actors,
        &session.world,
        metadata,
        CheckpointImages {
            loaded: Some(loaded),
            native: Some(native),
            sources: &[],
        },
    )
    .unwrap();
    check_authored(
        &decoded,
        &expected,
        &actors,
        &session.world,
        metadata,
        loaded,
        native,
        &bytes,
        old,
    );
    // Tag1 intentionally retains an occurrence even when the numeric definition becomes zero.
    let mut zero = actors.clone();
    zero.slots[0].definition = 0;
    let retained = ActorIdentityState::decode_owned_checkpoint(
        &bytes,
        &expected,
        &zero,
        &session.world,
        metadata,
        CheckpointImages {
            loaded: Some(loaded),
            native: Some(native),
            sources: &[],
        },
    )
    .unwrap();
    assert_eq!(retained.timeline, expected.timeline);
    malformed_payloads(
        &expected,
        &actors,
        &session.world,
        metadata,
        loaded,
        native,
        &bytes,
    );
}

/// No movie can submit independently malformed/trailing metadata or collapse source aliases.
fn malformed_payloads(
    current: &ActorIdentityState,
    actors: &Actors,
    world: &Arc<WorldBinding>,
    metadata: &DisplayStartMetadata,
    loaded: &RomImage,
    native: &RomImage,
    bytes: &[u8],
) {
    let before = current.clone();
    let reject = |payload: &[u8], occupancy: &Actors, selected_native: &RomImage, reason: &str| {
        expect_rejection(
            current,
            occupancy,
            world,
            metadata,
            loaded,
            selected_native,
            payload,
            reason,
        );
        assert_eq!(current, &before);
    };
    // Offsets come from the independently specified schema and supplied source lengths.
    // This fixture has no actor revisions/roles and two distinct allocations.
    let body = 17 + crate::checkpoint_binding::observation_binding(metadata).len();
    let timeline = body + 46 + 16 + loaded.len() + native.len();
    assert_eq!(&bytes[..9], b"ACTOROWN1");
    assert_eq!(&bytes[body..body + 8], b"ACTORCP3");
    assert_eq!(&bytes[timeline..timeline + 8], &19u64.to_le_bytes());
    assert_eq!(&bytes[timeline + 8..timeline + 16], &7u64.to_le_bytes());
    assert_eq!(bytes[timeline + 16], 1);
    assert_eq!(&bytes[timeline + 17..timeline + 25], &11u64.to_le_bytes());
    let mut bad = bytes.to_vec();
    bad[0] = 0;
    reject(
        &bad,
        actors,
        native,
        "unsupported actor owned checkpoint schema",
    );
    let mut bad = bytes.to_vec();
    bad[17] ^= 1;
    reject(
        &bad,
        actors,
        native,
        "actor owned checkpoint source identity or Reference changed",
    );
    let mut bad = bytes.to_vec();
    bad[body + 8] = 1;
    reject(
        &bad,
        actors,
        native,
        "actor checkpoint world revision differs",
    );
    let mut bad = bytes.to_vec();
    bad[timeline + 16] = 3;
    reject(
        &bad,
        actors,
        native,
        "invalid actor checkpoint occupant tag",
    );
    let mut bad = bytes.to_vec();
    bad[timeline + 8..timeline + 16].copy_from_slice(&0u64.to_le_bytes());
    reject(
        &bad,
        actors,
        native,
        "invalid actor checkpoint generation or birth order",
    );
    let mut bad = bytes.to_vec();
    bad[timeline + 17..timeline + 25].copy_from_slice(&19u64.to_le_bytes());
    reject(
        &bad,
        actors,
        native,
        "invalid actor checkpoint generation or birth order",
    );
    let mut absent = actors.clone();
    absent.slots[1].definition = 0;
    reject(
        bytes,
        &absent,
        native,
        "invalid actor checkpoint occupant tag",
    );
    reject(
        &bytes[..bytes.len() - 1],
        actors,
        native,
        "truncated actor checkpoint",
    );
    let mut bad = bytes.to_vec();
    bad.push(0x5a);
    reject(&bad, actors, native, "extra actor checkpoint metadata");
    allocation_corruption(current, actors, world, metadata, loaded, native, bytes);
}

/// No movie authors epoch overflow, poisoned capture or forbidden authored lineage.
fn authority_failures(session: &Session, loaded: &RomImage, native: &RomImage) {
    let metadata = session.control.metadata().unwrap();
    let actual = &session.actor_identity;
    let bytes = actual
        .encode_owned_checkpoint(
            &session.actors,
            &session.world,
            metadata,
            CheckpointImages {
                loaded: Some(loaded),
                native: Some(native),
                sources: &[],
            },
        )
        .unwrap();
    let mut overflow = actual.clone();
    overflow.live_epoch = u64::MAX;
    let before = overflow.clone();
    assert_eq!(
        ActorIdentityState::decode_owned_checkpoint(
            &[],
            &overflow,
            &session.actors,
            &session.world,
            metadata,
            CheckpointImages {
                loaded: Some(loaded),
                native: Some(native),
                sources: &[]
            }
        )
        .err()
        .as_deref(),
        Some("EpochOverflow")
    );
    assert_eq!(overflow, before);
    let mut poisoned = actual.clone();
    poisoned.retain_poison();
    let before = poisoned.clone();
    assert_eq!(
        poisoned
            .encode_owned_checkpoint(
                &session.actors,
                &session.world,
                metadata,
                CheckpointImages {
                    loaded: Some(loaded),
                    native: Some(native),
                    sources: &[]
                }
            )
            .err()
            .as_deref(),
        Some("poisoned actors cannot produce a checkpoint")
    );
    assert_eq!(poisoned, before);
    // Recovery is preflighted, detached and leaves the current poisoned owner poisoned.
    let recovered = ActorIdentityState::decode_owned_checkpoint(
        &bytes,
        &poisoned,
        &session.actors,
        &session.world,
        metadata,
        CheckpointImages {
            loaded: Some(loaded),
            native: Some(native),
            sources: &[],
        },
    )
    .unwrap();
    assert!(!recovered.is_poisoned());
    assert_eq!(recovered.live_epoch, poisoned.live_epoch + 1);
    assert_eq!(poisoned, before);
    forbidden_lineage(session, loaded, native);
}

/// No movie observes independently authored lineage values or old/fresh handles.
fn check_authored(
    decoded: &ActorIdentityState,
    expected: &ActorIdentityState,
    actors: &Actors,
    world: &Arc<WorldBinding>,
    metadata: &DisplayStartMetadata,
    loaded: &RomImage,
    native: &RomImage,
    bytes: &[u8],
    old: InstanceHandle,
) {
    assert_eq!(decoded.timeline.next_birth_order, 19);
    assert_eq!(decoded.timeline.slots[0].generation, 7);
    assert_eq!(
        decoded.timeline.slots[0]
            .occupant
            .as_ref()
            .unwrap()
            .birth_order,
        11
    );
    assert_eq!(
        decoded.timeline.slots[0]
            .occupant
            .as_ref()
            .unwrap()
            .provenance,
        ActorProvenance {
            authored_species: None,
            origin: ActorBirthOrigin::CanonicalImport
        }
    );
    assert_eq!(
        decoded.timeline.slots[0]
            .occupant
            .as_ref()
            .unwrap()
            .appearance,
        None
    );
    assert_eq!(
        decoded.timeline.slots[31],
        SlotIdentity {
            generation: 23,
            occupant: None
        }
    );
    assert_eq!(decoded.timeline, expected.timeline);
    assert_eq!(
        decoded.resolve(old, &actors),
        Err(ActorIdentityError::StaleEpoch)
    );
    let fresh = decoded.observe(0, true, None).handle.unwrap();
    assert_eq!(
        decoded.resolve(fresh, &actors),
        Ok(ActorKey::from_legacy(0))
    );
    assert_eq!(
        decoded
            .encode_owned_checkpoint(
                &actors,
                world,
                metadata,
                CheckpointImages {
                    loaded: Some(loaded),
                    native: Some(native),
                    sources: &[]
                }
            )
            .unwrap(),
        bytes.to_vec()
    );
}

/// No movie supplies forbidden authored lineage or appearance without a catalog.
fn forbidden_lineage(session: &Session, loaded: &RomImage, native: &RomImage) {
    let metadata = session.control.metadata().unwrap();
    let actual = &session.actor_identity;
    let mut invalid = actual.clone();
    invalid.timeline.next_birth_order = 4;
    invalid.timeline.slots[0] = SlotIdentity {
        generation: 3,
        occupant: Some(ActorOccupant {
            birth_order: 2,
            provenance: ActorProvenance {
                authored_species: Some(SpeciesId(7)),
                origin: ActorBirthOrigin::Dynamic {
                    template: ActorTemplateId(11),
                    part: 2,
                },
            },
            appearance: None,
        }),
    };
    assert_eq!(
        invalid
            .encode_owned_checkpoint(
                &session.actors,
                &session.world,
                metadata,
                CheckpointImages {
                    loaded: Some(loaded),
                    native: Some(native),
                    sources: &[]
                }
            )
            .err()
            .as_deref(),
        Some("actor checkpoint has authored lineage without a catalog")
    );
    let occupant = invalid.timeline.slots[0].occupant.as_mut().unwrap();
    occupant.provenance = ActorProvenance {
        authored_species: None,
        origin: ActorBirthOrigin::CanonicalImport,
    };
    occupant.appearance = Some(AppearanceId(13));
    assert_eq!(
        invalid
            .encode_owned_checkpoint(
                &session.actors,
                &session.world,
                metadata,
                CheckpointImages {
                    loaded: Some(loaded),
                    native: Some(native),
                    sources: &[]
                }
            )
            .err()
            .as_deref(),
        Some("actor checkpoint has an appearance without a catalog")
    );
}

/// No movie can prove an exact malformed-payload error without touching current authority.
fn expect_rejection(
    current: &ActorIdentityState,
    actors: &Actors,
    world: &Arc<WorldBinding>,
    metadata: &DisplayStartMetadata,
    loaded: &RomImage,
    native: &RomImage,
    bytes: &[u8],
    reason: &str,
) {
    assert_eq!(
        ActorIdentityState::decode_owned_checkpoint(
            bytes,
            current,
            actors,
            world,
            metadata,
            CheckpointImages {
                loaded: Some(loaded),
                native: Some(native),
                sources: &[]
            }
        )
        .err()
        .as_deref(),
        Some(reason)
    );
}

/// No movie can independently collapse two equal-byte allocations or corrupt saved source content.
fn allocation_corruption(
    current: &ActorIdentityState,
    actors: &Actors,
    world: &Arc<WorldBinding>,
    metadata: &DisplayStartMetadata,
    loaded: &RomImage,
    native: &RomImage,
    bytes: &[u8],
) {
    let body = 17 + crate::checkpoint_binding::observation_binding(metadata).len();
    let before = current.clone();
    let reject = |payload: &[u8], occupancy: &Actors, selected_native: &RomImage, reason: &str| {
        expect_rejection(
            current,
            occupancy,
            world,
            metadata,
            loaded,
            selected_native,
            payload,
            reason,
        );
        assert_eq!(current, &before);
    };
    // Equal fixture bytes ensure this fails for alias collapse rather than content.
    reject(
        bytes,
        actors,
        loaded,
        "checkpoint distinct allocations became aliases",
    );
    let mut bad = bytes.to_vec();
    bad[body + 46 + 8] ^= 1;
    reject(
        &bad,
        actors,
        native,
        "checkpoint allocation content differs",
    );
}

#[path = "actor_prepared_checkpoint_cases.rs"]
mod prepared_checkpoint_cases;
