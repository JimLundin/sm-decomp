//! Existing verifier assertions and fixtures, separate from persistent state codecs.
#![forbid(unsafe_code)]

use super::*;
use sm::game::message_composition::{prepared::*, *};
use sm::game::scene::component as sd;
use sm::game::scene::hdma_sources as hs;
use sm::game::scene::records as sc;
use sm::game::scene::start as ss;
use sm::game::scene::state as ds;
use sm::rendering::{self as r, IndexedSource, prepared::*};
use std::sync::Arc;

#[cfg(sm_verify_dll)]
pub(super) fn run() -> i32 {
    // Keep names and function pointers paired without duplicating assertions.
    macro_rules! cases {
            ($($test:ident),+ $(,)?) => { [$( (stringify!($test), $test as fn()), )+] };
        }
    let tests: [(&str, fn()); 66] = cases![
        prepared_message_pixels_delay_queue_and_checkpoint,
        prepared_message_rejects_changed_content_and_resets,
        independently_authored_raster_pixels_and_prefixes,
        room_pack_uses_native_header_selection_and_final_block_mirroring,
        room_pack_rejects_checksum_valid_structural_errors,
        room_pack_selection_rejects_missing_duplicate_and_option_values,
        expectation_parser_rejects_unknown_or_malformed_values,
        inventory_events_require_a_new_award_inside_the_same_live_room,
        boss_flags_require_a_rising_bit_in_the_corresponding_area,
        credits_require_enabled_script_advancement_not_ending_presence,
        death_must_begin_in_the_segment_and_reach_game_over,
        every_expectation_is_required_and_repeated_hits_keep_the_first_frame,
        suit_award_waits_for_later_hdma_and_control_return,
        door_requires_fresh_admission_and_the_same_completed_destination,
        elevator_landing_requires_the_linked_traversal_and_fresh_unlock,
        ceres_requires_observed_combat_and_the_same_encounter_escape_edge,
        bomb_requires_observed_damage_death_and_later_completion,
        bomb_deletion_only_finishes_an_already_observed_death,
        bomb_resets_on_room_actor_or_gameplay_discontinuity,
        bomb_does_not_reuse_an_episode_after_healing_or_completion,
        fresh_save_death_selected_restore_and_initialized_arrival_complete_once,
        loaded_history_and_unordered_restore_do_not_qualify,
        reset_and_slot_change_discard_save_history,
        state_rollbacks_discard_episode,
        wrong_first_arrival_cannot_be_repaired_by_later_room_entry,
        restoration_and_gameplay_initialization_are_required,
        scene_pixels_preserve_line_incarnations_and_redraw,
        scene_rejects_missing_inconsistent_and_unsupported_history,
        scene_media_rejects_missing_revisions_and_stale_aliases,
        scene_rejects_reference_quantization,
        scene_start_rejects_missing_fields_and_invalid_history_prefix,
        scene_start_rejects_changed_profile_and_audio_prefix,
        scene_component_preserves_complete_open_checkpoint_and_reopen,
        scene_component_reset_preserves_complete_owned_buffers_and_alias_views,
        scene_component_rejects_missing_resident_history_and_media_replacement,
        scene_component_failed_source_replacement_preserves_complete_owner,
        scene_component_rejects_missing_final_resident_assets,
        scene_component_rejects_each_native_resident_latch_alias,
        scene_component_rejects_missing_hdma_catalog,
        scene_component_rejects_changed_hdma_payload_under_same_fingerprint,
        scene_component_rejects_wrong_hdma_kinds_and_required_unbound_sources,
        scene_component_rejects_invalid_hdma_bank_windows,
        scene_component_admits_explicitly_disabled_hdma_sources,
        scene_component_rejects_conflicting_physical_hdma_aliases,
        scene_component_preserves_coherent_physical_hdma_aliases_on_reopen,
        scene_component_reset_preserves_coherent_physical_hdma_aliases,
        scene_component_preserves_schema3_delivery_effective_and_written_selection,
        scene_component_rejects_schema2_and_invalid_effective_color_selectors,
        scene_atmospheric_publication_retains_x_high_and_reuses_lifecycle,
        scene_atmospheric_settlement_delivers_and_checkpoints_visible_history,
        scene_atmospheric_settlement_poison_preserves_numeric_prefix,
        scene_namespace_interleaves_birth_samus_retirement_and_reuse,
        scene_namespace_birth_failures_preserve_counter_and_slot_prefixes,
        scene_namespace_samus_id_failure_keeps_completed_oam_and_other_records,
        scene_namespace_marker_retains_high_x_and_overlapping_word_prefixes,
        scene_samus_landing_native_words_preserve_full_frame_bounds,
        scene_samus_signed_tables_preserve_every_entry_and_bounds,
        scene_samus_fallback_and_frame_thresholds_keep_full_width,
        scene_samus_ceres_success_restores_y_then_x,
        scene_samus_ceres_failures_preserve_each_completed_prefix,
        scene_samus_upload_disabled_halves_still_publish_vmain,
        scene_samus_upload_empty_second_prepares_destination_after_first_words,
        scene_samus_upload_faults_retain_words_latches_and_poison,
        scene_resident_identity_and_full_extent_reject_before_aliases,
        scene_resident_media_failure_keeps_alias_prefix_and_prior_ids,
        scene_resident_append_adopts_supplied_ids_and_keeps_old_lease,
    ];
    let mut failed = 0usize;
    for (name, test) in &tests {
        let passed = std::panic::catch_unwind(*test).is_ok();
        println!(
            "test verify::coverage_checks::{name} ... {}",
            if passed { "ok" } else { "FAILED" }
        );
        failed += usize::from(!passed);
    }
    println!(
        "Verifier unit tests: {} passed; {failed} failed; {} total",
        tests.len() - failed,
        tests.len()
    );
    i32::from(failed != 0)
}

// Movies cannot author portable media identities or complete per-line incarnations.
// This synthetic display fixture reuses the accepted message font, map and colors;
// it supplies no complete G start, Reference capture or native audio evidence.
fn scene_fixture() -> (sc::SceneDescription, Arc<sc::SceneMedia>) {
    let (_, prepared) = prepared_message_fixture(0x3801);
    let revision = sc::Revision([1; 32]);
    let reference = |id| sc::AssetRef {
        id: sc::AssetId(id),
        revision,
    };
    let image_ref = |incarnation| sc::MediaRef {
        asset: reference(1),
        incarnation,
    };
    let palette_ref = |incarnation| sc::MediaRef {
        asset: reference(6),
        incarnation,
    };
    let saved = &prepared.saved.media;
    let colors = core::array::from_fn(|index| saved.palette(index as u8).unwrap());
    let restored_colors =
        core::array::from_fn(|index| prepared.restored.media.palette(index as u8).unwrap());
    let mut words: Vec<u16> = (0..0x8000)
        .map(|index| saved.word(index).unwrap())
        .collect();
    // Explicit authored latch words and the required 26 palette aliases.
    words.extend([0; 5]);
    words.extend_from_slice(&colors[..26]);
    let mut later_words = words.clone();
    for word in &mut later_words[0x5880..0x5c00] {
        *word ^= 3;
    }
    later_words[0x8005..0x801f].copy_from_slice(&restored_colors[..26]);
    let cells = |image: &[u16], first: usize| -> Arc<[sc::TileCell]> {
        image[first..first + 1024]
            .iter()
            .map(|word| sc::TileCell {
                tile: word & 0x3ff,
                palette: (word >> 10 & 7) as u8,
                priority: word & 0x2000 != 0,
                flip_horizontal: word & 0x4000 != 0,
                flip_vertical: word & 0x8000 != 0,
            })
            .collect::<Vec<_>>()
            .into()
    };
    let blank_cells = cells(&words, 0x5000);
    let hud_cells = cells(&words, 0x5880);
    let later_cells = cells(&later_words, 0x5880);
    let assets = vec![
        sc::SceneAsset::Room(reference(1)),
        sc::SceneAsset::Atlas(sc::TileAtlas {
            reference: reference(2),
            encoding: sc::TileEncoding::Planar4,
            tile_count: 1,
            data: vec![0; 16].into(),
        }),
        sc::SceneAsset::Atlas(sc::TileAtlas {
            reference: reference(3),
            encoding: sc::TileEncoding::Planar2,
            tile_count: 256,
            data: words[0x4000..0x4800].to_vec().into(),
        }),
        sc::SceneAsset::Map(sc::Tilemap {
            reference: reference(4),
            atlas: reference(2),
            palette: reference(6),
            width: 32,
            height: 32,
            cells: blank_cells.clone(),
        }),
        sc::SceneAsset::Map(sc::Tilemap {
            reference: reference(5),
            atlas: reference(3),
            palette: reference(6),
            width: 32,
            height: 32,
            cells: hud_cells.clone(),
        }),
        sc::SceneAsset::Palette(sc::Palette {
            reference: reference(6),
            colors,
        }),
        sc::SceneAsset::Visual(sc::Visual {
            reference: reference(7),
            frames: vec![sc::VisualFrame {
                parts: vec![sc::VisualPart {
                    atlas: reference(2),
                    material: reference(6),
                    tile: 0,
                    offset: sc::ScreenPosition { x: 0, y: 0 },
                    size: sc::ObjectSize::Small,
                }]
                .into(),
            }]
            .into(),
        }),
        sc::SceneAsset::Font(sc::Font {
            reference: reference(8),
            atlas: reference(3),
        }),
    ];
    let resident = assets
        .iter()
        .map(sc::SceneAsset::reference)
        .collect::<Vec<_>>()
        .into();
    let atlases: Arc<[sc::AtlasBinding]> = vec![
        sc::AtlasBinding {
            atlas: reference(2),
            first_word: 0,
            resident_data: words[0..16].to_vec().into(),
        },
        sc::AtlasBinding {
            atlas: reference(3),
            first_word: 0x4000,
            resident_data: words[0x4000..0x4800].to_vec().into(),
        },
    ]
    .into();
    let maps: Arc<[sc::MapBinding]> = vec![
        sc::MapBinding {
            map: reference(4),
            first_word: 0x5000,
        },
        sc::MapBinding {
            map: reference(5),
            first_word: 0x5880,
        },
    ]
    .into();
    let images = vec![
        sc::CompiledImage {
            reference: image_ref(1),
            palette: palette_ref(1),
            words: words.into(),
            atlases: atlases.clone(),
            maps: maps.clone(),
        },
        sc::CompiledImage {
            reference: image_ref(2),
            palette: palette_ref(2),
            words: later_words.into(),
            atlases,
            maps,
        },
    ];
    let palettes = vec![
        sc::PaletteIncarnation {
            reference: palette_ref(1),
            colors,
        },
        sc::PaletteIncarnation {
            reference: palette_ref(2),
            colors: restored_colors,
        },
    ];
    let media = Arc::new(sc::SceneMedia::new(revision, assets, palettes, images).unwrap());
    let objects: Arc<[sc::SceneObject]> = (0..128)
        .map(|order| sc::SceneObject {
            order,
            visual: reference(7),
            frame: 0,
            part: 0,
            atlas: reference(2),
            material: reference(6),
            tile: 0,
            position: sc::ScreenPosition { x: -256, y: 0 },
            size: sc::ObjectSize::Small,
            palette: 0,
            priority: 0,
            flip_horizontal: false,
            flip_vertical: false,
        })
        .collect::<Vec<_>>()
        .into();
    let plane = |role, map, atlas, cells, palette| sc::ScenePlane {
        role,
        map: reference(map),
        atlas: reference(atlas),
        palette,
        width: 32,
        height: 32,
        cells,
        scroll: sc::ScreenPosition { x: 0, y: -1 },
    };
    let lines = (0..224)
        .map(|line| {
            let incarnation = if line < 112 { 1 } else { 2 };
            let palette = palette_ref(incarnation);
            sc::SceneLine {
                media: image_ref(incarnation),
                palette,
                mode: sc::SceneMode::TilePlanes,
                blank: false,
                planes: [
                    plane(sc::PlaneRole::Main, 4, 2, blank_cells.clone(), palette),
                    plane(sc::PlaneRole::Secondary, 4, 2, blank_cells.clone(), palette),
                    plane(
                        sc::PlaneRole::HudFx,
                        5,
                        3,
                        if line < 112 {
                            hud_cells.clone()
                        } else {
                            later_cells.clone()
                        },
                        palette,
                    ),
                ],
                objects: objects.clone(),
                object_selection: sc::ObjectSelection {
                    atlases: [reference(2); 2],
                    size_set: 0,
                    first_order: None,
                    interlaced: false,
                    even_frame: false,
                },
                windows: sc::SceneWindows {
                    extents: [sc::WindowExtent { first: 0, last: 0 }; 2],
                    layers: [sc::LayerWindow {
                        first_enabled: false,
                        first_inverted: false,
                        second_enabled: false,
                        second_inverted: false,
                        combination: sc::WindowCombination::Union,
                    }; 6],
                },
                routing: [
                    sc::ScreenRouting {
                        visible: [false, false, true, false, false],
                        masked: [false; 5],
                    },
                    sc::ScreenRouting {
                        visible: [false; 5],
                        masked: [false; 5],
                    },
                ],
                affine: sc::SceneAffine {
                    matrix: [0; 4],
                    center: sc::ScreenPosition { x: 0, y: 0 },
                    scroll: sc::ScreenPosition { x: 0, y: 0 },
                    flip_horizontal: false,
                    flip_vertical: false,
                    bounded_field: false,
                    fill_outside: false,
                },
                pixelation: sc::ScenePixelation {
                    enabled: [false; 3],
                    size: 1,
                    origin: 1,
                    modulo: [0; 256],
                },
                color: sc::SceneColor {
                    clip: sc::WindowCondition::Never,
                    prevent_math: sc::WindowCondition::Never,
                    enabled: [false; 6],
                    use_subscreen: false,
                    subtract: false,
                    half: false,
                    fixed: [0; 3],
                    brightness: 15,
                },
                object_aliases: Arc::from([]),
            }
        })
        .collect::<Vec<_>>()
        .into();
    (
        sc::SceneDescription {
            tick: 7,
            gameplay: [2; 32],
            reference: [3; 32],
            revision,
            view: sc::SceneView {
                room: reference(1),
                camera: sc::ScreenPosition { x: 17, y: -3 },
            },
            geometry: sc::SceneGeometry {
                width: 256,
                height: 224,
                crop_origin: sc::ScreenPosition { x: 0, y: 0 },
                interlaced: false,
                even_frame: false,
                overscan: false,
                color: sc::ColorInterpretation::Bgr15Native,
                quantization: sc::SceneQuantization::Native,
            },
            resident,
            lines,
            status: sc::DisplayStatus {
                range_overflow: false,
                time_overflow: false,
                has_objects: false,
                even_frame: false,
                interlaced: false,
                overscan: false,
            },
            initial_object_overflow: sc::ObjectOverflow {
                range: false,
                time: false,
            },
        },
        media,
    )
}

// Movies cannot reach authored portable media/incarnation changes or prove redraw independence.
#[cfg_attr(test, test)]
fn scene_pixels_preserve_line_incarnations_and_redraw() {
    let (description, media) = scene_fixture();
    let scene = sc::Scene::new(description.clone(), media.clone()).unwrap();
    let before = scene.clone();
    let frame = scene.render().unwrap();
    assert_eq!((frame.tick, frame.revision), (7, description.revision));
    assert_eq!(frame.pixels.len(), 224 * 256);
    for (index, pixel) in frame.pixels.iter().enumerate() {
        let band = index % 256 / 8 % 2;
        let expected = if band == 0 {
            [0, 255, 0, 0]
        } else if index / 256 < 112 {
            [0, 0, 255, 0]
        } else {
            [255, 0, 0, 0]
        };
        assert_eq!(*pixel, expected, "Scene pixel {index}");
    }
    let mut later = description;
    later.tick = 8;
    let last = later.lines[223].clone();
    Arc::make_mut(&mut later.lines).fill(last);
    let replacement = sc::Scene::new(later, media.clone()).unwrap();
    assert_ne!(replacement.render().unwrap().pixels, frame.pixels);
    drop(media);
    assert_eq!(scene.render().unwrap(), frame);
    assert_eq!(scene, before);
    assert_eq!(scene.description(), before.description());
    assert_eq!(scene.lines()[0].media.incarnation, 1);
    assert_eq!(scene.lines()[223].palette.incarnation, 2);
    assert_eq!(
        frame.raster_overflow,
        r::Overflow {
            range: false,
            time: false
        }
    );
}

// Movies cannot reach authored portable missing media, inconsistent history or unsupported starts.
#[cfg_attr(test, test)]
fn scene_rejects_missing_inconsistent_and_unsupported_history() {
    let (description, media) = scene_fixture();
    for case in 0..9 {
        let mut changed = description.clone();
        let expected = match case {
            0 => {
                changed.lines = changed.lines[..223].to_vec().into();
                sc::SceneError::Incomplete
            }
            1 => {
                changed.resident = changed
                    .resident
                    .iter()
                    .copied()
                    .filter(|asset| asset.id != sc::AssetId(7))
                    .collect::<Vec<_>>()
                    .into();
                sc::SceneError::MissingAsset(sc::AssetId(7))
            }
            2 => {
                Arc::make_mut(&mut changed.lines)[0].media.incarnation = 99;
                sc::SceneError::MissingIncarnation(changed.lines[0].media)
            }
            3 => {
                Arc::make_mut(&mut changed.lines)[0].palette.incarnation = 99;
                sc::SceneError::MissingIncarnation(changed.lines[0].palette)
            }
            4 => {
                Arc::make_mut(&mut changed.lines)[0].palette = changed.lines[223].palette;
                sc::SceneError::InconsistentMedia(changed.lines[0].media)
            }
            5 => {
                Arc::make_mut(&mut changed.lines)[0].planes[0].role = sc::PlaneRole::HudFx;
                sc::SceneError::Order
            }
            6 => {
                let objects = &mut Arc::make_mut(&mut changed.lines)[0].objects;
                Arc::make_mut(objects)[0].order = 127;
                sc::SceneError::Order
            }
            7 => {
                Arc::make_mut(&mut changed.lines)[0].mode =
                    sc::SceneMode::AffinePlane { extended: true };
                sc::SceneError::ExtendedBackground
            }
            _ => {
                Arc::make_mut(&mut changed.lines)[0].pixelation.enabled[0] = true;
                sc::SceneError::EnabledMosaic(0)
            }
        };
        assert_eq!(
            sc::Scene::new(changed, media.clone()).err(),
            Some(expected),
            "Scene rejection {case}"
        );
    }
    let mut changed = description.clone();
    changed.geometry.width = 255;
    assert_eq!(
        sc::Scene::new(changed, media.clone()).err(),
        Some(sc::SceneError::Geometry)
    );
    let mut changed = description;
    changed.gameplay = [0; 32];
    assert_eq!(
        sc::Scene::new(changed, media).err(),
        Some(sc::SceneError::Identity)
    );
}

// Movies cannot reach authored portable registry/revision/start rejection or stale palette aliases.
#[cfg_attr(test, test)]
fn scene_media_rejects_missing_revisions_and_stale_aliases() {
    let (_, media) = scene_fixture();
    for case in 0..7 {
        let mut assets = media.assets().cloned().collect::<Vec<_>>();
        let mut palettes = media.palette_incarnations().cloned().collect::<Vec<_>>();
        let mut images = media.compiled_images().cloned().collect::<Vec<_>>();
        let expected = match case {
            0 => {
                assets.retain(|asset| asset.reference().id != sc::AssetId(6));
                sc::SceneError::MissingAsset(sc::AssetId(6))
            }
            1 => {
                palettes[0].reference.asset.revision = sc::Revision([99; 32]);
                sc::SceneError::Revision
            }
            2 => {
                images.push(images[0].clone());
                sc::SceneError::DuplicateIncarnation(images[0].reference)
            }
            3 => {
                palettes.remove(0);
                sc::SceneError::MissingIncarnation(images[0].palette)
            }
            4 => {
                images[0].words = images[0].words[..r::MAX_INDEXED_WORDS - 1].to_vec().into();
                sc::SceneError::Incomplete
            }
            5 => {
                Arc::make_mut(&mut images[1].words)[0x8005] ^= 1;
                sc::SceneError::InconsistentMedia(images[1].reference)
            }
            _ => {
                Arc::make_mut(&mut images[0].words)[0x4008] ^= 1;
                sc::SceneError::InconsistentMedia(images[0].reference)
            }
        };
        assert_eq!(
            sc::SceneMedia::new(media.revision(), assets, palettes, images).err(),
            Some(expected),
            "media rejection {case}"
        );
    }
}

// Movies cannot reach authored portable Reference quantization/start rejection.
// SOURCE01's Scene consumer admits Native and cannot claim Reference image support.
#[cfg_attr(test, test)]
fn scene_rejects_reference_quantization() {
    let (mut description, media) = scene_fixture();
    description.geometry.quantization = sc::SceneQuantization::Reference;
    assert_eq!(
        sc::Scene::new(description, media).err(),
        Some(sc::SceneError::Unsupported)
    );
}

// Movies cannot author a separate complete Large visual for sealed Samus content.
fn scene_start_samus_media(media: Arc<sc::SceneMedia>) -> Arc<sc::SceneMedia> {
    let reference = |id| sc::AssetRef {
        id: sc::AssetId(id),
        revision: media.revision(),
    };
    let mut assets = media.assets().cloned().collect::<Vec<_>>();
    assets.push(sc::SceneAsset::Atlas(sc::TileAtlas {
        reference: reference(15),
        encoding: sc::TileEncoding::Planar4,
        tile_count: 18,
        data: vec![0x00ff; 18 * 16].into(),
    }));
    assets.push(sc::SceneAsset::Visual(sc::Visual {
        reference: reference(16),
        frames: Arc::from([sc::VisualFrame {
            parts: Arc::from([sc::VisualPart {
                atlas: reference(15),
                material: reference(6),
                tile: 0,
                offset: sc::ScreenPosition { x: -3, y: -8 },
                size: sc::ObjectSize::Large,
            }]),
        }]),
    }));
    Arc::new(
        sc::SceneMedia::new(
            media.revision(),
            assets,
            media.palette_incarnations().cloned().collect(),
            media.compiled_images().cloned().collect(),
        )
        .unwrap(),
    )
}

// Movies cannot supply authored full-width Samus pointers and portable part identities.
fn scene_start_samus_catalog(revision: sc::Revision) -> Arc<sm::game::scene::samus::SamusCatalog> {
    use sm::game::scene::samus::{SamusCatalog, SamusMap, SamusPart};
    Arc::new(SamusCatalog {
        revision,
        pointers: Arc::from([0x8101]),
        maps: [(
            0x8101,
            SamusMap {
                count: 1,
                parts: Arc::from([SamusPart {
                    x: 0xfffd,
                    y: 0xf8,
                    native_word: 0x3400,
                    variants: Arc::new(sc::ObjectVariants {
                        size_sets: [(
                            0,
                            sc::ObjectSizeVariants {
                                small: sc::ObjectPartBinding {
                                    visual: sc::AssetRef {
                                        id: sc::AssetId(7),
                                        revision,
                                    },
                                    frame: 0,
                                    part: 0,
                                },
                                large: sc::ObjectPartBinding {
                                    visual: sc::AssetRef {
                                        id: sc::AssetId(16),
                                        revision,
                                    },
                                    frame: 0,
                                    part: 0,
                                },
                            },
                        )]
                        .into(),
                    }),
                }]),
            },
        )]
        .into(),
    })
}

// Movies cannot supply the independently authored expected sealed Samus catalog.
fn scene_expected_start_samus_catalog() -> Arc<sm::game::scene::samus::SamusCatalog> {
    use sm::game::scene::samus::{SamusCatalog, SamusMap, SamusPart};
    Arc::new(SamusCatalog {
        revision: sc::Revision([1; 32]),
        pointers: Arc::from([0x8101]),
        maps: [(
            0x8101,
            SamusMap {
                count: 1,
                parts: Arc::from([SamusPart {
                    x: 0xfffd,
                    y: 0xf8,
                    native_word: 0x3400,
                    variants: Arc::new(sc::ObjectVariants {
                        size_sets: [(
                            0,
                            sc::ObjectSizeVariants {
                                small: sc::ObjectPartBinding {
                                    visual: sc::AssetRef {
                                        id: sc::AssetId(7),
                                        revision: sc::Revision([1; 32]),
                                    },
                                    frame: 0,
                                    part: 0,
                                },
                                large: sc::ObjectPartBinding {
                                    visual: sc::AssetRef {
                                        id: sc::AssetId(16),
                                        revision: sc::Revision([1; 32]),
                                    },
                                    frame: 0,
                                    part: 0,
                                },
                            },
                        )]
                        .into(),
                    }),
                }]),
            },
        )]
        .into(),
    })
}

// Movies cannot author portable start/profile rejection. This synthetic metadata
// prefix intentionally omits DisplayDescription and cannot seal a G or display start.
// It exercises only the concrete constructor's admission before MissingDisplay.
fn scene_start_prefix() -> (
    ss::DisplayStartRequirements,
    ss::DisplayStartDescription,
    Arc<sc::SceneMedia>,
) {
    let (mut scene, media) = scene_fixture();
    let media = scene_start_samus_media(media);
    scene.tick = 0;
    let reference = ss::ReferenceProfile {
        fingerprint: scene.reference,
        release: [11; 32],
        capture: [12; 32],
        ticks_numerator: 37,
        ticks_denominator: 5,
        arithmetic_policy: [13; 32],
        input_policy: [14; 32],
        observation_policy: [15; 32],
        color_policy: [16; 32],
        slowdown_policy: [17; 32],
        glitch_policy: [18; 32],
        audio_timing_policy: [19; 32],
        image: ss::SceneImagePolicy {
            width: scene.geometry.width,
            height: scene.geometry.height,
            crop_origin: scene.geometry.crop_origin,
            color: scene.geometry.color,
            quantization: scene.geometry.quantization,
        },
    };
    let identity = ss::StartIdentity {
        schema: ss::DISPLAY_START_SCHEMA,
        gameplay: scene.gameplay,
        reference: scene.reference,
        capture: [4; 32],
        media: scene.revision,
        committed_tick: 0,
        next_input_record: 23,
    };
    let input_history = ss::InputHistory {
        capture: identity.capture,
        policy: reference.input_policy,
        ports: [ss::PortHistory {
            sampled: 3,
            previous: 1,
            pressed: 2,
            repeated: 0,
            repeat_counter: 17,
        }; 2],
        debug_second_port: true,
        repeat_first: 31,
        repeat_next: 7,
        released: 4,
        timed_held: 3,
        previous_timed_held: 1,
        newly_held: 2,
        held_timer: 13,
        held_timer_reset: 29,
        samus_held: 3,
        samus_pressed: 2,
        bindings: ss::ActionBindings {
            shoot: 1,
            jump: 2,
            run: 4,
            cancel: 8,
            switch: 16,
            aim_down: 32,
            aim_up: 64,
        },
        bug_fix_counter: 9,
    };
    let continuation = ss::ContinuationHistory {
        capture: identity.capture,
        observation_policy: reference.observation_policy,
        phase: ss::CommittedPhase::InitialObservation,
        next_input_record: identity.next_input_record,
        outer: 18,
        primary: 3,
        secondary: 7,
        music: 2,
        message: 4,
        auxiliary: 1,
        captured_mode: Some(8),
    };
    let audio_revision = sc::Revision([5; 32]);
    let declarations: Arc<[ss::AudioProgramDeclaration]> = vec![ss::AudioProgramDeclaration {
        program: sc::AssetId(1),
        bank: sc::AssetId(2),
        revision: audio_revision,
        definition: [6; 32],
    }]
    .into();
    let audio = ss::InitialAudioIntent {
        capture: identity.capture,
        revision: audio_revision,
        timing_policy: reference.audio_timing_policy,
        declarations: declarations.clone(),
        music: ss::OngoingMusic::Playing(ss::ProgramAnchor {
            program: sc::AssetId(1),
            revision: audio_revision,
            source_interval: 23,
            phase: 59,
            timing_policy: reference.audio_timing_policy,
        }),
        ambience: Arc::from([]),
    };
    let requirements = ss::DisplayStartRequirements {
        schema: identity.schema,
        gameplay: identity.gameplay,
        hdma_content: Arc::new(hs::HdmaCatalog::new(identity.gameplay, vec![]).unwrap()),
        atmosphere_content: scene_atmosphere_catalog(identity.media),
        samus_content: scene_expected_start_samus_catalog(),
        capture: identity.capture,
        reference,
        media: identity.media,
        audio_revision,
        audio_programs: declarations,
    };
    // The fixture has no atmospheric publications on any of its 224 visible lines.
    let producer_history: Arc<[Arc<[sc::ProducedObject]>]> = vec![Arc::from([]); 224].into();
    let description = ss::DisplayStartDescription {
        identity: Some(identity),
        reference: Some(reference),
        input_history: Some(input_history),
        continuation: Some(continuation),
        audio: Some(audio),
        display: None,
        resident: None,
        hdma_content: None,
        samus_content: Some(scene_start_samus_catalog(media.revision())),
        committed_scene: Some(
            sc::Scene::new(scene, media.clone())
                .unwrap()
                .with_producer_history(producer_history)
                .unwrap(),
        ),
    };
    (requirements, description, media)
}

// Movies cannot reach authored portable missing start fields or malformed history admission.
#[cfg_attr(test, test)]
fn scene_start_rejects_missing_fields_and_invalid_history_prefix() {
    let (requirements, description, media) = scene_start_prefix();
    assert!(matches!(
        ss::ValidatedDisplayStart::construct(description, media, &requirements),
        Err(ss::StartError::MissingDisplay)
    ));
    for case in 0..12 {
        let (requirements, mut description, media) = scene_start_prefix();
        let expected = match case {
            0 => {
                description.identity = None;
                ss::StartError::MissingIdentity
            }
            1 => {
                description.reference = None;
                ss::StartError::MissingReference
            }
            2 => {
                description.input_history = None;
                ss::StartError::MissingInputHistory
            }
            3 => {
                description.continuation = None;
                ss::StartError::MissingContinuation
            }
            4 => {
                description.audio = None;
                ss::StartError::MissingAudioAnchor
            }
            5 => {
                description.identity.as_mut().unwrap().schema += 1;
                ss::StartError::Schema
            }
            6 => {
                description.input_history.as_mut().unwrap().capture = [99; 32];
                ss::StartError::InputHistory
            }
            7 => {
                description.continuation.as_mut().unwrap().next_input_record += 1;
                ss::StartError::Continuation
            }
            8 => {
                description.identity.as_mut().unwrap().committed_tick = 1;
                ss::StartError::Continuation
            }
            9 => {
                description.continuation.as_mut().unwrap().outer = 4;
                ss::StartError::Continuation
            }
            10 => {
                description.continuation.as_mut().unwrap().captured_mode = Some(44);
                ss::StartError::Continuation
            }
            _ => {
                let history = description.continuation.as_mut().unwrap();
                history.outer = 0;
                history.captured_mode = Some(246);
                ss::StartError::Continuation
            }
        };
        let error = ss::ValidatedDisplayStart::construct(description, media, &requirements)
            .err()
            .unwrap();
        assert_eq!(
            std::mem::discriminant(&error),
            std::mem::discriminant(&expected),
            "start prefix rejection {case}: {error:?}"
        );
    }
}

// Movies cannot reach authored portable changed-profile or missing audio-anchor rejection.
#[cfg_attr(test, test)]
fn scene_start_rejects_changed_profile_and_audio_prefix() {
    for case in 0..12 {
        let (requirements, mut description, media) = scene_start_prefix();
        let profile = description.reference.as_mut().unwrap();
        match case {
            0 => profile.ticks_numerator += 1,
            1 => profile.release[0] ^= 1,
            2 => profile.capture[0] ^= 1,
            3 => profile.arithmetic_policy[0] ^= 1,
            4 => profile.input_policy[0] ^= 1,
            5 => profile.observation_policy[0] ^= 1,
            6 => profile.color_policy[0] ^= 1,
            7 => profile.slowdown_policy[0] ^= 1,
            8 => profile.glitch_policy[0] ^= 1,
            9 => profile.audio_timing_policy[0] ^= 1,
            10 => profile.image.width -= 1,
            _ => profile.image.quantization = sc::SceneQuantization::Reference,
        }
        assert_eq!(profile.fingerprint, requirements.reference.fingerprint);
        assert!(matches!(
            ss::ValidatedDisplayStart::construct(description, media, &requirements),
            Err(ss::StartError::Fingerprint)
        ));
    }
    for case in 0..4 {
        let (requirements, mut description, media) = scene_start_prefix();
        let audio = description.audio.as_mut().unwrap();
        match case {
            0 => audio.capture = [99; 32],
            1 => audio.timing_policy = [99; 32],
            2 => {
                if let ss::OngoingMusic::Playing(anchor) = &mut audio.music {
                    anchor.source_interval += 1;
                }
            }
            _ => audio.declarations = Arc::from([]),
        }
        let error = ss::ValidatedDisplayStart::construct(description, media, &requirements)
            .err()
            .unwrap();
        assert!(matches!(
            (case, error),
            (0..=2, ss::StartError::MissingAudioAnchor) | (3, ss::StartError::AudioDeclaration)
        ));
    }
}

// Movies cannot supply an independently authored complete saved display.
fn prepared_message_line() -> DisplayLine {
    let background = r::Background {
        horizontal_scroll: 0,
        vertical_scroll: u16::MAX,
        map_words: 0x5880,
        tile_words: 0x4000,
        wider: false,
        higher: false,
    };
    let window = r::Window {
        first_enabled: false,
        first_inverted: false,
        second_enabled: false,
        second_inverted: false,
        logic: 0,
    };
    DisplayLine {
        mode: 1,
        extended_background: false,
        forced_blank: false,
        backgrounds: [background; 3],
        windows: r::Windows {
            first: [0; 2],
            second: [0; 2],
            native_flags: 0,
            layers: [window; 6],
        },
        routing: r::Routing {
            enabled: [4, 0],
            windowed: [0; 2],
        },
        affine: r::Affine {
            matrix: [0; 8],
            horizontal_flip: false,
            vertical_flip: false,
            large_field: false,
            character_fill: false,
        },
        color: r::ColorMath {
            clip: 0,
            prevent: 0,
            enabled: [false; 6],
            add_subscreen: false,
            subtract: false,
            half: false,
            fixed: [0; 3],
            brightness: 15,
        },
    }
}

// Movies cannot author complete font/media identities or differing restored palettes.
fn prepared_message_fixture(text: u16) -> (Arc<Definitions>, PreparedStart) {
    let revision = Revision([1; 32]);
    let mut font = vec![0; 0x800];
    font[8..16].fill(0xff00);
    font[16..24].fill(0x00ff);
    let message = MessageDefinition {
        id: MessageId(1),
        kind: MessageKind::Content,
        size: BoxSize::Small,
        modifier: ButtonModifier::None,
        button_byte_offset: None,
        text: (0..32).map(|i| text + (i & 1)).collect::<Vec<_>>().into(),
    };
    let map = MessageDefinition {
        id: MessageId(2),
        kind: MessageKind::Map,
        size: BoxSize::Large,
        ..message.clone()
    };
    let definition = FontDefinition {
        id: AssetId(1),
        revision,
        first_word: 0x4000,
        words: font.clone().into(),
    };
    let definitions = Arc::new(Definitions::new(revision, definition, vec![message, map]).unwrap());
    let image = |restored: bool| {
        let mut words = vec![0; 0x8000];
        words[0x4000..0x4800].copy_from_slice(&font);
        for (i, word) in words[0x5880..0x5c00].iter_mut().enumerate() {
            *word = if restored {
                14
            } else {
                0x3801 + (i & 1) as u16
            };
        }
        let mut palette = [0; 256];
        palette[25] = if restored { 31 << 5 } else { 31 };
        palette[26] = if restored { 31 << 10 } else { 31 << 5 };
        Arc::new(IndexedImage::new(words, palette).unwrap())
    };
    let saved = FullDisplay {
        media_id: AssetId(2),
        revision,
        media: image(false),
        sprites: Arc::new(PreparedSprites {
            words: [0; 256],
            extension: [0x55; 32],
            tile_words: [0; 2],
            size: 0,
            rotation: None,
            interlace: false,
            even_frame: false,
        }),
        mosaic: Arc::new(PreparedMosaic {
            native_enabled: 0,
            background0_enabled: false,
            size: 1,
            start_line: 1,
            modulo: [0; 256],
        }),
        lines: Box::new([prepared_message_line(); 224]),
        overflow: r::Overflow {
            range: false,
            time: false,
        },
    };
    let restored = FullDisplay {
        media_id: AssetId(4),
        media: image(true),
        ..saved.clone()
    };
    let mut active = saved.clone();
    active.media_id = AssetId(3);
    for line in active.lines.iter_mut() {
        line.backgrounds[2].map_words = 0x5800;
    }
    (
        definitions,
        PreparedStart {
            saved,
            active,
            restored,
            tile_workspace: [0; 768],
            shoot_button: 0x80,
            run_button: 0x20,
            initial_interval: 0,
            initial_wait_pending: false,
        },
    )
}

fn prepared_message_pixels(
    frame: &MessageFrame,
    bands: [[u8; 4]; 2],
    rows: std::ops::Range<usize>,
) {
    assert_eq!(frame.pixels.len(), 224 * 256);
    for (i, pixel) in frame.pixels.iter().enumerate() {
        let expected = if rows.contains(&(i / 256)) {
            bands[i % 256 / 8 % 2]
        } else {
            [0; 4]
        };
        assert_eq!(*pixel, expected, "prepared pixel {i}");
    }
}

fn prepared_message_waits(
    owner: &mut PreparedOwner,
    sample: InputSample,
    intervals: std::ops::RangeInclusive<u64>,
) -> StepOutput {
    intervals
        .map(|interval| {
            let step = owner.resume(interval, sample).unwrap();
            assert_eq!(step.outcome, Outcome::Suspended);
            assert_eq!(step.effects, [Effect::AdvanceAudio]);
            step
        })
        .last()
        .unwrap()
}

// Shared movie pixels cannot prove saved-map recovery, 360 waits or stale-completion ownership.
#[cfg_attr(test, test)]
fn prepared_message_pixels_delay_queue_and_checkpoint() {
    let (definitions, start) = prepared_message_fixture(0x3801);
    let (mut owner, frame) = PreparedOwner::open(definitions.clone(), start.clone()).unwrap();
    prepared_message_pixels(&frame, [[0, 255, 0, 0], [0, 0, 255, 0]], 0..224);
    let sample = InputSample {
        new_keys: 1,
        held_keys: 0,
        bug_fix_counter: 1,
    };
    assert_eq!(owner.resume(1, sample).err(), Some(Fault::NoRequest));
    owner.queue(MessageId(1)).unwrap();
    let mut step = owner.resume(1, sample).unwrap();
    assert_eq!(step.outcome, Outcome::Suspended);
    assert_eq!(step.effects, [Effect::CancelSounds]);
    prepared_message_pixels(&step.frame, [[0; 4]; 2], 0..224);
    let checkpoint = owner.checkpoint().unwrap();
    (owner, _) = PreparedOwner::reopen(definitions.clone(), start.clone(), checkpoint).unwrap();
    step = prepared_message_waits(&mut owner, sample, 2..=14);
    prepared_message_pixels(&step.frame, [[0, 0, 255, 0], [16, 239, 140, 0]], 119..127);
    assert_eq!(owner.resume(14, sample).err(), Some(Fault::Interval));
    assert_eq!(owner.poll(MessageId(2)).unwrap(), None);
    owner.queue(MessageId(999)).unwrap();
    step = prepared_message_waits(&mut owner, sample, 15..=214);
    let checkpoint = owner.checkpoint().unwrap();
    let reopened = PreparedOwner::reopen(definitions.clone(), start.clone(), checkpoint).unwrap();
    assert_eq!(reopened.1.pixels, step.frame.pixels);
    owner = reopened.0;
    prepared_message_waits(&mut owner, sample, 215..=374);
    let dismiss = InputSample {
        held_keys: 1,
        ..sample
    };
    for (interval, input) in [(375, sample), (376, dismiss)] {
        step = owner.resume(interval, input).unwrap();
        assert_eq!(step.outcome, Outcome::Suspended);
        assert!(step.effects.is_empty());
    }
    prepared_message_waits(&mut owner, sample, 377..=388);
    step = owner.resume(389, sample).unwrap();
    assert_eq!(step.outcome, Outcome::Complete);
    assert_eq!(
        step.effects,
        [Effect::AdvanceAudio, Effect::ResumeMovementSound]
    );
    prepared_message_pixels(&step.frame, [[255, 0, 0, 0], [0, 255, 0, 0]], 0..224);
    assert_eq!(owner.resume(390, sample).err(), Some(Fault::NoRequest));
    owner.queue(MessageId(2)).unwrap();
    step = owner.resume(390, sample).unwrap();
    assert_eq!(step.outcome, Outcome::Suspended);
    assert_eq!(step.effects, [Effect::CancelSounds]);
    assert_eq!(owner.poll(MessageId(1)).unwrap(), Some(0));
    owner.queue(MessageId(999)).unwrap();
    prepared_message_waits(&mut owner, sample, 391..=413);
    step = owner.resume(414, dismiss).unwrap();
    assert_eq!(step.outcome, Outcome::Suspended);
    assert!(step.effects.is_empty());
    prepared_message_waits(&mut owner, sample, 415..=426);
    step = owner.resume(427, sample).unwrap();
    assert_eq!(step.outcome, Outcome::Complete);
    assert_eq!(
        step.effects,
        [
            Effect::AdvanceAudio,
            Effect::ResumeMovementSound,
            Effect::SelectPause
        ]
    );
    prepared_message_pixels(&step.frame, [[255, 0, 0, 0], [0, 255, 0, 0]], 0..224);
    assert_eq!(owner.poll(MessageId(2)).unwrap(), Some(0));
    assert_eq!(owner.resume(428, sample).err(), Some(Fault::NoRequest));
    owner.queue(MessageId(999)).unwrap();
    assert_eq!(
        owner.resume(428, sample).err(),
        Some(Fault::MissingMessage(MessageId(999)))
    );
}

// Movies cannot author missing media IDs, same-revision changes or fresh prepared reset admission.
#[cfg_attr(test, test)]
fn prepared_message_rejects_changed_content_and_resets() {
    let (definitions, start) = prepared_message_fixture(0x3801);
    let (mut owner, _) = PreparedOwner::open(definitions.clone(), start.clone()).unwrap();
    let sample = InputSample {
        new_keys: 0,
        held_keys: 0,
        bug_fix_counter: 0,
    };
    owner.queue(MessageId(1)).unwrap();
    owner.resume(1, sample).unwrap();
    let checkpoint = owner.checkpoint().unwrap();
    for index in 0..3 {
        let mut missing = start.clone();
        match index {
            0 => &mut missing.saved,
            1 => &mut missing.active,
            _ => &mut missing.restored,
        }
        .media_id = AssetId(0);
        assert_eq!(
            owner.reset(missing.clone()).err(),
            Some(Fault::MissingDisplay)
        );
        assert_eq!(
            PreparedOwner::open(definitions.clone(), missing).err(),
            Some(Fault::MissingDisplay)
        );
        let mut changed = start.clone();
        match index {
            0 => &mut changed.saved,
            1 => &mut changed.active,
            _ => &mut changed.restored,
        }
        .media_id = AssetId(99);
        assert_eq!(
            PreparedOwner::reopen(definitions.clone(), changed, checkpoint.clone()).err(),
            Some(Fault::ChangedContent)
        );
    }
    let (changed, _) = prepared_message_fixture(0x3802);
    assert_eq!(
        PreparedOwner::reopen(changed, start.clone(), checkpoint.clone()).err(),
        Some(Fault::ChangedContent)
    );
    for (word_delta, palette_delta) in [(1, 0), (0, 1)] {
        let mut changed = start.clone();
        let mut words = (0..0x8000)
            .map(|i| start.saved.media.word(i).unwrap())
            .collect::<Vec<_>>();
        let mut palette = core::array::from_fn(|i| start.saved.media.palette(i as u8).unwrap());
        words[0x5880] ^= word_delta;
        palette[26] ^= palette_delta;
        changed.saved.media = Arc::new(IndexedImage::new(words, palette).unwrap());
        assert_eq!(
            PreparedOwner::reopen(definitions.clone(), changed, checkpoint.clone()).err(),
            Some(Fault::ChangedContent)
        );
    }
    let mut changed = start.clone();
    changed.initial_wait_pending = true;
    assert_eq!(
        PreparedOwner::reopen(definitions.clone(), changed, checkpoint).err(),
        Some(Fault::ChangedContent)
    );
    let (mut reset, frame) = owner.reset(start).unwrap();
    prepared_message_pixels(&frame, [[0, 255, 0, 0], [0, 0, 255, 0]], 0..224);
    assert_eq!(reset.resume(1, sample).err(), Some(Fault::NoRequest));
    reset.queue(MessageId(1)).unwrap();
    assert_eq!(
        reset.resume(1, sample).unwrap().effects,
        [Effect::CancelSounds]
    );
}

// Movies share the renderer and cannot supply independent pixel/failure expectations.
#[cfg_attr(test, test)]
fn independently_authored_raster_pixels_and_prefixes() {
    for id in 38..=43 {
        assert!(
            sm::verify_room_background_case(id).is_some(),
            "raster case {id}"
        );
    }
}

// Movies cannot select tied/negative native headers, unsupported mappers or raw mirrors.
#[cfg_attr(test, test)]
fn room_pack_uses_native_header_selection_and_final_block_mirroring() {
    fn header(source: &mut [u8], location: usize, opcode: u8) {
        source[location + 0x15] = 0x20;
        source[location + 0x16] = 0;
        source[location + 0x19] = 0;
        source[location + 0x1c..location + 0x20].copy_from_slice(&[0xff, 0xff, 0, 0]);
        source[location + 0x3c..location + 0x3e].copy_from_slice(&[0, 0x80]);
        source[location + 0x40 - 0x8000] = opcode;
    }
    let mut raw = vec![0; 0x10200];
    // HiROM scores-2 while LoROM scores-16: max=0 still selects candidate0.
    raw[0xffd5] = 0x20;
    raw[0xffd6] = 7;
    raw[0x8000] = 0x78;
    assert_eq!(
        sm::room_pack::normalize_source(&raw).unwrap().len(),
        0x20000
    );
    raw.fill(0);
    header(&mut raw, 0x7fc0, 0x78);
    header(&mut raw, 0x81c0, 0x78);
    assert_eq!(
        sm::room_pack::normalize_source(&raw).unwrap().len(),
        0x20000
    );
    raw.fill(0);
    header(&mut raw, 0x81c0, 0x78);
    let normalized = sm::room_pack::normalize_source(&raw).unwrap();
    assert_eq!(normalized.len(), 0x10000);
    assert_eq!(normalized, raw[0x200..]);
    let mut odd = vec![0; 0x1007b];
    header(&mut odd, 0x81c0, 0x78);
    assert_eq!(
        sm::room_pack::normalize_source(&odd).unwrap().len(),
        0x10000
    );
    for location in [0xffc0, 0x101c0] {
        raw.fill(0);
        header(&mut raw, 0x7fc0, 0x4c);
        header(&mut raw, location, 0x78);
        assert_eq!(
            sm::room_pack::normalize_source(&raw)
                .unwrap_err()
                .to_string(),
            "room pack vanilla profile requires native-selected LoROM cartridge",
        );
    }
    let mut three_mib = vec![0x11; 0x300000];
    three_mib[0x100000..0x200000].fill(0x22);
    three_mib[0x200000..].fill(0x33);
    header(&mut three_mib, 0x7fc0, 0x78);
    let normalized = sm::room_pack::normalize_source(&three_mib).unwrap();
    assert_eq!(normalized.len(), 0x400000);
    assert_eq!(&normalized[..three_mib.len()], three_mib);
    assert_eq!(&normalized[three_mib.len()..], &three_mib[0x200000..]);
    assert!(sm::room_pack::normalize_source(&[]).is_err());
}

// Movies cannot supply deliberately malformed, checksum-valid portable packs.
#[cfg_attr(test, test)]
fn room_pack_rejects_checksum_valid_structural_errors() {
    fn pack(metadata: &[u8]) -> Vec<u8> {
        let mut bytes = b"SMRP\x01\x00\x01\x00\x00\x00\x00\x00".to_vec();
        bytes.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&[0; 32]);
        bytes.extend_from_slice(metadata);
        let checksum = sm::room_pack::content_fingerprint(&bytes[..20], &bytes[52..]);
        bytes[20..52].copy_from_slice(&checksum);
        bytes
    }
    fn rejected(metadata: &[u8], message: &str) {
        assert_eq!(
            sm::room_pack::load(&pack(metadata))
                .unwrap_err()
                .to_string(),
            message
        );
    }
    let mut room = 7u32.to_le_bytes().to_vec();
    room.extend_from_slice(&[0, 0, 0, 0, 1, 1, 0, 0, 2]);
    room.extend_from_slice(&7u32.to_le_bytes());
    room.extend_from_slice(&1u32.to_le_bytes());
    room.push(0);
    room.extend_from_slice(&7u32.to_le_bytes());
    for id in [7u32, 6] {
        let mut metadata = 2u32.to_le_bytes().to_vec();
        metadata.extend_from_slice(&room);
        metadata.extend_from_slice(&id.to_le_bytes());
        rejected(&metadata, "duplicate or noncanonical world/key ID");
    }
    let mut unknown = 1u32.to_le_bytes().to_vec();
    room[21] = 0xff;
    unknown.extend_from_slice(&room);
    rejected(&unknown, "unknown room selector tag");
    rejected(&65_537u32.to_le_bytes(), "world/key count is out of bounds");
    let mut truncated = 1u32.to_le_bytes().to_vec();
    truncated.extend_from_slice(&room[..7]);
    rejected(&truncated, "truncated world/key record");
    rejected(&[0; 56], "room pack lacks CurrentLoadStation site 6/0");
    rejected(&[0; 57], "trailing world/key records");
    for (offset, value, message) in [
        (4, 2, "unsupported room pack schema"),
        (6, 2, "unsupported room pack profile/options"),
        (8, 1, "unsupported room pack profile/options"),
    ] {
        let mut bytes = pack(&[0; 56]);
        bytes[offset] = value;
        let checksum = sm::room_pack::content_fingerprint(&bytes[..20], &bytes[52..]);
        bytes[20..52].copy_from_slice(&checksum);
        assert_eq!(
            sm::room_pack::load(&bytes).unwrap_err().to_string(),
            message
        );
    }
}

// Movies cannot select duplicate or missing application pack arguments.
#[cfg_attr(test, test)]
fn room_pack_selection_rejects_missing_duplicate_and_option_values() {
    for (args, message) in [
        (vec!["--room-pack"], "--room-pack requires a path"),
        (vec!["--room-pack", ""], "--room-pack requires a path"),
        (
            vec!["--room-pack", "--frames", "2"],
            "--room-pack requires a path",
        ),
        (vec!["--room-pack", "-h"], "--room-pack requires a path"),
        (
            vec!["--room-pack=path"],
            "room pack selection requires --room-pack PATH",
        ),
        (
            vec!["--room-pack", "absent", "--room-pack", "other"],
            "duplicate --room-pack selection",
        ),
    ] {
        let args: Vec<String> = args.into_iter().map(str::to_owned).collect();
        assert_eq!(
            sm::room_pack::select(&args).err().unwrap().to_string(),
            message
        );
    }
    let args = ["verify", "--debug", "rom", "inputs"].map(str::to_owned);
    let selection = sm::room_pack::select(&args).unwrap();
    assert!(selection.world.is_none());
    assert_eq!(selection.arguments, args);
}

fn gameplay() -> CoverageState {
    CoverageState {
        state: 8,
        area: 1,
        room: 0x9dc7,
        ..CoverageState::default()
    }
}

fn required(name: &str) -> RequiredExpectation {
    RequiredExpectation {
        name: name.into(),
        kind: Expectation::parse(name).unwrap(),
        first_frame: None,
        suit_progress: SuitProgress::AwaitingAward,
    }
}

#[cfg_attr(test, test)]
fn expectation_parser_rejects_unknown_or_malformed_values() {
    for value in [
        "",
        "area:7",
        "area:Crateria",
        "boss:ceres-ridley",
        "item:unknown",
        "pickup:health",
        "room:0000",
        "room:0x91f8",
        "room:FFFFF",
        "room:zzzz",
        "door:0000",
        "door:0x91f8",
        "elevator:FFFFF",
        "elevator:zzzz",
        "encounter:ridley",
        "credits:1",
        "state:",
        "state:-1",
        "state:+2",
        "state:45",
        "state:65536",
    ] {
        assert!(Expectation::parse(value).is_err(), "accepted {value}");
    }
    let menu = CoverageState {
        state: 2,
        ..CoverageState::default()
    };
    let state = Expectation::parse("state:2").unwrap();
    assert!(state.observed(&gameplay(), &menu, false));
    assert!(!state.observed(&menu, &gameplay(), false));
    assert_eq!(
        Expectation::parse("room:91f8"),
        Expectation::parse("room:91F8")
    );
    assert_eq!(
        Expectation::parse("door:a7de").unwrap(),
        Expectation::Door(0xa7de)
    );
    assert_eq!(
        Expectation::parse("elevator:A7DE").unwrap(),
        Expectation::Elevator(0xa7de)
    );
    assert_eq!(
        Expectation::parse("encounter:ceres-ridley").unwrap(),
        Expectation::CeresRidley
    );
    assert_eq!(
        Expectation::parse("boss:mother-brain").unwrap(),
        Expectation::Boss { area: 5, mask: 2 }
    );
}

#[cfg_attr(test, test)]
fn inventory_events_require_a_new_award_inside_the_same_live_room() {
    let before = gameplay();
    let mut after = before;
    after.items[0] = 4;
    after.capacity[1] = 5;
    let item = Expectation::parse("item:morph-ball").unwrap();
    let pickup = Expectation::parse("pickup:missile").unwrap();
    assert!(item.observed(&before, &after, false));
    assert!(pickup.observed(&before, &after, false));
    assert!(
        !item.observed(&after, &after, false),
        "loaded inventory is not a pickup"
    );
    assert!(!pickup.observed(&after, &after, false));
    for moved in [
        CoverageState {
            room: 0x9999,
            ..after
        },
        CoverageState { area: 2, ..after },
        CoverageState { state: 7, ..after },
    ] {
        assert!(!item.observed(&before, &moved, false));
        assert!(!pickup.observed(&before, &moved, false));
    }
    assert!(!item.observed(&CoverageState { state: 7, ..before }, &after, false));
}

#[cfg_attr(test, test)]
fn boss_flags_require_a_rising_bit_in_the_corresponding_area() {
    let boss = Expectation::parse("boss:mother-brain").unwrap();
    let before = CoverageState {
        area: 5,
        ..gameplay()
    };
    let mut after = before;
    after.bosses[5] = 1;
    assert!(!boss.observed(&before, &after, false));
    after.bosses[5] = 2;
    assert!(boss.observed(&before, &after, false));
    assert!(!boss.observed(&after, &after, false));
    assert!(!boss.observed(
        &CoverageState { area: 1, ..before },
        &CoverageState { area: 1, ..after },
        false
    ));
}

#[cfg_attr(test, test)]
fn credits_require_enabled_script_advancement_not_ending_presence() {
    let ending = CoverageState {
        state: 39,
        ..CoverageState::default()
    };
    let running = CoverageState {
        credits_enabled: 0x8000,
        credits_cursor: 0x8000,
        ..ending
    };
    assert!(!Expectation::Credits.observed(&ending, &ending, false));
    assert!(!Expectation::Credits.observed(&running, &running, false));
    assert!(Expectation::Credits.observed(
        &running,
        &CoverageState {
            credits_scroll: 1,
            ..running
        },
        false
    ));
    assert!(!Expectation::Credits.observed(
        &CoverageState {
            state: 1,
            ..running
        },
        &CoverageState {
            state: 1,
            credits_scroll: 1,
            ..running
        },
        false
    ));
    let finishing = CoverageState {
        cinematic_function: 0xdb9d,
        ..running
    };
    let finished = CoverageState {
        cinematic_function: 0xe0f4,
        ..ending
    };
    let completion = Expectation::parse("credits-complete").unwrap();
    assert!(completion.observed(&finishing, &finished, false));
    assert!(!completion.observed(&finished, &finished, false));
    assert!(!completion.observed(&finishing, &finishing, false));
    assert!(!completion.observed(&finishing, &ending, false));
    assert!(!completion.observed(&running, &finished, false));
}

#[cfg_attr(test, test)]
fn death_must_begin_in_the_segment_and_reach_game_over() {
    let mut coverage = ReplayCoverage {
        previous: gameplay(),
        expected: vec![required("death")],
        ..ReplayCoverage::default()
    };
    coverage.observe_expectations(
        CoverageState {
            state: 19,
            ..gameplay()
        },
        100,
    );
    assert_eq!(coverage.all_expected_frame(), None);
    coverage.observe_expectations(
        CoverageState {
            state: 25,
            ..gameplay()
        },
        110,
    );
    coverage.observe_expectations(
        CoverageState {
            state: 26,
            ..gameplay()
        },
        111,
    );
    assert_eq!(coverage.all_expected_frame(), Some(111));
    let mut loaded = ReplayCoverage {
        previous: CoverageState {
            state: 25,
            ..gameplay()
        },
        expected: vec![required("death")],
        ..ReplayCoverage::default()
    };
    loaded.observe_expectations(
        CoverageState {
            state: 26,
            ..gameplay()
        },
        200,
    );
    assert_eq!(
        loaded.all_expected_frame(),
        None,
        "loaded death phase is not an observed sequence"
    );
}

#[cfg_attr(test, test)]
fn every_expectation_is_required_and_repeated_hits_keep_the_first_frame() {
    let mut coverage = ReplayCoverage {
        previous: gameplay(),
        expected: vec![required("area:brinstar"), required("pickup:energy")],
        ..ReplayCoverage::default()
    };
    coverage.observe_expectations(gameplay(), 100);
    assert_eq!(coverage.all_expected_frame(), None);
    let after = CoverageState {
        capacity: [199, 0, 0, 0, 0],
        ..gameplay()
    };
    coverage.observe_expectations(after, 200);
    assert_eq!(coverage.all_expected_frame(), Some(200));
    coverage.observe_expectations(after, 300);
    assert_eq!(coverage.all_expected_frame(), Some(200));
}
#[cfg_attr(test, test)]
fn suit_award_waits_for_later_hdma_and_control_return() {
    for (name, mask) in [("item:varia", 1), ("item:gravity", 0x20)] {
        let before = CoverageState {
            player_handlers: [0xe695, 0xe725, 0xeb52],
            ..gameplay()
        };
        // Equipment collection precedes the message and HDMA animation.
        let awarded = CoverageState {
            items: [mask, 0],
            ..before
        };
        let stage3 = CoverageState {
            suit_substate: 3,
            player_handlers: [0xe713, 0xe8cd, 0xec1d],
            ..awarded
        };
        let stage4 = CoverageState {
            suit_substate: 4,
            ..stage3
        };
        let completed = CoverageState {
            suit_substate: 0,
            player_handlers: [0xe695, 0xe725, 0xeb52],
            ..stage4
        };
        let mut coverage = ReplayCoverage {
            previous: before,
            expected: vec![required(name)],
            ..ReplayCoverage::default()
        };
        coverage.observe_expectations(awarded, 100);
        coverage.observe_expectations(awarded, 120);
        assert_eq!(
            coverage.all_expected_frame(),
            None,
            "fresh award with normal handlers is not animation completion"
        );
        coverage.observe_expectations(stage3, 140);
        coverage.observe_expectations(stage4, 160);
        assert_eq!(coverage.all_expected_frame(), None, "HDMA has not finished");
        coverage.observe_expectations(completed, 200);
        assert_eq!(coverage.all_expected_frame(), Some(200));
        let mut loaded = ReplayCoverage {
            previous: awarded,
            expected: vec![required(name)],
            ..ReplayCoverage::default()
        };
        loaded.observe_expectations(stage3, 140);
        loaded.observe_expectations(stage4, 160);
        loaded.observe_expectations(completed, 200);
        assert_eq!(
            loaded.all_expected_frame(),
            None,
            "preloaded suit must not count even if its animation executes"
        );
        for lost_from in [stage4, completed] {
            let mut lost = ReplayCoverage {
                previous: before,
                expected: vec![required(name)],
                ..ReplayCoverage::default()
            };
            lost.observe_expectations(awarded, 100);
            lost.observe_expectations(stage3, 140);
            lost.observe_expectations(stage4, 160);
            lost.observe_expectations(
                CoverageState {
                    items: [0, 0],
                    ..lost_from
                },
                180,
            );
            assert_eq!(lost.all_expected_frame(), None, "lost suit must not count");
            lost.observe_expectations(completed, 200);
            assert_eq!(
                lost.all_expected_frame(),
                None,
                "a restored bit must not finish an interrupted animation"
            );
        }
        for interruption in [
            CoverageState {
                room: 0xa000,
                ..awarded
            },
            CoverageState {
                state: 12,
                ..awarded
            },
        ] {
            let mut interrupted = ReplayCoverage {
                previous: before,
                expected: vec![required(name)],
                ..ReplayCoverage::default()
            };
            interrupted.observe_expectations(awarded, 100);
            interrupted.observe_expectations(interruption, 120);
            interrupted.observe_expectations(stage3, 140);
            interrupted.observe_expectations(stage4, 160);
            interrupted.observe_expectations(completed, 200);
            assert_eq!(
                interrupted.all_expected_frame(),
                None,
                "room/state re-entry must not finish an old award"
            );
        }
    }
}

fn traversal() -> [CoverageState; 6] {
    let start = CoverageState {
        player_handlers: [0xe695, 0xe725, 0xeb52],
        ..gameplay()
    };
    let admission = CoverageState {
        state: 9,
        door_exit: 0x8100,
        door_phase: 0xe17d,
        ..start
    };
    let loading = CoverageState {
        state: 11,
        door_phase: 0xe2f7,
        door_interlocks: [1, 1],
        ..admission
    };
    let destination = CoverageState {
        room: 0xa7de,
        door_phase: 0xe310,
        ..loading
    };
    let header = CoverageState {
        area: 2,
        door_phase: 0xe38e,
        ..destination
    };
    let completed = CoverageState {
        state: 8,
        door_phase: 0xe737,
        door_interlocks: [0, 0],
        ..header
    };
    [start, admission, loading, destination, header, completed]
}

fn episode(name: &str, samples: &[CoverageState]) -> ReplayCoverage {
    let mut coverage = ReplayCoverage {
        previous: samples[0],
        expected: vec![required(name)],
        ..ReplayCoverage::default()
    };
    for (frame, &sample) in samples.iter().enumerate().skip(1) {
        coverage.observe_expectations(sample, frame);
    }
    coverage
}

#[cfg_attr(test, test)]
fn door_requires_fresh_admission_and_the_same_completed_destination() {
    let route = traversal();
    assert_eq!(episode("door:A7DE", &route).all_expected_frame(), Some(5));
    assert_eq!(episode("door:A7DF", &route).all_expected_frame(), None);
    assert_eq!(episode("door:A7DE", &route[1..]).all_expected_frame(), None);
    assert_eq!(episode("door:A7DE", &route[..5]).all_expected_frame(), None);
    for replacement in [
        CoverageState {
            state: 7,
            ..route[4]
        },
        CoverageState {
            room: 0xa7df,
            ..route[4]
        },
        CoverageState {
            door_exit: 0x8200,
            ..route[4]
        },
    ] {
        let mut broken = route;
        broken[4] = replacement;
        assert_eq!(episode("door:A7DE", &broken).all_expected_frame(), None);
    }
    let mut unpublished = route;
    unpublished[3].door_phase = 0xe353;
    assert_eq!(
        episode("door:A7DE", &unpublished).all_expected_frame(),
        None
    );
    let mut interlocked = route;
    interlocked[5].door_interlocks = [1, 0];
    assert_eq!(
        episode("door:A7DE", &interlocked).all_expected_frame(),
        None
    );
    let mut loopback = route;
    for sample in &mut loopback[3..] {
        sample.room = route[0].room;
        sample.area = route[0].area;
    }
    assert_eq!(
        episode("door:9DC7", &loopback).all_expected_frame(),
        Some(5)
    );
}

#[cfg_attr(test, test)]
fn elevator_landing_requires_the_linked_traversal_and_fresh_unlock() {
    let mut route = traversal();
    for sample in &mut route[1..4] {
        sample.elevator_status = 1;
        sample.elevator_flags = 1;
        sample.player_handlers = [0xe695, 0xe8ec, 0xec14];
    }
    for sample in &mut route[4..] {
        sample.elevator_status = 2;
        sample.elevator_flags = 0x80;
        sample.player_handlers = [0xe713, 0xe8ec, 0xec14];
    }
    let landed = CoverageState {
        elevator_status: 0,
        elevator_flags: 0,
        player_handlers: [0xe695, 0xe725, 0xeb52],
        ..route[5]
    };
    for previous_status in [2, 3] {
        let mut coverage = episode("elevator:A7DE", &route);
        assert_eq!(
            coverage.all_expected_frame(),
            None,
            "door arrival is not landing"
        );
        if previous_status == 3 {
            coverage.observe_expectations(
                CoverageState {
                    elevator_status: 3,
                    ..route[5]
                },
                6,
            );
        }
        coverage.observe_expectations(landed, 7);
        assert_eq!(coverage.all_expected_frame(), Some(7));
    }
    assert_eq!(
        episode("elevator:A7DE", &[route[5], landed]).all_expected_frame(),
        None
    );
    let departure = CoverageState {
        state: 8,
        door_phase: 0,
        ..route[1]
    };
    let mut source_ride = vec![route[0], departure, departure, departure];
    source_ride.extend_from_slice(&route[1..]);
    let mut fresh = episode("elevator:A7DE", &source_ride);
    fresh.observe_expectations(landed, 10);
    assert_eq!(fresh.all_expected_frame(), Some(10));
    source_ride[2].state = 12;
    let mut interrupted_source = episode("elevator:A7DE", &source_ride);
    interrupted_source.observe_expectations(landed, 10);
    assert_eq!(
        interrupted_source.all_expected_frame(),
        None,
        "paused departure must restart"
    );
    let mut preloaded_departure = route;
    preloaded_departure[0].elevator_status = 1;
    preloaded_departure[0].elevator_flags = 1;
    let mut preloaded = episode("elevator:A7DE", &preloaded_departure);
    preloaded.observe_expectations(landed, 7);
    assert_eq!(
        preloaded.all_expected_frame(),
        None,
        "preloaded departure is not fresh"
    );
    let mut normal_door = route;
    for sample in &mut normal_door[..4] {
        sample.elevator_status = 0;
        sample.elevator_flags = 0;
    }
    let mut entered = episode("elevator:A7DE", &normal_door);
    entered.observe_expectations(landed, 7);
    assert_eq!(
        entered.all_expected_frame(),
        None,
        "an elevator destination alone is insufficient"
    );
    for interrupted in [
        CoverageState {
            elevator_flags: 0,
            ..route[5]
        },
        CoverageState {
            room: 0xa7df,
            ..route[5]
        },
        CoverageState {
            state: 12,
            ..route[5]
        },
        CoverageState {
            elevator_status: 0,
            ..route[5]
        },
    ] {
        let mut coverage = episode("elevator:A7DE", &route);
        coverage.observe_expectations(interrupted, 6);
        coverage.observe_expectations(landed, 7);
        assert_eq!(coverage.all_expected_frame(), None);
    }
    for bad_landing in [
        CoverageState {
            elevator_flags: 1,
            ..landed
        },
        CoverageState {
            player_handlers: [0xe713, 0xe8ec, 0xec14],
            ..landed
        },
    ] {
        let mut coverage = episode("elevator:A7DE", &route);
        coverage.observe_expectations(bad_landing, 6);
        coverage.observe_expectations(landed, 7);
        assert_eq!(
            coverage.all_expected_frame(),
            None,
            "late unlock is not a landing edge"
        );
    }
}

#[cfg_attr(test, test)]
fn ceres_requires_observed_combat_and_the_same_encounter_escape_edge() {
    let combat = CoverageState {
        area: 6,
        room: 0xe0b5,
        actor_definition: 0x8000,
        actor_bank: 0xa6,
        actor_phase: 0xa6e8,
        ..gameplay()
    };
    let getaway = CoverageState {
        ceres_status: 1,
        actor_phase: 0xc04e,
        ..combat
    };
    let mut escaped = CoverageState {
        ceres_status: 2,
        timer_status: 1,
        actor_phase: 0xaa50,
        ..getaway
    };
    escaped.bosses[6] = 1;
    let route = [combat, combat, getaway, escaped];
    assert_eq!(
        episode("encounter:ceres-ridley", &route).all_expected_frame(),
        Some(3)
    );
    assert_eq!(
        episode("encounter:ceres-ridley", &route[2..]).all_expected_frame(),
        None
    );
    assert_eq!(
        episode("encounter:ceres-ridley", &route[..3]).all_expected_frame(),
        None
    );
    for interruption in [
        CoverageState {
            room: 0xe06b,
            ..getaway
        },
        CoverageState {
            state: 9,
            ..getaway
        },
        CoverageState {
            actor_definition: 0x8002,
            ..getaway
        },
        CoverageState {
            actor_bank: 0xaa,
            ..getaway
        },
        CoverageState {
            actor_phase: 0xa35b,
            ..getaway
        },
    ] {
        let mut broken = route;
        broken[2] = interruption;
        assert_eq!(
            episode("encounter:ceres-ridley", &broken).all_expected_frame(),
            None
        );
    }
    let mut preloaded = route;
    for sample in &mut preloaded {
        sample.bosses[6] = 1;
    }
    assert_eq!(
        episode("encounter:ceres-ridley", &preloaded).all_expected_frame(),
        None
    );
    let mut norfair = route;
    for sample in &mut norfair {
        sample.area = 2;
    }
    assert_eq!(
        episode("encounter:ceres-ridley", &norfair).all_expected_frame(),
        None
    );
}

fn frame(health: u16, dead: bool, flag: bool) -> CoverageState {
    CoverageState {
        state: 8,
        area: 0,
        room: 0x9804,
        bosses: [if flag { 4 } else { 0 }, 0, 0, 0, 0, 0, 0, 0],
        bomb_actor: Some(BombActor {
            slot: 7,
            definition: 0xeeff,
            deleted: false,
            health,
            phase: if dead { 0xc6ab } else { 0xc6bf },
            parameter: if dead { 0xc000 } else { 0 },
        }),
        ..CoverageState::default()
    }
}

fn hits(frames: &[CoverageState]) -> Vec<usize> {
    let mut coverage = ReplayCoverage {
        previous: frames[0],
        ..ReplayCoverage::default()
    };
    let mut hits = Vec::new();
    for (index, frame) in frames.iter().enumerate().skip(1) {
        if coverage.observe_bomb_torizo(frame) {
            hits.push(index);
        }
        coverage.previous = *frame;
    }
    hits
}

#[cfg_attr(test, test)]
fn bomb_requires_observed_damage_death_and_later_completion() {
    let live = frame(100, false, false);
    let damage = frame(50, false, false);
    let death = frame(0, true, false);
    let done = frame(0, true, true);
    assert_eq!(hits(&[live, damage, death, death, done]), vec![4]);
    // A lethal first observed hit still proves actual in-window damage.
    assert_eq!(hits(&[live, death, done]), vec![2]);
    assert_eq!(
        episode("boss:bomb-torizo", &[live, death, done]).all_expected_frame(),
        Some(2)
    );
    assert_eq!(
        episode("boss:bomb-torizo", &[live, done]).all_expected_frame(),
        None
    );
    assert!(hits(&[live, damage, death]).is_empty());
    assert!(hits(&[death, done]).is_empty());
    assert!(hits(&[live, done]).is_empty());
    assert!(hits(&[done, done]).is_empty());
    assert!(hits(&[live, frame(0, false, false), done]).is_empty());
}

#[cfg_attr(test, test)]
fn bomb_deletion_only_finishes_an_already_observed_death() {
    let live = frame(100, false, false);
    let death = frame(0, true, false);
    let mut deleted = death;
    deleted.bomb_actor.as_mut().unwrap().deleted = true;
    let mut done = deleted;
    done.bosses[0] = 4;
    assert_eq!(hits(&[live, death, done, done]), vec![2]);
    assert_eq!(
        episode("boss:bomb-torizo", &[live, death, done]).all_expected_frame(),
        Some(2)
    );
    assert!(
        hits(&[death, done]).is_empty(),
        "preloaded death cannot complete"
    );
    assert!(
        hits(&[deleted, done]).is_empty(),
        "preloaded deletion cannot complete"
    );
    assert!(
        hits(&[live, deleted, done]).is_empty(),
        "deleted actor cannot arm death"
    );
    assert!(
        hits(&[live, death, deleted, done]).is_empty(),
        "late flag cannot revive deletion"
    );
    assert!(
        hits(&[live, death, deleted, death, done]).is_empty(),
        "reappearance cannot revive death"
    );
}

#[cfg_attr(test, test)]
fn bomb_resets_on_room_actor_or_gameplay_discontinuity() {
    let live = frame(100, false, false);
    let death = frame(0, true, false);
    let done = frame(0, true, true);
    for variant in 0..6 {
        let mut interrupted = death;
        match variant {
            0 => interrupted.room = 0x9bc8,
            1 => interrupted.area = 2,
            2 => interrupted.state = 9,
            3 => interrupted.bomb_actor = None,
            4 => interrupted.bomb_actor.as_mut().unwrap().slot = 8,
            5 => interrupted.bomb_actor.as_mut().unwrap().definition = 0xef3f,
            _ => unreachable!(),
        }
        assert!(
            hits(&[live, death, interrupted, death, done]).is_empty(),
            "variant {variant}"
        );
    }
}

#[cfg_attr(test, test)]
fn bomb_does_not_reuse_an_episode_after_healing_or_completion() {
    let live = frame(100, false, false);
    let death = frame(0, true, false);
    let done = frame(0, true, true);
    assert!(hits(&[live, death, live, done]).is_empty());
    assert_eq!(hits(&[live, death, done, death, done]), vec![2]);
}

fn saved_state() -> CoverageState {
    CoverageState {
        state: 8,
        area: 1,
        room: 0xa201,
        selected_save_slot: 0,
        load_station: 3,
        saved_station: 3,
        saved_area: 1,
        player_handlers: [0xe695, 0xe725, 0xeb52],
        coroutines_idle: true,
        ..CoverageState::default()
    }
}

fn step(c: &mut ReplayCoverage, s: CoverageState, save: bool, restore: bool) -> bool {
    let complete = c.observe_save_continue(&s, save, restore);
    c.previous = s;
    complete
}

fn through_restore() -> (ReplayCoverage, CoverageState) {
    let mut s = saved_state();
    let mut c = ReplayCoverage {
        previous: s,
        ..ReplayCoverage::default()
    };
    assert!(!step(&mut c, s, true, false));
    s.room = 0x9cb3;
    assert!(!step(&mut c, s, false, false));
    for state in 19..=26 {
        s.state = state;
        assert!(!step(&mut c, s, false, false));
    }
    assert!(!step(&mut c, s, false, true));
    assert!(c.save_episode.is_some());
    (c, s)
}

fn finish(c: &mut ReplayCoverage, mut s: CoverageState) -> bool {
    s.room = 0xa201;
    for state in [5, 6, 7] {
        s.state = state;
        assert!(!step(c, s, false, false));
    }
    s.state = 8;
    step(c, s, false, false)
}

#[cfg_attr(test, test)]
fn fresh_save_death_selected_restore_and_initialized_arrival_complete_once() {
    let (mut c, s) = through_restore();
    assert!(finish(&mut c, s));
    assert!(!step(&mut c, saved_state(), false, false));
}

#[cfg_attr(test, test)]
fn loaded_history_and_unordered_restore_do_not_qualify() {
    let mut s = saved_state();
    let mut c = ReplayCoverage {
        previous: s,
        ..ReplayCoverage::default()
    };
    s.state = 26;
    assert!(!step(&mut c, s, false, true));
    assert!(!finish(&mut c, s));
    s = saved_state();
    c.previous = s;
    assert!(!step(&mut c, s, true, false));
    s.state = 26;
    assert!(!step(&mut c, s, false, false));
    assert!(!step(&mut c, s, false, true));
    assert!(!finish(&mut c, s));
}

#[cfg_attr(test, test)]
fn reset_and_slot_change_discard_save_history() {
    for state in [0, 1, 4, 5, 6, 7, 27, 31, 39] {
        let mut s = saved_state();
        let mut c = ReplayCoverage {
            previous: s,
            ..ReplayCoverage::default()
        };
        step(&mut c, s, true, false);
        s.state = state;
        assert!(!step(&mut c, s, false, false));
        assert!(c.save_episode.is_none());
    }
    let (mut c, mut s) = through_restore();
    s.selected_save_slot = 1;
    assert!(!step(&mut c, s, false, false));
    assert!(c.save_episode.is_none());
}

#[cfg_attr(test, test)]
fn state_rollbacks_discard_episode() {
    let mut s = saved_state();
    let mut c = ReplayCoverage {
        previous: s,
        ..ReplayCoverage::default()
    };
    step(&mut c, s, true, false);
    for state in [19, 20, 19] {
        s.state = state;
        step(&mut c, s, false, false);
    }
    assert!(c.save_episode.is_none());
    let (mut c, mut s) = through_restore();
    s.state = 5;
    step(&mut c, s, false, false);
    s.state = 26;
    step(&mut c, s, false, false);
    assert!(c.save_episode.is_none());
}

#[cfg_attr(test, test)]
fn wrong_first_arrival_cannot_be_repaired_by_later_room_entry() {
    let (mut c, mut s) = through_restore();
    for state in [5, 6, 7, 8] {
        s.state = state;
        assert!(!step(&mut c, s, false, false));
    }
    assert!(c.save_episode.is_none());
    s.room = 0xa201;
    assert!(!step(&mut c, s, false, false));
}

#[cfg_attr(test, test)]
fn restoration_and_gameplay_initialization_are_required() {
    let (mut c, mut s) = through_restore();
    for state in [5, 6, 7] {
        s.state = state;
        step(&mut c, s, false, false);
    }
    s.state = 8;
    s.room = 0xa201;
    s.coroutines_idle = false;
    assert!(!step(&mut c, s, false, false));
    s.coroutines_idle = true;
    s.player_handlers[0] = 0;
    assert!(!step(&mut c, s, false, false));
    s.player_handlers[0] = 0xe695;
    assert!(step(&mut c, s, false, false));
    let (mut c, mut s) = through_restore();
    s.state = 8;
    s.room = 0xa201;
    assert!(!step(&mut c, s, false, false));
    assert!(c.save_episode.is_none());
}

// Movies cannot author portable HDMA definitions or the independently expected registry.
fn scene_hdma_catalog(gameplay: [u8; 32], first_count: u8) -> Arc<hs::HdmaCatalog> {
    let reference = |id| hs::DefinitionRef {
        id: sc::AssetId(id),
        gameplay,
        definition: [21; 32],
    };
    Arc::new(
        hs::HdmaCatalog::new(
            gameplay,
            vec![
                hs::HdmaDefinition::Program(hs::HdmaProgramDefinition {
                    reference: reference(100),
                    schema: hs::HDMA_CONTENT_SCHEMA,
                    bytes: vec![0, 0].into(),
                }),
                hs::HdmaDefinition::Table(hs::HdmaTableDefinition {
                    reference: reference(101),
                    schema: hs::HDMA_CONTENT_SCHEMA,
                    bytes: vec![first_count, 0, 0x98, 0].into(),
                }),
                hs::HdmaDefinition::Indirect(hs::HdmaIndirectDefinition {
                    reference: reference(102),
                    schema: hs::HDMA_CONTENT_SCHEMA,
                    bytes: vec![0x12, 0x34, 0x56, 0x78].into(),
                }),
            ],
        )
        .unwrap(),
    )
}

// This authored fixture has no live atmospheric objects or published atmospheric parts.
fn scene_atmosphere_catalog(
    revision: sc::Revision,
) -> Arc<sm::game::scene::atmosphere::AtmosphereCatalog> {
    Arc::new(sm::game::scene::atmosphere::AtmosphereCatalog {
        revision,
        kinds: std::collections::BTreeMap::new(),
        sprites: std::collections::BTreeMap::new(),
    })
}

// Movies cannot author portable display starts. Every synthetic field is supplied
// explicitly; this fixture establishes component persistence at the supplied schema.
fn scene_display_description(
    media: &sc::SceneMedia,
    gameplay: [u8; 32],
    colors: [u16; 256],
    objects: [u8; 544],
) -> ds::DisplayDescription {
    let revision = media.revision();
    let reference = |id| sc::AssetRef {
        id: sc::AssetId(id),
        revision,
    };
    let program = hs::SourceBinding::Prepared {
        reference: hs::DefinitionRef {
            id: sc::AssetId(100),
            gameplay,
            definition: [21; 32],
        },
        windows: vec![hs::BankWindow {
            bank: 0x88,
            address: 0x8000,
            offset: 0,
            length: 2,
        }]
        .into(),
    };
    let table = hs::SourceBinding::Prepared {
        reference: hs::DefinitionRef {
            id: sc::AssetId(101),
            gameplay,
            definition: [21; 32],
        },
        windows: vec![hs::BankWindow {
            bank: 0x88,
            address: 0x9000,
            offset: 0,
            length: 4,
        }]
        .into(),
    };
    let indirect = hs::SourceBinding::LiveOwner {
        owner: hs::LiveOwner::HdmaTable1,
        windows: vec![hs::BankWindow {
            bank: 0x7e,
            address: 0x9800,
            offset: 0,
            length: 4,
        }]
        .into(),
    };
    let variable = hs::LiveOwner::ObjectVariable {
        object: 0,
        variable: hs::HdmaVariable::A,
    };
    let hdma_live = hs::HdmaLiveDescription::new(
        vec![hs::HdmaOwnedBuffer {
            owner: hs::LiveOwner::HdmaTable1,
            bytes: vec![0x81, 0x12, 0x34, 0x56],
        }],
        vec![
            hs::HdmaLiveRegion {
                owner: hs::LiveOwner::HdmaTable1,
                source: hs::HdmaRegionSource::Buffer {
                    storage: hs::LiveOwner::HdmaTable1,
                    offset: 0,
                },
                length: 4,
            },
            hs::HdmaLiveRegion {
                owner: hs::LiveOwner::WindowLeft,
                source: hs::HdmaRegionSource::Buffer {
                    storage: hs::LiveOwner::HdmaTable1,
                    offset: 1,
                },
                length: 2,
            },
            hs::HdmaLiveRegion {
                owner: variable,
                source: hs::HdmaRegionSource::ObjectVariable {
                    object: 0,
                    variable: hs::HdmaVariable::A,
                },
                length: 2,
            },
        ],
    )
    .unwrap();
    let sc::SceneAsset::Atlas(atlas) = media.asset(reference(2)).unwrap() else {
        panic!("fixture atlas")
    };
    let source = ds::SourceSpan {
        asset: atlas.reference,
        bytes: atlas
            .data
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect::<Vec<_>>()
            .into(),
        offset: 0,
    };
    let destination = ds::TileDestination {
        word: 0x2000,
        step: 1,
        lanes: ds::ByteLanes::Both,
        remap: 0,
        increment_on_high: true,
    };
    let transfer = ds::TileTransfer {
        source: ds::TransferSource::Asset(source.clone()),
        count: 2,
        destination,
    };
    let scrolling = core::array::from_fn(|slot| {
        let transfers = |wrapped| {
            core::array::from_fn(|half| ds::TileTransfer {
                source: ds::TransferSource::Scroll {
                    slot,
                    half,
                    wrapped,
                    offset: 0,
                },
                count: 2,
                destination: ds::TileDestination {
                    word: 0x3000 + slot as u16 * 32 + half as u16 * 2,
                    ..destination
                },
            })
        };
        ds::ScrollTransfer {
            enabled: 1,
            layout: if slot < 2 {
                ds::ScrollLayout::Column
            } else {
                ds::ScrollLayout::Row
            },
            first: transfers(false),
            wrapped: transfers(true),
            wrapped_count: 2,
            first_bytes: core::array::from_fn(|_| vec![0x12, 0x34]),
            wrapped_bytes: core::array::from_fn(|_| vec![0x56, 0x78]),
        }
    });
    let visual = ds::supplied_visuals(
        ds::FirefleaDescription {
            timer: 23,
            flash_index: 0,
            compatibility_word: 0,
            darkness: 0,
            compatibility_limit: 0,
            compatibility_shade: 0,
        },
        colors,
        [0x1234; 256],
        ds::PaletteEffects {
            slots: [ds::PaletteSlot {
                definition: 0,
                color_offset: 0,
                pre_action: 0,
                program: 0,
                delay: 0,
                counter: 0,
            }; 8],
            flags: 0,
            selected: 0,
            heat: 5,
            previous_heat: 7,
        },
        ds::AnimatedTiles {
            slots: [ds::AnimationSlot {
                definition: 0,
                counter: 0,
                program: 0,
                delay: 0,
                source: 0,
                size: 0,
                vram: 0,
            }; 6],
            flags: 0,
            selected: 0,
            last_instruction: 29,
        },
    );
    ds::DisplayDescription {
        media_revision: revision,
        phase: ds::DisplayPhase::Committed,
        objects: ds::ObjectStaging {
            bytes: objects,
            cursor: 512,
            native_order: ds::WordOrder::Little,
            produced: vec![],
        },
        hud: ds::HudState {
            tiles: [0x3801; 96],
            health: 99,
            maximum_health: 199,
            previous_health: 98,
            ammunition: [1, 2, 3],
            maximum_ammunition: [11, 12, 13],
            previous_ammunition: [0, 1, 2],
            selection: 2,
            previous_selection: 1,
            auto_cancel: 0,
            reserve_mode: 1,
            reserve_health: 31,
            palette: 0x1800,
            media: reference(5),
        },
        visual,
        player_palette: ds::PlayerPalette {
            hurt_flash: 27,
            visor_timer: 29,
            suit_index: 31,
            xray_progress: 33,
            timer: 35,
            suit_beam_speed: 37,
            suit_red_green: 39,
            crystal_flash_delay: 41,
        },
        atmospheric: ds::AtmosphericEffects {
            slots: core::array::from_fn(|slot| ds::AtmosphericSlot {
                remaining: 51 + slot as u16,
                x: 61 + slot as u16,
                y: 71 + slot as u16,
                frame_and_kind: 0,
            }),
        },
        atmosphere_catalog: scene_atmosphere_catalog(revision),
        producer_ids: sc::DisplayProducerIds {
            next: sc::ProducerObjectId(1),
        },
        atmospheric_producers: sm::game::scene::atmosphere::AtmosphericProducers {
            slots: [None; 4],
        },
        nmi: ds::NmiState {
            pending: 1,
            frame_byte: 11,
            frame_word: 0x1234,
            including_lag: 0xabcd,
            missed: 3,
            max_missed: 5,
            acknowledgment: 1,
        },
        samus: core::array::from_fn(|_| ds::SamusHalf {
            enabled: 1,
            descriptor: reference(8),
            first: transfer.clone(),
            second: ds::TileTransfer {
                count: 0,
                ..transfer.clone()
            },
        }),
        backgrounds: core::array::from_fn(|plane| ds::BackgroundState {
            tilemap: reference(if plane == 2 { 5 } else { 4 }),
            atlas: reference(if plane == 2 { 3 } else { 2 }),
            palette: reference(6),
            camera: [17, 0xfffd],
            scroll: [0, 0xffff],
            screen_layout: 0,
            character_layout: 0,
        }),
        scrolling,
        uploads: vec![ds::QueuedUpload {
            cursor: 0,
            transfer: transfer.clone(),
        }],
        upload_tail: 7,
        mode7_uploads: vec![ds::Mode7Upload {
            cursor: 0,
            source: ds::TransferSource::Asset(source.clone()),
            count: 2,
            target: ds::Mode7Target::Tiles(destination),
            transfer_mode: 0,
        }],
        mode7_tail: 7,
        readbacks: vec![ds::Readback {
            cursor: 0,
            destination: reference(2),
            first_word: 0,
            count: 2,
        }],
        readback_tail: 9,
        actor_queues: core::array::from_fn(|slot| ds::ActorDrawQueue {
            size: 4,
            cursor: 2,
            actors: vec![slot as u16 * 2 + 1, slot as u16 * 2 + 2],
        }),
        actor_tiles: ds::ActorTileStaging {
            pending: 1,
            size: 2,
            source: source.clone(),
            destination: 0x2222,
            transfer_source: 0x13,
            transfer_destination: 0x15,
            tilemap_bytes: vec![0xca, 0xfe],
        },
        controls: ds::DisplayControls {
            brightness: 15,
            object_layout: 0,
            mode: 1,
            fake_mode: 1,
            mosaic: 0,
            screens: [0; 4],
            previous_background3_map: 0x35,
            characters: [0; 2],
            affine_selection: 0,
            window_selection: [0; 3],
            window_positions: [0; 4],
            window_logic: [0; 2],
            main_layers: 4,
            previous_main_layers: 4,
            sub_layers: 0,
            main_window: 0,
            sub_window: 0,
            next_color_selection: 0xb2,
            color_selection: 0,
            previous_color_selection: 0,
            next_color_math: 0xa5,
            color_math: 0,
            previous_color_math: 0,
            fixed_color: [0; 3],
            geometry: 0,
            scroll_words: [0; 8],
            affine_words: [0; 6],
            nmi_enable: 0,
            hdma_enable: 0,
            committed_fake_mode: 1,
        },
        hdma: ds::HdmaState {
            bindings: [ds::HdmaBinding {
                enable: 1,
                bank_slot: 0,
                table: 0x9000,
            }; 6],
            objects: core::array::from_fn(|_| ds::HdmaObject {
                instruction: program.clone(),
                instruction_cursor: 0x8000,
                instruction_timer: 0,
                pre_instruction: 0,
                pre_instruction_bank: 0x88,
                timer: 0,
                variables: [0x3344, 0x5566, 0, 0, 0, 0],
            }),
            channels: core::array::from_fn(|_| ds::HdmaChannel {
                source: indirect.clone(),
                table: table.clone(),
                active: true,
                terminated: false,
                do_transfer: true,
                repeat_count: 0x81,
                source_address: 0x9000,
                table_cursor: 0x9003,
                indirect_cursor: 0x9800,
                mode: 0,
                from_destination: false,
                indirect: true,
                fixed: true,
                decrement: false,
                source_bank: 0x88,
                indirect_bank: 0x7e,
                destination: 0,
                transfer_cursor: 0,
                unused_bit: false,
                unused_byte: 0,
                dma_active: false,
            }),
            selected: 0,
            flags: 0x8000,
            hdma_timer: 17,
            dma_timer: 19,
            dma_busy: false,
        },
        hdma_live,
        interrupt: ds::InterruptState {
            current: 0,
            pending: 0,
            vertical: 0,
            horizontal: 0,
            previous_cre: 0,
            current_cre: 0,
            transition_flag: 0,
            transition_continuation: 0,
        },
        // These authored delivered values stay independent of staged and saved controls.
        delivery: ds::DisplayDeliveryDescription {
            horizontal_position: 37,
            vertical_position: 241,
            beam_frames: 0x1234_5678,
            horizontal_irq_enabled: true,
            vertical_irq_enabled: false,
            nmi_enabled: true,
            nmi_wanted: false,
            irq_wanted: true,
            in_nmi: false,
            in_irq: true,
            in_vblank: true,
            automatic_input_read: true,
            automatic_input_timer: 0x4567,
            counter_latch_gate: true,
            raster_suppressed: false,
        },
        effective: ds::EffectiveDisplayDescription {
            background3_map_words: 0x2c00,
            background3_map_wider: true,
            background3_map_higher: false,
            color_clip: 2,
            color_prevent_math: 1,
            color_add_subscreen: true,
            direct_color: true,
            color_math_enabled: [true, false, true, false, true, false],
            color_subtract: true,
            color_half: false,
            main_layers: 0xa4,
            forced_blank: true,
        },
        latches: ds::DisplayLatches {
            tile_pointer: 0x2345,
            tile_increment: 2,
            tile_remap: 0,
            tile_read_buffer: 0x6789,
            tile_increment_on_high: true,
            tile_padding: [0; 2],
            palette_pointer: 119,
            palette_second: true,
            palette_buffer: 0x99,
            object_written_address: 0x101,
            object_written_high_table: false,
            object_address: 0x102,
            object_high_table: true,
            object_second: true,
            object_buffer: 0x66,
            scroll_previous: 0x77,
            scroll_previous_second: 0x88,
            affine_previous: 0xaa,
            latched_horizontal: 137,
            latched_vertical: 139,
            horizontal_second: true,
            vertical_second: false,
            counter_latched: true,
            open_bus: 0xbb,
            ppu_open_bus: [0xcc, 0xdd],
        },
        raster: ds::RasterState {
            range_overflow: false,
            time_overflow: false,
            has_objects: false,
            even_frame: false,
            interlaced: false,
            frame_interlaced: false,
            overscan: false,
            frame_overscan: false,
            mosaic_origin: 1,
            mosaic_phase: 0,
            visible: true,
        },
        animated_sources: core::array::from_fn(|_| source.clone()),
    }
}

// Movies cannot author portable final resident media that differs from visible line history.
fn scene_component_fixture() -> (
    ss::DisplayStartRequirements,
    ss::DisplayStartDescription,
    Arc<sc::SceneMedia>,
) {
    let (mut requirements, mut start, media) = scene_start_prefix();
    let mut images = media.compiled_images().cloned().collect::<Vec<_>>();
    let mut palettes = media.palette_incarnations().cloned().collect::<Vec<_>>();
    let mut image = images[1].clone();
    image.reference.asset.id = sc::AssetId(9);
    image.reference.incarnation = 3;
    image.palette.asset.id = sc::AssetId(10);
    image.palette.incarnation = 3;
    let mut palette = palettes[1].clone();
    let staged_colors = palette.colors;
    palette.reference.asset.id = sc::AssetId(10);
    palette.reference.incarnation = 3;
    palette.colors[0] = 31 << 10;
    palette.colors[25] = 31;
    palette.colors[26] = 0x7fff;
    let words = Arc::make_mut(&mut image.words);
    words[0x8000..0x8005].copy_from_slice(&[0x2345, 1, 2, 0, 0x6789]);
    words[0x8005..0x801f].copy_from_slice(&palette.colors[..26]);
    let mut objects = [0; 544];
    objects[512..].fill(0x55);
    let mut resident_objects = objects;
    resident_objects[0] = 0x42;
    let resident = sm::game::scene::ResidentDisplayDescription {
        image: image.reference,
        palette: palette.reference,
        words: image.words.clone(),
        colors: palette.colors,
        objects: resident_objects,
        produced: Arc::from([]),
    };
    images.push(image);
    palettes.push(palette);
    let mut assets = media.assets().cloned().collect::<Vec<_>>();
    let revision = media.revision();
    let room = sc::AssetRef {
        id: sc::AssetId(9),
        revision,
    };
    let final_palette = sc::AssetRef {
        id: sc::AssetId(10),
        revision,
    };
    assets.push(sc::SceneAsset::Room(room));
    assets.push(sc::SceneAsset::Palette(sc::Palette {
        reference: final_palette,
        colors: resident.colors,
    }));
    let media = Arc::new(sc::SceneMedia::new(media.revision(), assets, palettes, images).unwrap());
    let committed = start.committed_scene.take().unwrap();
    let mut description = committed.description().clone();
    let mut resident_assets = description.resident.to_vec();
    resident_assets.extend([room, final_palette]);
    description.resident = resident_assets.into();
    start.committed_scene = Some(
        sc::Scene::new(description, media.clone())
            .unwrap()
            .with_producer_history(Arc::from(committed.producer_history().to_vec()))
            .unwrap(),
    );
    let hdma_content = scene_hdma_catalog(requirements.gameplay, 0x81);
    requirements.hdma_content = hdma_content.clone();
    start.hdma_content = Some(hdma_content);
    start.display = Some(scene_display_description(
        &media,
        requirements.gameplay,
        staged_colors,
        objects,
    ));
    start.resident = Some(resident);
    (requirements, start, media)
}

fn scene_copy_metadata(metadata: &sd::DisplayStartMetadata) -> sd::DisplayStartMetadata {
    sd::DisplayStartMetadata {
        identity: metadata.identity,
        reference: metadata.reference,
        input_history: metadata.input_history,
        continuation: metadata.continuation,
        audio: metadata.audio.clone(),
        samus_content: metadata.samus_content.clone(),
    }
}

fn scene_metadata(start: &ss::DisplayStartDescription) -> sd::DisplayStartMetadata {
    sd::DisplayStartMetadata {
        identity: start.identity.unwrap(),
        reference: start.reference.unwrap(),
        input_history: start.input_history.unwrap(),
        continuation: start.continuation.unwrap(),
        audio: start.audio.as_ref().unwrap().clone(),
        samus_content: start.samus_content.as_ref().unwrap().clone(),
    }
}

fn scene_same_metadata(actual: &sd::DisplayStartMetadata, expected: &sd::DisplayStartMetadata) {
    assert_eq!(actual.identity, expected.identity);
    assert_eq!(actual.reference, expected.reference);
    assert_eq!(actual.input_history, expected.input_history);
    assert_eq!(actual.continuation, expected.continuation);
    assert_eq!(actual.audio, expected.audio);
    assert_eq!(actual.samus_content, expected.samus_content);
}

fn scene_same_resident(
    actual: &sm::game::scene::ResidentDisplayDescription,
    expected: &sm::game::scene::ResidentDisplayDescription,
) {
    assert_eq!(
        (actual.image, actual.palette),
        (expected.image, expected.palette)
    );
    assert_eq!(actual.words, expected.words);
    assert_eq!(actual.colors, expected.colors);
    assert_eq!(actual.objects, expected.objects);
    assert_eq!(actual.produced, expected.produced);
}

// Movies cannot author divergent delivered controls or written OAM selection at portable reopening.
#[cfg_attr(test, test)]
fn scene_component_preserves_schema3_delivery_effective_and_written_selection() {
    let (requirements, start, media) = scene_component_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let checkpoint = opened.display.checkpoint().unwrap();
    let (description, _) = checkpoint.into_start_description(opened.metadata);
    let state = description.display.as_ref().unwrap();
    assert_eq!(
        state.delivery,
        ds::DisplayDeliveryDescription {
            horizontal_position: 37,
            vertical_position: 241,
            beam_frames: 0x1234_5678,
            horizontal_irq_enabled: true,
            vertical_irq_enabled: false,
            nmi_enabled: true,
            nmi_wanted: false,
            irq_wanted: true,
            in_nmi: false,
            in_irq: true,
            in_vblank: true,
            automatic_input_read: true,
            automatic_input_timer: 0x4567,
            counter_latch_gate: true,
            raster_suppressed: false,
        }
    );
    assert_eq!(
        state.effective,
        ds::EffectiveDisplayDescription {
            background3_map_words: 0x2c00,
            background3_map_wider: true,
            background3_map_higher: false,
            color_clip: 2,
            color_prevent_math: 1,
            color_add_subscreen: true,
            direct_color: true,
            color_math_enabled: [true, false, true, false, true, false],
            color_subtract: true,
            color_half: false,
            main_layers: 0xa4,
            forced_blank: true,
        }
    );
    assert_eq!(
        (
            state.controls.screens[2],
            state.controls.previous_background3_map,
            state.controls.color_selection,
            state.controls.previous_color_selection,
            state.controls.next_color_selection,
            state.controls.color_math,
            state.controls.previous_color_math,
            state.controls.next_color_math
        ),
        (0, 0x35, 0, 0, 0xb2, 0, 0, 0xa5)
    );
    assert_eq!(
        (
            state.latches.object_written_address,
            state.latches.object_written_high_table,
            state.latches.object_address,
            state.latches.object_high_table
        ),
        (0x101, false, 0x102, true)
    );
    assert_eq!(
        (state.interrupt.vertical, state.interrupt.horizontal),
        (0, 0)
    );
    assert_eq!((state.hdma.hdma_timer, state.hdma.dma_timer), (17, 19));
}

// Movies cannot reach schema3 rejection of old starts or malformed effective color selectors.
#[cfg_attr(test, test)]
fn scene_component_rejects_schema2_and_invalid_effective_color_selectors() {
    for case in 0..4 {
        let (mut requirements, mut start, media) = scene_component_fixture();
        match case {
            0 => start.identity.as_mut().unwrap().schema = 2,
            1 => {
                start.identity.as_mut().unwrap().schema = 2;
                requirements.schema = 2;
            }
            2 => start.display.as_mut().unwrap().effective.color_clip = 4,
            _ => start.display.as_mut().unwrap().effective.color_prevent_math = 4,
        }
        let error = ss::ValidatedDisplayStart::construct(start, media, &requirements)
            .err()
            .unwrap();
        match case {
            0 | 1 => assert!(matches!(error, ss::StartError::Schema)),
            _ => assert!(matches!(
                error,
                ss::StartError::Display(ds::DisplayError::Geometry)
            )),
        }
    }
}

// Movies cannot reach authored portable component open/reopen or final resident history.
#[cfg_attr(test, test)]
fn scene_component_preserves_complete_open_checkpoint_and_reopen() {
    let (requirements, start, media) = scene_component_fixture();
    let expected_state = format!("{:?}", start.display.as_ref().unwrap());
    let expected_resident = start.resident.as_ref().unwrap().clone();
    let expected_scene = start.committed_scene.as_ref().unwrap().clone();
    let metadata = scene_metadata(&start);
    let sealed = ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap();
    let opened = sd::DisplayComponent::open(sealed).unwrap();
    scene_same_metadata(&opened.metadata, &metadata);
    let checkpoint = opened.display.checkpoint().unwrap();
    assert_eq!(opened.display.committed_scene(), &expected_scene);
    assert_eq!(checkpoint.committed_scene(), &expected_scene);
    scene_same_resident(checkpoint.resident(), &expected_resident);
    assert_eq!(checkpoint.resident().image.incarnation, 3);
    assert_eq!(checkpoint.resident().palette.incarnation, 3);
    assert_eq!(checkpoint.resident().words[0x8000], 0x2345);
    assert_eq!(checkpoint.resident().colors[26], 0x7fff);
    assert_eq!(
        checkpoint.committed_scene().lines()[223].media.incarnation,
        2
    );
    let frame = opened.display.committed_scene().render().unwrap();
    assert_eq!(
        [frame.pixels[8], frame.pixels[200 * 256 + 8]],
        [[0, 0, 255, 0], [255, 0, 0, 0]]
    );
    let (description, media) = checkpoint.into_start_description(opened.metadata);
    drop(opened.display);
    assert_eq!(
        format!("{:?}", description.display.as_ref().unwrap()),
        expected_state
    );
    assert_eq!(
        description.hdma_content.as_deref(),
        Some(requirements.hdma_content.as_ref())
    );
    let display = description.display.as_ref().unwrap();
    assert_eq!(
        (
            display.nmi.pending,
            display.nmi.acknowledgment,
            display.nmi.including_lag
        ),
        (1, 1, 0xabcd)
    );
    assert_eq!(
        (
            display.objects.cursor,
            display.hud.health,
            display.upload_tail,
            display.mode7_tail,
            display.readback_tail
        ),
        (512, 99, 7, 7, 9)
    );
    let reopened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(description, media, &requirements).unwrap(),
    )
    .unwrap();
    scene_same_metadata(&reopened.metadata, &metadata);
    assert_eq!(reopened.display.committed_scene().render().unwrap(), frame);
    let checkpoint = reopened.display.checkpoint().unwrap();
    scene_same_resident(checkpoint.resident(), &expected_resident);
    let (description, _) = checkpoint.into_start_description(reopened.metadata);
    assert_eq!(
        format!("{:?}", description.display.as_ref().unwrap()),
        expected_state
    );
}

// Movies cannot reach authored portable buffer aliases, retained checkpoint bytes or successful reset.
#[cfg_attr(test, test)]
fn scene_component_reset_preserves_complete_owned_buffers_and_alias_views() {
    let (requirements, start, media) = scene_component_fixture();
    let metadata = scene_metadata(&start);
    let expected_scene = start.committed_scene.as_ref().unwrap().clone();
    let expected_resident = start.resident.as_ref().unwrap().clone();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let checkpoint = opened.display.checkpoint().unwrap();
    let retained = checkpoint.clone();
    let (mut reset, media) = checkpoint.into_start_description(opened.metadata);
    let reset_state = reset.display.as_mut().unwrap();
    assert_eq!(
        reset_state
            .hdma_live
            .read(&reset_state.hdma, hs::LiveOwner::WindowLeft, 0)
            .unwrap(),
        0x12
    );
    let variable = hs::LiveOwner::ObjectVariable {
        object: 0,
        variable: hs::HdmaVariable::A,
    };
    assert_eq!(
        reset_state
            .hdma_live
            .read(&reset_state.hdma, variable, 0)
            .unwrap(),
        0x44
    );
    reset_state
        .hdma_live
        .write(&mut reset_state.hdma, hs::LiveOwner::WindowLeft, 0, 0xee)
        .unwrap();
    reset_state
        .hdma_live
        .write(&mut reset_state.hdma, variable, 1, 0xaa)
        .unwrap();
    assert_eq!(
        reset_state
            .hdma_live
            .read(&reset_state.hdma, hs::LiveOwner::HdmaTable1, 1)
            .unwrap(),
        0xee
    );
    assert_eq!(reset_state.hdma.objects[0].variables[0], 0xaa44);
    let (retained_state, _) = retained.into_start_description(scene_copy_metadata(&metadata));
    let retained_state = retained_state.display.unwrap();
    assert_eq!(
        retained_state
            .hdma_live
            .read(&retained_state.hdma, hs::LiveOwner::HdmaTable1, 1)
            .unwrap(),
        0x12
    );
    assert_eq!(retained_state.hdma.objects[0].variables[0], 0x3344);
    reset.display.as_mut().unwrap().hud.health = 143;
    reset.display.as_mut().unwrap().nmi.acknowledgment = 5;
    let expected_reset = format!("{:?}", reset.display.as_ref().unwrap());
    let mut display = opened.display;
    let reset_metadata = display
        .reset(ss::ValidatedDisplayStart::construct(reset, media, &requirements).unwrap())
        .unwrap();
    scene_same_metadata(&reset_metadata, &metadata);
    let checkpoint = display.checkpoint().unwrap();
    assert_eq!(checkpoint.committed_scene(), &expected_scene);
    scene_same_resident(checkpoint.resident(), &expected_resident);
    let (reset, _) = checkpoint.into_start_description(reset_metadata);
    assert_eq!(
        format!("{:?}", reset.display.as_ref().unwrap()),
        expected_reset
    );
    assert_eq!(
        (
            reset.display.as_ref().unwrap().hud.health,
            reset.display.as_ref().unwrap().nmi.acknowledgment
        ),
        (143, 5)
    );
    let state = reset.display.unwrap();
    assert_eq!(
        state
            .hdma_live
            .read(&state.hdma, hs::LiveOwner::HdmaTable1, 1)
            .unwrap(),
        0xee
    );
    assert_eq!(
        state
            .hdma_live
            .read(&state.hdma, hs::LiveOwner::WindowLeft, 0)
            .unwrap(),
        0xee
    );
    assert_eq!(state.hdma.objects[0].variables[0], 0xaa44);
}

// Movies cannot reach authored portable missing resident/history or failed replacement admission.
#[cfg_attr(test, test)]
fn scene_component_rejects_missing_resident_history_and_media_replacement() {
    let (requirements, start, media) = scene_component_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    for case in 0..6 {
        let (mut description, media) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        match case {
            0 => description.resident = None,
            1 => description.committed_scene = None,
            2 => Arc::make_mut(&mut description.resident.as_mut().unwrap().words)[0] ^= 1,
            3 => description.resident.as_mut().unwrap().palette.incarnation = 99,
            4 => description.identity.as_mut().unwrap().media = sc::Revision([99; 32]),
            _ => {
                description.identity.as_mut().unwrap().committed_tick = 1;
                description.continuation.as_mut().unwrap().phase =
                    ss::CommittedPhase::SettledInterval;
            }
        }
        let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
            .and_then(|start| display.reset(start));
        let error = result.err().unwrap();
        assert!(matches!(
            (case, error),
            (0, ss::StartError::MissingResidentDisplay)
                | (1, ss::StartError::MissingCommittedHistory)
                | (2..=3, ss::StartError::Display(ds::DisplayError::Scene))
                | (4, ss::StartError::Revision)
                | (5, ss::StartError::CommitIdentity)
        ));
        let after = display.checkpoint().unwrap();
        assert_eq!(after.committed_scene(), before.committed_scene());
        scene_same_resident(after.resident(), before.resident());
        let (before_state, _) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        let (after_state, _) = after.into_start_description(scene_copy_metadata(&opened.metadata));
        assert_eq!(
            format!("{:?}", before_state.display),
            format!("{:?}", after_state.display)
        );
    }
}

// Movies cannot reach authored portable private-source/phase rejection or failed replacement preservation.
#[cfg_attr(test, test)]
fn scene_component_failed_source_replacement_preserves_complete_owner() {
    let (requirements, start, media) = scene_component_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    for case in 0..7 {
        let (mut description, media) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        let state = description.display.as_mut().unwrap();
        let expected = match case {
            0 => {
                state.hud.media.id = sc::AssetId(99);
                ds::DisplayError::Identity
            }
            1 => {
                if let ds::TransferSource::Asset(source) = &mut state.samus[0].first.source {
                    Arc::make_mut(&mut source.bytes)[0] ^= 1;
                }
                ds::DisplayError::Source
            }
            2 => {
                state.scrolling[0].first[0].count = 3;
                ds::DisplayError::Source
            }
            3 => {
                state.actor_tiles.source.offset = 33;
                ds::DisplayError::Source
            }
            4 => {
                state.objects.cursor = 513;
                ds::DisplayError::Alias { offset: 513 }
            }
            5 => {
                state.phase = ds::DisplayPhase::Producing;
                ds::DisplayError::Phase
            }
            _ => {
                state.animated_sources[0].asset.id = sc::AssetId(99);
                ds::DisplayError::Identity
            }
        };
        let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
            .and_then(|start| display.reset(start));
        let ss::StartError::Display(error) = result.err().unwrap() else {
            panic!("expected private display-source rejection")
        };
        assert_eq!(error, expected, "private source replacement {case}");
        let after = display.checkpoint().unwrap();
        assert_eq!(after.committed_scene(), before.committed_scene());
        scene_same_resident(after.resident(), before.resident());
        let (before_state, _) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        let (after_state, _) = after.into_start_description(scene_copy_metadata(&opened.metadata));
        assert_eq!(
            format!("{:?}", before_state.display),
            format!("{:?}", after_state.display)
        );
    }
}

fn scene_owner_unchanged(
    display: &sd::DisplayComponent,
    before: &sd::DisplayCheckpoint,
    metadata: &sd::DisplayStartMetadata,
) {
    let after = display.checkpoint().unwrap();
    assert_eq!(after.committed_scene(), before.committed_scene());
    scene_same_resident(after.resident(), before.resident());
    let (before_state, before_media) = before
        .clone()
        .into_start_description(scene_copy_metadata(metadata));
    let (after_state, after_media) = after.into_start_description(scene_copy_metadata(metadata));
    assert!(Arc::ptr_eq(&before_media, &after_media));
    assert_eq!(before_state.hdma_content, after_state.hdma_content);
    assert_eq!(
        format!("{:?}", before_state.display),
        format!("{:?}", after_state.display)
    );
}

// Movies cannot author final resident assets that are absent from otherwise valid visible history.
#[cfg_attr(test, test)]
fn scene_component_rejects_missing_final_resident_assets() {
    let (requirements, start, media) = scene_component_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    for missing in [sc::AssetId(9), sc::AssetId(10)] {
        let (mut description, media) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        let mut scene = description
            .committed_scene
            .take()
            .unwrap()
            .description()
            .clone();
        scene.resident = scene
            .resident
            .iter()
            .copied()
            .filter(|asset| asset.id != missing)
            .collect::<Vec<_>>()
            .into();
        let scene = sc::Scene::new(scene, media.clone())
            .unwrap()
            .with_producer_history(Arc::from(
                before.committed_scene().producer_history().to_vec(),
            ))
            .unwrap();
        assert_eq!(
            scene.render().unwrap(),
            before.committed_scene().render().unwrap()
        );
        assert_eq!(scene.lines()[223].media.incarnation, 2);
        description.committed_scene = Some(scene);
        let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
            .and_then(|start| display.reset(start));
        assert!(matches!(
            result.err().unwrap(),
            ss::StartError::MissingResidentMedia
        ));
        scene_owner_unchanged(&display, &before, &opened.metadata);
    }
}

// Movies cannot author individually inconsistent latch fields against immutable resident media.
#[cfg_attr(test, test)]
fn scene_component_rejects_each_native_resident_latch_alias() {
    let (requirements, start, media) = scene_component_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    assert_eq!(
        &before.resident().words[0x8000..0x8005],
        &[0x2345, 1, 2, 0, 0x6789]
    );
    for alias in 0..7 {
        let (mut description, media) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        let latches = &mut description.display.as_mut().unwrap().latches;
        match alias {
            0 => latches.tile_pointer ^= 1,
            1 => latches.tile_increment_on_high = !latches.tile_increment_on_high,
            2 => latches.tile_increment ^= 1,
            3 => latches.tile_remap ^= 1,
            4 => latches.tile_read_buffer ^= 1,
            5 => latches.tile_padding[0] ^= 1,
            _ => latches.tile_padding[1] ^= 1,
        }
        let resident = description.resident.as_ref().unwrap();
        assert_eq!(
            resident.words,
            media
                .compiled_images()
                .find(|image| image.reference == resident.image)
                .unwrap()
                .words
        );
        assert_eq!(
            resident.colors,
            media
                .palette_incarnations()
                .find(|palette| palette.reference == resident.palette)
                .unwrap()
                .colors
        );
        let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
            .and_then(|start| display.reset(start));
        assert!(matches!(
            result.err().unwrap(),
            ss::StartError::Display(ds::DisplayError::Scene)
        ));
        scene_owner_unchanged(&display, &before, &opened.metadata);
    }
}

// Movies cannot omit a portable gameplay catalog while supplying complete active HDMA state.
#[cfg_attr(test, test)]
fn scene_component_rejects_missing_hdma_catalog() {
    let (requirements, start, media) = scene_component_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    let (mut description, media) = before
        .clone()
        .into_start_description(scene_copy_metadata(&opened.metadata));
    description.hdma_content = None;
    let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
        .and_then(|start| display.reset(start));
    assert!(matches!(
        result.err().unwrap(),
        ss::StartError::MissingHdmaContent
    ));
    scene_owner_unchanged(&display, &before, &opened.metadata);
}

// Movies cannot author a changed catalog payload under unchanged gameplay and definition fingerprints.
#[cfg_attr(test, test)]
fn scene_component_rejects_changed_hdma_payload_under_same_fingerprint() {
    let (requirements, start, media) = scene_component_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    let (mut description, media) = before
        .clone()
        .into_start_description(scene_copy_metadata(&opened.metadata));
    let changed = scene_hdma_catalog(requirements.gameplay, 0x80);
    assert_eq!(changed.gameplay(), requirements.hdma_content.gameplay());
    let table = hs::DefinitionRef {
        id: sc::AssetId(101),
        gameplay: requirements.gameplay,
        definition: [21; 32],
    };
    assert_eq!(
        changed
            .definition(table, hs::ContentKind::Table)
            .unwrap()
            .reference(),
        requirements
            .hdma_content
            .definition(table, hs::ContentKind::Table)
            .unwrap()
            .reference()
    );
    description.hdma_content = Some(changed);
    let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
        .and_then(|start| display.reset(start));
    assert!(matches!(
        result.err().unwrap(),
        ss::StartError::Display(ds::DisplayError::Source)
    ));
    scene_owner_unchanged(&display, &before, &opened.metadata);
}

// Movies cannot author invalid portable HDMA kinds, dormant provenance or missing live owners.
#[cfg_attr(test, test)]
fn scene_component_rejects_wrong_hdma_kinds_and_required_unbound_sources() {
    let (requirements, start, media) = scene_component_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    for case in 0..7 {
        let (mut description, media) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        let state = description.display.as_mut().unwrap();
        let expected = match case {
            0 => {
                state.hdma.channels[0].table = state.hdma.objects[0].instruction.clone();
                ds::DisplayError::Source
            }
            1 => {
                state.hdma.objects[0].instruction = state.hdma.channels[0].table.clone();
                ds::DisplayError::Source
            }
            2 => {
                state.hdma.objects[0].instruction = state.hdma.channels[0].source.clone();
                ds::DisplayError::Source
            }
            3 => {
                state.hdma.channels[0].terminated = true;
                state.hdma.channels[0].table = hs::SourceBinding::Unbound;
                ds::DisplayError::Incomplete
            }
            4 => {
                state.hdma.objects[0].instruction_timer = 0x8000;
                state.hdma.objects[0].instruction = hs::SourceBinding::Unbound;
                ds::DisplayError::Incomplete
            }
            5 => {
                state.hdma_live = hs::HdmaLiveDescription::new(vec![], vec![]).unwrap();
                ds::DisplayError::Source
            }
            _ => {
                state.hdma.channels[0].source = hs::SourceBinding::Unbound;
                ds::DisplayError::Incomplete
            }
        };
        let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
            .and_then(|start| display.reset(start));
        let ss::StartError::Display(error) = result.err().unwrap() else {
            panic!("expected HDMA source admission rejection")
        };
        assert_eq!(error, expected, "HDMA source admission {case}");
        scene_owner_unchanged(&display, &before, &opened.metadata);
    }
}

// Movies cannot author malformed portable bank windows independently of transfer execution.
#[cfg_attr(test, test)]
fn scene_component_rejects_invalid_hdma_bank_windows() {
    let (requirements, start, media) = scene_component_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    for case in 0..7 {
        let (mut description, media) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        let state = description.display.as_mut().unwrap();
        let hs::SourceBinding::Prepared { windows, .. } = &mut state.hdma.channels[0].table else {
            panic!("fixture table binding")
        };
        match case {
            0 => *windows = vec![].into(),
            1 => Arc::make_mut(windows)[0].length = 5,
            2 => Arc::make_mut(windows)[0].address = 0xfffe,
            3 => Arc::make_mut(windows)[0].offset = 2,
            4 => {
                *windows = vec![
                    hs::BankWindow {
                        bank: 0x88,
                        address: 0x9000,
                        offset: 0,
                        length: 3,
                    },
                    hs::BankWindow {
                        bank: 0x88,
                        address: 0x9002,
                        offset: 2,
                        length: 2,
                    },
                ]
                .into();
            }
            5 => Arc::make_mut(windows)[0].length = 0,
            _ => Arc::make_mut(windows)[0].offset = usize::MAX,
        }
        let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
            .and_then(|start| display.reset(start));
        assert!(matches!(
            result.err().unwrap(),
            ss::StartError::Display(ds::DisplayError::Source)
        ));
        scene_owner_unchanged(&display, &before, &opened.metadata);
    }
}

// Movies cannot author explicit dormant Unbound sources or inspect their persisted portable descriptions.
#[cfg_attr(test, test)]
fn scene_component_admits_explicitly_disabled_hdma_sources() {
    let (requirements, mut start, media) = scene_component_fixture();
    let state = start.display.as_mut().unwrap();
    for channel in &mut state.hdma.channels {
        channel.active = false;
        channel.dma_active = false;
        channel.terminated = true;
        channel.table = hs::SourceBinding::Unbound;
        channel.source = hs::SourceBinding::Unbound;
    }
    for binding in &mut state.hdma.bindings {
        binding.enable = 0;
    }
    for object in &mut state.hdma.objects {
        object.instruction = hs::SourceBinding::Unbound;
        object.instruction_timer = 0x8000;
    }
    let expected = format!("{:?}", start.display.as_ref().unwrap());
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let (description, _) = opened
        .display
        .checkpoint()
        .unwrap()
        .into_start_description(opened.metadata);
    assert_eq!(
        format!("{:?}", description.display.as_ref().unwrap()),
        expected
    );
    let state = description.display.unwrap();
    assert_eq!(state.hdma.flags, 0x8000);
    assert!(state.hdma.channels.iter().all(|channel| matches!(
        (&channel.table, &channel.source),
        (hs::SourceBinding::Unbound, hs::SourceBinding::Unbound)
    )));
    assert!(
        state
            .hdma
            .objects
            .iter()
            .all(|object| matches!(object.instruction, hs::SourceBinding::Unbound))
    );
}

fn scene_hdma_live_window(
    owner: hs::LiveOwner,
    bank: u8,
    address: u16,
    offset: usize,
    length: usize,
) -> hs::SourceBinding {
    hs::SourceBinding::LiveOwner {
        owner,
        windows: vec![hs::BankWindow {
            bank,
            address,
            offset,
            length,
        }]
        .into(),
    }
}

fn scene_hdma_bound_byte(
    state: &mut ds::DisplayDescription,
    catalog: &hs::HdmaCatalog,
    channel: usize,
    bank: u8,
    address: u16,
) -> u8 {
    state.hdma.channels[channel]
        .source
        .clone()
        .read(
            catalog,
            &mut state.hdma_live,
            &mut state.hdma,
            hs::ContentKind::Indirect,
            bank,
            address,
        )
        .unwrap()
}

// Movies cannot author portable overlapping physical windows with independent expected backing bytes.
fn scene_coherent_hdma_alias_fixture() -> (
    ss::DisplayStartRequirements,
    ss::DisplayStartDescription,
    Arc<sc::SceneMedia>,
) {
    let (requirements, mut start, media) = scene_component_fixture();
    let state = start.display.as_mut().unwrap();
    state.hdma.channels[1].source =
        scene_hdma_live_window(hs::LiveOwner::WindowLeft, 0x7e, 0x9801, 0, 2);
    state.hdma.channels[1].indirect_cursor = 0x9801;
    state.hdma.channels[2].source =
        scene_hdma_live_window(hs::LiveOwner::HdmaTable1, 0x7e, 0x9802, 2, 2);
    state.hdma.channels[2].indirect_cursor = 0x9802;
    state.hdma.channels[3].source = scene_hdma_live_window(
        hs::LiveOwner::ObjectVariable {
            object: 0,
            variable: hs::HdmaVariable::A,
        },
        0x7e,
        0x9000,
        0,
        2,
    );
    state.hdma.channels[3].indirect_cursor = 0x9000;
    state.hdma.channels[4].source =
        scene_hdma_live_window(hs::LiveOwner::WindowLeft, 0x7f, 0x9800, 0, 2);
    state.hdma.channels[4].indirect_bank = 0x7f;
    (requirements, start, media)
}

// Movies cannot author individually valid portable bindings that conflict at the same physical byte.
#[cfg_attr(test, test)]
fn scene_component_rejects_conflicting_physical_hdma_aliases() {
    let (requirements, start, media) = scene_component_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    for case in 0..5 {
        let (mut description, media) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        let state = description.display.as_mut().unwrap();
        let binding = match case {
            0 => scene_hdma_live_window(hs::LiveOwner::WindowLeft, 0x7e, 0x9800, 0, 2),
            1 => scene_hdma_live_window(
                hs::LiveOwner::ObjectVariable {
                    object: 0,
                    variable: hs::HdmaVariable::A,
                },
                0x7e,
                0x9800,
                0,
                2,
            ),
            2..=3 => {
                let mut binding = if case == 2 {
                    state.hdma.channels[1].table.clone()
                } else {
                    state.hdma.objects[0].instruction.clone()
                };
                let hs::SourceBinding::Prepared { windows, .. } = &mut binding else {
                    panic!("fixture prepared source")
                };
                let window = &mut Arc::make_mut(windows)[0];
                window.bank = 0x7e;
                window.address = 0x9800;
                binding
            }
            _ => scene_hdma_live_window(hs::LiveOwner::HdmaTable1, 0x7e, 0x9800, 1, 3),
        };
        let kind = match case {
            2 => hs::ContentKind::Table,
            3 => hs::ContentKind::Program,
            _ => hs::ContentKind::Indirect,
        };
        binding
            .validate(&requirements.hdma_content, &state.hdma_live, kind, true)
            .unwrap();
        match case {
            2 => state.hdma.channels[1].table = binding,
            3 => state.hdma.objects[0].instruction = binding,
            _ => state.hdma.channels[1].source = binding,
        }
        let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
            .and_then(|start| display.reset(start));
        assert!(matches!(
            result.err().unwrap(),
            ss::StartError::Display(ds::DisplayError::Source)
        ));
        scene_owner_unchanged(&display, &before, &opened.metadata);
    }
}

// Movies cannot author legal portable physical alias windows or inspect complete checkpoint/reopen state.
#[cfg_attr(test, test)]
fn scene_component_preserves_coherent_physical_hdma_aliases_on_reopen() {
    let (requirements, start, media) = scene_coherent_hdma_alias_fixture();
    let expected = format!("{:?}", start.display.as_ref().unwrap());
    let metadata = scene_metadata(&start);
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    scene_same_metadata(&opened.metadata, &metadata);
    let before = opened.display.checkpoint().unwrap();
    let (mut description, media) = before.clone().into_start_description(opened.metadata);
    assert_eq!(
        format!("{:?}", description.display.as_ref().unwrap()),
        expected
    );
    let state = description.display.as_mut().unwrap();
    assert_eq!(
        scene_hdma_bound_byte(state, &requirements.hdma_content, 0, 0x7e, 0x9801),
        0x12
    );
    assert_eq!(
        scene_hdma_bound_byte(state, &requirements.hdma_content, 1, 0x7e, 0x9801),
        0x12
    );
    assert_eq!(
        scene_hdma_bound_byte(state, &requirements.hdma_content, 2, 0x7e, 0x9802),
        0x34
    );
    assert_eq!(
        scene_hdma_bound_byte(state, &requirements.hdma_content, 3, 0x7e, 0x9000),
        0x44
    );
    assert_eq!(
        scene_hdma_bound_byte(state, &requirements.hdma_content, 4, 0x7f, 0x9800),
        0x12
    );
    drop(opened.display);
    let reopened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(description, media, &requirements).unwrap(),
    )
    .unwrap();
    scene_same_metadata(&reopened.metadata, &metadata);
    let after = reopened.display.checkpoint().unwrap();
    assert_eq!(after.committed_scene(), before.committed_scene());
    scene_same_resident(after.resident(), before.resident());
    let (description, _) = after.into_start_description(reopened.metadata);
    assert_eq!(
        format!("{:?}", description.display.as_ref().unwrap()),
        expected
    );
}

// Movies cannot edit portable physical alias storage before reset or inspect retained checkpoint bytes.
#[cfg_attr(test, test)]
fn scene_component_reset_preserves_coherent_physical_hdma_aliases() {
    let (requirements, start, media) = scene_coherent_hdma_alias_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let before = opened.display.checkpoint().unwrap();
    let (mut replacement, media) = before
        .clone()
        .into_start_description(scene_copy_metadata(&opened.metadata));
    let state = replacement.display.as_mut().unwrap();
    state
        .hdma_live
        .write(&mut state.hdma, hs::LiveOwner::WindowLeft, 0, 0xab)
        .unwrap();
    state
        .hdma_live
        .write(
            &mut state.hdma,
            hs::LiveOwner::ObjectVariable {
                object: 0,
                variable: hs::HdmaVariable::A,
            },
            1,
            0xcd,
        )
        .unwrap();
    let expected = format!("{:?}", replacement.display.as_ref().unwrap());
    let mut display = opened.display;
    let metadata = display
        .reset(ss::ValidatedDisplayStart::construct(replacement, media, &requirements).unwrap())
        .unwrap();
    scene_same_metadata(&metadata, &opened.metadata);
    let after = display.checkpoint().unwrap();
    assert_eq!(after.committed_scene(), before.committed_scene());
    scene_same_resident(after.resident(), before.resident());
    let (mut replacement, _) = after.into_start_description(metadata);
    assert_eq!(
        format!("{:?}", replacement.display.as_ref().unwrap()),
        expected
    );
    let state = replacement.display.as_mut().unwrap();
    assert_eq!(
        scene_hdma_bound_byte(state, &requirements.hdma_content, 0, 0x7e, 0x9801),
        0xab
    );
    assert_eq!(
        scene_hdma_bound_byte(state, &requirements.hdma_content, 1, 0x7e, 0x9801),
        0xab
    );
    assert_eq!(
        scene_hdma_bound_byte(state, &requirements.hdma_content, 2, 0x7e, 0x9802),
        0x34
    );
    assert_eq!(
        scene_hdma_bound_byte(state, &requirements.hdma_content, 3, 0x7e, 0x9001),
        0xcd
    );
    assert_eq!(
        scene_hdma_bound_byte(state, &requirements.hdma_content, 4, 0x7f, 0x9800),
        0xab
    );
    assert_eq!(state.hdma.objects[0].variables[0], 0xcd44);
    let (mut retained, _) = before.into_start_description(opened.metadata);
    let state = retained.display.as_mut().unwrap();
    assert_eq!(
        scene_hdma_bound_byte(state, &requirements.hdma_content, 1, 0x7e, 0x9801),
        0x12
    );
    assert_eq!(
        scene_hdma_bound_byte(state, &requirements.hdma_content, 3, 0x7e, 0x9001),
        0x33
    );
    assert_eq!(state.hdma.objects[0].variables[0], 0x3344);
}

// Movies cannot author the explicit Small/Large identities for a portable geometry size set.
fn scene_atmospheric_variants(revision: sc::Revision) -> Arc<sc::ObjectVariants> {
    let binding = |id| sc::ObjectPartBinding {
        visual: sc::AssetRef {
            id: sc::AssetId(id),
            revision,
        },
        frame: 0,
        part: 0,
    };
    Arc::new(sc::ObjectVariants {
        size_sets: [(
            0,
            sc::ObjectSizeVariants {
                small: binding(7),
                large: binding(11),
            },
        )]
        .into(),
    })
}

// Movies cannot author the separate declared Large footprint in an otherwise unchanged component fixture.
fn scene_atmospheric_variant_media(
    mut start: ss::DisplayStartDescription,
    media: Arc<sc::SceneMedia>,
) -> (ss::DisplayStartDescription, Arc<sc::SceneMedia>) {
    let reference = |id| sc::AssetRef {
        id: sc::AssetId(id),
        revision: media.revision(),
    };
    let mut assets = media.assets().cloned().collect::<Vec<_>>();
    let sc::SceneAsset::Atlas(atlas) = assets
        .iter_mut()
        .find(|asset| asset.reference() == reference(2))
        .unwrap()
    else {
        panic!("fixture object atlas")
    };
    atlas.tile_count = 18;
    atlas.data = vec![0; 18 * 16].into();
    assets.push(sc::SceneAsset::Visual(sc::Visual {
        reference: reference(11),
        frames: vec![sc::VisualFrame {
            parts: vec![sc::VisualPart {
                atlas: reference(2),
                material: reference(6),
                tile: 0,
                offset: sc::ScreenPosition { x: 0, y: 0 },
                size: sc::ObjectSize::Large,
            }]
            .into(),
        }]
        .into(),
    }));
    let mut images = media.compiled_images().cloned().collect::<Vec<_>>();
    for image in &mut images {
        Arc::make_mut(&mut image.words)[..18 * 16].fill(0);
        let binding = Arc::make_mut(&mut image.atlases)
            .iter_mut()
            .find(|binding| binding.atlas == reference(2))
            .unwrap();
        binding.resident_data = vec![0; 18 * 16].into();
    }
    let palettes = media.palette_incarnations().cloned().collect();
    let media = Arc::new(sc::SceneMedia::new(media.revision(), assets, palettes, images).unwrap());
    // Rebuild the supplied asset spans against the explicitly expanded immutable atlas.
    let colors = media
        .palette_incarnations()
        .find(|palette| {
            palette.reference
                == sc::MediaRef {
                    asset: reference(6),
                    incarnation: 2,
                }
        })
        .unwrap()
        .colors;
    let objects = start.display.as_ref().unwrap().objects.bytes;
    start.display = Some(scene_display_description(
        &media,
        start.identity.as_ref().unwrap().gameplay,
        colors,
        objects,
    ));
    let committed = start.committed_scene.take().unwrap();
    let mut description = committed.description().clone();
    let mut resident = description.resident.to_vec();
    resident.push(reference(11));
    description.resident = resident.into();
    start.committed_scene = Some(
        sc::Scene::new(description, media.clone())
            .unwrap()
            .with_producer_history(Arc::from(committed.producer_history().to_vec()))
            .unwrap(),
    );
    Arc::make_mut(&mut start.resident.as_mut().unwrap().words)[..18 * 16].fill(0);
    (start, media)
}

// Movies cannot author a portable live lifecycle with retained unpublished OAM X-high.
fn scene_atmospheric_publication_fixture() -> (
    ss::DisplayStartRequirements,
    ss::DisplayStartDescription,
    Arc<sc::SceneMedia>,
) {
    use sm::game::scene::atmosphere as a;
    let (mut requirements, start, media) = scene_component_fixture();
    let (mut start, media) = scene_atmospheric_variant_media(start, media);
    let catalog = Arc::new(a::AtmosphereCatalog {
        revision: media.revision(),
        kinds: [(
            1,
            a::AtmosphericKind {
                duration_pointer: 0x8000,
                durations: vec![3].into(),
                frame_count: 1,
                tile_pointer: 0x9000,
                tiles: vec![a::AtmosphericTile {
                    native_word: 0,
                    variants: scene_atmospheric_variants(media.revision()),
                }]
                .into(),
            },
        )]
        .into(),
        sprites: std::collections::BTreeMap::new(),
    });
    requirements.atmosphere_content = catalog.clone();
    let state = start.display.as_mut().unwrap();
    state.atmosphere_catalog = catalog;
    state.objects.bytes = [0; 544];
    state.objects.bytes[512] = 1;
    state.objects.cursor = 0;
    state.objects.produced = vec![];
    state.atmospheric.slots[0] = ds::AtmosphericSlot {
        remaining: 2,
        x: 104,
        y: 80,
        frame_and_kind: 0x0100,
    };
    state.producer_ids = sc::DisplayProducerIds {
        next: sc::ProducerObjectId(24),
    };
    state.atmospheric_producers = a::AtmosphericProducers {
        slots: [Some(sc::ProducerObjectId(23)), None, None, None],
    };
    (requirements, start, media)
}

// Movies cannot author portable lifecycle IDs or retain unpublished OAM X-high at owned publication.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_atmospheric_publication_retains_x_high_and_reuses_lifecycle() {
    for created_slot in [None, Some(0)] {
        let (requirements, start, media) = scene_atmospheric_publication_fixture();
        let revision = media.revision();
        let reference = |id| sc::AssetRef {
            id: sc::AssetId(id),
            revision,
        };
        let context = sc::ObjectContext {
            selection: sc::ObjectSelection {
                atlases: [reference(2); 2],
                size_set: 0,
                first_order: None,
                interlaced: false,
                even_frame: false,
            },
            palette: sc::MediaRef {
                asset: reference(6),
                incarnation: 1,
            },
        };
        let start = ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap();
        let (state, result) =
            sm::verify_owned_atmosphere_prefix(start, [0, 0], 0, context, created_slot).unwrap();
        assert_eq!(result, Ok(()));
        assert_eq!(state.phase, ds::DisplayPhase::Producing);
        assert_eq!(&state.objects.bytes[..4], &[100, 76, 0, 0]);
        assert_eq!((state.objects.bytes[512], state.objects.cursor), (1, 4));
        let (producer, next) = if created_slot.is_some() {
            (sc::ProducerObjectId(24), sc::ProducerObjectId(25))
        } else {
            (sc::ProducerObjectId(23), sc::ProducerObjectId(24))
        };
        assert_eq!(state.producer_ids.next, next);
        assert_eq!(
            state.atmospheric_producers.slots,
            [Some(producer), None, None, None]
        );
        assert_eq!(
            (
                state.atmospheric.slots[0].remaining,
                state.atmospheric.slots[0].frame_and_kind
            ),
            (1, 0x0100)
        );
        assert_eq!(
            state.objects.produced,
            vec![sc::ProducedObject {
                producer,
                variants: scene_atmospheric_variants(revision),
                context,
                native_word: 0,
                object: sc::SceneObject {
                    order: 0,
                    visual: reference(7),
                    frame: 0,
                    part: 0,
                    atlas: reference(2),
                    material: reference(6),
                    tile: 0,
                    position: sc::ScreenPosition { x: 356, y: 76 },
                    size: sc::ObjectSize::Small,
                    palette: 0,
                    priority: 0,
                    flip_horizontal: false,
                    flip_vertical: false,
                },
            }]
        );
    }
}
// Movies cannot author a palette incarnation shared by supplied resident and portable producer IDs.
fn scene_atmospheric_settlement_media(
    mut start: ss::DisplayStartDescription,
    media: Arc<sc::SceneMedia>,
) -> (ss::DisplayStartDescription, Arc<sc::SceneMedia>) {
    let resident = start.resident.as_mut().unwrap();
    resident.palette.asset.id = sc::AssetId(6);
    let mut palettes = media.palette_incarnations().cloned().collect::<Vec<_>>();
    palettes.push(sc::PaletteIncarnation {
        reference: resident.palette,
        colors: resident.colors,
    });
    let mut images = media.compiled_images().cloned().collect::<Vec<_>>();
    images
        .iter_mut()
        .find(|image| image.reference == resident.image)
        .unwrap()
        .palette = resident.palette;
    let media = Arc::new(
        sc::SceneMedia::new(
            media.revision(),
            media.assets().cloned().collect(),
            palettes,
            images,
        )
        .unwrap(),
    );
    let scene = start.committed_scene.take().unwrap();
    start.committed_scene = Some(
        sc::Scene::new(scene.description().clone(), media.clone())
            .unwrap()
            .with_producer_history(Arc::from(scene.producer_history().to_vec()))
            .unwrap(),
    );
    (start, media)
}

// Movies cannot supply independent portable staging/resident producers or a quiesced authored settlement.
fn scene_atmospheric_settlement_fixture(
    pending: u8,
) -> (
    ss::DisplayStartRequirements,
    ss::DisplayStartDescription,
    Arc<sc::SceneMedia>,
) {
    let (requirements, start, media) = scene_atmospheric_publication_fixture();
    let (mut start, media) = scene_atmospheric_settlement_media(start, media);
    let state = start.display.as_mut().unwrap();
    state.nmi.pending = pending;
    state.visual.colors[0] = 31;
    state.visual.animated_tiles.flags = 0;
    for half in &mut state.samus {
        half.enabled = 0;
    }
    for scroll in &mut state.scrolling {
        scroll.enabled = 0;
    }
    state.uploads.clear();
    state.upload_tail = 0;
    state.mode7_uploads.clear();
    state.mode7_tail = 0;
    state.readbacks.clear();
    state.readback_tail = 0;
    state.actor_tiles.pending = 0;
    state.controls.nmi_enable = 0x80;
    state.controls.hdma_enable = 0;
    state.controls.main_layers = 0;
    state.controls.sub_layers = 0;
    state.controls.color_selection = 0;
    state.controls.color_math = 0;
    state.controls.geometry = 0;
    for binding in &mut state.hdma.bindings {
        binding.enable = 0;
    }
    for channel in &mut state.hdma.channels {
        channel.active = false;
    }
    state.delivery.horizontal_irq_enabled = false;
    state.delivery.vertical_irq_enabled = false;
    state.delivery.irq_wanted = false;
    state.delivery.in_irq = false;
    state.delivery.nmi_wanted = false;
    state.effective.forced_blank = false;
    state.effective.main_layers = 0;
    state.effective.color_clip = 0;
    state.effective.color_prevent_math = 0;
    state.effective.color_add_subscreen = false;
    state.effective.color_math_enabled = [false; 6];
    state.effective.color_subtract = false;
    state.effective.color_half = false;
    for entry in state.objects.bytes[..512].chunks_exact_mut(4) {
        entry[1] = 240;
    }
    let resident = start.resident.as_mut().unwrap();
    resident.objects = state.objects.bytes;
    resident.objects[..4].copy_from_slice(&[40, 20, 0, 0]);
    resident.objects[512] = 0;
    let reference = |id| sc::AssetRef {
        id: sc::AssetId(id),
        revision: media.revision(),
    };
    let context = scene_atmospheric_settlement_context(&media, 3);
    resident.produced = Arc::from([sc::ProducedObject {
        producer: sc::ProducerObjectId(22),
        variants: scene_atmospheric_variants(media.revision()),
        context,
        native_word: 0,
        object: sc::SceneObject {
            order: 0,
            visual: reference(7),
            frame: 0,
            part: 0,
            atlas: reference(2),
            material: reference(6),
            tile: 0,
            position: sc::ScreenPosition { x: 40, y: 20 },
            size: sc::ObjectSize::Small,
            palette: 0,
            priority: 0,
            flip_horizontal: false,
            flip_vertical: false,
        },
    }]);
    (requirements, start, media)
}

// Movies cannot supply authored object-context and later resident incarnation identities.
fn scene_atmospheric_settlement_context(
    media: &sc::SceneMedia,
    incarnation: u64,
) -> sc::ObjectContext {
    let reference = |id| sc::AssetRef {
        id: sc::AssetId(id),
        revision: media.revision(),
    };
    sc::ObjectContext {
        selection: sc::ObjectSelection {
            atlases: [reference(2); 2],
            size_set: 0,
            first_order: None,
            interlaced: false,
            even_frame: false,
        },
        palette: sc::MediaRef {
            asset: reference(6),
            incarnation,
        },
    }
}

// Movies cannot construct the reset seam with a delivered reused ID and retained no-raster visible history.
#[cfg(sm_verify_dll)]
fn scene_atmospheric_settlement_continue(
    mut display: sd::DisplayComponent,
    metadata: sd::DisplayStartMetadata,
    requirements: &ss::DisplayStartRequirements,
) -> (
    sd::DisplayComponent,
    ds::DisplayDescription,
    sm::game::scene::ResidentDisplayDescription,
) {
    let checkpoint = display.checkpoint().unwrap();
    let visible = checkpoint.committed_scene().clone();
    let before = checkpoint.resident().clone();
    let (description, media) = checkpoint
        .clone()
        .into_start_description(scene_copy_metadata(&metadata));
    let reset = display
        .reset(
            ss::ValidatedDisplayStart::construct(description, media.clone(), requirements).unwrap(),
        )
        .unwrap();
    scene_same_metadata(&reset, &metadata);
    scene_same_resident(display.checkpoint().unwrap().resident(), &before);
    assert_eq!(
        display.committed_scene().producer_history(),
        visible.producer_history()
    );
    let (mut description, media) = checkpoint.into_start_description(metadata);
    let state = description.display.as_mut().unwrap();
    state.nmi.pending = 1;
    state.delivery.nmi_wanted = true;
    state.objects.cursor = 0;
    state.atmospheric.slots[0] = ds::AtmosphericSlot {
        remaining: 2,
        x: 108,
        y: 84,
        frame_and_kind: 0x0100,
    };
    let mut context = scene_atmospheric_settlement_context(&media, 4);
    context.selection.even_frame = true;
    let image = sc::MediaRef {
        asset: before.image.asset,
        incarnation: 5,
    };
    let palette = sc::MediaRef {
        asset: before.palette.asset,
        incarnation: 5,
    };
    let metadata = display
        .reset(ss::ValidatedDisplayStart::construct(description, media, requirements).unwrap())
        .unwrap();
    let (display, state, resident, rows, result) = sm::verify_owned_atmosphere_settlement(
        display,
        ([0, 0], 0, context),
        [image, palette],
        Some(0),
        None,
    );
    assert_eq!(result, Ok(()));
    assert!(rows.is_empty());
    assert_eq!(state.phase, ds::DisplayPhase::Committed);
    assert_eq!(state.producer_ids.next, sc::ProducerObjectId(25));
    assert_eq!(
        state.atmospheric_producers.slots,
        [Some(sc::ProducerObjectId(24)), None, None, None]
    );
    assert_eq!(resident.produced[0].producer, sc::ProducerObjectId(24));
    assert_eq!(&resident.objects[..4], &[104, 80, 0, 0]);
    assert_eq!(
        resident.produced[0].object.position,
        sc::ScreenPosition { x: 360, y: 80 }
    );
    assert_eq!((resident.image, resident.palette), (image, palette));
    assert_eq!(display.committed_scene().tick(), visible.tick() + 1);
    assert_eq!(display.committed_scene().lines(), visible.lines());
    assert_eq!(
        display.committed_scene().producer_history(),
        visible.producer_history()
    );
    assert_ne!(
        resident.produced[0].producer,
        visible.producer_history()[0][0].producer
    );
    scene_atmospheric_settlement_reopen(&display, metadata, requirements);
    (display, state, resident)
}

// Movies cannot author pending/skipped portable publication ownership and separate retained visible IDs.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_atmospheric_settlement_delivers_and_checkpoints_visible_history() {
    for pending in [0, 1] {
        let (requirements, start, media) = scene_atmospheric_settlement_fixture(pending);
        let mut metadata = scene_metadata(&start);
        let original = start.committed_scene.as_ref().unwrap().clone();
        assert_eq!(original.tick(), 0);
        let original_resident = start.resident.as_ref().unwrap().clone();
        let context = scene_atmospheric_settlement_context(&media, 3);
        let image = sc::MediaRef {
            asset: original_resident.image.asset,
            incarnation: 4,
        };
        let palette = sc::MediaRef {
            asset: original_resident.palette.asset,
            incarnation: 4,
        };
        let display = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap()
        .display;
        let (display, state, resident, rows, result) = sm::verify_owned_atmosphere_settlement(
            display,
            ([0, 0], 0, context),
            [image, palette],
            None,
            None,
        );
        assert_eq!(result, Ok(()));
        scene_atmospheric_settlement_adoption(
            pending,
            &state,
            &resident,
            &original_resident,
            &rows,
        );
        let delivered = sc::ProducerObjectId(if pending == 0 { 22 } else { 23 });
        let scene = display.committed_scene();
        assert_eq!(scene.tick(), original.tick() + 1);
        assert_eq!(scene.tick(), 1);
        assert_eq!(scene.producer_history().len(), 224);
        for (line, history) in scene.lines().iter().zip(scene.producer_history()) {
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].producer, delivered);
            assert_eq!(
                history[0].object.position,
                if pending == 0 {
                    sc::ScreenPosition { x: 40, y: 20 }
                } else {
                    sc::ScreenPosition { x: 356, y: 76 }
                }
            );
            assert_eq!(&history[0].object, &line.objects[0]);
            assert_eq!(history[0].context.palette, palette);
            assert_eq!(history[0].context.selection, line.object_selection);
            assert!(line.object_selection.even_frame);
        }
        let pixels = rows.iter().flat_map(|row| row.pixels).collect::<Vec<_>>();
        assert_eq!(scene.render().unwrap().pixels, pixels);
        scene_same_resident(display.checkpoint().unwrap().resident(), &resident);
        assert_eq!(original.producer_history().len(), 224);
        assert!(
            original
                .producer_history()
                .iter()
                .all(|records| records.is_empty())
        );
        metadata.identity.committed_tick = scene.tick();
        metadata.continuation.phase = ss::CommittedPhase::SettledInterval;
        let (_, continued, later) =
            scene_atmospheric_settlement_continue(display, metadata, &requirements);
        assert_eq!(continued.nmi.including_lag, 0xabcf);
        assert_eq!(
            (continued.nmi.frame_byte, continued.nmi.frame_word),
            if pending == 0 {
                (12, 0x1235)
            } else {
                (13, 0x1236)
            }
        );
        assert_eq!(later.colors, continued.visual.colors);
    }
}

// Movies cannot force semantic admission to fail after completed numeric raster effects.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_atmospheric_settlement_poison_preserves_numeric_prefix() {
    let (requirements, start, media) = scene_atmospheric_settlement_fixture(1);
    let original = start.committed_scene.as_ref().unwrap().clone();
    let resident = start.resident.as_ref().unwrap();
    let context = scene_atmospheric_settlement_context(&media, 3);
    let image = sc::MediaRef {
        asset: resident.image.asset,
        incarnation: 4,
    };
    let palette = sc::MediaRef {
        asset: resident.palette.asset,
        incarnation: 4,
    };
    let display = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap()
    .display;
    let (display, state, resident, rows, result) = sm::verify_owned_atmosphere_settlement(
        display,
        ([0, 0], 0, context),
        [image, palette],
        None,
        Some(2),
    );
    assert_eq!(result, Err(ds::DisplayError::Scene));
    assert_eq!(state.phase, ds::DisplayPhase::Failed);
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter()
            .all(|row| row.pixels.iter().all(|pixel| *pixel == [0, 0, 255, 0]))
    );
    assert_eq!(
        (
            state.delivery.vertical_position,
            state.delivery.horizontal_position
        ),
        (2, 512)
    );
    assert_eq!(
        (
            state.nmi.pending,
            state.nmi.frame_word,
            state.nmi.including_lag
        ),
        (0, 0x1235, 0xabce)
    );
    assert_eq!(state.producer_ids.next, sc::ProducerObjectId(24));
    assert_eq!(resident.produced[0].producer, sc::ProducerObjectId(23));
    assert_eq!(&resident.objects[..4], &[100, 76, 0, 0]);
    assert_eq!((resident.image, resident.palette), (image, palette));
    assert_eq!(
        display.committed_scene().description(),
        original.description()
    );
    assert_eq!(
        display.committed_scene().producer_history(),
        original.producer_history()
    );
    assert!(matches!(display.checkpoint(), Err(ds::DisplayError::Phase)));
}
// Movies cannot independently supply pending and skipped numeric resident publication states.
#[cfg(sm_verify_dll)]
fn scene_atmospheric_settlement_adoption(
    pending: u8,
    state: &ds::DisplayDescription,
    resident: &sm::game::scene::ResidentDisplayDescription,
    original_resident: &sm::game::scene::ResidentDisplayDescription,
    rows: &[sm::rendering::prepared::RenderedLine],
) {
    assert_eq!(state.phase, ds::DisplayPhase::Committed);
    assert_eq!(rows.len(), 224);
    assert_eq!(state.producer_ids.next, sc::ProducerObjectId(24));
    assert_eq!(
        state.atmospheric_producers.slots,
        [Some(sc::ProducerObjectId(23)), None, None, None]
    );
    assert_eq!(state.objects.produced[0].producer, sc::ProducerObjectId(23));
    assert_eq!(&state.objects.bytes[..4], &[100, 76, 0, 0]);
    let delivered = sc::ProducerObjectId(if pending == 0 { 22 } else { 23 });
    assert_eq!(resident.produced[0].producer, delivered);
    assert_eq!(
        resident.objects,
        if pending == 0 {
            original_resident.objects
        } else {
            state.objects.bytes
        }
    );
    assert_eq!(
        resident.colors,
        if pending == 0 {
            original_resident.colors
        } else {
            state.visual.colors
        }
    );
    assert_eq!(
        (state.nmi.pending, state.nmi.missed, state.nmi.max_missed),
        if pending == 0 { (0, 4, 5) } else { (0, 0, 5) }
    );
    assert_eq!(
        (
            state.nmi.frame_byte,
            state.nmi.frame_word,
            state.nmi.including_lag
        ),
        if pending == 0 {
            (11, 0x1234, 0xabce)
        } else {
            (12, 0x1235, 0xabce)
        }
    );
    let expected_pixel = if pending == 0 {
        [255, 0, 0, 0]
    } else {
        [0, 0, 255, 0]
    };
    assert!(
        rows.iter()
            .all(|row| row.pixels.iter().all(|pixel| *pixel == expected_pixel))
    );
}

// Movies cannot supply complete component checkpoint metadata after a second settled display interval.
#[cfg(sm_verify_dll)]
fn scene_atmospheric_settlement_reopen(
    display: &sd::DisplayComponent,
    mut metadata: sd::DisplayStartMetadata,
    requirements: &ss::DisplayStartRequirements,
) {
    let checkpoint = display.checkpoint().unwrap();
    let resident = checkpoint.resident().clone();
    let scene = checkpoint.committed_scene().clone();
    assert_eq!(scene.tick(), 2);
    metadata.identity.committed_tick = scene.tick();
    let (description, media) = checkpoint.into_start_description(metadata);
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(description, media, requirements).unwrap(),
    )
    .unwrap();
    scene_same_resident(opened.display.checkpoint().unwrap().resident(), &resident);
    assert_eq!(
        opened.display.committed_scene().description(),
        scene.description()
    );
    assert_eq!(
        opened.display.committed_scene().producer_history(),
        scene.producer_history()
    );
}
// Movies cannot supply portable Samus catalogue parts or independently chosen semantic identities.
fn scene_namespace_samus_catalog(
    media: &sc::SceneMedia,
) -> Arc<sm::game::scene::samus::SamusCatalog> {
    use sm::game::scene::samus::{SamusCatalog, SamusMap, SamusPart};
    Arc::new(SamusCatalog {
        revision: media.revision(),
        pointers: Arc::from([0x8000]),
        maps: [(
            0x8000,
            SamusMap {
                count: 1,
                parts: Arc::from([SamusPart {
                    x: 0,
                    y: 0,
                    native_word: 0,
                    variants: scene_atmospheric_variants(media.revision()),
                }]),
            },
        )]
        .into(),
    })
}

// Movies cannot supply old staged semantic records independently of a later failed Samus identity.
fn scene_namespace_failed_id_fixture() -> (
    ss::DisplayStartRequirements,
    ss::DisplayStartDescription,
    Arc<sc::SceneMedia>,
) {
    let (requirements, mut start, media) = scene_atmospheric_settlement_fixture(1);
    let mut old = start.resident.as_ref().unwrap().produced[0].clone();
    old.object.position.x = 296;
    let mut untouched = old.clone();
    untouched.producer = sc::ProducerObjectId(23);
    untouched.object.order = 1;
    untouched.object.position.x = 40;
    let state = start.display.as_mut().unwrap();
    state.objects.bytes[..8].copy_from_slice(&[40, 20, 0, 0, 40, 20, 0, 0]);
    state.objects.produced = vec![old, untouched];
    (requirements, start, media)
}

// Movies cannot supply common portable lifecycle IDs interleaved between atmospheric births and Samus.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_namespace_interleaves_birth_samus_retirement_and_reuse() {
    let (requirements, start, media) = scene_atmospheric_settlement_fixture(1);
    let catalog = scene_namespace_samus_catalog(&media);
    catalog.validate(&media).unwrap();
    let context = scene_atmospheric_settlement_context(&media, 3);
    let display = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap()
    .display;
    let (states, result) =
        sm::verify_owned_namespace_interleaved(display, catalog, ([0, 0], 0, context));
    assert_eq!(result, Ok(()));
    assert_eq!(states.len(), 6);
    let expected = [
        (25, Some(24), 0, vec![]),
        (26, Some(24), 4, vec![25]),
        (26, Some(24), 8, vec![25, 24]),
        (26, None, 8, vec![25, 24]),
        (27, Some(26), 8, vec![25, 24]),
        (27, Some(26), 16, vec![25, 24, 25, 26]),
    ];
    for (state, (next, slot, cursor, produced)) in states.iter().zip(expected) {
        assert_eq!(state.phase, ds::DisplayPhase::Producing);
        assert_eq!(state.producer_ids.next, sc::ProducerObjectId(next));
        assert_eq!(
            state.atmospheric_producers.slots,
            [slot.map(sc::ProducerObjectId), None, None, None]
        );
        assert_eq!(state.objects.cursor, cursor);
        assert_eq!(
            state
                .objects
                .produced
                .iter()
                .map(|record| record.producer.0)
                .collect::<Vec<_>>(),
            produced
        );
    }
    assert_eq!(&states[1].objects.bytes[..4], &[200, 60, 0, 0]);
    assert_eq!(
        states[1].objects.produced[0].object.position,
        sc::ScreenPosition { x: 456, y: 60 }
    );
    assert_eq!(&states[2].objects.bytes[4..8], &[100, 76, 0, 0]);
    assert_eq!(
        states[2].objects.produced[1].object.position,
        sc::ScreenPosition { x: 100, y: 76 }
    );
    assert_eq!(
        (
            states[3].atmospheric.slots[0].remaining,
            states[3].atmospheric.slots[0].frame_and_kind
        ),
        (3, 0)
    );
    assert_eq!(states[3].objects.produced, states[2].objects.produced);
    assert_eq!(states[4].objects.produced, states[3].objects.produced);
    assert_eq!(
        states[5].objects.produced[2].object.position,
        sc::ScreenPosition { x: 200, y: 60 }
    );
    assert_eq!(
        states[5].objects.produced[3].object.position,
        sc::ScreenPosition { x: 100, y: 76 }
    );
    assert_eq!(states[5].objects.bytes[512], 1);
}

// Movies cannot inject zero/exhausted owned namespaces or invalid atmospheric slots at birth admission.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_namespace_birth_failures_preserve_counter_and_slot_prefixes() {
    for (next, slot, error) in [
        (0, 0, ds::DisplayError::Identity),
        (u64::MAX, 0, ds::DisplayError::Identity),
        (24, 4, ds::DisplayError::Geometry),
        (0, 4, ds::DisplayError::Geometry),
        (u64::MAX, 4, ds::DisplayError::Geometry),
    ] {
        let (requirements, start, media) = scene_atmospheric_settlement_fixture(1);
        let before = start.display.as_ref().unwrap().clone();
        let display = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap()
        .display;
        let (state, result) =
            sm::verify_owned_namespace_birth_prefix(display, sc::ProducerObjectId(next), slot);
        assert_eq!(result, Err(error));
        assert_eq!(state.phase, ds::DisplayPhase::Failed);
        assert_eq!(state.producer_ids.next, sc::ProducerObjectId(next));
        assert_eq!(state.atmospheric_producers, before.atmospheric_producers);
        assert_eq!(state.atmospheric, before.atmospheric);
        assert_eq!(state.objects.bytes, before.objects.bytes);
        assert_eq!(state.objects.cursor, before.objects.cursor);
        assert_eq!(state.objects.produced, before.objects.produced);
        assert_eq!((state.nmi.pending, state.nmi.frame_word), (1, 0x1234));
    }
}

// Movies cannot supply an invalid portable Samus ID after native-equivalent OAM stores complete.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_namespace_samus_id_failure_keeps_completed_oam_and_other_records() {
    for (producer, error) in [
        (0, ds::DisplayError::Scene),
        (25, ds::DisplayError::Identity),
        (24, ds::DisplayError::Identity),
        (u64::MAX, ds::DisplayError::Identity),
    ] {
        let (requirements, start, media) = scene_namespace_failed_id_fixture();
        let before = start.display.as_ref().unwrap().clone();
        let catalog = scene_namespace_samus_catalog(&media);
        catalog.validate(&media).unwrap();
        let context = scene_atmospheric_settlement_context(&media, 3);
        let display = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap()
        .display;
        let (state, result) = sm::verify_owned_samus_id_prefix(
            display,
            catalog,
            context,
            sc::ProducerObjectId(producer),
        );
        assert_eq!(result, Err(error));
        assert_eq!(state.phase, ds::DisplayPhase::Failed);
        let mut expected = before.objects.bytes;
        expected[..4].copy_from_slice(&[100, 76, 0, 0]);
        assert_eq!(state.objects.bytes, expected);
        assert_eq!(state.objects.cursor, 0);
        assert_eq!(state.producer_ids.next, sc::ProducerObjectId(24));
        assert_eq!(state.atmospheric_producers, before.atmospheric_producers);
        assert_eq!(
            state.objects.produced,
            vec![before.objects.produced[1].clone()]
        );
        assert_eq!(state.objects.produced[0].producer, sc::ProducerObjectId(23));
        assert_eq!((state.nmi.pending, state.nmi.frame_word), (1, 0x1234));
    }
}
// Movies cannot supply portable marker tile33 variants without an explicit independent atlas/material.
fn scene_namespace_marker_fixture() -> (
    ss::DisplayStartRequirements,
    ss::DisplayStartDescription,
    Arc<sc::SceneMedia>,
    Arc<sc::ObjectVariants>,
) {
    let (requirements, mut start, media) = scene_atmospheric_settlement_fixture(1);
    let reference = |id| sc::AssetRef {
        id: sc::AssetId(id),
        revision: media.revision(),
    };
    let data: Arc<[u16]> = vec![0; 51 * 16].into();
    let mut assets = media.assets().cloned().collect::<Vec<_>>();
    assets.push(sc::SceneAsset::Atlas(sc::TileAtlas {
        reference: reference(12),
        encoding: sc::TileEncoding::Planar4,
        tile_count: 51,
        data: data.clone(),
    }));
    for (id, size) in [(13, sc::ObjectSize::Small), (14, sc::ObjectSize::Large)] {
        assets.push(sc::SceneAsset::Visual(sc::Visual {
            reference: reference(id),
            frames: Arc::from([sc::VisualFrame {
                parts: Arc::from([sc::VisualPart {
                    atlas: reference(12),
                    material: reference(6),
                    tile: 33,
                    offset: sc::ScreenPosition { x: 0, y: 0 },
                    size,
                }]),
            }]),
        }));
    }
    let mut images = media.compiled_images().cloned().collect::<Vec<_>>();
    for image in &mut images {
        Arc::make_mut(&mut image.words)[0x1000..0x1000 + 51 * 16].fill(0);
        let mut bindings = image.atlases.to_vec();
        bindings.push(sc::AtlasBinding {
            atlas: reference(12),
            first_word: 0x1000,
            resident_data: data.clone(),
        });
        image.atlases = bindings.into();
    }
    let palettes = media.palette_incarnations().cloned().collect();
    let media = Arc::new(sc::SceneMedia::new(media.revision(), assets, palettes, images).unwrap());
    let old = start.committed_scene.take().unwrap();
    let mut description = old.description().clone();
    let mut resident = description.resident.to_vec();
    resident.extend([reference(12), reference(13), reference(14)]);
    description.resident = resident.into();
    start.committed_scene = Some(
        sc::Scene::new(description, media.clone())
            .unwrap()
            .with_producer_history(Arc::from(old.producer_history().to_vec()))
            .unwrap(),
    );
    let resident = start.resident.as_mut().unwrap();
    resident.words = media
        .compiled_images()
        .find(|image| image.reference == resident.image)
        .unwrap()
        .words
        .clone();
    let mut variants = (*scene_atmospheric_variants(media.revision())).clone();
    let binding = variants.size_sets.get_mut(&0).unwrap();
    binding.small.visual = reference(13);
    binding.large.visual = reference(14);
    (requirements, start, media, Arc::new(variants))
}

// Movies cannot author retained marker X-high or cursor-alias low-byte failures after native overlap.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_namespace_marker_retains_high_x_and_overlapping_word_prefixes() {
    use sm::game::scene::samus_body::MarkerPublication;
    for (cursor, producer, expected_cursor, error) in [
        (0, 0, 4, ds::DisplayError::Scene),
        (0, 22, 4, ds::DisplayError::Source),
        (0, 24, 4, ds::DisplayError::Identity),
        (544, 22, 0xb761, ds::DisplayError::Alias { offset: 546 }),
        (545, 22, 0x6121, ds::DisplayError::Alias { offset: 546 }),
    ] {
        let (requirements, start, media, variants) = scene_namespace_marker_fixture();
        let before = start.display.as_ref().unwrap().clone();
        let mut context = scene_atmospheric_settlement_context(&media, 3);
        context.selection.atlases = [sc::AssetRef {
            id: sc::AssetId(12),
            revision: media.revision(),
        }; 2];
        let catalog = Arc::new(sm::game::scene::samus::SamusCatalog {
            revision: media.revision(),
            pointers: Arc::from([0]),
            maps: std::collections::BTreeMap::new(),
        });
        catalog.validate(&media).unwrap();
        let marker = MarkerPublication {
            producer: sc::ProducerObjectId(producer),
            variants,
        };
        marker.validate(&media).unwrap();
        let display = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap()
        .display;
        let (state, result) =
            sm::verify_owned_marker_prefix(display, catalog, context, marker, cursor);
        assert_eq!(result, Err(error));
        assert_eq!(state.phase, ds::DisplayPhase::Failed);
        let mut expected = before.objects.bytes;
        if cursor == 0 {
            expected[..4].copy_from_slice(&[97, 183, 33, 56]);
        }
        assert_eq!(state.objects.bytes, expected);
        assert_eq!(state.objects.cursor, expected_cursor);
        assert_eq!(state.objects.bytes[512], 1);
        assert_eq!(state.producer_ids.next, sc::ProducerObjectId(24));
        assert_eq!(state.atmospheric_producers, before.atmospheric_producers);
        if cursor == 0 && producer == 22 {
            assert_eq!(state.objects.produced.len(), 1);
            let record = &state.objects.produced[0];
            assert_eq!(record.producer, sc::ProducerObjectId(22));
            assert_eq!(record.native_word, 0x3821);
            assert_eq!(
                record.object.position,
                sc::ScreenPosition { x: 353, y: 183 }
            );
            assert_eq!(
                (
                    record.object.visual.id,
                    record.object.atlas.id,
                    record.object.tile
                ),
                (sc::AssetId(13), sc::AssetId(12), 33)
            );
            assert_eq!(
                (
                    record.object.palette,
                    record.object.priority,
                    record.object.size
                ),
                (4, 3, sc::ObjectSize::Small)
            );
            assert_eq!(record.context, context);
        } else {
            assert!(state.objects.produced.is_empty());
        }
    }
}

// Movies cannot supply full landing frames, native byte order or injected table-bound failures.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_samus_landing_native_words_preserve_full_frame_bounds() {
    let little = [
        -1539, -6, 0, -768, -1539, -6, 0, -768, -771, -1539, -6, -768, -771, -1539, -6,
    ];
    let big = [
        -774, -1536, 0, -3, -774, -1536, 0, -3, -771, -774, -1536, -3, -771, -774, -1536,
    ];
    for (order, expected) in [(ds::WordOrder::Little, little), (ds::WordOrder::Big, big)] {
        for (frame, offset) in expected.into_iter().enumerate() {
            let (result, events) =
                sm::verify_samus_position_offset(0x148, frame as u16, 0, order, 0x80);
            assert_eq!(result, Ok(Ok(offset)));
            assert_eq!(events, [("frame", frame, 0), ("movement", 0, 0)]);
        }
    }
    for (doubled, frame) in [
        (0x148, 15),
        (0x148, 16),
        (0x148, 255),
        (0x148, 256),
        (0x148, 0xffff),
        (0x14a, 11),
        (0x14a, 12),
        (0x14c, 7),
        (0x14c, 8),
        (0x14e, 3),
        (0x14e, 4),
    ] {
        let (result, events) =
            sm::verify_samus_position_offset(doubled, frame, 0, ds::WordOrder::Little, 0x80);
        assert_eq!(result, Err(()));
        assert_eq!(
            events,
            [("frame", usize::from(frame), 0), ("movement", 0, 0)]
        );
    }
    for (doubled, frame, expected) in [
        (0x14a, 0, -1539),
        (0x14a, 10, -6),
        (0x14c, 0, -771),
        (0x14c, 6, -6),
        (0x14e, 0, -771),
        (0x14e, 2, -6),
        (0x14a, 0xfffc, -1539),
        (0x14c, 0xfff8, -1539),
        (0x14e, 0xfff4, -1539),
    ] {
        let (result, events) =
            sm::verify_samus_position_offset(doubled, frame, 0, ds::WordOrder::Little, 0x80);
        assert_eq!(result, Ok(Ok(expected)));
        assert_eq!(
            events,
            [("frame", usize::from(frame), 0), ("movement", 0, 0)]
        );
    }
}

// Movies cannot supply every signed table entry, full-frame overflow or wrapped pose/frame alias.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_samus_signed_tables_preserve_every_entry_and_bounds() {
    let crouching = [
        -8, 0, -8, 0, -4, -2, -4, -2, 0, 0, 0, 0, -4, 0, -4, 0, 5, 4, 5, 4, 0, 0, 0, 0,
    ];
    let drained = [
        7, 5, -8, -8, -8, -8, -8, -5, 4, 4, 4, 4, 0, 0, 4, -3, -5, 0, 0, 4, -3, -5, -3, 4, 0, 0, 4,
        0, 0, 4, 0, 0,
    ];
    for (doubled, movement, expected) in [
        (0x6a, 15, crouching.as_slice()),
        (0x6b, 15, crouching.as_slice()),
        (0x1d0, 27, drained.as_slice()),
        (0x1d2, 27, drained.as_slice()),
    ] {
        for (frame, offset) in expected.iter().enumerate() {
            let (result, events) = sm::verify_samus_position_offset(
                doubled,
                frame as u16,
                movement,
                ds::WordOrder::Big,
                0x80,
            );
            assert_eq!(result, Ok(Ok(*offset)));
            assert_eq!(
                events,
                [("frame", frame, 0), ("movement", usize::from(movement), 0)]
            );
        }
    }
    for (doubled, frame, movement) in [
        (0x6a, 24, 15),
        (0x6a, 256, 15),
        (0x6a, 0xffff, 15),
        (0x6c, 22, 15),
        (0x6e, 20, 15),
        (0x70, 18, 15),
        (0x72, 16, 15),
        (0x74, 14, 15),
        (0x76, 12, 15),
        (0x78, 10, 15),
        (0x7a, 8, 15),
        (0x7c, 6, 15),
        (0x7e, 4, 15),
        (0x80, 2, 15),
        (0x1d0, 32, 27),
        (0x1d0, 256, 27),
        (0x1d2, 32, 27),
        (0x1d2, 0xffff, 27),
    ] {
        let (result, events) =
            sm::verify_samus_position_offset(doubled, frame, movement, ds::WordOrder::Little, 0x80);
        assert_eq!(result, Err(()));
        assert_eq!(
            events,
            [
                ("frame", usize::from(frame), 0),
                ("movement", usize::from(movement), 0)
            ]
        );
    }
    for (doubled, frame) in [(0x6c, 0xfffe), (0x80, 0xffea)] {
        assert_eq!(
            sm::verify_samus_position_offset(doubled, frame, 15, ds::WordOrder::Little, 0x80).0,
            Ok(Ok(-8))
        );
    }
}

// Movies cannot supply signed fallback bytes or the full unsigned/signed frame threshold boundaries.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_samus_fallback_and_frame_thresholds_keep_full_width() {
    for (fallback, expected) in [(0x80, 128), (0x7f, -127), (0xff, 1)] {
        let (result, events) =
            sm::verify_samus_position_offset(0xffff, 0xffff, 1, ds::WordOrder::Little, fallback);
        assert_eq!(result, Ok(Ok(expected)));
        assert_eq!(
            events,
            [
                ("frame", 65535, 0),
                ("movement", 1, 0),
                ("fallback", 262144, usize::from(fallback))
            ]
        );
    }
    for (doubled, source) in [(0, 4), (0x136, 1244)] {
        for (frame, expected) in [
            (0, 128),
            (1, 128),
            (2, -1),
            (0x8001, -1),
            (0x8002, 128),
            (0xffff, 128),
        ] {
            let (result, events) =
                sm::verify_samus_position_offset(doubled, frame, 0, ds::WordOrder::Little, 0x80);
            assert_eq!(result, Ok(Ok(expected)));
            let mut prefix = vec![("frame", usize::from(frame), 0), ("movement", 0, 0)];
            if expected == 128 {
                prefix.push(("fallback", source, 128));
            }
            assert_eq!(events, prefix);
        }
    }
    for (doubled, source) in [(0x1d4, 1876), (0x1d6, 1884)] {
        for (frame, expected) in [
            (0, 1),
            (4, 1),
            (5, -3),
            (0x7fff, -3),
            (0x8000, -3),
            (0x8004, -3),
            (0x8005, 1),
            (0xffff, 1),
        ] {
            let (result, events) =
                sm::verify_samus_position_offset(doubled, frame, 27, ds::WordOrder::Little, 0xff);
            assert_eq!(result, Ok(Ok(expected)));
            let mut prefix = vec![("frame", usize::from(frame), 0), ("movement", 27, 0)];
            if expected == 1 {
                prefix.push(("fallback", source, 255));
            }
            assert_eq!(events, prefix);
        }
    }
    assert_eq!(
        sm::verify_samus_position_offset(0x69, 0, 15, ds::WordOrder::Little, 0xff),
        (
            Ok(Ok(1)),
            vec![("frame", 0, 0), ("movement", 15, 0), ("fallback", 424, 255)]
        )
    );
    for movement in [28, 255] {
        let (result, events) =
            sm::verify_samus_position_offset(0, 0, movement, ds::WordOrder::Little, 0x80);
        assert_eq!(result, Err(()));
        assert_eq!(
            events,
            [("frame", 0, 0), ("movement", usize::from(movement), 0)]
        );
    }
}

// Movies cannot provide an independently supplied Ceres temporary owner and its exact query/write sequence.
#[cfg(sm_verify_dll)]
fn scene_samus_ceres_literal_events() -> Vec<(&'static str, usize, usize)> {
    vec![
        ("ceres", 0x8000, 0),
        ("player_x", 0x1234, 0),
        ("player_y", 0x2345, 0),
        ("project", 0x1234, 0x2345),
        ("set_player_x", 0xfff0, 0),
        ("set_player_y", 0x10, 0),
        ("frame", 2, 0),
        ("movement", 0, 0),
        ("player_y", 0x10, 0),
        ("camera_y", 0x20, 0),
        ("set_body_y", 0xffef, 0),
        ("player_x", 0xfff0, 0),
        ("camera_x", 0x30, 0),
        ("set_body_x", 0xffc0, 0),
        ("body_x", 0xffc0, 0),
        ("body_y", 0xffef, 0),
        ("set_player_y", 0x2345, 0),
        ("set_player_x", 0x1234, 0),
    ]
}

// Movies cannot author independent projected points to prove completed Y-before-X success restoration.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_samus_ceres_success_restores_y_then_x() {
    let (result, player, body, events) = sm::verify_samus_ceres_position(0);
    assert_eq!(result, Ok((0xffc0, 0xffef)));
    assert_eq!(player, [0x1234, 0x2345]);
    assert_eq!(body, [0xffc0, 0xffef]);
    assert_eq!(events, scene_samus_ceres_literal_events());
}

// Movies cannot inject a failure at every Ceres observation/publication and inspect completed owner prefixes.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_samus_ceres_failures_preserve_each_completed_prefix() {
    for (failure, expected_player, expected_body) in [
        (1, [0x1234, 0x2345], [0x4567, 0x5678]),
        (2, [0x1234, 0x2345], [0x4567, 0x5678]),
        (3, [0x1234, 0x2345], [0x4567, 0x5678]),
        (4, [0x1234, 0x2345], [0x4567, 0x5678]),
        (5, [0xfff0, 0x2345], [0x4567, 0x5678]),
        (6, [0xfff0, 0x10], [0x4567, 0x5678]),
        (7, [0xfff0, 0x10], [0x4567, 0x5678]),
        (8, [0xfff0, 0x10], [0x4567, 0x5678]),
        (9, [0xfff0, 0x10], [0x4567, 0x5678]),
        (10, [0xfff0, 0x10], [0x4567, 0x5678]),
        (11, [0xfff0, 0x10], [0x4567, 0xffef]),
        (12, [0xfff0, 0x10], [0x4567, 0xffef]),
        (13, [0xfff0, 0x10], [0x4567, 0xffef]),
        (14, [0xfff0, 0x10], [0xffc0, 0xffef]),
        (15, [0xfff0, 0x10], [0xffc0, 0xffef]),
        (16, [0xfff0, 0x10], [0xffc0, 0xffef]),
        (17, [0xfff0, 0x2345], [0xffc0, 0xffef]),
        (18, [0x1234, 0x2345], [0xffc0, 0xffef]),
    ] {
        let (result, player, body, events) = sm::verify_samus_ceres_position(failure);
        assert_eq!(result, Err(failure));
        assert_eq!(player, expected_player);
        assert_eq!(body, expected_body);
        let mut expected_events = scene_samus_ceres_literal_events();
        expected_events.truncate(failure);
        assert_eq!(events, expected_events);
    }
}

// Movies cannot supply known resident sentinels independently of immutable visible line history.
#[cfg(sm_verify_dll)]
fn scene_samus_upload_fixture() -> (
    ss::DisplayStartRequirements,
    ss::DisplayStartDescription,
    Arc<sc::SceneMedia>,
) {
    let (requirements, mut start, media) = scene_atmospheric_settlement_fixture(1);
    let resident = start.resident.as_mut().unwrap();
    let mut images = media.compiled_images().cloned().collect::<Vec<_>>();
    let image = images
        .iter_mut()
        .find(|image| image.reference == resident.image)
        .unwrap();
    Arc::make_mut(&mut image.words)[0x7000..0x7005]
        .copy_from_slice(&[0xaaaa, 0xbbbb, 0xcccc, 0xdddd, 0xeeee]);
    resident.words = image.words.clone();
    let media = Arc::new(
        sc::SceneMedia::new(
            media.revision(),
            media.assets().cloned().collect(),
            media.palette_incarnations().cloned().collect(),
            images,
        )
        .unwrap(),
    );
    let old = start.committed_scene.take().unwrap();
    start.committed_scene = Some(
        sc::Scene::new(old.description().clone(), media.clone())
            .unwrap()
            .with_producer_history(Arc::from(old.producer_history().to_vec()))
            .unwrap(),
    );
    (requirements, start, media)
}

// Movies cannot supply post-adoption transfer/latch observations with a zero second count.
#[cfg(sm_verify_dll)]
fn scene_samus_upload_arguments(
    start: &ss::DisplayStartDescription,
) -> ([ds::SamusHalf; 2], ds::DisplayLatches) {
    let state = start.display.as_ref().unwrap();
    let mut halves = state.samus.clone();
    halves[0].enabled = 1;
    halves[1].enabled = 0;
    halves[0].first.count = 4;
    halves[0].first.destination.word = 0x7000;
    halves[0].first.destination.step = 1;
    halves[0].first.destination.remap = 0;
    halves[0].first.destination.increment_on_high = true;
    halves[0].second.count = 0;
    halves[0].second.destination.word = 0x7003;
    halves[0].second.destination.remap = 4;
    let mut latches = state.latches;
    latches.tile_pointer = 0x7011;
    latches.tile_increment = 33;
    latches.tile_remap = 3;
    latches.tile_increment_on_high = false;
    (halves, latches)
}

// Movies cannot inject invalid inactive descriptors to observe unconditional VMAIN with no transfer.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_samus_upload_disabled_halves_still_publish_vmain() {
    let (requirements, start, media) = scene_samus_upload_fixture();
    let before = start.resident.as_ref().unwrap().clone();
    let (mut halves, latches) = scene_samus_upload_arguments(&start);
    for half in &mut halves {
        half.enabled = 0;
        half.descriptor.id = sc::AssetId(0);
        half.first.destination.remap = 255;
    }
    let display = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap()
    .display;
    let (state, resident, result) = sm::verify_owned_samus_upload(display, halves, latches);
    assert_eq!(result, Ok(()));
    assert_eq!(state.phase, ds::DisplayPhase::Producing);
    assert_eq!(
        (
            state.latches.tile_increment,
            state.latches.tile_remap,
            state.latches.tile_increment_on_high
        ),
        (1, 0, true)
    );
    assert_eq!(state.latches.tile_pointer, 0x7011);
    assert_eq!(resident.words, before.words);
    assert_eq!(resident.colors, before.colors);
    assert_eq!(resident.objects, before.objects);
}

// Movies cannot inject unused invalid second remaps to observe destination preparation after the first DMA.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_samus_upload_empty_second_prepares_destination_after_first_words() {
    let (requirements, start, media) = scene_samus_upload_fixture();
    let before = start.resident.as_ref().unwrap().clone();
    let (halves, latches) = scene_samus_upload_arguments(&start);
    let display = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap()
    .display;
    let (state, resident, result) = sm::verify_owned_samus_upload(display, halves, latches);
    assert_eq!(result, Ok(()));
    let mut expected = before.words.to_vec();
    expected[0x7000..0x7005].copy_from_slice(&[0, 0, 0xcccc, 0xdddd, 0xeeee]);
    assert_eq!(resident.words.as_ref(), expected.as_slice());
    assert_eq!(state.latches.tile_pointer, 0x7003);
    assert_eq!(
        (
            state.latches.tile_increment,
            state.latches.tile_remap,
            state.latches.tile_increment_on_high
        ),
        (1, 0, true)
    );
    assert_eq!(resident.colors, before.colors);
    assert_eq!(resident.objects, before.objects);
}

// Movies cannot inject first/second remap or late source faults and inspect completed native resident bytes.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_samus_upload_faults_retain_words_latches_and_poison() {
    for (fault, pointer, words, error) in [
        (
            0,
            0x7011,
            [0xaaaa, 0xbbbb, 0xcccc, 0xdddd, 0xeeee],
            ds::DisplayError::Geometry,
        ),
        (
            1,
            0x7001,
            [0, 0xbb00, 0xcccc, 0xdddd, 0xeeee],
            ds::DisplayError::Source,
        ),
        (
            2,
            0x7003,
            [0, 0, 0xcccc, 0xdddd, 0xeeee],
            ds::DisplayError::Geometry,
        ),
        (
            3,
            0x7004,
            [0, 0, 0xcccc, 0, 0xee00],
            ds::DisplayError::Source,
        ),
    ] {
        let (requirements, start, media) = scene_samus_upload_fixture();
        let before = start.resident.as_ref().unwrap().clone();
        let (mut halves, latches) = scene_samus_upload_arguments(&start);
        match fault {
            0 => halves[0].first.destination.remap = 4,
            1 => {
                let ds::TransferSource::Asset(source) = &mut halves[0].first.source else {
                    panic!("authored source")
                };
                source.offset = 573;
            }
            2 => halves[0].second.count = 1,
            _ => {
                halves[0].second.count = 4;
                halves[0].second.destination.remap = 0;
                let ds::TransferSource::Asset(source) = &mut halves[0].second.source else {
                    panic!("authored source")
                };
                source.offset = 573;
            }
        }
        let display = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap()
        .display;
        let (state, resident, result) = sm::verify_owned_samus_upload(display, halves, latches);
        assert_eq!(result, Err(error));
        assert_eq!(state.phase, ds::DisplayPhase::Failed);
        assert_eq!(state.latches.tile_pointer, pointer);
        assert_eq!(
            (
                state.latches.tile_increment,
                state.latches.tile_remap,
                state.latches.tile_increment_on_high
            ),
            (1, 0, true)
        );
        let mut expected = before.words.to_vec();
        expected[0x7000..0x7005].copy_from_slice(&words);
        assert_eq!(resident.words.as_ref(), expected.as_slice());
        assert_eq!(resident.colors, before.colors);
        assert_eq!(resident.objects, before.objects);
    }
}

// Movies cannot inject authored native latch padding before resident incarnation publication.
#[cfg(sm_verify_dll)]
fn scene_resident_append_latches(start: &ss::DisplayStartDescription) -> ds::DisplayLatches {
    let mut latches = start.display.as_ref().unwrap().latches;
    latches.tile_pointer = 0x1234;
    latches.tile_increment_on_high = true;
    latches.tile_padding = [0x56, 0x9a];
    latches.tile_increment = 32;
    latches.tile_remap = 2;
    latches.tile_read_buffer = 0xbcde;
    latches
}

// Movies cannot inspect authored resident image identities in an immutable media lease.
#[cfg(sm_verify_dll)]
fn scene_resident_image(
    media: &sc::SceneMedia,
    reference: sc::MediaRef,
) -> Result<&sc::CompiledImage, ()> {
    media
        .compiled_images()
        .find(|image| image.reference == reference)
        .ok_or(())
}

// Movies cannot inspect authored resident palette identities in an immutable media lease.
#[cfg(sm_verify_dll)]
fn scene_resident_palette(
    media: &sc::SceneMedia,
    reference: sc::MediaRef,
) -> Result<&sc::PaletteIncarnation, ()> {
    media
        .palette_incarnations()
        .find(|palette| palette.reference == reference)
        .ok_or(())
}

// Movies cannot supply short resident images or mismatched identities before the native alias callback.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_resident_identity_and_full_extent_reject_before_aliases() {
    for (fault, expected) in [
        (0, ds::DisplayError::Identity),
        (1, ds::DisplayError::Identity),
        (2, ds::DisplayError::Incomplete),
        (3, ds::DisplayError::Identity),
    ] {
        let (_, start, media) = scene_atmospheric_settlement_fixture(1);
        let mut resident = start.resident.unwrap();
        let mut incarnations = [
            sc::MediaRef {
                incarnation: 7,
                ..resident.image
            },
            sc::MediaRef {
                incarnation: 8,
                ..resident.palette
            },
        ];
        if fault == 0 || fault == 3 {
            incarnations[0].asset.id = sc::AssetId(99);
        }
        if fault == 1 {
            incarnations[1].asset.id = sc::AssetId(99);
        }
        if fault >= 2 {
            resident.words = Arc::from(resident.words[..0x801e].to_vec());
        }
        let before = resident.clone();
        let (resident, calls, result) = sm::verify_resident_append_prefix(
            resident,
            media.clone(),
            incarnations,
            [0x1234, 0x5601, 32, 0x9a02, 0xbcde],
        );
        assert!(matches!(result, Err(error) if error == expected));
        assert_eq!(calls, 0);
        assert_eq!(resident.words, before.words);
        assert_eq!(
            (resident.image, resident.palette),
            (before.image, before.palette)
        );
        assert!(scene_resident_image(&media, incarnations[0]).is_err());
    }
}

// Movies cannot inject media rejection after literal native/color aliases or inspect the retained current media.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_resident_media_failure_keeps_alias_prefix_and_prior_ids() {
    let (requirements, start, media) = scene_atmospheric_settlement_fixture(1);
    let before = start.resident.as_ref().unwrap().clone();
    let latches = scene_resident_append_latches(&start);
    let compiled = scene_resident_image(&media, before.image).unwrap();
    let bindings = (compiled.atlases.clone(), compiled.maps.clone());
    let failed_image = sc::MediaRef {
        incarnation: 7,
        ..before.image
    };
    let old_words = compiled.words.clone();
    let display = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media.clone(), &requirements).unwrap(),
    )
    .unwrap()
    .display;
    let (display, resident, result, adopted) = sm::verify_owned_resident_append(
        display,
        [failed_image, before.palette],
        [0x1234; 256],
        latches,
        bindings.clone(),
    );
    assert!(matches!(result, Err(ds::DisplayError::Scene)));
    assert!(adopted.is_none());
    assert_eq!(
        (resident.image, resident.palette),
        (before.image, before.palette)
    );
    let mut expected = before.words.to_vec();
    expected[0x8000..0x8005].copy_from_slice(&[0x1234, 0x5601, 32, 0x9a02, 0xbcde]);
    expected[0x8005..0x801f].fill(0x1234);
    assert_eq!(resident.words.as_ref(), expected.as_slice());
    assert_eq!(resident.colors, [0x1234; 256]);
    assert_eq!(
        scene_resident_image(&media, before.image).unwrap().words,
        old_words
    );
    assert!(Arc::ptr_eq(display.committed_scene().media(), &media));
    let recovered = [
        sc::MediaRef {
            incarnation: 9,
            ..before.image
        },
        sc::MediaRef {
            incarnation: 10,
            ..before.palette
        },
    ];
    let (_, resident, result, adopted) =
        sm::verify_owned_resident_append(display, recovered, [0x1234; 256], latches, bindings);
    let current = result.unwrap();
    assert!(Arc::ptr_eq(&current, &adopted.unwrap()));
    assert!(scene_resident_image(&current, failed_image).is_err());
    assert_eq!(
        current.compiled_images().count(),
        media.compiled_images().count() + 1
    );
    assert_eq!(
        current.palette_incarnations().count(),
        media.palette_incarnations().count() + 1
    );
    assert_eq!(
        (resident.image, resident.palette),
        (recovered[0], recovered[1])
    );
}

// Movies cannot supply independent mutable resident values and new immutable IDs to prove actual lease adoption.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn scene_resident_append_adopts_supplied_ids_and_keeps_old_lease() {
    let (requirements, start, media) = scene_atmospheric_settlement_fixture(1);
    let before = start.resident.as_ref().unwrap().clone();
    let latches = scene_resident_append_latches(&start);
    let compiled = scene_resident_image(&media, before.image).unwrap();
    let bindings = (compiled.atlases.clone(), compiled.maps.clone());
    let old_words = compiled.words.clone();
    let old_colors = scene_resident_palette(&media, before.palette)
        .unwrap()
        .colors;
    let incarnations = [
        sc::MediaRef {
            incarnation: 7,
            ..before.image
        },
        sc::MediaRef {
            incarnation: 8,
            ..before.palette
        },
    ];
    let display = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media.clone(), &requirements).unwrap(),
    )
    .unwrap()
    .display;
    let lease = display.committed_scene().clone();
    let (display, resident, result, adopted) =
        sm::verify_owned_resident_append(display, incarnations, [0x1234; 256], latches, bindings);
    let current = result.unwrap();
    assert!(Arc::ptr_eq(&current, &adopted.unwrap()));
    assert!(!Arc::ptr_eq(&current, &media));
    assert_eq!(
        (resident.image, resident.palette),
        (incarnations[0], incarnations[1])
    );
    let mut expected = before.words.to_vec();
    expected[0x8000..0x8005].copy_from_slice(&[0x1234, 0x5601, 32, 0x9a02, 0xbcde]);
    expected[0x8005..0x801f].fill(0x1234);
    assert_eq!(resident.words.as_ref(), expected.as_slice());
    assert_eq!(
        scene_resident_image(&current, incarnations[0])
            .unwrap()
            .words
            .as_ref(),
        expected.as_slice()
    );
    assert_eq!(
        scene_resident_image(&current, incarnations[0])
            .unwrap()
            .palette,
        incarnations[1]
    );
    assert_eq!(
        scene_resident_palette(&current, incarnations[1])
            .unwrap()
            .colors,
        [0x1234; 256]
    );
    assert!(Arc::ptr_eq(lease.media(), &media));
    assert!(Arc::ptr_eq(display.committed_scene().media(), &media));
    assert_eq!(
        scene_resident_image(&media, before.image).unwrap().words,
        old_words
    );
    assert_eq!(
        scene_resident_palette(&media, before.palette)
            .unwrap()
            .colors,
        old_colors
    );
}
