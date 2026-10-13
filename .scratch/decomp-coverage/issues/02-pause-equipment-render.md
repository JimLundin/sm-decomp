# Translate and verify the complete pause equipment renderer

Type: task
Status: claimed
Blocked by: None
Triage: ready-for-agent

Translate these eight pinned-C routines:

- DrawMenuSpritemap
- DrawPauseScreenSpriteAnim
- EquipmentScreenDrawItemSelector
- EquipmentScreenDisplayReserveTankAmount
- EquipmentScreenDisplayReserveTankAmount_
- EquipmentScreenDisplayReserves_PaletteSetup
- WriteSamusWireframeTilemap
- WriteSamusWireframeTilemapAndQueue

Worker ownership: crates/port/src/pause_equipment_render.rs,
crates/port/tests/pause_equipment_render.rs and
tools/port/pause-equipment-render.toml. The coordinator owns lib/catalog wiring,
shared publication and the main Git/index. Use canonical WRAM, original ROM
tables, wrapping integer behavior and complete output comparison. Preserve
existing modules and other workers' edits.

Add actual-C fixture pairs for all roots, including animation boundaries,
reserve empty/full/partial and palette branches, OAM wrap/extension bits,
wireframe equipment variants and exact seven-byte queue output. Exercise
multi-call continuations using the existing family pipeline. Include independent
Rust refusal tests for invalid state/assets without partial writes.

Publish translating at claim, translated with the source commit and verified
only after actual-C verification and post-cleanup authentication. Live pause
gameplay integration remains pending.

## Comments

2026-10-13: selected from completed source findings; no new speculative scope
research is required. Actual-C fixtures are verification tasks.

2026-10-13T16:34Z: use a sealed bounded offline scope and the existing queue
engine through the Rust coverage command. Public lib/catalog wiring remains
with the original integration owner; the standalone family verifier and path
imported Rust tests can verify these new routines independently.

<!-- decomp-assignment:pause-equipment-render-offline -->

<!-- decomp-assignment-record:start -->

## Current assignment

Stable work: `pause-equipment-render-offline`
Input revision: `53e916e7fc7edf90ffde5e953a18bcc43fb0ee931256ebd01e446757818f27f1`
Lifecycle: `submitted`

Translate the eight-routine complete renderer and verify all roots through actual compiled C. Use only returned files and isolated worktree. Publish source commit and actual-C receipts; integrated/live stages remain pending. No speculative research or shared-source refactors.

Writable ownership:

- `crates/port/src/pause_equipment_render.rs`
- `crates/port/tests/pause_equipment_render.rs`
- `tools/port/pause-equipment-render.toml`

Claim owner: `decomp-run-20261013`; attempt: `1`.

## Completion evidence for this assignment

```toml
result = "submitted"
commit = "fa6e9cd747acabcb4568ccd2c0fbf532a9525c18"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-coverage/pause-equipment-render-scope.toml"
sha256 = "53e916e7fc7edf90ffde5e953a18bcc43fb0ee931256ebd01e446757818f27f1"

[[evidence]]
path = "/home/jim/sm-decomp/out/port/decomp-coverage-20261013/pause-equipment-render/1791910621780438-820687-0/verdict.toml"
sha256 = "26ec26b298c9221ded5525e149f91739bff70d18e92503553d2381ba6caea405"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-coverage/renderer/family-authenticate.stdout.toml"
sha256 = "26ec26b298c9221ded5525e149f91739bff70d18e92503553d2381ba6caea405"

[[evidence]]
path = "/home/jim/sm-decomp/out/port/decomp-coverage-20261013/direct-animation/verdict.toml"
sha256 = "ac83dfd2f1723dfbb7f94f21d67d8fdee5e83da1538bae23ecf0aa9c0c2508ec"

[[evidence]]
path = "/home/jim/sm-decomp/out/port/decomp-coverage-20261013/direct-animation-authenticate-retry.stdout"
sha256 = "d3d4006008b535eb48605e3074bbbc3837b490101d9cc5493e23a9ae318a5d89"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-coverage/renderer/direct-animation-authenticate-retry.stdout.toml"
sha256 = "945108a6466c0aa9fe78b0478407de6f9dedd44182433291b826c63539742659"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-coverage/renderer/submission-tests-retry.stdout.toml"
sha256 = "8b7b1587a2647e86ac63dfc1811feaf29695fa6cfca7e67de9a4f1b6200862c2"

[[evidence]]
path = "/home/jim/sm-decomp/out/port/decomp-coverage-20261013/submission-tests/renderer-submission-tests.stdout"
sha256 = "aa2e5b3a9be2515a13c5945b4c90e5a5c45cda0e23d718fd9b36c49b93f00b79"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-coverage/renderer/cleanup.stdout.toml"
sha256 = "db5bac8a65aebadf59f0fdf35212019050be91606a9165b298d3f3d192ecacdc"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-coverage/renderer/cleanup-deduplicate.stdout.toml"
sha256 = "390d0f2327615202f8b6247535835f1f4c9fb3f66e7afca93a2db3466e991b29"

```

<!-- decomp-assignment-record:end -->



## Completion evidence

2026-10-13T17:06Z: source delivered to main at `fa6e9cd747acabcb4568ccd2c0fbf532a9525c18`,
following implementation `a026c883cbd651663c0945155fe409886306e384` and its narrow
original-ROM palette expectation correction. Exactly the three owned files are
committed; eight new routines, 247 maintained generic fixtures and 360 direct
animation fixtures. Ten Rust tests passed through `sm-pipeline exec`, including
wrapping/alias/order semantics and unchanged-state refusal.

[Coverage result](../renderer/coverage-result.toml),
[validated submitted receipt](../renderer/submitted.stdout.toml),
[main-source actual-C authentication](../renderer/family-authenticate-main.stdout.toml)
and [direct animation verdict](/home/jim/sm-decomp/out/port/decomp-coverage-20261013/direct-animation/verdict.toml)
record all 607 full-WRAM/raw-return passing pairs. Direct images/inputs are stored
through the evidence crate, run `b6c848be5e261f793c9387ae7ef45cb05db33a21ed42cfd4116e32c4f13bd7e0`.
The validated submission captures input/artifact hashes through the same Store,
run `570aee759ab85150ed6f60f1f54cbd7e235bf8bd10c271fd0a8e41b2bd257872`.
No manual SHA bookkeeping was used for the new coverage delivery.

Both scoped utility passes completed: ordinary cleanup 0 bytes; deduplication
44,564,480 logical bytes over 741 checked snapshots. Both verdicts authenticated
post-cleanup; generic source also authenticated against merged main. Retained
failed setup and fixture-expectation attempts grant no coverage. Source/worktree
retention and final disposable build cleanup are recorded in the final handoff.

Translated and verified observations are published through Rust progress update
and appear newest-first in the existing report. Public lib/catalog registration
and live pause scheduler/hardware acceptance remain pending. The durable
`integrate-pause-equipment-render-offline` job is queued behind the original
Ceres host integration, so this implementation ticket remains claimed rather
than falsely resolving gameplay integration.

2026-10-13T17:20Z: final retirement is complete.22 exact small source dependencies
remain at their sealed paths; the implementer Git worktree is removed. Disposable
build/browser removal reclaimed752,100,601 logical bytes; remaining disk space
193,325,199,360 bytes. Both retained actual-C verdicts reauthenticated after
retirement. [Retirement receipt](../renderer/retirement.toml) and
[final handoff](../renderer/final-handoff.md) identify retained stores/readers.
The passing generic family is additionally stored in the shared evidence crate
as run `1a27ac79752f2df11e18bb1ce3e611986483e30612c6c045a1a29f1cac37e39a`;
its filesystem paths remain required by the current family authentication
contract. New stage reports, source velocity and actual fixture changes are
visible in the existing newest-ten report. Integration follow-up is
[task36](../../decomp-parallel/issues/36-equipment-renderer-integration.md).
