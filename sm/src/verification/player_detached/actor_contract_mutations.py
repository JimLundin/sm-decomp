"""Actor contract extension called from the existing detached mutation runner."""
import os
import re
import shutil

# name, fully-qualified selected Rust test, required assertion, exact production edits
ACTOR_CONTRACT_MUTATIONS = (
    ("actor_room_keeps_old_obj_receipts",
     "game::actors::palette_checks::unrelated_rooms_can_reuse_a_row_but_only_its_current_resource_is_resident",
     "actor.palette.actor_clear_revokes_obj_receipts",
     (("actors/media.rs", "self.rows[8..].fill(None);", "let _ = &self.rows[8..];"),)),
    ("actor_appearance_skips_residency_preflight",
     "game::actors::palette_checks::dynamic_appearance_rejects_missing_residency_and_conflicts_before_publication",
     "actor.palette.selection_requires_residency",
     (("actors/identity.rs",
       "world.actor_media().validate_appearance_residency(\n            appearance,\n            &self.palette_residency,\n            image,\n        )?;",
       "let _ = (world, image);"),)),
    ("actor_restore_reuses_live_epoch",
     "game::actors::identity::actor_identity_checks::discarded_future_handle_rejected_after_same_instance_restore_and_replay",
     "discarded_future_handle_rejected",
     (("actors/identity.rs", "self.live_epoch = epoch;", "self.live_epoch = epoch.saturating_sub(1);"),)),
    ("actor_projected_edit_discards_changes",
     "game::actors::codec_checks::actor_edits_publish_only_changed_fields_on_normal_return_and_unwind",
     "actor.contract.guard_unwind_changed_bytes",
     (("actors/access.rs", "            &self.edited,", "            &self.original,"),)),
    ("actor_daff_retires_occurrence",
     "game::actors::identity::actor_identity_checks::compatibility_bytes_and_daff_preserve_occurrence_but_nominal_rebirth_does_not",
     "actor.contract.daff_retains_occurrence",
     (("actors/identity.rs", "pub(super) fn convert_to_placeholder(&mut self, _slot: u16) {}",
       "pub(super) fn convert_to_placeholder(&mut self, slot: u16) { self.retire(slot); }"),)),
    ("actor_default_appearance_missing_from_media_revision",
     "game::actors::catalog_checks::default_appearance_changes_media_revision_only",
     "actor.contract.default_appearance_changes_media_revision",
     (("actors/media.rs", "(id, species.default_appearance).hash(&mut hash);", "id.hash(&mut hash);"),)),
)

def _build(source, work, name, run, edits=()):
    copied = work / name / "src"
    shutil.copytree(source, copied)
    for relative, before, after in edits:
        target = copied / "game" / relative
        original = target.read_text(encoding="utf-8")
        if original.count(before) != 1:
            raise RuntimeError(f"{name}: expected exactly one actor mutation anchor in {relative}")
        target.write_text(original.replace(before, after, 1), encoding="utf-8")
    main = copied / "verification" / "player_detached" / "main.rs"
    binary = work / name / ("actor-contracts.exe" if os.name == "nt" else "actor-contracts")
    # Mutants may leave a variable or import unused; the baseline stays strict.
    allow = ("-A", "unused") if edits else ()
    built = run(["rustc", "--edition=2024", "-O", "--test", str(main), "-o", str(binary)], 180, allow)
    if built.returncode != 0:
        raise RuntimeError(f"{name}: COMPILE ERROR; not a killed mutation\n{built.stderr}")
    return binary

def check_actor_contracts(source, work, run):
    """Called under existing GitHub-only main guard; preserves all previous mutants."""
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise RuntimeError("actor mutation execution follows the existing GitHub-only runner policy")
    baseline = _build(source, work, "actor-contract-baseline", run)
    all_tests = run([str(baseline), "--test-threads=1"], 30)
    summary = re.search(r"test result: ok\. (\d+) passed; 0 failed;", all_tests.stdout)
    if all_tests.returncode != 0 or summary is None or int(summary.group(1)) < 18:
        raise RuntimeError(f"actor contract baseline must execute at least18 tests\n{all_tests.stdout}\n{all_tests.stderr}")
    for name, selected, label, edits in ACTOR_CONTRACT_MUTATIONS:
        good = run([str(baseline), "--exact", selected, "--test-threads=1"], 30)
        if good.returncode != 0 or "1 passed; 0 failed;" not in good.stdout:
            raise RuntimeError(f"{name}: exact selected baseline did not pass\n{good.stdout}\n{good.stderr}")
        binary = _build(source, work, name, run, edits)
        result = run([str(binary), "--exact", selected, "--test-threads=1"], 30)
        output = result.stdout + result.stderr
        # libtest catches assertion panics and reports them on stdout, exit101.
        if (result.returncode != 101 or "0 passed; 1 failed;" not in result.stdout
                or f"test {selected} ... FAILED" not in result.stdout
                or label not in output or ("assertion `left == right` failed" not in output
                    and "assertion `left != right` failed" not in output)):
            raise RuntimeError(f"{name}: not the intended assertion kill\n{output}")
        panics = re.findall(r"panicked at [^\r\n]+:\r?\n([^\r\n]+)", output)
        expected_prefixes = ("actor guard fixture unwind", "assertion `left == right` failed", "assertion `left != right` failed")
        assertions = [message for message in panics if message.startswith("assertion ")]
        if (len(assertions) != 1 or label not in assertions[0]
                or any(not message.startswith(expected_prefixes) for message in panics)):
            raise RuntimeError(f"{name}: unrelated panic is not a kill\n{output}")
        print(f"{name}: KILLED by {label}", flush=True)
    return len(ACTOR_CONTRACT_MUTATIONS)


# New collision contracts; the six existing actor contracts above stay unchanged.
COLLISION_CONTRACT_MUTATIONS = (('collision_launch_recorded_twice',
  'game::player_collision_host::checks::unchecked_launch',
  'collision.owned.unchecked_launch_exact_once',
  (('player_collision/motion.rs',
    '    crate::game::launch_motion::record_movement(\n        &mut h.session_mut().projectiles.launch,\n        direction,\n        amount,\n    );',
    '    crate::game::launch_motion::record_movement(\n        &mut h.session_mut().projectiles.launch,\n        direction,\n        amount,\n    );\n    crate::game::launch_motion::record_movement(\n        &mut h.session_mut().projectiles.launch,\n        direction,\n        amount,\n    );'),)),
 ('collision_shinespark_discards_shared_sequence',
  'game::player_collision_host::checks::shared_sequence_real_motion',
  'collision.owned.sequence_shared_after_move',
  (('player_collision/motion.rs',
    '        let next = sequence.wrapping_add(fixed(&session.player.contact.y_acceleration));',
    '        let next = 0u32.wrapping_add(fixed(&session.player.contact.y_acceleration));'),)),
 ('collision_pose_probe_commits_early',
  'game::player_collision_host::checks::clearance_probe_keeps_position',
  'collision.owned.pose_probe_no_coordinate_commit',
  (('player_collision/motion.rs',
    '            let displacement = geometry::pose_probe(h, amount);',
    '            let displacement = geometry::pose_probe(h, amount);\n'
    '            commit_position(h, Axis::Y, displacement);'),)),
 ('collision_actor_cancel_after_commit',
  'game::player_collision_host::checks::actor_cancellation_precedes_position_and_launch',
  'collision.order.actor_cancel_before_launch',
  (('player_collision/motion.rs',
    '        clear_horizontal_speed(h);\n        commit_and_record(h, direction, signed);',
    '        commit_and_record(h, direction, signed);\n        clear_horizontal_speed(h);'),)),
 ('collision_terrain_cancel_before_launch',
  'game::player_collision_host::checks::terrain_cancellation_follows_position_and_launch',
  'collision.order.terrain_cancel_after_launch',
  (('player_collision/motion.rs',
    '        let result = geometry::terrain_move(h, Axis::X, signed);\n'
    '        record(h, direction, result.displacement);',
    '        let result = geometry::terrain_move(h, Axis::X, signed);\n'
    '        clear_horizontal_speed(h);\n'
    '        record(h, direction, result.displacement);'),)),
 ('collision_wall_uses_initial_pose_after_setup',
  'game::player_collision_host::checks::synthetic_wall_mechanism_rereads_pose_after_initial_readiness',
  'collision.synthetic.wall_late_pose_initial_readiness',
  (('player_collision/jump.rs',
    '        // Terrain effects may have changed the pose since the readiness decision.\n'
    '        let pose = h.session().player.pose;',
    '        // Deliberate fault: retain the initial pose across the terrain effect.\n'
    '        let pose = pose;'),)))

COLLISION_TESTS = ('game::player_collision_host::checks::unchecked_launch',
 'game::player_collision_host::checks::frame_reads_owned_clock',
 'game::player_collision_host::checks::radius_reads_owned_player',
 'game::player_collision_host::checks::shared_sequence_real_motion',
 'game::player_collision_host::checks::clearance_probe_keeps_position',
 'game::player_collision_host::checks::control_skip_clears_speed_without_motion',
 'game::player_collision_host::checks::grapple_rejects_empty_room_and_publishes_no_actor',
 'game::player_collision_host::checks::walking_captures_transition_before_current_pose_lookup',
 'game::player_collision_host::checks::equal_radius_pose_clears_prior_probe_flags',
 'game::player_collision_host::checks::synthetic_mechanism_commit_reads_live_owner_and_local_displacement',
 'game::player_collision_host::checks::synthetic_grapple_mechanism_refreshes_jump_inputs',
 'game::player_collision_host::checks::synthetic_wall_mechanism_rereads_pose_after_initial_readiness',
 'game::player_collision_host::checks::synthetic_mechanism_unwind_keeps_partial_owner_publication',
 'game::player_collision_host::checks::actor_cancellation_precedes_position_and_launch',
 'game::player_collision_host::checks::terrain_cancellation_follows_position_and_launch')


def check_collision_contracts(source, work, run):
    """Use the existing libtest build helper and require every named collision test."""
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise RuntimeError("collision mutation execution follows the existing GitHub-only runner policy")
    baseline = _build(source, work, "collision-contract-baseline", run)
    all_tests = run([str(baseline), "--test-threads=1"], 30)
    if all_tests.returncode != 0 or not re.search(r"test result: ok\. \d+ passed; 0 failed;", all_tests.stdout):
        raise RuntimeError(f"collision full detached baseline failed\n{all_tests.stdout}\n{all_tests.stderr}")
    for selected in COLLISION_TESTS:
        if f"test {selected} ... ok" not in all_tests.stdout:
            raise RuntimeError(f"collision baseline omitted or ignored {selected}")
    for name, selected, label, edits in COLLISION_CONTRACT_MUTATIONS:
        good = run([str(baseline), "--exact", selected, "--test-threads=1"], 30)
        if (good.returncode != 0 or "1 passed; 0 failed;" not in good.stdout
                or f"test {selected} ... ok" not in good.stdout or good.stderr.strip()):
            raise RuntimeError(f"{name}: exact selected baseline did not pass\n{good.stdout}\n{good.stderr}")
        binary = _build(source, work, name, run, edits)
        result = run([str(binary), "--exact", selected, "--test-threads=1"], 30)
        output = result.stdout + result.stderr
        panics = re.findall(r"panicked at ([^\r\n]+):\r?\n([^\r\n]+)", output)
        wanted = "assertion `left == right` failed: " + label
        source_name = ("player_collision_host.rs" if selected.endswith("actor_cancellation_precedes_position_and_launch")
                       or selected.endswith("terrain_cancellation_follows_position_and_launch")
                       or selected.endswith("synthetic_wall_mechanism_rereads_pose_after_initial_readiness")
                       else "player_collision_owned_checks.rs")
        if (result.returncode != 101 or "0 passed; 1 failed;" not in result.stdout
                or f"test {selected} ... FAILED" not in result.stdout
                or len(panics) != 1 or panics[0][1] != wanted
                or not re.search(r"(?:^|/)" + re.escape(source_name) + r":\d+:\d+$", panics[0][0].replace("\\", "/"))):
            raise RuntimeError(f"{name}: unrelated failure or survival is not the intended collision kill\n{output}")
        print(f"{name}: KILLED by {label}", flush=True)
    return len(COLLISION_CONTRACT_MUTATIONS)
