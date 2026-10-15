# Extract complete Offline family verification

Status: open
Blocked by: 02
Triage: ready-for-agent
Type: task

## Scope

Implement stage 2 of [the approved plan](../implementation.md). Own
`offline-verification`, Fixture values/codec/catalog data, ordinary and Native
verification, recipes/templates, scheduling, family inputs, candidate ownership,
indexed receipts and admission. Consolidate invocation expectations so both
execution adapters and authentication consume one recipe owner.

Keep public calls small, prepared state opaque and execution-adapter details
private. Translation scope approval remains a caller-owned narrow adapter.
Preserve complete readiness, indexed attribution, deadlines and no-relaunch
retry behavior. Do not add historical readers or arbitrary execution hooks.

## Validation

Compile through declared Cargo dependencies. Migrate and run existing verifier
behavior tests through the new interface. Retain CLI protocol tests in pipeline.
Full real Native proofs and final integration are ticket 05 responsibilities.

## Comments

- Approved design: [proposal](../proposal.md). Source ownership and worktree
  will be recorded before this ticket is claimed.

## Completion evidence

Pending implementation, checks and scoped cleanup.
