# Offline family architecture work map

| Ticket | Type | Responsibility |
| --- | --- | --- |
| [01](issues/01-design.md) | grilling | Shared module design and concrete verification criteria |
| [02](issues/02-foundations.md) | task | Extract tool-execution and decomp-source |
| [03](issues/03-verifier.md) | task | Own ordinary and Native verification in offline-verification |
| [04](issues/04-callers-and-contracts.md) | task | Migrate callers, source declarations and conservative selection |
| [05](issues/05-verification-and-cleanup.md) | task | Verify current behavior, integrate and clean retained runs |

## Decisions-so-far

- Candidate 01 selected; [design tree](design.md) holds settled facts and the
  current question frontier.
- Preserve current verification checks and the separate installed-controller
  identity rule. Historical compatibility is outside the selected scope.
- Compare three designs in [the recommendation](proposal.md). Choose separate
  Offline family, tool execution and Recovered C source owners, with the simple
  ordinary interface and opaque Native preparation.
- [Migration and verification](implementation.md) cover complete ordinary and
  Native ownership, shared helpers, conservative source selection and cleanup.
- The user approved implementation on 2026-10-15 after reviewing the diagrams.
  The shared-understanding checkpoint is complete; do not ask again.
- [Checkpoint](checkpoint.md) records current source, ownership and next work.
