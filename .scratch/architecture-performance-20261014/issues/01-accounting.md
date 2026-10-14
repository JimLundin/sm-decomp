# 01 — Confined output accounting

Status: resolved
Triage: ready-for-agent
Blocked by: None
Type: task
Owner: Existing ticket-10 confinement_performance owner; this session tracks review and adoption only until it finishes
Worktree (retired after main integration): /home/jim/sm-decomp-architecture-20261014/01-accounting
Branch: architecture/perf-01-accounting-20261014
Baseline: 1ce6494d7e0fdc9b8623faf3f9d9f507f4a8abb0

## Scope

crates/translation-execution/src/confinement.rs and confinement tests.

## Acceptance

Preserve every original confinement guard, logical sparse/unlinked extent accounting, ignored quota errors, process ownership and teardown. Adopt the existing owner's reviewed optimization, validate its exact source and evidence, and measure a bounded equivalent probe. No duplicate implementation while that owner is live.

## Comments

2026-10-14: Authorized by the user's selection of all five report candidates.
See ../spec.md for shared resource, source and evidence contracts.

Historical implementation review: `/root/offline_performance` identified a likely
false quota refusal in `retain_possible_targets`: descriptor-number matching
across unrelated processes can pin completed temporary files under a blocked
pipe writer. Reported to the existing owner with a separate-process regression
scenario; final source, guards and commit remain pending. Shared FD tables
created with `CLONE_FILES` must still retain possible write targets correctly.
The owner also found a pre-existing concurrent `read_dir`/unlink `ENOENT` race
and is validating a bounded complete-scan retry. Earlier focused performance
measurements are provisional until these final controls pass.

The independent-fork case reproduced the false quota refusal: unchanged V7
passed in 0.12 seconds, while candidate V5 refused with ENOSPC in 0.09 seconds.
The owner corrected table identity tracking for fork, shared `CLONE_FILES`,
successful close-range unshare, and exec (including the former thread identity).
Independent re-review found no remaining concrete issue in that fix or the
bounded complete-scan retry. Exact reviewed confinement source SHA-256:
`06b569915b4fb69260322c500f1e10f8c5e4d835b51ed95e6cdf1e38f0d599d4`.

Provisional source checkpoint `14c5cd507d9281534db3da94f21395357e075ba3`
was adopted into this worktree as `7a481ca` and the separate composition tree as
`1df7696`. Five descriptor lifecycle controls pass. Final full guards,
nonleader-exec regression, exact-source measurement and cleanup are still
pending with the original owner at that checkpoint. The completed records below
supersede that provisional status.

## Completion evidence

Adopted the original owner's reviewed implementation and completed evidence:
`14c5cd5` -> `7a481ca`, `370788a` -> `054b969`, and cleanup handoff `ccc04bc`
-> `449f1dc`. The assigned branch is clean and retained separately as requested.
Original owner session: `01a139b9-f1a2-7d30-b65f-f0741bb5d555`.
No competing implementation or production policy change was made.

The module keeps exact inode totals and checks current/in-flight targets on
every potentially growing syscall. Namespace changes invalidate the inventory;
descriptor-table identities preserve shared writes while avoiding unrelated
fork retention. Partial namespace snapshots are discarded and retried within
the observed-task bound, then refused if a complete snapshot is unavailable.
Logical sparse/unlinked extents, hard-link aliases, fatal quota errors and
descendant teardown remain enforced.

Final evidence is under
`/home/jim/sm-decomp-performance-20261014/confinement`:

- `final-v6-provenance.toml` binds source, readers, toolchain, unchanged budgets
  and the quiet measurement window. Source SHA-256 is the reviewed value above;
  measured helper SHA-256 is
  `8f825e97e00ce534878131d6838411aee99576f7e0e2a088e1501b0c67cda0b0`.
- Final guards: **16 runtime controls, 8 original guards and 32 stress
  repetitions passed**. The nonleader-exec control also passed its bounded case
  and four aggregate-overflow refusals. Logs are
  `logs/fd-table-v6-{runtime-guards,original-guards,concurrent-stress,nonleader-exec}.log`.
- Same-reader, opt-level-1 helper comparison, three-run medians with 256 retained
  files and 256 timed operations: regular writes **0.174422 -> 0.024090 s
  (7.24x)**; pipe pairs **0.195146 -> 0.044410 s (4.39x)**. Setup was timed
  separately. Retained timing files are `final-v6-baseline/timings.toml` and
  `final-v6-optimized/timings.toml`.
- Eight independent jobs with the final helper achieved **2.72x regular / 2.61x
  pipe throughput at four versus one worker** (`final-v6-parallel/parallel-timings.toml`).
  These are focused confinement comparisons, not full-pipeline speedups.
- Both scoped cleanup utility passes succeeded with **0 eligible bytes**.
  The disposable target removal reclaimed **1,281,384,448 allocated bytes**;
  observed free space increased by 1,281,368,064 bytes during removal. Final
  recorded available space was 183,735,812,096 bytes. See
  `final-v6-cleanup.toml` and `logs/final-v6-target-removal.log`.
- Final and historical source/tool/timing manifests passed after cleanup;
  retained readers still load after target removal. Persistent dependencies
  include the exact helper/readers, source snapshots, timing TOML, diagnostic
  logs and recorded host toolchain. No sealed Original or Live verdict was
  created by these focused controls.

The adopted full handoff is
`/home/jim/sm-decomp/.scratch/translation-scope-execution/ticket10/confinement-performance-handoff.md`.
The helper is a separate `sm-translation-sandbox` binary selected by an explicit
installed path/hash; a pipeline rebuild does not replace that installed helper.
The composition check stages both binaries and tests the new helper without
changing production configuration. See [combined source check](../composition.md).
