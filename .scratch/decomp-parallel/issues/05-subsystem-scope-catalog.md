# Catalog dependency-ready subsystem scopes

Status: resolved
Blocked by: 01
Triage: ready-for-agent
Type: research

Owner: `/root/subsystem_catalog`, preparation coordinator thread `01a13460-4aa9-7fe0-865e-a8f54ffca310`
Claimed: 2026-10-13; read-only catalog and child-ticket authoring only. Ticket 01 is resolved; legacy gameplay holdings stay retained.

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

Researched catalog: [subsystem-catalog.md](../catalog/subsystem-catalog.md).
It lists reused stages/evidence, exact proposed ownership, dependencies, gates
and readiness for every lane. [Later lanes](../catalog/later-lanes.md) supplies
eight concrete new-module boundary triplets and primary C definitions; all
missing caller/host/APU/SRAM/credits domains remain research holdings.

The concrete next Ceres continuation is sealed in
[ceres-continuation-holding.toml](../catalog/ceres-continuation-holding.toml):
finish the original running host integration, then the original submitted
controller integration, then research the first extra poll at ordinal7741,
frame8508/event63799. Existing78/controller/host claims and141broader Ridley
acceptance stay held. Concrete future tests/2240regression/8600capture-reuse argv
and controls are provided; no scheduler correction or gameplay gate ran.

Current native1–32-routine `translation-scope-v1` consumes only schema2bank86
projectile exercise. Non-bank86 caller contexts cannot be promoted from existing
live/callee counters; reviewed Rust exercise-contract extension is actionable
research work. No unsupported native scope or fabricated readiness is published.
All source/receipt facts use inspected baseline
`deceefdb568990fe273cc3f6cdf849726d8380bb`, recovered-C revision578f90b3 and
inventory SHA7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25.
Both initial and republished final reporting registry digests are recorded.

Children [11](11-finish-held-ceres-integration.md) and
[16](16-continue-held-whole-ridley.md) explicitly track held original work,
without duplicate implementation authority. Research children
[12](12-ceres-nmi-poll-continuation.md),
[13](13-host-hardware-lifecycle-scope.md),
[14](14-samus-frame-scope.md), [15](15-door-room-plm-scope.md),
[17](17-other-enemy-lifecycle-scope.md), [18](18-audio-ui-menu-scope.md),
[19](19-sram-save-load-scope.md) and [20](20-credits-cinematic-scope.md)
are actionable complete boundary research. The required generated exact
implementation/extension scopes become milestone blockers before route claim.

[Milestone/map edits](../catalog/milestone-map-proposal.md) are prepared for the
plan owner to apply; this worker owns no shared map/spec/milestone files.
Broad141real-boss integration is not a default Ceres06blocker. Research Answer
is complete; set this ticket resolved after the plan owner integrates that
proposal and Answer pointer. Root applied that integration before this resolution.
Catalog resolution grants no gameplay acceptance
or user launch authorization.

## Completion evidence

- [artifacts-sha256.txt](../catalog/artifacts-sha256.txt) seals all catalog
  artifacts and child11–20 tickets; SHA-256
  `88a412d0b161e2d1da977696156b6ced13ba29723b3a233dec4e75f275548fd4`.
- [evidence-sha256.txt](../catalog/evidence-sha256.txt) and
  [source-sha256.txt](../catalog/source-sha256.txt) rehash inspected primary
  receipts and exact current source. Both `sha256sum -c` checks pass. This is
  identity inspection, not fresh replay or full recursive verdict admission.
- `git diff --check` for owned catalog/ticket paths passes; retained extracted
  reporting snapshot is valid JSON and matches its recorded inventory digest.
  Existing CLI gates were checked against baseline main.rs1533and1495: native
  family-verify argv and batch--case-jobs are supported; embedded native scope
  gates retain their stricter reviewed context restrictions.
- No fixtures/replays/build/tests, gameplay source, registry publication,
  readers, queues, Git/index or held worktree changes occurred. Read-only
  research and documentation created no run/build directory. Scoped cleanup
  utility is unnecessary for this no-run step; reclaimed0bytes. Final measured
  available filesystem space219,766,968,320bytes. Held source snapshots,
  submitted worktrees, private ROM/movie/SRAM inputs, subject pointers, readers
  and shared immutable `out/evidence` remain preserved.

## Comments

Preparation worker `/root/subsystem_catalog` completed researched Answer and
child artifacts. Parent owns final plan/map integration and status transition;
source schemas may evolve in isolated preparation tooling, requiring new scope
input hashes at actual gameplay dispatch. No gameplay milestone is resolved.

Root confirmed plan/map integration:06blockers04,05,11,12,13;
07blockers06,13–18;08blockers07,13,18–20;09blockers08,14–20.
Ticket05 resolves researched catalog acceptance. No further catalog run or
artifact mutation accompanied the status transition; cleanup remains the
documented no-run0-byte result. Ticket05 is not included in the sealed
catalog/child artifact manifest, so its update changes no existing seal.
