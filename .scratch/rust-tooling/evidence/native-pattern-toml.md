# Native pattern TOML migration

Status: implementation complete

The timed declaration and 19 source-owned independent fixtures now use TOML;
their JSON source files and native JSON import routes are removed. The producer
and retained-evidence authenticator preserve and consume `declaration.toml` and
`independent-fixtures.toml`. External SingleStepTests corpus files remain JSON.

`pipeline-model::timed_pattern` owns typed declaration/selection validation shared
by the recompiler and verifier. `recomp` no longer depends on `serde_json`.
Inactive bus values are omitted in typed native fixture cycle tables; the
external corpus runner receives its original row shape in memory.

## Verification

- Model/recomp suite passed after conversion (selection, exclusion and prospective controls).
- Exhaustive production support over 256 opcodes and four M/X widths passed;
  prospective proof remains outside normal support (2/2).
- Native declaration scope and 19 literal fixture/absent-cycle controls passed (2/2).
- Retried native contract controls passed after the final loader change:
  `/tmp/sm-native-pattern-toml-unit.log`.
- Model/recomp all-target strict Clippy passed:
  `/tmp/sm-native-pattern-model-clippy.log`.
- Actual emitted-step integration passed in 160.45 seconds, including literal
  expectations, declaration tampering, binary tampering, truncated ledger,
  failed expected state, rejected override and reordered opcode coverage:
  `/tmp/sm-native-pattern-toml-controls.log`. Earlier attempts were blocked by
  concurrently incomplete iteration-promotion edits, then retried.
- Integrated strict Clippy reached a pre-existing concurrent HDMA insertion
  `collapsible_if` warning, reported to its owning agent; no pattern warnings:
  `/tmp/sm-native-pattern-toml-clippy.log`.

## Cleanup

The one-time migration converter was Rust and lives outside the project.
`cargo clean --manifest-path /tmp/sm-pattern-toml-migration/Cargo.toml` removed
225 files / 131.6 MiB. Disk available after cleanup: 236,558,917,632 bytes.
The implementation worktree and shared Cargo artifacts remain for integration.
The actual integration test removed its private source checkout and three
run/build directories after all controls passed. Immediately before automatic
removal its disposable root measured 1,314,600,467 bytes. No verification
processes remain for this task; the earlier blocked compilation attempts did
not create run evidence.
