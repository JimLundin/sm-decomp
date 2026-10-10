# 10: Assemble readiness gates, atomic acceptance and safe recovery

**What to build:** A repository-root readiness command runs all required gates, produces one honest brief, and advances a recoverable accepted state only after a complete pass.

**Blocked by:** 05: Validate every enabled timed pattern through the real step path; 08: Diagnose failures with predecessor and disabled-entry controls; 09: Reuse evidence only when its dependencies remain unchanged.

**Status:** ready-for-agent

- [ ] Expose repository-root readiness, iteration validation entry and diagnostic replay commands through just using existing generation/comparison machinery; no user-assembled command sequence is required. Full autonomous increment selection/expansion belongs to the iteration specification.
- [ ] Run doctor, check, lint, enabled-pattern instruction/timing validation, feedback/cache validation, cold fallback and current generated parity, baseline per-key coverage and fresh independent synchronization for the agreed initial domain.
- [ ] Retain all mandatory-check outcomes when multiple failures occur, with exact scopes, expected/completed frames/cases, evidence links and terminal pass/divergence/uncovered/error/incomplete classification.
- [ ] Write terminal verdict/brief and print their locations; return 0 for pass, 1 for validation failure and 2 for infrastructure/incomplete failure while preserving underlying stage codes.
- [ ] Atomically update the accepted pointer only when every mandatory gate completes and passes, binding exact run/domain/restorable source/config; retain predecessors and rejected candidates. Acceptance never implies a Git commit.
- [ ] Restore accepted or selected candidate snapshots into an isolated checkout or explicitly selected candidate without overwriting unrelated user edits.
- [ ] Report ready/not ready, exact source/domain, comparable coverage/fallback progress, supported patterns, timing limits/debt and evidence links. Time overruns are reported; actual timeouts are incomplete and block acceptance.
- [ ] Any missing mandatory gate, including feedback demonstrations completed in the following ticket, yields incomplete and blocks acceptance; integration must never temporarily waive a gate.

