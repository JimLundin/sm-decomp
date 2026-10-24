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
    built = run(["rustc", "--edition=2024", "-O", "--test", str(main), "-o", str(binary)], 180)
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
