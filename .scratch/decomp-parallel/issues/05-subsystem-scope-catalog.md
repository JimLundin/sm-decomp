# Catalog dependency-ready subsystem scopes

Status: open
Blocked by: 01
Triage: ready-for-agent
Type: research

Build a catalog of coherent Translation scopes so several sessions can select
independent work without splitting shared state or duplicating recovered Rust.
This read-only research proceeds beside tooling tickets 02-04 after ownership
reconciliation. Follow [translation progress](../../../docs/agents/translation-progress.md),
[assignments](../../../docs/agents/assignments.md),
[the pivot](../../../docs/DECOMP_PIVOT.md) and
[the revised strategy](../../decomp-strategy/revised-strategy.md).

## Questions to answer

For each whole subsystem or complete scheduled call boundary: what is already
translated/verified/integrated, what is directly exercised, which dependencies
and shared interfaces remain, which exact files can one worker own, and which
bounded actual-C/live/original-ROM gates will admit it? Current native scopes
support 1-32 coherent routines and restricted gameplay paths. A larger envelope
needs successive complete boundaries or an explicit reviewed Rust tooling
extension rather than unsupported flags or relaxed evidence.

## Candidate lanes

These module names are existing starting points, not approved writable scopes.
Research must seal exact files and dependencies before assignment. Shared
`lib.rs`, CLI/catalog, canonical RAM/ROM, ABI/callback wiring and registration
remain with the integrator. Keep each proposed fixture descriptor and test
specific to its lane; two workers never mutate the same routine or state owner.

| Coherent lane | Existing module/test candidates | Dependency boundary | Gate / milestone |
| --- | --- | --- | --- |
| Reset, scheduler and controller | `crates/pipeline/src/decomp_prefix.rs` and prefix observers/tests; reviewed scheduler files from ticket 78 | Submitted controller, integrated shared host, causal input/NMI contract; current claims retain these paths | Extra-poll diagnosis, controller negative control, ordinary reset and Ceres escape (06) |
| Shared host and hardware lifecycle | `crates/port/src/game_host.rs`, `crates/port/tests/game_host.rs`, dedicated hardware adapter files | Existing host integration; unchanged player and room consumers; shared callbacks handled by integrator | Material ordered PLM/HDMA/VRAM and reset/NMI/IRQ/DMA/PPU/APU effects; supports 06-08 |
| Samus, collisions and player projectiles | `samus.rs`, `samus_control.rs`, `samus_projectile_frame.rs`, `projectiles.rs`, projectile-flight/special/trails modules and matching tests under `crates/port/` | Canonical state/ROM and host contracts; player ownership disjoint from room camera | Complete scheduled movement/contact/projectile actual-C state and effects, mixed/live route exercise, original-ROM boundaries (07) |
| Rooms, doors, camera, PLMs and assets | `room.rs`, `room_scroll.rs`, `camera_scroll.rs`, `plm.rs`, `plm_instructions.rs`, `plm_render.rs` and matching tests | Host/state/ROM interfaces and material hardware effects; camera stays in this lane | Complete room/door/scroll/PLM actual-C boundaries, ordered graphics effects and exercised transitions (07) |
| Enemies, bosses and enemy projectiles | `enemy.rs`, `enemy_ai.rs`, instruction and enemy-projectile modules, `ridley_enemy_instructions.rs` and matching tests | Allocation/contact/room/host contracts; preserve claimed ticket 141 and its retained Ridley source/evidence | Whole lifecycle/script/contact/render boundaries, material actual-C and live ownership; no hidden spawn/drop/CPU fallback (07-09) |
| Audio, UI, menus and persistence | Existing bounded system/audio implementation and tests; dedicated save/UI files selected by research | Reset/audio/hardware interfaces and canonical player/room/SRAM state; preserve earlier evidence domains | Ordered audio ports/queues, menus/inventory, SRAM and fresh-process continuation, then credits route (08) |

## Work

- Reuse the lexical inventory, stage observations, recovered C/Rust modules and
  existing tickets/evidence. Historical leaf acceptance proves only its domain;
  inventory counts and routine hits do not imply caller/subsystem completion.
- Preserve existing claims, especially affected-family-assignment 03-07,
  Ceres 01-03 and decomp-port 78/141. Each candidate names its held ownership,
  missing dependencies, exact source baseline and stage limitations.
- Produce reviewed dependency-ready `translation-scope-v1` documents when its
  facts exist, with pinned exercise/dependency receipts, exact writable files
  and concrete pre-integration/integration argv. Mark missing facts as research
  blockers. Keep gameplay source unchanged in this catalog ticket.
- Create child implementation/research tickets for actual gaps in dependency
  order, assign each whole coherent boundary and add those children as blockers
  to milestones 06-09. Read-only lane work can proceed together; gameplay launch
  through the future workflow requires 04 and this catalog, plus child readiness.

## Acceptance

`## Answer` lists reused evidence and remaining work for every candidate lane,
with explicit readiness, exact disjoint ownership, dependency graph and gates.
At least the next Ceres scope is concrete; unknown later domains remain visible.
Child tasks are actionable and milestone blockers reflect them. Add an Answer
pointer to the map's Decisions so far. Catalog completion permits selecting
ready scopes after 04; it does not establish gameplay acceptance.

## Answer

Pending research.

## Completion evidence

Append catalog/scope and dependency receipt hashes, source revisions, linked
child tickets and reviewed ownership boundaries. Follow scoped
[run cleanup](../../../docs/agents/run-cleanup.md) for any investigation runs,
record measured bytes/free space and retained dependencies, or record that no
runs were created before resolving the research.
