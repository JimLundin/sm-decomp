# Combined source check

Status: resolved
Owner: session `01a13b35-079e-7151-8e5a-f759fee6bcf3`, `/root`
Branch: `architecture/perf-composition-20261014`
Frozen source: `bc5c1cf9ba2965ef6513273deb4880a93d770cc9`
Baseline: `1ce6494d7e0fdc9b8623faf3f9d9f507f4a8abb0`
Evidence: `/home/jim/sm-decomp-architecture-runs-20261014/composition`

All five candidates composed without conflicts. The temporary composition
checkout was removed after verification; the branch preserves exact source.
The five requested worktrees remain separate. This effort changed no main
source, installed controller, production policy or gameplay stage.

## Source mapping

| Candidate | Candidate commits | Composition commits |
| --- | --- | --- |
| 01 accounting | `7a481ca`, `054b969` (adopted `14c5cd5`, `370788a`) | `1df7696`, `bc5c1cf` |
| 02 scheduling | `d593bc8` | `fbc7bcb` |
| 03 builds | `3a732b1`, `0ddfae6` | `57e3f92`, `788ebc5` |
| 04 evidence | `35ef637` | `01da6ec` |
| 05 Live | `0e57ff9` | `85d67b3` |

Candidate01 additionally retains its completed cleanup handoff in `449f1dc`.
That documentation-only commit does not alter the frozen combined source.

## Verification

The affected pipeline/evidence/translation-execution Cargo packages were cleared
before compiling, avoiding stale cross-worktree binaries. The build completed
in 1m41s with existing warnings only. Commands used locked/offline Cargo,
jobs=1, shared workers=4, private temporary directories, and the released 03
target. The resulting controller hash differs from the candidate03 controller.

All **32 controls passed** on combined source:

| Interface exercised | Controls | Evidence log |
| --- | ---: | --- |
| Actual-C Fixture capacity, fresh and fork modes | 1 | `logs/fixture-capacity.log` |
| Build preparation, dependencies, toolchain, corruption, cancellation | 7 | `logs/build-preparation.log` |
| Compiled Live adapters, nesting, threads, rollback and Ridley frames | 8 | `logs/live-controls.log` |
| Evidence store/restoration and authentication | 15 | `logs/evidence-controls.log` |
| New sandbox binary through public confinement run | 1 | `logs/confinement-control.log` |

Exact test selections were `idle_fixture_chain_workers_release_shared_execution_capacity_in_both_modes`
in pipeline `family`; all ordinary `build_preparation`, `live`, `live_frames`
and evidence `store` tests; and
`independent_fork_descriptor_numbers_do_not_retain_other_process_files` in
`confinement_runtime`. Test threads=1. The ignored measurement was already run
for candidate03 and was not repeated as a composition benchmark.

The actual-C control retains two successful verdicts, covering three Fixture
pairs per execution mode. Both authenticated again after target and worktree
removal (`logs/after-cleanup-fresh.toml`, `logs/after-cleanup-fork.toml`).
The Live controls include 240 original-ROM Ridley frames against maintained
Rust behavior. They do not establish authenticated complete native Live replay.

## Binaries and installation

Both binaries were built from frozen combined source and retained under `bin/`:

- `sm-pipeline`: SHA-256
  `01371c1c92432eb6fe38814967cc5231744055aefb8d2bdce8db46453984c7b2`.
- `sm-translation-sandbox`: SHA-256
  `ae0368f4fc1aee17055720b66c47538830b5b36f9dd1e93a02f9b822876574ee`.

The sandbox is a separate target. Production selects its installed path/hash;
a controller rebuild alone does not replace it. The combined control explicitly
used this new sandbox through the test factory. Neither binary was installed
or bound to a production policy. The confinement performance comparison used
the owner's separately retained opt-level-1 V6 helper, not this debug artifact.

## Cleanup and retention

All writers stopped before both scoped utility passes:

```sh
bin/sm-pipeline cleanup --root /home/jim/sm-decomp-architecture-runs-20261014/composition --apply
bin/sm-pipeline cleanup --root /home/jim/sm-decomp-architecture-runs-20261014/composition --deduplicate --apply
```

Both succeeded with zero additional eligible bytes. Automatic Fixture cleanup
had already deduplicated **1,835,008 bytes** and removed **23,700,637 bytes** of
private Git/cache files. Identified disposable paths were then removed:

- Shared 03/composition Cargo target: **2,765,496,164 apparent bytes**.
- Empty private temporary directory: **4,096 apparent bytes**.
- Temporary composition worktree: **439,554,165 apparent bytes**.

Final available disk space was **186,173,870,080 bytes**. The observed increase
during final removal was 3,224,141,824 bytes on a shared filesystem; apparent
path sizes are not a physical disk delta. The earlier package clean is separately
recorded in `logs/package-clean.log` (1,133 files, Cargo-reported 1.9 GiB).

Both retained binary hashes and all candidate03 proof hashes passed after
removal. The exact Fixture source snapshot, successful verdicts, readers,
compiler/control logs and `completion.toml` remain. No retained verdict names
the removed composition checkout or Cargo target. `logs/changed-source.sha256`
records the verified source before retirement; the frozen Git branch can
recreate that checkout. The existing private ROM remains required to repeat
the frame control. No shared evidence-store object was removed.

## Other session handoff

The Native owner in session `01a139b9-f1a2-7d30-b65f-f0741bb5d555` has separate
strict reservation groups and Fixture waves in `builds.rs`,
`fixture_execution.rs`, and `translation_native.rs`. Those later changes are
outside this combined check. Preserve both implementations during any future
composition; candidate03's Native source-list edits only register `build_cache.rs`.
The exact five branches, binaries, validation limits and cleanup records are
ready for that owner. Production admission remains separate.
