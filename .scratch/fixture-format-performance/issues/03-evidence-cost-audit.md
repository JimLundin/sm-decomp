# Audit evidence costs and commit the binary migration

Status: resolved
Blocked by: 02
Triage: ready-for-agent
Type: task

## Scope

The user prioritizes performance, accepts incompatible formats, and authorizes
individual commits directly on `main`. Audit whether retaining evidence imposes
avoidable work in Fixture loading, execution, emission or authentication. Keep
the meaning and integrity of Fixture pairs and Fixture chains. Distinguish
unnecessary repeated work from the checks that establish current correctness.

Commit the completed migration in coherent groups, then commit any justified
performance improvements separately. Record what fidelity was preserved, which
textual details were normalized, and any remaining performance tradeoffs.

## Validation

- Trace active evidence reads, copies, hashes and serialization.
- Measure any proposed optimization against the committed implementation.
- Exercise affected authentication and execution contracts, including negative
  controls; do not weaken evidence validation to improve timings.
- Complete scoped cleanup for new runs. Retire this session's archived bundles
  under the user's subsequent direction recorded in ticket 04.

## Comments

The initial migration and verification are complete in ticket 02. This audit started from
the uncommitted migration on `main` at
`f39947c4a1e45c382deb5c842934c92bb28b1fe4`.

The user subsequently rejected historical replay, old readers and migration
scaffolding. Ticket 04 removes those compatibility paths and retires the task's
archive bundles; they are not completion dependencies.

## Completion evidence

- `1777785` shares bounded reads across SHA-256 and full-state comparison,
  authenticates the exact decoded Fixture bytes, and reuses current artifact
  inventory hashes. No persistent input-identity cache was introduced.
- 227 pipeline unit tests, 19 family integration tests, one callback actual-C
  test and one PLM actual-C test passed: 248 in total. The suites retained 28
  existing ignored tests. Logs are under `../evidence-audit/`.
- Nine alternating optimized samples per workload are in
  `../evidence-audit/results.json`. Captured-input logical read volume halved;
  ordinary root-case WRAM authentication reads dropped by one third, including
  inventory. Timing changes were modest and overlapping, so these do not prove
  a whole-verification throughput gain.
- The WRAM tampering control also changes both subjects to matching wrong bytes
  and rebinds the artifact inventory; the sealed case hashes still reject it.
- The initial actual-C cleanup deduplicated 1,048,576 bytes and removed
  23,700,637 bytes of cache/Git metadata. Follow-up cleanup/deduplication returned
  zero remaining candidates across 12 snapshots. Final shared cleanup with
  ticket 04 succeeded: the private test root had zero remaining candidates;
  native runs deduplicated another 24,248,320 bytes, and the task's archived
  bundles plus disposable private build/run directory were removed (2,488,766,464
  allocated bytes). Final available disk was 163,732,357,120 bytes.
- Integration is on `main`; ticket 04's current native follow-up is `1c7e592`.
  No historical reader or source bundle is a retained dependency. Exact scoped
  cleanup commands and logs are recorded in ticket 04.
