//! Movies cannot supply authored lineage and nonempty palette receipts to a detached restore.
use super::*;
use crate::actors::snapshot::CheckpointImages;
use crate::{
    actors::{content::*, media::*},
    room_entry::world::*,
    session::Point,
};

/// No movie composes the actual initializer/media fixture with an independent immutable catalog.
fn authored_world(change: u8, color: u8) -> Arc<WorldBinding> {
    let original = crate::actors::fixture::binding(change == 1, AppearanceId(20), true);
    let mut world = original.definition().unwrap().clone();
    let mut keys = original.compatibility_keys().clone();
    world
        .resources
        .insert(ResourceId(5), ResourceKind::TileTable);
    world.resources.insert(ResourceId(6), ResourceKind::Tiles);
    keys.resources.insert(
        ResourceId(5),
        ResourceKey::Long(LegacyAddress24([0, 0x90, 0xc0])),
    );
    keys.resources.insert(
        ResourceId(6),
        ResourceKey::Long(LegacyAddress24([0, 0x91, 0xc0])),
    );
    world.tilesets.insert(
        TilesetId(9),
        TilesetDefinition {
            tile_table: ResourceId(5),
            tiles: ResourceId(6),
            palette: ResourceId(4),
        },
    );
    let mut actors = original.actors().definitions().unwrap().clone();
    let mut actor_keys = original.actors().keys().unwrap().clone();
    actors.programs.insert(
        ActorProgramId(70),
        ActorProgramDefinition::CompatibleEnemyScript,
    );
    actor_keys
        .programs
        .insert(ActorProgramId(70), (0xa2, 0x9000));
    actors.templates.insert(
        ActorTemplateId(11),
        PopulationEntry {
            id: PopulationEntryId(100),
            species: SpeciesId(7),
            appearance: Some(AppearanceId(20)),
            position: Point {
                x: if change == 2 { 52 } else { 51 },
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
        .insert(ActorTemplateId(11), (0xa1, 0x8000));
    let binding = WorldBinding::prepared_with_actors(
        world.clone(),
        keys.clone(),
        actors,
        actor_keys,
        original.actor_media().definitions().unwrap().clone(),
    )
    .unwrap();
    // Same independently authored encoded stream used by the actual media-runtime fixture.
    let mut source = vec![0xff; 0x400000];
    let at = crate::rom::lorom_offset(0xb49000) as usize;
    source[at..at + 4].copy_from_slice(&[0xe4, 0x1f, color, 0xff]);
    let assets = PreparedRoomAssets::import(&source, &world, &keys).unwrap();
    WorldBinding::with_room_assets(binding, assets).unwrap()
}

/// No movie inspects authored occurrence lineage and both palette source tags after decoding.
pub fn run(session: &Session, loaded: &RomImage, native: &RomImage) {
    let world = authored_world(0, 0x66);
    let metadata = session.control.metadata().unwrap();
    let original = session.actor_identity.clone();
    let mut current = original.clone();
    let mut actors = session.actors.clone();
    assert_eq!(current.timeline.next_birth_order, 0);
    assert!(
        current
            .timeline
            .slots
            .iter()
            .all(|slot| slot.occupant.is_none())
    );
    let birth = current
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
    let old = current.commit_birth(birth);
    let revision = world.prepared_room_assets().unwrap().fingerprint();
    current.record_prepared_palette_residency(0, ResourceId(4), revision);
    current.record_palette_residency(9, Some(ResourceId(3)), loaded.clone());
    assert_eq!(
        world.actor_media().validate_material_residency(
            MaterialId(52),
            current.palette_residency(),
            loaded
        ),
        Ok(())
    );
    assert_eq!(
        world.actor_media().validate_appearance_residency(
            AppearanceId(20),
            current.palette_residency(),
            loaded
        ),
        Ok(())
    );
    let sources = [(ResourceId(3), loaded.clone())];
    let bytes = current
        .encode_owned_checkpoint(
            &actors,
            &world,
            metadata,
            CheckpointImages {
                loaded: Some(loaded),
                native: Some(native),
                sources: &sources,
            },
        )
        .unwrap();
    let before = current.clone();
    let decoded = ActorIdentityState::decode_owned_checkpoint(
        &bytes,
        &current,
        &actors,
        &world,
        metadata,
        CheckpointImages {
            loaded: Some(loaded),
            native: Some(native),
            sources: &sources,
        },
    )
    .unwrap();
    assert_occurrence(&decoded, &actors, old);
    assert_eq!(decoded.timeline, current.timeline);
    assert_eq!(decoded.palette_residency, current.palette_residency);
    assert_palettes(&decoded, &world, loaded);
    assert_eq!(
        decoded
            .encode_owned_checkpoint(
                &actors,
                &world,
                metadata,
                CheckpointImages {
                    loaded: Some(loaded),
                    native: Some(native),
                    sources: &sources
                }
            )
            .unwrap(),
        bytes
    );
    assert_eq!(current, before);
    changed_catalogs(&current, &actors, &world, metadata, loaded, native, &bytes);
    changed_palette_source(&current, &actors, &world, metadata, loaded, native, &bytes);
    assert_eq!(current, before);
    assert_eq!(session.actor_identity, original);
}

/// No movie directly distinguishes prepared fingerprint receipts from live allocation receipts.
fn assert_palettes(state: &ActorIdentityState, world: &WorldBinding, loaded: &RomImage) {
    let rows = state
        .palette_residency
        .checkpoint_rows()
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    assert_eq!((rows[0].0, rows[0].1), (0, ResourceId(4)));
    assert!(matches!(rows[0].2, PaletteProvenance::Prepared(revision)
        if *revision == world.prepared_room_assets().unwrap().fingerprint()));
    assert_eq!((rows[1].0, rows[1].1), (9, ResourceId(3)));
    assert!(matches!(rows[1].2, PaletteProvenance::Legacy(image) if image.same_allocation(loaded)));
    assert_eq!(
        world.actor_media().validate_material_residency(
            MaterialId(52),
            state.palette_residency(),
            loaded
        ),
        Ok(())
    );
    assert_eq!(
        world.actor_media().validate_appearance_residency(
            AppearanceId(20),
            state.palette_residency(),
            loaded
        ),
        Ok(())
    );
}

/// No movie independently validates changed media/gameplay catalogs under the same World revision.
fn changed_catalogs(
    current: &ActorIdentityState,
    actors: &Actors,
    world: &Arc<WorldBinding>,
    metadata: &DisplayStartMetadata,
    loaded: &RomImage,
    native: &RomImage,
    bytes: &[u8],
) {
    let before = current.clone();
    for change in [1, 2] {
        let changed = authored_world(change, 0x66);
        assert_eq!(changed.revision(), world.revision());
        if change == 1 {
            assert_eq!(
                changed.actor_gameplay_revision(),
                world.actor_gameplay_revision()
            );
            assert_ne!(changed.actor_media_revision(), world.actor_media_revision());
            assert_eq!(
                changed.actor_media().definitions().unwrap().appearances[&AppearanceId(20)].frames
                    [&(PoseId(9), PartRoleId(0))],
                FrameId(31)
            );
        } else {
            assert_ne!(
                changed.actor_gameplay_revision(),
                world.actor_gameplay_revision()
            );
            assert_eq!(changed.actor_media_revision(), world.actor_media_revision());
            assert_eq!(
                changed.actors().definitions().unwrap().templates[&ActorTemplateId(11)]
                    .position
                    .x,
                52
            );
        }
        let selected = ResourceId(if change == 1 { 4 } else { 3 });
        let mut matching = current.clone();
        matching.record_palette_residency(9, Some(selected), loaded.clone());
        let sources = [(selected, loaded.clone())];
        assert_eq!(
            changed.actor_media().validate_appearance_residency(
                AppearanceId(20),
                matching.palette_residency(),
                loaded
            ),
            Ok(())
        );
        accept_changed_catalog(
            &matching, actors, &changed, metadata, loaded, native, &sources,
        );
        assert_eq!(
            ActorIdentityState::decode_owned_checkpoint(
                bytes,
                current,
                actors,
                &changed,
                metadata,
                CheckpointImages {
                    loaded: Some(loaded),
                    native: Some(native),
                    sources: &sources
                }
            )
            .err()
            .as_deref(),
            Some("actor checkpoint gameplay or media revision differs")
        );
        assert_eq!(current, &before);
    }
    changed_prepared_catalog(current, actors, world, metadata, loaded, native, bytes);
}

/// No movie replaces a named palette allocation with separately authored bytes during detached decode.
fn changed_palette_source(
    current: &ActorIdentityState,
    actors: &Actors,
    world: &Arc<WorldBinding>,
    metadata: &DisplayStartMetadata,
    loaded: &RomImage,
    native: &RomImage,
    bytes: &[u8],
) {
    let before = current.clone();
    let data = (0..loaded.len())
        .map(|at| loaded.read(at))
        .collect::<Vec<_>>();
    let native_data = (0..native.len())
        .map(|at| native.read(at))
        .collect::<Vec<_>>();
    let changed = RomImage::zeroed(data.len());
    for (at, byte) in data.iter().enumerate() {
        changed.write(at, *byte);
    }
    changed.write(0, data[0] ^ 0x5a);
    let sources = [(ResourceId(3), changed.clone())];
    // All World/gameplay/media declarations remain exactly those originally supplied.
    let mut valid_changed = current.clone();
    valid_changed.record_palette_residency(9, Some(ResourceId(3)), changed.clone());
    let own = valid_changed
        .encode_owned_checkpoint(
            actors,
            world,
            metadata,
            CheckpointImages {
                loaded: Some(&changed),
                native: Some(native),
                sources: &sources,
            },
        )
        .unwrap();
    let accepted = ActorIdentityState::decode_owned_checkpoint(
        &own,
        current,
        actors,
        world,
        metadata,
        CheckpointImages {
            loaded: Some(&changed),
            native: Some(native),
            sources: &sources,
        },
    )
    .unwrap();
    assert_palettes(&accepted, world, &changed);
    assert_eq!(
        ActorIdentityState::decode_owned_checkpoint(
            bytes,
            current,
            actors,
            world,
            metadata,
            CheckpointImages {
                loaded: Some(&changed),
                native: Some(native),
                sources: &sources
            }
        )
        .err()
        .as_deref(),
        Some("checkpoint allocation content differs")
    );
    assert_eq!(current, &before);
    assert_eq!(
        (0..loaded.len())
            .map(|at| loaded.read(at))
            .collect::<Vec<_>>(),
        data
    );
    assert_eq!(
        (0..native.len())
            .map(|at| native.read(at))
            .collect::<Vec<_>>(),
        native_data
    );
    assert!(
        (0..data.len())
            .all(|at| changed.read(at) == if at == 0 { data[0] ^ 0x5a } else { data[at] })
    );
}

/// No movie independently inspects authored birth counters, species, template and appearance.
fn assert_occurrence(decoded: &ActorIdentityState, actors: &Actors, old: InstanceHandle) {
    assert_eq!(decoded.timeline.next_birth_order, 1);
    assert_eq!(decoded.timeline.slots[0].generation, 1);
    let occupant = decoded.timeline.slots[0].occupant.as_ref().unwrap();
    assert_eq!(occupant.birth_order, 0);
    assert_eq!(
        occupant.provenance,
        ActorProvenance {
            authored_species: Some(SpeciesId(7)),
            origin: ActorBirthOrigin::Dynamic {
                template: ActorTemplateId(11),
                part: 0
            },
        }
    );
    assert_eq!(occupant.appearance, Some(AppearanceId(20)));
    assert_eq!(
        decoded.resolve(old, &actors),
        Err(ActorIdentityError::StaleEpoch)
    );
}

/// No movie validates changed immutable palette content against its own updated receipt first.
fn changed_prepared_catalog(
    current: &ActorIdentityState,
    actors: &Actors,
    world: &Arc<WorldBinding>,
    metadata: &DisplayStartMetadata,
    loaded: &RomImage,
    native: &RomImage,
    bytes: &[u8],
) {
    let before = current.clone();
    let sources = [(ResourceId(3), loaded.clone())];
    let changed = authored_world(0, 0x67);
    assert_ne!(changed.revision(), world.revision());
    assert_eq!(
        changed.actor_gameplay_revision(),
        world.actor_gameplay_revision()
    );
    assert_eq!(changed.actor_media_revision(), world.actor_media_revision());
    let mut matching = current.clone();
    matching.record_prepared_palette_residency(
        0,
        ResourceId(4),
        changed.prepared_room_assets().unwrap().fingerprint(),
    );
    assert_palettes(&matching, &changed, loaded);
    accept_changed_catalog(
        &matching, actors, &changed, metadata, loaded, native, &sources,
    );
    assert_eq!(
        ActorIdentityState::decode_owned_checkpoint(
            bytes,
            current,
            actors,
            &changed,
            metadata,
            CheckpointImages {
                loaded: Some(loaded),
                native: Some(native),
                sources: &sources
            }
        )
        .err()
        .as_deref(),
        Some("actor checkpoint world revision differs")
    );
    assert_eq!(current, &before);
}

/// No movie independently admits changed catalog/source content before rejecting an older payload.
fn accept_changed_catalog(
    current: &ActorIdentityState,
    actors: &Actors,
    world: &Arc<WorldBinding>,
    metadata: &DisplayStartMetadata,
    loaded: &RomImage,
    native: &RomImage,
    sources: &[(ResourceId, RomImage)],
) {
    let own = current
        .encode_owned_checkpoint(
            actors,
            world,
            metadata,
            CheckpointImages {
                loaded: Some(loaded),
                native: Some(native),
                sources: sources,
            },
        )
        .unwrap();
    let accepted = ActorIdentityState::decode_owned_checkpoint(
        &own,
        current,
        actors,
        world,
        metadata,
        CheckpointImages {
            loaded: Some(loaded),
            native: Some(native),
            sources: sources,
        },
    )
    .unwrap();
    assert_eq!(accepted.timeline, current.timeline);
    assert_eq!(accepted.palette_residency, current.palette_residency);
}
