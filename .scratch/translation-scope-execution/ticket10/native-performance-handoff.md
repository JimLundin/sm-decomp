# Native execution performance

Status: claimed
Blocked by: None
Owner: `/root/native_performance`, delegated by `/root`.
Worktree: `/home/jim/sm-decomp-ticket10-work/native-performance`.
Branch: `ticket10-native-performance`; baseline: `528581b`.

This scope measures and improves native Fixture execution overhead. Owned
implementation paths are `crates/pipeline/src/translation_native.rs`,
`crates/pipeline/src/builds.rs`, `crates/pipeline/src/fixture_execution.rs` and
`crates/translation-execution/src/operation.rs`, plus focused supporting tests.
The shared counter handoff in `crates/pipeline/src/bounded_inputs.rs` is also
owned here by root's explicit follow-up. Confinement implementation belongs to
`/root/confinement_performance`.
Production authority, registry, claims and gameplay are outside this scope.

All new tools and probes are Rust. Measurements and logs are retained under
`/home/jim/sm-decomp-performance-20261014/native`; its private Cargo target uses
one build job. Shared execution budget: `SM_PIPELINE_WORKERS=4`,
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`. Root coordinates real
producer windows. Existing production binaries remain unchanged.

## Measurement contract

Use the complete original `enemy-jump-instructions` family: 16 Fixture pairs,
two routines, two compiler producers and 32 Fixture producers. Keep comparison,
coverage and each producer's bounds unchanged. Measure preparation/build,
per-Fixture execution, trusted-parent output handling, repeated source/result
authentication and finalization separately. Unchanged replay must preserve all
34 operation results. No full-catalog native execution is authorized here.

The earlier reported 23.666-second positive/replay proof and 51.40-second whole
control test are historical context, not fresh measurements. Prior `/tmp`
proofs are unavailable. The committed ignored proof reconstructs provenance
from the retained original 16-case family verdict and its sealed sources.

## Current checkpoint

Read AGENTS, domain glossary, issue-tracker/session/cleanup instructions and the
diagnosing-bugs skill. No ADR directory exists. An optimized unchanged native
test reader is building with `cargo test -p pipeline --bin sm-pipeline --release
--no-run --locked --offline --target-dir
/home/jim/sm-decomp-performance-20261014/native/target -j 1`.
Build log: `native/logs/build-baseline.log`. A development-only `deps` symlink
points to the main read-only dependency tree; confined inputs use canonical
paths. No native producer has run at this checkpoint.

## Completion evidence

Pending measurements, focused regression checks, cleanup and scoped commits.
Root owns final merged validation and worktree retirement.

## Fresh baseline and implementation checkpoint

The original provenance was reconstructed and authenticated in 1.87 seconds.
The fresh complete proof passed on the unchanged optimized Native reader:

| Measurement | Seconds |
| --- | ---: |
| Positive 16-pair proof, authentication and unchanged replay | 25.253 |
| Whole original test, including existing negative controls | 53.93 |
| Whole test user / system CPU | 10.88 / 6.86 |
| Two compiler producers | 17.789 |
| 32 Fixture producers | 5.699 |
| Fixture input identity import before durable intent | 2.863 |
| Fixture runner startup | 0.469 |
| Fixture sandbox through result publication | 1.664 |
| Fixture parent output publication | 0.648 |
| Finalization after the last producer | 0.540 |

These timestamp spans are wall time, not gameplay CPU. One compiler invocation's
timestamp was changed by the existing tampering control and is explicitly
omitted from that phase's sum. The whole-test perf sample includes the controls;
it attributed 40.98% of sampled CPU to the native reader, with the hottest
addresses in `sha2::sha256::compress256`. Its existing perf binary could not
unwind stacks; raw sample addresses and `nm` resolved the hot function.

Raw evidence is retained under the persistent `native` root:
`logs/provenance-baseline.log`, `logs/baseline-v1.log`,
`logs/baseline-v1-phases.toml`, `logs/baseline-v1.perf.data`, and
`logs/baseline-v1-perf-report.txt`. Proof: `baseline-v1/`, including all four
original control results and the exact small source snapshot. Coverage stayed
16 Fixture pairs, two routines and 34 producers for the positive result.

Reader: `tools/native-tests-baseline`, SHA-256
`d5739ed8481f78ac45ed64bcaa12079ab7448c1ce8f31d09b6bebe0cb944580d`.
All reader dependencies, including sha2, use standard Cargo release optimization
level 3. Fixed helper:
`/home/jim/sm-decomp-performance-20261014/confinement/tools/sm-translation-sandbox-v7-opt1`,
SHA-256 `fb403f0445d10cb3ce75fd2f928e6bedea27c4b9a1f5de8d445527252dcf2351`.
It is 73,352,152 bytes; the old path rehashed that unchanged helper for each
Fixture. Its optimization level is 1 and remains fixed across Native comparison.

Implemented after measurement:

- Reuse the protected stamped-input cache for read-only executable, helper and
  Fixture inputs, with full tree/stamp validation and deadline-bound cache
  publication locking. Prior runtime-only cache reuse omitted these shared inputs.
- Execute independent Fixture waves up to the shared worker limit. Continuations
  wait for a prior completed comparison. Every launched worker joins on failure;
  later waves stop, and returned cases remain in declaration order.
- Keep all prelaunch worst-case reservations in a wave until its join. The
  existing before-bytes field is a shared wave anchor; authentication checks the
  sum of the wave's reservations. Historical single-producer anchors remain
  valid. The intent records the Fixture job count, defaulting to one for old
  intents, so interrupted historical runs keep their serial schedule.
- Share trusted input bytes, paths and earliest deadline across worker threads.
  Focused tests cover aggregate exhaustion and wave failure/dependency behavior.

Two release type-check passes succeeded. The current exact optimized reader
build is in exec session `66887`, log `logs/build-waves-v2.log`. The first updated
build was stopped after 83 seconds of pipeline code generation to incorporate
root's late, narrow requirements override before a full rebuild completed.
Source is frozen for the active build. No new producer is running.

Existing ignored recipe proof now accepts `SM_NATIVE_RECIPE_SOURCE` and
`SM_NATIVE_RECIPE_REQUIREMENTS`; its provenance rationale describes complete
declarations against pinned C without a ticket-specific claim. Test-only
`SM_NATIVE_FIXTURE_JOBS` permits one/two/four comparisons without changing the
shared process capacity. Root owns the later complete `pause-equipment` proof;
that proof is not started here.

Concurrent architecture work has overlapping legacy scheduling/build-cache
branches in a different worktree. Root coordinates their later composition;
none of those edits is in this frozen comparison.
