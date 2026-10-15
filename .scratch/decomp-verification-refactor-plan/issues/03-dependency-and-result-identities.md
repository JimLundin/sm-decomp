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

Final source guard `2b2712b`, handoff `095d952e711c3ecbcb857471dbef9abe203b760b`:
the complete 177-family corpus passes under unchanged budgets. Documentation
excludes 174 and leaves three uncertain (7.77 seconds); audio behavior, rename
and deletion each select exactly 165 and exclude 12 (5.03/2.78/2.74 seconds).
Remaining uncertainties are generated PLM, the unknown Ridley-subsystem adapter
and the captured Ridley external dependency without changed-since history.
The corpus records 45 Rust parses, 437 closure hits and one final asset pass.
See [dependency-facts-handoff.md](../dependency-facts-handoff.md) and
[dependency-identities.md](../dependency-identities.md).

Both cleanup passes reclaimed zero disposable and 262,144 deduplicated bytes.
The corpus worktree was retired while retaining 223 exact report inputs; all four
reports reopened afterward and wrong-source readback refused. Root also retired
the implementation worktree; both matching old/new actual-C result readers passed
independently and all proof seals remained valid. The target transferred to 07.
Exact proof paths and retirement allocations are in
[integration-foundation.md](../integration-foundation.md). Final acceptance waits
for 02 and the combined required gates.

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
