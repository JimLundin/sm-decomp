# Research the native SRAM save/load transaction

Status: resolved
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

The exact researched result is admitted in
[finish receipt](../run-20261013/receipts/finish-research-19.stdout.toml).
Evidence crate full verification passed for20 files/91,802 bytes, retained run
`522819b568272309a06be459510d5129a4cad20582e8815fbcef7488f3ac9ed0`.
Inventory/verify receipts and the frozen Answer/state/ROM/children/cleanup package
remain retained. No-run cleanup is0 bytes, no writers or build targets.

The coordinator published tasks41–44 for full WRAM/SRAM adapter, four-routine
codec, same-owner companions and real file/fresh-process integration verification,
plus a shared21 provenance supplement. These are implementation/verification
tasks; no more subsystem inventory research is required for the recorded codec.
No new persistence translation, fixture pair or live acceptance is claimed.

<!-- decomp-assignment:ticket-e76fb87d41d3e1e6cab6286936a985d5b5c9d12bc1c0f99806ad86738940ba66 -->

<!-- decomp-assignment-record:start -->

## Current assignment

Stable work: `ticket-e76fb87d41d3e1e6cab6286936a985d5b5c9d12bc1c0f99806ad86738940ba66`
Input revision: `3a80add9c71186647255feadd5855a494d6859612ea56cc4c4ee5cd5ae5829cd`
Lifecycle: `complete`

Honor existing local ticket .scratch/decomp-parallel/issues/19-sram-save-load-scope.md. It is open, unclaimed and dependency-ready. Read its requirements and establish a complete Translation scope before implementation: observed exercise provenance, required dependencies, exact writable paths and concrete verification argv. Include a bounded pre-integration test and an integration gate. Use native translation-scope-v1 TOML; limit each initial experiment to approximately 60 seconds. Research facts do not grant acceptance. Preserve task-origin ownership until its implementation is integrated, verified and cleaned.

Writable ownership:

No maintained source writes.

Claim owner: `decomp-run-20261013`; attempt: `1`.

## Completion evidence for this assignment

```toml
result = "researched"
followups = []

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/run-20261013/research-19/Answer.md"
sha256 = "edfc9f46fd3d132a14ca95280048c9a03d99dfcedd501f41ecc4584112d11c5c"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/run-20261013/research-19/children.toml"
sha256 = "0ddbaa28749663eeacbb8cec00490f178dca253ffaf74ae2278ec082bd6d67d8"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/run-20261013/research-19/scope-holding.toml"
sha256 = "4b0b7a47eb3091e372635a5926cd9b18e14d74d71645f72d0278b98fabb69164"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/run-20261013/research-19/manual-handoff.toml"
sha256 = "5890ca656dc9c8a9529aff40d8aaf21a1c4f66d59bee522d58670091f88ce091"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/run-20261013/research-19/cleanup.toml"
sha256 = "f76870d58a117c7c03480cbbc5eb913cc3ced3027c898d37bb21bb27f1d08572"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/receipts/ticket-e76fb87d41d3e1e6cab6286936a985d5b5c9d12bc1c0f99806ad86738940ba66-1-request-a0115ae98d736627d36e0ee9955733b08ddb33482629febb8a0ab9db9836f7e3.toml"
sha256 = "daa763dab51ea64fba8aa6d947f3356a67722624f2eec95aeb4c46a510bbd691"

```

<!-- decomp-assignment-record:end -->


