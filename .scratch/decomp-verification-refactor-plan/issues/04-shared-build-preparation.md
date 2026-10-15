# Share eligible compiler preparation and artifacts across families

Status: open
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

Pending. Record compiler/process counts, build-hit reasons, full-workload effect,
source/flags/artifact identities, negative controls and scoped cleanup/admission.

## Comments

Common reference images are a measured design choice, not an all-game binary mandate.
