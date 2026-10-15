# Share eligible compiler preparation and artifacts across families

Status: claimed
Blocked by: 03
Triage: ready-for-agent
Type: task

## Scope and ownership

Own shared compiler preparation/cache extensions in `tool-execution` and build
recipe descriptions in `offline-verification`. The integration owner coordinates
shared recipe registration; storage/protocol work has separate ownership.

## Required work

Extend the existing preparation/cache implementation across compatible families.
Share authenticated compiler/runtime identity and frozen inputs; coalesce identical
eligible build requests under the existing key lock. Do not add another cache.
Keep input rechecks after waits, actual emitted dependency validation, artifact
authentication and atomic publication. Idle cache waits hold no producer lease.

Identify avoidable run-path variation in generated sources. Replace it only where
equivalence across physical directories is established; include semantic paths,
flags, target, environment, ABI, wrappers and linked runtime in the key. Compare
narrow build groups against any proposed shared image; measure cold-build savings
and broader incremental invalidation before selecting grouping.

Initially preserve all Native compiler producer slots. If sharing a compiled
artifact across those slots is warranted, specify and test a new current producer
contract with explicit build dependencies and reservations. Never report reused
compilation as a fresh compiler invocation.

## Acceptance

One preparation per compatible context and one build per eligible unique key.
Correct cache hits under two concurrent families; wrong bytes, compiler changes,
new includes, path-sensitive code and mutation while waiting refuse. All required
actual-C/Rust calls remain. Native provenance and capacity controls pass for any
revised build contract. Cold and warm measurements include discovery and sealing.

## Completion evidence

Implementer checkpoint `fb1cd4963711416cf7bb49e00e6ab91681641c61` is clean.
Production correction `cb00f32`, complete gates `330e8c0`: 13 actual controls
passed in 35.02 seconds. Shared compiler identity preparation removes one of
two version queries; three alternating cold/warm pairs show no reliable elapsed
improvement for this small workload. Final integration must measure its batch
effect. Semantic path, volatile macro, actual source-location, environment,
lock-wait mutation, corruption and hard-budget controls are covered.

Both proof cleanup passes reclaimed zero bytes; 3,454 retained files, 66 copied
cache artifacts and 63 actual programs authenticated afterward. Exact evidence
and retained dependencies are in [shared-preparation-handoff.md](../shared-preparation-handoff.md).
Worktree retirement and combined acceptance remain with root. The writer-free
target is now exclusively reassigned to provisional ticket 08; no retained
evidence depends on it and no cache was cleared.

Pending. Record compiler/process counts, build-hit reasons, full-workload effect,
source/flags/artifact identities, negative controls and scoped cleanup/admission.

## Comments

Common reference images are a measured design choice, not an all-game binary mandate.

2026-10-15: isolated implementation starts from ticket 02's frozen generic C
dependency-validation seam `b333ca5c488e9f923c5672c321284f870f8fbede` while
ticket 03 completes its independent source facts. This refines the implementation
dependency only: final acceptance still requires complete 02/03. Sharing immutable
compiler facts, with per-family cancellation and existing cache identities, does
not require narrowing source dependencies. New eligible artifacts remain gated
on exact path, emitted-input and environment proofs.

Owner: `/root/shared_preparation`, under integration session
`01a13e7f-ab01-7413-a1c2-288b08a4b719`. Worktree:
`/home/jim/sm-decomp-performance-builds`, branch
`performance/shared-preparation-20261015`. The worker owns
`crates/tool-execution/src/builds.rs`, `build_cache.rs`, related scoped tests and
its private handoff. Offline recipe/context attachment is coordinated with
`/root/bulk_context_resume`; main registration and integration stay with root.
Exclusive released build target: `/home/jim/sm-decomp-offline-verifier/target`;
retained Native tools are independent copies and no build writer survives there.
Private proof root: `/home/jim/sm-decomp-performance-builds-proof`.
