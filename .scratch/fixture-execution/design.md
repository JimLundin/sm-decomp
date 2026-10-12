# Proposed fixture execution module

Status: proposed

Source facts: [ticket01](issues/01-execution-facts.md).
Decisions: [ticket02](issues/02-execution-design.md).

## Behavior

One deep fixture execution module owns dependency scheduling, Rust-before-C
admission, isolated execution, common evidence handling and blocked descendants.
Existing family, callback and PLM adapters retain routine invocation and
adapter-specific comparison. These are real adapters at an existing seam;
the design does not introduce a separate hypothetical mock adapter.

Independent chains execute concurrently within the existing shared worker
budget. A continuation becomes runnable only after its predecessor passes all
comparisons. Any failure blocks descendants with an explicit causal record;
independent chains continue. Results retain canonical manifest order.

A separate single-threaded helper loads data once, then forks an isolated child
for each subject invocation. The Rust side executes and passes admission before
the C invocation starts. The helper parent never executes a fixture, receives
fixture state mutations or reloads unchanged data. Copy-on-write keeps globals,
heap and ROM mutations inside each child. Continuations obtain verified state
through the fixture module, never by inheriting a previous child process.

Ordinary/projectile helper setup retains raw ROM. Callback/PLM setup retains
their independently initialized patched-ROM/emulator/SPC state. Fork occurs in
the helper, away from multithreaded orchestration and telemetry.

The module must hide helper startup, bounded framing, request identities,
completion, process groups, timeout/cancellation, child accounting and cleanup.
There is one waiter for each process; helper totals and fixture child CPU are
reported with explicit semantics to avoid double-counting. Failed or incomplete
replies cannot become a passing fixture or trigger an implicit C retry.

## Evidence proposal

Retain complete per-case WRAM, returns, projectile host words, callback captures,
stdout/stderr and command/timing identities. Record blocked descendants and their
failed predecessor. Aggregate fixture metrics instead of durably rewriting a
separate metrics scope around every tiny process. Generic exec commands retain
their existing receipt behavior. Historical sealed artifacts remain unchanged.

Seal new helper/scheduler implementation and generated-host inputs explicitly.
Reauthenticate evidence after scoped cleanup using the established reader.

## Investigation proposal

First build a bounded throwaway Rust prototype for ordinary/projectile fixture
execution, including representative chain continuations. Compare:

1. Current fresh-process baseline.
2. Fresh-process execution with prompt completion and concurrent chains.
3. Preloaded helper with fresh fork children and concurrent chains.

Use identical fixture states, ROM and routine implementations. Keep each
experiment near or below 60 seconds; include startup, data loading, execution,
evidence handling and cleanup in the compared wall interval. Distinguish CPU
from wall time and report host load instead of promising a speedup. Reorder and
repeat fixtures to expose leaks; include deliberate state mutation, Rust
refusal, crash, timeout and cancellation controls.

Callback/PLM helper initialization is source-feasible but needs its own evidence
before rollout. Extend the common module to these adapters after the first
prototype establishes useful savings and isolation. The fresh execution adapter
remains an explicit comparison/control path; a failed helper does not silently
rerun a rejected fixture through another path.

The selected production change needs complete affected-family verification and
relevant admission controls, followed by live integration as required by the
changed evidence contracts. Repeated full-route performance sampling is outside
this investigation. Every run requires scoped cleanup before completion.

## Depth and test surface

The caller supplies declared fixture work and its existing adapter. Tests use
the same module interface and real local fixture processes. The implementation
owns ordering, isolation and completion rather than making callers learn those
facts. This produces locality for failure/cancellation rules and leverage across
the three adapter kinds.

Preserve existing full-state/admission/source-mutation controls. Add interface
tests for concurrent independent chains, blocked descendants, pristine-parent
isolation, malformed/truncated replies and descendant teardown. Internal
refactors should not require changing these behavioral tests.

Public Rust types and exact request framing remain to be designed after scope
and evidence retention decisions are settled. The bounded prototype is complete;
[results](results.md) record measurements, controls and cleanup. No production
changes have been made.
