# Review room-transition results, cost and retention

Status: resolved
Blocked by: 06
Triage: ready-for-agent
Owner: session `01a14300-1d4f-77e1-b198-a47e8d902bb7`
Started: 2026-10-16 11:19 UTC

## Scope and ownership

Continue independent monitoring of restart ticket 09, claimed at `764a8e5`,
source baseline `b80a083b9bb2d473a29b452db38bbb44ea24b765`. The implementer
owns source, builds, all short-run execution, publication and cleanup. It must
not wait for this reviewer's acknowledgment before executing frozen checks.
Roadmap/presentation retain ABI, route and predicate review.

This session owns only local notes under `.scratch/restart-verification/`.
Read completed frozen results under `restart/out/room-transition/` when they
exist; no output directory, game run, build, source edit or new gate is assigned.
Do not repeat the ABI/route audit. Diagnose only observed failures or cost
regressions; report concrete findings rather than unchanged status.
The integrator subsequently assigned the final read-only result report at
`restart/out/room-transition/evidence/verifier/independent-review.md`; no run or
build ownership was transferred.

## Review boundary

- The whole 27-routine coordinator is translated; three scheduled roots,
  continuation/phase ordering and eight private background commands are owned.
  The 53 named foreign services and legitimate child Rust hooks remain explicit
  dependencies, not evidence of whole-game Rust ownership.
- Existing 9,380-step inputs must preserve projectile/scrolling/door behavior
  while demonstrating both transitions, reached background commands, pending/
  resumed loading, actual IRQ completion and resumed input. Unreached directions,
  elevators/message boxes/alternate commands remain explicit coverage limits.
- Inspect one changed-native-recipe C reference followed by reuse for candidate,
  CLI and live fault. Review complete WRAM/external/effects equality and actual
  ownership. Watch whether warm build plus comparison stays within 30 seconds
  and documentation-only changes avoid a game process.
- Check whether published snapshot/update-output reuse avoids the redundant
  roughly 12-second status pass. This is process observation, not a new gate.
- Review final measured cleanup, worker retirement, retained dependencies and
  post-cleanup authentication using the integrator's existing outputs.

## Observations

Ticket and ownership read; implementation is in progress. No frozen run exists
yet. This review has no execution reservation and introduces no blocker.

Root reports runner/material wiring at `07eb02f`, before any verification run.
The checks will use the reached order of 14 phases, exact background-command
tuples/copies, actual IRQ-set NMI resumption, once-per-room service counts and
input/finish flags. Workers still own implementation and native controls. Root
will freeze and execute directly after integration, without executor handoff.

### First actual reference and candidate at `61ec98b`

- Root's first C reference passed all 9,380 steps/material checks in 17.193
  seconds. Its retained diagnostic also compares all WRAM and passive external
  hashes to the earlier door reference; only the intentional observer effects
  changed. No second C run was needed for this diagnostic.
- Continuity reports freeze at 11:36:25 UTC and reference receipt at 11:36:42.84,
  approximately 17.84 seconds versus 17.19 execution. Direct launch avoided the
  prior 345-second executor handoff wait.
- First Rust dev run stopped in 16.936 seconds including a 0.545-second forced
  rebuild: `room transition requires the complete native ROM view`. Native
  fatal explicitly prohibited C retry; no result or C/Rust parity is established.
  This is a contract guard failure, not an observed gameplay-output mismatch.
  Root owns the targeted fix; this reviewer requested no replay or ABI audit.
- Reference transitions first wait at 8750/9059, resume after IRQ at 8813/9122,
  return to game state 8 at 8831/9140 and resume movement at 8832/9141. These
  are reference observations only until a successful Rust comparison.

### Corrected candidate, normal CLI and fault results

- `9ee6419` corrects the cartridge/native ROM size contract without changing
  the C host, native package or input. dev-02 passes all 9,380 records and
  material checks in 15.356 seconds, including a 0.082-second cached build check.
  The earlier 0.545-second forced rebuild belongs to failed dev-01; the report
  keeps those costs distinct rather than requesting an unnecessary rerun.
- Scheduled roots [2,0,213] and all 27-routine observer counts match the C
  reference with zero owned C/refusal. State10 is reached inside state9 rather
  than through a separate scheduled entry. Twenty-one routines execute; pending/
  resumed counts are 126 and nested Rust child/service counts balance.
- CLI passes in 10.523 seconds with zero CPU fallback and byte-identical final
  WRAM/PNG. The live false-NMI-completion control fails at step 8,750, external
  state, in 13.865 seconds. docs-01 reuses the result in 0.189 seconds with no
  game process after substantive documentation edits.
- Final independent report written to the assigned verifier evidence path.
  This session ran no game/build/test for ticket09; root retains cleanup and
  publication. Awaiting the completed retention handoff.

### Progress-reporting shortcut was not applied at closeout

Observed tool metadata in integrator turn
`01a1446c-79d6-79d3-b21f-2121f4eceaaa`: command
`exec-5c260bcb-19a0-4c6e-8a3e-cb851270779f` ran a standalone `progress status`
before final publication and took 22.608 seconds. Required verified/integrated
updates then took 13.982 and 14.360 seconds. The earlier agreed snapshot/update-
output reuse therefore did not remove the redundant status in this closeout.
Reported this concrete process regression to integrator and continuity owner;
requested no new test or optimization work, only avoiding another redundant
status invocation. Gameplay evidence and completed cleanup are unaffected.

## Completion evidence

Completed at `cee31cf` on restart/playable-baseline. The integrator publishes
verified/integrated/live-exercised stages independently, with verified/live
scope limited to the 21 reached routines. Complete source contains all 27;
unreached branches and foreign services remain explicit limits.

Read the completed cleanup receipt: both worker worktrees retired and all
utility passes succeeded. Gross apparent removal 761,488,063 bytes; available
space 105,764,442,112 bytes; primary 1,486,857,397-byte warm target retained.
All 511 pre-cleanup hashes passed after retirement. Native/trace/cached-result
reuse then passed in 0.345 seconds with zero game processes. Final manifest
and authentication log each contain 542 entries; all log entries are OK.
The independent report is included unchanged in that manifest.

The integrator acknowledged the redundant 22.608-second status scan and
confirmed no further standalone scan; required publication output is sufficient.
No profiling or optimization task was introduced as a gameplay prerequisite.

This reviewer performed only completed-artifact/source-contract inspection,
read-only final byte comparison and report writing: no game/build/test run,
worktree or recurring process for ticket09. No verifier cleanup remains. Root
completed final retained-evidence cleanup; previous successful and failed
results remain intact.
