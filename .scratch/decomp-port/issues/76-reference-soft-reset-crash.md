# Reference live replay soft-reset crash
Status: open
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
