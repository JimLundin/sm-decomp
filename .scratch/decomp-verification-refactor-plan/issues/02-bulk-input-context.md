# Add a bulk Interface with one authenticated input context

Status: claimed
Blocked by: None
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

Tested foundation `b1f869102ac69897be674acfef682e3d867c6be6` passed 48 owner,
five catalog and four public ordinary/Native controls. One catalog parse, lazy
declaration decoding and shared captured facts are exercised through the public
Interface; the representative control records four Rust scans and 19 reuses.
Both cleanup passes reclaimed 4,980,736 bytes. See
[bulk-context-handoff.md](../bulk-context-handoff.md) for exact contracts and tools.

Root integrated it into `45b89f8`, retired the implementer worktree and removed
its disposable Cargo target. Independent ordinary and Native readers passed
afterward, with unchanged tool seals. Combined owner and Native capture controls
also passed. Exact retirement, bytes and proof pointers are in
[integration-foundation.md](../integration-foundation.md). Final complete Native
gates and ticket 01 baseline acceptance remain pending; this ticket stays claimed.

## Comments

2026-10-15 measured correction, explicitly approved by the integration owner:
the real 177-family capture has 6,967 distinct inputs / 477,164,947 bytes,
including 7,104,059 compiler bytes. Complete materialization succeeds at
1,046,226,292 charged I/O bytes. The following full freshness pass exhausts
the original single 1-GiB scope after another 27,394,048 bytes. Peak resident
reservation is only 347,912,221 bytes; the failure is aggregate I/O, not memory.

Ordinary capture now has two explicit phases: materialization and full
freshness/materialized authentication. Each keeps a 1-GiB ceiling and both use
the same original absolute deadline. With no enclosing scope their combined
ceiling is at most 2 GiB; any existing caller byte/deadline scope must still
aggregate both phases and remain stricter. Native and declaration resident
limits do not change. No input or freshness check is omitted and counters are
never reset per file. This is a reviewed composition correction, not a claim
of less I/O or faster execution. Complete real capture and inherited-budget /
deadline controls are required. Deferred noncompiler materialization remains
a future optimization requiring its own byte-identity and mutation evidence.

2026-10-15: full-catalog graph-capacity validation exposed a bulk capture defect:
the raw GeneratedPlm manifest names an obsolete synthetic ROM that ordinary
preparation should replace through the trusted generator. Its missing/stale file
must not block capture of the effective required inputs. A subsequent diagnostic
also hit aggregate input accounting before scheduler reservations. Root assigns
the narrow production-seam correction and public regression to the original bulk
owner in `/home/jim/sm-decomp-performance-generated` from `87fb368`; exact ownership
is in the checkpoint. Preserve the original registration, complete current
generated domain, base ROM identity, source freshness and existing limits.
This must pass through the real full-catalog capture path before final freeze.

Do not introduce a daemon, public editable graph or second authority ledger.

2026-10-15 scheduling refinement: the metrics interface is frozen at
`e08619089313c187dbaacc279a4a5c36adc19007`, and its core actual-C reuse regression
has passed. The independent bulk-context implementation can now proceed in an
isolated checkout while the final bounded-metrics controls and Native dependency
recovery complete. This removes the implementation scheduling blocker only;
acceptance and resolution still require ticket 01's final controls and the
recovered extraction gates. No in-flight verifier source or main production
source is edited by this provisional worker.
