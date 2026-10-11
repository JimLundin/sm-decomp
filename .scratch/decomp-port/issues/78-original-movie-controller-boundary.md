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

## Callable polling seam evidence

Implemented separate offline `poll-fixtures` mode in
`tools/decomp/prefix_host.c`: real pinned `ReadJoypadInputs` executes against
original controller-entry WRAM and actual captured $4218/$4219 register samples,
without a main/NMI/boot step. Observer now captures full entry/return states and
second-controller register reads; nonzero/incomplete second-controller backing
rejects. Movie candidate still executes ordinary reset with no fixture loading.

Retained final320frame run:
`out/port/controller-poll-primitive/20261011T064009.696175Z/verdict.json`.
All160 polls match all gameplay WRAM, including frame19 prelude.151 polls differ
only in one native CPU return-address stack byte, independently validated per
case as sole stack delta$94→$16;9 match full WRAM exactly. C must preserve$94
at the exact excluded byte; other stack and gameplay bytes remain compared.
FullWRAM equality is explicitly false overall. Sealed per-case source fixtures
record exact offsets. Hardware A-bit and unrelated room-byte controls both
reject independently; changed-RNG replay control and future-poll assignment
rejection remain present. Public3prefix tests pass; py_compile and diffcheck pass.

Ticket remainsclaimed: callablepoll foundation is proven, but authentic boot
clear and controller/dispatch scheduling are still unfinished; original movie
parity remainsunproven. No futurepoll shift or corrective movie state injection.

Cleanup applied and deduplicated all five finished retained primitive runs
(063625,063719,063820,063848,064009):0bytes reclaimed because this nonaccepting format
is unsupported by snapshotdedup. Failed temporary publictests automatically
removed directories. Retain native states, sealed binary/source/cases/logs and
original observer evidence; no implementer worktree.18,243,407,872bytes available.
Postcleanup authenticated finalbinary,host/driver/frameheader snapshots,pollcases,all native
WRAM states/events,ledger/timeline and all primitive/control logs against verdict.
