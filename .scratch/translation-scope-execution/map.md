# Translation scope execution

Status: implementing

Spec: [spec.md](spec.md). Selected candidate: Translation scope execution.

| Ticket | Type | Status | Blocked by |
| --- | --- | --- | --- |
| [01 — Confirm execution and acceptance contract](issues/01-execution-contract.md) | grilling | resolved | None |
| [02 — Durable attempts and supervisor interface](issues/02-durable-attempts.md) | task | claimed | 01 |
| [03 — Restricted rewrite exchange](issues/03-restricted-rewrite.md) | task | claimed | 01 |
| [04 — Enforced budgets and evidence finalization](issues/04-budgets-and-evidence.md) | task | claimed | 01 |
| [05 — Frozen preparation and verification adapters](issues/05-frozen-verification.md) | task | open | 02, 04 |
| [06 — Integration, progress and recovery](issues/06-integration-recovery.md) | task | open | 02, 05 |
| [07 — Complete restricted-path proof and cleanup](issues/07-restricted-path-proof.md) | task | open | 03, 04, 05, 06 |
| [08 — Reviewed production switch](issues/08-production-switch.md) | task | open | 07 |

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

## Current checkpoint

Created `/home/jim/sm-decomp-translation-execution` at baseline
`3a8b9ea28b716d90df55f798941bba68e1df6531` on
`architecture/translation-scope-execution-20261013`.
The user confirmed the complete contract. Tickets 02–04 are assigned to disjoint
Rust implementation ownership; root owns pipeline adapter/CLI and integration.
No production mutation is authorized before the reviewed switch.
