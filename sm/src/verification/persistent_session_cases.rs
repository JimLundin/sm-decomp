//! Movies cannot replace a prepared Session with independently supplied Native content.
#![cfg(sm_verify_dll)]
#![forbid(unsafe_code)]

use crate::game_state::{
    GameOwner, PreparedSamusBinding, SamusAuthority, SamusBindingDescription, SamusExecution,
    persistent_session::{
        ActorQueueProjection, SamusPublications, SessionStartDescription, SessionStartRequirements,
        SuppliedSessionStart,
    },
};
use crate::{samus_content_binding_cases as content, samus_game_binding_cases as binding};
use sm_game::scene::owned_samus::SamusSources as _;
use std::sync::Arc;
use {
    crate::game::scene::records as sc,
    sm_game::{
        external_inputs::ExternalInputs,
        scene::{
            samus_body::MarkerPublication,
            samus_content::ContentDescription,
            start as ss,
            state::{DisplayError, TransferSource},
        },
        session::Session,
    },
};

#[path = "session_audio_checkpoint_cases.rs"]
mod session_audio_checkpoint_cases;

fn roles(
    mut roles: ContentDescription,
    gameplay: [u8; 32],
    media: sc::Revision,
) -> ContentDescription {
    roles.gameplay = gameplay;
    for binding in Arc::make_mut(&mut roles.requests) {
        binding.request.half.descriptor.revision = media;
        for transfer in [
            &mut binding.request.half.first,
            &mut binding.request.half.second,
        ] {
            let TransferSource::Asset(source) = &mut transfer.source else {
                panic!("fixture asset source")
            };
            source.asset.revision = media;
        }
    }
    roles
}

/// Reuses the host's complete explicit Scene/Reference/input/audio/marker fixture.
/// Numeric gameplay starts empty; this is a synthetic lifecycle, not a captured game start.
pub fn persistent_session_native_rebinding_preserves_ready_owner(
    display_required: ss::DisplayStartRequirements,
    mut display: ss::DisplayStartDescription,
    media: Arc<sc::SceneMedia>,
    marker_variants: Arc<sc::ObjectVariants>,
) {
    let reference = |id| sc::AssetRef {
        id: sc::AssetId(id),
        revision: media.revision(),
    };
    let mut assets = media.assets().cloned().collect::<Vec<_>>();
    for (id, encoding, count, value) in [
        (201, sc::TileEncoding::Affine8, 512, 0x21),
        (202, sc::TileEncoding::Planar4, 1, 0x23),
    ] {
        assets.push(sc::SceneAsset::Atlas(sc::TileAtlas {
            reference: reference(id),
            encoding,
            tile_count: count,
            data: vec![value; count as usize * if id == 201 { 64 } else { 16 }].into(),
        }));
    }
    assets.extend([
        sc::SceneAsset::Room(reference(203)),
        sc::SceneAsset::Room(reference(204)),
    ]);
    let media = Arc::new(
        sc::SceneMedia::new(
            media.revision(),
            assets,
            media.palette_incarnations().cloned().collect(),
            media.compiled_images().cloned().collect(),
        )
        .unwrap(),
    );
    let scene = display.committed_scene.take().unwrap();
    display.committed_scene = Some(
        sc::Scene::new(scene.description().clone(), media.clone())
            .unwrap()
            .with_producer_history(Arc::from(scene.producer_history().to_vec()))
            .unwrap(),
    );
    let projection = core::array::from_fn(|layer| ActorQueueProjection {
        first: layer * 16,
        words: if layer == 7 { 2 } else { 16 },
    });
    let state = display.display.as_mut().unwrap();
    for (layer, queue) in state.actor_queues.iter_mut().enumerate() {
        queue
            .actors
            .resize(projection[layer].words, 0x400 + layer as u16);
    }
    let marker = MarkerPublication {
        producer: sc::ProducerObjectId(22),
        variants: marker_variants,
    };
    let publications = Arc::new(SamusPublications {
        body: sc::ProducerObjectId(21),
        echoes: [None; 4],
        object_atlases: [(0, [reference(2), reference(15)])].into(),
        object_first_order: Some(0),
    });
    let expected_roles = roles(
        content::expected(),
        display_required.gameplay,
        media.revision(),
    );
    let supplied_roles = roles(
        content::supplied(),
        display_required.gameplay,
        media.revision(),
    );
    let required = SessionStartRequirements {
        display: display_required,
        roles: expected_roles,
        marker: marker.clone(),
        actor_queue_projection: projection,
        samus_publications: publications.clone(),
        audio_catalog: Arc::new(crate::game_state::persistent_session::AudioRequestCatalog {
            music: std::collections::BTreeMap::new(),
            sounds: std::collections::BTreeMap::new(),
        }),
    };
    let codec_required = required.display.clone();
    let binary_contract =
        crate::game_state::persistent_session::SessionStartContract::new(required.clone());
    let binary_media = media.clone();
    let owner = GameOwner::with_world(crate::room_world_fixture::binding(false));
    owner.with(|game| {
        game.register_rom(binding::image());
        game.register_native_rom(binding::image());
    });
    let mut gameplay = owner.with(|game| Session::with_world(game.world().clone()));
    *gameplay.presentation.visual_mut() = state.visual.clone();
    *gameplay.presentation.palette_mut() = state.player_palette.clone();
    *gameplay.presentation.atmospheric_mut() = state.atmospheric.clone();
    for (layer, layout) in projection.iter().enumerate() {
        *gameplay.presentation.actor_size_mut(layer) = state.actor_queues[layer].size;
        for index in 0..layout.words {
            *gameplay.presentation.actor_entry_mut(layout.first + index) =
                state.actor_queues[layer].actors[index];
        }
    }
    let continuation = display.continuation.unwrap();
    *gameplay.control.primary_mut() = continuation.primary;
    *gameplay.control.secondary_mut() = continuation.secondary;
    let mut observations = ExternalInputs::default();
    observations.frame = state.nmi.frame_word;
    observations.transitions.frame_gamma = gameplay.player.transitions.frame_gamma;
    let native = owner.with(|game| {
        game.prepare_samus_binding(
            SamusBindingDescription {
                world: game.world().clone(),
                roles: supplied_roles.clone(),
                spritemaps: display.samus_content.as_ref().unwrap().clone(),
                request_media: media.clone(),
                execution: SamusExecution::RetainedNative,
            },
            &SamusBindingDescription {
                world: game.world().clone(),
                roles: required.roles.clone(),
                spritemaps: required.display.samus_content.clone(),
                request_media: media.clone(),
                execution: SamusExecution::RetainedNative,
            },
        )
        .unwrap()
    });
    owner
        .start_supplied_session(SuppliedSessionStart::new(
            SessionStartDescription {
                gameplay: Some(gameplay),
                observations: Some(observations),
                display,
                roles: Some(supplied_roles),
                marker: Some(marker),
                actor_queue_projection: Some(projection),
                samus_publications: Some(publications),
                audio_requests: Some(crate::game_state::persistent_session::AudioRequests {
                    music: crate::game_state::persistent_session::MusicRequests {
                        tracks: [0; 8],
                        delays: [0; 8],
                        write: 0,
                        read: 0,
                        entry: 0,
                        timer: 0,
                        current_track: 0,
                        data_index: 0,
                        track_index: 0,
                        meanings: core::array::from_fn(|_| None),
                        entry_meaning: None,
                        current_meaning: None,
                    },
                    sounds: core::array::from_fn(|_| {
                        crate::game_state::persistent_session::SoundRequests {
                            read: 0,
                            write: 0,
                            state: 0,
                            current: 0,
                            clear_delay: 0,
                            maximum_queued: 0,
                            queue: [0; 16],
                            meanings: core::array::from_fn(|_| None),
                            current_meaning: None,
                        }
                    }),
                    uploading: 0,
                    downtime: 0,
                    catalog: required.audio_catalog.clone(),
                    pending_mutation: false,
                }),
            },
            media,
            required,
        ))
        .unwrap();
    assert_audio_reference_binding(&owner, &codec_required);
    assert_projectiles_checkpoint(&owner, &codec_required);
    let (numeric_start, _) = owner.checkpoint_session().unwrap().into_start().unwrap();
    sm_game::actors::snapshot::assert_numeric_checkpoint(
        numeric_start
            .gameplay
            .as_ref()
            .unwrap()
            .control
            .metadata()
            .unwrap(),
    );
    owner.with(|game| {
        let native = game.capture_native_rom_binding().unwrap();
        crate::game::actors::identity::assert_owned_checkpoint(
            numeric_start.gameplay.as_ref().unwrap(),
            game.loaded_rom.as_ref().unwrap(),
            &native,
        );
    });
    crate::game::session_checkpoint::assert_gameplay_checkpoint(
        numeric_start
            .gameplay
            .as_ref()
            .unwrap()
            .control
            .metadata()
            .unwrap(),
    );
    let binary_checkpoint =
        assert_full_session_binary(&owner, binary_media.clone(), &binary_contract);
    assert_ready_owner_rejects_native_binding(&owner, native);
    assert_clean_runtime_writer_preserves_admission(&owner);
    assert_dirty_runtime_lease_preserves_native_mutation(&owner);
    assert_binary_restore_after_failed_interval(
        &owner,
        &binary_checkpoint,
        binary_media,
        &binary_contract,
    );
    session_audio_checkpoint_cases::run(&owner, &binary_contract);
}

/// Movies cannot forge a complete owned Session envelope or immutable content binding.
fn assert_full_session_binary(
    owner: &GameOwner,
    media: Arc<sc::SceneMedia>,
    contract: &crate::game_state::persistent_session::SessionStartContract,
) -> Vec<u8> {
    let bytes = owner.encode_session_checkpoint().unwrap();
    assert_eq!(&bytes[..10], b"SMSESSION\x01");
    let identity = owner.with(|game| game.actor_identity().clone());
    let ram = owner.with(|game| {
        (0..game.ram().len())
            .map(|at| game.compatibility_byte(at))
            .collect::<Vec<_>>()
    });
    let mut schema = bytes.clone();
    schema[9] = 2;
    let mut count = bytes.clone();
    count[10..18].copy_from_slice(&u64::MAX.to_le_bytes());
    let mut binding = bytes.clone();
    binding[18] ^= 0x80;
    let mut trailing = bytes.clone();
    trailing.push(0xa7);
    for rejected in [
        schema,
        count,
        binding,
        trailing,
        bytes[..bytes.len() - 1].to_vec(),
    ] {
        assert!(
            owner
                .restore_session_checkpoint(&rejected, media.clone(), contract)
                .is_err()
        );
        assert_eq!(owner.encode_session_checkpoint().unwrap(), bytes);
        owner.with(|game| {
            assert_eq!(*game.actor_identity(), identity);
            assert!(
                ram.iter()
                    .enumerate()
                    .all(|(at, byte)| game.compatibility_byte(at) == *byte)
            );
        });
    }
    // Same revision and extent, independently changed actual allocation content.
    let mut changed = contract.requirements().clone();
    let allocation = Arc::make_mut(changed.roles.allocations.values_mut().next().unwrap());
    Arc::make_mut(&mut allocation.bytes)[0] ^= 0x40;
    let changed = crate::game_state::persistent_session::SessionStartContract::new(changed);
    assert!(
        owner
            .restore_session_checkpoint(&bytes, media.clone(), &changed)
            .is_err()
    );
    assert_eq!(owner.encode_session_checkpoint().unwrap(), bytes);
    owner
        .restore_session_checkpoint(&bytes, media, contract)
        .unwrap();
    assert_eq!(owner.encode_session_checkpoint().unwrap(), bytes);
    owner.with(|game| {
        crate::game::actors::identity::assert_installed_epoch(&identity, game.actor_identity())
    });
    assert_leased_owner_rejects_binary_restore(owner, &bytes, contract);
    bytes
}

/// Movies cannot invoke the public restore gate while the gameplay lease is held.
fn assert_leased_owner_rejects_binary_restore(
    owner: &GameOwner,
    bytes: &[u8],
    contract: &crate::game_state::persistent_session::SessionStartContract,
) {
    let (_, media) = owner.checkpoint_session().unwrap().into_start().unwrap();
    let (leased, epoch) = owner.with(|game| {
        let epoch = game.compatibility_write_epoch();
        (game.lease_persistent_session().unwrap().unwrap(), epoch)
    });
    assert_eq!(
        owner
            .restore_session_checkpoint(bytes, media, contract)
            .err()
            .as_deref(),
        Some("Session restore ownership unavailable"),
    );
    owner.with(|game| game.return_persistent_session(leased, true, epoch));
    assert_eq!(owner.encode_session_checkpoint().unwrap(), bytes);
}

/// Movies cannot inject a malformed save while a failed Runtime owner is retained.
fn assert_binary_restore_after_failed_interval(
    owner: &GameOwner,
    bytes: &[u8],
    media: Arc<sc::SceneMedia>,
    contract: &crate::game_state::persistent_session::SessionStartContract,
) {
    assert!(owner.checkpoint_session().is_err());
    assert_eq!(owner.with(|game| game.ram().read_byte(0x09c2)), 0x7b);
    let mut malformed = bytes.to_vec();
    malformed[9] = 2;
    assert_eq!(
        owner
            .restore_session_checkpoint(&malformed, media.clone(), contract)
            .err()
            .as_deref(),
        Some("unsupported Session checkpoint schema"),
    );
    assert!(owner.checkpoint_session().is_err());
    assert_eq!(owner.with(|game| game.ram().read_byte(0x09c2)), 0x7b);
    owner
        .restore_session_checkpoint(bytes, media, contract)
        .unwrap();
    assert_eq!(owner.encode_session_checkpoint().unwrap(), bytes);
    assert_eq!(owner.with(|game| game.ram().read_byte(0x09c2)), 0x00);
}

/// No movie covers malformed Projectiles checkpoints or an independently valid binding mismatch.
fn assert_projectiles_checkpoint(owner: &GameOwner, required: &ss::DisplayStartRequirements) {
    use sm_game::scene::component::DisplayStartMetadata;
    use {sm_game::launch_motion::MovementDirection, sm_game::session::Projectiles};

    let (start, _) = owner.checkpoint_session().unwrap().into_start().unwrap();
    let display = start.display;
    let a = DisplayStartMetadata {
        identity: display.identity.unwrap(),
        reference: display.reference.unwrap(),
        input_history: display.input_history.unwrap(),
        continuation: display.continuation.unwrap(),
        audio: display.audio.unwrap(),
        samus_content: display.samus_content.unwrap(),
    };
    ss::validate_control_metadata(&a, required).unwrap();
    let mut expected = Projectiles::default();
    expected.shots[0].x = 0x1234;
    expected.shots[0].damage = 0x2345;
    expected.shots[0].x_fraction = 0x3456;
    expected.shots[9].x = 0xabcd;
    expected.shots[9].damage = 0xbcde;
    expected.shots[9].x_fraction = 0xcdef;
    expected.trails[0].left.x = 0x4567;
    expected.trails[0].left.cursor = 0x5678;
    expected.trails[0].right.y = 0x6789;
    expected.trails[0].right.remaining = 0x789a;
    expected.trails[17].left.x = 0x89ab;
    expected.trails[17].left.cursor = 0x9abc;
    expected.trails[17].right.y = 0xabce;
    expected.trails[17].right.remaining = 0xbcdf;
    expected.controls.cooldown = 0x1357;
    expected.controls.flare_timers[2] = 0x2468;
    crate::game::launch_motion_ram::record_observed(
        &mut expected.launch,
        MovementDirection::Left,
        -0x12345,
    );
    crate::game::launch_motion_ram::record_observed(
        &mut expected.launch,
        MovementDirection::Down,
        0x23456,
    );
    let bytes = expected.encode_checkpoint(&a).unwrap();
    let decoded = Projectiles::decode_checkpoint(&bytes, &a).unwrap();
    assert_eq!(decoded, expected);
    assert_eq!((decoded.shots[0].x, decoded.shots[9].x), (0x1234, 0xabcd));
    assert_eq!(
        (decoded.trails[0].left.x, decoded.trails[17].left.x),
        (0x4567, 0x89ab)
    );
    assert_eq!(decoded.controls.cooldown, 0x1357);
    assert_eq!(
        Projectiles::decode_checkpoint(&bytes[..bytes.len() - 1], &a)
            .err()
            .as_deref(),
        Some("truncated actor checkpoint")
    );
    let mut extra = bytes.clone();
    extra.push(0xa6);
    assert_eq!(
        Projectiles::decode_checkpoint(&extra, &a).err().as_deref(),
        Some("trailing projectile checkpoint fields")
    );
    let mut b = a.clone();
    b.reference.fingerprint = [0xb6; 32];
    b.identity.reference = b.reference.fingerprint;
    let mut required_b = required.clone();
    required_b.reference = b.reference;
    assert_ne!(a.reference.fingerprint, b.reference.fingerprint);
    ss::validate_control_metadata(&b, &required_b).unwrap();
    assert_eq!(
        Projectiles::decode_checkpoint(&bytes, &b).err().as_deref(),
        Some("projectile checkpoint source identity or Reference changed")
    );
}

/// No movie covers mismatched detached payloads decoded under another valid Reference.
fn assert_audio_reference_binding(owner: &GameOwner, required: &ss::DisplayStartRequirements) {
    use crate::game_state::persistent_session::AudioRequests;
    use sm_game::scene::component::DisplayStartMetadata;

    let (start, _) = owner.checkpoint_session().unwrap().into_start().unwrap();
    let requests = start.audio_requests.unwrap();
    let display = start.display;
    let a = DisplayStartMetadata {
        identity: display.identity.unwrap(),
        reference: display.reference.unwrap(),
        input_history: display.input_history.unwrap(),
        continuation: display.continuation.unwrap(),
        audio: display.audio.unwrap(),
        samus_content: display.samus_content.unwrap(),
    };
    assert_eq!(a.audio.revision, sc::Revision([5; 32]));
    assert_eq!(
        a.audio.music,
        ss::OngoingMusic::Playing(ss::ProgramAnchor {
            program: sc::AssetId(1),
            revision: sc::Revision([5; 32]),
            source_interval: 23,
            phase: 59,
            timing_policy: a.audio.timing_policy,
        })
    );
    assert!(requests.catalog.music.is_empty());
    assert!(requests.catalog.sounds.is_empty());
    assert_eq!(requests.music.tracks, [0; 8]);
    assert_eq!(requests.music.delays, [0; 8]);
    assert!(requests.sounds.iter().all(|sound| sound.queue == [0; 16]));

    let mut b = a.clone();
    b.reference.fingerprint = [0xa4; 32];
    b.reference.ticks_numerator = a.reference.ticks_numerator.checked_add(1).unwrap();
    b.identity.reference = b.reference.fingerprint;
    let mut required_b = required.clone();
    required_b.reference = b.reference;
    assert_ne!(a.reference.fingerprint, b.reference.fingerprint);
    assert_ne!(a.reference.ticks_numerator, b.reference.ticks_numerator);
    assert_eq!(a.reference.ticks_denominator, b.reference.ticks_denominator);
    ss::validate_control_metadata(&a, required).unwrap();
    ss::validate_control_metadata(&b, &required_b).unwrap();
    requests.validate(&a, &requests.catalog).unwrap();
    requests.validate(&b, &requests.catalog).unwrap();
    assert_eq!(a.audio, b.audio);

    let bytes = requests.encode_checkpoint(&a, &requests.catalog).unwrap();
    assert_eq!(
        AudioRequests::decode_checkpoint(&bytes, &a, requests.catalog.clone()).unwrap(),
        requests
    );
    assert_eq!(
        AudioRequests::decode_checkpoint(&bytes, &b, requests.catalog.clone())
            .err()
            .as_deref(),
        Some("audio checkpoint Reference declaration changed")
    );
}

pub(crate) fn assert_ready_owner_rejects_native_binding(
    owner: &GameOwner,
    native: PreparedSamusBinding,
) {
    let before = owner.checkpoint_session().unwrap().into_start().unwrap();
    owner.with(|game| {
        let catalog = game.samus_spritemaps().unwrap().unwrap();
        let media = game.samus_request_media().unwrap().unwrap();
        assert_eq!(game.samus_execution(), Ok(Some(SamusExecution::Prepared)));
        assert_eq!(game.samus_authority(), SamusAuthority::Available);
        assert_eq!(game.install_samus_binding(native), Err(DisplayError::Phase));
        assert_eq!(game.samus_execution(), Ok(Some(SamusExecution::Prepared)));
        assert_eq!(game.samus_authority(), SamusAuthority::Available);
        assert!(Arc::ptr_eq(
            &catalog,
            &game.samus_spritemaps().unwrap().unwrap()
        ));
        assert!(Arc::ptr_eq(
            &media,
            &game.samus_request_media().unwrap().unwrap()
        ));
        assert_eq!(game.begin_samus_mutation().err(), Some(DisplayError::Phase));
    });
    let after = owner.checkpoint_session().unwrap().into_start().unwrap();
    assert_eq!(after.0.gameplay, before.0.gameplay);
    assert_eq!(after.0.observations, before.0.observations);
    assert_eq!(after.0.display.identity, before.0.display.identity);
    assert_eq!(after.0.display.reference, before.0.display.reference);
    assert_eq!(
        after.0.display.input_history,
        before.0.display.input_history
    );
    assert_eq!(after.0.display.continuation, before.0.display.continuation);
    assert_eq!(after.0.display.audio, before.0.display.audio);
    assert_eq!(after.0.display.display, before.0.display.display);
    let actual_resident = after.0.display.resident.as_ref().unwrap();
    let expected_resident = before.0.display.resident.as_ref().unwrap();
    assert_eq!(actual_resident.image, expected_resident.image);
    assert_eq!(actual_resident.palette, expected_resident.palette);
    assert_eq!(actual_resident.words, expected_resident.words);
    assert_eq!(actual_resident.colors, expected_resident.colors);
    assert_eq!(actual_resident.objects, expected_resident.objects);
    assert_eq!(actual_resident.produced, expected_resident.produced);
    assert!(Arc::ptr_eq(&after.1, &before.1));
}

// No movie reaches this retained Prepared Runtime lease's own-write seam.
fn assert_clean_runtime_writer_preserves_admission(owner: &GameOwner) {
    use crate::game::session_runtime::Runtime;
    owner.with(|game| {
        let world = game.world().clone();
        let catalog = game.samus_spritemaps().unwrap().unwrap();
        let media = game.samus_request_media().unwrap().unwrap();
        let mut runtime = Runtime::new(game);
        runtime.update_maw_visibility(0);
        assert!(Arc::ptr_eq(&runtime.session().world, &world));
        assert_eq!(runtime.session().inventory.health.current, 0x0000);
        drop(runtime);
        assert_eq!(game.samus_authority(), SamusAuthority::Available);
        assert_eq!(game.samus_execution(), Ok(Some(SamusExecution::Prepared)));
        assert!(Arc::ptr_eq(game.world(), &world));
        assert!(Arc::ptr_eq(
            &catalog,
            &game.samus_spritemaps().unwrap().unwrap()
        ));
        assert!(Arc::ptr_eq(
            &media,
            &game.samus_request_media().unwrap().unwrap()
        ));
        game.checkpoint_session().unwrap();
    });
    owner.checkpoint_session().unwrap();
}

// No movie holds a Runtime lease while invoking a native compatibility byte helper.
fn assert_dirty_runtime_lease_preserves_native_mutation(owner: &GameOwner) {
    use crate::game::session_runtime::Runtime;
    let before = owner.checkpoint_session().unwrap().into_start().unwrap();
    assert_eq!(
        before.0.gameplay.as_ref().unwrap().inventory.health.current,
        0x0000
    );
    owner.with(|game| {
        let world = game.world().clone();
        let native = game.capture_native_rom_binding().unwrap();
        assert_eq!(game.samus_execution(), Ok(Some(SamusExecution::Prepared)));
        assert_eq!(game.samus_authority(), SamusAuthority::Available);
        assert_eq!(game.ram().read_byte(0x09c2), 0x00);
        assert_eq!(game.ram().read_byte(0x09c3), 0x00);
        let runtime = Runtime::new(game);
        assert_eq!(runtime.byte(0x09c2), 0x00);
        assert_eq!(runtime.byte(0x09c3), 0x00);
        runtime.samus_game().compatibility_write_byte(0x09c2, 0x7b);
        assert_eq!(runtime.samus_game().ram().read_byte(0x09c2), 0x7b);
        assert_eq!(runtime.samus_game().ram().read_byte(0x09c3), 0x00);
        let observed_low = runtime.byte(0x09c2);
        let observed_high = runtime.byte(0x09c3);
        drop(runtime);
        assert_eq!(game.ram().read_byte(0x09c2), 0x7b);
        assert_eq!(game.ram().read_byte(0x09c3), 0x00);
        assert_eq!(observed_low, 0x7b);
        assert_eq!(observed_high, 0x00);
        assert!(Arc::ptr_eq(&world, game.world()));
        assert!(native.same_allocation(&game.capture_native_rom_binding().unwrap()));
        assert_eq!(game.samus_authority(), SamusAuthority::Revoked);
        assert_eq!(game.samus_execution(), Err(DisplayError::Poisoned));
        assert!(game.checkpoint_session().is_err());
        assert_eq!(
            game.lease_persistent_session().err(),
            Some(DisplayError::Poisoned)
        );
        assert_eq!(game.ram().read_byte(0x09c2), 0x7b);
    });
    assert!(owner.checkpoint_session().is_err());
}

pub use session_audio_checkpoint_cases::{
    bank_upload_physical_prefixes, bank_upload_physical_timeout_boundaries,
};
