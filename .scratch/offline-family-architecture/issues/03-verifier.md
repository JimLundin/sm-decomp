# Extract complete Offline family verification

Status: claimed
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
- Implementation starts from the compiling foundation checkpoint `446dd64`.
  Ticket 02's final verification/integration remains required before resolving
  this ticket. Root owns ordinary extraction and public/source interfaces in
  `/home/jim/sm-decomp-offline-verifier`; `/root/gameplay_architecture` owns the
  new common recipe files in `/home/jim/sm-decomp-offline-recipes`;
  `/root/translation_architecture` owns Native extraction and its application
  bridge in `/home/jim/sm-decomp-offline-native`. See the checkpoint for exact
  writable groups. These worktrees do not change main's baseline test inputs.

## Completion evidence

Pending implementation, checks and scoped cleanup.
