# Original acceptance implementation checkpoint

Session: `01a139b9-f1a2-7d30-b65f-f0741bb5d555`
Branch: `affected-family-assignment`
Review baseline: `808efe2ee473e633a42bbf330f3872f5a0499df9`
Ticket: [10 — Execute the original production acceptance contracts](../issues/10-original-acceptance-adapters.md)

The current source adds confined Cargo/native/compact adapters, a distinct
original composite contract, reviewed Rust preparation and frozen ordered
continuation. The original production registry, old stopped attempt, four held
claims and three Submitted source results are unchanged. No original scope has
completed through the new installed command at this checkpoint.

## Latest continuation checkpoint

2026-10-14 performance priority: the user asked for serious speed improvements
before further sealed evidence work, including measured parallel execution.
The performance changes are integrated on `affected-family-assignment` through
`3c2e9e3`. Native commits are `cfdbc6e`, `00bf076`; confinement commits are
`326c2aa`, `1198963`, `3c2e9e3`. The starting performance baseline was
`528581b246e6ace9748e711743482f72e83cf9e3`.

The complete original 16-pair Native control test fell from 43.11 to 25.56
seconds with both changes; its positive proof/authentication/replay portion
fell from 17.662 to 10.682 seconds and fixture wall time from 5.480 to 0.843
seconds. Fixed-helper 1/2/4-worker fixture spans were 3.103/1.805/1.212 seconds:
2.56x throughput at four workers within the unchanged shared process capacity.
Continuation order, failure joins and aggregate resource budgets are preserved.
Final same-reader sandbox medians show 7.24x regular-write and 4.39x pipe
speedups. These measured small-family and focused gains do not establish
whole-catalog throughput; see [performance results](performance-results.md).

All 31 focused pipeline tests, 12 operation controls, 16 sandbox runtime
controls, eight original sandbox guards and 32 stress repetitions passed.
The complete actual-C control proof passed with the combined Native/final V6
helper. Merged `cargo check -p pipeline --bin sm-pipeline --locked --offline
-j 1` passed in 58.51 seconds. Both independent source-review axes found zero
issues; [the review](performance-review-20261014.md) records their exact frozen
source scopes and limits. All implementation and review workers are finished.

Concurrent private equipment source commit `847d76f` implements all 29 routines,
passes all 13 independent Rust controls and all 427 declared actual-C Fixture
pairs. Its complete Native proof, replay and post-cleanup authentication took
97.78 seconds using two fixture workers. Both source-review axes passed; the
source and manifest remained unchanged. The shared inventory now contains the
reviewed `verified` observation as well as the earlier `translated` and
`verified` observations. This is private bootstrap and confined verification;
managed integration, Live replay and original-ROM parity remain pending. See
[the equipment handoff](../../equipment-interaction-concurrent-20261014/verification-handoff.md).

Both integrated performance worktrees were removed after retaining their exact
source dependencies. Selected archives are `native/source-final-dfc06db` and
`confinement/source-final-ccc04bc` under
`/home/jim/sm-decomp-performance-20261014`. Seven Native proof sets and the
equipment proof (856 producers, 12,569 references) authenticated again after
retirement. All retained source/tool/timing and historical manifests passed
their integrity checks. Normal and deduplicating cleanup succeeded; identified
removals plus deduplication total 2,767,081,472 bytes including private equipment
work. Last observed available space was 187,063,205,888 bytes on the shared
filesystem. The existing main-checkout Cargo cache and required proof readers,
helpers, private inputs and source snapshots remain retained.

The final helper is
`/home/jim/sm-decomp-performance-20261014/confinement/tools/sm-translation-sandbox-fd-table-v6-opt1`,
SHA-256 `8f825e97e00ce534878131d6838411aee99576f7e0e2a088e1501b0c67cda0b0`.
Rebuilding pipeline alone does not select or install it. The existing production
Installation, registry, fence, claims and stopped attempt remain unchanged.
No full catalog run or new Original attempt was started.

The separate architecture-performance session retains five completed candidate
worktrees and its combined source `bc5c1cf9ba2965ef6513273deb4880a93d770cc9`.
Its [composition handoff](/home/jim/sm-decomp-architecture-20261014/03-builds/.scratch/architecture-performance-20261014/composition.md)
records 32 passing controls and two actual-C results authenticated after cleanup.
Those candidates have not been composed with this Native implementation.
Preserve our strict reservations/waves and their `build_cache.rs` source-list
registrations during later composition. The five requested worktrees belong
to that session and remain separate.

Ticket 10 remains claimed. The next performance work is largest-family
reference/finalization scaling, shared scheduling and build reuse, with finite
whole-catalog budgets. The Native reference inventory still has a 65,536 ceiling.
The unfinished Original Offline reuse changes are preserved in isolated commit
`20b0316f87fe31146e652d262c39b28a946dad33` on
`ticket10-original-reuse-parked`, at
`/home/jim/sm-decomp-ticket10-work/original-reuse`; they are not integrated or
tested and still need `translation_execution::journaled_candidate_gate`.
Installed Original acceptance and ordered continuation/recovery remain pending.
Do not repeat the old 600-second Cargo prerequisite unchanged. JSON format
maintenance remains deferred.

## Historical checkpoints

Prior `/tmp/sm-ticket10-*` proof, reader, target and worktree directories were
absent after the environment resumed. The earlier records below are historical
results, not currently authenticated evidence. Do not recreate those paths and
pretend they contain the original results. New measurements and frozen tools
use `/home/jim/sm-decomp-performance-20261014`; the two integrated performance
worktrees formerly under `/home/jim/sm-decomp-ticket10-work` are retired.

Integrated source through `e0b7303` includes the completed Original prerequisite
regression, compact verdict-only recovery, native operation indexing, finite
per-pass discovery and shared callback/PLM argument builders. Ticket 10 is still
claimed. No Original production Installation or new production attempt has been
created. The production claims, registry, source results and old stopped attempt
remain unchanged. The sections below this checkpoint also retain earlier proof
history; they are not a claim that the final production run has completed.

The current catalog was fully prepared in preview 03: 177 families and 150,204
Fixture pairs, plus the pause composite's separate 360 direct-animation pairs.
The preview is `/tmp/sm-ticket10-preparation-preview03`; its continuation config
SHA-256 is `09ca2142197bc866abdf0238d50ba17d4ab78a389b7d7e6c2e4bc60accf6beed`.
It uses reviewed source `30bef71` and must not be installed after later fixes.
The missing production C archive was copied from the already verified cache;
its exact pin remains
`69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`.
Both cleanup passes completed on all three preparation previews, reclaiming
zero bytes. Source, config, failures and unchanged old production status remain
retained under `/tmp/sm-ticket10-root-verification`.

The retained `30bef71` controller is
`/tmp/sm-ticket10-root-verification/bin/sm-pipeline-30bef71-opt1`, SHA-256
`8a916402c9659b5f65cc39742744421fdce3755bfbf774948f62308a5bc7e07a`.
It was built with pipeline optimization level 1 in 9m40s. Earlier installed and
retained readers are preserved. No preview controller overwrites production.

The native receipt size regression failed at 11,685,590 bytes for 2,048 storage
declarations, then passed with sealed bounded chunks in 0.23 seconds. This is a
storage test, not producer or Offline acceptance. Commit `17cfde1` introduces a
small index bound to exact context, prepared identity and candidate identity;
historical inline receipts retain their encoding and evidence-byte semantics.
Metadata is reserved before launch, counted in evidence and included in retention.
The first green reader and exact source are retained in `native-index-v1/`, reader
SHA-256 `46072e59633c08ba334b59d5ab596c3c877feeb33e57617c6990bad10c4e0da7`.
Supplementary retention/tamper controls are being checked. Larger declared tree,
reference, trusted-read and aggregate catalog limits still require validation.

The complete pause native proof passed all 247 primary and 360 direct-animation
actual-C pairs, with replay and post-cleanup authentication. Captured Ridley
completed 670 family pairs but native receipt finalization hit the old discovery
counter ceiling. Its repaired read-only audit authenticates 1,343 producers in
27.15 seconds and creates no native acceptance. Existing material callback proofs
also authenticate after the shared argument-builder change. Full details and
immutable source/readers remain in the native handoff and proof directories.

The two-axis review against `808efe2…30bef71` is recorded in `review-30bef71.md`.
The compact recovery defect is fixed in root commits `cb85b82`, `32b2082` and
`fdc7053`; the final seven focused controls pass. Their five retained green
fixtures authenticate after both cleanup passes and removal of the new recovery
worktree/target. The exact regression first failed with `compact immutable output
changed`, then recovered the same expired stopped verdict without a producer.
Its proof root is `/tmp/sm-ticket10-compact-recovery-proof-20261014`.

The public Original prerequisite regression is integrated as `094db08`. It keeps
the real pause paths, 247-case primary declaration and all ten Rust declarations,
omits auxiliary direct animation and observes that exact refusal through public
run/inspect/resume. It preserves source, claim, Installation, evidence extents and
file modification times. Its retained replay passed after worktree retirement in
23.08 seconds. Keep `/tmp/sm-complete-supervisor-1300786-0`, the retained failed
setup fixture, and the reader under
`/tmp/sm-ticket10-original-regression-verification/bin/`.

This continuation retired the integrated Original regression worktree
(448,421,888 allocated bytes), the compact recovery target (3,021,004,800 bytes)
and its worktree (448,434,176 bytes): 3,917,860,864 bytes reclaimed. The Original
worker had separately retired its 1,856,675,840-byte target. Required proof roots,
source snapshots and frozen readers remain. The post-retirement authentication
logs are `original-authentication-after-retirement.*` and
`compact-recovery-retired-{green,initial}.*`. Available space was
166,723,866,624 bytes during the following guarded-index build.

The native helper v4 remains the last validated helper. A later candidate needs
validated ptrace birth-event and descriptor-exit handling before another compact
Cargo prerequisite run. All original adversarial guards must pass. No replacement
helper or dependency-cache success has been assumed.

Current scale measurements do not establish the preview's 64 GiB / 3,600-second
aggregate Offline budget as sufficient. The source/spec review distinguishes
these ticket-10 aggregate choices from the original per-command obligations.
Retain every original control and family; review a finite whole-catalog budget
and authenticated native reuse between identical candidate/integrated closures.
Original Cargo controls must execute on both identities. Final suite, review of
repairs/storage/scale, real installed execution, ordered continuation/recovery,
retained authentication and final cleanup remain outstanding.

Worker checkpoints through the final compact proof and Cargo `e67aed2` are
integrated. The separately sealed
Cargo/rustc toolchain is independent of the native compiler. Imported original
integration reserves only the exact original job and Submitted result under the
current fenced integrator; ordinary admission still requires matching original
coordinator lineage. Terminal continuation views also bind their completion
envelope. Completed original publications now seal their final controller
projections; a terminal transition must preserve all other reviewed metadata,
the exact ledger inventory and unrelated effort bytes. The first transition
cannot reseal a changed reviewed metadata boundary. The public status regression
was observed failing without this check, then passing with it restored.

Original Cargo execution uses one compiler job within the shared four-worker
pool. Explicit multi-job requests reserve all requested slots atomically and
release partial acquisitions while waiting; a concurrency/exclusion/recovery
control passes. This preserves original command deadlines and avoids the
measured observer overhead of four simultaneous compiler jobs.

Verification logs are under `/tmp/sm-ticket10-root-verification` and earlier
`.scratch/translation-scope-execution/ticket10-verification`. The combined
`cargo check`, all four continuation controls, the shared worker-budget control
and all fourteen ordinary migration controls pass (two historical opt-in
diagnoses remain ignored). Logs include `continuation-resealed-red2.*`,
`continuation-green.*`, `worker-budget.*`, `migration-after-projection.*` and
`check-continuation.*`.

The obsolete full-opt3 release build was terminated before installing a
controller. The opt-level 1 controller built successfully in 5m15s and is
retained at `/tmp/sm-ticket10-root-verification/bin/sm-pipeline-2d1caa9-opt1`,
SHA-256 `a6f771451c77f0bd26e865491ae9a723c0f87c8df78bec7dfb7c899d39b5fd49`.
Its exact build command/profile is in `controller-2d1caa9-build.md` beside it.
The original Rust builder was exercised in
`/tmp/sm-ticket10-preparation-preview01`. It stopped on captured-Ridley's
dependency-count ceiling before any production marker/attempt. Both cleanup
passes succeeded with zero bytes reclaimed. Full source/intent/config remain
retained. Family/scope error context was added for subsequent preparation.
Final suite, two-axis code review, real installed production acceptance and
final cleanup remain outstanding.

Reviewed optimized helper:
`/tmp/sm-ticket10-native-proofs/confinement-v4/tools/sm-translation-sandbox`,
SHA-256 `1ec74e0bcc1024ae460d74b47148114526bf6743e2cc4a87302f9ecf551ab6d9`.
All eight original confinement guards and seven focused runtime controls passed.
It also distinguishes read-only directory opens from the complete `O_TMPFILE`
flag combination. Earlier helpers and failed proofs remain immutable. The
native worker still owns large-family reservation reconciliation, the captured
Ridley input profile (5,371 dependencies and one 71,595,494-byte original event
log), and bounded transient descriptor inspection. Default limits for other
recipes stay unchanged.

The host's `real-09` Cargo proof passes all 31 original controls on candidate
and integrated source (41.80/48.10 seconds), with unchanged replay, confinement
denial controls, both cleanup passes and post-cleanup authentication. Pause
likewise passes all 10 original Rust controls on both identities. These proofs
grant no complete original composite or gameplay Offline acceptance.

Native material proofs pass all 7 projectile and all 3 room-scroll actual-C pairs;
each room-scroll fixture preserves 17 ordered effects. Replay and post-cleanup
authentication pass. The current conservative catalog has 177 families;
captured-Ridley registration is integrated, with its larger input bound pending.

Compact proofs 02–04 authenticated all five historical captures, then retained
the exact frame-2240 experiment's 60-second native timeout. Unchanged replay
created no new capture. Both cleanup passes and post-cleanup authentication
pass. This is failure/recovery evidence, not diagnostic, Live or original-ROM
acceptance. Proof 05 with v4 completed native capture/packing through frame 2240,
then C compilation exhausted its 27.339-second remainder of the original whole
60-second deadline. It retained the 632,947-byte capture and exact failure;
unchanged replay, both cleanup passes and post-cleanup authentication passed.
No fresh candidate/control comparison ran. The compact handoff holds the exact
artifact hashes and timing records. Compact Cargo external-dependency
preparation is still being exercised; no original compact Cargo acceptance has
passed yet.

Preserve the unrelated assignment edit at
`.scratch/decomp-parallel/assignments/verify-e3b0c44298fc1c14.md` and pre-existing
untracked retained outputs. They are outside this ticket's writable ownership.
Keep all old source/readers/private input closures, especially the architecture
worktree and original compact-controller worktree. Do not retire new workers or
their build targets until their proof dependencies have been retained and
authenticated. Do not resolve this ticket before the required installed-command
progress, continuation/recovery and cleanup are established.
