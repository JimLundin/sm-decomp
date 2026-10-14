# Review complete synchronous enemy lifecycle transactions

Status: open
Blocked by: 11, 21, 24, 25
Triage: ready-for-agent
Type: task

Consumers: 31, 32 and milestones 07/09.
Origin: resolved research17; [sealed proposal](../run-20261013/research-17/children.toml), key `17-enemy-transaction-adapter`.

## Question / Work

Prepare an exact reviewed Rust infrastructure scope for the proposed
enemy_lifecycle adapter/model/host/ABI and tests. Shared family, inputs, live,
model, main and catalog hooks remain integrator-owned. Review the actual player24
and PLM25 service domains before preparing ownership; this ticket grants no
source writes. Reuse the single native-root exercise contract21 and current
authority/source-epoch checks after Ceres11.

Carry full 128 KiB canonical WRAM, actual eight host words, raw and normalized
returns and ordered service effects. Preserve enemy/SpawnData/extended overlays,
byte-slot/list-tail/current-index aliases and partial native resets. Execute real
SpawnEnemy, normal death-projectile, Samus_DealDamage, CallSomeSamusCode(4), PLM,
audio and sprite continuations synchronously on the latest state with complete
copyback. Bind recursive initializer/allocation/RNG/queue/result footprints.

Immutable whole-call preflight must refuse unsupported continuations before
effects; an accepted service error is fatal with no C retry. Ridley's foreign
allocator/item-drop adapter deliberately refuses nested continuations and does
not establish this contract. Waver uses normal EnemyDeathAnimation/F345; keep
boss-specific Enemy_ItemDrop continuations outside the selected domain. Preserve
native137 and held141 lineage and ownership.

## Answer / Acceptance

Publish exact implementation child, baseline, writable hunks, source-bound
actual-C positive/nested/order tests and independent unchanged-refusal,
first-service and mid-service fatal-error controls. Compare full state, returns
and all eight host words; rerun affected consumers and native137.

Proposed initial gate: cargo test -p pipeline --locked --offline --test
enemy_lifecycle_adapter, capped at 60 seconds. Merged gate adds callback/family/
live tests and is capped at 3600 seconds. These targets are future requirements,
not executed evidence. Use shared workers4 and exact claimed isolated ownership.

## Completion evidence

Pending reviewed infrastructure scope and implementation child. The retained
[Answer](../run-20261013/research-17/Answer.md), proposed child02 and evidence
inventory record the researched facts. Infrastructure acceptance adds no
gameplay stage. Both cleanup passes, required source/store/reader retention,
worktree retirement and post-cleanup authentication precede resolution.

## Comments

2026-10-13: this is adapter/test implementation work. Use the completed state,
service and failure contract; speculative research is deferred. Keep complete
live transaction requirements while allowing independently supported offline
reference-C translation and fixture verification to proceed.
