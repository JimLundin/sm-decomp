# Refactor whole-batch verification for performance

Status: approved; implementation active
Requested: 2026-10-15
Planning baseline: `5c011eb5d02a54d74903715bffb3f8c15ab78c51`

The user requested a refactor plan for slow whole decomp batch / integration
verification, explicitly welcomed significant architecture changes, and then
instructed "please go ahead and implement". Implementation follows this plan.
No installed policy or achieved verification stage changes merely on that authorization.

## Recommendation

Build a bulk verification Module in `offline-verification` that owns one frozen
input/catalog snapshot and a private typed graph of required work for the batch.
Reuse compiler preparation and eligible artifacts across families. Write Offline
evidence as indexed immutable bundles during execution, with bounded staging and
independent authentication. Keep the existing shared worker pool and the existing
Translation journal as their respective capacity and attempt authorities.

This builds on the approved
[Offline family extraction](../offline-family-architecture/proposal.md).
It is a performance follow-up, not a competing extraction. Moving source into
new crates alone does not remove repeated preparation, broad dependency
selection, per-case file traffic or sequential Native admission.

The significant changes are:

1. One authenticated input/catalog context for the complete batch, with lazy
   Fixture payload loading and explicit current-source checks.
2. A private work graph that coalesces eligible build work and schedules ready
   work across families under the existing capacity and frozen ordering rules.
3. Indexed Offline evidence produced directly, replacing repeated directory
   inventories and large persistent trees of individual state files.
4. Separate computation identity from attempt/admission identity so build reuse
   can be precise without weakening production authorization or provenance.

Accurate dependency discovery, useful timing, and optimized installed tools are
supporting changes. A daemon, remote execution and cached C outputs
are not part of the recommended first implementation.

## Where each change helps

These are expected effects of removing work, not measured speedup claims.
Implementation scope is relative to this refactor and excludes the extraction
already underway.

| Change | Complete fresh verification | Incremental or retained-result runs | Implementation scope |
| --- | --- | --- | --- |
| Shared input/catalog context | Removes repeated discovery, decoding and eligible preparation | Same savings during planning and admission | Medium |
| Precise dependencies and identities | Enables safe build sharing; does not reduce mandatory cases | Avoids unnecessary selection and invalidation | Medium |
| Shared build preparation/artifacts | Avoids duplicate eligible compilation; cold runs still build distinct artifacts | Reuses unchanged eligible artifacts | Medium to large; Native producer changes are a separate gate |
| Direct indexed evidence | Reduces small-file traffic, repeated inventories and state reads | Makes complete retained-result authentication and cleanup cheaper | Large; prototype decides storage and transport |
| Private work graph | Keeps available capacity busy with independent families and ready cases | Reconciles existing work and schedules remaining work | Large; existing ordering and reservation contracts matter |
| Optimized installed tools | May reduce hashing, parsing and comparison CPU time | May reduce startup and admission overhead | Small to medium code change, with full installation validation |

Instrumentation comes first so implementation effort follows the measured cost.
The most substantial rearchitecture is direct evidence production plus a shared
batch work graph. The shared context and identity work provide their foundation.

## Evidence and limits

The [performance assessment](../decomp-batch-performance-20261015/findings.md)
found 160/177 families selected with no changed verification inputs. First
uncertainties for 158 families arise from `include` in two documentation comments.
Selected families may authenticate and reuse results; this is not a count of
recompilations. The final reduced selection after a proper fix is unmeasured.

The current ordinary planning path builds the catalog three times. Ordinary
family verification inventories its run, then batch admission inventories it
again and reads case snapshots. Ordinary execution already has pristine fork
workers, ready queues, binary Fixtures and authenticated C artifact caching.
Native Original acceptance currently iterates families sequentially; Native
Fixture execution joins each bounded wave. Existing Native operation indexes
already compact attribution, and must be reused rather than reinvented.

The previous probe's 333-MB debug executable hashes itself before most command
dispatch. Stripped startup/catalog samples improved, but the stripped planning
comparison timed out under concurrent compilation. No new whole-batch speedup
has been established. Historical incomplete metrics cannot supply a current
performance baseline.

## Workloads and success

Primary success is less elapsed time for the **same complete required integration
domain**, including preparation, actual producers, authentication, publication
and cleanup. The proposal must benefit full-catalog work, not only a smaller
changed-family selection.

Also measure unchanged retries and narrow edits. Separate these contracts:

- Full required domain with fresh producers: run every prescribed Fixture pair;
  reuse compiled artifacts only when that contract permits it.
- Full required domain allowing accepted-result reuse: authenticate every
  required retained result with current inputs; no replacement execution when
  reuse is valid.
- Incremental selection: omit only families proven unaffected.
- Same-attempt recovery: reconcile existing intents/results without creating
  another producer or extending a saved deadline.

Each phase has structural and semantic acceptance criteria in
[validation.md](validation.md). Latency targets are set from the baseline and its
variance before a candidate comparison. File count, byte volume and compiler
invocation reductions are reported separately from elapsed-time improvement.
No promised speedup multiplier is part of this proposal.

## Constraints

- Rust implementation, orchestration and tests; no Python dependency.
- Preserve full WRAM, returns, host state, capture expectations and ordered
  callback/hardware observations required by each recipe.
- Preserve Rust-before-C admission, Fixture chain inheritance, pristine child
  isolation, bounded output/read/memory/deadline accounting and descendant cleanup.
- Required Live replay remains fresh and keeps its complete prescribed horizon
  and ownership/effect gates. Original-ROM parity remains a separate requirement.
- Unknown dependencies remain affected. A claimed hash or filesystem timestamp
  is not proof of content or admissibility.
- Preserve complete Native producer attribution, source/claim lineage, current
  attempt identity and installed controller/sandbox identity.
- Independent admission and post-cleanup authentication remain independent;
  an in-memory cache is never a retained evidence dependency.
- Preserve the existing four-process shared capacity while comparing designs.
  Additional scheduler threads do not create additional producer capacity.
- Do not add historical-format readers, receipt conversion utilities or a
  parallel production ledger. A format change updates current producers and
  consumers together; existing frozen evidence is not silently rewritten.

## Delivery and dependencies

Read [architecture.md](architecture.md) for the selected design, alternatives,
Interface and ownership. [map.md](map.md) lists nine dependency-ordered tickets;
ticket 01 is claimed. Their order does not confer ownership of other efforts.
Current execution and ownership are recorded in [checkpoint.md](checkpoint.md).

Before implementation, reconcile the
[active extraction checkpoint](../offline-family-architecture/checkpoint.md),
finish its required verification/cleanup, freeze its integrated source, and use
the repository's session/ownership procedures. Its main Git/index and worker
worktrees remain owned by that effort. Existing production prerequisite stops
are not resolved by this plan.

Startup reconciliation observed the extraction coordinator stopped with a session
error while its final compiler child remained alive. Its complete clean source is
committed at `8bf8acf610e1a94a1a3bccd5d718dae4a4244857`. Isolated provisional
implementation may proceed from this commit while the existing final build and
gates are recovered. Acceptance, baseline conclusions and main integration remain
dependent on completing those gates; the frozen verifier checkout is not edited
by performance workers.

Two explicit contract changes need their own verified cutovers: packed Native
output references, and any Native sharing/scheduling that changes required
compiler slots, reservation groups or failure ordering. Initially preserve those
obligations. Performance is not grounds to reinterpret an old frozen policy.

## Plan completion evidence

Three independent designs were produced through the codebase-design skill:
[bulk verifier](designs/01-bulk-verifier.md),
[durable action graph](designs/02-action-graph.md), and
[local session](designs/03-local-session.md). The recommendation combines the
bulk Interface with a private graph and existing durable operation records.
The daemon remains a measured extension.

Planning used source/document reads only. No compilers, verification producers,
worktrees, shared caches, installed tools or existing claims were changed.
There are no new verification-run artifacts to clean up. Earlier assessment
cleanup is recorded in its own resolved ticket. Implementation tickets require
their own measured cleanup and post-cleanup admission evidence before resolution.
