# Legacy owner reconciliation

Status: reconciliation complete for retained holdings; tooling adoption approved
Observed at: 2026-10-13 08:45:38 UTC
Owner: `/root/legacy_audit`, real thread `01a13462-702f-74f0-8a9a-a2a08dd1cb7f`
Coordinator: `01a13460-4aa9-7fe0-865e-a8f54ffca310`

Gameplay remains stopped. This inventory does not release claims, merge source,
run gameplay gates, or activate the future canonical ledger. Source and evidence
already retained by the checkpoint remain required.

## Confirmed durable ownership

The only discovered durable ledger for the held efforts is
`.scratch/ceres-continuation/ledger.toml`, SHA-256
`77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89`.
It uses coordinator `ceres-20261013`, capacity 3 and the same integration owner.

| Job | Stored state | Exact durable claim | Retained action |
| --- | --- | --- | --- |
| compact-controller | submitted | `(compact-controller, 48656a2-compact-v1, 1, ceres-20261013)` | Preserve submission `223f8a1b0c7382817b1290d13c7a41f4e722d5e1`; integration gates remain held. |
| integrate-compact-controller | ready | None; never claimed | Keep integration revision `48656a2-compact-v1-1` linked to the original submission. |
| shared-game-host | submitted | `(shared-game-host, 48656a2-host-v1, 1, ceres-20261013)` | Preserve submission `73547105d870f8b05f8c6bcf55b768c223889fdb`, already merged by `e8c9c3e10ac71bf0377dc00d7de5764a4263b445`; durable admission remains held. |
| integrate-shared-game-host | running | `(integrate-shared-game-host, 48656a2-host-v1-1, 1, ceres-20261013)` | Reconcile the failed original root before any recovery update; finish original admission later. |

Submitted ownership stays held even when an implementer has stopped. The
controller worktree `/tmp/sm-ceres-continuation-compact-controller-1` remains at
its exact submitted commit. The host implementation worktree no longer appears
in Git's retained worktree list; merged source and original evidence remain.

## Markdown-only ownership

There is no `.scratch/affected-family-assignment/ledger.toml` and no discovered
decomp-port ledger. Affected-family tickets 03–07 and decomp-port tickets 78 and
141 remain claimed. Their real ticket identities and original source/evidence
lineage must remain fenced. They have no durable claim tuple to copy, and this
inspection must not invent one.

The Ridley candidate `/tmp/sm-ridley-whole-live-source` and its frozen tooling,
reader and evidence remain dependencies of ticket 141. Its full 129,712-input
live admission and original dependencies 137/138 remain open.

## Initial liveness observations

Read-only thread listing at the observation time reports Ceres root
`01a133eb-76e2-7073-9aef-d97c03a812e4` as `systemError`; recovery coordinator
`01a133b1-ec57-7a81-92e0-106d26e60b0d` and earlier recovery root
`01a13317-fa0e-7052-8c5a-8e665cf86f23` as `idle`. Exact worker and last-message
inspection is still pending. A failed parent does not establish stopped children.

Filtered process inspection found only the progress server PID 330613, command
`target/release/sm-pipeline progress serve --root /home/jim/sm-decomp --bind 127.0.0.1:4185`,
started Tue Oct 13 10:03:43 2026. Process absence alone does not release ownership.
The active progress-report session `01a13394-29b8-76a3-aab7-5ab8eff60694` retains
its separate preparation edits and current main Git ownership; these are not
stopped gameplay claims.

## Completed recovery inspection and bridge

[legacy-owner-bridge.toml](legacy-owner-bridge.toml) now records twelve retained
holdings, exact durable Ceres tuples, the ready unclaimed integration job,
Markdown-only tickets, source/evidence/reader/private-input dependencies,
conservative path fences and owned next actions. Its immutable SHA-256 is
`b4046304db39ed5d70501d0b7b29a24abe75b17c03fd0c20ead702857161b4c5`;
[the sidecar](legacy-owner-bridge.sha256) authenticates it. No held ticket is
resolved by this bridge and no claim is released.

Direct `read_thread` inspections and targeted local session metadata establish
the original identities. Provider `notLoaded` by itself was not treated as
stopped evidence: terminal turns, source retention and process observations are
kept in [legacy-liveness-inspection.json](legacy-liveness-inspection.json) and
[gameplay-liveness-inspection.json](gameplay-liveness-inspection.json), sealed by
[sealed-observations.sha256](sealed-observations.sha256).

| Holding / worker | Real thread | Observed boundary |
| --- | --- | --- |
| Original Ceres integrator `/root` | `01a133eb-76e2-7073-9aef-d97c03a812e4` | `systemError`, final failed turns; three known children inspected |
| Compact controller `/root/ceres_scope` | `01a133ec-4ef9-78b2-a8d7-248726023ce5` | Idle/completed; reports every child process finished; submitted ownership retained |
| Shared host `/root/host_scope` | `01a133ec-68e2-7033-9ee7-dba26d4db746` | `notLoaded` with terminal completed submission; source merged, claim retained |
| Selection `/root/parallel_work` | `01a13153-18fd-7a22-951e-427b7cde2146` | `notLoaded` with terminal completed source submissions |
| Queue `/root/live_feedback` | `01a13153-2eef-7070-b421-0db591d3ee60` | `notLoaded` with terminal completed source submissions |
| Translation `/root/fixture_iteration` | `01a13152-ffec-7000-93c8-a992c6d5ea12` | `notLoaded` with terminal completed source submissions |
| Original affected-family root | `01a13151-b3ec-72e0-8912-f95c230f9e18` | `notLoaded`, final implementation turn failed; original final admission remains pending |
| Later recovery roots | `01a13317-fa0e-7052-8c5a-8e665cf86f23`, `01a133b1-ec57-7a81-92e0-106d26e60b0d` | Idle terminal turns, second root completed recovery; preserve unresolved original acceptance |
| Ticket78 `/root/controller_adapter` | `01a129ac-6d9c-7bd2-be35-6bd00236aa32` | `notLoaded` with terminal completed bounded diagnostic |
| Ticket141 `/root/ridley_live_finish` | `01a1334a-6c95-73f3-97db-08beb2f6ef20` | Idle/completed; frozen candidate and broad full-live gate remain held |
| Surviving preparation engine `/root/ownership_engine` | `01a13453-b599-79c1-84d5-713e6d1e3f57` | Active existing assignment preserved; no duplicate launch |
| Former preparation session worker `/root/spec_finish` | `01a1342e-6cad-7cb1-80d7-6a484c469bbf` | Idle/completed before implementation, clean isolated source retained |

The original preparation root `01a13402-afaa-7501-b97b-15d3b08434be` and
ticket01 reviewer `01a13421-0483-7072-adba-413a21cb343c` have failed turns.
Their names recur in other trees; UUIDs establish identity. Replacement
session worker `01a13464-cb24-77c2-aa7b-ac2aaacd5359` is explicitly bound in
the coordinator's approval.

The supported exact original-token stopped observation
[host-integrator-stopped.toml](host-integrator-stopped.toml), SHA-256
`2df802bd699e159fd9109a6d8d25762696dc5dc9067b3e89be9905bcf5a22446`,
is sealed for later coordinator review. It has not been admitted and changes
neither the running ledger claim nor the submitted children. The replacement
coordinator owns that next reconciliation action and every remaining original
gate/cleanup action listed by the bridge; gameplay remains held during preparation.

## Checked preparation maintenance adoption

[preparation-recovery.toml](preparation-recovery.toml), SHA-256
`1338edb2e6d648d293aba30bc5b0b4d3f0f30c6170f0ca1aa8542b88a6a8bb39`,
proposes exact-path isolated maintenance against `deceefd`, preserving original
affected-family claims and Ridley's focused shared registration. The coordinator
approved it in [approved-maintenance.toml](../preparation/approved-maintenance.toml),
SHA-256 `c223a2b40b399d9ca9b1047e3f54bd59ab215c07b80528cbf8f1ce118385cce3`.
The two maintenance workers own disjoint files in their existing isolated
worktrees. The session adapter's additive `main.rs` wiring is a reviewed
maintenance exception; original Ridley/shared host registration remains held
for later focused sequential integration. Main Git/index stays with reporting
thread `01a13394-29b8-76a3-aab7-5ab8eff60694`.

## Validation and cleanup

TOML parsing passes for the bridge, recommendation and supported stopped
observation. Every bridge object carrying `path` and `sha256` passes a recursive
hash check, including eleven exact legacy infrastructure request/receipt artifacts.
All four checkpoint manifests and Ridley source/reader/hardware manifests pass.
The Ceres ledger remains byte-for-byte at its original hash and the controller
worktree remains clean. The final process scan sees only the intentional report
server PID330613 and this read-only inspection; no gameplay or build writer is
observed. Process absence releases no claim.

This reconciliation created only small authored evidence/documents in the owned
directory. No run, fixture, replay, build or disposable worktree was created;
run cleanup is not applicable and reclaimed bytes are **0**. Existing submitted
worktrees, frozen sources, readers, shared store and private inputs remain
intentionally retained. Available space at final inspection was
**223,256,883,200 bytes**. Legacy admission cleanup remains an owned held action,
not an uncompleted cleanup run created by this audit. Main Git checkpointing is
left to its observed owner; this reviewer performed no index mutation.
