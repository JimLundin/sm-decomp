# Pause equipment coverage delivery

Updated2026-10-13T17:21Z. Main branch `affected-family-assignment`, source
`fa6e9cd747acabcb4568ccd2c0fbf532a9525c18`.

Eight new routines are committed in [the renderer](/home/jim/sm-decomp/crates/port/src/pause_equipment_render.rs),
with [247 maintained fixtures](/home/jim/sm-decomp/tools/port/pause-equipment-render.toml)
and10 meaningful Rust tests. Source commit `a026c883cbd651663c0945155fe409886306e384`
implements all eight; `fa6e9cd747acabcb4568ccd2c0fbf532a9525c18` corrects12 fixture
palette expectations to the independently observed original-ROM value1536.
Only the three approved source/test/manifest files changed.

The maintained generic family passed247 full128KiB WRAM/raw-return pairs across
seven direct roots, including inherited-state chains. The exact three-argument
DrawPauseScreenSpriteAnim gate passed360 additional pairs; total607. Tests cover
OAM wraps/extension aliases, signed timer/terminator boundaries, reserve rounding/
flicker/palette, wireframe variants/seven-byte VRAM queue and unchanged refusal.
Original C providers are sm_81.c/sm_82.c/sm_rtl.c; no guessed gameplay stubs.

[Result](coverage-result.toml), [validated queue submission](submitted.stdout.toml),
[retired-source family authentication](family-authenticate-retired.stdout.toml)
and [retired-source animation execution](direct-animation-authenticate-retired.stdout.toml)
retain the passing gates. Actual-C acceptance is bounded to the declared domain.
Rejected dependency-directory setup and incorrect independent fixture-value runs
remain diagnostic records and earn no coverage.

The translated and verified stages are published independently in the existing
[progress viewer](http://127.0.0.1:4185/). Its newest-ten table shows exact UTC
submission times and +8 definitions/138 C body lines. The merged-source
translated observation supersedes the earlier worker observation to establish
current-checkout provenance; that correction adds no new routines. Maintained
progress history now has109 checkpoints/1469 current definitions/0 undated
uncommitted definitions; verified933. Source, verified, integrated and live
counts remain separate. Reporting edits changed no gameplay/harness source.

## Evidence storage and cleanup

Retain shared `/home/jim/sm-decomp/out/evidence`, exact `bin/direct-animation`
and `.scratch/decomp-progress/latest-submissions/bin/{sm-pipeline,sm-evidence}`.
Trusted runs:

- animation input/image/pair closure:
  `b6c848be5e261f793c9387ae7ef45cb05db33a21ed42cfd4116e32c4f13bd7e0`
- passing generic family closure:
  `1a27ac79752f2df11e18bb1ce3e611986483e30612c6c045a1a29f1cac37e39a`
- validated submission input/artifact capture:
  `570aee759ab85150ed6f60f1f54cbd7e235bf8bd10c271fd0a8e41b2bd257872`

All hashes and retained capture objects are produced through the maintained
pipeline/evidence modules. The generic family still needs its original filesystem
paths; its stored copy does not change the current authentication contract.
Direct animation writes compact images directly through Store and reuses only
two temporary WRAM files; those files were removed after execution.

Both scoped cleanup passes succeeded:0 disposable utility bytes and44,564,480
logical duplicate snapshot bytes/741 checked snapshots. [Retirement](retirement.toml)
retains22 exact small source dependencies and removes the implementer Git
worktree/builds. Disposable build/browser removal752,100,601 logical bytes;
remaining space193,325,199,360 bytes. Both verdicts authenticate after retirement.
Only exact small sources remain at the old worktree path, with shared dependency
access; no Git metadata or target. Original Ceres/other owners are preserved.

Repeat authentication without rebuilding gameplay:

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 .scratch/decomp-progress/latest-submissions/bin/sm-pipeline family-authenticate --checkout /home/jim/sm-decomp --verdict /home/jim/sm-decomp/out/port/decomp-coverage-20261013/pause-equipment-render/1791910621780438-820687-0/verdict.toml
.scratch/decomp-coverage/renderer/bin/direct-animation authenticate /home/jim/sm-decomp/out/port/decomp-coverage-20261013/direct-animation/verdict.toml
```

## Remaining integration

The validated durable submission is `pause-equipment-render-offline`, revision
`53e916e7fc7edf90ffde5e953a18bcc43fb0ee931256ebd01e446757818f27f1`, attempt1,
coordinator `decomp-run-20261013`. Integration job
`integrate-pause-equipment-render-offline` waits behind the original active Ceres
host integration. Public lib/catalog/caller and live pause hardware/schedule
acceptance remain pending. Do not mark this ticket resolved or report
integrated/live coverage based solely on source arriving on main.
[Task36](../../decomp-parallel/issues/36-equipment-renderer-integration.md)
tracks that dependency. Do not merge the already merged source again.

User priority is concrete reference-C translation and actual-C fixtures.
Adapters, integration tests and exact failure diagnosis are tasks; speculative
research and broad refactoring are deferred. The completed research18/19 facts
are admitted and linked as tasks33–44/shared21. Remaining speculative successor
work40 is deferred. Independent offline coverage does not require speculative
live-route provenance.

The old host audit also completed106/106 passing current-source authentications.
Those are reusable existing results, adding no new routines. Original host
admission still needs the11 compiler failures/13 interrupted/46 unattempted
families resolved. Canonical alias and exact-provider scratch probes passed;
maintained repairs await the exact original owner. Preserve original claims,
source and seals; no full batch rerun before bounded compiler repair.
