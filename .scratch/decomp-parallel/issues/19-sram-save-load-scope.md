# Research the native SRAM save/load transaction

Status: open
Blocked by: 05
Triage: ready-for-agent
Type: research

Consumer:08; Ceres GameState33 dependency also visible to06
Read-only work; implementation needs accepted player/room/13host continuation

## Work

Read [later lanes](../catalog/later-lanes.md) and pinned SaveToSram/LoadFromSram
in bank81, slot/checksum/complement/packed map and player state dependencies.
Map caller semantics including Ceres GameState33's SaveToSram transition. Reuse
canonical byte layout; no independent copied SRAM state or modern save format.
There is no dedicated Rust persistence implementation/complete receipt here.

Exact proposed files: `crates/port/src/persistence.rs`,
`crates/port/tests/persistence.rs`, `tools/port/persistence.toml`.
Canonical WRAM/SRAM accessors, lib.rs, CLI/hardware/live registration remain
integrator-owned. Select the complete transaction root and dependencies without
splitting shared save state across workers.

## Acceptance / Answer

Source/ROM/state extents and observed whole-root receipts, checksum/invalid-slot/
failure/unchanged-surrounding state controls, complete direct/indirect dependency
reviews and concrete writable/input hashes. Current native bank86exercise cannot
promote this81caller; create explicit reviewed Rust extension task if no supported
provenance. Produce exact1–32 routine implementation child/scopes with bounded
actual-C WRAM/SRAM/return/order gates and original-ROM comparison. Fresh-process
ordinary-reset save/load/continue must preserve resources/room/item/map/state
without candidate snapshot injection or hidden fallback.60s initial/3600s merged
argv; make08(and needed06Ceres-save domain) depend on exact generated tasks.

## Completion evidence

Pending Answer/task/scope/artifact hashes, cleanup/authentication or no-run
result, bytes/free space and retained source/readers/stores/private inputs.
