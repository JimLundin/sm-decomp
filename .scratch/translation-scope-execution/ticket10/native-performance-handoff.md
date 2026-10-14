# Native execution performance

Status: claimed
Blocked by: None
Owner: `/root/native_performance`, delegated by `/root`.
Worktree: `/home/jim/sm-decomp-ticket10-work/native-performance`.
Branch: `ticket10-native-performance`; baseline: `528581b`.
Implementation commit: `5cda6a02ada0434baae86c0ae1f9074158bbd8d4`.
Worker state: implementation, measurements and run cleanup complete; root owns
integration, independent review and worktree retirement. No native child process
or build remains active.

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

## Initial checkpoint (historical)

Read AGENTS, domain glossary, issue-tracker/session/cleanup instructions and the
diagnosing-bugs skill. No ADR directory exists. An optimized unchanged native
test reader is building with `cargo test -p pipeline --bin sm-pipeline --release
--no-run --locked --offline --target-dir
/home/jim/sm-decomp-performance-20261014/native/target -j 1`.
Build log: `native/logs/build-baseline.log`. A development-only `deps` symlink
points to the main read-only dependency tree; confined inputs use canonical
paths. No native producer has run at this checkpoint.

## Completion evidence

The adjacent measurements with the same V7 helper show a **4.89× native Fixture
throughput improvement** and a **1.29× complete-control-test improvement**.
Every positive proof retained 16 Fixture pairs, two routines and 34 durable
producers; unchanged replay preserved all producer result timestamps.

| Adjacent complete proofs | Unchanged reader | Updated reader, four Fixture jobs |
| --- | ---: | ---: |
| Proof directory | `baseline-quiet-v2` | `waves-controls-v1` |
| Positive proof, authentication, replay and tampering control, seconds | 17.662 | 13.317 |
| Whole test including all original negative controls, seconds | 43.11 | 33.54 |
| Whole-test user / system CPU, seconds | 10.92 / 4.25 | 6.44 / 3.38 |
| Two compiler producers, seconds | 10.748 | 10.834 |
| Fixture wall span, seconds | 5.480 | 1.120 |
| Finalization after the last producer, seconds | 0.512 | 0.499 |

Compiler time in these adjacent runs differs by less than one percent. The
earlier fresh baseline below is retained, but its slower compiler time is not
the main before/after comparison. Times are observations on a shared host,
not statistical confidence intervals. The coordinated measurement window was
2026-10-14 17:16:48–17:21:00 UTC. Architecture builds were stopped for this
window; unrelated host activity remained outside this session's ownership.

The updated reader's fixed-helper one/two/four comparison isolates scheduling:

| Fixture jobs | Positive proof, seconds | Compiler producers, seconds | Fixture wall span, seconds | Fixture throughput versus one job | Whole perf-only test user / system CPU, seconds |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19.881 | 15.263 | 3.103 | 1.00× | 4.69 / 4.22 |
| 2 | 14.229 | 11.008 | 1.805 | 1.72× | 4.80 / 1.74 |
| 4 | 16.882 | 14.266 | 1.212 | 2.56× | 5.36 / 2.28 |

These runs are `waves-jobs1-v1`, `waves-jobs2-v1` and `waves-jobs4-v1`.
Only Fixture job count changed; the shared process capacity remained four.
Compiler variation prevents claiming monotonic end-to-end scaling from these
three observations. Their perf-only mode keeps coverage, independent
authentication, unchanged replay and invocation-tampering checks; the original
negative-control families ran separately in `waves-controls-v1`.

| Trusted-parent step, seconds | One job | Two jobs | Four jobs |
| --- | ---: | ---: | ---: |
| Preparation | 2.488 | 2.545 | 2.717 |
| Independent authentication | 0.233 | 0.195 | 0.198 |
| Unchanged replay | 0.203 | 0.214 | 0.210 |
| Reference collection | 0.034 | 0.029 | 0.024 |

Input identity reuse accounts for the serial improvement. Across the 32
Fixture producers, identity import fell from 2.847 seconds in the quiet baseline
to 0.341 seconds in the updated serial run (8.36× lower). Runner startup was
0.427 versus 0.399 seconds; sandbox execution through result publication was
1.318 versus 1.401 seconds; parent output publication was 0.590 versus 0.646
seconds. These are summed file-timestamp spans, not gameplay CPU. With parallel
workers the sums overlap; wall span is the throughput measure. The analyzer
omits the compiler invocation timestamp modified by the tampering control.
The new per-proof `performance.toml` retains its original timestamp and all
producer timings explicitly.

Validation completed:

- Two release type-check passes and an optimized release reader build passed.
  All reader dependencies, including sha2, use normal release opt-level 3.
- Focused pipeline suite: 31 passed, zero failed, nine explicit real-proof tests
  ignored; 5.31 seconds. This covers shared read limits/deadlines, summed wave
  reservations, reconciliation, worker failure joins, continuation order and
  existing native/worker controls. Log: `logs/focused-waves-v2.log`.
- Operation integration suite: 12 passed, zero failed; 3.49 seconds after its
  release build. The new cache test covers equal-length content changes with
  restored mtime, symlink introduction and historical authentication. Existing
  crash/recovery, deadline, retention and tampering controls passed.
  Log: `logs/operation-waves-v2.log`.
- The full original real-family controls passed on the updated reader:
  preparation gaps, undeclared compiler inclusion, runtime parent/sibling
  reads/writes, changed producer requests and aggregate reservation overflow.
- The updated reader authenticated historical serial receipts with no
  `fixture_jobs` field. All six finished proof sets authenticated after both
  cleanup passes and again after private target retirement; no producer ran.

The exact retained reader is `tools/native-tests-waves-v2`, SHA-256
`78804b95e8a815f3b074910e85ec5b54f7e5d054ec9979b74192f15fe7b30bdb`.
The read-only Rust phase analyzer is `tools/native-performance-analyze-v2`,
SHA-256 `4a61cd9243f468cafd9036cba52a663110915ca044793d507713963e12422b4c`.
Its source is beside this handoff. For every proof, `logs/NAME.log` contains the
raw test and `/usr/bin/time -v` output, and `logs/NAME-phases-v2.toml` records
every original producer's phases and post-producer finalization. All artifacts
are under `/home/jim/sm-decomp-performance-20261014/native`.

`retained-evidence-sha256.txt` binds the readers, helper, provenance, positive
receipts, measurements, focused-test logs, cleanup logs and final authentication
logs. Its SHA-256 is
`a5777e0582f9dbaa0822c7423144f07d5c11e97dcd0fae4a728c208b379a3f5c`.
Provenance remains `provenance/provenance.toml`, SHA-256
`c6f972b1a9400971e870714e3b241e14e4c99ce767ca4da0315736bb5ec6b88a`.

Cleanup used the unchanged retained utility
`/home/jim/sm-decomp-production-execution/bin/sm-pipeline`, separately on
`baseline-v1`, `baseline-quiet-v2`, `waves-jobs1-v1`, `waves-jobs2-v1`,
`waves-jobs4-v1`, `waves-controls-v1` and `provenance`:

```sh
sm-pipeline cleanup --root PROOF --apply
sm-pipeline cleanup --root PROOF --deduplicate --apply
```

Normal cleanup reclaimed zero bytes. Deduplication reclaimed 10,485,760 bytes
for each complete-control proof and 4,194,304 bytes for each jobs comparison:
44,040,192 bytes total. Provenance cleanup reclaimed zero bytes. Both passes
succeeded; logs are `logs/NAME-cleanup.log` and `logs/NAME-deduplicate.log`.
The utility did not identify the private Cargo target as a supported run
intermediate (`logs/native-cleanup-plan.log`). After retaining the exact readers
and observing all native builds stopped, the identified disposable target was
removed with `rm -r -- /home/jim/sm-decomp-performance-20261014/native/target`.
It occupied 387,956,736 bytes before removal
(`logs/private-target-before-removal.log`). This removal completed before
root's later message requesting target retention; the merged release check
must use another target or rebuild. No tool or proof depends on that target.
Observed remaining free space was 183,465,463,808 bytes
(`logs/disk-after-cleanup.log`).

Final authentication logs are `logs/NAME-after-target-retirement-auth.log`.
Each positive receipt authenticated 34 producers and 670 references. Every
complete-control proof also authenticated compile denial (two producers),
runtime denial (34 producers) and reservation denial (zero producers).
Retain all proof source snapshots, provenance, operation inputs/results and
references, readers and fixed helper. Root owns integration and removal of this
still-present worktree, followed by authentication after source retirement.
Root also owns the separate complete equipment proof and any later comparison
using the final improved confinement helper. None of these Native measurements
uses that later helper.

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

Two release type-check passes succeeded. The exact optimized reader build
completed successfully in 12m23s, log `logs/build-waves-v2.log`; its source is
the implementation commit above. The first updated build was stopped after
83 seconds of pipeline code generation to incorporate root's late, narrow
requirements override. All later measurements used the completed v2 reader.

Existing ignored recipe proof now accepts `SM_NATIVE_RECIPE_SOURCE` and
`SM_NATIVE_RECIPE_REQUIREMENTS`; its provenance rationale describes complete
declarations against pinned C without a ticket-specific claim. Test-only
`SM_NATIVE_FIXTURE_JOBS` permits one/two/four comparisons without changing the
shared process capacity. Root owns the separate complete `pause-equipment`
proof; no such producer was started by this worker.

Concurrent architecture work has overlapping legacy scheduling/build-cache
branches in a different worktree. Root coordinates their later composition;
none of those edits is in this frozen comparison.
