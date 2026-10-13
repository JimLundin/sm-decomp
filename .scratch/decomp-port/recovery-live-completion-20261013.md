# Authenticated bounded gameplay integration

Status: accepted retained live evidence after cleanup
Owner: root integration coordinator
Integration commit: `3ce28ba58aba319b60bbc949153baafd4433132e`

Twelve affected actual-C Offline families pass20,943pairs and authenticate after
both cleanup passes. Their summary is
`recovery-integrated-final-authentication-summary.toml`. The independent trail
and composed native domains retain exact source at `/tmp/sm-recovery-bounded-source`.
No old seals or snapshots were changed.

## Four-subject live evidence

Trail9000 verdict:
`out/recovery-session/trails-live/1791860457650339558-98848/verdict.toml`.
All four subjects match state and ordered effects with zero fallback. Each mixed
subject owns3,824Rust `HandleProjectileTrails` frames,167active,0refusals.
Three movement opcodes have no natural hits in this prefix. The trail domain
does not independently install native137; the composed gate below preserves it.
Post-cleanup receipt: `recovery-trails-live-post-cleanup-authentication.toml`
(four subjects,94artifacts each).

Composed129712 verdict:
`out/recovery-session/composed-live-release/1791861180321696848-129445/verdict.toml`.
All four subjects match periodic gameplay/hardware, full terminal WRAM/SRAM,
ordered PPU/APU effects and complete captures with zero fallback. The retained
authentication reparses counters and reruns native137/bounded-frame admission.
Post-cleanup receipt: `recovery-composed-post-cleanup-authentication.toml`
(four subjects,1,881artifacts each), verdict SHA256
`5e224bda08b3f597b07dd3a03c59bf8cf136b288993459c62e453ef3fac400c2`.

| Native root | Rust total per mixed subject | Active Rust | Refused/C |
| --- | ---: | ---: | ---: |
| MainScrollingRoutine |94425|35580|0|
| UpdateScrollVarsUpdateMap |3875|3875|0|
| CalculateLayer2PosAndScrollsWhenScrolling |94608|94608|0|
| HandleProjectile |97049|21746|0|
| DrawPlayerExplosions2 |102632|21558|0|
| DrawBombAndProjectileExplosions |96336|20998|0|
| ProcessEnemyInstructions, bounded attack script |2393|36timer-due|209673|
| EprojRunAll |95325|866|0|
| SpawnEprojWithRoomGfx |16|16|0|
| SpawnEprojWithGfx |8|4|0|

Native137 preserves6179642scheduler calls and all16births, plus4A387/4A395
births. The attack script opcode totals match C exactly:
E5172,E4D24,E4BE4,E4CA2,E84D18,E9044,E90912,812F2 (48total).
Raw pointer kind, full WRAM and all eight external host words are captured.

## Completion limits

This is C/Rust translation integration, not original-ROM movie completion.
The raw129712-input route terminates in Ceres roomDF8D/state8, Samus173/159,
RNG13022. Original controller/update scheduling and credits progression remain
separate work in ticket78 and the strategy reconsideration. Broader Ridley AI,
real boss, explosion and other enemy scripts do not inherit this bounded
attack acceptance. The isolated whole candidate remains retained separately.

## Cleanup and retention

Trail explicit ordinary cleanup reclaimed47,310,242bytes; dedup0. Composed
ordinary cleanup reclaimed48,176,296bytes; dedup0. Both retained authentications
pass after cleanup. Shared packed store `out/evidence`, per-subject pointers,
retained live binaries and `sm-evidence` readers remain required dependencies.
All eight stale implementer Git registrations were retired; exact source-only
trail/Samus/current snapshots remain authenticated. Only the main Git worktree
remains. Offline batch dedup reclaimed4,522,901,504bytes; final explicit passes0.

Validation:46integrated port semantic tests,17wholeRidley controls,
10Ridley pipeline controls,3compiled live ABI controls,7live admission controls
and3source-inventory regressions pass. Port all-target Clippy is strict; pipeline
Clippy passes with four recorded unchanged baseline exemptions. Progress feature
tests/browser/concurrency are separate evidence. Available disk is recorded in
`recovery-live-final-disk.txt`.
