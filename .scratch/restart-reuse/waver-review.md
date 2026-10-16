# Independent review: Waver actor and production composition

Reviewer `01a145c8-9bc8-78a2-b0f9-8519ad5f75f5`,2026-10-16.
Disposition: **retained controls and production composition support the bounded
four-definition source/actual-C/integration scope.** No Waver live gameplay,
whole lifecycle or original-ROM parity is demonstrated.

Evidence root `/home/jim/sm-decomp-restart/restart/out/waver/`.
Native source `5931d6cbb07cfd2971f405f5d8e4a2ed88ac48fc`, actor control
`c5e437646bb59e436135a051207d5a4b80ddcbf0`, production composition
`c8887088b6ab7bac46f72ccc9436ba3c2f1a838c`. Candidate archives/executables and
native library authenticate. Later unrelated integration is not this subject.

## Scope, reuse and actual-C controls

Own Waver_Init A386ED, Waver_Main A3874C, Waver_Instr_1 A386E3 and Waver_Func_1
A387FE. Native terrain, sine, scheduler/interpreter, contact/death/draw remain
dependencies. Main's Init/Main/helper expressions are selective reuse; its
searched implementation lacked the event instruction. Preserve nonstandard
SignExtend8(8000)==8000, current-slot versus explicit-k and equal-selector no-op
before lookup. Material's completed finite-admission source audit is reused;
waver.c/hooks/prepare hashes match its recorded identities. No duplicate audit.

All28 retained C/Rust full-WRAM/effect/observer/scalar-output pairs compare equal
and recorded WRAM/effect seals authenticate. They comprise24 shared body pairs,
two inherited Main chains and two ProcessEnemyInstructions pairs. Both chain
inputs independently match their compared Init/Instruction predecessor outputs.
Aggregate Rust completions are Init4/Main14/Instruction4/helper6; native service
requests/completions are horizontal14/sine11/vertical11 in each owner’s pairs.
Three Main cases take horizontal collision without sine/vertical continuation.

The inspected control checks exact owner deltas, full observer snapshots,
ordered horizontal-only or horizontal/sine/vertical requests, root endpoints
and same-jp instruction result. Spike allocation/full-pool cases compare actual
D094 effects and level-word mutation/preservation. Diagnostic roots route through
CallEnemyAi/CallEnemyInstr; helper is a direct fixture, not a scheduler slot.
The two generic interpreter pairs use original ProcessEnemyInstructions and
the actual event86E3/Sleep812F ROM sequence. Both reset k=current; the fixture
name interpreter-instruction-distinct-k identifies a seeded variant, not an
independent distinct-k interpreter guarantee. Direct instruction pairs supply
the explicit-k semantic comparison.

Fourteen admissions comprise12 malformed/unsupported-state pre-C rejections,
one immutable short-ROM error and one legal unchanged callback refusal with
real C fallback. Source checks immutable WRAM, fallback effects and no Rust
completion. These do not assert arbitrary malformed states are safe for C.
All19 distinct fatal controls authenticate raw stdout/stderr, exactexit2 and
no timeout, with1599ms combined elapsed time. They cover transport/state/order,
refusal/error/panic, invalid output/continuation, reentrancy and reciprocal
Waver/player guards. Native failure after accepted work cannot retry C.

Retained tests show15semantic and20runtime passes (3runtime tests filtered),
workspace and final-example Clippy/build success. Semantic compile/run is
0.67/0.46seconds; runtime4.57/0.05seconds; workspaceClippy9.24seconds and actor
control build3.98seconds. These are individual receipts, not total delivery cost.

## Eight ordinary production-composition calls

The frozen composition uses the shared production installers for ten owners.
Each C comparator disables only its target, retaining nine other registrations;
each Rust call retains all ten. Waver calls its ordinary native AI adapter,
player calls Samus_MoveDown(0), and persistence calls SaveToSram/LoadFromSram(0).
This is controlled invocation through production installation, not a live route.

Independent comparisons pass for all four C/Rust input-WRAM/input-SRAM,
final-WRAM/final-SRAM, ordered effects and scalar-output pairs. The source
compares the entire typed counter structure against exact expected deltas,
including all unrelated owners. Retained unrelated-owner counter bytes also
remain identical from beginning to end.

| Target | C calls | Rust completions | Total native service completions |
| --- | ---: | ---: | --- |
| Waver Main | 1 | 1 | horizontal2, sine2, vertical2 |
| Player Down | 1 | 1 | terrain2 |
| Persistence Save | 1 | 1 | disk writes2 |
| Persistence Load | 1 | 1 | none |

Those service totals include C. There are no refusals, unavailable services or
invalid requests. Rust player feedback/terrain branch counts are1 each. Seven
other registered owners are unexercised; unchanged counters establish this
composition boundary, not broader gameplay coverage.

Both Save results independently match all five retained24 reference artifacts;
saved files match final SRAM. Load after deliberate WRAM clearing returns
Loaded(0) and restores declared fields. The source checks original A5 backup
and preservation of files/SRAM by non-persistence calls. This compact check
reuses existing codec evidence without repeating its body/fatal matrix. It does
not prove user file reopen, durability or successful gameplay continuation.
All ten input/dependency seals authenticate, including the exact unchanged27
library and retained25/24 inputs. Composition Clippy/build record1.75/0.60seconds.

## Identities and cleanup

| Artifact | SHA-256 |
| --- | --- |
| Native library,366561ea recipe | `193f1be1aed69c68ac9b130b5d94707d553d24fcbd01cbc15052cb4da72c9911` |
| Actor control executable | `674607fafc852f99d86480fab34bd7d8045dad9cdc911f79d0e080e6b86f3d89` |
| Actor control result | `b3a9a87deea6fa0ef9e7fba48b96bd8a7adfb33613588dc2f023f696d56f9270` |
| Composition executable | `42c487f9f7f1c7a908f53ffd89feee4fa2a86e4fd269b94f7994697b44c63fff` |
| Composition result | `d59a82c7ae067657e161b6452210dc0d3cd2119c499fbbfc07f4aa4e6bb18768` |
| Composition final counters | `671e2c0e1e65fafad4fb3f2bf10fb5d0845963ab790292db153ae2e0c3dd63c9` |

Actor/control and whole-root ordinary/deduplicating cleanup receipts each report
zero disposable bytes. Native retirement reclaimed222617600bytes; Rust retirement
reclaimed222638080bytes: **445255680bytes total**. Remaining original-path source
allocations are106496 and90112bytes; recorded available disk96741961728bytes.
Both workers are absent from Git worktree registration and have no .git or
target. All15 cited source copies match worker-source/. Composition candidate
seals pass after retirement. Preserve the pinned archive/ROM, native source/
library, exact source archives/executables, raw controls/composition/save files,
and shared24/25 references. Ripper28 and the root warm target have separate owners.

Only Markdown, source inspection, retained comparisons and hash checks were
performed by this reviewer. No build, native control, test, game or reference
producer was launched. Reviewer cleanup0bytes. Root owns final stage publication
and the enclosing seal after immutable report copying; no unpublished final
manifest is represented as authenticated here.
