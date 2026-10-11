# Utilities section final audit

Status: audit complete; integration runs and cleanup pending with orchestrator

2026-10-11. Read-only audit of `/tmp/sm-decomp-pipeline` after the utilities
agent disconnected. Production sources were frozen throughout this audit.
No Cargo builds, tests, source edits, cleanup or ticket resolution were performed.

## Maintained Python and JSON inventory

The maintained tree contains no `.py`, `.pyc`, `.json` or `.jsonl` files when
excluding external `deps/`, disposable `target/`, historical `out/` and
`.scratch/`, Git metadata and agent configuration directories. The scan used
`rg --files --hidden` with those explicit exclusions. Existing historical
evidence under `.scratch/` and `out/` retains original filenames and formats.

Production `serde_json` calls occur only at the two external SingleStepTests
ingestion boundaries:

- `crates/pipeline/src/patterns.rs:31`: typed external case ingestion.
- `crates/fuzz/src/main.rs:78`: external native-mode opcode corpus ingestion.

The pattern integration tests construct the same external corpus format.
Bootstrap extracts and counts pinned external `.n.json` files; these are not
project metadata. The typed timed-pattern contract names external corpus files.
The paired-checkpoint test explicitly verifies that an old `pair.json` sidecar
is absent. No maintained utility emits internal JSON or invokes Python.

`tools/port` and `tools/decomp` contain 142 TOML configuration files.
Dependency/patch pins, movie corpus metadata, emulator sidecars, diagnostic
reports and packed evidence receipts use TOML. Large timed cases and ledgers
use explicit bounded binary formats, `SMTC` and `SMTL` version 1. Evidence
provenance validates schema version 2 and canonical TOML. Bootstrap remains an
independent locked Rust workspace, with no JSON or Python dependency.

Historical JSON links in documentation identify retained historical runs or
external sources. Current operational examples and recipes use Rust and native
TOML/binary formats. `.github/workflows/ci.yml` checks the bootstrap separately,
then workspace formatting, strict Clippy and tests, and hardware timing. The
workspace tests include both maintained-language and JSON-configuration guards
in `crates/pipeline/tests/no_python.rs`.

## Verification inspected

These are existing orchestrator logs; this agent did not rerun builds:

- `/tmp/sm-root-bootstrap-final.log`: independent bootstrap test passes (1).
- `/tmp/sm-root-bootstrap-clippy-final.log`: strict bootstrap Clippy passes.
- `/tmp/sm-root-doctor-final.log`: required host tools, pinned toolchain/ROM,
  patched hardware source, 256 external corpus files, both emulators and all
  three pinned movies pass.
- `/tmp/sm-root-clippy-auth-final.log`: latest strict workspace Clippy passes.
- `/tmp/sm-root-workspace-final-regression.log`: relevant utility controls pass,
  including evidence 11, movie 6, both native-language/configuration guards,
  bounded timing codecs, packed evidence, checkpoint sidecars, storage controls
  and emulator/diagnostic CLI tests. The expensive separately verified suites
  were explicitly filtered from this integration run by the orchestrator.

Final full pattern timing and feedback runs are still orchestrator-owned. Their
acceptance and post-cleanup authentication must be recorded before resolving
the migration tickets. Earlier historical-format successes are not substituted
for native-format final proof.

## Cleanup candidates inspected

Measurements are a snapshot, not reclaimed bytes. `du -sb` reports apparent
bytes; allocated bytes were measured separately using `du -s -B1`. Hardlinks
and live writers can make actual reclaimed bytes differ.

| Directory | Apparent bytes | Allocated bytes | Required preservation/check |
| --- | ---: | ---: | --- |
| `tools/bootstrap/target` | 277,628,164 | 280,858,624 | Wait for all bootstrap/doctor invocations; preserve any executable referenced by retained evidence. |
| `/tmp/sm-retained-verifier/target` | 134,441,106 | 135,655,424 | Wait for both retained verifier processes; retain its executable outside the target if future authentication needs it. |
| `out/runs/1791744704261672208-2/checks/patterns/build` | 257,453,953 | 259,837,952 | Interrupted run is terminal `incomplete`; retain generated cases, all per-batch binaries, reports, logs and source snapshot. |
| `out/runs/1791744701480934086-2/build` | 119,669,431 | 122,159,104 | Stranded failed feedback has no terminal verdict; confirm writer stopped and preserve diagnostic manifest, generated feedback binaries and logs. |
| `out/port/rust-migration/native-pattern-full/build` | 904,017,403 | 910,819,328 | Historical JSON format: authenticate using retained reader and audit owner inventories before removing only unsealed compilation intermediates. |

The stranded feedback manifest records the original compilation path, but its
sealed artifact's `evidence` field points to `generated/feedback-probe`.
`readiness_origins::artifact` authenticates this owned evidence path. Both the
generated copy and original executable currently have SHA-256
`2b958cb0c077127eef84753856b55c98562694619ae014cb551e7b21335671ad` and
3,046,352 bytes. Preserve the generated copy; do not manufacture a terminal
verdict merely to enable cleanup.

No retained TOML references to `checks/patterns/build/` were found in the
interrupted pattern owner outside that build directory. Its timed generator and
per-batch runner binaries are separate retained files. The native cleanup utility
recognizes terminal owner-level `build`, not this nested `checks/patterns/build`
layout. Use the documented unsupported-format scoped cleanup procedure.

The native cleanup utility must be attempted first for run owners. After writer
and dependency confirmation, proposed scoped Cargo cleanup commands are:

```sh
cargo clean --manifest-path tools/bootstrap/Cargo.toml
cargo clean --manifest-path /tmp/sm-retained-verifier/Cargo.toml
cargo clean --manifest-path Cargo.toml --target-dir out/runs/1791744704261672208-2/checks/patterns/build
cargo clean --manifest-path Cargo.toml --target-dir out/runs/1791744701480934086-2/build/feedback
```

For the historical `native-pattern-full/build`, prefer its original cleanup
utility if supported; otherwise remove only audited unsealed intermediates and
reauthenticate preserved evidence. Do not include that directory in a broad
cleanup. The main integration target and worktree remain needed by active runs
and retained source identities.

Already removed directories confirmed absent: `/tmp/sm-native-config-convert/target`,
`/tmp/sm-fixture-check/target`, `out/port/rust-migration/clippy-target` and
`out/port/rust-migration/build-target`.

Disk availability at this audit: 217,728,839,680 bytes on `/tmp` (about 202.8 GiB).
`/home/jim/sm-decomp` remains clean. Integration owner must append actual cleanup
commands/output, reclaimed bytes and post-cleanup authentication before closure.

## Actual cleanup receipts

The orchestrator subsequently completed the four scoped Cargo cleanup commands
listed above. Existing receipts inspected directly:

| Receipt | Removed files | Cargo reported bytes |
| --- | ---: | ---: |
| `/tmp/sm-root-cleanup-bootstrap-target.log` | 1,373 | 293.1 MiB |
| `/tmp/sm-root-cleanup-interrupted-pattern-build.log` | 958 | 292.5 MiB |
| `/tmp/sm-root-cleanup-retained-verifier-target.log` | 430 | 149.2 MiB |
| `/tmp/sm-root-cleanup-feedback-stranded-build.log` | 795 | 115.6 MiB |

The retained verifier was first copied to
`/tmp/sm-retained-verifier/sm-retained-verifier`; its current SHA-256 is
`b9eb09262fd06889f4af5e735ce5b55a5188b855fed93f3be371751a1a26a874`.
The interrupted pattern nested build, verifier target and stranded feedback
build/feedback directories are absent. The preserved stranded feedback
`generated/feedback-probe` still matches its sealed hash recorded above.

Bootstrap's initial cleanup succeeded, then a later invocation rebuilt
`tools/bootstrap/target`. At follow-up inspection it contained 138,007,470
apparent bytes. The orchestrator confirmed that the active readiness `just check`
and bootstrap tests recreated it; final cleanup is scheduled after readiness
completes. Do not report bootstrap output as absent at final closure merely
because the earlier cleanup receipt passed.

## Historical pattern build retention audit

Historical complete pattern owner: `out/runs/1791735444283297030-2`, terminal
`pass`, with output at `out/port/rust-migration/native-pattern-full`. The older
format remains historical and is authenticated by the retained executable
`/tmp/sm-pipeline-before-metadata`. Its original authentication implementation
is preserved in the owner's `source/files/crates/pipeline/src/patterns.rs`.

That reader checks the owned source inventory and all snapshot files, pinned
declaration/fixtures, reconstructed complete external corpus, expected/actual
ledgers, retained `timed-gen`, each retained batch `timed-run`, bodies/prepared
inputs, build/generation/execution receipts and logs, and timing oracle sources.
It does not read compilation files under output `build/`. No `build/` references
were found in the owner's manifest/verdict or instruction/timing reports.
Preserving the historical owner, output files outside `build/` and retained
reader keeps the required evidence dependencies available.

Exact pre-cleanup and post-cleanup authentication command:

```sh
/tmp/sm-pipeline-before-metadata pattern-authenticate --run /tmp/sm-decomp-pipeline/out/runs/1791735444283297030-2 --out /tmp/sm-decomp-pipeline/out/port/rust-migration/native-pattern-full --timing
```

Attempt the matching old-format utility first:

```sh
/tmp/sm-pipeline-before-metadata cleanup --root /tmp/sm-decomp-pipeline/out/port/rust-migration/native-pattern-full --apply
```

The original cleanup implementation recognizes `target/`, not `build/`, so it
cannot select this historical output's compilation directory. After confirming
no writers and passing authentication, the documented scoped fallback is:

```sh
cargo clean --manifest-path Cargo.toml --target-dir out/port/rust-migration/native-pattern-full/build
```

Then repeat the same retained-reader authentication. The orchestrator has started
historical pre-cleanup authentication in session 78711 and owns the subsequent
cleanup/reauthentication if it passes. This agent has not executed authentication,
cleanup or builds and has made no production edits. The historical build remains 904,017,403
apparent bytes / 910,819,328 allocated bytes pending the orchestrator's coordinated
action. Current free disk at follow-up: 216,348,327,936 bytes (about 201.5 GiB).
