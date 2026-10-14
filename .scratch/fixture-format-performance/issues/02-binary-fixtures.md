# Make binary fixtures authoritative

Status: resolved
Blocked by: 01
Triage: ready-for-agent
Type: task

## Scope

Implement a Rust Fixture module with bounded typed binary metadata, indexed merged
spans and shared bytes. Migrate active Offline family source fixtures and all readers,
generators, catalogs and tests. Add inspect/import tooling for editing and review.
Remove repeated fixture payloads from case/preparation records. Preserve captured-state,
continuation, adapter and evidence semantics. No runtime TOML-fixture fallback.

Do not modify installed production policies, historical ledgers, retained evidence,
other worktrees or gameplay implementations.

## Validation

- Compare complete metadata and full prepared WRAM across the migrated corpus.
- Test malformed binary extents, version, optional data, overlap and continuations.
- Exercise current actual-C family and adapter tests plus fresh authentication.
- Measure production input loading on the same three study families.
- Complete scoped cleanup and retain exact evidence sources/results.

## Completion evidence

Implemented indexed, typed little-endian fixtures, shared raw spans, atomic
authoring import, inspection and case-aware diff. Removed the duplicated TOML
projection parsers. Family runs retain binary manifests and compact case hashes;
prepared native records reopen sealed binary input instead of serializing WRAM
payloads. Readers, catalogs, generators and standalone camera/trail producers
publish and consume the new format. Small configuration and verdict records
remain metadata documents.

`../results-production-migration.json` records the complete migration;
`../results-final-corpus.json` authenticates the installed corpus and the sole
Ridley dependency filename redirect. Final sizes: 272,739,363 -> 40,045,625 bytes;
149,120 cases, 6,956 continuations, 1,123 captured cases; 5,016,816 writes ->
1,550,905 spans. All metadata and full modeled WRAM comparisons passed.

The final executable/source measurement is `../results-final-loading.json`:
five warm read/SHA-256/decode samples per selected family, with medians 2.522,
3.202 and 2.496 ms. This is input loading only. No full-corpus actual-C execution,
live replay or gameplay performance claim is made.

Validation completed across the initial run and focused fixes/retries:

- All 225 non-ignored pipeline binary tests passed. Initial helper-dependent
  failures were rerun with privately built sandbox/rewrite helpers. One existing
  inventory test now canonicalizes its source path before bounded access.
- All 19 non-ignored family tests passed, including the new full-state regression;
  the overflow test now expects rejection at import, before execution.
- Affected-family CLI: 9 passed; callback actual-C: 1 passed; PLM actual-C: 1 passed.
- New Fixture CLI regression: 1 passed; existing benchmark controls: 7 passed;
  standalone trail producer tests: 2 passed. The benchmark controls use verdict
  metadata, so their temporary fixture-format edit was reverted before the passing run.
- Codec/corpus/generator/progress/native tests are included in the pipeline count.
  `git diff --check` and formatting checks passed.

Logs are retained beside the results, including failed attempts and successful
reruns. The first final-build attempt received SIGTERM; the identical retry
completed. Private debug symbols were stripped before the later checks to reduce
repeated executable hashing; code and source contracts were unchanged.

Retained actual-C runs:

- `out/fixture-format-performance/20261014T202443Z/actual-c/out/1792009502092051-397531-0/verdict.toml`
- `out/fixture-format-performance/20261014T202443Z/actual-c/out/1792009511021964-399864-0/verdict.toml`

Each run verifies two Fixture pairs against compiled C and authenticates after
cleanup. The retained reader, pinned test oracle, complete required source
snapshot and before/after WRAM remain in that directory. The exact original
format-study sources match their old hashes in `source-original/`; the final
measurement source and binaries are in `source-final/` and `bin/`.

Cleanup completed after all writers exited. The retained reader ran both
`cleanup --root /tmp/sm-binary-fixtures-j6gExW/test-runs --apply` and
`cleanup --root /tmp/sm-binary-fixtures-j6gExW/test-runs --deduplicate --apply`,
successfully with zero remaining utility candidates. It also ran both cleanup
passes over the retained `actual-c` directory, successfully with zero remaining
bytes to reclaim and 12 checked snapshots. The test's first cleanup had already
deduplicated 1,048,576 bytes and removed 23,700,637 bytes of private cache and Git
metadata, recorded in `actual-c/retention.toml`.

After saving required binaries, exact source snapshots, representative measured
inputs and both versions of the redirected captured Fixture, removed only the
owned `/tmp/sm-binary-fixtures-j6gExW` directory with `rm -r --`.
Immediately before removal, `du -sb` measured 3,613,418,392 apparent bytes;
allocated storage measured by `du -s -B1` was 3,628,359,680 bytes. Removal
succeeded and the directory no longer exists. Remaining filesystem space:
172,384,874,496 bytes. This is separate from the earlier study cleanup in
ticket 01. No worktrees were created by this change.

Both retained actual-C verdicts authenticated successfully again using the
retained reader after the private build directory was removed: each remains
`offline-verified` with two Fixture pairs. See
`../authenticate-after-cleanup-fresh.log` and
`../authenticate-after-cleanup-fork.log`. Required dependencies are the retained
reader, the `actual-c` source/oracle snapshot and its run artifacts; measured
binary inputs are retained under `fixtures/`. Cleanup logs and disk measurements
are beside this ticket's result files.

`sha256sum --check ../retained-final.sha256` passed for all 423 retained files;
the check output is `../retained-checksums-verified.log`. Implementation was
validated on `main`, based on `f39947c4a1e45c382deb5c842934c92bb28b1fe4`, and
subsequently committed at the user's request:

- `604b17b`: isolated inventory test-path fix.
- `5105f5c`: binary Fixture module, authoritative corpus, readers and tests.
- `f7dcbd0`: standalone capture producers and retained writer helper.

Ticket 03 records the follow-up evidence-cost audit. No PR is required for local
integration.

The user's subsequent direction explicitly retires historical results and
reader/source retention. Ticket 04 removed this session's archived runs,
obsolete readers, study/migration programs and checksum inventories. Paths to
those bundles above describe the initial validation and are no longer completion
dependencies. Current Rust format and actual-C verification tests remain.
