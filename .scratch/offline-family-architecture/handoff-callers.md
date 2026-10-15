# Pipeline caller migration handoff

Scope: application callers of the Offline-family verifier, source access, build
execution and supporting tools. The implementation worktree is
`/home/jim/sm-decomp-offline-callers`, branch
`architecture/offline-callers-20261015`, based on `b0ab6d7`.

## Commits and integration

- `9d526e9`: migrate pipeline callers to the owned verification and tool APIs;
  remove replaced application modules and templates. Integrated by the parent as
  `690b037`.
- `fca2497`: finish direct imports, macro-spaced references and cache result
  types; update fixture ownership documentation. Integrated as `dd1ccb3`.
- `8f7782b`: move Cargo artifact selection and source discovery into the pipeline
  application; include the batch tests' explicit catalog type imports.

Local commit `3e2482f` is the cherry-pick of the Native owner's checkpoint
`3b9e8e8`. It is an imported dependency, not an additional caller change to
cherry-pick. The caller source at final handoff is `8f7782b`.

## Ownership and interface changes

- Batch delegates registered verification and reuse to
  `OfflineFamilies::verify_registered`, while retaining aggregate reporting and
  prior elapsed seconds/status for reused results.
- Catalog consumers import `offline_verification::catalog` directly. Live uses
  the authenticated declaration and does not reopen the preceding
  `manifest.smf`.
- Existing CLI request/response protocols call verifier or authoring APIs.
  Fixture CLI uses the explicit fixture codec. Private application document
  helpers live in `pipeline::documents`.
- Source-only callers use `decomp_source`; operation entry points retain typed
  source-cache lifetime through `decomp_source::context`. Bounded byte IO,
  build cache, process execution and metrics use `tool_execution`.
- `pipeline::tool_helpers` owns helper composition and RNG build recipes;
  FixtureWorker delegates to verifier authoring.
- Cargo artifact policy is application-owned. The CLI request keeps its
  original fields. The exact evidence/decomp whitelist and existing source
  closure moved to `tool_helpers::cargo_inputs`. The lower builder accepts
  explicit paths and keeps cache keys, leases, process logs, target cleanup and
  the existing Cargo argv (`--locked --offline --release`, jobs 1).
- The Cargo wrapper owns the same metrics Run, including selection failures;
  the lower builder uses its supplied Context. Cache-key/cache-hit settings,
  process accounting and error finalization stay within that single receipt.
- Source stamps include `documents.rs`; affected-family selection covers all
  three extracted crates. Synthetic snapshots use `source_stamps::all()` and
  private dependency copies instead of symlink ancestors.

The parent owns final verifier cleanup, including obsolete uncompiled Native
recipe removal, and the Native owner owns the remaining Native caller/test
migration. No gameplay or translation assignments changed here.

## Verification

Logs are retained in [callers-verification](callers-verification/).

- `check-cargo-owner.log`: passed on `8f7782b` source before commit. Exact command:

  ```sh
  env CARGO_TARGET_DIR=/home/jim/sm-decomp-offline-callers/out/callers-target \
    CARGO_BUILD_JOBS=2 \
    TMPDIR=/home/jim/sm-decomp-offline-callers/out/callers-tmp \
    SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline \
    SM_PIPELINE_WORKERS=4 \
    cargo check --locked --offline -p tool-execution --lib --message-format=short
  ```

- `no-python-standalone.log`: the two existing no-Python/maintained-configuration
  tests passed. They were compiled directly from the maintained test source:

  ```sh
  env CARGO_MANIFEST_DIR=/home/jim/sm-decomp-offline-callers/crates/pipeline \
    rustc --edition=2024 --test crates/pipeline/tests/no_python.rs \
    -o out/callers-tools/no_python
  out/callers-tools/no_python --test-threads=1
  ```

- `format-owned.log`: formatting passed for Rust files added or changed between
  `b0ab6d7` and `fca2497`, with `rustfmt --check --edition 2024 --config
  skip_children=true`. The six files in `8f7782b` passed the same command, and
  `git diff --check` passed before commit.
- `check-initial.log` and `check-callers.log` preserve intermediate compiler
  failures. Their caller macro/import failures were corrected in `fca2497`.
- `check-native-integrated.log` is a preliminary pipeline compile after importing
  `3e2482f`; it failed on the still-in-progress Native migration. It contained no
  remaining caller-owned diagnostics. It is not a successful pipeline build.

Full pipeline compilation, CLI tests, combined Live/build checks, verifier and
Native suites, and actual-C verification are integration-owner gates. They were
deliberately not run against this intermediate combined source. No behavioral
verdict or authenticated verification result was produced by this worktree.

## Completion evidence

All processes using the caller target and temporary directories finished before
cleanup. These checks generated compiler intermediates, temporary Rust editing
tools, a standalone test executable and diagnostic logs; no finished run/batch
directories or Live evidence stores were created. The project run-cleanup
utility therefore had no applicable run directory here.

Two scoped Cargo cleanup passes were completed:

```sh
cargo clean --locked --offline \
  --target-dir /home/jim/sm-decomp-offline-callers/out/callers-target
```

- First pass: `cargo-clean.log` records 1,302 removed files, 381.5 MiB. Allocated
  target bytes measured before cleanup were 401,522,688. Identified temporary
  Rust editor/test files accounted for another 16,531,456 bytes and were removed
  individually with `rm`; the empty temporary directories were removed with
  `rmdir` (TMPDIR: 4,096 allocated bytes). Free space then was 133,784,121,344
  bytes.
- The final lower-crate check recreated the private target. Final pass:
  `cargo-clean-final.log` records 315 removed files, 87.7 MiB. Its allocated size
  before cleanup was 92,254,208 bytes. Empty `out/callers-tmp` and
  `out/callers-tools` directories (4,096 bytes each) were removed with `rmdir`.
  Absence of all three paths was checked afterward.
- Free space after final cleanup was 146,288,668,672 bytes. Other sessions share
  this filesystem, so the free-space delta is not used to measure this work's
  reclaimed bytes.

The private `deps/rustysnes` copy remains for worktree retirement (48,750,592
allocated bytes at final cleanup). It is a build dependency, not retained
verification evidence. No source snapshot or evidence path requires this
worktree to survive. The integration owner will remove the worktree after
cherry-picking the final source and evidence commits. Caller cleanup is finished;
integration tests and worktree retirement remain the integration owner's work.
