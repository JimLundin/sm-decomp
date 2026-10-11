# Conditional promotion and native retention documentation

Status: open
Blocked by: 07
Triage: ready-for-agent
Type: task
Phase: conditional: binary selected in 03

Promote only if exact gates pass, warm median 45k full wall improves at least 15%, and aggregate sampled peak RSS and allocated staging/private peak each grow at most 5% at identical capacity within frozen absolute budgets. A smaller gain or resource exception needs a recorded owner decision with measurements. Otherwise retain opt-in.

## Acceptance

Keep text rollback, immutable historical readers/verdicts and source/protocol receipts. Use native cleanup/dedup, authenticate retained subject/store dependencies and run live-authenticate only when a complete verdict exists; preserve failed-run diagnostics and report incomplete authentication; record reclaimed/free bytes and dependencies. Remove only integrated implementer worktrees after preserving exact cited sources.

Follow the complete contract in [plan.md](../plan.md). Record source revision,
controls, measured results and scoped cleanup under Completion evidence before
resolving. Implementation is pending.
