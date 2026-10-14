# 03 — Authenticated build preparation

Status: resolved
Triage: ready-for-agent
Blocked by: None
Type: task
Owner: /root/implement_builds under session 01a13b35-079e-7151-8e5a-f759fee6bcf3
Worktree: /home/jim/sm-decomp-architecture-20261014/03-builds
Branch: architecture/perf-03-builds-20261014
Baseline: 1ce6494d7e0fdc9b8623faf3f9d9f507f4a8abb0

## Scope

crates/pipeline/src/main.rs, builds.rs and family.rs; build portions of live.rs after coordination with 05; crates/pipeline/tests/processes.rs; this effort's plan.

## Acceptance

Concentrate build identity/dependency preparation; deduplicate input identity work, preserve revalidation after waits, authentication and atomic cache publication. Reuse stable generated artifacts with exact source/tool/flag/environment keys. Keep confined Cargo cache restrictions and fresh Live captures unchanged. Public cache mutation/corruption/concurrency controls and bounded cold/warm comparison.

## Comments

2026-10-14: Authorized by the user's selection of all five report candidates.
See ../spec.md for shared resource, source and evidence contracts.

Independent cache review by `/root/live_performance` found two defects in the
initial implementation, before completion: a newly added Rust `include!` could
fall outside the discovered closure during a concurrent edit, and rustup could
select different compilers for checkout identity and run-directory compilation.
The implementer confirmed both. Final publication must verify actual compiler
dep-info against the authenticated closure and use the same pinned Rust
compiler for discovery, identity and build. A separate concurrency review also
identified the helper scan's unbounded subprocess; it now uses the existing
timed execution interface. These fixes and sibling-wait cancellation passed
the final focused regressions and independent re-review recorded below.

## Completion evidence

Completed; see the frozen implementation, final validation and cleanup records below.

### Superseded historical checkpoint — 2026-10-14

Replacement implementer: `/root/implement_builds`, coordinator session
`01a13b35-079e-7151-8e5a-f759fee6bcf3`. Scoped uncommitted source is in the owned
03-builds worktree. Private Cargo target:
`/home/jim/sm-decomp-architecture-runs-20261014/03-builds/target` (jobs=1,
locked/offline). Initial compilation found and fixed the moved CacheSeal
benchmark import and one Path/PathBuf mismatch. First test process session
90821 compiled before final source edits; final rebuild is required.

Implemented operation-local C preparation, bounded independent C compilation,
standalone fixture_worker/RNG helper reuse, and source fingerprint registration
for the extracted build_cache.rs. Public tests are in
`crates/pipeline/tests/build_preparation.rs`; existing cache protocol tests remain
in `processes.rs`. The retained ignored measurement uses
`SM_BUILD_PREPARATION_PROOF` and optionally `SM_BUILD_BASELINE_PIPELINE`.
No measurements have run yet; the coordinator is arranging a quiet window.

Read-only review identified two cache correctness gaps to fix before freezing:
dependency discovery must be tied to bytes actually compiled (including changing
transitive include sources), and helper compilation outside checkout must use
the exact toolchain selected for its identity. The implementation will freeze
known roots before discovery, compare actual compilation dep-info before cache
publication, and pin the Rust toolchain executable. A third review fix is already
applied: helper dependency generation now uses the existing bounded compiler
execution and shared lease, rather than an unbounded output wait. C parallel
failure still joins in-flight sibling work; explicit signal teardown is to be
covered. No source commit or completion claim yet.

### Frozen implementation and final validation

Source commits: `3a732b1` and `0ddfae6` on
`architecture/perf-03-builds-20261014`. The latter only retains the exact header,
program and cache/metrics records for each measurement sample. Code ownership
also includes one-line source fingerprint registration in `inputs.rs`,
`decomp_job.rs` and `translation_execution_tests.rs`, plus two registration lines
in `translation_native.rs`; the coordinator approved this extension and alerted
the Native owner.

The build preparation module now owns compiler/repository/environment identity,
unique initial input hashing, cache locks, authenticated restore and atomic
publication. One preparation covers all C translation units, with a bounded
queue using the existing shared process leases. A failed unit cancels sibling
cache-lock waits without hiding its actual compiler diagnostic. Already-running
compilers and shared-worker waits retain their existing limits and signal
teardown. The interface remains the typed `c-build` command; the fixed
`helper-build` recipes are `fixture-worker`, `random-library` and `random-cli`.

Stable helpers use the checkout-selected absolute Rust compiler, including when
run directories select a different rustup toolchain. Known roots freeze before
discovery; the dependency list emitted by the actual compiler must match the
discovered closure before publication. Header/options/corruption controls,
transitive include changes and post-wait input checks remain fail-closed. The
helper recipes explicitly reject env!/option_env! dependencies, which none of
the shipped helpers use. Generated Live replay `libfamilies.a` stays fresh:
its per-run source paths and file! semantics are not normalized for reuse.
Confined Cargo compilation and fresh Live replay captures are unchanged.

Independent cache and concurrency reviews found the discovery race, toolchain
context mismatch, unbounded new helper scan and sibling cache-wait hang during
implementation; all were fixed and covered before final gates. The final
read-only review found no remaining concrete defect in those changes.

Final checks (locked/offline, jobs=1; test_threads=1):

- `cargo test -p pipeline --locked --offline --jobs 1 --test processes --test build_preparation -- --test-threads=1`: 7 new controls and 4 existing controls passed. New controls include independent C progress, one compiler identity probe per build, actual held-cache-lock mutation, failed-sibling cancellation, multiple-compiler SIGTERM/reaping, helper corruption/include invalidation, transitive discovery race, and checkout toolchain selection across run overrides and a cache wait.
- `cargo test -p pipeline --locked --offline --jobs 1 --test family fork_parent_is_pristine_and_execution_seals_cannot_be_stripped -- --exact --nocapture --test-threads=1`: passed, exercising three actual-C Fixture pairs, pristine fork children, authentication and rejection after stripping the execution seal. Its temporary test project ran both automatic cleanup passes and was removed.
- `cargo test -p pipeline --locked --offline --jobs 1 --test build_preparation retained_cold_warm_build_preparation_measurement -- --exact --ignored --nocapture --test-threads=1`: passed in 34.31s; used the shared `/home/jim/sm-decomp/out/pipeline` root and workers=4.
- Final compile succeeded in 1m25s; `git diff --check` passed. Only existing dead-code warnings remain.

All logs and proof files are under
`/home/jim/sm-decomp-architecture-runs-20261014/03-builds`:
`logs/final-compile.log`, `logs/final-tests.log`, `logs/family-control.log`,
`logs/measurement.log`, `proof/measurements.toml`. Initial compile diagnostics
and the intentional stopped/restarted intermediate compile are retained under
`logs/`. The measured candidate reader is retained separately as
`bin/sm-pipeline`, SHA-256
`484341e15304019cbf95340bf998c5010c76cd2e20f95bc69d3d8c9fa55d57fc`;
`proof/candidate-binary.sha256` uses that durable copy while
`candidate-build-path.sha256` preserves the original Cargo path.

Bounded observations for eight C translation units:

| Sample | Wall seconds | Compiler identity probes |
| --- | ---: | ---: |
| Installed comparator, cold | 3.698 | 8 |
| Installed comparator, warm | 3.343 | 8 |
| Prepared build, comparator cache | 3.323 | 1 |
| Prepared build, cold | 3.600 | 1 |
| Prepared build, warm | 3.336 | 1 |

All eight comparator cache entries were reused with identical v4 keys, and all
five linked programs have the same SHA-256. Three stable helpers took 4.960s
cold and 4.341s warm; all three warm results were authenticated hits with identical
artifact bytes, and cold/warm RNG exhaustive output matched over all 65,536
seeds. This establishes removed work and compatible reuse, not a controlled
end-to-end speedup: the comparator is the retained installed binary rather than
a fresh identical-profile baseline build, timings include CLI self-authentication,
and other sessions were active before/after the sample. Host observations are
retained in `logs/measurement-host-before.txt` and `measurement-host-after.txt`.

Run cleanup and retained-target disposition are recorded below.

### Cleanup and handoff

All verification/test processes finished before cleanup. The retained candidate
binary ran both required scoped utility passes:

```sh
/home/jim/sm-decomp-architecture-runs-20261014/03-builds/bin/sm-pipeline cleanup --root /home/jim/sm-decomp-architecture-runs-20261014/03-builds/proof --apply
/home/jim/sm-decomp-architecture-runs-20261014/03-builds/bin/sm-pipeline cleanup --root /home/jim/sm-decomp-architecture-runs-20261014/03-builds/proof --deduplicate --apply
```

Both succeeded, reclaiming **0 bytes** (`logs/cleanup.toml` and
`logs/cleanup-deduplicated.toml`). Per-object temporary dependency files and C
objects had already been removed by the successful builds. All retained reader,
program, header snapshot, helper library/CLI and exhaustive RNG hashes still
passed after cleanup (`proof/retained-files.sha256`,
`logs/after-cleanup-authentication.log`). No Live replay was run for this ticket,
so there is no new Live verdict requiring live-authenticate.

Remaining filesystem space: **183,098,662,912 bytes**, recorded in
`logs/free-space.txt`. Retained proof: 108,984,792 logical bytes, including its
exact generated source checkouts and compiler adapter; measured reader retained
separately under `bin/`. These paths plus the two source commits provide the
proof dependencies. The user's separate review worktree remains in place.

The 2,135,640,990-byte private Cargo target was released to the coordinator for
all-five composition verification. After that verification, its 2,765,496,164
apparent bytes were removed and every retained candidate03 proof hash passed
again. See [composition cleanup](../composition.md) and
`/home/jim/sm-decomp-architecture-runs-20261014/composition/logs/after-target-removal-build-proof.log`.
No candidate03 producer or disposable target remains. Source and coordination
records are committed in the retained review worktree.
