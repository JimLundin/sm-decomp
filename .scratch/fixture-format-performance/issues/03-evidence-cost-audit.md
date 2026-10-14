# Audit evidence costs and commit the binary migration

Status: claimed
Blocked by: 02
Triage: ready-for-agent
Type: task

## Scope

The user prioritizes performance, accepts incompatible formats, and authorizes
individual commits directly on `main`. Audit whether retaining evidence imposes
avoidable work in Fixture loading, execution, emission or authentication. Keep
the meaning and integrity of Fixture pairs and Fixture chains. Distinguish
archived evidence storage from work on active execution paths.

Commit the completed migration in coherent groups, then commit any justified
performance improvements separately. Record what fidelity was preserved, which
textual details were normalized, and any remaining performance tradeoffs.

## Validation

- Trace active evidence reads, copies, hashes and serialization.
- Measure any proposed optimization against the committed implementation.
- Exercise affected authentication and execution contracts, including negative
  controls; do not weaken evidence validation to improve timings.
- Complete scoped cleanup for any new runs and preserve their dependencies.

## Comments

The initial migration and verification are complete in ticket 02. Its historical
results and sealed artifacts remain unchanged. This follow-up audit starts from
the uncommitted migration on `main` at
`f39947c4a1e45c382deb5c842934c92bb28b1fe4`.
