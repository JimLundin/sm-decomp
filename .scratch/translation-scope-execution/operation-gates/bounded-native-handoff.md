# Bounded trusted input and native preparation

Owner: `/root/bounded_git` in the recovered deterministic Rust pipeline session.
Status: focused implementation, verification, retained source authentication and
run cleanup complete; root owns the combined proof and shared checkpoint.
Worktree: `/home/jim/sm-decomp-translation-execution`
Branch: `architecture/translation-scope-execution-20261013`
Latest shared checkpoint: `c37d9e8e537cbf85316858aa1a3ade1204385fed`

Owned source is `crates/pipeline/src/bounded_inputs.rs`, `inputs.rs`,
`family.rs`, `translation_native.rs`, `builds.rs`, `batch.rs`,
`source_inventory.rs`, and IO-only production changes in `translation.rs`.
The narrow `translation.rs` native fixture test helper was exposed and its test
pin/minimal headers updated by `/root/managed_admission` for the root's managed
positive proof; its patch was settled before the final rebuild. This worker
does not mutate main source, Git/index, production authority, queue or progress.
The earlier bounded Git scope is complete in `bounded-git-handoff.md` and is
not part of this scope.

## APIs and constraints

`inputs::bounded` registers the new `bounded_inputs.rs` helper. Explicit
`until(deadline_unix_ms, max_bytes, action)` scopes share aggregate usage and
inherit the earliest deadline and smallest byte cap. `limited(action)` preserves
the installed scope; its standalone cap is 3600 seconds and 1 GiB. Explicit
byte limits may not exceed 8 GiB. `reconcile(max_bytes, action)` grants trusted
receipt finalization/authentication a separate bounded 60 seconds after producer
teardown; producer launchers still require their original saved deadline.

Bounded operations include `check`, `account`, `count`, `safe`, `entries`,
`read`, `read_limit`, `text`, `digest`, `write`, `tree`, and
`authenticate_tar_source`. Files are regular, opened with `O_NOFOLLOW` and
`O_NONBLOCK`, and verified for size and metadata stability. Reads/hashes check
64 KiB chunks. Source files are at most 64 MiB, documents 32 MiB, discovered
paths 65,536 and recursive depth 32. Paths reject symlinks and literal `.` or
`..` components. Writes check regular files before truncation.

The Rust gzip/tar authenticator compares archive members directly with pinned
source, without extraction or external `tar`. It bounds compressed bytes to
32 MiB, expansion to 128 MiB and members to 4096. It checks checksums, numeric
fields, one root, unique safe paths, padding/terminators and gzip trailing data.
Links, special members, sparse/unknown PAX semantics and binary numeric fields
are refused. Source-cache identity binds archive name, revision and hash; cache
hits rehash the archive and every source file.

Source/inventory authentication, C/header/module discovery, artifact inventories,
candidate source preparation and receipt replay now use inherited bounds.
Circular/repeated module references are deduplicated. Runtime setup and native
production use the persisted gate deadline. Production errors force a rejected
verdict and zero verified counts. Strict execution reserves outputs through
`operation::reserve_bytes_until`; expired producers publish only bounded trusted
logs/diagnostics. Fallible `clean_bounded` and `names_bounded` preserve existing
lexical facts, with deadline and lexical-name extent checks. Root wired the
fallible context reader and retained this helper in the proof inputs.

## Retained verification

Evidence is in `operation-gates/bounded-native/`. Shared execution settings are
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline` and
`SM_PIPELINE_WORKERS=4`; disposable target is
`/tmp/sm-translation-bounded-native-target`.

The first focused pass had 40 passing tests and one inventory fixture failure.
The failed fixture used a literal `../..` path rejected by the new path checks;
the fixture now derives the same root through `parent()`. The ignored actual-C
fixture also now hashes its large test controller through `crate::digest`,
preserving the source helper's 64 MiB cap.

`test-build-final-03.log` rebuilt successfully in 1m 59s. The final seven normal
filter logs each passed: inputs 13, family 5, source inventory 6, catalog 4,
native 3, strict builds 2, and scope 8; total 41 tests with four explicitly
ignored proofs/catalog. `source-sha256.txt` checked all eight owned source hashes
before and after that run. Rustfmt `--check` passed all eight without mutation.

The initial final scope filter could not exercise its optional native positive
because the private worktree ROM was absent. After installing only the ignored
`rom/sm.sfc` copy, `scope-native-actual-final-01.log` exercises that exact test
with actual compiled C: one pass in 2.30s. Its private ROM SHA-256 is
`12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`;
`private-rom-sha256.txt` records the borrowed input. The test's exact temporary
project was retired by its `Drop`; no native fixture output is a retained
evidence dependency. The worktree ROM copy was removed after this scope's cleanup
and the other worker reported its dependency retired.
`private-rom-authenticate-before-removal-final-01.log` records the exact identity
immediately before removal; 3,145,728 logical bytes were retired. The main private
ROM was read only and remains intact.

`catalog-actual-final-01.log` is a retained failed optional timing probe: catalog
discovery returned 176 families in 18.973514803s, exceeding the existing ignored
test's 10-second assertion. This is a performance limitation; no speedup claim,
timing-policy change or repeat matrix was made. Normal correctness/discovery
tests passed and the diagnostic failure remains retained.

`clippy-final-02.log` completes with `-D clippy::all` and explicit exceptions
for findings outside this owned scope: `filter_next`, `collapsible_if`,
`chunks_exact_to_as_chunks`, `too_many_arguments`, `large_enum_variant`,
`needless_borrows_for_generic_args`, and `op_ref`. This Clippy log precedes the
final test-only fixture fixes and later shared integration changes; the root
owns final combined Clippy validation. The initial failing log is
retained; the owned manual-modulus finding was corrected to `is_multiple_of`.
No source formatting mutation was made after root froze actual-c02.

The existing `family-final-01.log` exercises the actual pinned archive/source:
revision `578f90b3cc49557bb70060ad033bb90b8cf8ac50`, archive SHA-256
`69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`.
The current manifest inventory fits the 32 MiB document cap: 179 files,
approximately 276 MB total; the largest manifest is 22,771,777 bytes.

The root owns the combined restricted actual-C/native supervisor proof,
shared checkpoint and production-switch decision. These focused checks do
not replace that proof or authorize production activation.

## Executed final commands and cleanup

Final rebuild used:

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
CARGO_BUILD_JOBS=1 cargo test -p pipeline --bin sm-pipeline --no-run \
  --locked --offline --target-dir /tmp/sm-translation-bounded-native-target
```

Each normal filter ran directly against the resulting unit executable with
`--nocapture --test-threads=1`. Final02 logs are retained, along with the optional
catalog timing failure and single positive native scope rerun. The executable
was `/tmp/sm-translation-bounded-native-target/debug/build/pipeline/7fa9a98880d37858/out/sm_pipeline-7fa9a98880d37858`.

After all native/test/build writers finished, both required project cleanup
passes succeeded through retained `native-gates/tools/sm-pipeline-cleanup`:

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
.scratch/translation-scope-execution/native-gates/tools/sm-pipeline-cleanup \
  cleanup --root /home/jim/sm-decomp-translation-execution/.scratch/translation-scope-execution/operation-gates/bounded-native --apply

SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
.scratch/translation-scope-execution/native-gates/tools/sm-pipeline-cleanup \
  cleanup --root /home/jim/sm-decomp-translation-execution/.scratch/translation-scope-execution/operation-gates/bounded-native --deduplicate --apply

cargo clean --target-dir /tmp/sm-translation-bounded-native-target
```

`cleanup-ordinary-final-01.toml` and `cleanup-deduplicate-final-01.toml`
record zero eligible bytes; the immutable final test and diagnostic logs remain.
`writer-observation-before-cleanup.log` records no remaining producer, only
concurrent read-only hash/size/process inspection. `build-cleanup-final-01.log`
records 2480 removed files / 2.0 GiB. Measured logical target bytes before removal
were 2,095,146,445; the target is now absent. Available disk bytes immediately
after removal were 188,244,197,376. `controller-identities-sha256.txt` records the
retired test executable and retained cleanup utility identities.

`source/` retains exactly the eight final tested owned files, 361,210 bytes,
without Git metadata or build output. Its hashes match `source-sha256.txt`,
including the settled native fixture helper before the managed proof's later
explicit-ROM helper adjustment. The current shared `translation.rs` may advance
for that test-only adjustment; this snapshot retains the exact tested subject.
`source-authenticate-after-cleanup-final-01.log` authenticated all eight worktree
sources after target retirement. The final source snapshot and compact evidence
are authenticated by the retained hash manifest after both cleanup passes.
After adding that exact snapshot and removing the private ROM, both final02
cleanup receipts again record zero eligible bytes.
`source-snapshot-authenticate-after-cleanup-final-01.log` reauthenticates all eight
snapshot files. Final available disk bytes were 188,247,060,480, recorded in
`disk-after-cleanup-final-02.log`. `retained-sha256.txt` covers final test and
failure logs, source snapshot, cleanup receipts and seals; the final
authentication log records every retained check passing.

No fixture temporary project or executable is a retained evidence dependency.
The retained cleanup utility, source snapshot, test/diagnostic logs, source seals
and cleanup receipts remain. Main pinned source/archive and private ROM are
read-only inputs to reproduce the actual tests. This active shared implementation
worktree stays with root for final integration; no integration worktree was
created or retired by this scope. Root owns final combined proofs, source
checkpoint and the separate production-switch decision.
