"""Actor DLL mutation recipes, loaded by actor_dll_mutations.py."""

RECIPES = [
    {
        'fault': 'F1',
        'name': 'actor_multipart_initializes_extra_part',
        'path': 'crates/sm_game/src/enemy_population.rs',
        'before': 'remaining: SpawnSearch::remaining_parts(parts),',
        'after': 'remaining: SpawnSearch::remaining_parts(parts).wrapping_add(1),',
        'case': 7,
        'export': 'sm_verify_actor_population_animation_case',
        'required_assertion': 'exact actual-boundary order',
        'require_anchor_count': 1,
        'executed': False,
        'qualification': 'Selected unchanged baseline must return nonempty capture. Mutant must compile and reach this selected case, emit exactly the named assertion failure, and report failed capture. Empty capture, unnamed trace mismatch, crash, timeout, missing export and unrelated panic do not qualify.',
        'selected': 'population:7',
        'edits': [
            {
                'path': 'crates/sm_game/src/enemy_population.rs',
                'before': 'remaining: SpawnSearch::remaining_parts(parts),',
                'after': 'remaining: SpawnSearch::remaining_parts(parts).wrapping_add(1),',
            },
        ],
        'assertion_source': 'actor_population_animation_observer.rs',
        'assertion_message': 'assertion `left == right` failed: exact actual-boundary order',
        'expected_values': None,
    },
    {
        'fault': 'F3',
        'name': 'actor_animation_caches_selector',
        'path': 'sm/src/game/actors/animation.rs',
        'before': 'let current = ActorKey::from_legacy(h.session().actors.controls.selected);',
        'after': 'let current = key;',
        'case': 32,
        'export': 'sm_verify_actor_population_animation_case',
        'required_assertion': 'live selector for second opcode',
        'require_anchor_count': 1,
        'executed': False,
        'qualification': 'Selected unchanged baseline must return nonempty capture. Mutant must compile and reach this selected case, emit exactly the named assertion failure, and report failed capture. Empty capture, unnamed trace mismatch, crash, timeout, missing export and unrelated panic do not qualify.',
        'selected': 'population:32',
        'edits': [
            {
                'path': 'sm/src/game/actors/animation.rs',
                'before': 'let current = ActorKey::from_legacy(h.session().actors.controls.selected);',
                'after': 'let current = key;',
            },
        ],
        'assertion_source': 'actor_population_animation_cases.rs',
        'assertion_message': 'assertion `left == right` failed: live selector for second opcode',
        'expected_values': None,
    },
    {
        'fault': 'F4',
        'name': 'actor_respawn_wipes_retained_map',
        'path': 'sm/src/game/actors/population.rs',
        'before': """\
    place_entry(h, key, entry, birth);
    let palette = h.resolve_spawn(key, Native).palette;""",
        'after': """\
    place_entry(h, key, entry, birth);
    h.edit_actor(key, Native, |a| a.spritemap = 0);
    let palette = h.resolve_spawn(key, Native).palette;""",
        'case': 16,
        'export': 'sm_verify_actor_population_animation_case',
        'required_assertion': 'preserved respawn field',
        'require_anchor_count': 1,
        'executed': False,
        'qualification': 'Selected unchanged baseline must return nonempty capture. Mutant must compile and reach this selected case, emit exactly the named assertion failure, and report failed capture. Empty capture, unnamed trace mismatch, crash, timeout, missing export and unrelated panic do not qualify.',
        'selected': 'population:16',
        'edits': [
            {
                'path': 'sm/src/game/actors/population.rs',
                'before': """\
    place_entry(h, key, entry, birth);
    let palette = h.resolve_spawn(key, Native).palette;""",
                'after': """\
    place_entry(h, key, entry, birth);
    h.edit_actor(key, Native, |a| a.spritemap = 0);
    let palette = h.resolve_spawn(key, Native).palette;""",
            },
        ],
        'assertion_source': 'actor_population_animation_cases.rs',
        'assertion_message': 'assertion `left == right` failed: preserved respawn field',
        'expected_values': None,
    },
    {
        'fault': 'F2',
        'name': 'actor_birth_publication_after_initializer',
        'mutations': [
            {
                'path': 'sm/src/game/actors/population.rs',
                'before': 'let handle = place(h, key, entry, Creation::Dynamic, birth);',
                'after': 'let _ = place(h, key, entry, Creation::Dynamic, None);',
            },
            {
                'path': 'sm/src/game/actors/population.rs',
                'before': """\
        last_handle = handle;
        finish_initialization(h, key, &definition, Creation::Dynamic);""",
                'after': """\
        finish_initialization(h, key, &definition, Creation::Dynamic);
        let handle = birth.map(|birth| h.session_mut().actor_identity.commit_birth(birth));
        last_handle = handle;""",
            },
        ],
        'required_assertion': 'actor.initializer.first_callback_occurrence_committed',
        'export': 'sm_verify_room_runtime_guard',
        'baseline_result': True,
        'mutant_result': False,
        'executed': False,
        'fixture': 'actor-initializer-verification/actor_initializer_checks.rs',
        'registration': 'actor-initializer-verification/HANDOFF.md',
        'require_anchor_count': 1,
        'qualification': 'Baseline must pass first; compile error, unrelated panic, missing callback, empty output, crash or timeout never qualify. Require exact named assertion after actual initializer entry.',
        'selected': 'guard',
        'edits': [
            {
                'path': 'sm/src/game/actors/population.rs',
                'before': 'let handle = place(h, key, entry, Creation::Dynamic, birth);',
                'after': 'let _ = place(h, key, entry, Creation::Dynamic, None);',
            },
            {
                'path': 'sm/src/game/actors/population.rs',
                'before': """\
        last_handle = handle;
        finish_initialization(h, key, &definition, Creation::Dynamic);""",
                'after': """\
        finish_initialization(h, key, &definition, Creation::Dynamic);
        let handle = birth.map(|birth| h.session_mut().actor_identity.commit_birth(birth));
        last_handle = handle;""",
            },
        ],
        'assertion_source': 'actor_initializer_checks.rs',
        'assertion_message': 'actor.initializer.first_callback_occurrence_committed',
        'expected_values': None,
    },
    {
        'fault': 'F6',
        'name': 'actor_foreign_unwind_restores_stale_identity',
        'mutations': [
            {
                'path': 'sm/src/game/session_runtime.rs',
                'before': """\
        export_session_preflighted(state, mutation.game());
        let result = action(mutation.game());""",
                'after': """\
        export_session_preflighted(state, mutation.game());
        let stale_identity = mutation.game().actor_identity().clone();
        let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| action(mutation.game()))) {
            Ok(result) => result,
            Err(error) => {
                let _ = mutation.game().take_actor_identity();
                mutation.game().return_actor_identity(stale_identity);
                std::panic::resume_unwind(error);
            }
        };""",
            },
        ],
        'required_assertion': 'actor.initializer.after_unwind_nested_occurrence',
        'export': 'sm_verify_room_runtime_guard',
        'baseline_result': True,
        'mutant_result': False,
        'executed': False,
        'fixture': 'actor-initializer-verification/actor_initializer_checks.rs',
        'registration': 'actor-initializer-verification/HANDOFF.md',
        'require_anchor_count': 1,
        'qualification': 'Baseline must pass first; compile error, unrelated panic, missing callback, empty output, crash or timeout never qualify. Require exact named assertion after actual initializer entry. Deliberate nested panic alone is not a kill; require later semantic assertion.',
        'expected_caught_panic': 'actor initializer fixture deliberate nested dispatch unwind',
        'selected': 'guard',
        'edits': [
            {
                'path': 'sm/src/game/session_runtime.rs',
                'before': """\
        export_session_preflighted(state, mutation.game());
        let result = action(mutation.game());""",
                'after': """\
        export_session_preflighted(state, mutation.game());
        let stale_identity = mutation.game().actor_identity().clone();
        let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| action(mutation.game()))) {
            Ok(result) => result,
            Err(error) => {
                let _ = mutation.game().take_actor_identity();
                mutation.game().return_actor_identity(stale_identity);
                std::panic::resume_unwind(error);
            }
        };""",
            },
        ],
        'assertion_source': 'actor_initializer_checks.rs',
        'assertion_message': 'assertion `left == right` failed: actor.initializer.after_unwind_nested_occurrence',
        'expected_values': None,
    },
    {
        'name': 'F5_duplicate_actor_angle_owner',
        'fault': 'F5',
        'selected': 'guard',
        'required_assertion': 'actor.owner.angle_roundtrip.foreign_entry.angle slot=0',
        'edits': [
            {
                'path': 'crates/sm_game/src/actors/state.rs',
                'before': """\
pub struct Actors {
""",
                'after': """\
pub struct Actors {
    pub f5_duplicate_angle: u16,
""",
                'expected_matches': 1,
            },
            {
                'path': 'crates/sm_game/src/actors/state.rs',
                'before': """\
impl Clone for Actors {
    fn clone(&self) -> Self {
        Self {
""",
                'after': """\
impl Clone for Actors {
    fn clone(&self) -> Self {
        Self {
            f5_duplicate_angle: self.f5_duplicate_angle,
""",
                'expected_matches': 1,
            },
            {
                'path': 'sm/src/game/actors/codec.rs',
                'before': """\
pub(in crate::game) fn import_common(
    s: &mut Session,
    bytes: &impl Fn(usize) -> u8,
    byte_len: usize,
) {
    assert!(byte_len >= COMMON_END, "short actor common import");
""",
                'after': """\
pub(in crate::game) fn import_common(
    s: &mut Session,
    bytes: &impl Fn(usize) -> u8,
    byte_len: usize,
) {
    assert!(byte_len >= COMMON_END, "short actor common import");
    s.actors.f5_duplicate_angle = u16::from_le_bytes([bytes(0x97dc), bytes(0x97dd)]);
""",
                'expected_matches': 1,
            },
            {
                'path': 'sm/src/game/session_encoding.rs',
                'before': """\
    for (index, cell) in s.room.terrain.cells().iter().enumerate() {
        span(0x16402 + index, &[cell.behavior]);
    }
    s.presentation.encode_projection(&mut span);
""",
                'after': """\
    for (index, cell) in s.room.terrain.cells().iter().enumerate() {
        span(0x16402 + index, &[cell.behavior]);
    }
    span(0x97dc, &s.actors.f5_duplicate_angle.to_le_bytes());
    s.presentation.encode_projection(&mut span);
""",
                'expected_matches': 1,
            },
            {
                'path': 'sm/src/game/actors/snapshot.rs',
                'before': """\
    Actors,
    slots,
    saved,
""",
                'after': """\
    Actors,
    f5_duplicate_angle,
    slots,
    saved,
""",
            },
        ],
        'assertion_source': 'actor_owner_checks.rs',
        'assertion_message': 'assertion `left == right` failed: actor.owner.angle_roundtrip.foreign_entry.angle slot=0',
        'expected_values': [
            '4096',
            '12800',
        ],
    },
    {
        'name': 'actor_F7',
        'fault': 'F7',
        'selected': 'guard',
        'required_assertion': 'actor.contact.f7_stage_before_aliased_map',
        'edits': [
            {
                'path': 'sm/src/game/actors/contacts.rs',
                'before': """\
    let key = current(h);
    h.session_mut().room.enemy_processing_stage = 9;
    #[cfg(sm_verify_dll)]
    super::movement_verification::body_stage(h);
    if h.resolve_actor(key, AccessMode::Native).spritemap == 0 {""",
                'after': """\
    let key = current(h);
    let cached_map = h.resolve_actor(key, AccessMode::Native).spritemap;
    h.session_mut().room.enemy_processing_stage = 9;
    #[cfg(sm_verify_dll)]
    super::movement_verification::body_stage(h);
    if cached_map == 0 {""",
            },
        ],
        'assertion_source': 'actor_contact_checks.rs',
        'assertion_message': 'assertion `left == right` failed: actor.contact.f7_stage_before_aliased_map',
        'expected_values': [
            '77',
            '0',
        ],
    },
    {
        'name': 'actor_F8',
        'fault': 'F8',
        'selected': 'guard',
        'required_assertion': 'actor.contact.f8_same_pose_collision_ignores_appearance slot=1',
        'edits': [
            {
                'path': 'sm/src/game/actors/content_runtime.rs',
                'before': '                    ActorPartWord::X => return part.offset.x,',
                'after': """\
                    ActorPartWord::X => {
                        let slot = self.session().actors.controls.selected / 64;
                        let appearance = self.session().actor_identity.observe(slot, true, None).appearance;
                        let visual_x = appearance.map(|appearance| {
                            let visual = captured.world.actor_media().resolve_part(
                                appearance, position.pose, part.role, crate::game::session::Point::default()
                            ).expect("validated appearance for collision");
                            match visual.form {
                                super::scene::ActorVisualForm::Sprites(pieces) => pieces.first().map_or(0, |piece| piece.offset.x),
                                super::scene::ActorVisualForm::Tilemap(tilemap) => tilemap.offset.x,
                            }
                        }).unwrap_or(0);
                        return part.offset.x.wrapping_add(visual_x);
                    },""",
            },
        ],
        'assertion_source': 'actor_contact_checks.rs',
        'assertion_message': 'assertion `left == right` failed: actor.contact.f8_same_pose_collision_ignores_appearance slot=1',
        'expected_values': [
            '(0, 85)',
            '(16, 0)',
        ],
    },
    {
        'name': 'actor_flash_frozen_clear_embedded_palette',
        'fault': 'renderer',
        'selected': 'guard',
        'required_assertion': 'actor.media.actual_prepared_palette_mode mode=Flash',
        'edits': [
            {
                'path': 'sm/src/game/actors/media_runtime.rs',
                'before': """\
runtime.actor_draw_compatible(
                                bank, address, x, y, palette, tile, base, offscreen,
                            );""",
                'after': """\
runtime.actor_draw_prepared(
                                bank, address, x, y, palette, tile, base, offscreen,
                            );""",
            },
        ],
        'assertion_source': 'actor_media_runtime_checks.rs',
        'assertion_message': 'assertion `left == right` failed: actor.media.actual_prepared_palette_mode mode=Flash',
        'expected_values': None,
    },
    {
        'name': 'kraid_actual_writer_keeps_old_shot_gate',
        'fault': 'kraid_actual_writer_keeps_old_shot_gate',
        'selected': 'world:11',
        'required_assertion': 'actor.mutable.actual_writer_updates_pinned_definition',
        'assertion_source': 'actor_mutable_content_checks.rs',
        'assertion_message': 'assertion `left == right` failed: actor.mutable.actual_writer_updates_pinned_definition',
        'expected_values': [
            '36864',
            '32844',
        ],
        'edits': [
            {
                'path': 'sm/src/game/sm_a7.rs',
                'before': 'rom_record(0xa0, enemy_ptr, game).set_word(offset_of!(EnemyDef, shot_ai), (0xa7804ci32) as u16);',
                'after': 'rom_record(0xa0, enemy_ptr, game).set_word(offset_of!(EnemyDef, shot_ai), 0x9000u16);',
            },
        ],
    },
    {
        'name': 'extended_contact_ignores_live_shot_disable',
        'fault': 'extended_contact_ignores_live_shot_disable',
        'selected': 'world:11',
        'required_assertion': 'actor.mutable.live_shot_consumer hardware_after_writer',
        'assertion_source': 'actor_mutable_content_checks.rs',
        'assertion_message': 'assertion `left == right` failed: actor.mutable.live_shot_consumer hardware_after_writer',
        'expected_values': None,
        'edits': [
            {
                'path': 'sm/src/game/actors/contacts.rs',
                'before': 'ignored(h.definition_routine_offset(&definition, DefinitionRoutine::Shot))',
                'after': 'false',
            },
        ],
    },
]
