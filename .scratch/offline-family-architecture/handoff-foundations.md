# Foundation extraction handoff

Status: implementation checkpoint; focused verification and cleanup pending

Owner: `/root/pipeline_architecture`, delegated by `/root` for ticket 02.
Worktree: `/home/jim/sm-decomp-offline-foundations`.
Branch: `architecture/offline-foundations-20261015`.
Baseline: `a6d53c5`.

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

All Cargo commands use `--locked --offline`,
`CARGO_BUILD_JOBS=2`,
`CARGO_TARGET_DIR=/home/jim/sm-decomp-offline-foundations/out/foundations-target`,
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, and
`SM_PIPELINE_WORKERS=4`.

- `cargo check --locked --offline -p tool-execution -p decomp-source`: passed.
- `cargo check --locked --offline -p pipeline`: passed. Imports reported unused
  during the first check were removed before the checkpoint check.
- `cargo fmt --all`: applied to the owned extraction.
- `git diff --check`: passed at the initial source checkpoint.

Build diagnostics are in
`.scratch/offline-family-architecture/foundations-verification/`.
`check-foundations.log` records the combined checkpoint check. Earlier check
logs are retained there as diagnostics. Focused library, pipeline input/source,
process and compiler preparation suites remain pending at this checkpoint.
The root coordinator requested that compiler/actual-C suites wait for its
ordinary baseline to finish. No ignored Native/ROM proof is claimed.

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

Cleanup pending: complete focused tests, wait for writers, run scoped cleanup
and deduplication, retain logs, then remove disposable build output. The parent
integrator owns removal of this worktree after integration and any required
source/evidence relocation. No verification run artifacts have been produced
beyond Cargo build output at this checkpoint.

Next command from the assigned worktree, once the current check has exited:

```sh
env CARGO_TARGET_DIR=/home/jim/sm-decomp-offline-foundations/out/foundations-target CARGO_BUILD_JOBS=2 SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 cargo test --locked --offline -p tool-execution -p decomp-source
```
