# Original movie controller boundary alignment
Status: claimed
Blocked by: None
Triage: ready-for-agent

Raw video-frame inputs diverge from original-ROM update timing before Ceres
frame9000. Resolve the bounded controller seam before claiming original movie
progression. Preserve prelude polls, waits and input edges; no corrective state
injection or later-state matching.

## Existing diagnostic evidence

`docs/DECOMP_PREFIX.md` and
`out/decomp/ticket07/20261010T182149.168780Z` retain native input-register
reads, timeline, full completed states and changed-gameplay control. First
unsupported boundary is original prelude poll frame19, before native main
dispatch. First compared mismatch frame272: original controller words0 vs
C0080, while game_state1/RNG758 match.

Pinned runtime removes original NMI ReadJoypadInputs at8095E1. Replaying every
poll as RtlRunFrame therefore cannot preserve the original schedule. Investigate
actual ReadJoypadInputs entry/return with hardware register samples separately
from completed update/continuation boundaries; avoid silently dropping polls.

Legacy `out/sync/full-any-lsnes.csv` indicates original frame43000 still has
health62/roomCF80 while raw C replay is in gameover. Legacy capture metadata
lacks current provenance seals and is diagnostic only. Current replay evidence
is tracked in ticket73; original-ROM parity remains unproven.

## Verification required

Reproduce bounded320-frame diagnostic. Establish an explicit controller/dispatch
mapping that handles frame19 and frame272 without altering gameplay state.
Require read-only original-ROM observer, sealed assets/harness/binaries, changed
gameplay rejection control and cleanup. Extend through first Ceres divergence
only after bounded mapping is sound.

## Interim causal evidence

Reproduced current320frameoriginal/Ccapture twice. Final retained run:
`out/port/controller-boundary/20261011T063208.149600Z/verdict.json`.
Nativecompleted-event1025 atframe272 precedes actualcontrollerpoll1030(0080),
but derivedCstep151 consumes1030beforeitsfirstcompletion. New sealed
controller-causality ledger and `future_poll_assignments` report this causal
violation explicitly; no future-sample reassignment or state correction occurs.
Bootclear1005 atframe254 severs earlierpoll association. FirstCpoll301 occurs
insidebootcompletion(coroutine2/game_state0) afteritsclear, while150earlier
nativepolls have no Cpoll seam. ActualCinputmath is therefore not implicated
bythefirst0080mismatch; the adapter feeds a future poll.

Regression firstfailed(missingledger), thenpasses sourceordering/prelude/clear
example. Publicthreeprefixtests pass, includingnewcaptureeventassertions and
separatechanged-RNG detection. `git diff --check` passes. Stillclaimed:
fullsoundmapping requires callablebootNMI/controller and resetclear boundaries;
interceptingfirstCpoll withzero alone drops nativeearlypoll effects and would
be unsound. No gameplayexecutionadapter was invented.

Cleanup applied/deduplicatedbothfinishedcontroller-boundaryruns:0bytes(format
nonacceptingunsupported). Authenticatednativeevents,Cbinary,allnative/Ccompleted
WRAMsnapshots and finalcausalityledger aftercleanup. ExistingROMduplicates
alreadyshared/no newbytesreclaimed. Temporarypublictests useautomaticdirectory
cleanup.18GiBavailablewhileintegrationfullmoviewrites; noimplementerworktree.
