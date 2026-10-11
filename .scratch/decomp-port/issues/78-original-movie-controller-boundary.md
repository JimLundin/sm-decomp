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

## Executed causal first-completion seam

Implemented isolated `causal-polling` candidate restoring real polling after
C NMI and suppressing only the main-entry poll relocated by upstream. Native
150prelude samples execute in exact order during the logo/reset coroutine's
150yielding NMI calls. Its real64KiB reset clear then removes those inputs;
first gameplay completion observes cleared words before sample1030 is presented.
No WRAM snapshots load into this candidate; no coroutine aliases are changed.

Final source-stable320frame evidence:
`out/port/controller-causal-seam/20261011T064410.228683Z/verdict.json`.
All compared SEMANTIC fields match firstnativecompleted gameplay boundary at
frame272/event1025:game_state1/RNG758/heldpressedprevious0. Candidatecompletion
atstep151 precedes its next poll; all150earlier held samples match source order,
and exactlyone postcompletion sample matches originalpoll1030. Separate actual
linked RNG-bit control changes758→759. Existing160offlinepoll fixtures and both
controls stillpass; public3prefix tests pass58.084s; diffcheck passes.

This is actual bounded execution progress beyond ordinal diagnosis, not full
WRAM or video-frame parity. The original boot video timing and later waits
remainunproved; generaldiagnostic replay stillrejects futurepoll assignment and
overallverdict nonaccepting. Ticket78 remainsclaimed.

Cleanup applied/deduplicated both finished causalruns064325/064410:0bytes
reclaimed; unsupported nonacceptingformat retained. Publictest tempdirectories
removed automatically.17,794,060,288bytes available; noimplementerworktree.
Postcleanup authenticated finalbinary/host/driver/basehost/frameheader,allnative
WRAM/events,causalinputs and candidate/controlevents/fullcompletedWRAM/terminal
states/logs against verdict. Retain these dependencies as boundedproof.

## Ten causal completed updates

Extended restored-NMI polling execution to all160captured polls and all10native
completed updates through sourceframe316. Each completion compares its entire
preceding poll prefix and count to originalsource order, rejecting ordinal
shifts even when heldvalues repeat. AllSEMANTIC fields match at everycompletion;
changed-RNG controls compare all10candidate/control updates separately. Explicit
native wait events remain retained/unmapped; noclaim ofhardwareframe/fullWRAM
parity or authentic APUwait timing. Ticket78 remainsclaimed.

Finalsource-stable320frame evidence:
`out/port/controller-causal-updates/20261011T064741.189171Z/verdict.json`.
All10semantic comparisons pass; all160polls execute;3publicprefix tests
pass51.072s; diffcheck passes. Realresetclearing retained, noWRAMinjection.

Cleanup applied/deduplicated both finished064702/064741runs:0bytes reclaimed
(nonacceptingformat unsupported); publictest tempdirectories automatically
removed.17,074,159,616bytes available. Authenticated finalbinary/allhost snapshots
includingframeheader,nativeallWRAM/events,causalinputs,all10candidate/control
completedstates,terminals/events/logs againstverdict aftercleanup. Noimplementer
worktree; retained sealeddependencies prove boundedcomparison only.

## Forty-nine causal updates and hardware-only upload waits

Extended actualoriginal capture to640frames and recorded passive upload entry,
allreal2140..2143reads, normal/short/abortreturn hooks. Realcross-TU
RtlApuUpload entry/returnobservations record2Cuploads. Bootnativeupload168→254
has278147registerreads; music316→355 has140879reads. Both contain0NMI/0controller
polls. Thirdnativeuploadstarts639 and remainsunfinished. These are actual
hardware-only SPC handshake waits; insertingCgameplay/NMIsteps would bewrong.
RealAPUwaitruntime remainsunfinished; no fabricatedwaitframes implemented.

Source-stablefinal640frame evidence:
`out/port/controller-apu-wait/20261011T065804.359904Z/verdict.json`.
Matchingcausalprefix49updates throughnativeframe393. Firstpollprovenancefailure
ordinal50/frame396:199C precedingpolls versus200native.82pairedsemanticmatches
are diagnosticonly beyondthatfailure. Native229completedupdates/C231; explicit
completion_count_match false and overallnonaccepting. Percompletionpollcount/
order stillgate claims; nofuturestate search or WRAMcorrection. Laterframe355
pollstack23→21 belongs todifferentcallsite; offline arithmeticfixtures remain
explicitlyfirst160pre320calls ratherthanrelaxingstackexclusion.

Prefixhost nowusesexisting HeadlessPrepareFrame beforeeachRtlRunFrame, handling
knownSoftResetffffsentinel viaactualresetcoroutine; priorraw640SIGSEGV atstep346
is retained. Finalraw and causalprocesses complete.3public320prefix tests
pass62.582s; diffcheck passes. Ticket78 remainsclaimed actuallaterpoll/wait
alignment unresolved.

Cleanup applied/deduplicated all5finishedAPUwaitruns065135/065323/065458/065630/
065804:0bytes reclaimed (unsupported nonacceptingformat), includingfailedruns.
Publictestdirectories automaticallyremoved.13,201,436,672bytesavailable during
finalrun; noimplementerworktree. Postcleanup authenticated finalbinary/allhost
snapshots includingframeheader,nativeallWRAM/events,causalinputs,allcandidate/
controlcompletedstates/terminals/events/logs againstverdict. RetainpassiveAPU
captureandpriorfailureevidence; nosealedartifactsremoved.

## Restored within-dispatch VBlank poll

Firstmissingpoll is grounded in FileSelectMenu_0_FadeOutConfigGfx calling
WaitUntilEndOfVblankAndClearHdma($88829E). Nativebytes call$8082C5 hardware
4212VBlank predicate wait; pinnedC explicitlyremoves wait. Nativeframe394poll
occurs inside pendingupdate50 beforeframe396completion. Implemented actual
cross-TU helperwrapper executing realNMI/controllerpoll insidependingdispatch
and consumingnextactualsample in order, leavingwaiting_for_nmi untouched.
OriginalDMA/HDMAclear stillruns; nestedNMIobservescontinuation ratherthanfalse
completedupdate. Pollconsumption separatesfromhoststepcount; noextra gameplay
update/noWRAMsnapshot/no coroutinealiaswrites.

Finalsource-stable640capture:
`out/port/controller-vblank-seam/20261011T070303.316475Z/verdict.json`.
Causalprefix improves49→51updates throughnativeframe397. Actualwithin-dispatch
wait hit1; omitted-waitcontrol reproducesoldordinal50failure(C199/native200
priorpolls), independentlyofchangedRNGcontrol. Nextprovenancefailure remains
ordinal52; futurecauseunproved. Overallnonaccepting, video4212timing andAPUwait
runtime stillunproven.3public320tests pass77.772s; diffcheckpasses.

Cleanup apply/dedup bothfinished070132/070303runs0bytes (unsupportedformat).
Prototypeforcedwaitingflag existedonlyfirstintermediate; finalpreservesnative
flagbehavior, evidenceclearlyseparate. Publictesttempdirectories removed.
11,209,625,600bytesavailable whilefinalrunactive. Postcleanupauth finalbinary,
allhostsnapshots/nativeWRAM/events/causalinputs/allcandidate,RNGcontrol,and
omittedwaitcontrolcompletedstates/terminals/events/logs. Retainsealeddependencies;
noimplementerworktree. Ticket78remainsclaimed nextpollseam unfinished.

## Screen-enable wait restores the complete 640-frame causal prefix

The next missing poll came from FileSelectMenu_16 calling native
ScreenOnWaitNMI ($80:8382), which clears force blank then calls WaitForNMI.
Native $80:8338 stores byte 1 to $05B4; the pinned C ScreenOn omits that wait.
The wrapper now executes real ScreenOn, restores that genuine waiting flag,
and runs the real NMI/controller primitive within the pending dispatch. It
consumes the next observed sample and records a continuation, without another
gameplay update, snapshot loading or coroutine writes.

All three pinned ScreenOn callers map to the same native routine: original
JSL $808382 appears at GameOverMenu_1_Init $819260, FileSelectMenu_16 $81A038,
and Kraid_TransferTopHalfToVram $A7C29A. The 640-frame candidate executes the
validated file-select call once. ScreenOff was not changed.

Final source-stable evidence:
`out/port/controller-screen-on-seam/20261011T070726.941312Z/verdict.json`.
All 229 completed updates match semantic fields and exact preceding poll
sequence/count through original frame 639. Candidate/native completion counts
both equal 229. The omitted-screen-wait control independently reproduces the
previous ordinal 52 failure (202 candidate versus 203 native preceding polls),
while retaining the HDMA wait seam. RNG and omitted-HDMA-wait controls also run.
Three public 320-frame tests pass in 83.574 seconds; diff check passes.

The overall verdict remains nonaccepting: hardware-frame timing, full WRAM
parity and the real APU handshake scheduler remain unproved. Ticket 78 stays
claimed. This advances the verified causal frontier from 51 to 229 updates.

Cleanup applied and deduplicated the sole finished screen-on run: zero bytes
reclaimed (unsupported nonaccepting format). Temporary test directories were
removed automatically. Remaining space: 9,310,453,760 bytes. After cleanup,
authenticated the binary, all host snapshots, native WRAM/events, causal inputs,
and all four candidate/control subjects' completed states, terminals, event
logs and process logs against the verdict. No implementer worktree remains;
sealed dependencies are retained.
