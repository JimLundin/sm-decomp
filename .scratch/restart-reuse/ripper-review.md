# Independent review: Ripper initialization and patrol

Reviewer `01a145c8-9bc8-78a2-b0f9-8519ad5f75f5`,2026-10-16.
Disposition: **retained controls and production composition support the bounded
four-root source/actual-C/integration scope.** No live Ripper gameplay,
whole lifecycle or original-ROM parity is established.

Evidence root `/home/jim/sm-decomp-restart/restart/out/ripper/`.
Native source `bda3bb86a31e69df1521eec94ec823fac3623f2c`, actor control
`4263fdacba7d650505a8b147ec6f6ba599caf1bb`, final composition
`cdf130f8e95fd755e536e4c5f57006afd90240b0`. All three candidate archive/executable
manifests pass after worker retirement. Composition uses the unchanged native
ccdee5df recipe; no new native generation or control replay was needed.

## Reuse and bounded actual-C scope

Own JetPowerRipper_Init A2E318, JetPowerRipper_Main A2E353, Ripper_Init A2E49F
and Ripper_Main A2E4DA. Shot, frozen, contact/death/draw, scheduler/interpreter
and horizontal terrain remain native. This is initialization/patrol ownership.

The completed preparation and native-admission audit are reused. Main's <=63
guard is narrower than the pinned65-row runtime table: row64 is admitted here,
with `(uint16)(8*parameter1)` aliases, nonzero parameter2 direction, C/D velocity
fields and collision reload from stored row rather than negation. The audit
authenticated all520 runtime-table bytes and the passive inactive layered
Waver/Ripper wrappers. Current native c/hooks/prepare hashes match that audit;
this review does not repeat table extraction or terrain research.

Independent comparison passes for all **53 full-WRAM/effect pairs**:
46shared Ripper cases, six chain steps and one Waver coexistence pair. The52
Ripper observer pairs and recorded WRAM/effect hashes also match. The Waver
coexistence pair has no observer files or pair hash fields; none are inferred.
Ripper Rust completions aggregate JetInit11/JetMain15/Init11/Main15, with30
horizontal services per owner. Raw records confirm unchanged Waver counters
through every Ripper pair and unchanged Ripper counters through Waver.

Both three-step chains are Init→collision→return-air. All four predecessor
output/input edges independently compare byte-identical; observed collision
results compare and the harness asserts1then0. The comparator checks actual
CallEnemyAi dispatch, exact owner/service deltas, full observer snapshots and
unchanged Y/subY. Clear packedX displacement equals captured C/D amount once,
catching double movement through layered wrappers. Spike allocation/full-pool
cases assert actual D094 effects and level-word mutation/preservation.

Waver coexistence uses one horizontal/sine/vertical sequence and one horizontal
displacement. Ripper remains installed throughout; Waver's hook is disabled for
its C comparator and installed for Rust. Its both-hooks metadata describes the
Rust case, not the disabled-target comparator.

Thirteen admissions comprise10 unsupported-state immutable pre-C rejections,
short-ROM519/520bytes, short-WRAM and legal unchanged refusal with actual-C
fallback. Nineteen distinct fatal receipts authenticate stdout/stderr and
verified exit2/no timeout; total1918ms. Invalidcollision explicitly injects
trait Ok(2) to exercise Rust fatal propagation: native terrain did not return2.
Other controls cover transport/slot/amount/state/generation, duplicate/outside-
Main requests, refusal/error/panic, recursion, reciprocal actor guards and
service-observer failure. Failures after accepted service do not retry C.

Retained logs show10semantic tests (0.63s compile/0.10s run),20runtime tests
(4.71/0.06s,3filtered), workspaceClippy8.29s, actor-controlClippy2.12s and build
2.74s. These are individual receipt timings, not complete delivery time.

## Six ordinary production-composition calls

The control calls actual CallEnemyAi for JetMain A2E353, RipperMain A2E4DA and
WaverMain A3874C through the shared production installers. All11owners remain
installed for Rust; C comparators disable only the target owner. Jet and ordinary
Ripper share one owner. Complete typed counter equality against expected deltas
checks every other owner, including player and persistence.

All three C/Rust input-WRAM/input-SRAM, final-WRAM/final-SRAM, effects and output
pairs independently match. All six input/dependency seals pass. Each target
records one Rust completion and one C invocation, with no refusal, unavailable
service or invalid request. Ripper horizontal requests/completions total4 across
both variants and owners; Waver service totals are2/2/2 across both owners.
These totals include C. No player or persistence routine is invoked, and all
unrelated counters remain unchanged. All six calls preserve seeded SRAM and
create no saves directory. Registered but unexercised owners gain no reach claim.

The first compositionClippy report rejected enum names sharing Main suffix;
root's final candidate only renamed those identifiers. Original worker source
and final root source are separately retained, as are the failed and passing
logs. Final Clippy/build record0.15/0.34seconds. The resulting check exercises
the changed actor installer/horizontal boundary without replaying accepted27
player/Save/Load comparisons or old body/fatal matrices.

## Identities and cleanup

| Artifact | SHA-256 |
| --- | --- |
| Native library,ccdee5df recipe | `88113830b6b6a9dccf0d69c7e107c72f320d68e249d3f6efeb41ddb278f53451` |
| Actor-control executable | `e61bcdc66d4a8c9fbe2a65563f728e758013da1c3218e8f384f9a35671e2aaaa` |
| Actor-control result | `783549b8459b120700506d8838cc477f5dcea687639797dd3f856a8e15d3014a` |
| Runtime table | `b8c7ce12514576f554227bdf8a4e22d00bee41cd4b7b3aeea5dac2a594ecbcfb` |
| Composition executable | `d9ce7806abff20be87eba24be328cb9b075a5723692cc7944c97f37e344df664` |
| Composition result | `20f8e906a54a7e497921053acf1ec6627d00863df9ce2edb683bd5907542e9d2` |
| Composition final counters | `015362d658077b76d1ae64e35eed3509fabb355a52cae511148ab8207f053757` |

Controls and whole-root ordinary/deduplicating cleanup receipts each report
zero disposable bytes. Rust/native retirement reclaimed445628416bytes; retiring
the composition worker reclaimed222961664bytes: **668590080bytes total**.
Remaining source allocations73728+106496+28672bytes; available disk96104493056.
All three workers are absent from Git registration with no .git or target.
All15 original worker-source files match retained original paths. The final
root composition snapshot separately matches frozen candidate source, preserving
the identifier-only lint fix without rewriting the original worker evidence.

Retain exact source archives/executables, pinned C/library/manifest, private ROM,
raw pairs/process logs, composition files and the accepted Waver input dependency.
Root warm target and independently owned successors remain outside disposal.
Reviewer work is Markdown, source/receipt inspection, comparisons and hashes;
no build, test, native, game or reference producer was launched. Reviewer cleanup
is0bytes. Root owns publication and enclosing sealing after copying this immutable
report; no unpublished final manifest is claimed authenticated here.
