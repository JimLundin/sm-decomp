# Reference live replay soft-reset crash
Status: resolved
Blocked by: None
Triage: ready-for-agent

The whole supplied movie cannot currently exercise bank86 live leaves: the
reference C process crashes at frame43188 before any mixed process starts.
Handle the intended soft-reset sentinel at the proper runtime boundary and
revalidate movie progression; an out-of-range guard alone would mask gameplay.

## Diagnosis

Exact retained binary:
`out/port/projectile-movement-live/20261011T060355.160063Z/live`.
Whole129712frame replay failed with SIGSEGV(-11). Two independent debugger
replays of the first45000frames reproduce the same null indirect call:
RunOneFrameOfGameInner -> RunOneFrameOfGame -> RtlRunFrameCompare.
At fault port_frame43188, game_state(WRAM0998)=ffff and coroutine_state_0
(WRAM0611)=09. Caller indexes kGameStateFuncs[65535], a45entrytable; RIP0.
Source SoftReset(sm_81.c36) deliberately writesffff; GameOverMenu_7 invokes it
when fade-out completes. RunOneFrameOfGameInner has no sentinel handling.
Immediate cause is an unhandled reset sentinel; proper reset ownership and
why the movie reached game-over remain to be investigated.

Reproduction: in fresh cwd with fixtures/ and saves/, run the retained binary
with arguments `c /home/jim/sm-decomp/rom/sm.sfc
/home/jim/sm-decomp/movies/super-metroid-any-3653.inputs 45000 terminal.bin`.
Debugger adds `-batch -ex run -ex 'bt 10'` and examines g_ram+0998/+0611.
Logs: `out/port/projectile-movement-crash/diagnostic/gdb.log` and
`out/port/projectile-movement-crash/state/gdb.log`.
Identity manifest seals logs, exact binary, ROM and inputs:
`out/port/projectile-movement-crash/identity.json`.
Gdb9.2 was downloaded/extracted under/tmp without system installation.

## Cleanup evidence

Finished whole-movie run cleanup utility0bytes; automatic target removal and
47,468,962bytes unsealed archives removed. Diagnostic repeated hardware/state
outputs removed after retaining backtraces/identities:4,706,156,564bytes.
Utility applied/deduplicated diagnostic root0bytes (format unsupported).
Previously authenticated parity-prefix snapshots shared identical inodes,
reclaiming3,828,920,841bytes without changing hashes. Logs/binary identities
validated after cleanup.26GiBavailable; no implementer worktree created.

## Completion evidence

Fixed the missing independent-host soft-reset transition. The pinned comparison
runtime already redirects CPU to80:8462 and coroutine_state_0 to3 forffff.
The independent headless host now applies that same transition before its next
frame. This enters Vector_RESET_Async's explicitly marked soft-reset position,
preserving hardware state, SRAM and imported input timeline; no invalid-index
guard or hardware power cycle was introduced.

Regression exercises the actual Vector_RESET_Async coroutine routing through
its firstAPUupload. Before fix the control fails; after fix it enters position3,
skipslogo/MEMSEL/initialRAMclear and redirectsCPUcorrectly. Normalgame states
preserve their coroutine. Softreset+live-tooltests10pass(1optional skip);
baseline tests2pass; decomp_live tests1pass/1optional skip.

Original45000frame reproduction now completes twice in independent C processes:
`out/port/soft-reset-reference/20261011T061242.225740Z/verdict.json`.
At44000state4 and45000state1, fallback0; repeated WRAM/SRAM, progress and
ordered diagnostics match. This proves the43188crash fixed, without proving
original-ROM movie progression or bank86 Rust exercise. Full129712 C/mixed
replay remains tracked under ticket73 by the integration owner.

Baseline and live drivers now snapshot/hash headless_frame.h alongside the
host. First45000regression ran before automatic header inventory was added;
`header-identity.json` records the manually retained unchanged header digest
and exact binary afterward. Do not treat that manual addition as an earlier
sealed driver contract. Final baseline-driver tests cover updated snapshots.
Live generatedhost compiles local retained headless.c/header, preventing
later edits from changing its captured dependency.

Cleanup utility applied/deduplicated reference root0bytes(formatunsupported).
Shared byte-identical repeat fixtures after comparison, reclaiming52,068,903
bytes. Binary, both terminalstates and332WRAMfixturehashes in each process
reauthenticated after cleanup. Header snapshot retained; no debug logs added
to code or implementerworktree created.25GiBavailable during root's active
fullmovie run. Commit contains hostfix, regression and driverdependencyseals.
