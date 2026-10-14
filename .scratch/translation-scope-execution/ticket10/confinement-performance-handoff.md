# Confinement accounting performance

Status: ready for integration; final v6 guards, measurements and worker cleanup complete
Owner: `/root/confinement_performance`; parent `/root` owns integration
Worktree: `/home/jim/sm-decomp-ticket10-work/confinement`
Branch: `ticket10-confinement-scale-20261014`
Baseline: `1ad71205589a28f5cfe49df6539080023b0872e3`

Source checkpoint: `14c5cd507d9281534db3da94f21395357e075ba3`.
Final test checkpoint: `370788a44da0b9a5518a5c1e9bac6ceac8d8e224`.

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
Final readers including the nonleader-exec control are
`tools/confinement-runtime-final-v6-opt1` and
`tools/confinement-original-final-v6-opt1`; their source snapshots are in
`source-final-v6/`. The helper source and bytes are unchanged.

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

- Final v6 runtime controls passed 16/16 in 18.95 s, original guards 8/8 in
  32.36 s, and stress repetitions 32/32 in 10.99 s:
  `logs/fd-table-v6-runtime-guards.log`, `logs/fd-table-v6-original-guards.log`,
  `logs/fd-table-v6-concurrent-stress.log`. All ran serially with the unchanged
  budgets. The combined tool session 68422 exited successfully.
- Nonleader exec passed a bounded case and four post-exec aggregate-overflow
  refusals in 0.22 s: `logs/fd-table-v6-nonleader-exec.log`. The runtime reader
  was rebuilt in 2.10 s (`logs/fd-table-v6-exec-control-build.log`); the helper
  stayed frozen.
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
  final V6 checks are recorded above.

## Final measured performance

Same frozen Rust producer, opt-level 1 helpers, 256 operations after separately
timed setup. Three-run medians; focused confinement results, not Cargo acceptance.

| Steady workload | Unchanged V7 | Final v6 |
| --- | ---: | ---: |
| Regular writes, zero retained files | 0.040936 s | 0.026362 s |
| Regular writes, 256 retained files | 0.174422 s | 0.024090 s |
| Pipe pairs, zero retained files | 0.063728 s | 0.044225 s |
| Pipe pairs, 256 retained files | 0.195146 s | 0.044410 s |

Populated cases improve 7.24x and 4.39x. Sampled helper steady CPU changes
150 to 10 ms (regular) and 170 to 20 ms (pipe); kernel tick granularity is
10 ms, sampled every 2 ms. Populated setup changes 0.227474 to 0.155621 s and
0.241122 to 0.153789 s. These are separate setup medians, not steady IO time.

| Eight independent jobs | 1 worker | 2 workers | 4 workers |
| --- | ---: | ---: | ---: |
| Regular writes | 1.654305 s | 0.893009 s | 0.607344 s |
| Pipe pairs | 1.826053 s | 1.037842 s | 0.699265 s |

Four workers give 2.72x/2.61x wall throughput; sampled helper CPU increases
0.96 to 1.49 s and 1.03 to 1.70 s. Retained TOML:
`final-v6-baseline/timings.toml`, `final-v6-optimized/timings.toml`,
`final-v6-parallel/parallel-timings.toml`, with matching logs. The quiet
measurement window ran 2026-10-14 17:31:07–17:31:18 UTC with other producer and
build jobs held. V7 failed the scaling assertion after all 12 samples; v6
scaling and 1/2/4-worker checks passed. No budgets were increased.

Earlier v3 results remain under `final-baseline/`, `final-optimized/` and
`final-parallel/`; `final-source-and-tools.sha256` identifies that historical
version, not v6.

V7 helper SHA-256:
`fb403f0445d10cb3ce75fd2f928e6bedea27c4b9a1f5de8d445527252dcf2351`.
Historical measured v3 helper SHA-256:
`588136025345694fb4052e1f4b0968a45cbf0720547ef3b0b85994d785752627`.

## Integration commands

No confinement writer remains at this checkpoint. Build
`logs/fd-table-v6-control-build.log` passed in 39.22 s. Final checks above used
`SM_CONFINEMENT_TEST_HELPER` set to v6, the retained runtime reader with
`--test-threads=1 --nocapture`, and its ignored
`many_threaded_writes_and_exits_stress` with `SM_CONFINEMENT_THREAD_STRESS=32`.
The original reader used `TMPDIR` set to artifact `temporary/`.

For a merged-source check, use a fresh isolated Cargo target, one Cargo job and
the shared environment above:

```sh
cargo --config 'profile.dev.package.translation-execution.opt-level=1' \
  test -p translation-execution --locked --offline \
  --test confinement --test confinement_runtime -- --test-threads=1
```

The explicit stress selector is
`--ignored --exact many_threaded_writes_and_exits_stress --test-threads=1`, with
`SM_CONFINEMENT_THREAD_STRESS=32`. The retained performance reader's selectors
are `retained_files_do_not_multiply_streaming_io_cost` and
`independent_sandbox_workers`, each with `--ignored --exact` and a fresh
`SM_CONFINEMENT_PERFORMANCE_ROOT`. Set `SM_CONFINEMENT_TEST_HELPER` to the
exact retained helper path above for retained-reader checks.

Root reported its combined Native plus v6 proof passed after these measurements;
root owns that evidence and integration. The next real Cargo dependency check
must retain the existing 600-second budget. Focused confinement measurements
alone do not establish that workflow's completion.

## Completion evidence

Both required utility passes succeeded after all producers/builds stopped:

```sh
/home/jim/sm-decomp-production-execution/bin/sm-pipeline cleanup \
  --root /home/jim/sm-decomp-performance-20261014/confinement --apply
/home/jim/sm-decomp-production-execution/bin/sm-pipeline cleanup \
  --root /home/jim/sm-decomp-performance-20261014/confinement --deduplicate --apply
```

Each reported zero bytes and no recognized disposable run targets. Logs:
`logs/final-v6-cleanup.log`, `logs/final-v6-deduplicate.log`. The identified
disposable Cargo target was then removed with `rm -r --` after confirming it
was a directory, not a symlink. Its allocation was 1,281,384,448 bytes; the
observed free-space increase during removal was 1,281,368,064 bytes.
`logs/final-v6-target-removal.log` retains both measurements. Final recorded
available space is 183,735,812,096 bytes; other workers share the filesystem.
Retained evidence occupies 1,302,941,696 allocated bytes.

`final-v6-cleanup.toml` records commands, measurements and dependencies.
`final-v6-provenance.toml` binds helper/source/test commits, toolchain, exact
paths, unchanged budgets and the measurement window.
`final-v6-source-tools-and-timings.sha256` uses persistent absolute paths and
passed after cleanup (`logs/final-v6-post-cleanup-integrity.log`). Historical
v3 source entries are preserved under `source-per-operation-v3/`; running
`sha256sum -c ../final-source-and-tools.sha256` from that directory passed
(`logs/historical-v3-post-cleanup-integrity.log`). All three retained final
readers load and list their tests from the artifact directory after removal
of the Cargo target. No reader requires that target for these checks.

Retain the exact v6 helper, final runtime/original readers, fixed performance
reader, all historical diagnostic helpers/readers/source, final source
snapshots, raw TOML timings and red/green logs. Original compiler controls also
need the installed host toolchain recorded in the provenance file. No sealed
Original/live verdict or shared evidence store object was created here.
No confinement producer/build remains. Parent owns worktree retirement after
integration; it must retain these persistent source/readers and recheck the
manifests. No completed ticket, gameplay stage or Original acceptance is claimed.
