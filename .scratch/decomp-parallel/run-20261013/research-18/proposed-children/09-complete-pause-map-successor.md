# Close complete pause/map/unpause subsystem and dependency partition

Type: research
Status: proposed; coordinator review/publication required
Origin: research 18 exact claim, attempt 1; see [Answer](../Answer.md) and [manifest](../children.toml).
Admission: `research-before-native-implementation-scope`. No ownership or gameplay acceptance is granted.

Reviewed MainPause closure contains at least 79 routines after several existing services were excluded. Finish all required dependency closure rather than taking an arbitrary32 routines. MainPauseRoutine's8 way table is separate from GameState 15's filter/HUD/palette work and the full12→13→14→15 /16→17→18→8 pause/unpause sequence.

Research actual reverse-DMA VRAM backup, gameplay palette/register/PPU restore, map-scroll/map-resource rendering and Samus pose/beam restoration. Include true GameState 17 unpause-hook yield/resume and existing 109 Kraid/Draygon/MotherBrain hooks, HDMA/animtile disable/enable and audio cancellation/schedule.

Equipment/rendering are separate accepted dependencies. Propose exact remaining coherent root/dependency triplets only after complete closure, original resource/input seals and source-bound material exercise. The pause_menu triplet below is provisional; no source path, adapter or scope is admitted.

## Exact proposed files and ownership

- `crates/port/src/pause_menu.rs`
- `crates/port/tests/pause_menu.rs`
- `tools/port/pause-menu.toml`

The coordinator owns public lib/state/ROM/SRAM wiring, shared pipeline/ABI/catalog registration, main Git/index, tickets and inventory. Isolated workers are not alone in the codebase; they must retain other workers' changes and use only returned exact writable files/branch/worktree/baseline. These proposals authorize no shared edit.

## Dependencies

- 18-pause-equipment-interaction
- 18-pause-equipment-render
- 22-23-hardware-and-NMI-contract
- 14-player-pose-beam-state
- 109-accepted-room-pause-hooks
- 18-audio-service
- 12-controller-polls

Parent must replace dependency descriptions with exact completed local task paths, current input/dependency evidence hashes and supported observed-root receipts before native admission. Historical bounded leaves remain historical until current-source authentication.

## Proposed verification commands

All new targets/manifests/adapters here are unimplemented, and all commands are unrun. Review must register real adapter support and require nonzero executed tests/verification receipts before accepting these gates.

Submission, at most 60 seconds:

```sh
cargo test -p port --locked --offline --test pause_menu
```

Merged gate, at most 3600 seconds:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-verify --manifest tools/port/pause-menu.toml --adapter pause-menu --output out/port/pause-menu --case-jobs 4
```

Exact four-subject material Live replay and independent original-ROM argv/timeouts are a required infrastructure/research deliverable before implementation scope publication. No invented current live flag or adapter string is admitted. Native scope argv must not override source/checkout/root/changed-since; the integrator separately records the exact baseline and all-affected selection on frozen source with workers 4.

## Acceptance and controls

- Finish8 way MainPause and states12→13→14→15/16→17→18→8 closure, including true unpause-hook coroutine and GameState 15 HUD/filter/palette work.
- Partition actual map/sprite/tile/backups/restoration and all owned dependencies into coherent1–32 routine scopes with exact files.
- Bind reverse-DMA VRAM backup/restore, palette/register/PPU state, Samus pose/beam, HDMA/animtile, map scroll/assets and room hooks.
- Obtain direct material roots/selector/yield/resume and original-ROM route/timing evidence, define exact adapters and gates before native promotion.

## Limitations and cleanup

- 79 is a reviewed lower bound with excluded existing services, not a complete inventory closure or approved scope.
- Proposed new module/test/manifest/adapter is unimplemented; all new gate argv are unrun and require separately reviewed adapter registration.
- No current supported exercise, native Scope admission, verified gameplay or original-ROM verdict is asserted.

After any future run, wait for writers, run both scoped project cleanup passes, preserve source/readers/store/private-input dependencies, retire integrated worktrees, and reauthenticate retained receipts. Record measured reclaimed/free bytes including zero. This research created no experiment or run; [cleanup](../cleanup.toml) records that result.

