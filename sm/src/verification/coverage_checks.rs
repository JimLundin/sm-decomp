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
    let tests: [(&str, fn()); 121] = cases![
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
        scene_start_rejects_missing_samus_content_after_existing_prefixes,
        scene_start_rejects_invalid_samus_catalog_before_reset,
        scene_start_rejects_changed_samus_catalog_and_requirements,
        scene_component_transfers_supplied_samus_metadata_on_reset_and_reopen,
        samus_content_rejects_allocation_and_role_changes,
        samus_content_rejects_authored_request_changes,
        samus_content_rejects_invalid_payload_and_transfer_extents,
        samus_content_prepared_lease_retains_sealed_allocation,
        samus_game_rejected_expectation_preserves_binding,
        samus_game_native_capture_retains_allocation,
        samus_game_prepared_replacement_is_rejected,
        samus_game_failed_install_preserves_binding,
        samus_game_reference_import_keeps_revoked_scope,
        samus_game_panicking_callback_keeps_prefix,
        samus_game_adoption_uses_captured_target,
        persistent_session_native_rebinding_preserves_ready_owner,
        samus_game_body_projection_roundtrip,
        samus_game_body_projection_crossing_writes,
        samus_game_body_foreign_adoption_and_prefix,
        samus_driver_body_projection_preserves_odd_lanes,
        samus_driver_cannon_failure_preserves_prefix,
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
        dsp_voice_seven_registers_keep_native_signed_pitch_envelope_rules,
        dsp_global_registers_keep_native_flags_fir_and_shadow_rules,
        dsp_key_registers_apply_immediately_with_wrapped_apu_directory,
        dsp_invalid_register_addresses_reject_before_state_or_memory_effects,
        dsp_reset_clears_every_dirty_voice_field_and_preserves_each_padding_byte,
        dsp_reset_restores_native_globals_without_changing_apu_memory_pointer_or_padding,
        dsp_cycle_brr_gaussian_and_envelope_have_independent_literal_samples,
        dsp_cycle_noise_signed_mixing_and_full_buffer_keep_ordered_progress,
        dsp_cycle_echo_fir_signed_clip_and_wrapped_writeback_are_independent,
        dsp_cycle_dirty_adsr_and_fir_failures_preserve_exact_completed_prefixes,
        apu_timers_preserve_restored_rollover_read_clear_and_enable_phase,
        apu_bus_preserves_boot_ports_ram_shadow_and_masked_dsp_history,
        apu_callback_failures_keep_history_cycle_and_reset_prefixes,
        spc_native_cold_matrix_00_through_3f,
        spc_native_cold_matrix_40_through_7f,
        spc_native_cold_matrix_80_through_bf,
        spc_native_cold_matrix_c0_through_ff,
        spc_reset_stop_and_fetch_failure_keep_native_prefixes,
        spc_alu_literals_cover_signed_carry_overflow_and_memory_order,
        spc_shift_literals_preserve_memory_then_flags_and_accumulator_results,
        spc_word_literals_cover_native_half_carry_and_page_wrapped_words,
        spc_division_bcd_and_multiply_have_literal_native_results,
        spc_branch_and_direct_bits_keep_native_predicates_cycles_and_read_order,
        spc_absolute_bits_have_literal_carry_and_write_results,
        spc_calls_and_stack_keep_literal_vector_order_and_wrap,
        spc_indirect_moves_and_indexed_jumps_keep_literal_bus_addresses,
        spc_bus_failures_preserve_exact_native_register_and_memory_prefixes,
        spc_brk_and_reti_keep_literal_flags_and_pull_order,
        spc_indexed_pointer_and_jump_reads_wrap_before_bus_access,
        spc_stack_index_and_dbnz_bus_failures_preserve_native_prefixes,
        sample_delivery_lifecycle_preserves_missing_pcm_zero_length_and_prepare_failures,
        sample_delivery_byte_cursor_and_skipped_refills_keep_literal_pcm_and_cadence,
        sample_delivery_s16lsb_volumes_and_odd_cursor_have_literal_ordered_bytes,
        sample_delivery_writer_and_refill_failures_keep_completed_chunk_prefixes,
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
            parts: Arc::from([
                sc::VisualPart {
                    atlas: reference(15),
                    material: reference(6),
                    tile: 0,
                    offset: sc::ScreenPosition { x: -3, y: -8 },
                    size: sc::ObjectSize::Large,
                },
                sc::VisualPart {
                    atlas: reference(15),
                    material: reference(6),
                    tile: 0,
                    offset: sc::ScreenPosition { x: -2, y: -7 },
                    size: sc::ObjectSize::Large,
                },
            ]),
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
            // Supplied fixture selector; this setup does not bind a native request.
            pointer: 0x8101,
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
        // This original supplied start had no completed undrained readbacks.
        read_results: vec![],
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

// Movies cannot omit a sealed Samus catalog behind earlier authored admission failures.
#[cfg_attr(test, test)]
fn scene_start_rejects_missing_samus_content_after_existing_prefixes() {
    for case in 0..7 {
        let (mut requirements, mut start, media) = scene_component_fixture();
        start.samus_content = None;
        match case {
            0 => start.display = None,
            1 => start.resident = None,
            2 => start.committed_scene = None,
            3 => start.identity.as_mut().unwrap().schema = 4,
            4 => {
                start.identity.as_mut().unwrap().committed_tick = 1;
                start.continuation.as_mut().unwrap().phase = ss::CommittedPhase::SettledInterval;
            }
            5 => {
                requirements.reference.image.width = 255;
                start.reference.as_mut().unwrap().image.width = 255;
            }
            _ => {}
        }
        let error = ss::ValidatedDisplayStart::construct(start, media, &requirements)
            .err()
            .unwrap();
        assert!(matches!(
            (case, error),
            (0, ss::StartError::MissingDisplay)
                | (1, ss::StartError::MissingResidentDisplay)
                | (2, ss::StartError::MissingCommittedHistory)
                | (3, ss::StartError::Schema)
                | (4, ss::StartError::CommitIdentity)
                | (5, ss::StartError::ReferenceImage)
                | (6, ss::StartError::MissingSamusContent)
        ));
    }
}

// Movies cannot author malformed portable Samus catalogs or observe rejected reset ownership.
#[cfg_attr(test, test)]
fn scene_start_rejects_invalid_samus_catalog_before_reset() {
    let (requirements, start, media) = scene_component_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    for case in 0..9 {
        let (mut start, media) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        let catalog = Arc::make_mut(start.samus_content.as_mut().unwrap());
        let map = catalog.maps.get_mut(&0x8101).unwrap();
        let part = &mut Arc::make_mut(&mut map.parts)[0];
        match case {
            0 => catalog.revision = sc::Revision([99; 32]),
            1 => catalog.pointers = Arc::from([0x8103]),
            2 => map.count = 2,
            3 => Arc::make_mut(&mut part.variants).size_sets.clear(),
            4 => {
                Arc::make_mut(&mut part.variants)
                    .size_sets
                    .get_mut(&0)
                    .unwrap()
                    .large
                    .visual
                    .id = sc::AssetId(8)
            }
            5 => {
                Arc::make_mut(&mut part.variants)
                    .size_sets
                    .get_mut(&0)
                    .unwrap()
                    .large
                    .frame = 1
            }
            6 => {
                Arc::make_mut(&mut part.variants)
                    .size_sets
                    .get_mut(&0)
                    .unwrap()
                    .large
                    .part = 2
            }
            7 => part.native_word = 0x3401,
            _ => {
                let variants = Arc::make_mut(&mut part.variants);
                let binding = variants.size_sets.remove(&0).unwrap();
                variants.size_sets.insert(8, binding);
            }
        }
        let error = ss::ValidatedDisplayStart::construct(start, media, &requirements)
            .and_then(|start| display.reset(start))
            .err()
            .unwrap();
        assert!(matches!(
            (case, error),
            (0, ss::StartError::Display(ds::DisplayError::Revision))
                | (1..=2, ss::StartError::Display(ds::DisplayError::Incomplete))
                | (3..=8, ss::StartError::Display(ds::DisplayError::Scene))
        ));
        scene_owner_unchanged(&display, &before, &opened.metadata);
        assert_eq!(
            opened.metadata.samus_content,
            scene_expected_start_samus_catalog()
        );
    }
}

// Movies cannot supply different media-valid Samus catalogs under unchanged start identities.
#[cfg_attr(test, test)]
fn scene_start_rejects_changed_samus_catalog_and_requirements() {
    let (requirements, start, media) = scene_component_fixture();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    for case in 0..8 {
        let mut required = requirements.clone();
        let (mut start, media) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        let catalog = Arc::make_mut(start.samus_content.as_mut().unwrap());
        let map = catalog.maps.get_mut(&0x8101).unwrap();
        match case {
            0 => catalog.pointers = Arc::from([0x8101, 0x8101]),
            1 => map.count = 0,
            2 => Arc::make_mut(&mut map.parts)[0].x = 0xfffc,
            3 => Arc::make_mut(&mut map.parts)[0].y = 0xf7,
            4 => Arc::make_mut(&mut map.parts)[0].native_word = 0x3600,
            5 => {
                let part = map.parts[0].clone();
                map.parts = Arc::from([part.clone(), part]);
            }
            6 => {
                Arc::make_mut(&mut Arc::make_mut(&mut map.parts)[0].variants)
                    .size_sets
                    .get_mut(&0)
                    .unwrap()
                    .large
                    .part = 1
            }
            _ => {
                let map = Arc::make_mut(&mut required.samus_content)
                    .maps
                    .get_mut(&0x8101)
                    .unwrap();
                Arc::make_mut(&mut map.parts)[0].x = 0xfffc;
            }
        }
        assert_eq!(
            start.samus_content.as_ref().unwrap().validate(&media),
            Ok(())
        );
        assert_eq!(required.samus_content.validate(&media), Ok(()));
        let error = ss::ValidatedDisplayStart::construct(start, media, &required)
            .and_then(|start| display.reset(start))
            .err()
            .unwrap();
        assert!(matches!(
            error,
            ss::StartError::Display(ds::DisplayError::Incomplete)
        ));
        scene_owner_unchanged(&display, &before, &opened.metadata);
        assert_eq!(
            opened.metadata.samus_content,
            scene_expected_start_samus_catalog()
        );
    }
}

// Movies cannot replace G-supplied Samus metadata while reopening an older display checkpoint.
#[cfg_attr(test, test)]
fn scene_component_transfers_supplied_samus_metadata_on_reset_and_reopen() {
    let (mut requirements, start, media) = scene_component_fixture();
    let old_requirements = requirements.clone();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media.clone(), &requirements).unwrap(),
    )
    .unwrap();
    assert_eq!(
        opened.metadata.samus_content,
        scene_expected_start_samus_catalog()
    );
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    let expected = Arc::make_mut(&mut requirements.samus_content);
    expected.pointers = Arc::from([0x8235]);
    let mut map = expected.maps.remove(&0x8101).unwrap();
    let part = &mut Arc::make_mut(&mut map.parts)[0];
    (part.x, part.y, part.native_word) = (0x1234, 9, 0x7600);
    Arc::make_mut(&mut part.variants)
        .size_sets
        .get_mut(&0)
        .unwrap()
        .large
        .part = 1;
    expected.maps.insert(0x8235, map);
    let mut metadata = scene_copy_metadata(&opened.metadata);
    metadata.samus_content = scene_start_samus_catalog(media.revision());
    let supplied = Arc::make_mut(&mut metadata.samus_content);
    supplied.pointers = Arc::from([0x8235]);
    let mut map = supplied.maps.remove(&0x8101).unwrap();
    let part = &mut Arc::make_mut(&mut map.parts)[0];
    (part.x, part.y, part.native_word) = (0x1234, 9, 0x7600);
    Arc::make_mut(&mut part.variants)
        .size_sets
        .get_mut(&0)
        .unwrap()
        .large
        .part = 1;
    supplied.maps.insert(0x8235, map);
    let expected_metadata = sd::DisplayStartMetadata {
        samus_content: requirements.samus_content.clone(),
        ..scene_copy_metadata(&opened.metadata)
    };
    let supplied = metadata.samus_content.clone();
    assert!(!Arc::ptr_eq(&supplied, &requirements.samus_content));
    let (wrong, media) = before
        .clone()
        .into_start_description(scene_copy_metadata(&metadata));
    assert!(matches!(
        ss::ValidatedDisplayStart::construct(wrong, media, &old_requirements),
        Err(ss::StartError::Display(ds::DisplayError::Incomplete))
    ));
    scene_owner_unchanged(&display, &before, &opened.metadata);
    let (start, media) = before.clone().into_start_description(metadata);
    assert_eq!(
        start.samus_content.as_deref(),
        Some(requirements.samus_content.as_ref())
    );
    assert!(Arc::ptr_eq(
        start.samus_content.as_ref().unwrap(),
        &supplied
    ));
    let metadata = display
        .reset(ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap())
        .unwrap();
    scene_same_metadata(&metadata, &expected_metadata);
    assert!(Arc::ptr_eq(&metadata.samus_content, &supplied));
    scene_owner_unchanged(&display, &before, &metadata);
    let (start, media) = display
        .checkpoint()
        .unwrap()
        .into_start_description(metadata);
    assert!(Arc::ptr_eq(
        start.samus_content.as_ref().unwrap(),
        &supplied
    ));
    let reopened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
    )
    .unwrap();
    scene_same_metadata(&reopened.metadata, &expected_metadata);
    assert!(Arc::ptr_eq(&reopened.metadata.samus_content, &supplied));
    scene_owner_unchanged(&reopened.display, &before, &reopened.metadata);
    assert_eq!(
        opened.metadata.samus_content,
        scene_expected_start_samus_catalog()
    );
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

// Movies cannot author the independent role-content admission and backing cases.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_content_rejects_allocation_and_role_changes() {
    sm::samus_content_rejects_allocation_and_role_changes();
}

// Movies cannot author the independent role-content admission and backing cases.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_content_rejects_authored_request_changes() {
    sm::samus_content_rejects_authored_request_changes();
}

// Movies cannot author the independent role-content admission and backing cases.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_content_rejects_invalid_payload_and_transfer_extents() {
    sm::samus_content_rejects_invalid_payload_and_transfer_extents();
}

// Movies cannot author the independent role-content admission and backing cases.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_content_prepared_lease_retains_sealed_allocation() {
    sm::samus_content_prepared_lease_retains_sealed_allocation();
}

// Movies cannot author independent Game admission and captured-allocation failures.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_game_rejected_expectation_preserves_binding() {
    sm::samus_game_rejected_expectation_preserves_binding();
}

// Movies cannot author independent Game admission and captured-allocation failures.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_game_native_capture_retains_allocation() {
    sm::samus_game_native_capture_retains_allocation();
}

// Movies cannot author independent Game admission and captured-allocation failures.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_game_prepared_replacement_is_rejected() {
    sm::samus_game_prepared_replacement_is_rejected();
}

// Movies cannot author independent Game admission and captured-allocation failures.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_game_failed_install_preserves_binding() {
    sm::samus_game_failed_install_preserves_binding();
}

// Movies cannot author independent owner mutation, unwind and selected-adoption cases.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_game_reference_import_keeps_revoked_scope() {
    sm::samus_game_reference_import_keeps_revoked_scope();
}

// Movies cannot author independent owner mutation, unwind and selected-adoption cases.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_game_panicking_callback_keeps_prefix() {
    sm::samus_game_panicking_callback_keeps_prefix();
}

// Movies cannot author independent owner mutation, unwind and selected-adoption cases.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_game_adoption_uses_captured_target() {
    sm::samus_game_adoption_uses_captured_target();
}

// Movies cannot construct independent body aliases or short-ROM driver failures.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_driver_body_projection_preserves_odd_lanes() {
    sm::samus_driver_body_projection_preserves_odd_lanes();
}

// Movies cannot construct independent body aliases or short-ROM driver failures.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_driver_cannon_failure_preserves_prefix() {
    sm::samus_driver_cannon_failure_preserves_prefix();
}

// Movies cannot construct independently supplied crossing body writes and foreign prefixes.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_game_body_projection_roundtrip() {
    sm::samus_game_body_projection_roundtrip();
}

// Movies cannot construct independently supplied crossing body writes and foreign prefixes.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_game_body_projection_crossing_writes() {
    sm::samus_game_body_projection_crossing_writes();
}

// Movies cannot construct independently supplied crossing body writes and foreign prefixes.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn samus_game_body_foreign_adoption_and_prefix() {
    sm::samus_game_body_foreign_adoption_and_prefix();
}

// Movies cannot supply undrained readback payloads or malformed detached Display checkpoints.
#[cfg(sm_verify_dll)]
fn scene_read_results_codec_and_reopen() {
    let (requirements, mut start, media, _) = scene_namespace_marker_fixture();
    assert!(start.display.as_ref().unwrap().read_results.is_empty());
    let expected = vec![
        (
            sc::AssetRef {
                id: sc::AssetId(2),
                revision: media.revision(),
            },
            vec![0x13, 0xa7, 0x00, 0xff],
        ),
        (
            sc::AssetRef {
                id: sc::AssetId(12),
                revision: media.revision(),
            },
            vec![0x29, 0x80, 0x41],
        ),
    ];
    start.display.as_mut().unwrap().read_results = expected.clone();
    let opened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(start, media.clone(), &requirements).unwrap(),
    )
    .unwrap();
    let metadata = opened.metadata;
    let mut display = opened.display;
    let before = display.checkpoint().unwrap();
    let (captured, _) = before
        .clone()
        .into_start_description(scene_copy_metadata(&metadata));
    assert_eq!(captured.display.as_ref().unwrap().read_results, expected);
    scene_owner_unchanged(&display, &before, &metadata);
    let bytes = before.encode_checkpoint(&metadata, &requirements).unwrap();
    assert_eq!(&bytes[..8], b"SMDISPL\x02");
    let decoded =
        sd::DisplayCheckpoint::decode_checkpoint(&bytes, media.clone(), &metadata, &requirements)
            .unwrap();
    let (reopen, reopen_media) = decoded
        .clone()
        .into_start_description(scene_copy_metadata(&metadata));
    assert_eq!(reopen.display.as_ref().unwrap().read_results, expected);
    let reopened = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(reopen, reopen_media, &requirements).unwrap(),
    )
    .unwrap();
    scene_owner_unchanged(&reopened.display, &before, &metadata);
    let (replacement, replacement_media) = decoded
        .clone()
        .into_start_description(scene_copy_metadata(&metadata));
    let returned = display
        .reset(
            ss::ValidatedDisplayStart::construct(replacement, replacement_media, &requirements)
                .unwrap(),
        )
        .unwrap();
    scene_same_metadata(&returned, &metadata);
    scene_owner_unchanged(&display, &before, &metadata);
    let (mut bad, bad_media) = decoded
        .clone()
        .into_start_description(scene_copy_metadata(&metadata));
    bad.display.as_mut().unwrap().read_results[0].1 = vec![0x88, 0x99];
    bad.display.as_mut().unwrap().read_results[1].0.id = sc::AssetId(0);
    let rejected = ss::ValidatedDisplayStart::construct(bad, bad_media, &requirements)
        .and_then(|start| display.reset(start));
    assert!(matches!(
        rejected,
        Err(ss::StartError::Display(ds::DisplayError::Identity))
    ));
    scene_owner_unchanged(&display, &before, &metadata);
    let (retained, _) = display
        .checkpoint()
        .unwrap()
        .into_start_description(scene_copy_metadata(&metadata));
    assert_eq!(retained.display.as_ref().unwrap().read_results, expected);
    assert_eq!(
        sd::DisplayCheckpoint::decode_checkpoint(
            &bytes[..bytes.len() - 1],
            media.clone(),
            &metadata,
            &requirements
        )
        .err()
        .as_deref(),
        Some("truncated actor checkpoint")
    );
    let mut extra = bytes.clone();
    extra.push(0xe8);
    assert_eq!(
        sd::DisplayCheckpoint::decode_checkpoint(&extra, media.clone(), &metadata, &requirements)
            .err()
            .as_deref(),
        Some("trailing display checkpoint fields")
    );
    let mut old_schema = bytes.clone();
    old_schema[7] = 1;
    assert_eq!(
        sd::DisplayCheckpoint::decode_checkpoint(
            &old_schema,
            media.clone(),
            &metadata,
            &requirements
        )
        .err()
        .as_deref(),
        Some("unsupported display checkpoint schema")
    );
    scene_read_results_changed_bindings(&before, &bytes, &metadata, &requirements);
    sd::DisplayCheckpoint::assert_decoder_rejections(
        &bytes,
        media.clone(),
        &metadata,
        &requirements,
    );
    scene_owner_unchanged(&display, &before, &metadata);
}

// Movies cannot independently validate a changed Reference or catalog before payload rejection.
#[cfg(sm_verify_dll)]
fn scene_read_results_changed_bindings(
    before: &sd::DisplayCheckpoint,
    bytes: &[u8],
    metadata: &sd::DisplayStartMetadata,
    requirements: &ss::DisplayStartRequirements,
) {
    let (mut reference_start, media) = before
        .clone()
        .into_start_description(scene_copy_metadata(metadata));
    let mut required_b = requirements.clone();
    required_b.reference.fingerprint = [0xb8; 32];
    reference_start.reference = Some(required_b.reference);
    reference_start.identity.as_mut().unwrap().reference = [0xb8; 32];
    let old = reference_start.committed_scene.take().unwrap();
    let mut description = old.description().clone();
    description.reference = [0xb8; 32];
    reference_start.committed_scene = Some(
        sc::Scene::new(description, media.clone())
            .unwrap()
            .with_producer_history(Arc::from(old.producer_history().to_vec()))
            .unwrap(),
    );
    let valid_b = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(reference_start, media.clone(), &required_b).unwrap(),
    )
    .unwrap();
    assert_ne!(valid_b.metadata.reference, metadata.reference);
    assert_eq!(
        sd::DisplayCheckpoint::decode_checkpoint(
            bytes,
            media.clone(),
            &valid_b.metadata,
            &required_b
        )
        .err()
        .as_deref(),
        Some("display checkpoint metadata or immutable content changed")
    );
    let mut assets = media.assets().cloned().collect::<Vec<_>>();
    assets.push(sc::SceneAsset::Room(sc::AssetRef {
        id: sc::AssetId(0x4208),
        revision: media.revision(),
    }));
    let changed_media = Arc::new(
        sc::SceneMedia::new(
            media.revision(),
            assets,
            media.palette_incarnations().cloned().collect(),
            media.compiled_images().cloned().collect(),
        )
        .unwrap(),
    );
    let (mut content_start, _) = before
        .clone()
        .into_start_description(scene_copy_metadata(metadata));
    let old = content_start.committed_scene.take().unwrap();
    content_start.committed_scene = Some(
        sc::Scene::new(old.description().clone(), changed_media.clone())
            .unwrap()
            .with_producer_history(Arc::from(old.producer_history().to_vec()))
            .unwrap(),
    );
    let valid_c = sd::DisplayComponent::open(
        ss::ValidatedDisplayStart::construct(content_start, changed_media.clone(), requirements)
            .unwrap(),
    )
    .unwrap();
    scene_same_metadata(&valid_c.metadata, metadata);
    assert_ne!(changed_media.as_ref(), media.as_ref());
    assert_eq!(
        sd::DisplayCheckpoint::decode_checkpoint(
            bytes,
            changed_media,
            &valid_c.metadata,
            requirements
        )
        .err()
        .as_deref(),
        Some("display checkpoint metadata or immutable content changed")
    );
}

// Movies cannot replace a prepared Session with independently supplied Native content.
#[cfg(sm_verify_dll)]
#[cfg_attr(test, test)]
fn persistent_session_native_rebinding_preserves_ready_owner() {
    scene_read_results_codec_and_reopen();
    let (requirements, start, media, variants) = scene_namespace_marker_fixture();
    sm::persistent_session_native_rebinding_preserves_ready_owner(
        requirements,
        start,
        media,
        variants,
    );
}

// Movies cannot independently inspect native voice fields after authored register writes.
#[cfg_attr(test, test)]
fn dsp_voice_seven_registers_keep_native_signed_pitch_envelope_rules() {
    use sm::snes::dsp::{DspState, dsp_read, write_register};
    let ram = sm::apu_ram::ApuRam::new();
    let mut state = DspState::new();
    state.channel[7].pitch = 0xc155;
    state.channel[7].gainValue = 99;
    write_register(&mut state, &ram, 0x70, 0x80);
    write_register(&mut state, &ram, 0x71, 0xff);
    assert_eq!(
        (state.channel[7].volumeL, state.channel[7].volumeR),
        (-128, -1)
    );
    assert_eq!(
        (dsp_read(&state, 0x70), dsp_read(&state, 0x71)),
        (0x80, 0xff)
    );
    write_register(&mut state, &ram, 0x72, 0xab);
    assert_eq!(state.channel[7].pitch, 0x01ab);
    write_register(&mut state, &ram, 0x73, 0xff);
    assert_eq!(state.channel[7].pitch, 0x3fab);
    write_register(&mut state, &ram, 0x72, 0x19);
    assert_eq!(state.channel[7].pitch, 0x3f19);
    write_register(&mut state, &ram, 0x73, 0x80);
    assert_eq!(state.channel[7].pitch, 0x0019);
    assert_eq!(dsp_read(&state, 0x73), 0x80);
    write_register(&mut state, &ram, 0x74, 0xff);
    assert_eq!(state.channel[7].srcn, 0xff);
    write_register(&mut state, &ram, 0x75, 0xf5);
    assert_eq!(state.channel[7].adsrRates, [192, 2, 0, 0]);
    assert!(!state.channel[7].useGain);
    write_register(&mut state, &ram, 0x75, 0xff);
    assert_eq!(state.channel[7].adsrRates, [1, 2, 0, 0]);
    write_register(&mut state, &ram, 0x75, 0x05);
    assert_eq!(state.channel[7].adsrRates, [192, 64, 0, 0]);
    assert!(state.channel[7].useGain);
    write_register(&mut state, &ram, 0x76, 0xff);
    assert_eq!(
        (state.channel[7].adsrRates[2], state.channel[7].sustainLevel),
        (1, 2048)
    );
    write_register(&mut state, &ram, 0x76, 0x00);
    assert_eq!(
        (state.channel[7].adsrRates[2], state.channel[7].sustainLevel),
        (0, 256)
    );
    write_register(&mut state, &ram, 0x77, 0xe3);
    assert_eq!(
        (state.channel[7].adsrRates[3], state.channel[7].gainMode),
        (1280, 3)
    );
    assert!(!state.channel[7].directGain);
    assert_eq!(state.channel[7].gainValue, 99);
    write_register(&mut state, &ram, 0x77, 0xed);
    assert_eq!(
        (state.channel[7].adsrRates[3], state.channel[7].gainMode),
        (128, 3)
    );
    write_register(&mut state, &ram, 0x77, 0x7f);
    assert!(state.channel[7].directGain);
    assert_eq!(state.channel[7].gainValue, 2032);
    assert_eq!(
        (state.channel[7].adsrRates[3], state.channel[7].gainMode),
        (128, 3)
    );
    for (value, rate, mode) in [(0x80, 0, 0), (0xa1, 2048, 1), (0xc2, 1536, 2)] {
        write_register(&mut state, &ram, 0x77, value);
        assert_eq!(
            (state.channel[7].adsrRates[3], state.channel[7].gainMode),
            (rate, mode)
        );
        assert!(!state.channel[7].directGain);
        assert_eq!(state.channel[7].gainValue, 2032);
    }
    assert_eq!((state.channel[0].pitch, state.channel[0].volumeL), (0, 0));
}

// Movies cannot independently inspect native global/FIR fields or author ignored shadows.
#[cfg_attr(test, test)]
fn dsp_global_registers_keep_native_flags_fir_and_shadow_rules() {
    use sm::snes::dsp::{DspState, dsp_read, write_register};
    let ram = sm::apu_ram::ApuRam::new();
    let mut state = DspState::new();
    for (address, value) in [
        (0x0c, 0x80),
        (0x1c, 0x7f),
        (0x2c, 0xff),
        (0x3c, 0x81),
        (0x0d, 0xf0),
    ] {
        write_register(&mut state, &ram, address, value);
        assert_eq!(dsp_read(&state, address), value);
    }
    assert_eq!((state.masterVolumeL, state.masterVolumeR), (-128, 127));
    assert_eq!(
        (state.echoVolumeL, state.echoVolumeR, state.feedbackVolume),
        (-1, -127, -16)
    );
    write_register(&mut state, &ram, 0x6c, 0xff);
    assert_eq!(
        (state.reset, state.mute, state.echoWrites, state.noiseRate),
        (true, true, false, 1)
    );
    write_register(&mut state, &ram, 0x6c, 0x00);
    assert_eq!(
        (state.reset, state.mute, state.echoWrites, state.noiseRate),
        (false, false, true, 0)
    );
    write_register(&mut state, &ram, 0x6c, 0x2d);
    assert_eq!(
        (state.reset, state.mute, state.echoWrites, state.noiseRate),
        (false, false, false, 128)
    );
    write_register(&mut state, &ram, 0x2d, 0x81);
    assert_eq!(
        state.channel.map(|c| c.pitchModulation),
        [true, false, false, false, false, false, false, true]
    );
    write_register(&mut state, &ram, 0x3d, 0x42);
    assert_eq!(
        state.channel.map(|c| c.useNoise),
        [false, true, false, false, false, false, true, false]
    );
    write_register(&mut state, &ram, 0x4d, 0xa5);
    assert_eq!(
        state.channel.map(|c| c.echoEnable),
        [true, false, true, false, false, true, false, true]
    );
    write_register(&mut state, &ram, 0x5d, 0xff);
    write_register(&mut state, &ram, 0x6d, 0xaa);
    assert_eq!((state.dirPage, state.echoBufferAdr), (0xff00, 0xaa00));
    state.echoRemain = 19;
    state.echoBufferIndex = 23;
    write_register(&mut state, &ram, 0x7d, 0xf0);
    assert_eq!(state.echoDelay, 1);
    write_register(&mut state, &ram, 0x7d, 0xfb);
    assert_eq!(state.echoDelay, 5632);
    assert_eq!((state.echoRemain, state.echoBufferIndex), (19, 23));
    for (address, value) in [
        (0x0f, 0x80),
        (0x1f, 0xff),
        (0x2f, 0),
        (0x3f, 0x7f),
        (0x4f, 0xfe),
        (0x5f, 0x81),
        (0x6f, 1),
        (0x7f, 0x55),
    ] {
        write_register(&mut state, &ram, address, value);
        assert_eq!(dsp_read(&state, address), value);
    }
    assert_eq!(state.firValues, [-128, -1, 0, 127, -2, -127, 1, 85]);
    state.ram[0x7c] = 0xff;
    write_register(&mut state, &ram, 0x7c, 0x57);
    assert_eq!(dsp_read(&state, 0x7c), 0);
    state.channel[7].gain = 37;
    state.channel[7].sampleOut = -91;
    for address in [
        0x08, 0x09, 0x0a, 0x0b, 0x0e, 0x1d, 0x78, 0x79, 0x7a, 0x7b, 0x7e,
    ] {
        write_register(&mut state, &ram, address, 0x39);
        assert_eq!(dsp_read(&state, address), 0x39);
    }
    assert_eq!(
        (state.channel[7].gain, state.channel[7].sampleOut),
        (37, -91)
    );
    assert_eq!(
        (state.dirPage, state.echoBufferAdr, state.echoDelay),
        (0xff00, 0xaa00, 5632)
    );
    assert_eq!(state.firValues, [-128, -1, 0, 127, -2, -127, 1, 85]);
}

// Movies cannot author wrapped APU directories and independently inspect immediate key effects.
#[cfg_attr(test, test)]
fn dsp_key_registers_apply_immediately_with_wrapped_apu_directory() {
    use sm::snes::dsp::{DspState, dsp_read, write_register};
    let ram = sm::apu_ram::ApuRam::new();
    let mut state = DspState::new();
    for (address, value) in [(0x02fc, 0x34), (0x02fd, 0x12), (0, 0xcd), (1, 0xab)] {
        ram.write(address, value);
    }
    let before_ram: [u8; 65536] = core::array::from_fn(|i| ram.read_at(i));
    write_register(&mut state, &ram, 0x5d, 0xff);
    write_register(&mut state, &ram, 0x04, 0x40);
    write_register(&mut state, &ram, 0x74, 0xff);
    write_register(&mut state, &ram, 0x75, 0x05);
    for ch in &mut state.channel {
        ch.keyOn = true;
        ch.keyOff = true;
        ch.previousFlags = 7;
        ch.decodeOffset = 0x3456;
        ch.decodeBuffer = [11; 19];
        ch.gain = 513;
        ch.adsrState = 2;
        ch.rateCounter = 99;
        ch.pitchCounter = 77;
        ch.old = -9;
        ch.older = 8;
    }
    write_register(&mut state, &ram, 0x4c, 0x81);
    assert_eq!(dsp_read(&state, 0x4c), 0x81);
    assert_eq!(state.channel.map(|c| c.keyOn), [false; 8]);
    assert_eq!(
        (state.channel[0].decodeOffset, state.channel[7].decodeOffset),
        (0xabcd, 0x1234)
    );
    assert_eq!(
        (state.channel[0].adsrState, state.channel[7].adsrState),
        (0, 3)
    );
    for index in [0, 7] {
        let ch = &state.channel[index];
        assert_eq!((ch.previousFlags, ch.gain), (0, 0));
        assert_eq!(ch.decodeBuffer, [0; 19]);
        assert_eq!(
            (ch.rateCounter, ch.pitchCounter, ch.old, ch.older),
            (99, 77, -9, 8)
        );
        assert!(ch.keyOff);
    }
    assert_eq!(
        (
            state.channel[1].previousFlags,
            state.channel[1].gain,
            state.channel[1].adsrState
        ),
        (7, 513, 2)
    );
    assert_eq!(state.channel[1].decodeBuffer, [11; 19]);
    write_register(&mut state, &ram, 0x5c, 0x80);
    assert_eq!(dsp_read(&state, 0x5c), 0x80);
    assert_eq!(
        state.channel.map(|c| c.keyOff),
        [false, false, false, false, false, false, false, true]
    );
    assert_eq!(
        (
            state.channel[0].adsrState,
            state.channel[1].adsrState,
            state.channel[7].adsrState
        ),
        (0, 2, 4)
    );
    write_register(&mut state, &ram, 0x5c, 0);
    assert_eq!(state.channel.map(|c| c.keyOff), [false; 8]);
    assert_eq!(state.channel[7].adsrState, 4);
    write_register(&mut state, &ram, 0x5c, 0x80);
    write_register(&mut state, &ram, 0x4c, 0x80);
    assert_eq!(
        (dsp_read(&state, 0x4c), state.channel[7].adsrState),
        (0x80, 3)
    );
    assert!(state.channel[7].keyOff);
    assert!(!state.channel[7].keyOn);
    state.channel[7].gain = 37;
    state.channel[7].decodeBuffer[0] = 91;
    write_register(&mut state, &ram, 0x4c, 0);
    assert_eq!(
        (
            dsp_read(&state, 0x4c),
            state.channel[7].gain,
            state.channel[7].decodeBuffer[0]
        ),
        (0, 37, 91)
    );
    assert!((0..65536).all(|i| ram.read_at(i) == before_ram[i]));
}

// Movies cannot send invalid full-u8 DSP addresses or inspect rejection before field/memory effects.
#[cfg_attr(test, test)]
fn dsp_invalid_register_addresses_reject_before_state_or_memory_effects() {
    use sm::snes::dsp::{DspState, dsp_read, write_register};
    let ram = sm::apu_ram::ApuRam::new();
    let mut state = DspState::new();
    state.ram = core::array::from_fn(|i| i as u8 ^ 0xa5);
    state.channel[7].decodeBuffer = [13; 19];
    state.channel[7].pitch = 0x3fed;
    state.channel[7].gain = 509;
    state.channel[7].gainMode = 3;
    state.channel[7].keyOn = true;
    state.channel[7].keyOff = true;
    state.channel[7].adsrState = 4;
    state.firValues = [-17; 8];
    state.sampleBuffer = [23; 1068];
    ram.fill(0x67);
    let before = state;
    let before_ram: [u8; 65536] = core::array::from_fn(|i| ram.read_at(i));
    let voice = |c: &sm::types::DspChannel| {
        (
            (c.pitch, c.pitchCounter, c.pitchModulation),
            (c.decodeBuffer, c.srcn, c.decodeOffset),
            (c.previousFlags, c.old, c.older, c.useNoise),
            (c.adsrRates, c.rateCounter, c.adsrState),
            (c.sustainLevel, c.useGain, c.gainMode, c.directGain),
            (c.gainValue, c.gain, c.keyOn, c.keyOff),
            (c.sampleOut, c.volumeL, c.volumeR, c.echoEnable),
        )
    };
    let globals = |s: &DspState| {
        (
            (s.dirPage, s.evenCycle, s.mute, s.reset),
            (s.masterVolumeL, s.masterVolumeR, s.noiseSample),
            (s.noiseRate, s.noiseCounter, s.echoWrites),
            (s.echoVolumeL, s.echoVolumeR, s.feedbackVolume),
            (s.echoBufferAdr, s.echoDelay, s.echoRemain),
            (s.echoBufferIndex, s.firBufferIndex, s.firValues),
            (s.firBufferL, s.firBufferR, s.sampleBuffer, s.sampleOffset),
        )
    };
    for address in [128, 255] {
        assert!(std::panic::catch_unwind(|| dsp_read(&state, address)).is_err());
        assert_eq!(state.ram, before.ram);
        assert_eq!(
            state.channel.map(|c| voice(&c)),
            before.channel.map(|c| voice(&c))
        );
        assert_eq!(globals(&state), globals(&before));
        assert!((0..65536).all(|i| ram.read_at(i) == before_ram[i]));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                write_register(&mut state, &ram, address, 0xff);
            }))
            .is_err()
        );
        assert_eq!(state.ram, before.ram);
        assert_eq!(
            state.channel.map(|c| voice(&c)),
            before.channel.map(|c| voice(&c))
        );
        assert_eq!(globals(&state), globals(&before));
        assert!((0..65536).all(|i| ram.read_at(i) == before_ram[i]));
    }
}

// Movies cannot dirty every native DSP voice field or independently inspect preserved padding.
#[cfg_attr(test, test)]
fn dsp_reset_clears_every_dirty_voice_field_and_preserves_each_padding_byte() {
    use sm::snes::dsp::DspState;
    let mut state = DspState::new();
    for (index, voice) in state.channel.iter_mut().enumerate() {
        voice.pitch = 0xffff;
        voice.pitchCounter = 0xffff;
        voice.pitchModulation = true;
        voice.decodeBuffer = [-0x1234; 19];
        voice.srcn = 0xff;
        voice.decodeOffset = 0xffff;
        voice.previousFlags = 0xe5;
        voice.old = -123;
        voice.older = 123;
        voice.useNoise = true;
        voice.adsrRates = [123; 4];
        voice.rateCounter = 0xffff;
        voice.adsrState = 0xff;
        voice.sustainLevel = 0xffff;
        voice.useGain = true;
        voice.gainMode = 0xff;
        voice.directGain = true;
        voice.gainValue = 0xffff;
        voice.gain = 0xffff;
        voice.keyOn = true;
        voice.keyOff = true;
        voice.sampleOut = -255;
        voice.volumeL = -128;
        voice.volumeR = 127;
        voice.echoEnable = true;
        voice.snapshot_padding0 = [0x10 + index as u8];
        voice.snapshot_padding1 = [0x20 + index as u8];
        voice.snapshot_padding2 = [0x30 + index as u8];
        voice.snapshot_padding3 = [0x40 + index as u8];
        voice.snapshot_padding4 = [0x50 + index as u8];
        voice.snapshot_padding5 = [0x60 + index as u8];
        voice.snapshot_padding6 = [0x70 + index as u8];
    }
    let padding = |voice: sm::types::DspChannel| {
        [
            voice.snapshot_padding0,
            voice.snapshot_padding1,
            voice.snapshot_padding2,
            voice.snapshot_padding3,
            voice.snapshot_padding4,
            voice.snapshot_padding5,
            voice.snapshot_padding6,
        ]
    };
    let before_padding = state.channel.map(padding);
    state.reset();
    for voice in &state.channel {
        assert_eq!(
            (
                voice.pitch,
                voice.pitchCounter,
                voice.srcn,
                voice.decodeOffset
            ),
            (0, 0, 0, 0)
        );
        assert_eq!((voice.previousFlags, voice.old, voice.older), (0, 0, 0));
        assert_eq!(voice.decodeBuffer, [0; 19]);
        assert_eq!(voice.adsrRates, [0; 4]);
        assert_eq!(
            (
                voice.rateCounter,
                voice.adsrState,
                voice.sustainLevel,
                voice.gainMode
            ),
            (0, 0, 0, 0)
        );
        assert_eq!(
            (
                voice.gainValue,
                voice.gain,
                voice.sampleOut,
                voice.volumeL,
                voice.volumeR
            ),
            (0, 0, 0, 0, 0)
        );
        assert_eq!(
            [
                voice.pitchModulation,
                voice.useNoise,
                voice.useGain,
                voice.directGain
            ],
            [false; 4]
        );
        assert_eq!([voice.keyOn, voice.keyOff, voice.echoEnable], [false; 3]);
    }
    assert_eq!(state.channel.map(padding), before_padding);
}

// Movies cannot dirty all DSP globals, pointer and pads and inspect reset without native execution.
#[cfg_attr(test, test)]
fn dsp_reset_restores_native_globals_without_changing_apu_memory_pointer_or_padding() {
    use sm::snes::dsp::{DspState, dsp_read};
    let ram = sm::apu_ram::ApuRam::new();
    ram.fill(0x67);
    let pointer = ram.as_mut_ptr();
    assert!(!pointer.is_null());
    let mut state = DspState::new();
    state.apu_ram = pointer;
    state.ram = [0xa5; 128];
    state.dirPage = 0xffff;
    state.evenCycle = true;
    state.mute = false;
    state.reset = false;
    state.masterVolumeL = -128;
    state.masterVolumeR = 127;
    state.noiseSample = 1234;
    state.noiseRate = 0xffff;
    state.noiseCounter = 0xffff;
    state.echoWrites = true;
    state.echoVolumeL = -128;
    state.echoVolumeR = 127;
    state.feedbackVolume = -1;
    state.echoBufferAdr = 0xffff;
    state.echoDelay = 0xffff;
    state.echoRemain = 0xffff;
    state.echoBufferIndex = 0xffff;
    state.firBufferIndex = 0xff;
    state.firValues = [-1; 8];
    state.firBufferL = [-123; 8];
    state.firBufferR = [123; 8];
    state.sampleBuffer = [-1; 1068];
    state.sampleOffset = 0xffff;
    state.snapshot_padding0 = [0xaa];
    state.snapshot_padding1 = [0xbb];
    state.snapshot_padding2 = [0xcc, 0xdd];
    let before_ram: [u8; 65536] = core::array::from_fn(|i| ram.read_at(i));
    state.reset();
    let mut expected_shadow = [0; 128];
    expected_shadow[0x7c] = 0xff;
    assert_eq!(state.ram, expected_shadow);
    assert_eq!(dsp_read(&state, 0x7c), 0xff);
    assert_eq!(
        (state.dirPage, state.evenCycle, state.mute, state.reset),
        (0, false, true, true)
    );
    assert_eq!(
        (state.masterVolumeL, state.masterVolumeR, state.noiseSample),
        (0, 0, -0x4000)
    );
    assert_eq!(
        (state.noiseRate, state.noiseCounter, state.echoWrites),
        (0, 0, false)
    );
    assert_eq!(
        (state.echoVolumeL, state.echoVolumeR, state.feedbackVolume),
        (0, 0, 0)
    );
    assert_eq!(
        (state.echoBufferAdr, state.echoDelay, state.echoRemain),
        (0, 1, 1)
    );
    assert_eq!(
        (
            state.echoBufferIndex,
            state.firBufferIndex,
            state.sampleOffset
        ),
        (0, 0, 0)
    );
    assert_eq!(state.firValues, [0; 8]);
    assert_eq!(state.firBufferL, [0; 8]);
    assert_eq!(state.firBufferR, [0; 8]);
    assert_eq!(state.sampleBuffer, [0; 1068]);
    assert_eq!(state.apu_ram, pointer);
    assert_eq!(state.snapshot_padding0, [0xaa]);
    assert_eq!(state.snapshot_padding1, [0xbb]);
    assert_eq!(state.snapshot_padding2, [0xcc, 0xdd]);
    assert!((0..65536).all(|i| ram.read_at(i) == before_ram[i]));
}

// Movies share the DSP and cannot independently establish BRR, Gaussian or envelope sample values.
#[cfg_attr(test, test)]
fn dsp_cycle_brr_gaussian_and_envelope_have_independent_literal_samples() {
    use sm::snes::dsp::DspState;
    // Retained BRR equations, Gaussian weights [55, 960, 974, 59] at offset 129.
    let vectors = [
        [
            14336, -16384, 2048, -2048, -12288, 10240, -8192, 6144, -4096, 4096, -14336, 12288,
            8192, -6144, -10240, 0,
        ],
        [
            15273, -2066, 111, -1944, -14111, -2990, -10996, -4165, -8001, -3405, 15239, -6194,
            2385, -3909, -13905, -13036,
        ],
        [
            -14651, 0, 15783, -4730, 0, 14674, -12988, 396, 8834, -12205, 0, -9038, -9037, -14898,
            2600, -13846,
        ],
        [
            -15011, 0, 14244, -9222, 0, -15036, 0, -14408, 2782, -11968, 0, -10756, -11136, 15353,
            -6373, 8841,
        ],
        [
            0, -4096, 0, -4096, -4096, 0, -4096, 0, -4096, 0, -4096, 0, 0, -4096, -4096, 0,
        ],
    ];
    let samples = [
        [3839, 3870, 3823, 3854, 3809, 3839],
        [3866, 3897, 3850, 3881, 3835, 3866],
        [3011, 3035, 2999, 3023, 2987, 3011],
        [3001, 3025, 2989, 3013, 2977, 3001],
        [3429, 3457, 3416, 3443, 3403, 3430],
    ];
    for ((header, decoded), pcm) in [0xc0, 0xc4, 0xc8, 0xcc, 0xd0]
        .into_iter()
        .zip(vectors)
        .zip(samples)
    {
        let ram = sm::apu_ram::ApuRam::new();
        let mut state = DspState::new();
        ram.write(0xfffc, header);
        for (index, byte) in [0x78, 0x1f, 0xa5, 0xc3, 0xe2, 0x96, 0x4d, 0xb0]
            .into_iter()
            .enumerate()
        {
            ram.write(0xfffdu16.wrapping_add(index as u16), byte);
        }
        state.masterVolumeL = 127;
        state.masterVolumeR = -128;
        state.echoDelay = 1;
        state.echoRemain = 1;
        let voice = &mut state.channel[0];
        voice.pitch = 0x0811;
        voice.pitchCounter = 0xffff;
        voice.decodeOffset = 0xfffc;
        voice.decodeBuffer[16..].copy_from_slice(&[-9000, 15000, -7000]);
        voice.old = 1000;
        voice.older = -2000;
        voice.gain = 1950;
        voice.adsrRates = [1; 4];
        voice.sustainLevel = 2040;
        voice.volumeL = 64;
        voice.volumeR = -64;
        voice.keyOn = true;
        voice.keyOff = true;
        for expected in [(2047, 1), (2039, 2), (2031, 2)] {
            state.cycle(&ram);
            let voice = &mut state.channel[0];
            assert_eq!((voice.gain, voice.adsrState), expected);
            assert_eq!(voice.rateCounter, 0);
            voice.pitch = 0;
        }
        let voice = &state.channel[0];
        assert_eq!(&voice.decodeBuffer[..3], &[-9000, 15000, -7000]);
        assert_eq!(&voice.decodeBuffer[3..], &decoded);
        assert_eq!(
            (voice.decodeOffset, voice.pitchCounter, voice.previousFlags),
            (5, 0x0810, 0)
        );
        assert_eq!((voice.old, voice.older), (decoded[15], decoded[14]));
        assert_eq!(voice.sampleOut, pcm[5]);
        assert_eq!(&state.sampleBuffer[..6], &pcm);
        assert_eq!((state.ram[8], state.ram[9]), (126, (pcm[5] >> 7) as u8));
        assert_eq!((state.sampleOffset, state.evenCycle), (3, true));
        assert!(voice.keyOn && voice.keyOff);
    }
}

// Movies share the DSP and cannot author signed saturation, dirty noise or inspect a full sample tail.
#[cfg_attr(test, test)]
fn dsp_cycle_noise_signed_mixing_and_full_buffer_keep_ordered_progress() {
    use sm::snes::dsp::DspState;
    let ram = sm::apu_ram::ApuRam::new();
    let mut state = DspState::new();
    state.sampleBuffer = [23; 1068];
    state.sampleOffset = 533;
    state.noiseSample = -16384;
    state.noiseRate = 1;
    state.masterVolumeL = -128;
    state.masterVolumeR = 127;
    state.echoDelay = 1;
    state.echoRemain = 1;
    for (voice, (left, right)) in state.channel.iter_mut().zip([
        (-128, 127),
        (-128, 127),
        (64, -64),
        (64, -64),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ]) {
        voice.useNoise = true;
        voice.useGain = true;
        voice.directGain = true;
        voice.gainValue = 2048;
        voice.volumeL = left;
        voice.volumeR = right;
    }
    state.cycle(&ram);
    assert_eq!(&state.sampleBuffer[1066..], &[1, 0]);
    assert_eq!(
        (
            state.sampleOffset,
            state.evenCycle,
            state.noiseSample,
            state.noiseCounter
        ),
        (534, true, 8192, 0)
    );
    assert_eq!(state.channel.map(|voice| voice.sampleOut), [-16384; 8]);
    assert_eq!((state.ram[8], state.ram[9]), (128, 128));
    state.cycle(&ram);
    assert_eq!(&state.sampleBuffer[..1066], &[23; 1066]);
    assert_eq!(&state.sampleBuffer[1066..], &[1, 0]);
    assert_eq!(
        (state.sampleOffset, state.evenCycle, state.noiseSample),
        (534, false, 4096)
    );
    assert_eq!(state.channel.map(|voice| voice.sampleOut), [8192; 8]);
    assert_eq!(state.firBufferIndex, 2);
    // Negative modulation narrows to u16 before clamping, even with an uncommittable offset.
    state.sampleOffset = 0xffff;
    state.noiseSample = -32768;
    state.channel[1].pitchModulation = true;
    state.channel[1].pitch = 0x3000;
    state.channel[1].pitchCounter = 7;
    state.cycle(&ram);
    assert_eq!(state.channel[1].pitchCounter, 0x4006);
    assert_eq!(state.channel.map(|voice| voice.sampleOut), [-32768; 8]);
    assert_eq!(
        (
            state.sampleOffset,
            state.evenCycle,
            state.noiseSample,
            state.firBufferIndex
        ),
        (0xffff, true, 0, 3)
    );
    state.noiseSample = 3;
    state.cycle(&ram);
    assert_eq!(state.noiseSample, 1);
    state.cycle(&ram);
    assert_eq!(state.noiseSample, -16384);
    assert_eq!(state.sampleOffset, 0xffff);
    assert_eq!(&state.sampleBuffer[..1066], &[23; 1066]);
    assert_eq!(&state.sampleBuffer[1066..], &[1, 0]);
}

// Movies share the DSP and cannot independently establish signed FIR clip, wrapped echo bytes or mute order.
#[cfg_attr(test, test)]
fn dsp_cycle_echo_fir_signed_clip_and_wrapped_writeback_are_independent() {
    use sm::snes::dsp::DspState;
    // Seven 31750 terms narrow 222250 to 25642 before the final signed tap.
    for (last, muted, pcm, bytes) in [
        (-128, false, [9185, 9258], [0xcc, 0x15, 0x32, 0xea]),
        (127, false, [32511, 32767], [0xfe, 0x42, 0x00, 0xbd]),
        (127, true, [0, 0], [0xfe, 0x42, 0x00, 0xbd]),
    ] {
        let ram = sm::apu_ram::ApuRam::new();
        let mut state = DspState::new();
        for (address, byte) in [(0xfffe, 0), (0xffff, 0x40), (0, 0), (1, 0xc0)] {
            ram.write(address, byte);
        }
        state.echoBufferAdr = 0xfffe;
        state.echoDelay = 3;
        state.echoRemain = 1;
        state.firBufferIndex = 7;
        state.firBufferL = [16000; 8];
        state.firBufferR = [-16000; 8];
        state.firValues = [127; 8];
        state.firValues[7] = last;
        state.echoVolumeL = 127;
        state.echoVolumeR = -128;
        state.feedbackVolume = 63;
        state.echoWrites = true;
        state.mute = muted;
        state.noiseSample = 1024;
        let voice = &mut state.channel[0];
        voice.useNoise = true;
        voice.useGain = true;
        voice.directGain = true;
        voice.gainValue = 2048;
        voice.volumeL = 64;
        voice.volumeR = -64;
        voice.echoEnable = true;
        state.cycle(&ram);
        assert_eq!(&state.sampleBuffer[..2], &pcm);
        assert_eq!(
            [ram.read(0xfffe), ram.read(0xffff), ram.read(0), ram.read(1)],
            bytes
        );
        assert_eq!((state.firBufferL[7], state.firBufferR[7]), (8192, -8192));
        assert_eq!(&state.firBufferL[..7], &[16000; 7]);
        assert_eq!(&state.firBufferR[..7], &[-16000; 7]);
        assert_eq!(
            (
                state.firBufferIndex,
                state.echoBufferIndex,
                state.echoRemain
            ),
            (0, 0, 3)
        );
        assert_eq!(state.channel[0].sampleOut, 1024);
        state.echoWrites = false;
        state.mute = true;
        state.cycle(&ram);
        assert_eq!(&state.sampleBuffer[2..4], &[0, 0]);
        assert_eq!(
            [ram.read(0xfffe), ram.read(0xffff), ram.read(0), ram.read(1)],
            bytes
        );
        assert_eq!(
            (
                state.firBufferIndex,
                state.echoBufferIndex,
                state.echoRemain
            ),
            (1, 1, 2)
        );
        assert_eq!((state.sampleOffset, state.evenCycle), (2, false));
    }
}

// Movies cannot restore invalid ADSR/FIR indices or independently inspect each failure's committed prefix.
#[cfg_attr(test, test)]
fn dsp_cycle_dirty_adsr_and_fir_failures_preserve_exact_completed_prefixes() {
    use sm::snes::dsp::DspState;
    let ram = sm::apu_ram::ApuRam::new();
    let mut state = DspState::new();
    state.ram = [0xa5; 128];
    state.sampleBuffer = [27; 1068];
    state.sampleOffset = 0xffff;
    state.evenCycle = true;
    state.noiseSample = 1024;
    state.noiseRate = 1;
    state.echoWrites = true;
    state.firBufferL = [17; 8];
    state.firBufferR = [-17; 8];
    state.channel[0].useNoise = true;
    state.channel[0].useGain = true;
    state.channel[0].directGain = true;
    state.channel[0].gainValue = 1024;
    state.channel[1].pitch = 1;
    state.channel[1].pitchCounter = 5;
    let voice = &mut state.channel[2];
    voice.pitch = 1;
    voice.pitchCounter = 0xffff;
    voice.decodeOffset = 0xfffc;
    voice.decodeBuffer[16..].copy_from_slice(&[11, 22, 33]);
    voice.adsrState = 5;
    voice.gain = 501;
    voice.sampleOut = -99;
    state.channel[3].pitch = 2;
    state.channel[3].pitchCounter = 19;
    ram.write(0xfffc, 0xd0);
    for index in 0..8 {
        ram.write(0xfffdu16.wrapping_add(index), 0xf1);
    }
    let before_ram: [u8; 65536] = core::array::from_fn(|index| ram.read_at(index));
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| state.cycle(&ram))).is_err());
    assert_eq!(state.channel[0].sampleOut, 512);
    assert_eq!((state.ram[8], state.ram[9]), (64, 4));
    assert_eq!(state.channel[1].pitchCounter, 6);
    let voice = &state.channel[2];
    assert_eq!((voice.decodeOffset, voice.pitchCounter), (5, 0));
    assert_eq!(&voice.decodeBuffer[..3], &[11, 22, 33]);
    assert_eq!(
        &voice.decodeBuffer[3..],
        &[
            -4096, 0, -4096, 0, -4096, 0, -4096, 0, -4096, 0, -4096, 0, -4096, 0, -4096, 0
        ]
    );
    assert_eq!((voice.old, voice.older), (0, -4096));
    assert_eq!(
        (voice.adsrState, voice.gain, voice.sampleOut),
        (5, 501, -99)
    );
    assert_eq!((state.ram[0x28], state.ram[0x29]), (0xa5, 0xa5));
    assert_eq!((state.ram[0x38], state.ram[0x39]), (0xa5, 0xa5));
    assert_eq!(state.channel[3].pitchCounter, 19);
    state.channel[2].adsrState = 4;
    state.channel[2].gain = 5;
    state.channel[7].useNoise = true;
    state.channel[7].useGain = true;
    state.channel[7].directGain = true;
    state.channel[7].gainValue = 2048;
    state.firBufferIndex = 8;
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| state.cycle(&ram))).is_err());
    assert_eq!(
        state.channel.map(|voice| voice.sampleOut),
        [512, 0, 0, 0, 0, 0, 0, 1024]
    );
    assert_eq!(
        (state.ram[0x78], state.ram[0x79], state.channel[2].gain),
        (128, 8, 0)
    );
    assert_eq!(state.firBufferL, [17; 8]);
    assert_eq!(state.channel[3].pitchCounter, 21);
    assert_eq!(state.firBufferR, [-17; 8]);
    assert_eq!(state.firBufferIndex, 8);
    assert_eq!((state.echoBufferIndex, state.echoRemain), (0, 0));
    assert_eq!((state.noiseSample, state.noiseCounter), (1024, 0));
    assert_eq!((state.sampleOffset, state.evenCycle), (0xffff, true));
    assert_eq!(state.sampleBuffer, [27; 1068]);
    assert!((0..65536).all(|index| ram.read_at(index) == before_ram[index]));
}

// Movies cannot restore arbitrary timer dividers/counters and inspect zero-target rollover or read-clear bytes.
#[cfg_attr(test, test)]
fn apu_timers_preserve_restored_rollover_read_clear_and_enable_phase() {
    use sm::snes::apu::ApuRegisters;
    let clocks = |t: sm::types::Timer| (t.cycles, t.divider, t.counter);
    let ram = sm::apu_ram::ApuRam::new();
    ram.fill(0xa5);
    let mut apu = ApuRegisters::new();
    apu.cycles = 1;
    apu.cpuCyclesLeft = 1;
    for (timer, (cycles, divider, target, counter, enabled)) in apu.timer.iter_mut().zip([
        (0, 255, 0, 255, true),
        (1, 7, 8, 254, false),
        (0, 1, 2, 15, true),
    ]) {
        timer.cycles = cycles;
        timer.divider = divider;
        timer.target = target;
        timer.counter = counter;
        timer.enabled = enabled;
    }
    apu.cycle(None, || panic!("non-DSP phase"));
    assert_eq!(
        apu.timer.map(clocks),
        [(127, 0, 0), (0, 7, 254), (15, 0, 0)]
    );
    assert_eq!((apu.cycles, apu.cpuCyclesLeft), (2, 0));
    let counters = [0xfd, 0xfe, 0xff].map(|a| apu.read(&ram, a, |_| panic!()));
    assert_eq!(counters, [0, 254, 0]);
    assert_eq!(apu.timer.map(|t| t.counter), [0; 3]);
    assert_eq!([ram.read(0xfd), ram.read(0xfe), ram.read(0xff)], [0xa5; 3]);
    apu.timer[0].divider = 7;
    apu.timer[0].counter = 5;
    apu.timer[1].divider = 9;
    apu.timer[1].counter = 6;
    apu.timer[2].divider = 11;
    apu.timer[2].counter = 7;
    apu.write(&ram, 0xf1, 0x87, |_, _| panic!());
    assert_eq!(
        apu.timer
            .map(|t| (t.cycles, t.divider, t.counter, t.enabled)),
        [(127, 7, 5, true), (0, 0, 0, true), (15, 11, 7, true)]
    );
    apu.write(&ram, 0xf1, 0x81, |_, _| panic!());
    assert_eq!(
        apu.timer.map(|t| (t.divider, t.counter, t.enabled)),
        [(7, 5, true), (0, 0, false), (11, 7, false)]
    );
    apu.write(&ram, 0xf1, 0x87, |_, _| panic!());
    assert_eq!(apu.timer.map(clocks), [(127, 7, 5), (0, 0, 0), (15, 0, 0)]);
    assert_eq!(ram.read(0xf1), 0x87);
    // 255 complete 128-cycle periods leave divider 255; the next event matches target zero.
    let mut apu = ApuRegisters::new();
    apu.cycles = 1;
    apu.timer[0].enabled = true;
    for _ in 0..32640 {
        apu.cycle(None, || {});
    }
    assert_eq!(clocks(apu.timer[0]), (0, 255, 0));
    apu.cycle(None, || {});
    assert_eq!(clocks(apu.timer[0]), (127, 0, 1));
    assert_eq!(apu.read(&ram, 0xfd, |_| panic!()), 1);
    assert_eq!(apu.read(&ram, 0xfd, |_| panic!()), 0);
}

// Movies cannot author full DSP addresses or failed reads and independently inspect boot/port/RAM effects.
#[cfg_attr(test, test)]
fn apu_bus_preserves_boot_ports_ram_shadow_and_masked_dsp_history() {
    use sm::snes::apu::ApuRegisters;
    use sm::snes::dsp::{DspState, dsp_read, write_register};
    use std::panic::{AssertUnwindSafe as unwind, catch_unwind as catch};
    let ram = sm::apu_ram::ApuRam::new();
    ram.fill(0xa5);
    let mut apu = ApuRegisters::new();
    let mut dsp = DspState::new();
    dsp.ram[0x7f] = 0x69;
    apu.romReadable = true;
    apu.inPorts = [11, 22, 33, 44, 55, 66];
    for (address, expected) in [
        (0xffbf, 0xa5),
        (0xffc0, 0xcd),
        (0xfffe, 0xc0),
        (0xffff, 0xff),
    ] {
        assert_eq!(apu.read(&ram, address, |_| panic!()), expected);
    }
    apu.write(&ram, 0xffc0, 0x18, |_, _| panic!());
    apu.write(&ram, 0xffff, 0x19, |_, _| panic!());
    assert_eq!(apu.read(&ram, 0xffc0, |_| panic!()), 0xcd);
    for address in [0xf0, 0xf1, 0xfa, 0xfb, 0xfc] {
        assert_eq!(apu.read(&ram, address, |_| panic!()), 0);
    }
    let ports: [u8; 6] = core::array::from_fn(|i| apu.read(&ram, 0xf4 + i as u16, |_| panic!()));
    assert_eq!(ports, [11, 22, 33, 44, 55, 66]);
    apu.write(&ram, 0xf0, 0x71, |_, _| panic!());
    assert_eq!(
        (ram.read(0xf0), apu.read(&ram, 0xf0, |_| panic!())),
        (0x71, 0)
    );
    apu.write(&ram, 0xf2, 0xff, |_, _| panic!());
    assert_eq!(apu.read(&ram, 0xf2, |_| panic!()), 0xff);
    apu.write(&ram, 0xf3, 0x35, |_, _| {
        panic!("high DSP writes are ignored")
    });
    assert_eq!(
        apu.read(&ram, 0xf3, |address| {
            assert_eq!(address, 0x7f);
            dsp_read(&dsp, address)
        }),
        0x69
    );
    let history = apu.history;
    assert_eq!(
        (history.count, history.addr[0], history.val[0]),
        (1, 0xff, 0x35)
    );
    assert_eq!(ram.read(0xf3), 0x35);
    apu.write(&ram, 0xf2, 0x7f, |_, _| panic!());
    apu.write(&ram, 0xf3, 0x4d, |a, v| {
        write_register(&mut dsp, &ram, a, v)
    });
    assert_eq!(
        (dsp.firValues[7], dsp.ram[0x7f], ram.read(0xf3)),
        (77, 0x4d, 0x4d)
    );
    assert_eq!(apu.history.count, 2);
    assert_eq!(&apu.history.addr[..2], &[0xff, 0x7f]);
    assert_eq!(&apu.history.val[..2], &[0x35, 0x4d]);
    assert!(
        catch(unwind(
            || apu.read(&ram, 0xf3, |_| panic!("DSP read failure"))
        ))
        .is_err()
    );
    assert_eq!((apu.history.count, ram.read(0xf3)), (2, 0x4d));
    for (a, v) in [(0xfa, 0), (0xfb, 2), (0xfc, 255)] {
        apu.write(&ram, a, v, |_, _| panic!());
        assert_eq!(apu.read(&ram, a, |_| panic!()), 0);
    }
    assert_eq!(apu.timer.map(|t| t.target), [0, 2, 255]);
    assert_eq!(
        [ram.read(0xfa), ram.read(0xfb), ram.read(0xfc)],
        [0, 2, 255]
    );
    for (index, value) in [91, 92, 93, 94, 95, 96].into_iter().enumerate() {
        apu.write(&ram, 0xf4 + index as u16, value, |_, _| panic!());
    }
    assert_eq!(apu.outPorts, [91, 92, 93, 94]);
    assert_eq!(apu.inPorts, [11, 22, 33, 44, 95, 96]);
    let shadow: [u8; 6] = core::array::from_fn(|i| ram.read(0xf4 + i as u16));
    assert_eq!(shadow, [91, 92, 93, 94, 95, 96]);
    apu.write(&ram, 0xf1, 0x30, |_, _| panic!());
    assert_eq!(apu.inPorts, [0, 0, 0, 0, 95, 96]);
    assert_eq!(apu.outPorts, [91, 92, 93, 94]);
    assert!(!apu.romReadable);
    assert_eq!(apu.read(&ram, 0xffc0, |_| panic!()), 0x18);
    assert_eq!(apu.read(&ram, 0xffff, |_| panic!()), 0x19);
    assert_eq!(ram.read(0xf1), 0x30);
}

// Movies cannot restore invalid history counts or fail DSP callbacks between history, cadence and RAM commits.
#[cfg_attr(test, test)]
fn apu_callback_failures_keep_history_cycle_and_reset_prefixes() {
    use sm::snes::apu::ApuRegisters;
    let clocks = |t: sm::types::Timer| (t.cycles, t.divider, t.counter);
    use sm::snes::dsp::{DspState, write_register};
    use std::panic::{AssertUnwindSafe as unwind, catch_unwind as catch};
    let ram = sm::apu_ram::ApuRam::new();
    ram.fill(0xa5);
    let mut apu = ApuRegisters::new();
    let mut dsp = DspState::new();
    apu.dspAdr = 0x7f;
    apu.history.addr = [0xa1; 256];
    apu.history.val = [0xb2; 256];
    for (count, committed) in [(257, 258), (0xffff_ffff, 0)] {
        apu.history.count = count;
        assert!(catch(unwind(|| apu.write(&ram, 0xf3, 0x44, |_, _| panic!()))).is_err());
        assert_eq!(apu.history.count, committed);
        assert_eq!(apu.history.addr, [0xa1; 256]);
        assert_eq!(apu.history.val, [0xb2; 256]);
        assert_eq!(ram.read(0xf3), 0xa5);
    }
    assert!(
        catch(unwind(|| apu.write(&ram, 0xf3, 0x44, |address, value| {
            write_register(&mut dsp, &ram, address, value);
            panic!("DSP write failure");
        })))
        .is_err()
    );
    let history = apu.history;
    assert_eq!(
        (history.count, history.addr[0], history.val[0]),
        (1, 0x7f, 0x44)
    );
    assert_eq!((dsp.firValues[7], ram.read(0xf3)), (68, 0xa5));
    apu.history.count = 256;
    apu.write(&ram, 0xf3, 0x55, |a, v| {
        write_register(&mut dsp, &ram, a, v)
    });
    assert_eq!((apu.history.count, apu.history.val[0]), (256, 0x44));
    assert_eq!((dsp.firValues[7], ram.read(0xf3)), (85, 0x55));
    apu.cycles = 32;
    apu.timer[0].divider = 2;
    apu.timer[0].target = 3;
    apu.timer[0].counter = 4;
    apu.timer[0].enabled = true;
    assert!(
        catch(unwind(|| apu.cycle(Some(9), || {
            dsp.cycle(&ram);
            panic!("DSP cycle failure");
        })))
        .is_err()
    );
    assert_eq!((apu.cpuCyclesLeft, apu.cycles), (8, 32));
    assert_eq!(clocks(apu.timer[0]), (0, 2, 4));
    assert_eq!((dsp.sampleOffset, dsp.evenCycle), (1, true));
    apu.cycle(None, || dsp.cycle(&ram));
    assert_eq!((apu.cpuCyclesLeft, apu.cycles), (7, 33));
    assert_eq!(clocks(apu.timer[0]), (127, 0, 5));
    apu.cycles = 0xffff_ffff;
    apu.cycle(Some(0), || panic!("non-DSP phase"));
    assert_eq!((apu.cpuCyclesLeft, apu.cycles), (255, 0));
    apu.cycle(None, || dsp.cycle(&ram));
    assert_eq!(
        (apu.cpuCyclesLeft, apu.cycles, dsp.sampleOffset),
        (254, 1, 3)
    );
    apu.pad = [0x22; 6];
    apu.snapshot_padding0 = [0x11; 2];
    let history = apu.history;
    apu.inPorts = [3; 6];
    apu.outPorts = [4; 4];
    for t in &mut apu.timer {
        t.target = 9;
        t.counter = 10;
        t.enabled = true;
    }
    apu.begin_reset();
    assert_eq!(apu.read(&ram, 0xfffe, |_| panic!()), 0xc0);
    assert_eq!((apu.cycles, apu.cpuCyclesLeft), (1, 254));
    assert_eq!((apu.dspAdr, ram.read(0xf3)), (0x7f, 0x55));
    apu.finish_reset(&ram);
    assert!(apu.romReadable);
    assert_eq!((apu.dspAdr, apu.cycles), (0, 0));
    assert_eq!((apu.cpuCyclesLeft, apu.history.count), (7, 0));
    assert_eq!(apu.inPorts, [0; 6]);
    assert_eq!(apu.outPorts, [0; 4]);
    assert_eq!(
        apu.timer
            .map(|t| (t.cycles, t.divider, t.target, t.counter, t.enabled)),
        [(0, 0, 0, 0, false); 3]
    );
    assert_eq!(apu.history.addr, history.addr);
    assert_eq!(apu.history.val, history.val);
    assert_eq!((apu.pad, apu.snapshot_padding0), ([0x22; 6], [0x11; 2]));
    assert!((0..65536).all(|i| ram.read_at(i) == 0));
}

// Movies do not enumerate cold native-SPC opcodes 00..3f with independent cycle/PC/PSW observations.
#[cfg_attr(test, test)]
fn spc_native_cold_matrix_00_through_3f() {
    use sm::snes::spc::{BusOperation::*, Spc};
    let rows = [
        (2, 0x0201, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (5, 0x0203, 0x00),
        (3, 0x0202, 0x02),
        (4, 0x0203, 0x02),
        (3, 0x0201, 0x02),
        (6, 0x0202, 0x02),
        (2, 0x0202, 0x02),
        (6, 0x0203, 0x02),
        (5, 0x0203, 0x00),
        (4, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (4, 0x0201, 0x00),
        (6, 0x0203, 0x02),
        (8, 0x0000, 0x10),
        (4, 0x0202, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (7, 0x0203, 0x00),
        (4, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (5, 0x0203, 0x02),
        (6, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (5, 0x0201, 0x02),
        (6, 0x0202, 0x80),
        (5, 0x0202, 0x02),
        (2, 0x0201, 0x02),
        (2, 0x0201, 0x80),
        (4, 0x0203, 0x03),
        (6, 0x0000, 0x00),
        (2, 0x0201, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (5, 0x0203, 0x00),
        (3, 0x0202, 0x02),
        (4, 0x0203, 0x02),
        (3, 0x0201, 0x02),
        (6, 0x0202, 0x02),
        (2, 0x0202, 0x02),
        (6, 0x0203, 0x02),
        (5, 0x0203, 0x01),
        (4, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (4, 0x0201, 0x00),
        (5, 0x0203, 0x00),
        (4, 0x0202, 0x00),
        (2, 0x0202, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (7, 0x0203, 0x00),
        (4, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (5, 0x0203, 0x02),
        (6, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (5, 0x0201, 0x02),
        (6, 0x0202, 0x00),
        (5, 0x0202, 0x02),
        (2, 0x0201, 0x02),
        (2, 0x0201, 0x00),
        (3, 0x0202, 0x03),
        (8, 0x0000, 0x00),
    ];
    for (index, (expected_cycles, expected_pc, psw)) in rows.into_iter().enumerate() {
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.sp = 0xff;
        let ram = sm::apu_ram::ApuRam::new();
        ram.write(0x0200, index as u8 + 0x00);
        let cycles = cpu.run_opcode(|operation| match operation {
            Read(a) => ram.read(a),
            Write(a, v) => {
                ram.write(a, v);
                0
            }
        });
        assert_eq!(
            (cycles, cpu.cyclesUsed, cpu.pc),
            (expected_cycles, expected_cycles, expected_pc)
        );
        assert_eq!(
            [cpu.n, cpu.v, cpu.p, cpu.b, cpu.h, cpu.i, cpu.z, cpu.c],
            [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1].map(|bit| psw & bit != 0)
        );
    }
}

// Movies do not enumerate cold native-SPC opcodes 40..7f with independent cycle/PC/PSW observations.
#[cfg_attr(test, test)]
fn spc_native_cold_matrix_40_through_7f() {
    use sm::snes::spc::{BusOperation::*, Spc};
    let rows = [
        (2, 0x0201, 0x20),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (5, 0x0203, 0x00),
        (3, 0x0202, 0x02),
        (4, 0x0203, 0x02),
        (3, 0x0201, 0x02),
        (6, 0x0202, 0x02),
        (2, 0x0202, 0x02),
        (6, 0x0203, 0x02),
        (4, 0x0203, 0x00),
        (4, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (4, 0x0201, 0x00),
        (6, 0x0203, 0x02),
        (6, 0xff00, 0x00),
        (4, 0x0202, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (7, 0x0203, 0x00),
        (4, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (5, 0x0203, 0x02),
        (6, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (5, 0x0201, 0x02),
        (4, 0x0202, 0x03),
        (5, 0x0202, 0x02),
        (2, 0x0201, 0x02),
        (2, 0x0201, 0x02),
        (4, 0x0203, 0x03),
        (3, 0x0000, 0x00),
        (2, 0x0201, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (5, 0x0203, 0x00),
        (3, 0x0202, 0x03),
        (4, 0x0203, 0x03),
        (3, 0x0201, 0x03),
        (6, 0x0202, 0x03),
        (2, 0x0202, 0x03),
        (6, 0x0203, 0x03),
        (4, 0x0203, 0x00),
        (4, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (4, 0x0201, 0x00),
        (7, 0x0203, 0x00),
        (5, 0x0000, 0x00),
        (2, 0x0202, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (7, 0x0203, 0x00),
        (4, 0x0202, 0x03),
        (5, 0x0203, 0x03),
        (5, 0x0203, 0x03),
        (6, 0x0202, 0x03),
        (5, 0x0203, 0x03),
        (5, 0x0201, 0x03),
        (5, 0x0202, 0x02),
        (5, 0x0202, 0x02),
        (2, 0x0201, 0x02),
        (2, 0x0201, 0x02),
        (3, 0x0202, 0x03),
        (6, 0x0000, 0x00),
    ];
    for (index, (expected_cycles, expected_pc, psw)) in rows.into_iter().enumerate() {
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.sp = 0xff;
        let ram = sm::apu_ram::ApuRam::new();
        ram.write(0x0200, index as u8 + 0x40);
        let cycles = cpu.run_opcode(|operation| match operation {
            Read(a) => ram.read(a),
            Write(a, v) => {
                ram.write(a, v);
                0
            }
        });
        assert_eq!(
            (cycles, cpu.cyclesUsed, cpu.pc),
            (expected_cycles, expected_cycles, expected_pc)
        );
        assert_eq!(
            [cpu.n, cpu.v, cpu.p, cpu.b, cpu.h, cpu.i, cpu.z, cpu.c],
            [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1].map(|bit| psw & bit != 0)
        );
    }
}

// Movies do not enumerate cold native-SPC opcodes 80..bf with independent cycle/PC/PSW observations.
#[cfg_attr(test, test)]
fn spc_native_cold_matrix_80_through_bf() {
    use sm::snes::spc::{BusOperation::*, Spc};
    let rows = [
        (2, 0x0201, 0x01),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (5, 0x0203, 0x00),
        (3, 0x0202, 0x02),
        (4, 0x0203, 0x02),
        (3, 0x0201, 0x02),
        (6, 0x0202, 0x02),
        (2, 0x0202, 0x02),
        (6, 0x0203, 0x02),
        (5, 0x0203, 0x00),
        (4, 0x0202, 0x80),
        (5, 0x0203, 0x80),
        (2, 0x0202, 0x02),
        (4, 0x0201, 0x00),
        (5, 0x0203, 0x00),
        (4, 0x0202, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (7, 0x0203, 0x00),
        (4, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (5, 0x0203, 0x02),
        (6, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (5, 0x0201, 0x02),
        (5, 0x0202, 0x0b),
        (5, 0x0202, 0x80),
        (2, 0x0201, 0x80),
        (2, 0x0201, 0x80),
        (12, 0x0201, 0xc8),
        (5, 0x0201, 0x02),
        (2, 0x0201, 0x04),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (5, 0x0203, 0x00),
        (3, 0x0202, 0x80),
        (4, 0x0203, 0x80),
        (3, 0x0201, 0x80),
        (6, 0x0202, 0x80),
        (2, 0x0202, 0x80),
        (6, 0x0203, 0x80),
        (4, 0x0203, 0x00),
        (4, 0x0202, 0x00),
        (5, 0x0203, 0x00),
        (2, 0x0202, 0x03),
        (4, 0x0201, 0x00),
        (4, 0x0201, 0x00),
        (2, 0x0202, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (7, 0x0203, 0x00),
        (4, 0x0202, 0x80),
        (5, 0x0203, 0x80),
        (5, 0x0203, 0x80),
        (6, 0x0202, 0x80),
        (5, 0x0203, 0x80),
        (5, 0x0201, 0x80),
        (5, 0x0202, 0x02),
        (5, 0x0202, 0x00),
        (2, 0x0201, 0x00),
        (2, 0x0201, 0x00),
        (3, 0x0201, 0x80),
        (4, 0x0201, 0x02),
    ];
    for (index, (expected_cycles, expected_pc, psw)) in rows.into_iter().enumerate() {
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.sp = 0xff;
        let ram = sm::apu_ram::ApuRam::new();
        ram.write(0x0200, index as u8 + 0x80);
        let cycles = cpu.run_opcode(|operation| match operation {
            Read(a) => ram.read(a),
            Write(a, v) => {
                ram.write(a, v);
                0
            }
        });
        assert_eq!(
            (cycles, cpu.cyclesUsed, cpu.pc),
            (expected_cycles, expected_cycles, expected_pc)
        );
        assert_eq!(
            [cpu.n, cpu.v, cpu.p, cpu.b, cpu.h, cpu.i, cpu.z, cpu.c],
            [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1].map(|bit| psw & bit != 0)
        );
    }
}

// Movies do not enumerate cold native-SPC opcodes c0..ff with independent cycle/PC/PSW observations.
#[cfg_attr(test, test)]
fn spc_native_cold_matrix_c0_through_ff() {
    use sm::snes::spc::{BusOperation::*, Spc};
    let rows = [
        (2, 0x0201, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (5, 0x0203, 0x00),
        (4, 0x0202, 0x00),
        (5, 0x0203, 0x00),
        (4, 0x0201, 0x00),
        (7, 0x0202, 0x00),
        (2, 0x0202, 0x03),
        (5, 0x0203, 0x00),
        (6, 0x0203, 0x00),
        (4, 0x0202, 0x00),
        (5, 0x0203, 0x00),
        (2, 0x0202, 0x02),
        (4, 0x0201, 0x00),
        (9, 0x0201, 0x02),
        (4, 0x0202, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (7, 0x0203, 0x00),
        (5, 0x0202, 0x00),
        (6, 0x0203, 0x00),
        (6, 0x0203, 0x00),
        (7, 0x0202, 0x00),
        (4, 0x0202, 0x00),
        (5, 0x0202, 0x00),
        (5, 0x0202, 0x00),
        (5, 0x0202, 0x00),
        (2, 0x0201, 0x80),
        (2, 0x0201, 0x02),
        (6, 0x0203, 0x00),
        (3, 0x0201, 0x02),
        (2, 0x0201, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (5, 0x0203, 0x00),
        (3, 0x0202, 0x02),
        (4, 0x0203, 0x02),
        (3, 0x0201, 0x02),
        (6, 0x0202, 0x02),
        (2, 0x0202, 0x02),
        (4, 0x0203, 0x02),
        (5, 0x0203, 0x00),
        (3, 0x0202, 0x02),
        (4, 0x0203, 0x02),
        (3, 0x0201, 0x01),
        (4, 0x0201, 0x00),
        (3, 0x0201, 0x00),
        (2, 0x0202, 0x00),
        (8, 0x0000, 0x00),
        (4, 0x0202, 0x00),
        (7, 0x0203, 0x00),
        (4, 0x0202, 0x02),
        (5, 0x0203, 0x02),
        (5, 0x0203, 0x02),
        (6, 0x0202, 0x02),
        (3, 0x0202, 0x02),
        (4, 0x0202, 0x02),
        (5, 0x0203, 0x00),
        (4, 0x0202, 0x02),
        (2, 0x0201, 0x00),
        (2, 0x0201, 0x02),
        (6, 0x0202, 0x00),
        (3, 0x0201, 0x00),
    ];
    for (index, (expected_cycles, expected_pc, psw)) in rows.into_iter().enumerate() {
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.sp = 0xff;
        let ram = sm::apu_ram::ApuRam::new();
        ram.write(0x0200, index as u8 + 0xc0);
        let cycles = cpu.run_opcode(|operation| match operation {
            Read(a) => ram.read(a),
            Write(a, v) => {
                ram.write(a, v);
                0
            }
        });
        assert_eq!(
            (cycles, cpu.cyclesUsed, cpu.pc),
            (expected_cycles, expected_cycles, expected_pc)
        );
        assert_eq!(
            [cpu.n, cpu.v, cpu.p, cpu.b, cpu.h, cpu.i, cpu.z, cpu.c],
            [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1].map(|bit| psw & bit != 0)
        );
    }
}

// Movies cannot fail native SPC reset/vector/fetch reads or inspect stopped-return versus stored cycles.
#[cfg_attr(test, test)]
fn spc_reset_stop_and_fetch_failure_keep_native_prefixes() {
    use sm::snes::spc::{BusOperation::*, Spc};
    use std::panic::{AssertUnwindSafe as unwind, catch_unwind as catch};
    let flags = |s: &Spc| [s.c, s.z, s.v, s.n, s.i, s.h, s.p, s.b, s.stopped];
    let mut cpu = Spc::new();
    cpu.a = 55;
    cpu.x = 66;
    cpu.y = 77;
    cpu.sp = 88;
    cpu.pc = 0xbeef;
    cpu.c = true;
    cpu.z = true;
    cpu.v = true;
    cpu.n = true;
    cpu.i = true;
    cpu.h = true;
    cpu.p = true;
    cpu.b = true;
    cpu.stopped = true;
    cpu.cyclesUsed = 99;
    let mut reads = Vec::new();
    assert!(
        catch(unwind(|| cpu.reset(|op| {
            reads.push(op);
            match op {
                Read(0xfffe) => 0x34,
                _ => panic!("high vector"),
            }
        })))
        .is_err()
    );
    assert_eq!(reads, [Read(0xfffe), Read(0xffff)]);
    assert_eq!((cpu.a, cpu.x, cpu.y, cpu.sp, cpu.pc), (0, 0, 0, 0, 0xbeef));
    assert_eq!(flags(&cpu), [true; 9]);
    assert_eq!(cpu.cyclesUsed, 99);
    let ram = sm::apu_ram::ApuRam::new();
    let mut apu = sm::snes::apu::ApuRegisters::new();
    apu.begin_reset();
    reads.clear();
    cpu.reset(|op| {
        reads.push(op);
        match op {
            Read(a) => apu.read(&ram, a, |_| panic!()),
            Write(_, _) => panic!(),
        }
    });
    assert_eq!(reads, [Read(0xfffe), Read(0xffff)]);
    assert_eq!((cpu.pc, cpu.cyclesUsed), (0xffc0, 0));
    assert_eq!(flags(&cpu), [false; 9]);
    for opcode in [0xef, 0xff] {
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        ram.write(0x0200, opcode);
        assert_eq!(
            cpu.run_opcode(|op| match op {
                Read(a) => ram.read(a),
                _ => panic!(),
            }),
            3
        );
        assert!(cpu.stopped);
        assert_eq!(cpu.pc, 0x0201);
        assert_eq!(
            cpu.run_opcode(|_| panic!("stopped performs no bus read")),
            1
        );
        assert_eq!((cpu.pc, cpu.cyclesUsed), (0x0201, 0));
    }
    cpu.stopped = false;
    cpu.pc = 0xffff;
    cpu.cyclesUsed = 99;
    assert!(catch(unwind(|| cpu.run_opcode(|_| panic!("opcode fetch")))).is_err());
    assert_eq!((cpu.pc, cpu.cyclesUsed), (0, 0));
}

// Movies do not supply independent native ADC/SBC/logic flags or ordered two-memory operand observations.
#[cfg_attr(test, test)]
fn spc_alu_literals_cover_signed_carry_overflow_and_memory_order() {
    use sm::snes::spc::{BusOperation::*, Spc};
    let rows = [
        (0x08, 0x80, 0x7f, false, 0xff, 0xc8),
        (0x28, 0x80, 0x7f, false, 0, 0x4a),
        (0x48, 0x80, 0x7f, false, 0xff, 0xc8),
        (0x68, 0x80, 0x7f, false, 0x80, 0x49),
        (0x88, 0x80, 0x7f, false, 0xff, 0x80),
        (0xa8, 0x80, 0x7f, false, 0, 0x43),
        (0x88, 0x7f, 1, false, 0x80, 0xc8),
        (0x88, 0xff, 0, true, 0, 0x0b),
        (0xa8, 0, 1, true, 0xff, 0x80),
        (0xa8, 0x80, 1, true, 0x7f, 0x41),
        (0x68, 0x7f, 0x7f, false, 0x7f, 0x4b),
    ];
    for (opcode, a, value, carry, expected, psw) in rows {
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.a = a;
        cpu.c = carry;
        cpu.v = true;
        cpu.h = true;
        let ram = sm::apu_ram::ApuRam::new();
        ram.write(0x0200, opcode);
        ram.write(0x0201, value);
        let mut trace = Vec::new();
        assert_eq!(
            cpu.run_opcode(|op| {
                trace.push(op);
                match op {
                    Read(a) => ram.read(a),
                    _ => panic!(),
                }
            }),
            2
        );
        assert_eq!((cpu.a, cpu.pc), (expected, 0x0202));
        assert_eq!(trace, [Read(0x0200), Read(0x0201)]);
        assert_eq!(
            [cpu.n, cpu.v, cpu.p, cpu.b, cpu.h, cpu.i, cpu.z, cpu.c],
            [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1].map(|b| psw & b != 0)
        );
    }
    let mut cpu = Spc::new();
    cpu.pc = 0x0200;
    let ram = sm::apu_ram::ApuRam::new();
    for (a, v) in [
        (0x0200, 0x89),
        (0x0201, 0x10),
        (0x0202, 0x20),
        (0x10, 0xff),
        (0x20, 1),
    ] {
        ram.write(a, v);
    }
    let mut trace = Vec::new();
    assert_eq!(
        cpu.run_opcode(|op| {
            trace.push(op);
            match op {
                Read(a) => ram.read(a),
                Write(a, v) => {
                    ram.write(a, v);
                    0
                }
            }
        }),
        6
    );
    assert_eq!(
        trace,
        [
            Read(0x0200),
            Read(0x0201),
            Read(0x0202),
            Read(0x10),
            Read(0x20),
            Write(0x20, 0)
        ]
    );
    assert_eq!(
        (ram.read(0x20), cpu.c, cpu.h, cpu.z, cpu.v),
        (0, true, true, true, false)
    );
}

// Movies cannot independently establish nonzero native shift/rotate/decrement flags and write bus order.
#[cfg_attr(test, test)]
fn spc_shift_literals_preserve_memory_then_flags_and_accumulator_results() {
    use sm::snes::spc::{BusOperation::*, Spc};
    for (memory_op, accumulator_op, value, carry, expected, psw) in [
        (0x0b, 0x1c, 0x80, false, 0, 3),
        (0x2b, 0x3c, 0x80, true, 1, 1),
        (0x4b, 0x5c, 1, false, 0, 3),
        (0x6b, 0x7c, 1, true, 0x80, 0x81),
        (0x8b, 0x9c, 0, true, 0xff, 0x81),
        (0xab, 0xbc, 0xff, true, 0, 3),
    ] {
        let ram = sm::apu_ram::ApuRam::new();
        ram.write(0x0200, memory_op);
        ram.write(0x0201, 0x10);
        ram.write(0x10, value);
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.a = 55;
        cpu.c = carry;
        let mut trace = Vec::new();
        assert_eq!(
            cpu.run_opcode(|op| {
                trace.push(op);
                match op {
                    Read(a) => ram.read(a),
                    Write(a, v) => {
                        ram.write(a, v);
                        0
                    }
                }
            }),
            4
        );
        assert_eq!(
            trace,
            [
                Read(0x0200),
                Read(0x0201),
                Read(0x10),
                Write(0x10, expected)
            ]
        );
        assert_eq!((ram.read(0x10), cpu.a, cpu.pc), (expected, 55, 0x0202));
        assert_eq!([cpu.n, cpu.z, cpu.c], [0x80, 2, 1].map(|b| psw & b != 0));
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.a = value;
        cpu.c = carry;
        ram.write(0x0200, accumulator_op);
        assert_eq!(
            cpu.run_opcode(|op| match op {
                Read(0x0200) => accumulator_op,
                _ => panic!(),
            }),
            2
        );
        assert_eq!((cpu.a, cpu.pc), (expected, 0x0201));
        assert_eq!([cpu.n, cpu.z, cpu.c], [0x80, 2, 1].map(|b| psw & b != 0));
    }
}

// Movies do not establish native word half-carry quirks or wrapped low/high writes independently.
#[cfg_attr(test, test)]
fn spc_word_literals_cover_native_half_carry_and_page_wrapped_words() {
    use sm::snes::spc::{BusOperation::*, Spc};
    for (opcode, ya, word, expected_ya, expected_word, psw, cycles) in [
        (0x3a, 0x1234, 0xffff, 0x1234, 0, 0x4b, 6),
        (0x1a, 0x1234, 0, 0x1234, 0xffff, 0xc9, 6),
        (0x5a, 0x1000, 0x1000, 0x1000, 0x1000, 0x4b, 4),
        (0x7a, 0x0fff, 0, 0x0fff, 0, 0x08, 5),
        (0x7a, 0x7fff, 1, 0x8000, 1, 0xc8, 5),
        (0x9a, 0, 0, 0, 0, 0x0b, 5),
        (0x9a, 0, 1, 0xffff, 1, 0x80, 5),
    ] {
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.p = true;
        cpu.a = ya as u8;
        cpu.y = (ya >> 8) as u8;
        cpu.c = true;
        cpu.v = true;
        cpu.h = true;
        let ram = sm::apu_ram::ApuRam::new();
        ram.write(0x0200, opcode);
        ram.write(0x0201, 0xff);
        ram.write(0x01ff, word as u8);
        ram.write(0x0100, (word >> 8) as u8);
        let mut trace = Vec::new();
        assert_eq!(
            cpu.run_opcode(|op| {
                trace.push(op);
                match op {
                    Read(a) => ram.read(a),
                    Write(a, v) => {
                        ram.write(a, v);
                        0
                    }
                }
            }),
            cycles
        );
        assert_eq!(
            &trace[..4],
            &[Read(0x0200), Read(0x0201), Read(0x01ff), Read(0x0100)]
        );
        assert_eq!(
            (cpu.a, cpu.y),
            (expected_ya as u8, (expected_ya >> 8) as u8)
        );
        assert_eq!(
            (ram.read(0x01ff), ram.read(0x0100)),
            (expected_word as u8, (expected_word >> 8) as u8)
        );
        assert_eq!(
            [cpu.n, cpu.v, cpu.h, cpu.z, cpu.c],
            [0x80, 0x40, 8, 2, 1].map(|b| psw & b != 0)
        );
        if opcode == 0x1a || opcode == 0x3a {
            assert_eq!(
                &trace[4..],
                &[
                    Write(0x01ff, expected_word as u8),
                    Write(0x0100, (expected_word >> 8) as u8)
                ]
            );
        } else {
            assert_eq!(trace.len(), 4);
        }
    }
}

// Movies do not supply independent native division-by-zero, overflow quotient, BCD or multiply flag observations.
#[cfg_attr(test, test)]
fn spc_division_bcd_and_multiply_have_literal_native_results() {
    use sm::snes::spc::{BusOperation::*, Spc};
    for (opcode, a, x, y, carry, half, expected_a, expected_y, psw, cycles) in [
        (0x9e, 0x34, 0, 0x12, true, false, 0xff, 0x34, 0xc9, 12),
        (0x9e, 0x34, 3, 0x12, true, false, 0x11, 1, 0x41, 12),
        (0x9e, 0, 17, 1, true, false, 15, 1, 9, 12),
        (0x9e, 20, 10, 0, true, false, 2, 0, 1, 12),
        (0xcf, 0xff, 0, 0xff, true, false, 1, 0xfe, 0x81, 9),
        (0xcf, 0, 0, 0xff, true, false, 0, 0, 3, 9),
        (0xdf, 0x9a, 0, 0, false, false, 0, 0, 3, 3),
        (0xdf, 0x15, 0, 0, false, true, 0x1b, 0, 8, 3),
        (0xbe, 0, 0, 0, false, false, 0x9a, 0, 0x80, 3),
        (0xbe, 0xa0, 0, 0, true, true, 0x40, 0, 8, 3),
        (0xbe, 0, 0, 0, true, true, 0, 0, 0x0b, 3),
        (0x9f, 0x8f, 0, 0, true, false, 0xf8, 0, 0x81, 5),
    ] {
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.a = a;
        cpu.x = x;
        cpu.y = y;
        cpu.c = carry;
        cpu.h = half;
        assert_eq!(
            cpu.run_opcode(|op| match op {
                Read(0x0200) => opcode,
                _ => panic!(),
            }),
            cycles
        );
        assert_eq!(
            (cpu.a, cpu.y, cpu.x, cpu.pc),
            (expected_a, expected_y, x, 0x0201)
        );
        assert_eq!(
            [cpu.n, cpu.v, cpu.h, cpu.z, cpu.c],
            [0x80, 0x40, 8, 2, 1].map(|b| psw & b != 0)
        );
    }
}

// Movies cannot enumerate independent wrapped branch predicates and every nonzero direct-page bit selector.
#[cfg_attr(test, test)]
fn spc_branch_and_direct_bits_keep_native_predicates_cycles_and_read_order() {
    use sm::snes::spc::{BusOperation::*, Spc};
    for (opcode, when_false, when_true) in [
        (0x10, true, false),
        (0x30, false, true),
        (0x50, true, false),
        (0x70, false, true),
        (0x90, true, false),
        (0xb0, false, true),
        (0xd0, true, false),
        (0xf0, false, true),
    ] {
        for (flag, taken) in [(false, when_false), (true, when_true)] {
            let mut cpu = Spc::new();
            cpu.pc = 0xffff;
            cpu.n = flag;
            cpu.v = flag;
            cpu.c = flag;
            cpu.z = flag;
            let mut trace = Vec::new();
            let cycles = cpu.run_opcode(|op| {
                trace.push(op);
                match op {
                    Read(0xffff) => opcode,
                    Read(0) => 0xfc,
                    _ => panic!(),
                }
            });
            assert_eq!((cycles, cpu.pc), if taken { (4, 0xfffd) } else { (2, 1) });
            assert_eq!(trace, [Read(0xffff), Read(0)]);
            assert_eq!([cpu.n, cpu.v, cpu.c, cpu.z], [flag; 4]);
        }
    }
    for (set, clear, bbs, bbr, mask) in [
        (0x02, 0x12, 0x03, 0x13, 1),
        (0x22, 0x32, 0x23, 0x33, 2),
        (0x42, 0x52, 0x43, 0x53, 4),
        (0x62, 0x72, 0x63, 0x73, 8),
        (0x82, 0x92, 0x83, 0x93, 16),
        (0xa2, 0xb2, 0xa3, 0xb3, 32),
        (0xc2, 0xd2, 0xc3, 0xd3, 64),
        (0xe2, 0xf2, 0xe3, 0xf3, 128),
    ] {
        let ram = sm::apu_ram::ApuRam::new();
        ram.write(0x0201, 0xfe);
        ram.write(0x0202, 0xfd);
        ram.write(0x01fe, mask);
        for (opcode, expected, cycles, pc) in [
            (clear, 0, 4, 0x0202),
            (set, mask, 4, 0x0202),
            (bbs, mask, 7, 0x0200),
            (bbr, mask, 5, 0x0203),
        ] {
            let mut cpu = Spc::new();
            cpu.pc = 0x0200;
            cpu.p = true;
            cpu.c = true;
            ram.write(0x0200, opcode);
            let mut trace = Vec::new();
            assert_eq!(
                cpu.run_opcode(|op| {
                    trace.push(op);
                    match op {
                        Read(a) => ram.read(a),
                        Write(a, v) => {
                            ram.write(a, v);
                            0
                        }
                    }
                }),
                cycles
            );
            assert_eq!((cpu.pc, ram.read(0x01fe), cpu.c), (pc, expected, true));
            if opcode == bbs || opcode == bbr {
                assert_eq!(
                    trace,
                    [Read(0x0200), Read(0x0201), Read(0x01fe), Read(0x0202)]
                );
            }
        }
    }
}

// Movies cannot independently observe all encoded absolute-bit carry rules and memory write effects.
#[cfg_attr(test, test)]
fn spc_absolute_bits_have_literal_carry_and_write_results() {
    use sm::snes::spc::{BusOperation::*, Spc};
    for (opcode, carry, expected_c, expected_value, cycles) in [
        (0x0a, false, true, 8, 5),
        (0x2a, false, false, 8, 5),
        (0x4a, true, true, 8, 4),
        (0x6a, true, false, 8, 4),
        (0x8a, true, false, 8, 5),
        (0xaa, false, true, 8, 4),
        (0xca, false, false, 0, 6),
        (0xea, true, true, 0, 5),
    ] {
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.c = carry;
        let ram = sm::apu_ram::ApuRam::new();
        for (a, v) in [(0x0200, opcode), (0x0201, 7), (0x0202, 0x60), (7, 8)] {
            ram.write(a, v);
        }
        let mut trace = Vec::new();
        assert_eq!(
            cpu.run_opcode(|op| {
                trace.push(op);
                match op {
                    Read(a) => ram.read(a),
                    Write(a, v) => {
                        ram.write(a, v);
                        0
                    }
                }
            }),
            cycles
        );
        assert_eq!(
            (cpu.c, ram.read(7), cpu.pc),
            (expected_c, expected_value, 0x0203)
        );
        assert_eq!(
            &trace[..4],
            &[Read(0x0200), Read(0x0201), Read(0x0202), Read(7)]
        );
        if opcode == 0xca || opcode == 0xea {
            assert_eq!(&trace[4..], &[Write(7, 0)]);
        } else {
            assert_eq!(trace.len(), 4);
        }
    }
}

// Movies cannot inspect independent TCALL/vector/stack traces or wrapped native CALL/BRK/RETI state.
#[cfg_attr(test, test)]
fn spc_calls_and_stack_keep_literal_vector_order_and_wrap() {
    use sm::snes::spc::{BusOperation::*, Spc};
    let run = |cpu: &mut Spc, ram: &sm::apu_ram::ApuRam| {
        let mut trace = Vec::new();
        let cycles = cpu.run_opcode(|op| {
            trace.push(op);
            match op {
                Read(a) => ram.read(a),
                Write(a, v) => {
                    ram.write(a, v);
                    0
                }
            }
        });
        (cycles, trace)
    };
    for (opcode, vector) in [
        (0x01, 0xffde),
        (0x11, 0xffdc),
        (0x21, 0xffda),
        (0x31, 0xffd8),
        (0x41, 0xffd6),
        (0x51, 0xffd4),
        (0x61, 0xffd2),
        (0x71, 0xffd0),
        (0x81, 0xffce),
        (0x91, 0xffcc),
        (0xa1, 0xffca),
        (0xb1, 0xffc8),
        (0xc1, 0xffc6),
        (0xd1, 0xffc4),
        (0xe1, 0xffc2),
        (0xf1, 0xffc0),
    ] {
        let mut cpu = Spc::new();
        cpu.pc = 0x1234;
        cpu.sp = 0;
        let ram = sm::apu_ram::ApuRam::new();
        ram.write(0x1234, opcode);
        ram.write(vector, 0x78);
        ram.write(vector + 1, 0x56);
        let (cycles, trace) = run(&mut cpu, &ram);
        assert_eq!((cycles, cpu.pc, cpu.sp), (8, 0x5678, 0xfe));
        assert_eq!(
            trace,
            [
                Read(0x1234),
                Write(0x0100, 0x12),
                Write(0x01ff, 0x35),
                Read(vector),
                Read(vector + 1)
            ]
        );
    }
    let ram = sm::apu_ram::ApuRam::new();
    let mut cpu = Spc::new();
    cpu.pc = 0xfffe;
    cpu.sp = 0xff;
    for (a, v) in [(0xfffe, 0x3f), (0xffff, 0x56), (0, 0x34)] {
        ram.write(a, v);
    }
    let (cycles, trace) = run(&mut cpu, &ram);
    assert_eq!((cycles, cpu.pc, cpu.sp), (8, 0x3456, 0xfd));
    assert_eq!(
        trace,
        [
            Read(0xfffe),
            Read(0xffff),
            Read(0),
            Write(0x01ff, 0),
            Write(0x01fe, 1)
        ]
    );
}

// Movies cannot inspect independent BRK flag pushes or RETI flag/PC pull order.
#[cfg_attr(test, test)]
fn spc_brk_and_reti_keep_literal_flags_and_pull_order() {
    use sm::snes::spc::{BusOperation::*, Spc};
    let run = |cpu: &mut Spc, ram: &sm::apu_ram::ApuRam| {
        let mut trace = Vec::new();
        let cycles = cpu.run_opcode(|op| {
            trace.push(op);
            match op {
                Read(a) => ram.read(a),
                Write(a, v) => {
                    ram.write(a, v);
                    0
                }
            }
        });
        (cycles, trace)
    };
    let ram = sm::apu_ram::ApuRam::new();
    let mut cpu = Spc::new();
    cpu.pc = 0x0200;
    cpu.c = true;
    cpu.n = true;
    cpu.v = true;
    cpu.h = true;
    cpu.i = true;
    for (a, v) in [(0x0200, 0x0f), (0xffde, 0x66), (0xffdf, 0x77)] {
        ram.write(a, v);
    }
    let (cycles, trace) = run(&mut cpu, &ram);
    assert_eq!(
        (cycles, cpu.pc, cpu.sp, cpu.i, cpu.b),
        (8, 0x7766, 0xfd, false, true)
    );
    assert_eq!(
        trace,
        [
            Read(0x0200),
            Write(0x0100, 2),
            Write(0x01ff, 1),
            Write(0x01fe, 0xcd),
            Read(0xffde),
            Read(0xffdf)
        ]
    );
    let mut cpu = Spc::new();
    cpu.pc = 0x0200;
    cpu.sp = 0xff;
    for (a, v) in [
        (0x0200, 0x7f),
        (0x0100, 0x3d),
        (0x0101, 0x78),
        (0x0102, 0x56),
    ] {
        ram.write(a, v);
    }
    let (cycles, trace) = run(&mut cpu, &ram);
    assert_eq!((cycles, cpu.pc, cpu.sp), (6, 0x5678, 2));
    assert_eq!(
        [cpu.n, cpu.v, cpu.p, cpu.b, cpu.h, cpu.i, cpu.z, cpu.c],
        [false, false, true, true, true, true, false, true]
    );
    assert_eq!(
        trace,
        [Read(0x0200), Read(0x0100), Read(0x0101), Read(0x0102)]
    );
}

// Movies cannot independently inspect wrapped indirect addressing and store dummy-read bus order.
#[cfg_attr(test, test)]
fn spc_indirect_moves_and_indexed_jumps_keep_literal_bus_addresses() {
    use sm::snes::spc::{BusOperation::*, Spc};
    for (opcode, x, y, operand, high, target, cycles) in [
        (0xf4, 3, 0, 0xfe, 0, 0x0101, 4),
        (0xf5, 3, 0, 0xfe, 0xff, 1, 5),
        (0xf6, 0, 3, 0xfe, 0xff, 1, 5),
        (0xf7, 0, 3, 0xff, 0, 1, 6),
        (0xd7, 0, 3, 0xff, 0, 1, 7),
    ] {
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.a = 55;
        cpu.x = x;
        cpu.y = y;
        cpu.p = true;
        let ram = sm::apu_ram::ApuRam::new();
        for (a, v) in [
            (0x0200, opcode),
            (0x0201, operand),
            (0x0202, high),
            (0x01ff, 0xfe),
            (0x0100, 0xff),
            (target, 0x80),
        ] {
            ram.write(a, v);
        }
        let mut trace = Vec::new();
        assert_eq!(
            cpu.run_opcode(|op| {
                trace.push(op);
                match op {
                    Read(a) => ram.read(a),
                    Write(a, v) => {
                        ram.write(a, v);
                        0
                    }
                }
            }),
            cycles
        );
        if opcode == 0xd7 {
            assert_eq!((cpu.a, ram.read(target)), (55, 55));
            assert_eq!(
                &trace[2..],
                &[Read(0x01ff), Read(0x0100), Read(1), Write(1, 55)]
            );
        } else {
            assert_eq!((cpu.a, cpu.n, cpu.z), (0x80, true, false));
            assert_eq!(trace.last(), Some(&Read(target)));
        }
        assert_eq!((cpu.x, cpu.y), (x, y));
    }
}

// Movies cannot independently inspect pointer-page wrapping and indexed-jump word reads.
#[cfg_attr(test, test)]
fn spc_indexed_pointer_and_jump_reads_wrap_before_bus_access() {
    use sm::snes::spc::{BusOperation::*, Spc};
    let mut cpu = Spc::new();
    cpu.pc = 0xffff;
    cpu.x = 1;
    cpu.p = true;
    let ram = sm::apu_ram::ApuRam::new();
    for (a, v) in [
        (0xffff, 0xe7),
        (0, 0xff),
        (0x0100, 0xfe),
        (0x0101, 0xff),
        (0xfffe, 0x80),
    ] {
        ram.write(a, v);
    }
    let mut trace = Vec::new();
    assert_eq!(
        cpu.run_opcode(|op| {
            trace.push(op);
            match op {
                Read(a) => ram.read(a),
                _ => panic!(),
            }
        }),
        6
    );
    assert_eq!((cpu.a, cpu.x, cpu.pc), (0x80, 1, 1));
    assert_eq!(
        trace,
        [
            Read(0xffff),
            Read(0),
            Read(0x0100),
            Read(0x0101),
            Read(0xfffe)
        ]
    );
    let mut cpu = Spc::new();
    cpu.pc = 0x0200;
    cpu.x = 2;
    for (a, v) in [
        (0x0200, 0x1f),
        (0x0201, 0xfe),
        (0x0202, 0xff),
        (0, 0x78),
        (1, 0x56),
    ] {
        ram.write(a, v);
    }
    trace.clear();
    assert_eq!(
        cpu.run_opcode(|op| {
            trace.push(op);
            match op {
                Read(a) => ram.read(a),
                _ => panic!(),
            }
        }),
        6
    );
    assert_eq!(cpu.pc, 0x5678);
    assert_eq!(
        trace,
        [Read(0x0200), Read(0x0201), Read(0x0202), Read(0), Read(1)]
    );
}

// Movies do not inject failed word high reads or writes to observe native flags and low-byte commit prefixes.
#[cfg_attr(test, test)]
fn spc_bus_failures_preserve_exact_native_register_and_memory_prefixes() {
    use sm::snes::spc::{BusOperation::*, Spc};
    use std::panic::{AssertUnwindSafe as unwind, catch_unwind as catch};
    for (opcode, fail, registers, psw, memory, last) in [
        (
            0xba,
            3,
            (0x0202, 0xff, 66, 0xff, 5),
            0x21,
            (0xff, 0xff),
            Read(0x0100),
        ),
        (
            0xda,
            4,
            (0x0202, 0xff, 66, 0xff, 5),
            0x21,
            (55, 0xff),
            Write(0x0100, 66),
        ),
        (
            0x3a,
            5,
            (0x0202, 0xff, 66, 0xff, 6),
            0x23,
            (0, 0xff),
            Write(0x0100, 0),
        ),
    ] {
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.a = 55;
        cpu.x = 0xff;
        cpu.y = 66;
        cpu.sp = if opcode == 0x2d { 0 } else { 0xff };
        cpu.p = true;
        cpu.c = true;
        if opcode == 0xfe {
            cpu.y = 0;
        }
        let ram = sm::apu_ram::ApuRam::new();
        for (a, v) in [
            (0x0200, opcode),
            (0x0201, 0xff),
            (0x01ff, 0xff),
            (0x0100, 0xff),
        ] {
            ram.write(a, v);
        }
        let mut trace = Vec::new();
        assert!(
            catch(unwind(|| cpu.run_opcode(|op| {
                trace.push(op);
                if trace.len() == fail + 1 {
                    panic!("bus failure");
                }
                match op {
                    Read(a) => ram.read(a),
                    Write(a, v) => {
                        ram.write(a, v);
                        0
                    }
                }
            })))
            .is_err()
        );
        assert_eq!(trace.len(), fail + 1);
        assert_eq!(trace.last(), Some(&last));
        assert_eq!(cpu.a, 55);
        assert_eq!((cpu.pc, cpu.x, cpu.y, cpu.sp, cpu.cyclesUsed), registers);
        assert_eq!(
            [cpu.n, cpu.v, cpu.p, cpu.b, cpu.h, cpu.i, cpu.z, cpu.c],
            [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1].map(|b| psw & b != 0)
        );
        assert_eq!((ram.read(0x01ff), ram.read(0x0100)), memory);
    }
}

// Movies do not inject failing stack, postincrement, or DBNZ bus callbacks to observe native committed prefixes.
#[cfg_attr(test, test)]
fn spc_stack_index_and_dbnz_bus_failures_preserve_native_prefixes() {
    use sm::snes::spc::{BusOperation::*, Spc};
    use std::panic::{AssertUnwindSafe as unwind, catch_unwind as catch};
    for (opcode, fail, registers, psw, memory, last) in [
        (
            0x2d,
            1,
            (0x0201, 0xff, 66, 0, 4),
            0x21,
            (0xff, 0xff),
            Write(0x0100, 55),
        ),
        (
            0xae,
            1,
            (0x0201, 0xff, 66, 0, 4),
            0x21,
            (0xff, 0xff),
            Read(0x0100),
        ),
        (
            0xaf,
            1,
            (0x0201, 0, 66, 0xff, 4),
            0x21,
            (0xff, 0xff),
            Write(0x01ff, 55),
        ),
        (
            0xfe,
            1,
            (0x0202, 0xff, 0xff, 0xff, 4),
            0x21,
            (0xff, 0xff),
            Read(0x0201),
        ),
    ] {
        let mut cpu = Spc::new();
        cpu.pc = 0x0200;
        cpu.a = 55;
        cpu.x = 0xff;
        cpu.y = 66;
        cpu.sp = if opcode == 0x2d { 0 } else { 0xff };
        cpu.p = true;
        cpu.c = true;
        if opcode == 0xfe {
            cpu.y = 0;
        }
        let ram = sm::apu_ram::ApuRam::new();
        for (a, v) in [
            (0x0200, opcode),
            (0x0201, 0xff),
            (0x01ff, 0xff),
            (0x0100, 0xff),
        ] {
            ram.write(a, v);
        }
        let mut trace = Vec::new();
        assert!(
            catch(unwind(|| cpu.run_opcode(|op| {
                trace.push(op);
                if trace.len() == fail + 1 {
                    panic!("bus failure");
                }
                match op {
                    Read(a) => ram.read(a),
                    Write(a, v) => {
                        ram.write(a, v);
                        0
                    }
                }
            })))
            .is_err()
        );
        assert_eq!(trace.len(), fail + 1);
        assert_eq!(trace.last(), Some(&last));
        assert_eq!(cpu.a, 55);
        assert_eq!((cpu.pc, cpu.x, cpu.y, cpu.sp, cpu.cyclesUsed), registers);
        assert_eq!(
            [cpu.n, cpu.v, cpu.p, cpu.b, cpu.h, cpu.i, cpu.z, cpu.c],
            [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1].map(|b| psw & b != 0)
        );
        assert_eq!((ram.read(0x01ff), ram.read(0x0100)), memory);
    }
}

// Movies cannot observe missing device PCM, zero-length callbacks, or failed preparation without inventing output bytes.
#[cfg_attr(test, test)]
fn sample_delivery_lifecycle_preserves_missing_pcm_zero_length_and_prepare_failures() {
    use sm::SampleDelivery;
    use std::panic::{AssertUnwindSafe as unwind, catch_unwind as catch};
    let mut delivery = SampleDelivery::new();
    assert!(delivery.deliver_to(0, 255, |_| panic!(), |_, _| panic!()));
    assert!(
        catch(unwind(|| delivery.deliver_to(
            1,
            128,
            |_| panic!(),
            |_, _| panic!()
        )))
        .is_err()
    );
    delivery.prepare(1);
    assert!(!delivery.deliver_to(
        1,
        128,
        |_| false,
        |_, _| panic!("missing PCM writes nothing")
    ));
    let mut writes = Vec::new();
    assert!(!delivery.deliver_to(
        3,
        0,
        |_| panic!("same missing block"),
        |index, value| {
            assert!(index < 3);
            writes.push((index, value));
        }
    ));
    assert_eq!(writes, [(0, 0), (1, 0), (2, 0)]);
    writes.clear();
    assert!(delivery.deliver_to(
        1,
        128,
        |block| {
            assert_eq!(block.len(), 1);
            block[0] = [
                i16::from_ne_bytes([0x11, 0x22]),
                i16::from_ne_bytes([0x33, 0x44]),
            ];
            true
        },
        |index, value| writes.push((index, value))
    ));
    assert_eq!(writes, [(0, 0x11)]);
    for frames in [0, usize::MAX / 4 + 1] {
        assert!(catch(unwind(|| delivery.prepare(frames))).is_err());
    }
    let mut tail = [0; 3];
    assert!(delivery.deliver(&mut tail, 128, |_| panic!(
        "failed prepare keeps cursor and PCM"
    )));
    assert_eq!(tail, [0x22, 0x33, 0x44]);
    delivery.release();
    assert!(delivery.deliver_to(0, 1, |_| panic!(), |_, _| panic!()));
    assert!(
        catch(unwind(|| delivery.deliver_to(
            1,
            0,
            |_| panic!(),
            |_, _| panic!()
        )))
        .is_err()
    );
    delivery.prepare(1);
    assert!(!delivery.deliver_to(
        1,
        128,
        |_| false,
        |_, _| panic!("release discards prior PCM")
    ));
}

// Movies cannot independently supply native byte patterns or observe exact refill cadence across odd callback lengths.
#[cfg_attr(test, test)]
fn sample_delivery_byte_cursor_and_skipped_refills_keep_literal_pcm_and_cadence() {
    use sm::SampleDelivery;
    let mut delivery = SampleDelivery::new();
    delivery.prepare(2);
    let refills = std::cell::Cell::new(0);
    let mut refill = |block: &mut [[i16; 2]]| {
        assert_eq!(block.len(), 2);
        let call = refills.get() + 1;
        refills.set(call);
        let first = match call {
            1 => 0x11,
            2 | 4 => return false,
            3 => 0x91,
            5 => 0xa1,
            _ => panic!("extra refill"),
        };
        for (index, pair) in block.iter_mut().enumerate() {
            let byte = first + index as u8 * 4;
            *pair = [
                i16::from_ne_bytes([byte, byte + 1]),
                i16::from_ne_bytes([byte + 2, byte + 3]),
            ];
        }
        true
    };
    let rows: [(usize, &[u8], usize); 4] = [
        (3, &[0x11, 0x12, 0x13], 1),
        (7, &[0x14, 0x15, 0x16, 0x17, 0x18, 0x11, 0x12], 2),
        (6, &[0x13, 0x14, 0x15, 0x16, 0x17, 0x18], 2),
        (
            17,
            &[
                0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96,
                0x97, 0x98, 0xa1,
            ],
            5,
        ),
    ];
    for (length, expected, calls) in rows {
        let mut writes = Vec::new();
        assert!(
            delivery.deliver_to(length, 128, &mut refill, |index, value| {
                assert!(index < length);
                writes.push((index, value));
            })
        );
        assert_eq!(
            writes,
            expected.iter().copied().enumerate().collect::<Vec<_>>()
        );
        assert_eq!(refills.get(), calls);
    }
}

// Movies cannot author independent signed PCM/volume boundaries or inspect SDL zero-fill before scaled odd-byte writes.
#[cfg_attr(test, test)]
fn sample_delivery_s16lsb_volumes_and_odd_cursor_have_literal_ordered_bytes() {
    use sm::SampleDelivery;
    let rows: [(u8, usize, &[u8]); 7] = [
        (0, 8, &[]),
        (1, 8, &[0xff, 0, 0, 0xff, 0, 0, 0, 0]),
        (64, 8, &[0xff, 0x3f, 0, 0xc0, 1, 0, 0xff, 0xff]),
        (127, 8, &[0xff, 0x7e, 0, 0x81, 2, 0, 0xfe, 0xff]),
        (128, 0, &[0xff, 0x7f, 0, 0x80, 3, 0, 0xfd, 0xff]),
        (129, 8, &[0xfe, 0x80, 0, 0x7f, 3, 0, 0xfd, 0xff]),
        (255, 8, &[0xfe, 0xfe, 0, 1, 5, 0, 0xfb, 0xff]),
    ];
    for (volume, zeroes, expected) in rows {
        let mut delivery = SampleDelivery::new();
        delivery.prepare(2);
        let mut writes = Vec::new();
        assert!(delivery.deliver_to(
            8,
            volume,
            |block| {
                block.copy_from_slice(&[
                    [
                        i16::from_ne_bytes([0xff, 0x7f]),
                        i16::from_ne_bytes([0, 0x80]),
                    ],
                    [i16::from_ne_bytes([3, 0]), i16::from_ne_bytes([0xfd, 0xff])],
                ]);
                true
            },
            |index, value| {
                assert!(index < 8);
                writes.push((index, value));
            }
        ));
        let expected_writes: Vec<_> = (0..zeroes)
            .map(|index| (index, 0))
            .chain(expected.iter().copied().enumerate())
            .collect();
        assert_eq!(writes, expected_writes);
    }
    let mut delivery = SampleDelivery::new();
    delivery.prepare(1);
    let mut first = [0];
    assert!(delivery.deliver(&mut first, 128, |block| {
        block[0] = [i16::from_ne_bytes([1, 2]), i16::from_ne_bytes([3, 4])];
        true
    }));
    assert_eq!(first, [1]);
    let mut writes = Vec::new();
    assert!(delivery.deliver_to(
        3,
        64,
        |_| panic!("odd cursor stays in block"),
        |index, value| {
            assert!(index < 3);
            writes.push((index, value));
        }
    ));
    assert_eq!(writes, [(0, 0), (1, 0), (2, 0), (0, 0x81), (1, 1)]);
}

// Movies cannot fail byte writers or refills midway to establish native retained-PCM and committed-chunk prefixes.
#[cfg_attr(test, test)]
fn sample_delivery_writer_and_refill_failures_keep_completed_chunk_prefixes() {
    use sm::SampleDelivery;
    use std::panic::{AssertUnwindSafe as unwind, catch_unwind as catch};
    let rows: [(u8, usize, &[(usize, u8)]); 2] = [
        (128, 3, &[(0, 1), (1, 2), (2, 3)]),
        (64, 6, &[(0, 0), (1, 0), (2, 0), (3, 0), (0, 0), (1, 1)]),
    ];
    for (volume, failure, expected) in rows {
        let mut delivery = SampleDelivery::new();
        delivery.prepare(1);
        let mut writes = Vec::new();
        assert!(
            catch(unwind(|| delivery.deliver_to(
                4,
                volume,
                |block| {
                    block[0] = [i16::from_ne_bytes([1, 2]), i16::from_ne_bytes([3, 4])];
                    true
                },
                |index, value| {
                    assert!(index < 4);
                    writes.push((index, value));
                    if writes.len() == failure {
                        panic!("writer failed");
                    }
                }
            )))
            .is_err()
        );
        assert_eq!(writes, expected);
        let mut repeated = [0; 4];
        assert!(delivery.deliver(&mut repeated, 128, |_| panic!(
            "current chunk cursor did not commit"
        )));
        assert_eq!(repeated, [1, 2, 3, 4]);
    }
    let mut delivery = SampleDelivery::new();
    delivery.prepare(1);
    let mut calls = 0;
    let mut writes = Vec::new();
    assert!(
        catch(unwind(|| delivery.deliver_to(
            6,
            128,
            |block| {
                calls += 1;
                if calls == 1 {
                    block[0] = [
                        i16::from_ne_bytes([0x11, 0x22]),
                        i16::from_ne_bytes([0x33, 0x44]),
                    ];
                    return true;
                }
                block[0][0] = i16::from_ne_bytes([0x55, 0x66]);
                panic!("second refill failed after changing one sample");
            },
            |index, value| {
                assert!(index < 6);
                writes.push((index, value));
            }
        )))
        .is_err()
    );
    assert_eq!(calls, 2);
    assert_eq!(writes, [(0, 0x11), (1, 0x22), (2, 0x33), (3, 0x44)]);
    let mut retained = [0; 4];
    assert!(delivery.deliver(&mut retained, 128, |block| {
        calls += 1;
        assert_eq!(block[0][0].to_ne_bytes(), [0x55, 0x66]);
        assert_eq!(block[0][1].to_ne_bytes(), [0x33, 0x44]);
        false
    }));
    assert_eq!(calls, 3);
    assert_eq!(retained, [0x55, 0x66, 0x33, 0x44]);
}
