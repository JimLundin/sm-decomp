# Parallel-session preparation readiness

Status: tooling and catalog complete; actual drill final integration/validation active
Blocked by: 04
Triage: ready-for-agent
Updated: 2026-10-13

The [spec](../spec.md), [ticket map](../map.md) and
[session runbook](../../../docs/agents/decomp-sessions.md) define the workflow.
Preparation completes tickets 01–05. The user will request the gameplay swarm
separately; current preparation creates no gameplay assignment or acceptance.

## Preparation gates

| Ticket | Status | Evidence |
| --- | --- | --- |
| [01 Legacy reconciliation](../issues/01-reconcile-legacy-ownership.md) | resolved | Twelve retained holdings, immutable original/successor bridges, exact stopped/live owner observations and checked maintenance adoption |
| [02 Ownership and integration fencing](../issues/02-fence-writable-paths-and-integration-sessions.md) | resolved | Integrated Rust engine; cross-effort path/ticket/submission fences, shared capacity and real-session generation checks; focused controls and cleanup complete |
| [03 Rust commands and handoffs](../issues/03-maintained-rust-commands-and-handoffs.md) | resolved | Typed infrastructure/session admission, checkpoint/Start/Resume/Adopt and exact integration retries; focused controls and cleanup complete |
| [04 Actual concurrency/recovery drill](../issues/04-concurrency-and-crash-recovery-drill.md) | claimed | Independent real A/B sessions and workers; live-child/recovery/retry/budget controls passed; serial integration, final validation and cleanup active |
| [05 Subsystem catalog](../issues/05-subsystem-scope-catalog.md) | resolved | Published whole-subsystem catalog and children 11–20; exact held Ceres continuation and milestone blockers; later missing facts remain research |

## Exact source and ownership

The actual drill execution subject is frozen at
`c8799f326ffe198b652301f08866f1b7a7bd9ebc` on
`decomp-preparation-20261013`, checkout
`/tmp/sm-decomp-preparation-integration-20261013`. Its source fingerprint is
`4d87e457e895a03f82f5a7ccdd891f7769b83d25d48caf2fd3a55432e5929338`.
Reporting source `263ad5a` is included. The
[integration status](integration-status.md) retains source/test/recovery history.
The frozen executable and Rust harness are retained in `../drill/` outside
disposable build trees. Executable bytes remain outside Git; build and SHA-256
receipts identify the retained reader.

Final combined locked/offline verification passed 23 assignment, 15 session
and 10 authority controls; formatting and build passed. Original failing
diagnostics and corrected passing logs remain in
[integration-evidence](integration-evidence/artifacts.sha256).
A validator-only semantic correction is being tested separately: the engine's
actual duplicate-ticket refusal `ticket is claimed` must be tied to the exact
held ticket/claim and sealed disjoint candidate scope. It consumes the unchanged
frozen subject; executed engine and validator source versions stay distinct.

Preparation coordinator is real thread
`01a13460-4aa9-7fe0-865e-a8f54ffca310`. Original preparation coordinator failure
and recovered worker ownership are preserved in reconciliation. Reporting
finished and released main Git/index after both `main` and
`affected-family-assignment` reached
`9ed933daa151f52e3cf922e6559c7681330e0f85`. Preparation now owns that Git/index
and its isolated integration worktree. The report server remains reporting-owned.

## Production boundary

Production `.scratch/decomp-parallel/authority.toml` and `ledger.toml` are
absent. The original Ceres ledger remains authoritative under `ceres-20261013`,
capacity 3, seal
`77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89`.
Both submitted controller/host and the running host integration retain their
original claims, source, evidence readers and acceptance gates. A stopped
submitter or parent does not release any of them. The original controller
worktree remains a held unintegrated submission.

All production sessions share
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline` and
`SM_PIPELINE_WORKERS=4`; assignment capacity 3 is separate. The disposable
actual drill uses its own tiny Rust fixture, repository, authority and budget
root, with no private ROM/movie or production source input.

The planned [cutover package](cutover/README.md) preserves exact legacy lineage.
Final production readiness uses a private mount namespace with a temporary
prepared policy view and maintained `validate-activation`. Host canonical
[authority/ledger](readiness/README.md) remain absent throughout. SHA-256 seals
establish retained artifact identity; real process/session observations remain
an explicit tool inspection trust boundary.

## Later swarm launch

After preparation resolution and the user's separate launch instruction,
reinspect all real owners/processes and exact source/ledger/bridge identities.
Use the final readiness package against its retained frozen source subject,
register current real dispatcher/integrator roles and activate only the reviewed
policy. A change to mutation/admission code requires fresh actual drill evidence;
validator-only and documentation changes require an explicit source-equivalence
review. Then choose dependency-ready whole subsystems from the catalog. Continue
held legacy claims through their original acceptance gates rather than creating
[duplicate implementations](../catalog/subsystem-catalog.md). Native promotion
still accepts bank86 projectile exercise provenance; other lanes require the
researched contract extensions.

## Cleanup

Completed engine, session, integration-test and validator implementer worktrees
were retired after source/evidence retention and both cleanup passes. Their
small historical evidence paths and main relocation manifests authenticate.
Private targets reclaimed 1,260,555,955, 1,884,150,932, 1,218,133,244 and
1,481,441,788 bytes; root combined target reclaimed 1,806,295,023 bytes. The
frozen executable remains a required evidence reader. Actual-drill, final
validator and readiness-helper cleanup remain part of ticket 04 completion.
