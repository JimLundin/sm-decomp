// Generated once from .github/scripts/test_summary.py before it was deleted; never edit.
// The logs are cargo test output from Quality runs; see test_summary.rs.

const LOG_41687356099: &str = r#"   Compiling sm_data v0.1.0 (/home/runner/work/sm-decomp/sm-decomp/crates/sm_data)
   Compiling sm_game v0.1.0 (/home/runner/work/sm-decomp/sm-decomp/crates/sm_game)
   Compiling sm_runtime v0.1.0 (/home/runner/work/sm-decomp/sm-decomp/crates/sm_runtime)
   Compiling sm_tools v0.1.0 (/home/runner/work/sm-decomp/sm-decomp/crates/sm_tools)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 24.40s
     Running unittests src/lib.rs (target/debug/deps/sm_data-04bafd2c9d559eeb)

running 6 tests
test enemy_speeds::tests32::reads_both_halves_and_rejects_outside ... ok
test enemy_speeds::tests32::rejects_other_content ... ok
test enemy_speeds::tests::offsets_cross_tables_and_straddle_words ... ok
test enemy_speeds::tests::reads_words_from_both_tables ... ok
test enemy_speeds::tests::rejects_other_content ... ok
test enemy_speeds::tests::rejects_reads_outside_the_block ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/record_layout.rs (target/debug/deps/record_layout-adc919b25ec22839)

running 15 tests
test area_select_palettes_round_trip_every_run ... ok
test column_list_records_round_trip ... ok
test column_major_records_round_trip ... ok
test ceres_ridley_getaway_round_trips ... ok
test crocomire_melt_tilemap_round_trips ... ok
test demo_scenes_round_trip_every_entry ... ok
test phantoon_flame_rain_skips_each_rows_fourth_word ... ok
test phantoon_timers_round_trip ... ok
test grapple_records_round_trip ... ok
test single_field_records_round_trip ... ok
test enemy_speed_tables_round_trip_every_word ... ok
test single_word_tables_round_trip ... ok
test table_words_and_grids_lay_out_row_major ... ok
test row_major_records_round_trip ... ok
test fixed_array_records_round_trip ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/rom_import.rs (target/debug/deps/rom_import-3975b9ea6431e6c6)

running 1 test
test records_import_from_the_original_rom ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/sm_game-9cf6ba0f52a0f963)

running 39 tests
test actors::identity::actor_identity_checks::compatibility_bytes_and_daff_preserve_occurrence_but_nominal_rebirth_does_not ... ok
test actors::identity::actor_identity_checks::canonical_transition_uses_final_bytes_and_clears_authored_lineage ... ok
test actors::identity::actor_identity_checks::discarded_future_handle_rejected_after_same_instance_restore_and_replay ... ok
test actors::identity::actor_identity_checks::exceptional_legacy_births_do_not_clamp_or_allocate_nominal_metadata ... ok
test actors::identity::actor_identity_checks::generation_and_order_overflow_leave_birth_unpublished ... ok
test actors::identity::actor_identity_checks::snapshot_binding_and_epoch_rejections_preserve_live_identity ... ok
test actors::identity::actor_identity_checks::epoch_overflow_preflight_precedes_any_canonical_image_mutation ... ok
test display::control::tests::bytes_match_the_original_constants ... ok
test display::hdma::tests::a_direct_table_becomes_one_write_per_line ... ok
test display::preparation::tests::mirrors_round_trip_every_byte ... ok
test enemy_projectiles::table::tests::slot_words_cover_the_encoded_table ... ok
test fx_encoding::control_lane_table::an_unowned_display_has_the_identity_control_graph ... ok
test actors::codec::word_paths::word_paths_match_the_byte_projection ... ok
test hdma_program::tests::table_words_follow_the_ram_names ... ok
test hud::tests::initial_tiles_match_the_original_table ... ok
test inventory::tests::the_view_reads_and_stores_the_owned_inventory ... ok
test menu_window_encoding::tests::every_window_lane_lies_in_the_lane_range ... ok
test probe::tests::completion_requires_every_probe_and_stays_at_first_complete_frame ... ok
test probe::tests::configuration_discards_setup_hits_and_empty_lists_cannot_complete ... ok
test actors::codec::word_paths::the_whole_word_table_answers_as_the_match_does ... ok
test projection::tests::decoded_views_are_their_session_fields ... ok
test cinematic_state::tests::the_session_projects_every_cinematic_word ... ok
test ram::mirror::match_the_named_ram_table ... ok
test save_file::tests::a_saved_file_loads_back_with_a_valid_checksum ... ok
test scene::hdma_sources::region_slots::region_slots_answer_as_the_region_map_does ... ok
test scene::irq::table_tests::the_gameplay_hud_split_is_two_interrupts ... ok
test session_cohort::tests::the_cinematic_table_loads_from_its_projected_words ... ok
test session_cohort::tests::the_menu_screens_load_from_their_projected_words ... ok
test session_cohort::tests::the_menu_state_and_inventory_load_from_their_projected_words ... ok
test session_cohort::tests::the_session_words_load_from_their_projected_words ... ok
test projection::tests::layouts_are_their_session_fields ... ok
test session_encoding::player_lane_tests::raw_transition_lanes_project_from_their_fields ... ok
test session_encoding::player_lane_tests::the_input_projection_answers_as_projected_byte_does ... ok
test session_encoding::player_lane_tests::player_lanes_project_from_their_fields ... ok
test sprite_output::tests::copy_arrow_matches_the_original_branches ... ok
test word_transfers::tests::named_words_and_transfers_work_over_any_host ... ok
test session_encoding::player_lane_tests::the_scalar_lane_table_answers_as_the_search_does ... ok
test fx_encoding::control_lane_table::the_table_answers_as_the_lane_search_does ... ok
test projection::tests::view_words_are_their_projected_words ... ok

test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s

     Running tests/collision_wrap.rs (target/debug/deps/collision_wrap-358cac6e9c5af14e)

running 1 test
test out_of_room_level_words_wrap_like_the_long_indexed_read ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/enemy_cores.rs (target/debug/deps/enemy_cores-66542adf492ddd94)

running 3 tests
test the_balloon_rises_as_it_inflates_and_sinks_as_it_deflates ... ok
test the_swoop_accelerates_to_its_cap_and_ends_at_the_arc_end ... ok
test yard_velocity_signs_keep_or_negate_the_speed ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/escape_typewriter.rs (target/debug/deps/escape_typewriter-e8ed5553f52838d2)

running 4 tests
test a_vram_address_with_bit_15_keeps_it_in_the_record ... ok
test zebes_setup_copies_the_text_colours_and_clears_the_state ... ok
test types_each_character_after_the_timer_runs_out ... ok
test zebes_strokes_use_library_three ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/grapple_swing.rs (target/debug/deps/grapple_swing-5264ced305f75d12)

running 7 tests
test angular_velocity_is_capped_at_480h_both_ways ... ok
test gravity_accelerates_by_quadrant ... ok
test extra_velocity_is_outside_the_cap ... ok
test length_changes_stop_at_8_and_3fh ... ok
test liquid_halves_the_angle_acceleration_only ... ok
test rising_deceleration_applies_only_against_the_angle ... ok
test straight_down_has_no_acceleration_and_settles_a_slow_swing ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/player_words.rs (target/debug/deps/player_words-930af1a70b6a54d8)

running 1 test
test every_player_word_reads_the_field_its_address_projects_to ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/state_layout.rs (target/debug/deps/state_layout-173e429dbf60ec86)

running 42 tests
test actor_speed_scratch_round_trips_every_word ... ok
test actor_spare_words_round_trip_every_word ... ok
test botwoon_death_words_round_trip ... ok
test camera_round_trips_every_field ... ok
test crocomire_melt_round_trips_every_word ... ok
test crocomire_scroll_indirect_round_trips ... ok
test boss_ram_round_trips_every_word ... ok
test death_sequence_round_trips_every_word ... ok
test crocomire_scroll_round_trips_every_word ... ok
test flow_round_trips_every_field ... ok
test grapple_previous_start_round_trips_every_word ... ok
test enemy_projectile_table_round_trips_every_field ... ok
test inputs_read_every_observation_from_its_address ... ok
test inventory_round_trips_every_field ... ok
test legacy_objects_round_trip_every_byte ... ok
test legacy_samus_requests_round_trip_every_byte ... ok
test bg2_staging_round_trips_every_word ... ok
test map_screen_round_trips_every_word ... ok
test liquid_round_trips_every_field ... ok
test message_box_round_trips_every_word ... ok
test no_two_owners_claim_the_same_byte ... ok
test options_menu_round_trips_every_word ... ok
test pause_hooks_round_trip_every_word ... ok
test player_settings_round_trip_every_word ... ok
test player_spare_words_round_trip_every_word ... ok
test menu_state_round_trips_every_field ... ok
test projection_words_are_little_endian_byte_pairs ... ok
test power_bomb_round_trips_every_word ... ok
test hdma_objects_round_trip_every_field ... ok
test room_state_words_round_trip ... ok
test actor_extended_ram_round_trips_every_word ... ok
test room_view_round_trips_every_field ... ok
test room_scrolls_round_trip_every_word ... ok
test session_words_are_projected ... ok
test room_entry_scroll_round_trips_every_field ... ok
test save_selection_round_trips_every_word ... ok
test star_records_map_their_original_bytes ... ok
test wavy_samus_round_trips_every_word ... ok
test work_robot_palette_round_trips_every_word ... ok
test supplied_menu_window_round_trips_all_hidden_fields ... ok
test visual_fades_round_trip_every_field ... ok
test session_round_trips_random_ram ... ok

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 42.68s

     Running tests/tilemap_queue.rs (target/debug/deps/tilemap_queue-00791cdffa3b6305)

running 2 tests
test a_queued_record_is_seven_bytes_at_the_tail ... ok
test queued_records_drain_in_order ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/sm_runtime-65594793ef65a222)

running 15 tests
test rendering::memory::tests::a_typed_vram_upload_matches_its_port_writes ... ok
test rendering::memory::tests::cgram_pairs_bytes_and_masks_the_high_read ... ok
test rendering::memory::tests::oam_low_table_wraps_into_the_high_table ... ok
test rendering::memory::tests::typed_oam_and_cgram_uploads_match_their_port_writes ... ok
test rendering::memory::tests::vram_reads_return_the_prefetched_word ... ok
test rendering::memory::tests::obsel_sets_size_and_tile_bases ... ok
test rendering::memory::tests::vram_remap_rotates_the_low_bits ... ok
test rendering::registers::tests::colour_data_sets_only_selected_channels ... ok
test rendering::memory::tests::vram_writes_follow_the_increment_byte ... ok
test rendering::registers::tests::mode7_registers_share_their_own_latch ... ok
test rendering::registers::tests::mosaic_restarts_at_line_zero_until_the_next_frame ... ok
test rendering::registers::tests::screen_and_mode_registers ... ok
test rendering::registers::tests::scroll_pairs_share_one_latch ... ok
test rendering::registers::tests::setini_flags ... ok
test rendering::registers::tests::window_selects_decode_two_layers_each ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/sm_tools-35660b79e13f7d03)

running 15 tests
test free_run::tests::a_tick_past_the_reference_fails ... ok
test lockstep::tests::disagreement_counts_each_side ... ok
test lockstep::tests::only_projected_domains_are_compared ... ok
test observation::tests::a_changed_field_is_named_with_both_values ... ok
test lockstep::tests::the_first_divergence_keeps_its_tick_domain_and_last_equal_tick ... ok
test free_run::tests::each_stream_keeps_its_first_divergence_and_spread ... ok
test observation::tests::array_elements_and_list_entries_have_their_own_paths ... ok
test observation::tests::an_encoded_observation_decodes_to_itself ... ok
test observation::tests::signed_fields_display_as_signed ... ok
test observation::tests::tables_follow_the_declarations ... ok
test observation::tests::swapped_effect_lists_do_not_hash_alike ... ok
test trace::tests::a_stream_some_tick_lacks_has_no_running_digest ... ok
test observation::tests::equal_observations_hash_equal_and_have_no_differences ... ok
test trace::tests::one_changed_tick_changes_its_stream_and_the_combined_digest ... ok
test trace::tests::a_trace_reads_back_every_tick ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/sm_pipeline/main.rs (target/debug/deps/sm_pipeline-d05521b8b578d8c9)

running 11 tests
test json::tests::setting_a_key_keeps_its_place ... ok
test json::tests::writes_as_python_json_dumps ... ok
test corpus::tests::the_sample_rotates_through_each_pool ... ok
test pack::tests::sums_as_cpython_does ... ok
test pack::tests::packs_longest_first_and_keeps_prefix_rows_together ... ok
test route::tests::lightest_takes_the_first_of_equal_loads ... ok
test route::tests::rounds_as_python_does ... ok
test scenario::tests::names_match_the_python_patterns ... ok
test toml::tests::reads_the_subset_the_repository_uses ... ok
test toml::tests::rejects_what_it_does_not_read ... ok
scope rows needs known row ids; unknown: []
scope rows needs known row ids; unknown: ['no-such-row']
test equivalence::plans_as_plan_verify_py_does has been running for over 60 seconds
test equivalence::plans_as_plan_verify_py_does ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 80.62s

   Doc-tests sm_data

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests sm_game

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests sm_runtime

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests sm_tools

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

"#;

const LOG_41619370268: &str = r#"   Compiling sm_data v0.1.0 (/home/runner/work/sm-decomp/sm-decomp/crates/sm_data)
   Compiling sm_game v0.1.0 (/home/runner/work/sm-decomp/sm-decomp/crates/sm_game)
   Compiling sm_runtime v0.1.0 (/home/runner/work/sm-decomp/sm-decomp/crates/sm_runtime)
   Compiling sm_tools v0.1.0 (/home/runner/work/sm-decomp/sm-decomp/crates/sm_tools)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 24.94s
     Running unittests src/lib.rs (target/debug/deps/sm_data-04bafd2c9d559eeb)

running 6 tests
test enemy_speeds::tests32::reads_both_halves_and_rejects_outside ... ok
test enemy_speeds::tests32::rejects_other_content ... ok
test enemy_speeds::tests::offsets_cross_tables_and_straddle_words ... ok
test enemy_speeds::tests::reads_words_from_both_tables ... ok
test enemy_speeds::tests::rejects_other_content ... ok
test enemy_speeds::tests::rejects_reads_outside_the_block ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/record_layout.rs (target/debug/deps/record_layout-adc919b25ec22839)

running 15 tests
test area_select_palettes_round_trip_every_run ... ok
test column_list_records_round_trip ... ok
test column_major_records_round_trip ... ok
test ceres_ridley_getaway_round_trips ... ok
test demo_scenes_round_trip_every_entry ... ok
test crocomire_melt_tilemap_round_trips ... ok
test grapple_records_round_trip ... ok
test phantoon_flame_rain_skips_each_rows_fourth_word ... ok
test phantoon_timers_round_trip ... ok
test single_field_records_round_trip ... ok
test enemy_speed_tables_round_trip_every_word ... ok
test single_word_tables_round_trip ... ok
test table_words_and_grids_lay_out_row_major ... ok
test row_major_records_round_trip ... ok
test fixed_array_records_round_trip ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/rom_import.rs (target/debug/deps/rom_import-3975b9ea6431e6c6)

running 1 test
test records_import_from_the_original_rom ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/sm_game-9cf6ba0f52a0f963)

running 32 tests
test actors::identity::actor_identity_checks::compatibility_bytes_and_daff_preserve_occurrence_but_nominal_rebirth_does_not ... ok
test actors::identity::actor_identity_checks::canonical_transition_uses_final_bytes_and_clears_authored_lineage ... ok
test actors::identity::actor_identity_checks::discarded_future_handle_rejected_after_same_instance_restore_and_replay ... ok
test actors::identity::actor_identity_checks::exceptional_legacy_births_do_not_clamp_or_allocate_nominal_metadata ... ok
test actors::identity::actor_identity_checks::generation_and_order_overflow_leave_birth_unpublished ... ok
test actors::identity::actor_identity_checks::snapshot_binding_and_epoch_rejections_preserve_live_identity ... ok
test display::control::tests::bytes_match_the_original_constants ... ok
test display::hdma::tests::a_direct_table_becomes_one_write_per_line ... ok
test enemy_projectiles::table::tests::slot_words_cover_the_encoded_table ... ok
test display::preparation::tests::mirrors_round_trip_every_byte ... ok
test actors::identity::actor_identity_checks::epoch_overflow_preflight_precedes_any_canonical_image_mutation ... ok
test hud::tests::initial_tiles_match_the_original_table ... ok
test hdma_program::tests::table_words_follow_the_ram_names ... ok
test probe::tests::completion_requires_every_probe_and_stays_at_first_complete_frame ... ok
test probe::tests::configuration_discards_setup_hits_and_empty_lists_cannot_complete ... ok
test inventory::tests::the_view_reads_and_stores_the_owned_inventory ... ok
test projection::tests::decoded_views_are_their_session_fields ... ok
test cinematic_state::tests::the_session_projects_every_cinematic_word ... ok
test ram::mirror::match_the_named_ram_table ... ok
test save_file::tests::a_saved_file_loads_back_with_a_valid_checksum ... ok
test scene::irq::table_tests::the_gameplay_hud_split_is_two_interrupts ... ok
test session_cohort::tests::the_cinematic_table_loads_from_its_projected_words ... ok
test session_cohort::tests::the_menu_screens_load_from_their_projected_words ... ok
test session_cohort::tests::the_menu_state_and_inventory_load_from_their_projected_words ... ok
test session_cohort::tests::the_session_words_load_from_their_projected_words ... ok
test projection::tests::layouts_are_their_session_fields ... FAILED
test session_encoding::player_lane_tests::raw_transition_lanes_project_from_their_fields ... ok
test session_encoding::player_lane_tests::player_lanes_project_from_their_fields ... ok
test word_transfers::tests::named_words_and_transfers_work_over_any_host ... ok
test sprite_output::tests::copy_arrow_matches_the_original_branches ... ok
test fx_encoding::control_lane_table::the_table_answers_as_the_lane_search_does ... ok
test projection::tests::view_words_are_their_projected_words ... ok

failures:

---- projection::tests::layouts_are_their_session_fields stdout ----

thread 'projection::tests::layouts_are_their_session_fields' (2967) panicked at crates/sm_game/src/fx_encoding.rs:1403:54:
control publication: Incomplete
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    projection::tests::layouts_are_their_session_fields

test result: FAILED. 31 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s

error: test failed, to rerun pass `-p sm_game --lib`
     Running tests/collision_wrap.rs (target/debug/deps/collision_wrap-358cac6e9c5af14e)

running 1 test
test out_of_room_level_words_wrap_like_the_long_indexed_read ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/enemy_cores.rs (target/debug/deps/enemy_cores-66542adf492ddd94)

running 3 tests
test the_balloon_rises_as_it_inflates_and_sinks_as_it_deflates ... ok
test the_swoop_accelerates_to_its_cap_and_ends_at_the_arc_end ... ok
test yard_velocity_signs_keep_or_negate_the_speed ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/escape_typewriter.rs (target/debug/deps/escape_typewriter-e8ed5553f52838d2)

running 4 tests
test a_vram_address_with_bit_15_keeps_it_in_the_record ... ok
test types_each_character_after_the_timer_runs_out ... ok
test zebes_strokes_use_library_three ... ok
test zebes_setup_copies_the_text_colours_and_clears_the_state ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/grapple_swing.rs (target/debug/deps/grapple_swing-5264ced305f75d12)

running 7 tests
test angular_velocity_is_capped_at_480h_both_ways ... ok
test extra_velocity_is_outside_the_cap ... ok
test gravity_accelerates_by_quadrant ... ok
test length_changes_stop_at_8_and_3fh ... ok
test liquid_halves_the_angle_acceleration_only ... ok
test straight_down_has_no_acceleration_and_settles_a_slow_swing ... ok
test rising_deceleration_applies_only_against_the_angle ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/player_words.rs (target/debug/deps/player_words-930af1a70b6a54d8)

running 1 test
test every_player_word_reads_the_field_its_address_projects_to ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/state_layout.rs (target/debug/deps/state_layout-173e429dbf60ec86)

running 39 tests
test actor_speed_scratch_round_trips_every_word ... ok
test actor_spare_words_round_trip_every_word ... ok
test botwoon_death_words_round_trip ... ok
test camera_round_trips_every_field ... ok
test crocomire_melt_round_trips_every_word ... ok
test death_sequence_round_trips_every_word ... ok
test boss_ram_round_trips_every_word ... ok
test flow_round_trips_every_field ... ok
test enemy_projectile_table_round_trips_every_field ... ok
test inputs_read_every_observation_from_its_address ... ok
test inventory_round_trips_every_field ... ok
test legacy_objects_round_trip_every_byte ... ok
test legacy_samus_requests_round_trip_every_byte ... ok
test bg2_staging_round_trips_every_word ... ok
test liquid_round_trips_every_field ... ok
test map_screen_round_trips_every_word ... ok
test message_box_round_trips_every_word ... ok
test no_two_owners_claim_the_same_byte ... ok
test options_menu_round_trips_every_word ... ok
test pause_hooks_round_trip_every_word ... ok
test player_settings_round_trip_every_word ... ok
test player_spare_words_round_trip_every_word ... ok
test menu_state_round_trips_every_field ... ok
test projection_words_are_little_endian_byte_pairs ... ok
test hdma_objects_round_trip_every_field ... ok
test power_bomb_round_trips_every_word ... ok
test room_state_words_round_trip ... ok
test room_view_round_trips_every_field ... ok
test room_entry_scroll_round_trips_every_field ... ok
test room_scrolls_round_trip_every_word ... ok
test session_words_are_projected ... ok
test actor_extended_ram_round_trips_every_word ... ok
test star_records_map_their_original_bytes ... ok
test save_selection_round_trips_every_word ... ok
test wavy_samus_round_trips_every_word ... ok
test work_robot_palette_round_trips_every_word ... ok
test supplied_menu_window_round_trips_all_hidden_fields ... ok
test session_round_trips_random_ram ... ok
test visual_fades_round_trip_every_field ... ok

test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 54.58s

     Running tests/tilemap_queue.rs (target/debug/deps/tilemap_queue-00791cdffa3b6305)

running 2 tests
test a_queued_record_is_seven_bytes_at_the_tail ... ok
test queued_records_drain_in_order ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/sm_runtime-65594793ef65a222)

running 15 tests
test rendering::memory::tests::a_typed_vram_upload_matches_its_port_writes ... ok
test rendering::memory::tests::cgram_pairs_bytes_and_masks_the_high_read ... ok
test rendering::memory::tests::obsel_sets_size_and_tile_bases ... ok
test rendering::memory::tests::oam_low_table_wraps_into_the_high_table ... ok
test rendering::memory::tests::typed_oam_and_cgram_uploads_match_their_port_writes ... ok
test rendering::memory::tests::vram_reads_return_the_prefetched_word ... ok
test rendering::memory::tests::vram_remap_rotates_the_low_bits ... ok
test rendering::registers::tests::colour_data_sets_only_selected_channels ... ok
test rendering::registers::tests::mode7_registers_share_their_own_latch ... ok
test rendering::registers::tests::mosaic_restarts_at_line_zero_until_the_next_frame ... ok
test rendering::memory::tests::vram_writes_follow_the_increment_byte ... ok
test rendering::registers::tests::scroll_pairs_share_one_latch ... ok
test rendering::registers::tests::screen_and_mode_registers ... ok
test rendering::registers::tests::window_selects_decode_two_layers_each ... ok
test rendering::registers::tests::setini_flags ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/sm_tools-35660b79e13f7d03)

running 15 tests
test lockstep::tests::disagreement_counts_each_side ... ok
test free_run::tests::a_tick_past_the_reference_fails ... ok
test lockstep::tests::only_projected_domains_are_compared ... ok
test free_run::tests::each_stream_keeps_its_first_divergence_and_spread ... ok
test observation::tests::a_changed_field_is_named_with_both_values ... ok
test lockstep::tests::the_first_divergence_keeps_its_tick_domain_and_last_equal_tick ... ok
test observation::tests::an_encoded_observation_decodes_to_itself ... ok
test observation::tests::signed_fields_display_as_signed ... ok
test observation::tests::array_elements_and_list_entries_have_their_own_paths ... ok
test observation::tests::swapped_effect_lists_do_not_hash_alike ... ok
test observation::tests::tables_follow_the_declarations ... ok
test trace::tests::a_stream_some_tick_lacks_has_no_running_digest ... ok
test trace::tests::one_changed_tick_changes_its_stream_and_the_combined_digest ... ok
test observation::tests::equal_observations_hash_equal_and_have_no_differences ... ok
test trace::tests::a_trace_reads_back_every_tick ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests sm_data

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests sm_game

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests sm_runtime

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests sm_tools

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p sm_game --lib`
"#;

const LOG_41457358300: &str = r#"   Compiling sm_data v0.1.0 (/home/runner/work/sm-decomp/sm-decomp/crates/sm_data)
   Compiling sm_game v0.1.0 (/home/runner/work/sm-decomp/sm-decomp/crates/sm_game)
   Compiling sm_runtime v0.1.0 (/home/runner/work/sm-decomp/sm-decomp/crates/sm_runtime)
   Compiling sm_tools v0.1.0 (/home/runner/work/sm-decomp/sm-decomp/crates/sm_tools)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 24.14s
     Running unittests src/lib.rs (target/debug/deps/sm_data-04bafd2c9d559eeb)

running 6 tests
test enemy_speeds::tests32::reads_both_halves_and_rejects_outside ... ok
test enemy_speeds::tests32::rejects_other_content ... ok
test enemy_speeds::tests::offsets_cross_tables_and_straddle_words ... ok
test enemy_speeds::tests::rejects_other_content ... ok
test enemy_speeds::tests::reads_words_from_both_tables ... ok
test enemy_speeds::tests::rejects_reads_outside_the_block ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/record_layout.rs (target/debug/deps/record_layout-adc919b25ec22839)

running 15 tests
test area_select_palettes_round_trip_every_run ... ok
test column_list_records_round_trip ... ok
test column_major_records_round_trip ... ok
test ceres_ridley_getaway_round_trips ... ok
test demo_scenes_round_trip_every_entry ... ok
test crocomire_melt_tilemap_round_trips ... ok
test phantoon_flame_rain_skips_each_rows_fourth_word ... ok
test phantoon_timers_round_trip ... ok
test grapple_records_round_trip ... ok
test row_major_records_round_trip ... ok
test enemy_speed_tables_round_trip_every_word ... ok
test single_field_records_round_trip ... ok
test table_words_and_grids_lay_out_row_major ... ok
test single_word_tables_round_trip ... ok
test fixed_array_records_round_trip ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/rom_import.rs (target/debug/deps/rom_import-3975b9ea6431e6c6)

running 1 test
test records_import_from_the_original_rom ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/sm_game-9cf6ba0f52a0f963)

running 27 tests
test actors::identity::actor_identity_checks::compatibility_bytes_and_daff_preserve_occurrence_but_nominal_rebirth_does_not ... ok
test actors::identity::actor_identity_checks::canonical_transition_uses_final_bytes_and_clears_authored_lineage ... ok
test actors::identity::actor_identity_checks::discarded_future_handle_rejected_after_same_instance_restore_and_replay ... ok
test actors::identity::actor_identity_checks::exceptional_legacy_births_do_not_clamp_or_allocate_nominal_metadata ... ok
test actors::identity::actor_identity_checks::generation_and_order_overflow_leave_birth_unpublished ... ok
test actors::identity::actor_identity_checks::snapshot_binding_and_epoch_rejections_preserve_live_identity ... ok
test actors::identity::actor_identity_checks::epoch_overflow_preflight_precedes_any_canonical_image_mutation ... ok
test display::control::tests::bytes_match_the_original_constants ... ok
test enemy_projectiles::table::tests::slot_words_cover_the_encoded_table ... ok
test hdma_program::tests::table_words_follow_the_ram_names ... ok
test display::hdma::tests::a_direct_table_becomes_one_write_per_line ... ok
test hud::tests::initial_tiles_match_the_original_table ... ok
test probe::tests::completion_requires_every_probe_and_stays_at_first_complete_frame ... ok
test probe::tests::configuration_discards_setup_hits_and_empty_lists_cannot_complete ... ok
test ram::mirror::match_the_named_ram_table ... ok
test scene::irq::table_tests::the_gameplay_hud_split_is_two_interrupts ... ok
test save_file::tests::a_saved_file_loads_back_with_a_valid_checksum ... ok
test inventory::tests::the_view_reads_and_stores_the_owned_inventory ... ok
test session_cohort::tests::the_menu_screens_load_from_their_projected_words ... ok
test session_cohort::tests::the_menu_state_and_inventory_load_from_their_projected_words ... ok
test session_cohort::tests::the_session_words_load_from_their_projected_words ... ok
test session_encoding::player_lane_tests::raw_transition_lanes_project_from_their_fields ... ok
test cinematic_state::tests::the_session_projects_every_cinematic_word ... ok
test word_transfers::tests::named_words_and_transfers_work_over_any_host ... ok
test session_cohort::tests::the_cinematic_table_loads_from_its_projected_words ... ok
test session_encoding::player_lane_tests::player_lanes_project_from_their_fields ... ok
test sprite_output::tests::copy_arrow_matches_the_original_branches ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/collision_wrap.rs (target/debug/deps/collision_wrap-358cac6e9c5af14e)

running 1 test
test out_of_room_level_words_wrap_like_the_long_indexed_read ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/enemy_cores.rs (target/debug/deps/enemy_cores-66542adf492ddd94)

running 3 tests
test the_balloon_rises_as_it_inflates_and_sinks_as_it_deflates ... ok
test the_swoop_accelerates_to_its_cap_and_ends_at_the_arc_end ... ok
test yard_velocity_signs_keep_or_negate_the_speed ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/escape_typewriter.rs (target/debug/deps/escape_typewriter-e8ed5553f52838d2)

running 4 tests
test a_vram_address_with_bit_15_keeps_it_in_the_record ... ok
test zebes_setup_copies_the_text_colours_and_clears_the_state ... ok
test types_each_character_after_the_timer_runs_out ... ok
test zebes_strokes_use_library_three ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/grapple_swing.rs (target/debug/deps/grapple_swing-5264ced305f75d12)

running 7 tests
test angular_velocity_is_capped_at_480h_both_ways ... ok
test extra_velocity_is_outside_the_cap ... ok
test gravity_accelerates_by_quadrant ... ok
test length_changes_stop_at_8_and_3fh ... ok
test rising_deceleration_applies_only_against_the_angle ... ok
test straight_down_has_no_acceleration_and_settles_a_slow_swing ... ok
test liquid_halves_the_angle_acceleration_only ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/player_words.rs (target/debug/deps/player_words-930af1a70b6a54d8)

running 1 test
test every_player_word_reads_the_field_its_address_projects_to ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/state_layout.rs (target/debug/deps/state_layout-173e429dbf60ec86)

running 29 tests
test actor_speed_scratch_round_trips_every_word ... FAILED
test boss_ram_round_trips_every_word ... FAILED
test bg2_staging_round_trips_every_word ... FAILED
test actor_extended_ram_round_trips_every_word ... FAILED
test camera_round_trips_every_field ... FAILED
test botwoon_death_words_round_trip ... FAILED
test death_sequence_round_trips_every_word ... FAILED
test crocomire_melt_round_trips_every_word ... FAILED
test inputs_read_every_observation_from_its_address ... ok
test flow_round_trips_every_field ... FAILED
test enemy_projectile_table_round_trips_every_field ... FAILED
test hdma_objects_round_trip_every_field ... FAILED
test inventory_round_trips_every_field ... FAILED
test map_screen_round_trips_every_word ... FAILED
test liquid_round_trips_every_field ... FAILED
test menu_state_round_trips_every_field ... FAILED
test message_box_round_trips_every_word ... FAILED
test projection_words_are_little_endian_byte_pairs ... ok
test options_menu_round_trips_every_word ... FAILED
test power_bomb_round_trips_every_word ... FAILED
test room_entry_scroll_round_trips_every_field ... FAILED
test room_state_words_round_trip ... FAILED
test no_two_owners_claim_the_same_byte ... FAILED
test session_words_are_projected ... ok
test supplied_menu_window_round_trips_all_hidden_fields ... FAILED
test room_view_round_trips_every_field ... FAILED
test save_selection_round_trips_every_word ... FAILED
test visual_fades_round_trip_every_field ... FAILED
test session_round_trips_random_ram ... ok

failures:

---- actor_speed_scratch_round_trips_every_word stdout ----

thread 'actor_speed_scratch_round_trips_every_word' (3182) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- boss_ram_round_trips_every_word stdout ----

thread 'boss_ram_round_trips_every_word' (3184) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- bg2_staging_round_trips_every_word stdout ----

thread 'bg2_staging_round_trips_every_word' (3183) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- actor_extended_ram_round_trips_every_word stdout ----

thread 'actor_extended_ram_round_trips_every_word' (3181) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- camera_round_trips_every_field stdout ----

thread 'camera_round_trips_every_field' (3186) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- botwoon_death_words_round_trip stdout ----

thread 'botwoon_death_words_round_trip' (3185) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- death_sequence_round_trips_every_word stdout ----

thread 'death_sequence_round_trips_every_word' (3188) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- crocomire_melt_round_trips_every_word stdout ----

thread 'crocomire_melt_round_trips_every_word' (3187) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- flow_round_trips_every_field stdout ----

thread 'flow_round_trips_every_field' (3190) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- enemy_projectile_table_round_trips_every_field stdout ----

thread 'enemy_projectile_table_round_trips_every_field' (3189) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- hdma_objects_round_trip_every_field stdout ----

thread 'hdma_objects_round_trip_every_field' (3191) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- inventory_round_trips_every_field stdout ----

thread 'inventory_round_trips_every_field' (3193) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- map_screen_round_trips_every_word stdout ----

thread 'map_screen_round_trips_every_word' (3195) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- liquid_round_trips_every_field stdout ----

thread 'liquid_round_trips_every_field' (3194) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- menu_state_round_trips_every_field stdout ----

thread 'menu_state_round_trips_every_field' (3196) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- message_box_round_trips_every_word stdout ----

thread 'message_box_round_trips_every_word' (3197) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- options_menu_round_trips_every_word stdout ----

thread 'options_menu_round_trips_every_word' (3199) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- power_bomb_round_trips_every_word stdout ----

thread 'power_bomb_round_trips_every_word' (3200) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- room_entry_scroll_round_trips_every_field stdout ----

thread 'room_entry_scroll_round_trips_every_field' (3202) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- room_state_words_round_trip stdout ----

thread 'room_state_words_round_trip' (3203) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- no_two_owners_claim_the_same_byte stdout ----

thread 'no_two_owners_claim_the_same_byte' (3198) panicked at crates/sm_game/tests/state_layout.rs:1253:5:
bytes encoded more than once: ["0x776 x2", "0x94a x2", "0x1778 x2", "0x1779 x2", "0x177a x2", "0x177b x2", "0x177c x2", "0x177d x2", "0x177e x2", "0x177f x2", "0x1780 x2", "0x1781 x2", "0x1782 x2", "0x1783 x2"]

---- supplied_menu_window_round_trips_all_hidden_fields stdout ----

thread 'supplied_menu_window_round_trips_all_hidden_fields' (3208) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- room_view_round_trips_every_field stdout ----

thread 'room_view_round_trips_every_field' (3204) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- save_selection_round_trips_every_word stdout ----

thread 'save_selection_round_trips_every_word' (3205) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete

---- visual_fades_round_trip_every_field stdout ----

thread 'visual_fades_round_trip_every_field' (3209) panicked at crates/sm_game/src/fx_encoding.rs:1359:35:
control owner: Incomplete


failures:
    actor_extended_ram_round_trips_every_word
    actor_speed_scratch_round_trips_every_word
    bg2_staging_round_trips_every_word
    boss_ram_round_trips_every_word
    botwoon_death_words_round_trip
    camera_round_trips_every_field
    crocomire_melt_round_trips_every_word
    death_sequence_round_trips_every_word
    enemy_projectile_table_round_trips_every_field
    flow_round_trips_every_field
    hdma_objects_round_trip_every_field
    inventory_round_trips_every_field
    liquid_round_trips_every_field
    map_screen_round_trips_every_word
    menu_state_round_trips_every_field
    message_box_round_trips_every_word
    no_two_owners_claim_the_same_byte
    options_menu_round_trips_every_word
    power_bomb_round_trips_every_word
    room_entry_scroll_round_trips_every_field
    room_state_words_round_trip
    room_view_round_trips_every_field
    save_selection_round_trips_every_word
    supplied_menu_window_round_trips_all_hidden_fields
    visual_fades_round_trip_every_field

test result: FAILED. 4 passed; 25 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.31s

error: test failed, to rerun pass `-p sm_game --test state_layout`
     Running tests/tilemap_queue.rs (target/debug/deps/tilemap_queue-00791cdffa3b6305)

running 2 tests
test a_queued_record_is_seven_bytes_at_the_tail ... ok
test queued_records_drain_in_order ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/sm_runtime-65594793ef65a222)

running 15 tests
test rendering::memory::tests::a_typed_vram_upload_matches_its_port_writes ... ok
test rendering::memory::tests::cgram_pairs_bytes_and_masks_the_high_read ... ok
test rendering::memory::tests::oam_low_table_wraps_into_the_high_table ... ok
test rendering::memory::tests::vram_remap_rotates_the_low_bits ... ok
test rendering::memory::tests::typed_oam_and_cgram_uploads_match_their_port_writes ... ok
test rendering::memory::tests::vram_reads_return_the_prefetched_word ... ok
test rendering::memory::tests::obsel_sets_size_and_tile_bases ... ok
test rendering::registers::tests::colour_data_sets_only_selected_channels ... ok
test rendering::memory::tests::vram_writes_follow_the_increment_byte ... ok
test rendering::registers::tests::mode7_registers_share_their_own_latch ... ok
test rendering::registers::tests::mosaic_restarts_at_line_zero_until_the_next_frame ... ok
test rendering::registers::tests::scroll_pairs_share_one_latch ... ok
test rendering::registers::tests::setini_flags ... ok
test rendering::registers::tests::window_selects_decode_two_layers_each ... ok
test rendering::registers::tests::screen_and_mode_registers ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/sm_tools-35660b79e13f7d03)

running 15 tests
test free_run::tests::a_tick_past_the_reference_fails ... ok
test lockstep::tests::disagreement_counts_each_side ... ok
test lockstep::tests::only_projected_domains_are_compared ... ok
test free_run::tests::each_stream_keeps_its_first_divergence_and_spread ... ok
test observation::tests::a_changed_field_is_named_with_both_values ... ok
test lockstep::tests::the_first_divergence_keeps_its_tick_domain_and_last_equal_tick ... ok
test observation::tests::an_encoded_observation_decodes_to_itself ... ok
test observation::tests::array_elements_and_list_entries_have_their_own_paths ... ok
test observation::tests::signed_fields_display_as_signed ... ok
test observation::tests::swapped_effect_lists_do_not_hash_alike ... ok
test observation::tests::tables_follow_the_declarations ... ok
test trace::tests::a_stream_some_tick_lacks_has_no_running_digest ... ok
test trace::tests::one_changed_tick_changes_its_stream_and_the_combined_digest ... ok
test trace::tests::a_trace_reads_back_every_tick ... ok
test observation::tests::equal_observations_hash_equal_and_have_no_differences ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests sm_data

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests sm_game

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests sm_runtime

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests sm_tools

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p sm_game --test state_layout`
"#;

const CASES: &[Golden] = &[
    Golden {
        label: "run-41687356099",
        log: LOG_41687356099,
        derive: Log::Unchanged,
        packages: &[("sm_data", &["record_layout.rs", "rom_import.rs"]), ("sm_game", &["collision_wrap.rs", "enemy_cores.rs", "escape_typewriter.rs", "grapple_swing.rs", "player_words.rs", "state_layout.rs", "tilemap_queue.rs"]), ("sm_runtime", &[]), ("sm_tools", &[])],
        report: r#"### Target package tests

| Test binary | Result | Passed | Failed | Ignored | Filtered |
| --- | --- | --- | --- | --- | --- |
| `sm_data (unit)` | ok | 6 | 0 | 0 | 0 |
| `tests/record_layout.rs` | ok | 15 | 0 | 0 | 0 |
| `tests/rom_import.rs` | ok | 1 | 0 | 0 | 0 |
| `sm_game (unit)` | ok | 39 | 0 | 0 | 0 |
| `tests/collision_wrap.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/enemy_cores.rs` | ok | 3 | 0 | 0 | 0 |
| `tests/escape_typewriter.rs` | ok | 4 | 0 | 0 | 0 |
| `tests/grapple_swing.rs` | ok | 7 | 0 | 0 | 0 |
| `tests/player_words.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/state_layout.rs` | ok | 42 | 0 | 0 | 0 |
| `tests/tilemap_queue.rs` | ok | 2 | 0 | 0 | 0 |
| `sm_runtime (unit)` | ok | 15 | 0 | 0 | 0 |
| `sm_tools (unit)` | ok | 15 | 0 | 0 | 0 |
| `sm_pipeline (unit)` | ok | 11 | 0 | 0 | 0 |
| `sm_data (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_game (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_runtime (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_tools (doc)` | ok | 0 | 0 | 0 | 0 |

"#,
        diagnostics: r#""#,
        fails: false,
    },
    Golden {
        label: "run-41619370268",
        log: LOG_41619370268,
        derive: Log::Unchanged,
        packages: &[("sm_data", &["record_layout.rs", "rom_import.rs"]), ("sm_game", &["collision_wrap.rs", "enemy_cores.rs", "escape_typewriter.rs", "grapple_swing.rs", "player_words.rs", "state_layout.rs", "tilemap_queue.rs"]), ("sm_runtime", &[]), ("sm_tools", &[])],
        report: r#"### Target package tests

| Test binary | Result | Passed | Failed | Ignored | Filtered |
| --- | --- | --- | --- | --- | --- |
| `sm_data (unit)` | ok | 6 | 0 | 0 | 0 |
| `tests/record_layout.rs` | ok | 15 | 0 | 0 | 0 |
| `tests/rom_import.rs` | ok | 1 | 0 | 0 | 0 |
| `sm_game (unit)` | **FAILED** | 31 | 1 | 0 | 0 |
| `tests/collision_wrap.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/enemy_cores.rs` | ok | 3 | 0 | 0 | 0 |
| `tests/escape_typewriter.rs` | ok | 4 | 0 | 0 | 0 |
| `tests/grapple_swing.rs` | ok | 7 | 0 | 0 | 0 |
| `tests/player_words.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/state_layout.rs` | ok | 39 | 0 | 0 | 0 |
| `tests/tilemap_queue.rs` | ok | 2 | 0 | 0 | 0 |
| `sm_runtime (unit)` | ok | 15 | 0 | 0 | 0 |
| `sm_tools (unit)` | ok | 15 | 0 | 0 | 0 |
| `sm_data (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_game (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_runtime (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_tools (doc)` | ok | 0 | 0 | 0 | 0 |

"#,
        diagnostics: r#"test binaries that failed or gave no result: sm_game (unit)
"#,
        fails: true,
    },
    Golden {
        label: "run-41457358300",
        log: LOG_41457358300,
        derive: Log::Unchanged,
        packages: &[("sm_data", &["record_layout.rs", "rom_import.rs"]), ("sm_game", &["collision_wrap.rs", "enemy_cores.rs", "escape_typewriter.rs", "grapple_swing.rs", "player_words.rs", "state_layout.rs", "tilemap_queue.rs"]), ("sm_runtime", &[]), ("sm_tools", &[])],
        report: r#"### Target package tests

| Test binary | Result | Passed | Failed | Ignored | Filtered |
| --- | --- | --- | --- | --- | --- |
| `sm_data (unit)` | ok | 6 | 0 | 0 | 0 |
| `tests/record_layout.rs` | ok | 15 | 0 | 0 | 0 |
| `tests/rom_import.rs` | ok | 1 | 0 | 0 | 0 |
| `sm_game (unit)` | ok | 27 | 0 | 0 | 0 |
| `tests/collision_wrap.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/enemy_cores.rs` | ok | 3 | 0 | 0 | 0 |
| `tests/escape_typewriter.rs` | ok | 4 | 0 | 0 | 0 |
| `tests/grapple_swing.rs` | ok | 7 | 0 | 0 | 0 |
| `tests/player_words.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/state_layout.rs` | **FAILED** | 4 | 25 | 0 | 0 |
| `tests/tilemap_queue.rs` | ok | 2 | 0 | 0 | 0 |
| `sm_runtime (unit)` | ok | 15 | 0 | 0 | 0 |
| `sm_tools (unit)` | ok | 15 | 0 | 0 | 0 |
| `sm_data (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_game (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_runtime (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_tools (doc)` | ok | 0 | 0 | 0 | 0 |

"#,
        diagnostics: r#"test binaries that failed or gave no result: tests/state_layout.rs
"#,
        fails: true,
    },
    Golden {
        label: "unbuilt-test-file",
        log: LOG_41687356099,
        derive: Log::Unchanged,
        packages: &[("sm_data", &["record_layout.rs", "rom_import.rs"]), ("sm_game", &["collision_wrap.rs", "enemy_cores.rs", "escape_typewriter.rs", "grapple_swing.rs", "player_words.rs", "state_layout.rs", "tilemap_queue.rs", "not_built.rs"]), ("sm_runtime", &[]), ("sm_tools", &[])],
        report: r#"### Target package tests

| Test binary | Result | Passed | Failed | Ignored | Filtered |
| --- | --- | --- | --- | --- | --- |
| `sm_data (unit)` | ok | 6 | 0 | 0 | 0 |
| `tests/record_layout.rs` | ok | 15 | 0 | 0 | 0 |
| `tests/rom_import.rs` | ok | 1 | 0 | 0 | 0 |
| `sm_game (unit)` | ok | 39 | 0 | 0 | 0 |
| `tests/collision_wrap.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/enemy_cores.rs` | ok | 3 | 0 | 0 | 0 |
| `tests/escape_typewriter.rs` | ok | 4 | 0 | 0 | 0 |
| `tests/grapple_swing.rs` | ok | 7 | 0 | 0 | 0 |
| `tests/player_words.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/state_layout.rs` | ok | 42 | 0 | 0 | 0 |
| `tests/tilemap_queue.rs` | ok | 2 | 0 | 0 | 0 |
| `sm_runtime (unit)` | ok | 15 | 0 | 0 | 0 |
| `sm_tools (unit)` | ok | 15 | 0 | 0 | 0 |
| `sm_pipeline (unit)` | ok | 11 | 0 | 0 | 0 |
| `sm_data (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_game (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_runtime (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_tools (doc)` | ok | 0 | 0 | 0 | 0 |
| `tests/not_built.rs` | **not run** | | | | |

"#,
        diagnostics: r#"expected test binaries that did not run: tests/not_built.rs
"#,
        fails: true,
    },
    Golden {
        label: "cut-off-binary",
        log: LOG_41687356099,
        derive: Log::Lines(139),
        packages: &[("sm_data", &["record_layout.rs", "rom_import.rs"]), ("sm_game", &["collision_wrap.rs", "enemy_cores.rs", "escape_typewriter.rs", "grapple_swing.rs", "player_words.rs", "state_layout.rs", "tilemap_queue.rs"]), ("sm_runtime", &[]), ("sm_tools", &[])],
        report: r#"### Target package tests

| Test binary | Result | Passed | Failed | Ignored | Filtered |
| --- | --- | --- | --- | --- | --- |
| `sm_data (unit)` | ok | 6 | 0 | 0 | 0 |
| `tests/record_layout.rs` | ok | 15 | 0 | 0 | 0 |
| `tests/rom_import.rs` | ok | 1 | 0 | 0 | 0 |
| `sm_game (unit)` | ok | 39 | 0 | 0 | 0 |
| `tests/collision_wrap.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/enemy_cores.rs` | ok | 3 | 0 | 0 | 0 |
| `tests/escape_typewriter.rs` | ok | 4 | 0 | 0 | 0 |
| `tests/grapple_swing.rs` | ok | 7 | 0 | 0 | 0 |
| `tests/player_words.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/state_layout.rs` | **no result** | | | | |
| `sm_data (doc)` | **not run** | | | | |
| `sm_game (doc)` | **not run** | | | | |
| `tests/tilemap_queue.rs` | **not run** | | | | |
| `sm_runtime (unit)` | **not run** | | | | |
| `sm_runtime (doc)` | **not run** | | | | |
| `sm_tools (unit)` | **not run** | | | | |
| `sm_tools (doc)` | **not run** | | | | |

"#,
        diagnostics: r#"expected test binaries that did not run: sm_data (doc), sm_game (doc), tests/tilemap_queue.rs, sm_runtime (unit), sm_runtime (doc), sm_tools (unit), sm_tools (doc)
test binaries that failed or gave no result: tests/state_layout.rs
"#,
        fails: true,
    },
    Golden {
        label: "windows-paths",
        log: LOG_41687356099,
        derive: Log::WindowsPaths,
        packages: &[("sm_data", &["record_layout.rs", "rom_import.rs"]), ("sm_game", &["collision_wrap.rs", "enemy_cores.rs", "escape_typewriter.rs", "grapple_swing.rs", "player_words.rs", "state_layout.rs", "tilemap_queue.rs"]), ("sm_runtime", &[]), ("sm_tools", &[])],
        report: r#"### Target package tests

| Test binary | Result | Passed | Failed | Ignored | Filtered |
| --- | --- | --- | --- | --- | --- |
| `sm_data (unit)` | ok | 6 | 0 | 0 | 0 |
| `tests/record_layout.rs` | ok | 15 | 0 | 0 | 0 |
| `tests/rom_import.rs` | ok | 1 | 0 | 0 | 0 |
| `sm_game (unit)` | ok | 39 | 0 | 0 | 0 |
| `tests/collision_wrap.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/enemy_cores.rs` | ok | 3 | 0 | 0 | 0 |
| `tests/escape_typewriter.rs` | ok | 4 | 0 | 0 | 0 |
| `tests/grapple_swing.rs` | ok | 7 | 0 | 0 | 0 |
| `tests/player_words.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/state_layout.rs` | ok | 42 | 0 | 0 | 0 |
| `tests/tilemap_queue.rs` | ok | 2 | 0 | 0 | 0 |
| `sm_runtime (unit)` | ok | 15 | 0 | 0 | 0 |
| `sm_tools (unit)` | ok | 15 | 0 | 0 | 0 |
| `sm_pipeline (unit)` | ok | 11 | 0 | 0 | 0 |
| `sm_data (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_game (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_runtime (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_tools (doc)` | ok | 0 | 0 | 0 | 0 |

"#,
        diagnostics: r#""#,
        fails: false,
    },
    Golden {
        label: "crlf",
        log: LOG_41687356099,
        derive: Log::Crlf,
        packages: &[("sm_data", &["record_layout.rs", "rom_import.rs"]), ("sm_game", &["collision_wrap.rs", "enemy_cores.rs", "escape_typewriter.rs", "grapple_swing.rs", "player_words.rs", "state_layout.rs", "tilemap_queue.rs"]), ("sm_runtime", &[]), ("sm_tools", &[])],
        report: r#"### Target package tests

| Test binary | Result | Passed | Failed | Ignored | Filtered |
| --- | --- | --- | --- | --- | --- |
| `sm_data (unit)` | ok | 6 | 0 | 0 | 0 |
| `tests/record_layout.rs` | ok | 15 | 0 | 0 | 0 |
| `tests/rom_import.rs` | ok | 1 | 0 | 0 | 0 |
| `sm_game (unit)` | ok | 39 | 0 | 0 | 0 |
| `tests/collision_wrap.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/enemy_cores.rs` | ok | 3 | 0 | 0 | 0 |
| `tests/escape_typewriter.rs` | ok | 4 | 0 | 0 | 0 |
| `tests/grapple_swing.rs` | ok | 7 | 0 | 0 | 0 |
| `tests/player_words.rs` | ok | 1 | 0 | 0 | 0 |
| `tests/state_layout.rs` | ok | 42 | 0 | 0 | 0 |
| `tests/tilemap_queue.rs` | ok | 2 | 0 | 0 | 0 |
| `sm_runtime (unit)` | ok | 15 | 0 | 0 | 0 |
| `sm_tools (unit)` | ok | 15 | 0 | 0 | 0 |
| `sm_pipeline (unit)` | ok | 11 | 0 | 0 | 0 |
| `sm_data (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_game (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_runtime (doc)` | ok | 0 | 0 | 0 | 0 |
| `sm_tools (doc)` | ok | 0 | 0 | 0 | 0 |

"#,
        diagnostics: r#""#,
        fails: false,
    },
];
