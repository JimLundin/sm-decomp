# Make dependency selection and computation identity precise

Status: claimed
Blocked by: 02
Triage: ready-for-agent
Type: task

## Scope and ownership

Own `offline-verification` dependency facts/source requirements and the affected
selection consumer in `pipeline`. Coordinate source inventories with
`decomp-source`; retain all caller-independent source operations there.

## Required work

Recognize supported Rust dependencies through syntax, plus authenticated compiler
dependency inventories and explicit runtime inputs. Words in comments or ordinary
string literals must not create dependency edges or unsupported-input refusals;
paths supplied to real include macros remain dependencies. Preserve exact source hashing.
Cover cfg/features, includes, macros, generated files, environment, C dependencies,
ROM/captures, aliases, additions, renames and deletions. Incomplete knowledge leaves
the family affected with an explanatory reason.

Define separate content, build, verification-computation and attempt/admission
identities as in [architecture](../architecture.md). Reuse existing dependency
records without compiling the whole catalog merely to decide selection. Preserve
broad source closure until controls prove a narrower contract; unrelated CLI
independence is a tested result, not an assumption. Installed controller identity
and complete frozen requirements remain separate mandatory checks.

## Acceptance

Known complete families are skipped for unrelated documentation; real transitive
changes select all affected families. The two observed `include` comment cases
cannot trigger uncertainty. Genuine unresolved dependencies remain affected.
Demonstrate missing/stale closure and concurrent mutation refusal. Reused result
lookup cannot accept a wrong requirement, candidate, comparison implementation,
recipe, producer identity or claim/attempt.

## Completion evidence

Pending. Record selected/skipped counts and every remaining uncertainty on the
current corpus, planner reads/time, mutation controls, exact source and cleanup.

## Comments

Do not promise that fixing the first 158 refusals eliminates all later uncertainty.

2026-10-15 scheduling refinement: the pure owned-source-facts implementation may
start independently from ticket 02's coherent catalog checkpoint `94104cc` while
captured compiler-input plumbing is finished by its owner. The scanner accepts
source bytes and emits owned facts; this slice does not depend on mutable compiler
capture state. Integrate its consumers with the next frozen ticket 02 checkpoint.
Final acceptance, complete identity integration and ticket resolution still depend
on 02; this does not promote its provisional source into accepted evidence.

Assigned isolated worktree: `/home/jim/sm-decomp-performance-dependencies`, branch
`performance/dependency-facts-20261015`, baseline
`94104cc224b6997c7eb1ad946be451af168287b5`. Root owns main and frozen combined-source
verification. The worker owns its scanner, dependency consumers, identity
integration and scoped tests, with a durable handoff in this plan directory.
