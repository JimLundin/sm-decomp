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
| [08 — Reviewed production switch](issues/08-production-switch.md) | task | open | 07 |
| [09 — Model rewrite and one-command launch](issues/09-model-rewrite-launch.md) | task | resolved | None |

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
- Existing active production ownership remains with the current background
  coordinator; this branch does not change it.
- The direct model connection is the Foundry Responses endpoint using the
  existing `FOUNDRY_TOKEN`. Rust owns the ordered launch and all supervisor steps;
  immutable HTTP intents prevent silent reevaluation after a crash. See
  [operator guide](../../docs/TRANSLATION_EXECUTION.md#one-command-foundry-launch)
  and [model launch checkpoint](model-launch/checkpoint.md).

## Current checkpoint

Model rewrite and the one-command ordered launcher are committed at
`93e9d55b19630ac8fa2e3526ba963296385ea516`. Ticket 09 is resolved with a real
Foundry/native lifecycle, one model call, four passing actual-C gates, ordered
recovery tests, retained authentication and complete measured cleanup. The
[model launch checkpoint](model-launch/checkpoint.md) binds exact source/readers,
evidence and the prepared user command. That command reports `ready` for the
approved isolated enemy-jump instructions. Ticket 08 and the production switch
remain open; main authority, claims, source, ledgers and progress are preserved.

Created `/home/jim/sm-decomp-translation-execution` at baseline
`3a8b9ea28b716d90df55f798941bba68e1df6531` on
`architecture/translation-scope-execution-20261013`.
The user confirmed the complete contract. Tickets 02–04 are assigned to disjoint
Rust implementation ownership; root owns pipeline adapter/CLI and integration.
No production mutation is authorized before the reviewed switch.

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

The real three-original Ceres migration package is separately
`validated-with-blockers`; no production policy or migration was activated.
[Preparation checkpoint](production-preparation/preparation-checkpoint-27b8b0a.md)
records unsupported Submitted/linked Integrate sequencing, per-claim installation
and catalog preparation, untouched canonical work, current-owner reconciliation
and the separate switch decision required by ticket 08. Final production HEAD,
authority, both ledgers and progress match the saved baseline. Ticket 08 is the
next open dependency-ready work; approval must wait for concrete supported
per-original preparation.
