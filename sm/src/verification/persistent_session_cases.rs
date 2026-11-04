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

struct PersistentFixture {
    owner: Box<GameOwner>,
    supplied: SuppliedSessionStart,
    native: PreparedSamusBinding,
    codec_required: ss::DisplayStartRequirements,
    binary_contract: crate::game_state::persistent_session::SessionStartContract,
    binary_media: Arc<sc::SceneMedia>,
}

/// Reuses the host's complete explicit Scene/Reference/input/audio/marker fixture.
/// Numeric gameplay starts empty; this is a synthetic lifecycle, not a captured game start.
fn persistent_fixture(
    display_required: ss::DisplayStartRequirements,
    mut display: ss::DisplayStartDescription,
    media: Arc<sc::SceneMedia>,
    marker_variants: Arc<sc::ObjectVariants>,
) -> PersistentFixture {
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
        dialogue: None,
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
    // Explicitly authored zero values for this synthetic lifecycle fixture.
    gameplay.fx = sm_game::fx_state::FxState::supplied([0; 11]);
    gameplay.menu_window = sm_game::menu_window_state::WindowState::initialized(
        0,
        0,
        Default::default(),
        Default::default(),
        0,
    );
    gameplay.game_loop = sm_game::game_loop::GameLoop::initialized(
        0,
        Default::default(),
        Default::default(),
        [0; 4],
        [0; 7],
    );
    *gameplay.presentation.workspace_mut() = state.workspace.clone();
    gameplay.timing.countdown = sm_game::escape_countdown::State::initialized(
        [0; 3],
        [0; 2],
        sm_game::scene::state::WordOrder::Little,
    );
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
    let input_history = display.input_history.unwrap();
    for word in sm_game::controller_state::Word::ALL {
        *word.get_mut(&mut gameplay.control) = *word.get_history(&input_history);
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
    let supplied = SuppliedSessionStart::new(
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
    );
    PersistentFixture {
        owner,
        supplied,
        native,
        codec_required,
        binary_contract,
        binary_media,
    }
}

/// Reuses the host's complete explicit Scene/Reference/input/audio/marker fixture.
/// Numeric gameplay starts empty; this is a synthetic lifecycle, not a captured game start.
pub fn persistent_session_native_rebinding_preserves_ready_owner(
    display_required: ss::DisplayStartRequirements,
    display: ss::DisplayStartDescription,
    media: Arc<sc::SceneMedia>,
    marker_variants: Arc<sc::ObjectVariants>,
) {
    let PersistentFixture {
        owner,
        supplied,
        native,
        codec_required,
        binary_contract,
        binary_media,
    } = persistent_fixture(display_required, display, media, marker_variants);
    owner.start_supplied_session(supplied).unwrap();
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
        numeric_start.gameplay.as_ref().unwrap(),
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
    record_observed(&mut expected.launch, MovementDirection::Left, -0x12345);
    record_observed(&mut expected.launch, MovementDirection::Down, 0x23456);
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
        let catalog = game.samus_content().samus_spritemaps().unwrap().unwrap();
        let media = game.samus_request_media().unwrap().unwrap();
        assert_eq!(game.samus_execution(), Ok(Some(SamusExecution::Prepared)));
        assert_eq!(game.samus_authority(), SamusAuthority::Available);
        assert_eq!(game.install_samus_binding(native), Err(DisplayError::Phase));
        assert_eq!(game.samus_execution(), Ok(Some(SamusExecution::Prepared)));
        assert_eq!(game.samus_authority(), SamusAuthority::Available);
        assert!(Arc::ptr_eq(
            &catalog,
            &game.samus_content().samus_spritemaps().unwrap().unwrap()
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
        let catalog = game.samus_content().samus_spritemaps().unwrap().unwrap();
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
            &game.samus_content().samus_spritemaps().unwrap().unwrap()
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

/// Record one launch-motion step the way the collision owner does, after the
/// verifier observes it.
fn record_observed(
    motion: &mut sm_game::launch_motion::LaunchMotion,
    direction: sm_game::launch_motion::MovementDirection,
    amount: i32,
) {
    crate::game::player_collision_owned_checks::record_launch(direction, amount);
    motion.record_movement(direction, amount);
}

/// Movies cannot forge initialized Legacy ownership masks before the public admission gate.
pub fn persistent_session_rejects_forged_legacy_fx_ownership(
    required: ss::DisplayStartRequirements,
    display: ss::DisplayStartDescription,
    media: Arc<sc::SceneMedia>,
    variants: Arc<sc::ObjectVariants>,
) {
    use sm_game::persistent_session::{SessionStartError, ValidatedSessionStart};
    let fixture = || {
        persistent_fixture(
            required.clone(),
            display.clone(),
            media.clone(),
            variants.clone(),
        )
    };
    let valid = fixture();
    let world = valid.owner.with(|game| game.world().clone());
    let mut admitted = ValidatedSessionStart::construct(valid.supplied, &world)
        .unwrap()
        .open();
    admitted.checkpoint().unwrap();
    for lanes in [[0; 4], [1, 0, 0, 0], [0, 0, 0, 1 << 63]] {
        let supplied = fixture();
        let (mut start, media, required) = supplied.supplied.into_parts();
        let gameplay = start.gameplay.as_mut().unwrap();
        assert!(!gameplay.presentation.is_owned());
        assert!(gameplay.fx.is_initialized());
        gameplay.fx_control_ownership = sm_game::fx_state::ControlOwnership {
            initialized: true,
            source_lanes: lanes,
        };
        let world = supplied.owner.with(|game| game.world().clone());
        let result = ValidatedSessionStart::construct(
            SuppliedSessionStart::new(start, media, required),
            &world,
        );
        assert!(matches!(
            result,
            Err(SessionStartError::Binding(DisplayError::Source))
        ));
    }
    admitted.gameplay.fx_control_ownership.initialized = false;
    assert!(matches!(admitted.checkpoint(), Err(DisplayError::Source)));
}

/// Movies cannot supply independently conflicting cold scalar cycles with no publication origin.
pub fn persistent_session_rejects_conflicting_cold_fx_cycles(
    required: ss::DisplayStartRequirements,
    display: ss::DisplayStartDescription,
    media: Arc<sc::SceneMedia>,
    variants: Arc<sc::ObjectVariants>,
) {
    use sm_game::scene::hdma_sources::{
        BankWindow, HdmaLiveDescription, HdmaLiveRegion, HdmaOwnedBuffer, HdmaRegionSource,
        HdmaVariable, LiveOwner, SourceBinding,
    };
    use sm_game::{fx_encoding as fx, persistent_session::ValidatedSessionStart};
    let supplied = persistent_fixture(required, display, media, variants);
    let world = supplied.owner.with(|game| game.world().clone());
    let mut session = ValidatedSessionStart::construct(supplied.supplied, &world)
        .unwrap()
        .open()
        .gameplay;
    let owner = |variable| LiveOwner::ObjectVariable {
        object: 0,
        variable,
    };
    let window = |owner, address| SourceBinding::LiveOwner {
        owner,
        windows: vec![BankWindow {
            bank: 0x7e,
            address,
            offset: 0,
            length: 2,
        }]
        .into(),
    };
    let state = &mut session.presentation.owner_mut().unwrap().state;
    // A is already declared by the shared fixture; this cycle also supplies B.
    let (buffers, regions) = state.hdma_live.checkpoint_parts();
    let buffers = buffers
        .iter()
        .map(|(&owner, bytes)| HdmaOwnedBuffer {
            owner,
            bytes: bytes.clone(),
        })
        .collect();
    let mut regions = regions.values().copied().collect::<Vec<_>>();
    regions.push(HdmaLiveRegion {
        owner: owner(HdmaVariable::B),
        source: HdmaRegionSource::ObjectVariable {
            object: 0,
            variable: HdmaVariable::B,
        },
        length: 2,
    });
    let mut live = HdmaLiveDescription::new(buffers, regions).unwrap();
    if let Some(layout) = state.hdma_live.window_control_layout() {
        live.bind_window_controls(layout.clone(), None).unwrap();
    }
    live.bind_native_origins(state.hdma_live.native_origins().clone())
        .unwrap();
    state.hdma_live = live;
    state.hdma.channels[6].source = window(owner(HdmaVariable::A), 0x1920);
    state.hdma.channels[7].source = window(owner(HdmaVariable::B), 0x1914);
    state.hdma.common.objects[0].a = 0x1111;
    state.hdma.common.objects[0].b = 0x2222;
    session.fx_control_ownership = Default::default();
    assert!(matches!(
        fx::adopt_control_ownership(&mut session),
        Err(DisplayError::Source)
    ));
    assert_eq!(
        session
            .presentation
            .owner()
            .unwrap()
            .state
            .hdma
            .common
            .objects[0]
            .a,
        0x1111
    );
    assert_eq!(
        session
            .presentation
            .owner()
            .unwrap()
            .state
            .hdma
            .common
            .objects[0]
            .b,
        0x2222
    );
    let mut selected = session.clone();
    for variable in [HdmaVariable::A, HdmaVariable::B] {
        for byte in 0..2 {
            selected
                .presentation
                .owner_mut()
                .unwrap()
                .state
                .hdma_live
                .note_native_origin(owner(variable), byte, 0x1920 + byte)
                .unwrap();
        }
    }
    fx::adopt_control_ownership(&mut selected).unwrap();
    assert_eq!(fx::control_word(&selected, 0x1914), 0x2222);
    assert_eq!(fx::control_word(&selected, 0x1920), 0x2222);
    let state = &mut session.presentation.owner_mut().unwrap().state;
    state.hdma.common.objects[0].a = 0x3333;
    state.hdma.common.objects[0].b = 0x3333;
    fx::adopt_control_ownership(&mut session).unwrap();
    fx::validate_control_ownership(&session).unwrap();
    assert_eq!(fx::control_word(&session, 0x1914), 0x3333);
    assert_eq!(fx::control_word(&session, 0x1920), 0x3333);
    let state = &session.presentation.owner().unwrap().state;
    assert_eq!(state.hdma.common.objects[0].a, 0x3333);
    assert_eq!(state.hdma.common.objects[0].b, 0);
}

/// Movies cannot construct independent native windows over one supplied Workspace allocation.
fn workspace_alias_fixture(session: &mut Session) {
    use sm_game::presentation_workspace::{Binding, Origin, Point, Role, Window};
    use sm_game::scene::hdma_sources::{
        BankWindow, HdmaLiveDescription, HdmaLiveRegion, HdmaRegionSource, HdmaVariable, LiveOwner,
        SourceBinding,
    };
    let state = &mut session.presentation.owner_mut().unwrap().state;
    let revision = state.revision();
    for binding in state.workspace.bindings.0.values_mut() {
        binding.window = None;
    }
    state.workspace.allocations.insert(901, vec![0; 2]);
    for (role, address, id) in [
        (Role::MessageText, 0x4000, 9901),
        (Role::MessageDescriptor, 0x5000, 9902),
    ] {
        state.workspace.bindings.0.insert(
            role,
            Binding {
                origin: Origin::Allocation(901),
                base: 0,
                extent: 2,
                asset: sc::AssetRef {
                    id: sc::AssetId(id),
                    revision,
                },
                window: Some(Window {
                    bank: 0x7e,
                    address,
                    length: 2,
                }),
            },
        );
    }
    state.workspace.validate(state.revision()).unwrap();
    state.hdma_live = HdmaLiveDescription::new(
        vec![],
        vec![
            HdmaLiveRegion {
                owner: LiveOwner::MessageScanlines,
                source: HdmaRegionSource::Workspace {
                    point: Point::new(Role::MessageText, 0),
                },
                length: 2,
            },
            HdmaLiveRegion {
                owner: LiveOwner::ObjectVariable {
                    object: 0,
                    variable: HdmaVariable::A,
                },
                source: HdmaRegionSource::ObjectVariable {
                    object: 0,
                    variable: HdmaVariable::A,
                },
                length: 2,
            },
        ],
    )
    .unwrap();
    state
        .hdma_live
        .validate_workspace(&state.workspace)
        .unwrap();
    for channel in &mut state.hdma.channels {
        channel.source = SourceBinding::Unbound;
        channel.table = SourceBinding::Unbound;
        channel.active = false;
        channel.dma_active = false;
    }
    for object in &mut state.hdma.objects {
        object.instruction = SourceBinding::Unbound;
    }
    state.hdma.channels[0].source = SourceBinding::LiveOwner {
        owner: LiveOwner::MessageScanlines,
        windows: vec![BankWindow {
            bank: 0x7e,
            address: 0x6000,
            offset: 0,
            length: 2,
        }]
        .into(),
    };
    session.fx_control_ownership = Default::default();
    sm_game::fx_encoding::adopt_control_ownership(session).unwrap();
}

/// Movies cannot alias supplied allocation bytes at 4000/5000 then inject a selected-origin restore.
pub fn persistent_session_workspace_publication_exports_actual_aliases(
    required: ss::DisplayStartRequirements,
    display: ss::DisplayStartDescription,
    media: Arc<sc::SceneMedia>,
    variants: Arc<sc::ObjectVariants>,
) {
    use sm_game::presentation_workspace::{Point, Role};
    use sm_game::scene::hdma_sources::LiveOwner;
    use sm_game::{fx_encoding as fx, persistent_session::ValidatedSessionStart};
    let supplied = persistent_fixture(required, display, media, variants);
    let world = supplied.owner.with(|game| game.world().clone());
    let mut session = ValidatedSessionStart::construct(supplied.supplied, &world)
        .unwrap()
        .open()
        .gameplay;
    workspace_alias_fixture(&mut session);
    let read = |session: &Session| {
        fx::session_source_byte(session, LiveOwner::MessageScanlines, 0).unwrap()
    };
    let mut native = std::collections::BTreeMap::new();
    for (at, value) in [(0x4000, 0x11), (0x5000, 0x22), (0x6000, 0x33)] {
        native.insert(at, value);
        native.insert(at + 1, value + 1);
    }
    fx::adopt_session_sources(
        &mut session,
        &|at| native.get(&at).copied().unwrap_or(0),
        &|at| (0x5000..0x5002).contains(&at).then_some(2),
    )
    .unwrap();
    assert_eq!(read(&session), 0x22);
    let origins = session
        .presentation
        .owner()
        .unwrap()
        .state
        .hdma_live
        .native_origins();
    assert_eq!(
        origins
            .get(&(LiveOwner::MessageScanlines, 0))
            .unwrap()
            .address,
        0x5000
    );
    session
        .publish_workspace_byte(Point::new(Role::MessageDescriptor, 0), 0x7b)
        .unwrap();
    let origins = session
        .presentation
        .owner()
        .unwrap()
        .state
        .hdma_live
        .native_origins();
    assert!(!origins.contains_key(&(LiveOwner::MessageScanlines, 0)));
    assert!(origins.contains_key(&(LiveOwner::MessageScanlines, 1)));
    assert_eq!(read(&session), 0x7b);
    let mut exported = std::collections::BTreeMap::new();
    fx::encode_sources(&session, |at, bytes| {
        for (offset, &byte) in bytes.iter().enumerate() {
            exported.insert(at + offset, byte);
        }
    });
    for at in [0x4000, 0x5000, 0x6000] {
        assert_eq!(exported.get(&at), Some(&0x7b));
        assert_eq!(exported.get(&(at + 1)), Some(&0x23));
    }
    fx::adopt_session_sources(
        &mut session,
        &|at| exported.get(&at).copied().unwrap_or(0),
        &|_| None,
    )
    .unwrap();
    assert_eq!(read(&session), 0x7b);
    fx::validate_control_ownership(&session).unwrap();
}

/// Movies cannot replace the live Workspace binding inside a supplied raster callback.
struct RebindSettlement {
    reached: bool,
}

/// Movies cannot inject an unsupported room callback into this quiescent settlement fixture.
impl sm_game::scene::commit::RoomEffects for RebindSettlement {
    fn follow_transition(
        &mut self,
        _: &mut sm_game::scene::commit::SettlementContext<'_, '_>,
    ) -> Result<(), DisplayError> {
        Err(DisplayError::Source)
    }
}

/// Movies cannot invoke unsupported destination callbacks in this quiescent fixture.
impl sm_game::scene::owned_hdma::Destinations for RebindSettlement {
    fn read_destination(
        &mut self,
        _: &mut sm_game::scene::commit::SettlementContext<'_, '_>,
        _: u8,
    ) -> Result<u8, DisplayError> {
        Err(DisplayError::Source)
    }
    fn write_destination(
        &mut self,
        _: &mut sm_game::scene::commit::SettlementContext<'_, '_>,
        _: u8,
        _: u8,
    ) -> Result<(), DisplayError> {
        Err(DisplayError::Source)
    }
}

/// Movies cannot inject a legal role rebind and deliberately end immediately after its next query.
impl sm_game::scene::commit::SettlementEffects for RebindSettlement {
    fn raster_row(
        &mut self,
        context: &mut sm_game::scene::commit::SettlementContext<'_, '_>,
        _: u16,
    ) -> Result<sc::ObjectContext, DisplayError> {
        use sm_game::presentation_workspace::{Origin, Role};
        use sm_game::scene::hdma_sources::{HdmaVariable, LiveOwner};
        let scalar_source = LiveOwner::ObjectVariable {
            object: 0,
            variable: HdmaVariable::A,
        };
        assert_eq!(context.read_control_byte(0x1914)?, 0x11);
        assert_eq!(context.read_source(scalar_source, 0)?, 0x11);
        let mut bindings = context.state().workspace.bindings.clone();
        let binding = bindings.0.get_mut(&Role::MessageText).unwrap();
        binding.origin = Origin::Allocation(902);
        binding.window = None;
        context.replace_workspace_bindings(bindings)?;
        assert_eq!(context.read_source(LiveOwner::MessageScanlines, 0)?, 0x22);
        assert_eq!(context.read_source(scalar_source, 0)?, 0x22);
        assert_eq!(context.read_control_byte(0x1914)?, 0x22);
        context.write_control_byte(0x1914, 0x7b)?;
        assert_eq!(context.read_source(LiveOwner::MessageScanlines, 0)?, 0x7b);
        self.reached = true;
        Err(DisplayError::Geometry)
    }
}

/// Movies cannot supply independent allocations then replace their role binding during settlement.
pub fn persistent_session_settlement_rebind_queries_current_workspace(
    required: ss::DisplayStartRequirements,
    display: ss::DisplayStartDescription,
    media: Arc<sc::SceneMedia>,
    variants: Arc<sc::ObjectVariants>,
) {
    use sm_game::scene::hdma_sources::{BankWindow, SourceBinding};
    use sm_game::{fx_encoding as fx, persistent_session::ValidatedSessionStart};
    let supplied = persistent_fixture(required, display, media, variants);
    let world = supplied.owner.with(|game| game.world().clone());
    let mut session = ValidatedSessionStart::construct(supplied.supplied, &world)
        .unwrap()
        .open()
        .gameplay;
    workspace_alias_fixture(&mut session);
    let state = &mut session.presentation.owner_mut().unwrap().state;
    state
        .workspace
        .allocations
        .get_mut(&901)
        .unwrap()
        .fill(0x11);
    state.workspace.allocations.insert(902, vec![0x22; 2]);
    let SourceBinding::LiveOwner { windows, .. } = &mut state.hdma.channels[0].source else {
        panic!("constructed live source")
    };
    Arc::make_mut(windows)[0] = BankWindow {
        bank: 0x7e,
        address: 0x1914,
        offset: 0,
        length: 2,
    };
    state.delivery.horizontal_irq_enabled = false;
    state.delivery.vertical_irq_enabled = false;
    state.delivery.nmi_wanted = false;
    session.fx_control_ownership = Default::default();
    fx::adopt_control_ownership(&mut session).unwrap();
    let display = session.presentation.owner_mut().unwrap();
    display.begin_interval(false).unwrap();
    display.refresh().unwrap();
    display.begin_settlement().unwrap();
    let mut effects = RebindSettlement { reached: false };
    assert!(matches!(
        session.settle_display(&mut effects),
        Err(DisplayError::Geometry)
    ));
    assert!(effects.reached);
    assert_eq!(session.presentation.workspace().allocations[&901][0], 0x11);
    assert_eq!(session.presentation.workspace().allocations[&902][0], 0x7b);
}

/// Movies cannot forge supplied FX initialization or ownership masks at the public GameOwner gate.
pub fn persistent_session_game_owner_rejects_forged_fx_admission(
    required: ss::DisplayStartRequirements,
    display: ss::DisplayStartDescription,
    media: Arc<sc::SceneMedia>,
    variants: Arc<sc::ObjectVariants>,
) {
    use sm_game::fx_state::{ControlOwnership, FxState};
    use sm_game::persistent_session::SessionStartError;
    let fixture = || {
        persistent_fixture(
            required.clone(),
            display.clone(),
            media.clone(),
            variants.clone(),
        )
    };
    let valid = fixture();
    valid.owner.start_supplied_session(valid.supplied).unwrap();
    valid.owner.checkpoint_session().unwrap();
    for case in 0..4 {
        let supplied = fixture();
        let (mut start, media, required) = supplied.supplied.into_parts();
        let gameplay = start.gameplay.as_mut().unwrap();
        assert!(!gameplay.presentation.is_owned());
        match case {
            0 => gameplay.fx = FxState::default(),
            1 => {
                gameplay.fx_control_ownership = ControlOwnership {
                    initialized: true,
                    source_lanes: [0; 4],
                }
            }
            2 => {
                gameplay.fx_control_ownership = ControlOwnership {
                    initialized: false,
                    source_lanes: [1, 0, 0, 0],
                }
            }
            _ => {
                gameplay.fx_control_ownership = ControlOwnership {
                    initialized: true,
                    source_lanes: [0, 0, 0, 1 << 63],
                }
            }
        }
        let before = supplied.owner.with(|game| {
            (
                game.compatibility_write_epoch(),
                game.actor_identity().clone(),
                (0..game.ram().len())
                    .map(|at| game.compatibility_byte(at))
                    .collect::<Vec<_>>(),
            )
        });
        let result = supplied
            .owner
            .start_supplied_session(SuppliedSessionStart::new(start, media, required));
        match (case, result) {
            (0, Err(SessionStartError::Binding(DisplayError::Incomplete)))
            | (1..=3, Err(SessionStartError::Binding(DisplayError::Source))) => {}
            _ => panic!("constructed FX admission result"),
        }
        supplied.owner.with(|game| {
            assert!(!game.persistent_session_current());
            assert_eq!(game.compatibility_write_epoch(), before.0);
            assert_eq!(*game.actor_identity(), before.1);
            assert!(
                before
                    .2
                    .iter()
                    .enumerate()
                    .all(|(at, &byte)| { game.compatibility_byte(at) == byte })
            );
        });
    }
}
