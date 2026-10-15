# Fresh frozen-source Native verification

Status: active; baseline gates and cleanup finished, recipe/full-real acceptance gaps retained
Owner: `/root` integration owner; `/root/native_verification_resume` has returned
the live proof after checked recovery of stopped `/root/recover_extraction_verification`
Integration owner session: `01a13e7f-ab01-7413-a1c2-288b08a4b719`
Frozen source: `7b27ea1a29bb2ade3bc6483f99bf59626e298671`
Worktree: `/home/jim/sm-decomp-performance-integration`
Branch: `performance/verification-integration-20261015`

## Current outcomes (supersedes historical launch checkpoints below)

The resumed twelve-recipe gate exited 101 after 850.51 seconds in libtest
(850.52 seconds externally). Eleven recipes passed, totaling 6,863 Fixture
pairs. Ceres-pad failed during C linking, before its required 1,296 pairs.
`recipes/summary.toml` and `logs/recipes-resume-01.{log,time}` are authoritative.
Its receipt records rejection; receipt existence is not acceptance.

The missing C symbols are transitive dependencies retained by
`sm_90.c`'s Samus handler table. The existing Ceres recipe supplements
`sm_86.c`/`sm_rtl.c` with only `sm_90.c`. Root owns a focused actual-C recipe
repair in a separate worktree; this frozen source stays unchanged.

Both cleanup passes completed for `recipes/` and `recipes-tmp/`, reclaiming
zero additional bytes after the harness's final 262,144-byte deduplication.
All 27,539 sealed receipt/intent/result/preparation/summary files passed
post-cleanup hashing (`logs/recipes-before-cleanup.sha256` and
`logs/recipes-after-cleanup-seals.log`). A separate
`recipes/observed-authentication.toml` records eleven successes and the actual
zero-pair, one-operation Ceres rejection. The original expected-pass manifest
remains unchanged. Independent authentication passed in 162.48 seconds in
exec `13646`; outputs are `logs/recipes-observed-authentication.{log,time}`.

The original full-real gate (exec `65060`) finished with exit 101:
`operation total deadline exhausted during declared input discovery` in the
C compiler operation. It took 112.16 seconds in libtest, 113.01 externally.
No Fixture pair ran, and later full-real negative controls were not reached.
The original gate/compiler/reconciliation budgets remain unchanged. Both
cleanup passes completed for `full-real/` and `full-real-tmp/`, all zero bytes;
every sealed member passed afterward. Its separate observed-outcome manifest
records zero pairs and one rejected operation. Independent authentication of
that failure passed. Original expected-pass manifests are diagnostic
gate requirements, not successful receipts.

The default retained application reader completed in exec `12823`: 164 passed,
two failed, 21 ignored, 342.60 seconds. Both failures were the trusted-input
symlink refusal (fixture-domain generation and Ridley command construction).
The ignored C/ROM symlinks in the integration checkout were replaced with
independent byte-checked copies, preserving every tracked source byte. Both
affected tests then passed with the same reader (6.62 seconds and 0.00 seconds).
The other 164 successes were not repeated. Original failures remain in
`logs/application.{log,time}`; corrected-prerequisite checks are in
`logs/application-*-local-inputs.{log,time}` and copy checks in
`logs/application-{inputs-*,c-source-copy-check.log,local-rom.sha256}`.

Both cleanup passes after the suite and after the two targeted checks passed,
all zero bytes. No retained verdict or completion remains in this unit scope.
Its source-independent behavioral controls are complete; the ignored real
Native gates retain the gaps above. The suite built its `recomp` prerequisites
in the integration checkout's disposable `target/`, which remains for final
retirement. No performance worker owns that target. The retained Native tools,
prerequisites and source stay available independently of it.

These interrupted or failed observations establish no complete-batch speedup.
Remaining acceptance includes repaired Ceres coverage, successful full-real
positive/negative controls, final combined-source gates and post-retirement
authentication. Installed production tools and policies remain unchanged.

## Ownership and required scope

This is a new proof, independent of the failed historical PLM attempt. The
candidate combines the extracted owners, metrics changes, storage-only example
and Native operation admission fix. The integration owner keeps this worktree
frozen and owns merges, shared registration, final admission and retirement.
This verifier writes only its new proof parent and this handoff. Ignored
read-only-use links supply the existing `deps/snesrev-sm`, `deps/rustysnes`
and private ROM prerequisites; no tracked candidate source is modified.

Retained parent:
`/home/jim/sm-decomp-performance-native-proof/7b27ea1-20261015`.

Required gates retain the original Native plan: complete catalog preparation,
twelve complete recipe declarations / 8,159 Fixture pairs with PLM first, the
complete 16-pair full-real Native proof including negative controls, applicable
application checks, both cleanup passes and independent retained authentication.
No `SM_NATIVE_PERFORMANCE_ONLY`, domain reduction, deadline enlargement or
historical-attempt replay is authorized.

## Source and build checkpoint

`source-commit.txt` and `tracked-source.sha256` seal the complete source.
`frozen-source/` and `catalog-source/` were independently copied from the exact
Git archive and checked against those hashes; each has a copied private ROM.
`prerequisites.sha256` records the pinned C archive, original/copied ROM and
captured Ridley input hashes. The Native harness retains and authenticates all
other declared absolute Original capture dependencies at their existing paths.

The idle old extraction Cargo target is exclusively reserved for these builds:
`CARGO_TARGET_DIR=/home/jim/sm-decomp-offline-verifier/target`. Its earlier emitted
readers are already separately retained. Process inspection found no writer
before reuse. All Cargo commands use locked/offline release, one job, and a
separate 1,800-second build bound. All producers use the shared root
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, `SM_PIPELINE_WORKERS=4`.

The reader build completed successfully (exit 0). Its externally measured wall
time was 17:12.39; Cargo's compilation summary reported 15m41s. The emitted
reader was independently copied to `tools/sm-pipeline-tests`, compared with the
emitted file, and sealed in `tools/reader.sha256`:

`3ee16ec9010ac48178ad60bec656f3796e7a7f8f0d39cf585da0e8347f9510fd`.

Its Cargo fingerprint is retained in `logs/reader-build-fingerprint.json`.
TMPDIR is `<proof-parent>/build-tmp`.

```sh
timeout --signal=TERM 1800s cargo test --locked --offline --release -j 1 \
  -p pipeline --bin sm-pipeline --no-run
```

Exact stdout/stderr and command/resource results are in `logs/reader-build.log`
and `logs/reader-build.time`. The matching CLI and separate sandbox,
recorded-rewrite and evidence helper build is active in exec session `1066`:

```sh
timeout --signal=TERM 1800s cargo build --locked --offline --release -j 1 \
  -p pipeline -p translation-execution -p evidence --bins
```

Observed on recovery: Cargo PID `1622004`, timeout PID `1622002`, starting
2026-10-15 16:56:53 Europe/Stockholm (14:56:53 UTC). Its exact output and
resource result are `logs/controller-helpers-build.log` and
`logs/controller-helpers-build.time`. Inspect liveness before resuming; do not
launch a duplicate. Retain and hash each freshly emitted executable after it
finishes. The 33 CLI integration targets are not rebuilt merely to obtain the
reader. Later execution invokes retained binaries directly so a narrowed Cargo
command cannot change the feature cohort.

### Checked recovery, 2026-10-15

The integrator observed the old worker stopped and delegated only this proof
parent and handoff to the replacement verifier. Recovery process inspection
found no surviving Cargo, rustc, reader or producer. The helper build had
finished successfully: Cargo reports 21m58s, while the external resource log
reports 37m41.13 and exit 0. The host/session interruption leaves that wall-clock
discrepancy unexplained; it is retained as observed and is not a performance
comparison. No completed build or catalog gate was repeated.

Freshly emitted helpers were copied independently from the exclusively held
target, compared byte for byte, and sealed in `tools/helpers.sha256`:

| Retained executable | SHA-256 |
| --- | --- |
| `sm-pipeline` | `20c8a35fab96ce9e34e6c8817d26eb4abb1b30f73c162116f31d093d91fbe108` |
| `sm-translation-sandbox` | `3795227bf9930061fd767a82e8897f42df56f7347d7767e155c1080f5bf22187` |
| `sm-translation-recorded-rewrite` | `fa774ed35b6251b4e465ea053ac0273219d87297befc871bacd37e49164b6b3f` |
| `sm-evidence` | `3d698b74aff8aadb0b77e14fe39bed35fb481553774e01c3ea871f9c3a7d35e6` |

The retained reader seal, all frozen tracked-source hashes, private prerequisite
hashes and clean integrated HEAD were checked again before producer launch.
Logs: `logs/*-recovery-check.log`,
`logs/frozen-source-before-producers-recovery.log`,
`logs/prerequisites-before-producers-recovery.log`,
`logs/recovery-processes.txt`, `logs/recovery-boot-id.txt` and
`logs/recovery-observed.txt`.

Both retained-CLI cleanup passes completed for each of `catalog/` and
`catalog-tmp/`; all four reclaimed zero bytes. Every catalog record and generated
source hash passed afterward. Exact stdout/stderr/resource results are retained
under `logs/catalog[-tmp]-cleanup*`; free space is in
`logs/recovery-free-space-after-catalog.txt`. Catalog has no Native receipt or
producer to authenticate. Its post-cleanup check authenticates the retained
catalog records and generated-input seals without replaying preparation.

Next approved command is recorded in `logs/recipes-command.txt`: the exact
retained reader selects one ignored recipe gate with all twelve declarations,
PLM first, unchanged per-family/producer limits, shared four-worker capacity,
and `SM_NATIVE_PERFORMANCE_ONLY` absent. No prior `recipes/` output exists.
Checkpoint its process identity immediately after launch. The frozen assembly
remains unchanged; application diagnostics on that source do not authorize a
source edit by this verifier.

Recipe launch checkpoint: exec session `11384`, timer PID `9353`, retained-reader
PID `9356`, both observed starting 2026-10-15 18:53:32 Europe/Stockholm
(16:53:32 UTC). `logs/recipes.started.txt`, `logs/recipes-processes.txt` and
`logs/recipes-proc-start.stat` retain the start identity. Initial stdout selects
one test. Observe that exact process/`logs/recipes.log`; do not duplicate it.

### Live-process handback to the integrator

At the integrator's explicit direction, this verifier returns the still-running
recipe process without terminating it or launching another gate. Exclusive
proof/handoff ownership returns to the integration owner. Latest liveness and
kernel start ticks are recorded in `logs/recipes-handoff-observed.txt`,
`logs/recipes-handoff-processes.txt` and `logs/recipes-handoff-proc.stat`.
`logs/recovery-handoff-seals.sha256` binds recovered tool/source/prerequisite,
catalog-cleanup and next-command records. Recipe stdout remains
`logs/recipes.log`; `/usr/bin/time` writes final resource statistics and the
process exit status to `logs/recipes.time` when it completes. There is no
separate exit-code sidecar. Exec session `11384` can be polled if available;
otherwise inspect the same PID/start identity and the final time/log records.

No complete twelve-recipe gate result or new failure was available at handback.
The latest observation found all 3,941 PLM producer results plus its published
Native receipt and ordinary verdict; their initial seals/mtimes are retained in
`logs/recipe-handoff-first-receipt.sha256` and
`logs/recipe-handoff-first-receipt.stat`. Full retry/cleanup/independent
authentication still needs the gate's final outcome. Completed
catalog preparation remains passed and cleaned; its retained seals passed.
The required real recipe domain is still all twelve declarations / 8,159 pairs.
PLM is first; the successful catalog result does not admit actual-C execution.

After the live recipe process stops, inspect its exact first failure or complete
result before any retry. Seal final receipts, intents, operation indexes/chunks,
producer results and their mtimes. Check the harness's per-family cleanup
receipts; run both retained-CLI cleanup passes over `recipes/` and `recipes-tmp/`
as needed, preserving every sealed artifact and source dependency. Independently
authenticate the retained manifest using the exact commands in
`logs/retained-authentication-commands.txt`, then recheck receipt/result hashes
and mtimes. A failure before receipt publication remains a diagnostic failure;
do not manufacture a Native authentication manifest or rerun its producers.

The next independent sequential gate is the complete full-real proof.
`logs/full-real-command.txt` contains its exact reader/argv/environment and
output paths; create only its absent `full-real-tmp/` before launch and retain
new process identity. Its original 16-pair/34-producer positive, source and
attribution tamper controls, compiler denial, runtime denials and zero-producer
reservation rejection remain required. `SM_NATIVE_PERFORMANCE_ONLY` is absent.
The default application reader command and matching helpers are preserved in
`logs/application-command.txt`; the known frozen-source catalog/planning
diagnostic must be retained without changing this assembly. No additional gate
has been launched. The integrator decides whether to monitor directly or
delegate a later gate, and owns merges, worktree/target retirement and final
post-retirement admission.

## Gate configuration

### Checked interrupted-process recovery

At 17:02 UTC root found neither original timer `9353` nor reader `9356` nor
their producers alive. The boot ID was unchanged, no OOM diagnostic appeared,
and the original resource file was empty: there is no observed final exit code.
The inherited exec session `11384` was unavailable to root. The cause is unknown;
the original stdout/resource files remain untouched. PLM and projectile-script
receipts existed, while collision retained its original intent and completed
producer records. Observation/process/boot/seal files are
`logs/recipes-interruption-*`.

Root resumed the exact twelve-recipe command against the same source and output
through the existing journal. New root exec session: `60673`; timer `76138`,
reader `76139`. Outputs are `logs/recipes-resume-01.log` and
`logs/recipes-resume-01.time`. This is same-attempt recovery: completed operations
are authenticated, and the retained intent preserves the original deadline.
No source, requirement, producer limit, operation identity or recipe domain was
changed. Collision subsequently published its Native receipt. Full gate outcome,
remaining controls, cleanup and independent retained authentication are pending.

The retained reader selected exactly one ignored test for each required gate;
selection stdout is retained in `logs/*-selected-tests.log`. Complete catalog
preparation passed (1 test, 0 failed, 186 filtered out): 177 catalog families /
150,204 declared Fixture pairs, plus the complete 360-pair direct-animation
auxiliary. It took 87.23 seconds in libtest and 87.32 seconds externally. These
are preparation results; no compiler or Fixture was launched. The host was
contended, including the matching helper build; this is no throughput claim.

Exact command/time and stdout are retained in `logs/catalog.time` and
`logs/catalog.log`. `logs/catalog-counts.txt` summarizes the actual records;
`logs/catalog-records.sha256` seals all catalog records and
`logs/catalog-generated-source.sha256` seals the generated PLM manifest and
synthetic ROM in `catalog-source/.scratch/translation-native/plm-instructions/`.
Catalog summary SHA-256:
`c441fbc043d0002b0a9963813bab87c3f0de0d67453c407a4177dc61a4014124`.
Its completed `catalog/` and separate `catalog-tmp/` still need both cleanup
passes when the matching CLI build completes.

The twelve families, in order, are `plm-spawn`, `projectile-script-leaves`,
`collision`, `power_bomb`, `game-host-projectile`, `game-host-room-scroll`,
`ridley-subsystem-captured`, `plm-instructions`, `pause-direct-animation`,
`animated-tile-frames`, `animated-tile-spawns`, `ceres-pad`.

Recipes retain four Fixture jobs in the shared four-worker pool, 600-second
per-family gates, 64-GiB aggregate output, 180-second compiler limits, and
five-second / 16-MiB Fixture limits. Preparation uses the original 600-second /
8-GiB contexts. The full-real proof retains its 180-second gate, 1-GiB output,
60-second compilers and five-second / 512-KiB Fixtures. Reconciliation and
independent Native authentication keep their original 60-second bounds.

Proof roots will be `catalog/`, `recipes/` and `full-real/` under this new parent.
Each explicit ignored gate must select exactly one test. Retain all source,
producer identities/results, indexed operations, receipts and failure diagnostics.
Check unchanged receipt/result hashes and mtimes on retry and authentication.

## Cleanup and remaining work

The current outcomes above record completed cleanup and remaining writers.
Wait for every relevant writer before cleaning its completed or failed root.
Use the separately retained current CLI:

```sh
<proof-parent>/tools/sm-pipeline cleanup --root <finished-root> --apply
<proof-parent>/tools/sm-pipeline cleanup --root <finished-root> --deduplicate --apply
```

Retain exact stdout, bytes reclaimed, free space and source/reader/private-input
dependencies. Independently authenticate Native receipts afterward using the
retained test reader's exact `retained_native_proofs_authenticate_without_execution`
test and the proof's `retained-authentication.toml`. The integrator owns worktree/
target retirement and the final post-retirement admission. A concrete gate failure
is reported immediately; it is not silently retried or treated as acceptance.
