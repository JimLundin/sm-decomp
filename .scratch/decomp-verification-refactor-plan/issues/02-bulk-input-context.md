# Add a bulk Interface with one authenticated input context

Status: open
Blocked by: 01
Triage: ready-for-agent
Type: task

## Scope and ownership

Own the bulk facet and private input/catalog context in `offline-verification`,
its `pipeline::batch` integration and necessary bounded/source-context extensions
in `tool-execution`/`decomp-source`. Keep the extraction's dependency direction.
Assign exact paths from the frozen extracted source before implementation.

## Required work

Keep standalone verification simple and add bulk verification/admission for a
complete application-selected requirement. Share the parsed catalog, declarations,
source facts and authenticated immutable bytes within one invocation. Payloads
load lazily against bound identities with explicit memory/byte/deadline limits.
Preserve Native prepared handles, scope-validation timing and pending outcomes.

Workers consume a coherent captured source/input set. Keep namespace, compiler
dependency, post-wait and current-source publication checks. On changed input,
refuse instead of silently mixing revisions or restarting a Native attempt.
A remembered digest, read-only mode or mmap is not an immutable snapshot.

First preserve current evidence formats and recipes. A metadata-only refinement
must bind metadata and payload identity; distinguish avoiding decode from avoiding
the full-file read needed to validate a monolithic Fixture container.

## Acceptance

Single-family and bulk results have the same complete canonical observations and
source/admission meaning. At most one full declaration decode per unique manifest
occurs in a batch. Tampering, newly discovered input, source mutation and exhausted
budgets refuse. Independent admission needs no compiler or invocation cache.
Public tests exercise ordinary and prepared Native callers through their Interface.

## Completion evidence

Pending. Record removed reads/decodes, bounded cold/warm timings, complete controls,
source/tool identities, scoped cleanup and post-cleanup authentication.

## Comments

Do not introduce a daemon, public editable graph or second authority ledger.
