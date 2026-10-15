# Foundation extraction handoff

Status: foundation implementation and focused verification complete; ready for integration

Owner: `/root/pipeline_architecture`, delegated by `/root` for ticket 02.
Worktree: `/home/jim/sm-decomp-offline-foundations`.
Branch: `architecture/offline-foundations-20261015`.
Baseline: `a6d53c5`.
Compiling implementation checkpoint: `446dd64`.

## Implemented interfaces

- `tool-execution`: `bounded` input accounting/reading/publication;
  `build_cache` compiler preparation and cache authentication; `builds`
  execution, C/Cargo builds and `StandaloneBuildRequest`/`StandaloneArtifact`;
  process leases, digest/executable resolution, atomic TOML, temporary nonce,
  metrics and resource sampling. It depends on `pipeline-model` for telemetry.
- `decomp-source`: existing serialized `Source`, `verify_source[_until]`,
  `c_units`, `authenticate_inventory[_until]`, typed `Inventory`/`Routine`/
  `Bank` and `inventory[_until]`, `DefinitionSource::at_line` and
  `recovered_definition`. Archive and lexical helpers are private.
- `decomp_source::context::{limited, until, discovery, discovery_with_limit,
  reconcile, shared_scope}` and `SourceContext::enter` preserve the typed
  source cache with the same inherited byte/path/deadline scope. Cached reuse
  rehashes the archive and each source file. Reconciliation and new operations
  start fresh caches. Generic bounded IO contains no source identity.
- Both libraries expose `IMPLEMENTATION_SOURCES: &[(&str, &str)]` with full
  crate-relative source paths and compiled `include_str!` bytes. These include
  each manifest and all production modules, excluding test-support modules.

Pipeline retains explicit temporary imports in `build_cache.rs`, `metrics.rs`,
`resource_sampler.rs`, `builds.rs`, `family.rs`, `family_model.rs` and
`bounded_inputs.rs`. The latter composes source-aware scope entry with generic
IO. Fixture/RNG helper choices and all Strict/Native recipes remain in pipeline.
Source inventory CLI publication remains there. Translation consumers now use
typed definition facts; the installed controller identity fence is unchanged.

## Checks and paths

Cargo build/check/test commands used `--locked --offline`,
`CARGO_BUILD_JOBS=2` (the test-binary prebuild used one job),
`CARGO_TARGET_DIR=/home/jim/sm-decomp-offline-foundations/out/foundations-target`,
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, and
`SM_PIPELINE_WORKERS=4`.

- `cargo check --locked --offline -p tool-execution -p decomp-source`: passed.
- `cargo check --locked --offline -p pipeline`: passed. Imports reported unused
  during the first check were removed before the checkpoint check.
- `cargo check --locked --offline -p tool-execution -p decomp-source -p pipeline`:
  passed in 28.82 seconds, with existing pipeline dead-code warnings.
- `cargo check --locked --offline -p pipeline --tests`: all test targets passed
  in 73 seconds. One test-only document read now imports the bounded owner
  directly, after removal of its parent module's `family` import.
- `cargo test --locked --offline -p tool-execution -p decomp-source --
  --test-threads=1`: 11 execution tests and 13 source tests passed; none ignored.
  This includes actual pinned source authentication, archive refusal controls,
  lexical facts, byte/path/deadline inheritance and shared worker leases.
- `cargo test --locked --offline -p decomp-source
  pinned_source_cache_rechecks_files_and_binds_changed_revision --
  --test-threads=1`: passed after extending the existing freshness test to
  cover reconciliation's fresh cache and restoration of the enclosing cache
  and input scope. The same test checks cache reuse across worker threads and
  fresh authentication for a new operation.
- `cargo tree --locked --offline -p decomp-source --depth 2`: dependency direction
  confirmed; neither lower crate can reach pipeline or translation-execution.
- `cargo test --locked --offline -p pipeline --test build_preparation --test
  processes --no-run`: passed; prebuilt with one Cargo job in 4m 46s while the
  coordinator's ordinary baseline finished, without executing test producers.
- `cargo test --locked --offline -p pipeline --test build_preparation --test
  processes -- --test-threads=1`: passed after the coordinator reported its
  baseline complete. Build preparation: 7 passed in 153.33 seconds; the optional
  retained performance measurement was ignored. Processes: 4 passed in 66.83
  seconds. This covers real C object/header invalidation, cache corruption and
  abandoned publication, input mutation and changed dependency closure during
  cache waits, pinned helper toolchains, independent C units, sibling failure
  cancellation, SIGTERM reaping and killed lease-holder recovery.
- `cargo fmt --all`: applied to the owned extraction.
- `cargo fmt --all -- --check` and `git diff --check`: passed after the final
  source changes. Earlier formatting diagnostics are retained with the logs.

Build diagnostics are in
`.scratch/offline-family-architecture/foundations-verification/`.
`check-foundations.log` records the combined checkpoint check. Earlier check
logs are retained there as diagnostics, including successful compiler/process
tests, the source-context lifetime check, cleanup receipts and disk measurements.
Trailing empty lines in the diagnostic text captures were normalized for the
repository whitespace check; no test or cleanup result was omitted.
There were 35 distinct focused tests passing, plus the source-context test rerun
after its reconciliation assertions were added. No ignored Native/ROM proof or
retained performance measurement is claimed.

## Integration work remaining

The next owner must extend ordinary and Native source inventories, source-copy
lists and test workspace constructors with both `IMPLEMENTATION_SOURCES`
arrays; preserve the existing conservative pipeline source superset. Existing
source paths continue to point to temporary imports for this checkpoint.
The complete migration must replace those temporary imports with direct owner
imports and move recipe choices under the Offline family owner. Parent owns
the final conservative affected-family path changes and installed source lists.

The worktree has private reflink/copies of `deps/rustysnes`, `deps/snesrev-sm`
and the pinned source archive. They are build/test prerequisites, not changed
source or shared state. No production policy, queue, gameplay or main-checkout
files were modified.

## Cleanup and next command

All test writers finished with exit 0 before cleanup. Test workspaces were
removed by their existing RAII fixtures; the selected proof root was empty.
Both required passes used the just-built pipeline executable:

```sh
out/foundations-target/debug/sm-pipeline cleanup --root /home/jim/sm-decomp-offline-foundations/out/foundations-proof --apply
out/foundations-target/debug/sm-pipeline cleanup --root /home/jim/sm-decomp-offline-foundations/out/foundations-proof --deduplicate --apply
cargo clean --locked --offline --target-dir /home/jim/sm-decomp-offline-foundations/out/foundations-target
```

Both pipeline cleanup receipts report zero bytes; deduplication checked zero
snapshots. Cargo removed 5,522 build files, reporting 3.2 GiB of logical output.
The target occupied 3,094,654,976 allocated bytes before removal. The two finished
one-off Rust extraction tools and their source files were removed as identified
unsealed intermediates (9,256,960 allocated bytes), followed by the two empty
proof directories (8,192 bytes). The final filesystem observation reported
157,824,446,464 bytes available. The small diagnostics and cleanup receipts are
retained in this handoff's directory and committed with the follow-up.

No ordinary/Native/Live verdict or acceptance pointer was produced by these
focused tests, so no retained run depends on the deleted target, proof root or
pipeline executable. The existing pinned dependencies in the main checkout
remain intact. The parent integrator owns removal of this worktree and its
private dependency copies after integration; no source snapshot referenced by
retained verification evidence needs relocation from this worktree.

Next action: integrate the foundation checkpoint and follow-up, complete the
source inventories and caller migration described above, then run the full
ordinary/Native gates on the frozen integrated source. The root coordinator
will supply a fresh worktree/API checkpoint for the next non-Native caller task.
