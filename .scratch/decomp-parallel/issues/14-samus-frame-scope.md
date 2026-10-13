# Research complete Samus movement/contact frame continuation

Status: resolved
Blocked by: 05
Triage: ready-for-agent
Type: research

Consumers:07,09
Read-only research; implementation depends on required13 hardware contract

## Work

Read [catalog](../catalog/subsystem-catalog.md) and [later lanes](../catalog/later-lanes.md).
Reuse current samus/samus_control/collision/projectile flight/special/power-bomb,
the accepted whole projectile roots and resolved139 trail root. Identify the
complete player beta/frame movement/input/contact root and its exact dynamic
pose/handler/collision dispatch, ROM tables, integer/byte aliases and host calls.
Do not infer a caller hit from existing helper or projectile counters.

Candidate exact envelope: `crates/port/src/samus.rs`, `samus_control.rs`,
`collision.rs`, `samus_projectile_frame.rs`, `projectiles.rs`,
`projectile_flight.rs`, `projectile_special.rs`, `projectile_trails.rs`,
`power_bomb.rs`; matching exact tests under `crates/port/tests/` and proposed
`tools/port/samus-frame.toml`. Select only needed files into one reviewed scope;
camera_scroll/room modules remain read-only room dependencies. Integrator owns
lib.rs, canonical state/ROM, live registration and shared callback adapters.

## Acceptance / Answer

State existing translated/verified/integrated/live-exercised domains and the
remaining complete root; pin source+observed root exercise and each dependency.
Current native bank86-only exercise schema is a blocker to non-bank86 promotion:
create a reviewed Rust extension task if no matching supported receipt exists.
Publish exact1–32 coherent routine task/scope with writable/input hashes when
facts exist. Require actual-C complete WRAM/return/transient/order behavior,
wrap/alias/collision controls, material independent mixed route exercise,
original-ROM matching boundaries and visible fallback. Use60s initial and
3600s merged-source gates. Add new implementation children to07/09 and needed13
dependencies; preserve accepted projectile/trail domains and explicit Ceres
drawing refusal rather than copying upstream undefined-local behavior.

## Completion evidence

Pending Answer/child/scope/artifact hashes, no-run or scoped-run cleanup and
retained source/store/readers/private-input authentication.

2026-10-13 production research completed by actual worker
`01a13571-e61e-7e31-8e29-d8f82488f33f`, admitted through its exact native claim
in `../launch-20261013/receipts/finish-14.stdout.toml`. See
[Answer](../launch-20261013/research-14/Answer.md), SHA-256
`bbd540b7f792dfbf60e492a61dfc665994fc274fb826cc3583f767683ffb9e50`,
and its eight-artifact manifest/manual handoff. Normal beta movement/contact is
separate from alfa input; direct-root provenance, dynamic services and the real
collision displacement contract remain required. Published children21/24 retain
these gaps as milestones07/09 blockers. No new gameplay stage is claimed.
No run/build/worktree was created; cleanup0 bytes, all28 input seals and eight
artifact seals passed, observed available space221118025728 bytes. Retained
source/evidence/store/readers/private inputs remain listed in cleanup.toml.

<!-- decomp-assignment:ticket-6e51e2d92fc5cf720229a4033777377810617f068f09b01ef80339e23a6f8df7 -->

<!-- decomp-assignment-record:start -->

## Current assignment

Stable work: `ticket-6e51e2d92fc5cf720229a4033777377810617f068f09b01ef80339e23a6f8df7`
Input revision: `838efcfad1e05746363aa40f54971b62333e35c0874ffd2639f841929bc2e7b3`
Lifecycle: `complete`

Honor existing local ticket .scratch/decomp-parallel/issues/14-samus-frame-scope.md. It is open, unclaimed and dependency-ready. Read its requirements and establish a complete Translation scope before implementation: observed exercise provenance, required dependencies, exact writable paths and concrete verification argv. Include a bounded pre-integration test and an integration gate. Use native translation-scope-v1 TOML; limit each initial experiment to approximately 60 seconds. Research facts do not grant acceptance. Preserve task-origin ownership until its implementation is integrated, verified and cleaned.

Writable ownership:

No maintained source writes.

Claim owner: `decomp-launch-20261013`; attempt: `1`.

## Completion evidence for this assignment

```toml
result = "researched"
followups = []

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/launch-20261013/research-14/Answer.md"
sha256 = "bbd540b7f792dfbf60e492a61dfc665994fc274fb826cc3583f767683ffb9e50"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/launch-20261013/research-14/artifacts.sha256"
sha256 = "ea0d0d92ad4f5cdeea9b74470b7d635ec3e366fe8e79aba7669bf71f6f0114a3"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/launch-20261013/research-14/cleanup.toml"
sha256 = "2660332ae9ec040b8dff34805b3291937d1e8f39367f243941a4dd0d1eea13d7"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/receipts/ticket-6e51e2d92fc5cf720229a4033777377810617f068f09b01ef80339e23a6f8df7-1-request-14b6b8a1806d5fd7622a07cbc71fb0a4428b04918cf7b8c06a28782d0cc83f21.toml"
sha256 = "89e1ad81ce574a807c71052b38d80fba178524b08533679679ace46e7fc9a192"

```

<!-- decomp-assignment-record:end -->
