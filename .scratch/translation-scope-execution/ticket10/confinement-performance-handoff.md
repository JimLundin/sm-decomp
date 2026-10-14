# Confinement accounting performance

Status: claimed; provisional v6 checkpoint, final guards and cleanup pending
Owner: `/root/confinement_performance`; parent `/root` owns integration
Worktree: `/home/jim/sm-decomp-ticket10-work/confinement`
Branch: `ticket10-confinement-scale-20261014`
Baseline: `1ad7120`

Own only `crates/translation-execution/src/confinement.rs`, focused confinement
tests/benchmark source and this handoff. Other workers own native/build/operation
modules. Production authority is unchanged. No full Cargo proof was launched.

Artifacts: `/home/jim/sm-decomp-performance-20261014/confinement`.
Cargo target: `/home/jim/sm-decomp-performance-20261014/confinement-target`.
Use one Cargo job, `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline` and
`SM_PIPELINE_WORKERS=4`. The ignored `deps` symlink is development metadata,
never an approved confined input. Budgets stay fixture 5 seconds, original
command 60 seconds, native compiler 180 seconds, compact dependency build 600 seconds.

## Current source checkpoint

Current helper relative to artifact root:
`tools/sm-translation-sandbox-fd-table-v6-opt1`, SHA-256
`8f825e97e00ce534878131d6838411aee99576f7e0e2a088e1501b0c67cda0b0`.
Exact source: `source-fd-table-v6/confinement.rs`, SHA-256
`06b569915b4fb69260322c500f1e10f8c5e4d835b51ed95e6cdf1e38f0d599d4`.
Reader: `tools/confinement-runtime-fd-table-v6-opt1`.

The helper caches exact logical inode totals and observes current/in-flight
write targets and trusted logs at every growing syscall entry and exit.
Namespace/descriptor changes invalidate that inventory; unknown calls remain
conservative. Operation-scoped O_PATH pins preserve possibly unlinked targets
without retaining every unrelated writable FD. Existing tmpfs byte/inode caps,
fatal quota errors, aliases, sparse extents and descendant ownership remain.

FD-table identity follows clone flags and ptrace events. Fork/vfork receive
fresh identities; CLONE_FILES children share, including FORK events from
CLONE_FILES|SIGCHLD. Successful close_range(UNSHARE) and exec receive fresh
identities. Kernel-reported former TIDs handle nonleader exec remapping.
Explicit duplicate sources are retained only for pending writers with both the
same descriptor number and the same table identity.

Incomplete tree snapshots caused by an in-flight unlink are discarded entirely
and retried at most once per observed task, without resuming candidates.
Persistent failure still refuses with bounded path context. Missing entries
are never counted as zero.

## Focused evidence

All paths below are relative to the artifact root.

- Five descriptor controls passed v6 in 1.11 s:
  `logs/fd-table-v6-descriptor-controls.log`. Independent fork, thread
  close_range(UNSHARE), shared-table child exec, shared-thread swap, and
  CLONE_FILES|SIGCHLD cross-process swap all passed. Each shared case includes
  a bounded positive and four aggregate-overflow refusals.
- Unchanged V7 passed the same five in 1.04 s:
  `logs/fd-table-v7-descriptor-controls.log`.
- V5 failed both unshare/exec with ENOSPC in 0.20 s:
  `logs/fd-table-v5-unshare-exec-red.log`. Independent fork passed V7 in 0.12 s
  and failed V5 in 0.09 s: `logs/independent-fd-original.log` and
  `logs/independent-fd-red.log`.
- Earlier overbroad pin retention passed V7, failed the first optimization,
  and passed operation-scoped pins: `logs/retention-original.log`,
  `logs/retention-red.log`, `logs/retention-green.log`. A blocked pipe overlaps
  32 create/write/unlink/close cycles totaling 512 KiB; live output is one
  16 KiB file under an unchanged 128 KiB quota.
- Concurrent tree-observation failure reproduced on unchanged V7 and context
  v4: `logs/concurrent-baseline-stress.log`, `logs/concurrent-context-stress.log`.
  V5 complete-snapshot retries passed 32/32 stress in 19.93 s, 11 runtime
  controls in 27.17 s, and all eight original guards in 47.70 s:
  `logs/complete-concurrent-stress.log`, `logs/complete-runtime-guards.log`,
  `logs/complete-original-guards.log`. These are historical V5 checks;
  final V6 checks remain pending.

## Performance before final lifecycle fixes

Same frozen Rust producer, opt-level 1 helpers, 256 operations after separately
timed setup. Three-run medians; focused confinement results, not Cargo acceptance.

| Steady workload | Unchanged V7 | Operation-scoped v3 |
| --- | ---: | ---: |
| Regular writes, zero retained files | 0.041421 s | 0.024893 s |
| Regular writes, 256 retained files | 0.176921 s | 0.025496 s |
| Pipe pairs, zero retained files | 0.063227 s | 0.045189 s |
| Pipe pairs, 256 retained files | 0.225341 s | 0.043701 s |

Populated cases improve 6.94x and 5.16x. Sampled helper steady CPU changes
150 to 10 ms (regular) and 180 to 20 ms (pipe); kernel tick granularity is
10 ms, sampled every 2 ms. Populated setup changes 0.233301 to 0.167210 s and
0.240676 to 0.158333 s.

| Eight independent jobs | 1 worker | 2 workers | 4 workers |
| --- | ---: | ---: | ---: |
| Regular writes | 1.689545 s | 0.961046 s | 0.630628 s |
| Pipe pairs | 1.983999 s | 1.323598 s | 0.864216 s |

Four workers give 2.68x/2.30x wall throughput; sampled helper CPU increases
1.00 to 1.61 s and 1.14 to 2.10 s. Retained TOML:
`final-baseline/timings.toml`, `final-optimized/timings.toml`,
`final-parallel/parallel-timings.toml`, with matching logs.
`final-source-and-tools.sha256` identifies v3, not v6.

V7 helper SHA-256:
`fb403f0445d10cb3ce75fd2f928e6bedea27c4b9a1f5de8d445527252dcf2351`.
Measured v3 helper SHA-256:
`588136025345694fb4052e1f4b0968a45cbf0720547ef3b0b85994d785752627`.

## Next commands and completion evidence

No confinement writer remains at this checkpoint. Build
`logs/fd-table-v6-control-build.log` passed in 39.22 s. Set
`SM_CONFINEMENT_TEST_HELPER` to v6 and run the retained runtime reader with
`--test-threads=1 --nocapture`, followed by its ignored
`many_threaded_writes_and_exits_stress` with `SM_CONFINEMENT_THREAD_STRESS=32`.
Run all eight original controls with `TMPDIR` set to artifact `temporary/`.

Build with one Cargo job and the shared environment above:
`cargo --config 'profile.dev.package.translation-execution.opt-level=1' test -p translation-execution --locked --offline --test confinement --test confinement_runtime --test confinement_performance --no-run`.
Use the stated Cargo target. Coordinate final quiet measurements through root;
the current equipment correctness proof has just completed.

Final exact-source guards, measurements, source/reader manifest and cleanup are
pending. Run `/home/jim/sm-decomp-production-execution/bin/sm-pipeline cleanup
--root ROOT --apply`, then the same with `--deduplicate --apply`. Retain failed
diagnostics, readers and source snapshots; remove only identified disposable
Cargo outputs. Record reclaimed bytes, free disk and retained dependencies here.
Parent owns final worktree retirement after integration. No ticket completion,
gameplay stage or sealed Original acceptance is claimed.
