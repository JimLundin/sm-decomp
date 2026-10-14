# Implement restricted Rewrite capsule exchange

Status: resolved
Blocked by: 01
Triage: ready-for-agent
Owner: /root (recovered restricted rewrite integration)

Own bounded Rewrite capsules and Source replacements, path/hash validation and
the production rewrite adapter. The supervisor applies/commits exact source.
Restrict the actual worker invocation; current inherited collaboration tools
cannot satisfy this requirement. See [the spec](../spec.md).

## Acceptance

Scripted rewrite adapter and real denial tests prove stale/foreign/oversized
replacements reject before mutation, protected files and host verifiers remain
inaccessible, and missing enforcement refuses launch. The worker receives only
approved source/context and a bounded result channel. Candidate compiler/runtime
confinement is coordinated with tickets 04/05.

## Completion evidence

Implementation is checkpointed at `27b8b0a`. The supervisor invokes the installed
rewrite executable in declared-input Linux namespaces, passes only the immutable
Rewrite capsule and a bounded structured response, validates original hashes and
approved relative paths, then applies/commits exact source itself. Verification
declarations, host tools, credentials, queue and evidence remain outside worker
capabilities. Launch refuses when the required enforcement is unavailable.

`verification/supervisor-core-final-02.stdout.log` passes eight replacement and
six restricted-adapter tests, with eight confinement and fourteen Git tests.
They include foreign/stale/aliased replacements, protected input and host-tool
denial, actual timeout/output enforcement and immutable declared context.
The complete native actual-C proof executes one restricted recorded rewrite;
[native checkpoint](../supervisor-native/actual-c-02-checkpoint.json) retains its
exact installation and evidence. The new managed diagnostic proof exercises an
installed Rust protocol worker with `recorded = false`.

The complete managed proof passes with an installed Rust protocol worker using
`recorded = false`. Its disposable Cargo build is locked/offline with pinned
`serde_json = 1.0.151`; the trusted controller applies its Source replacements.
Verified source is `f845516b975c3bfc413800c6bed9e329696a79e6`.
Both scoped cleanup passes, measured target/helper retirement and post-cleanup
authentication pass. [Final completion](../completion-20261014.md) and the
[managed checkpoint](../managed-gates/actual-c-managed-11-checkpoint.json) retain
the exact evidence and diagnostic limitations. Unrestricted collaboration
workers remain outside the admitted rewrite interface.
