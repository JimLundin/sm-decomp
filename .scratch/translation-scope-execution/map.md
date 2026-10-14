# Translation scope execution

Status: implementing

Spec: [spec.md](spec.md). Selected candidate: Translation scope execution.

| Ticket | Type | Status | Blocked by |
| --- | --- | --- | --- |
| [01 — Confirm execution and acceptance contract](issues/01-execution-contract.md) | grilling | resolved | None |
| [02 — Durable attempts and supervisor interface](issues/02-durable-attempts.md) | task | resolved | 01 |
| [03 — Restricted rewrite exchange](issues/03-restricted-rewrite.md) | task | resolved | 01 |
| [04 — Enforced budgets and evidence finalization](issues/04-budgets-and-evidence.md) | task | resolved | 01 |
| [05 — Frozen preparation and verification adapters](issues/05-frozen-verification.md) | task | resolved | 02, 04 |
| [06 — Integration, progress and recovery](issues/06-integration-recovery.md) | task | resolved | 02, 05 |
| [07 — Complete restricted-path proof and cleanup](issues/07-restricted-path-proof.md) | task | resolved | 03, 04, 05, 06 |
| [08 — Reviewed production switch](issues/08-production-switch.md) | task | resolved | None |
| [09 — Model rewrite and one-command launch](issues/09-model-rewrite-launch.md) | task | resolved | None |
| [10 — Original production acceptance adapters](issues/10-original-acceptance-adapters.md) | task | open | None |

## Decisions so far

- [Spec confirmed requirements](spec.md#confirmed-requirements) records six user
  decisions: enforced restrictions, deliberate retries, structured replacements,
  staged switch, frozen verification rules and hard budgets.
- Three designs were compared: a minimal executor, a durable flexible controller,
  and a supervisor with a single rewrite exchange. Select the small supervisor
  surface backed by the durable controller journal.
- Native gameplay and infrastructure are real execution adapters. Existing queue,
  cache and authentication depth should be retained behind the new module seam.
- Current freely tooled collaboration workers cannot enforce the selected worker
  contract. Restrict the invocation itself and prove denial cases.
- The user authorized migration on 2026-10-14 and paused other work. Checked
  authority transfers and the production ownership registry are now active;
  original claim/result lineage stays intact.
- The direct model connection is the Foundry Responses endpoint using the
  existing `FOUNDRY_TOKEN`. Rust owns the ordered launch and all supervisor steps;
  immutable HTTP intents prevent silent reevaluation after a crash. See
  [operator guide](../../docs/TRANSLATION_EXECUTION.md#one-command-foundry-launch)
  and [model launch checkpoint](model-launch/checkpoint.md).

## Current checkpoint

Production ownership migration is complete and active. [Ticket 08](issues/08-production-switch.md)
records the exact [activation](production-migration/activation.stdout.toml),
original claims, current registered roles and operator command. The installed
plan imports `shared-game-host`, preserves its original Running integration and
orders `compact-controller` afterward. The canonical pause-equipment submission
is explicitly stopped pending its complete composite acceptance adapter.

Legacy readiness records the missing original verification producers; it cannot
publish successful completion. Shared-host callback/catalog acceptance, compact
controller causal diagnostics, pause equipment's 607 Fixture pairs plus 10 Rust
tests, and authenticated sequential publication remain required work. Activation
alone establishes no new gameplay stage or parity verdict. Both production
invocations returned the same typed shared-host prerequisite stop and exit 1,
with no completed scope. All 45 production-file hashes stayed unchanged across
replay; post-cleanup inspection authenticated the stopped attempt and all 58
historical model-launch hashes passed. Both cleanup passes and measured target/
helper retirement are complete. The [production checkpoint](production-migration/checkpoint.md)
records the retained command and evidence.

[Ticket 10](issues/10-original-acceptance-adapters.md) is the next open task. It
owns the original host/controller/pause acceptance adapters and authenticated
successor preparation. Productive execution of those original jobs remains
pending; the overall spec remains implementing despite the completed migration.

Model rewrite and the one-command ordered launcher are committed at
`93e9d55b19630ac8fa2e3526ba963296385ea516`. Ticket 09 is resolved with a real
Foundry/native lifecycle, one model call, four passing actual-C gates, ordered
recovery tests, retained authentication and complete measured cleanup. The
[model launch checkpoint](model-launch/checkpoint.md) binds exact source/readers,
evidence and the prepared user command. That command reports `ready` for the
approved isolated enemy-jump instructions. Its completed proof and cleanup remain
valid historical evidence; the current production migration is recorded above.

Created `/home/jim/sm-decomp-translation-execution` at baseline
`3a8b9ea28b716d90df55f798941bba68e1df6531` on
`architecture/translation-scope-execution-20261013`.
The user confirmed the complete contract. Implementation used disjoint Rust
ownership; root owns pipeline adapter/CLI and integration. The later explicit
production-switch authorization is recorded in ticket 08.

Verified Rust source checkpoint:
`f845516b975c3bfc413800c6bed9e329696a79e6`; proof/documentation checkpoint 9dd4361.
Completed tickets and retained receipts are checkpointed at
`9083275fc80a1f66ddc7a7c7ddca149692123a5a`.
The complete deterministic native actual-C proof, infrastructure lifecycle,
deliberate retry/repair, bounded failure/accounting, confinement and recovery
gates pass. The complete managed proof now passes exact original queue/ticket/
progress publication and byte-identical accepted pending recovery with zero
producer allocation. Both cleanup passes, measured target/helper retirement and
post-retirement retained authentication are complete. Tickets 02–07 are resolved.
See [final completion](completion-20261014.md),
[recovery handoff](recovery-20261013-second.md),
[managed checkpoint](managed-gates/actual-c-managed-11-checkpoint.json) and
[47 final references](verification/completion-f845516.references.json).

The historical three-original Ceres migration package was separately
`validated-with-blockers` before activation.
[Preparation checkpoint](production-preparation/preparation-checkpoint-27b8b0a.md)
records the then-outstanding Submitted/linked Integrate mechanism, per-claim
installation, catalog preparation, canonical work, owner reconciliation and
separate switch decision. The current `ceres-production-01` and
`canonical-production-01` packages preserve those originals under active checked
ownership. Missing acceptance producers remain explicit; historical preparation
reports should not be read as current activation status.
