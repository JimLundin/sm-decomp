# Ticket 137 admission reconciliation

Audited on 2026-10-12 at integration revision
`b6f30bd3bab4a0bae4fc5329f6f3e38e4b55164c`. Read-only assignment;
the integration owner owns ticket status and acceptance pointers.

Recommendation: **PASS for the completed ticket 137 implementation and its
documented Rust migration lineage. Resolve 137 and unblock 138.** There is no
remaining native allocation, scheduler or material-capture gap. Do not rerun
the full route for this reconciliation. Do not describe historical seals as
current-source authentication of the entire coordinator checkout.

## Requirements

| Requirement | Result | Evidence and qualification |
| --- | --- | --- |
| 1. Native initializers, no-op, highest-free allocation and partial resets | Pass | `692805c`, retained 5,409-case initializer/allocation family, semantic controls and WithGfx inspection. Current initializer/scheduler files equal rustfmt of the implementation checkpoint byte-for-byte. Inner saturation writes only parameter; native WithGfx first reads its validated enemy graphics source, as C does. |
| 2. Immutable complete-call support and accepted-error handling | Pass | Current `observed_projectile_ownership.rs` has unchanged SHA-256 `b8e77711094a2de4e0222c587d81a104f48af25fde7b60335ba409063fc54b56`. All nine definition headers/native scripts, canonical slots, terrain extents/cycles/newborn closure and sound cursors are checked before publication. Retained compiled controls prove refusal and accepted-error termination without C retry. |
| 3. Actual eight-word external host bridge, including same-bank callers | Pass | Retained compiled bridge controls and full-WRAM/eight-word live captures; current native bridge templates retain all actual globals, three native entries, success-only publication and accepted-error exit 4. Native post-migration replay uses `synchronous-eight-word-host-v2-with-gfx`. |
| 4. Actual-C full-state/return/host differential domain | Pass | Six-family batch: 13,624 cases; independent WithGfx inspection: 1,530 calls/9,182 checked members; chain inspection: 1,840 steps/11,040 checked artifacts. Covers both impact branches, F suppression, deletion/reuse, lower/higher newborn scheduling, all highest-free slots, saturation, area/source aliases and coordinate/speed/fraction boundaries. Supplemental actual-C Ceres pad/platform boundaries add 1,674 cases. Controlled chains remain separate from live replay. |
| 5. Independent semantics, ABI and checks | Pass with recorded formatting qualification | Seven retained verification-log hashes still match the completion audit: 298 port tests, strict all-target Clippy, 12 ownership tests, 7 compiled live controls, 6 native oracle/host controls and 22 other controls (1 opt-in skip). Historical workspace formatting debt was explicitly disclosed; later native final-delivery records successful fmt/Clippy. This audit did not rerun test suites or claim current coordinator strict Clippy. |
| 6. Four-process full route, parity and exercised exclusive ownership | Pass | Original full WithGfx run and later native full-route regression both reach 129,712 inputs. Both mixed subjects: native entries 16/95,325/8, allocations 16 Ridley/4 pad/4 platform, all nine scheduler IDs exclusively Rust, zero entry/ID refusal and zero CPU fallback. Original inspection checks all 6 allocation pairs and 18 scheduler pairs, complete WRAM/all 8 host words. |
| 7. Scoped cleanup and retained authentication | Pass | Original batch/live/supplemental cleanup and authentication receipts are retained. Native full replay was also cleaned, its worktree removed with exact source/evidence snapshots retained, and its 4 subjects/1,479 artifacts each successfully reauthenticated natively during this audit. No new gameplay replay, build, extraction or implementer worktree was created. |

All five blocker tickets 32, 37, 80, 81 and 88 are resolved. Original-ROM parity
and whole-game Rust ownership remain outstanding and do not form a hidden
ticket 137 acceptance claim.

## Historical admission and exact identities

The original completion audit scoped requirements 1–5. Its pending-live limits
are superseded by the final ticket/checkpoint update and completed live records;
the audit JSON itself is historical and should be preserved.

- Original replay:
  `out/port/observed-projectile-with-gfx-live-full/20261011T142927.603436Z/verdict.json`.
  SHA-256 `a5aa33ec2b0a38e18c7d052ea1ffc91855d67e516a364947dca31fdba931aead`.
  Retained binary SHA-256
  `324faa9bb10b99a3552232fd5b5b1e977ccc0caf168d976ec8dee5523d56bb6d`.
- Completed live inspection SHA-256:
  `2abf2011ecd3ea55a5a62c62146fb2ff89f1f12d6d7a1570538a3aa2c917b556`.
  This audit separately verified all 96 native allocation/scheduler artifact
  hashes in each of the four subjects against the final verdict: 384 matches.
- Original offline batch:
  `out/port/observed-projectile-with-gfx-final/batch.json`.
  SHA-256 `e9e48d3f76ff922f2e436bd58dc0411a9b9ef885135f723db9f39d8b9828106e`.
  All six verdict hashes still match the completion audit. Its `replays = []`
  is a missing linkage, not evidence of an unfinished replay.
- WithGfx offline inspection SHA-256:
  `021e1bf89482dd5e9573172a4fbedc5d97972f526529fcdc0136bf01ad5132ce`.
- Chain inspection SHA-256:
  `d7f6ea99dc10a8d34e9ab37fc5fc82ac548094831b45ee88aa929879fe35fc4f`.
- Supplemental Ceres authentication SHA-256:
  `3fafdb2ed9f3fd52d3f9e72312497c5e8bb032d54611784a7865fd2add0af097`.

These older JSON records prove the frozen implementation at `692805c` and
the recorded post-cleanup admission. Python tooling and former current-source
paths have since migrated. Their old dependency hashes must not be rebound to
new source or described as current-source authentication today.

## Native migration lineage

The retained post-migration replay is
`/tmp/sm-pipeline-upgrade-integration/out/ticket-03/final-live/1791814326101526304-2/verdict.toml`.
SHA-256 `b54db24d3e7332b51e00d40055e51b2877420d5637e4a48d4e415af19186a0fe`.
Its complete-route ownership counts are the same as the original 137 admission.
Its declared gameplay/source snapshot matches the current relevant files:

| Source | Current and native snapshot SHA-256 |
| --- | --- |
| `enemy_projectiles.rs` | `4c99175b8eb0be86554e46f7424e6ee7aa43510afebe2d555ea2dcb890e7f0de` |
| `enemy_projectile_instructions.rs` | `d1ee84dda96d514fba6fb76982aaabac76d1f6d9e324c5d8d2babf5e4fbf10c2` |
| `observed_projectile_ownership.rs` | `b8e77711094a2de4e0222c587d81a104f48af25fde7b60335ba409063fc54b56` |
| `memory.rs` | `7c0df2e77ed2d05dda05bd6d9e98fef3f7ec5f1c76426da504229af698bfecc5` |
| `lib.rs` | `8121559033d5f1a62411b4c1bb105ffa83ed23df72d706afd1bd3a5926f6dd7f` |

The memory refactor moved the existing accessors without changing their bodies.
The pipeline implementation has advanced since this native replay; matching
gameplay hashes and retained authentication establish lineage, not current
authentication of every pipeline source dependency.

Fresh read-only authentication during this audit:

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
  /tmp/sm-assignment-target/debug/sm-pipeline live-authenticate \
  /tmp/sm-pipeline-upgrade-integration/out/ticket-03/final-live/1791814326101526304-2/verdict.toml
```

Exit 0, 4 subjects, 1,479 artifacts per subject, binary valid. Receipt:
`parallel-137-admission-native-live-authentication.toml`, SHA-256
`dbe569627ce016c5a8dedbb3ecb64fdd04c839ef5831d4ded68a1e46264b34a7`.
The reader also checks retained stdout counters/progress and native admission.
Ordinary telemetry did not expose worker-wait and hashing stage durations;
no measured time attribution is claimed.

## Reconciliation and next verification

Root can append the seven-requirement result and both replay pointers to ticket
137, then resolve it and claim 138. Prefer a new TOML acceptance/reconciliation
pointer linking immutable historical/native verdicts over rewriting their seals.
The old batch's missing replay linkage can be documented there.

For future current-source integration after changes, the smallest native offline
refresh preserving all existing 137 implementation case domains is 7,826 cases:

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
  /tmp/sm-assignment-target/debug/sm-pipeline batch verify \
  --batch /home/jim/sm-decomp/out/port/parallel-137-current/batch.toml \
  --families observed-projectile-initializers observed-projectile-frame-boundaries observed-projectile-ownership \
  --jobs 2 --case-jobs 4
```

All corresponding `tools/port/*.toml` manifests exist. Counts are
5,409/577/1,840; kinds are offline-projectile/offline-projectile/projectile.
`--profile observed-projectiles` instead also refreshes the 5,798 contact/rendering
cases. New native batches require their own cleanup and current family
authentication. Historical JSON authentication is not a substitute.

## Cleanup and retention

This audit created only two small report/receipt files. It created no run,
build target, extraction or worktree; scoped reclamation is zero bytes.
Host process inspection found no old replay writer PIDs or active
`sm-pipeline`/`sm-evidence` writers after authentication completed. No shared
store, sealed source, verdict or unrelated worktree was modified.

Prior original live cleanup: ordinary 0 bytes, dedup 969,383,498 snapshot bytes;
batch dedup 2,722,496,512 bytes; supplemental Ceres dedup 330,301,440 bytes.
Native cleanup and source retention are recorded in
`.scratch/pipeline-upgrade/evidence/final-delivery.md` and the retained
`out/ticket-03/logs/final-*-auth-after-removal.toml` receipts. Preserve the
native snapshot at `/tmp/sm-pipeline-upgrade-integration`, its readers,
subject `evidence.toml` pointers, `final-evidence-store`, all referenced
shared objects and exact ROM/dependency paths. Preserve the original
diagnostic/stopped runs and original full replay independently.
Measured disk available at audit completion: 211,444,858,880 bytes; other
parallel work can change this figure.
