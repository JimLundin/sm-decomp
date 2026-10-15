# Native extraction handoff

Status: Native implementation and local verification/cleanup complete; root owns final proofs and worktree retirement
Owner: `/root/translation_architecture`, delegated by `/root`
Parent session: `01a13c3b-d421-7bb3-80a2-ec91045678e7`
Worktree: `/home/jim/sm-decomp-offline-native`
Branch: `architecture/offline-native-20261015`
Baseline: `446dd6418a9f91d4bd90163955b66c84a55ef9a9`

## Ownership

Own `offline-verification` Native preparation, opaque prepared state,
execution/admission, indexed operations and private Native tests. Own the
pipeline Translation scope adapter, Native proof tests and Native-specific
caller migration in `translation_execution.rs`, `translation_original.rs`,
`translation_original_prepare.rs`, `translation_original_tests.rs`,
`translation_migration.rs`, `translation_execution_tests.rs` and
`translation_managed_tests.rs`.

Root owns Cargo/lib scaffolding, ordinary verification, shared source stamps,
builds and ordinary callers. `/root/gameplay_architecture` owns common
`recipe.rs`/`recipe_commands.rs`; `/root/pipeline_architecture` owns foundation
follow-up fixes. Other worktrees, source and production claims remain intact.
No gameplay stage is claimed by this infrastructure extraction.

## Agreed interface

- `HostInputs` contains trusted compiled host-source stamps and producer hash;
  `OfflineFamilies` owns it. `inputs::with_host` scopes its use without a
  global authority cache; `inputs::required_sources` adds all three lower owners.
- Native declaration/preparation calls use that context. `PreparedNative`
  keeps all fields private, preserves current serialized records and exposes
  immutable contract/source/identity/resource summaries.
- Verification takes a target, confined environment and trusted host.
  Authentication uses retained prepared/receipt data and source paths without
  current host checks or producer launches. `AuthenticatedNative` represents
  both passed and rejected outcomes with report/reference getters.
- `ScopeAdmission` is caller-owned; pipeline retains `validate_scope` and
  ticket/assignment facts. It is called only during current provenance checks,
  never during completed replay or retained admission.
- Common recipe code uses explicit model values, with no Native-owned type or
  operation traversal. Current serde tags, compiler/Fixture limits, coverage,
  indexed attribution and retry/ownership semantics remain unchanged.

## Verification and cleanup

Use `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`,
`SM_PIPELINE_WORKERS=4`, locked/offline Cargo, isolated build output and one
Cargo build job. Root owns the final real Native proofs after integration;
the audited commands and complete recipe matrix are in the main checkout's
`native-verification-plan.md` beside this handoff.

The following implementation checks passed with locked/offline Cargo, isolated
`out/native-target`, one build job and the shared four-worker pool:

- `cargo check --locked --offline -p offline-verification`:
  `native-verification/check-library.log`.
- `cargo test --locked --offline -p offline-verification --lib --no-run`:
  `native-verification/check-library-tests.log`.
- `cargo test --locked --offline -p offline-verification --lib native::tests::
  -- --test-threads=1`: all 11 Native tests passed;
  `native-verification/native-unit-tests.log`.
- `cargo check --locked --offline -p pipeline`:
  `native-verification/check-pipeline.log`.

These are implementation checks, not frozen-source actual-C verification.
The first application test check failed on missing private C include inputs,
two caller-owned Catalog imports, and three accidental PreparedOriginal getter
uses. Recovered C was copied into this isolated worktree, the Original accesses
were fixed, and the caller imports were added locally only for checking. The
Native commit excludes `batch.rs`; the caller owner publishes that fix.
The final `-p pipeline --tests` check passed and is recorded separately in
`native-verification/check-pipeline-tests-2.log`. After the test declaration
helper was switched to the public API, all 11 Native tests passed again in
`native-verification/native-unit-tests-2.log`. The targeted formatter and
source-diff whitespace checks passed. Captured Cargo test logs preserve their
trailing blank line; Git's whole-artifact whitespace check reports those two
raw-log lines only.

No real Native producer or ignored proof has run here. Synthetic unit-test
directories are self-cleaning. Utility cleanup, local build retirement and
disposable dependency removal are complete. Parent owns integrated worktree
retirement after exact required source is retained. No private ROM was copied.

## Follow-up implementation

The initial compiling extraction is `3b9e8e8`; the Native follow-up is
`f5433bb`. The follow-up shares the full
expected compiler Program between execution and Native authentication through
the private recipe and compiler owners. Authentication recovers original
retained checkout bindings from the recorded candidate or headless input, so
an equivalent new checkout can reuse the complete old result.

Pipeline production callers and tests use public Native requests, opaque
prepared accessors, scalar operation counts and retained references. Shared
operation scopes remain in `decomp_source::context`; generic bounded IO stays
in `tool_execution::bounded`. Scope approval remains in the pipeline adapter.

Native proof snapshots use `source_stamps::all()` and copy the complete nested
port source tree. The full proof accepts optional `SM_NATIVE_FULL_SOURCE`.
The catalog freeze requires explicit `SM_NATIVE_CATALOG_SOURCE` and refuses
the compiling checkout because it authors generated PLM inputs. Recipe proofs
preserve the original fresh ten-minute preparation scope for each family;
catalog preparation stays lazy and does not retain expanded manifests.

## Retained read-only Native gate

Recipe and full proofs write `retained-authentication.toml` in their proof root.
It records only exact prepared/checkout/receipt paths and expected outcomes.
The full proof records all four cases: passing `run`, rejected
`compile-denial`, passing denial-checking `runtime-denial`, and rejected
zero-producer `reservation-denial`. The optional performance-only route records
only the passing baseline; final acceptance must leave
`SM_NATIVE_PERFORMANCE_ONLY` unset so all negative controls run.

After cleanup, and again after implementer worktree/build retirement, run the
retained test reader with:

```sh
env SM_NATIVE_AUTHENTICATION_MANIFEST=<proof-root>/retained-authentication.toml \
  <retained-test-reader> --ignored --exact \
  translation_native::tests::retained_native_proofs_authenticate_without_execution \
  --nocapture --test-threads=1
```

Run it separately for the recipe proof root and the full proof's `actual-c`
root (or its explicitly selected name). This test reads the saved paths and
calls `PreparedNative::authenticate` only. It never calls source preparation,
generation, execution, retry, cleanup or `current_checkout()`, and does not
require today's producer hash. New owner-stamped indexed formats are the only
current-reader target; old pinned readers/source snapshots remain separate.

## Completion evidence

All local test/build writers finished successfully before the final two
sequential cleanup passes. Root provided the existing project utility at
`/home/jim/sm-decomp/target/debug/sm-pipeline` for implementation-log cleanup;
its SHA-256 was
`d542f3ea5eba461bc2e0ae6420410840308846ff9f6b7943be061cdf057164c2`.
The utility supported the same cleanup schema; root builds a separate current
reader for final actual-C evidence.

```sh
/home/jim/sm-decomp/target/debug/sm-pipeline cleanup --root /home/jim/sm-decomp-offline-native/.scratch/offline-family-architecture/native-verification --apply
/home/jim/sm-decomp/target/debug/sm-pipeline cleanup --root /home/jim/sm-decomp-offline-native/.scratch/offline-family-architecture/native-verification --deduplicate --apply
env CARGO_TARGET_DIR=/home/jim/sm-decomp-offline-native/out/native-target CARGO_BUILD_JOBS=1 cargo clean --locked --offline
```

Both cleanup receipts report zero bytes and no targets; the deduplicating pass
checked zero snapshots. Cargo removed 3,797 files, reporting 1.6 GiB total, and
the isolated target directory no longer exists. Identified disposable copies
`deps/rustysnes`, `deps/snesrev-sm`, and the two `/tmp/sm-offline-native-*` Rust
editing helpers/source files were removed: 59,507,933 measured logical bytes.
`df -B1` then reported 147,531,984,896 bytes available. Other sessions were
active, so free-space changes are not used to attribute reclaimed bytes.

Small successful/failing check logs, formatter output, both cleanup receipts,
the cleanup reader hash, Cargo cleanup output, disposable-file measurements
and remaining disk measurement are retained under `native-verification/`.
No evidence from this work depends on the removed build or dependency copies.
The remaining integration action is root's removal of this clean Git worktree
and final proof authentication from its independently retained sources/reader.
