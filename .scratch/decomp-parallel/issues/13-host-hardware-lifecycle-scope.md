# Research one complete host/hardware lifecycle boundary

Status: resolved
Blocked by: 05
Triage: ready-for-agent
Type: research

Consumers:06–08
Read-only work can proceed beside11; held host source mutation waits11

## Question / Work

Reuse `game_host.rs`'s typed SpawnPlm/SpawnHdma/FlushVram transaction and ten
material actual-C pairs, described in [catalog](../catalog/subsystem-catalog.md).
Select a complete accepted-NMI/VRAM or reset lifecycle scheduled boundary from
sm_80/82 with original reference evidence. Review WRAM/hardware pending state,
DMA/HDMA, NMI enabled/disabled, IRQ, PPU and APU domains and complete external
services. Keep unchanged Samus projectile/room consumers and current refusal
versus fatal-after-host/no native retry contract.

Exact held files: `crates/port/src/game_host.rs`,
`crates/port/tests/game_host.rs`, `tools/port/game-host-projectile.toml`,
`tools/port/game-host-room-scroll.toml`. Proposed dedicated files:
`crates/port/src/hardware_lifecycle.rs`, `crates/port/tests/hardware_lifecycle.rs`,
`tools/port/hardware-lifecycle.toml`. Shared lib/state/ROM and all pipeline
registration/templates/adapters are solely integrator-owned. Select only the
needed exact files; do not take held host files before11's completion.

## Acceptance / Answer

Publish an exercised coherent1–32-routine boundary with source/exercise hashes,
every direct/indirect/MMIO dependency review and exact source/test/manifest
ownership. Existing native exercise reader accepts bank86 projectile receipts
only: locate supported provenance or create an actionable reviewed Rust
exercise-contract extension task instead of substituting unrelated counters.
Require actual-C full state/returns and ordered material effects, refusal before
mutation, fatal-after-host/no retry and unchanged-consumer regression. Pre/merged
argv,60s initial/3600s integration limits and original-ROM gate must be concrete.
Create exact implementation/extension children and add required consumers as
blockers before gameplay dispatch. No global host redesign or fabricated readiness.

## Completion evidence

Pending Answer, generated child/scope receipts and hashes. No-run or scoped-run
cleanup with measured bytes/free space, retained dependencies and authentication.

2026-10-13 production research completed by actual worker
`01a13571-7282-7660-99e1-227a72a2c4ff`, admitted through the exact native claim
in `../launch-20261013/receipts/finish-13.stdout.toml`. See
[Answer](../launch-20261013/research-13/answer.md), SHA-256
`46e394a53e870358a793fdc2b18858c63310df2f7f94646436468efad9bd862d`,
and its sealed manifest/handoff. The complete eleven-routine bank80 NMI body is
identified; native exercise and full hardware-state adapter remain missing.
Published reviewed next-research children21–23 preserve these dependencies in
milestones06–08. No gameplay stage is claimed. No run/build/worktree was created;
cleanup reclaimed0 bytes and all40 input/five artifact seals passed. Final
available space and retained dependencies are recorded in the worker handoff.

<!-- decomp-assignment:ticket-5b357039d4b20673c7d62a43260abada8f7f4f65d7c26999c08138037691dc12 -->

<!-- decomp-assignment-record:start -->

## Current assignment

Stable work: `ticket-5b357039d4b20673c7d62a43260abada8f7f4f65d7c26999c08138037691dc12`
Input revision: `785e3c24a0e08cd0ab9815084aade2973c955bfcdbf96b48d00a1862d3771771`
Lifecycle: `complete`

Honor existing local ticket .scratch/decomp-parallel/issues/13-host-hardware-lifecycle-scope.md. It is open, unclaimed and dependency-ready. Read its requirements and establish a complete Translation scope before implementation: observed exercise provenance, required dependencies, exact writable paths and concrete verification argv. Include a bounded pre-integration test and an integration gate. Use native translation-scope-v1 TOML; limit each initial experiment to approximately 60 seconds. Research facts do not grant acceptance. Preserve task-origin ownership until its implementation is integrated, verified and cleaned.

Writable ownership:

No maintained source writes.

Claim owner: `decomp-launch-20261013`; attempt: `1`.

## Completion evidence for this assignment

```toml
result = "researched"
followups = []

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/launch-20261013/research-13/answer.md"
sha256 = "46e394a53e870358a793fdc2b18858c63310df2f7f94646436468efad9bd862d"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/launch-20261013/research-13/children.toml"
sha256 = "187c439f4518c8d7dccc62d9a6b36fe854841dde16f90a6641a0c3b3b55c0da1"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/launch-20261013/research-13/handoff.md"
sha256 = "28f331698cc21cf25fd407f668edae613866a9dae189168da650204dc4a2bdcb"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/launch-20261013/research-13/artifacts.sha256"
sha256 = "40fbada100102956b6e510d74b4da36b32237db4ec2488fac0dfecb4a17eb799"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/receipts/ticket-5b357039d4b20673c7d62a43260abada8f7f4f65d7c26999c08138037691dc12-1-request-504928e558451b942a29f1a489a3fc14e2a502ea9ef24260f4c6024d9768f019.toml"
sha256 = "d566a9918798bae1e01e4e4259be3dac7a9423430c3fd8aac7b1a97758a97343"

```

<!-- decomp-assignment-record:end -->
