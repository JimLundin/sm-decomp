# Implement restricted Rewrite capsule exchange

Status: claimed
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

Pending implementation, scoped commit, restricted-launch proof and cleanup.
